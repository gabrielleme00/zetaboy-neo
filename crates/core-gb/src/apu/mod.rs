mod channel;
mod utils;

// Modulation units
mod duty_cycle;
mod envelope;
mod length_counter;
mod pulse_phase_timer;
mod sweep;

use channel::{NoiseChannel, PulseChannel, WaveChannel};
use utils::{
    apply_volume_reduction, get_panned_output, mix_samples,
    panning_indexes::*,
};

#[derive(Clone)]
pub struct Apu {
    enabled: bool,
    frame_sequencer_step: u8,
    frame_sequencer_counter: u16,
    master_volume: u8,
    sound_panning: u8,
    channel_1: PulseChannel,
    channel_2: PulseChannel,
    channel_3: WaveChannel,
    channel_4: NoiseChannel,
    tick_divider: bool,
}

impl Apu {
    pub fn new() -> Self {
        let mut apu = Self {
            enabled: false,
            frame_sequencer_step: 0,
            frame_sequencer_counter: 0,
            master_volume: 0,
            sound_panning: 0,
            channel_1: PulseChannel::new_with_sweep(),
            channel_2: PulseChannel::new(),
            channel_3: WaveChannel::new(),
            channel_4: NoiseChannel::new(),
            tick_divider: false,
        };
        
        apu.reg_write_byte(0xFF10, 0x80);
        apu.reg_write_byte(0xFF11, 0xBF);
        apu.reg_write_byte(0xFF12, 0xF3);
        apu.reg_write_byte(0xFF14, 0xBF);
        apu.reg_write_byte(0xFF16, 0x3F);
        apu.reg_write_byte(0xFF19, 0xBF);
        apu.reg_write_byte(0xFF1A, 0x7F);
        apu.reg_write_byte(0xFF1B, 0xFF);
        apu.reg_write_byte(0xFF1C, 0x9F);
        apu.reg_write_byte(0xFF1E, 0xBF);
        apu.reg_write_byte(0xFF20, 0xFF);
        apu.reg_write_byte(0xFF23, 0xBF);
        apu.reg_write_byte(0xFF24, 0x77);
        apu.reg_write_byte(0xFF25, 0xF3);
        apu.reg_write_byte(0xFF26, 0xF1);
        
        apu
    }

    pub fn sample_stereo(&self) -> (f32, f32) {
        if !self.enabled {
            return (0.0, 0.0);
        }

        // Calculate the mixed output of all channels
        let (left, right) = {
            let ch1 = self.channel_1.analog_output();
            let ch2 = self.channel_2.analog_output();
            let ch3 = self.channel_3.analog_output();
            let ch4 = self.channel_4.analog_output();

            let left_sample = self.generate_left_sample(ch1, ch2, ch3, ch4);
            let right_sample = self.generate_right_sample(ch1, ch2, ch3, ch4);

            (left_sample, right_sample)
        };

        (left, right)
    }

    pub fn sample_mono(&self) -> f32 {
        let (left, right) = self.sample_stereo();
        (left + right) / 2.0
    }

    fn generate_left_sample(
        &self,
        ch1_dac_out: f32,
        ch2_dac_out: f32,
        ch3_dac_out: f32,
        ch4_dac_out: f32,
    ) -> f32 {
        let left_master_volume = (self.master_volume & 0b01110000) >> 4;
        let pan = self.sound_panning;
        let ch1_panned = get_panned_output(pan, CH1_PAN_L, ch1_dac_out);
        let ch2_panned = get_panned_output(pan, CH2_PAN_L, ch2_dac_out);
        let ch3_panned = get_panned_output(pan, CH3_PAN_L, ch3_dac_out);
        let ch4_panned = get_panned_output(pan, CH4_PAN_L, ch4_dac_out);
        let mix = mix_samples(ch1_panned, ch2_panned, ch3_panned, ch4_panned);
        apply_volume_reduction(mix, left_master_volume)
    }

    fn generate_right_sample(
        &self,
        ch1_dac_out: f32,
        ch2_dac_out: f32,
        ch3_dac_out: f32,
        ch4_dac_out: f32,
    ) -> f32 {
        let right_master_volume = self.master_volume & 0b111;
        let pan = self.sound_panning;
        let ch1_panned = get_panned_output(pan, CH1_PAN_R, ch1_dac_out);
        let ch2_panned = get_panned_output(pan, CH2_PAN_R, ch2_dac_out);
        let ch3_panned = get_panned_output(pan, CH3_PAN_R, ch3_dac_out);
        let ch4_panned = get_panned_output(pan, CH4_PAN_R, ch4_dac_out);
        let mix = mix_samples(ch1_panned, ch2_panned, ch3_panned, ch4_panned);
        apply_volume_reduction(mix, right_master_volume)
    }

