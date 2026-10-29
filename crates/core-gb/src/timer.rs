pub struct Timer {
    /// Internal 16-bit counter; 0xFF04 exposes the upper 8 bits
    div: u16,               
    /// TIMA - Timer counter (0xFF05)
    tima: u8,               
    /// TMA - Timer modulo (0xFF06)
    tma: u8,                
    /// TAC - Timer control (0xFF07)
    tac: u8,                
    /// Previous state of (enabled && selected_bit set)
    prev_edge: bool,        
    /// True during the 1 M-cycle delay after TIMA overflows
    overflow_pending: bool, 
}

impl Timer {
    pub fn new() -> Self {
        Timer {
            div: 0,
            tima: 0,
            tma: 0,
            tac: 0,
            prev_edge: false,
            overflow_pending: false,
        }
    }

    /// Advances the timer by 1 M-cycle (4 T-cycles).
    /// Returns true if a Timer interrupt should be requested this cycle.
    pub fn tick_m_cycle(&mut self) -> bool {
        let mut request_interrupt = false;

        if self.overflow_pending {
            self.overflow_pending = false;
            self.tima = self.tma;
            request_interrupt = true;
        }

        self.div = self.div.wrapping_add(4);
        self.update_edge();

        request_interrupt
    }

    /// Re-evaluates the falling-edge detector against the current DIV/TAC
    /// state and increments TIMA if a falling edge just occurred.
    fn update_edge(&mut self) {
        let edge = self.selected_edge();
        if self.prev_edge && !edge {
            self.increment_tima();
        }
        self.prev_edge = edge;
    }

    fn increment_tima(&mut self) {
        let (result, overflowed) = self.tima.overflowing_add(1);
        self.tima = result;
        if overflowed {
            self.overflow_pending = true;
        }
    }

    /// Current level of the internal (enable AND selected-bit) signal.
    fn selected_edge(&self) -> bool {
        (self.tac & 0x04 != 0) && (self.div & self.selected_bit()) != 0
    }

    /// Returns the bit position in DIV that is used for the selected timer frequency.
    fn selected_bit(&self) -> u16 {
        match self.tac & 0x03 {
            0 => 1 << 9, // 4096 Hz
            1 => 1 << 3, // 262144 Hz
            2 => 1 << 5, // 65536 Hz
            3 => 1 << 7, // 16384 Hz
            _ => unreachable!(),
        }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF04 => (self.div >> 8) as u8,
            0xFF05 => self.tima,
            0xFF06 => self.tma,
            0xFF07 => self.tac | 0xF8, // top 5 bits always read as 1
            _ => 0xFF,
        }
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF04 => {
                // Any write resets the internal counter to 0. If the
                // selected bit was 1, this is itself a falling edge.
                self.div = 0;
                self.update_edge();
            }
            0xFF05 => {
                // A write during the overflow-delay window cancels the
                // pending TMA reload and interrupt.
                self.overflow_pending = false;
                self.tima = value;
            }
            0xFF06 => {
                self.tma = value;
            }
            0xFF07 => {
                // Changing TAC (enable or clock select) can also flip the
                // selected-bit signal and cause a spurious increment.
                self.tac = value & 0x07;
                self.update_edge();
            }
            _ => {}
        }
    }
}
