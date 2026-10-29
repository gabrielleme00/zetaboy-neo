use super::{Apu, Cart, Dma, Interrupt, Joypad, Ppu, Timer, AudioOutput};

pub struct Bus {
    m_cycles: u32,
    cart: Cart,
    ppu: Ppu,
    apu: Apu,
    wram: Vec<u8>,
    joypad: Joypad,
    timer: Timer,
    io: Vec<u8>,
    hram: Vec<u8>,
    ie: u8,
    dma: Dma,
    pub audio_output: AudioOutput,
}

impl Bus {
    pub fn new(rom: Vec<u8>) -> Self {
        Self {
            cart: Cart::new(rom),
            ppu: Ppu::new(),
            apu: Apu::new(),
            wram: vec![0; 0x2000],
            timer: Timer::new(),
            m_cycles: 0,
            joypad: Joypad::new(),
            io: vec![0; 0x80],
            hram: vec![0; 0x7F],
            ie: 0,
            dma: Dma::new(),
            audio_output: AudioOutput::new(),
        }
    }

    /// Sets the bus to the state it should be in immediately after the boot ROM has run.
    pub fn set_post_boot_state(&mut self) {
        self.raw_write(0xFF40, 0x91); // Screen ON, BG ON, Tile Map 0x9800
        self.raw_write(0xFF47, 0xFC); // Set BG palette data
        self.raw_write(0xFF50, 0x01); // Disable the boot ROM
    }

    /// Advances the bus by one M-cycle.
    pub fn tick_m_cycle(&mut self) {
        self.m_cycles = self.m_cycles.wrapping_add(1);

        let ppu_irq = self.ppu.tick_m_cycle();
        if ppu_irq.vblank {
            self.raise_interrupt(Interrupt::VBlank);
        }
        if ppu_irq.stat {
            self.raise_interrupt(Interrupt::LcdStat);
        }

        if self.timer.tick_m_cycle() {
            self.raise_interrupt(Interrupt::Timer);
        }

        self.apu.tick_m_cycle();

        let stereo_sample = self.apu.sample_stereo();
        self.audio_output.sample_audio(stereo_sample);

        if let Some(transfer) = self.dma.tick_m_cycle() {
            self.handle_dma_transfer(transfer.src_addr, transfer.byte_idx);
        }
    }

    /// Returns the number of M-cycles that have elapsed.
    pub fn cycles(&self) -> u32 {
        self.m_cycles
    }

    /// Returns the pending interrupts (IF & IE & 0x1F).
    pub fn pending_interrupts(&self) -> u8 {
        self.int_flags() & self.int_enable() & 0x1F
    }

    /// Returns the current interrupt flags (IF) value.
    pub fn int_flags(&self) -> u8 {
        self.io[0x0F]
    }

    /// Returns the current interrupt enable (IE) value.
    pub fn int_enable(&self) -> u8 {
        self.ie
    }

    /// Sets the interrupt flags (IF) value.
    pub fn set_int_flags(&mut self, value: u8) {
        self.io[0x0F] = value;
    }

    /// Raises the specified interrupt by setting its corresponding flag
    /// in the interrupt flags (IF) register.
    fn raise_interrupt(&mut self, flag: Interrupt) {
        let current_flags = self.int_flags();
        self.set_int_flags(current_flags | flag as u8);
    }

    /// Returns a reference to the current frame buffer of the PPU.
    pub fn frame_buffer(&self) -> &[u32] {
        self.ppu.frame_buffer()
    }

    /// Returns `true` if any joypad button is currently pressed.
    pub fn any_button_pressed(&self) -> bool {
        self.io[0x00] & 0x0F != 0x0F
    }

    /// Sets the state of the specified joypad button.
    /// If the button is pressed, it will raise a joypad interrupt.
    pub fn set_joypad_button(&mut self, button: u8, pressed: bool) {
        if self.joypad.set_button(button, pressed) {
            self.raise_interrupt(Interrupt::Joypad);
        }
    }
}

/// Bus read/write operations with automatic ticking of M-cycles.
impl Bus {
    pub fn read_byte(&mut self, address: u16) -> u8 {
        let val = self.raw_read(address);
        self.tick_m_cycle();
        val
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.raw_write(address, value);
        self.tick_m_cycle();
    }

    pub fn read_word(&mut self, address: u16) -> u16 {
        let low = self.read_byte(address) as u16;
        let high = self.read_byte(address.wrapping_add(1)) as u16;
        (high << 8) | low
    }

    pub fn write_word(&mut self, address: u16, value: u16) {
        let low = (value & 0xFF) as u8;
        let high = (value >> 8) as u8;
        self.write_byte(address, low);
        self.write_byte(address.wrapping_add(1), high);
    }

    pub fn raw_read(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x7FFF => self.cart.read_byte(address),
            0x8000..=0x9FFF => self.ppu.vram_read_byte(address - 0x8000),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize],
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize],
            0xFE00..=0xFE9F => self.ppu.oam_read_byte(address - 0xFE00),
            0xFF00..=0xFF7F => match address {
                0xFF00 => self.joypad.read(),
                0xFF04..=0xFF07 => self.timer.read_byte(address),
                0xFF10..=0xFF3F => self.apu.reg_read_byte(address),
                0xFF40..=0xFF45 => self.ppu.reg_read_byte(address),
                0xFF46 => self.dma.read_register(),
                0xFF47..=0xFF4B => self.ppu.reg_read_byte(address),
                _ => self.io[(address - 0xFF00) as usize],
            },
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize],
            0xFFFF => self.ie,
            _ => 0xFF,
        }
    }

    pub fn raw_write(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x7FFF => self.cart.write_byte(address, value),
            0x8000..=0x9FFF => self.ppu.vram_write_byte(address - 0x8000, value),
            0xC000..=0xDFFF => self.wram[(address - 0xC000) as usize] = value,
            0xE000..=0xFDFF => self.wram[(address - 0xE000) as usize] = value,
            0xFE00..=0xFE9F => self.ppu.oam_write_byte(address - 0xFE00, value),
            0xFF00..=0xFF7F => match address {
                0xFF00 => self.joypad.write(value),
                0xFF04..=0xFF07 => self.timer.write_byte(address, value),
                0xFF10..=0xFF3F => self.apu.reg_write_byte(address, value),
                0xFF40..=0xFF45 => self.ppu.reg_write_byte(address, value),
                0xFF46 => self.dma.write_register(value),
                0xFF47..=0xFF4B => self.ppu.reg_write_byte(address, value),
                _ => self.io[(address - 0xFF00) as usize] = value,
            },
            0xFF80..=0xFFFE => self.hram[(address - 0xFF80) as usize] = value,
            0xFFFF => self.ie = value,
            _ => {}
        }
    }

    pub fn handle_dma_transfer(&mut self, src_addr: u16, byte_idx: u8) {
        // Adjust the source address for the ECHO RAM region (0xE000-0xFDFF)
        // to point to the corresponding WRAM address
        let src_addr = if src_addr >= 0xE000 {
            src_addr - 0x2000
        } else {
            src_addr
        };

        let data = match src_addr {
            // Bypass the VRAM mode-3 lock for the DMA source read
            0x8000..=0x9FFF => self.ppu.vram_dma_read(src_addr - 0x8000),
            _ => self.raw_read(src_addr),
        };
        self.ppu.oam_dma_write(byte_idx, data);
    }
}