    pub fn reg_read_byte(&self, addr: u16) -> u8 {
        match addr {
            0xFF10 => self.channel_1.get_sweep_settings() | 0x80,
            0xFF11 => self.channel_1.get_length_settings() | 0x3F,
            0xFF12 => self.channel_1.get_envelope_settings(),
            0xFF14 => self.channel_1.get_period_high_control_settings() | 0xBF,
            0xFF16 => self.channel_2.get_length_settings() | 0x3F,
            0xFF17 => self.channel_2.get_envelope_settings(),
            0xFF19 => self.channel_2.get_period_high_control_settings() | 0xBF,
            0xFF1A => self.channel_3.get_dac_settings(),
            0xFF1C => self.channel_3.get_volume_settings(),
            0xFF1E => self.channel_3.get_period_high_control_settings() | 0xBF,
            0xFF21 => self.channel_4.get_envelope_settings(),
            0xFF22 => self.channel_4.get_frequency_settings(),
            0xFF23 => self.channel_4.get_control_settings() | 0xBF,
            0xFF24 => self.master_volume,
            0xFF25 => self.sound_panning,
            0xFF26 => self.get_master_control(),
            0xFF30..=0xFF3F => self.channel_3.read_wave_ram((addr - 0xFF30) as u8),
            _ if addr < 0xFF10 || addr > 0xFF3F => panic!("Invalid APU address: 0x{:04X}", addr),
            _ => 0xFF,
        }
    }

    pub fn reg_write_byte(&mut self, addr: u16, value: u8) {
        if !self.enabled && addr != 0xFF26 {
            return;
        }

        match addr {
            0xFF10 => self.channel_1.set_sweep_settings(value),
            0xFF11 => self.channel_1.set_length_settings(value),
            0xFF12 => self.channel_1.set_envelope_settings(value),
            0xFF13 => self.channel_1.set_period_low_settings(value),
            0xFF14 => self.channel_1.set_period_high_control_settings(value),
            0xFF16 => self.channel_2.set_length_settings(value),
            0xFF17 => self.channel_2.set_envelope_settings(value),
            0xFF18 => self.channel_2.set_period_low_settings(value),
            0xFF19 => self.channel_2.set_period_high_control_settings(value),
            0xFF1A => self.channel_3.set_dac_settings(value),
            0xFF1B => self.channel_3.set_length_settings(value),
            0xFF1C => self.channel_3.set_volume_settings(value),
            0xFF1D => self.channel_3.set_period_low_settings(value),
            0xFF1E => self.channel_3.set_period_high_control_settings(value),
            0xFF20 => self.channel_4.set_length_settings(value),
            0xFF21 => self.channel_4.set_envelope_settings(value),
            0xFF22 => self.channel_4.set_frequency_settings(value),
            0xFF23 => self.channel_4.set_control_settings(value),
            0xFF24 => self.master_volume = value,
            0xFF25 => self.sound_panning = value,
            0xFF26 => self.set_master_control(value),
            0xFF30..=0xFF3F => self.channel_3.write_wave_ram((addr - 0xFF30) as u8, value),
            _ => {}
        }
    }

    /// Advances the APU by one M-cycle (4 T-cycles).
    pub fn tick_m_cycle(&mut self) {
        for _ in 0..4 {
            self.tick_t_cycle();
        }
    }

    pub fn tick_t_cycle(&mut self) {
        // APU runs at 2MHz, but it's called at 4MHz (every T-cycle)
        // So we only actually tick the APU every other call
        self.tick_divider = !self.tick_divider;
        if !self.tick_divider {
            return;
        }

        if !self.enabled {
            return;
        }

        self.channel_1.tick();
        self.channel_2.tick();
        self.channel_3.tick();
        self.channel_4.tick();

        // Clock frame sequencer at 512 Hz (every 4096 APU ticks = 8192 T-cycles)
        self.frame_sequencer_counter += 1;
        if self.frame_sequencer_counter >= 4096 {
            self.frame_sequencer_counter = 0;
            self.clock_frame_sequencer();
        }
    }

    fn clock_frame_sequencer(&mut self) {
        if matches!(self.frame_sequencer_step, 0 | 2 | 4 | 6) {
            self.channel_1.clock_length_counter();
            self.channel_2.clock_length_counter();
            self.channel_3.clock_length_counter();
            self.channel_4.clock_length_counter();
        }

        if matches!(self.frame_sequencer_step, 2 | 6) {
            self.channel_1.clock_sweep();
        }

        if matches!(self.frame_sequencer_step, 7) {
            self.channel_1.clock_envelope();
            self.channel_2.clock_envelope();
            self.channel_4.clock_envelope();
        }

        self.frame_sequencer_step = (self.frame_sequencer_step + 1) % 8;
    }

    fn get_master_control(&self) -> u8 {
        let mut value = 0;
        if self.enabled {
            value |= 1 << 7;
        }
        if self.channel_4.channel_enabled {
            value |= 1 << 3;
        }
        if self.channel_3.channel_enabled {
            value |= 1 << 2;
        }
        if self.channel_2.channel_enabled {
            value |= 1 << 1;
        }
        if self.channel_1.channel_enabled {
            value |= 1 << 0;
        }
        value
    }

    fn set_master_control(&mut self, value: u8) {
        let was_enabled = self.enabled;
        self.enabled = (value & (1 << 7)) != 0;
        
        if was_enabled && !self.enabled {
            self.channel_1 = PulseChannel::new_with_sweep();
            self.channel_2 = PulseChannel::new();
            self.channel_3 = WaveChannel::new();
            self.channel_4 = NoiseChannel::new();
            self.master_volume = 0;
            self.sound_panning = 0;
            self.frame_sequencer_step = 0;
            self.frame_sequencer_counter = 0;
        }
    }
}
