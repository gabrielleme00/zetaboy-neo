mod apu;
mod audio_output;
mod bus;
mod cart;
mod cpu;
mod dma;
mod joypad;
mod ppu;
mod timer;

pub use crate::apu::Apu;
pub use crate::audio_output::AudioOutput;
pub use crate::bus::Bus;
pub use crate::cart::Cart;
pub use crate::cpu::Cpu;
pub use crate::dma::Dma;
pub use crate::joypad::{Joypad, buttons};
pub use crate::ppu::{Ppu, SCREEN_HEIGHT, SCREEN_SIZE, SCREEN_WIDTH};
pub use crate::timer::Timer;

use core_common::{AudioSink, ControllerState, SystemCore};

const T_CYCLES_PER_M_CYCLE: u32 = 4;
const T_CYCLES_PER_FRAME: u32 = 70_224;

pub enum Interrupt {
    VBlank = 0x01,
    LcdStat = 0x02,
    Timer = 0x04,
    Serial = 0x08,
    Joypad = 0x10,
}

pub struct GameBoy {
    cpu: Cpu,
    bus: Bus,
    audio_sink: Option<Box<dyn AudioSink>>,
    audio_buffer: Vec<f32>,
}

impl GameBoy {
    pub fn new(rom: Vec<u8>) -> Self {
        GameBoy {
            cpu: Cpu::new(),
            bus: Bus::new(rom),
            audio_sink: None,
            audio_buffer: Vec::new(),
        }
    }

    pub fn new_with_audio_sink(rom: Vec<u8>, sink: Box<dyn AudioSink>) -> Self {
        let mut gb = Self::new(rom);
        gb.set_audio_sink(sink);
        gb
    }

    pub fn set_post_boot_state(&mut self) {
        self.cpu.set_post_boot_state();
        self.bus.set_post_boot_state();
    }

    pub fn set_audio_sink(&mut self, sink: Box<dyn AudioSink>) {
        self.bus.audio_output.set_audio_sample_rate(sink.sample_rate());
        self.audio_sink = Some(sink);
    }
}

impl SystemCore for GameBoy {
    fn step(&mut self) {
        self.cpu.step(&mut self.bus);
    }

    fn step_frame(&mut self) {
        let target_cycles = T_CYCLES_PER_FRAME;
        let mut frame_cycles = 0;

        while frame_cycles < target_cycles {
            let m_cycles = self.cpu.step(&mut self.bus);
            frame_cycles += m_cycles * T_CYCLES_PER_M_CYCLE;
        }

        if let Some(sink) = &mut self.audio_sink {
            self.bus.audio_output.drain_audio(&mut self.audio_buffer);
            sink.push_samples(&self.audio_buffer);
        }
    }

    fn audio_ready_for_frame(&self) -> bool {
        match &self.audio_sink {
            Some(s) => s.space_available() >= (s.sample_rate() as usize / 20) * s.channels(),
            None => true,
        }
    }

    fn frame_buffer(&self) -> &[u32] {
        self.bus.frame_buffer()
    }

    fn set_input(&mut self, input: &ControllerState) {
        use joypad::buttons::*;
        self.bus.set_joypad_button(BTN_UP, input.up);
        self.bus.set_joypad_button(BTN_DOWN, input.down);
        self.bus.set_joypad_button(BTN_LEFT, input.left);
        self.bus.set_joypad_button(BTN_RIGHT, input.right);
        self.bus.set_joypad_button(BTN_A, input.a);
        self.bus.set_joypad_button(BTN_B, input.b);
        self.bus.set_joypad_button(BTN_START, input.start);
        self.bus.set_joypad_button(BTN_SELECT, input.select);
    }
}
