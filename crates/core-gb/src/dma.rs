pub struct Transfer {
    pub src_addr: u16,
    pub byte_idx: u8,
}

pub struct Dma {
    active: bool, // Indicates whether a DMA transfer is currently active
    source: u16,  // High byte written to $FF46 << 8
    byte_idx: u8, // 0..160
    delay: u8,    // Setup delay (1 M-cycle)
    reg_val: u8,  // Value stored in 0xFF46
}

impl Dma {
    pub fn new() -> Self {
        Self {
            active: false,
            source: 0,
            byte_idx: 0,
            delay: 0,
            reg_val: 0,
        }
    }

    pub fn write_register(&mut self, value: u8) {
        self.reg_val = value;
        self.source = (value as u16) << 8;
        self.byte_idx = 0;
        self.delay = 1; // 1 M-cycle delay before transfer starts
        self.active = true;
    }

    pub fn read_register(&self) -> u8 {
        self.reg_val
    }

    pub fn is_active(&self) -> bool {
        self.active
    }

    /// Advances DMA by 1 M-cycle (4 dots / T-cycles).
    /// Should be called inside your system's `tick_m_cycle()` loop.
    pub fn tick_m_cycle(&mut self) -> Option<Transfer> {
        if !self.active {
            return None;
        }

        // Handle 1 M-cycle setup delay
        if self.delay > 0 {
            self.delay -= 1;
            return None;
        }

        // Copy 1 byte
        let src_addr = self.source + self.byte_idx as u16;
        let transfer = Transfer {
            src_addr,
            byte_idx: self.byte_idx,
        };

        self.byte_idx += 1;
        if self.byte_idx >= 160 {
            self.active = false; // DMA finished
        }

        Some(transfer)
    }
}
