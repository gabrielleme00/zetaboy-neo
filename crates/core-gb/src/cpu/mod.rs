mod execute;

use crate::bus::Bus;

/// Represents the state of the Interrupt Master Enable (IME) flag in the CPU.
#[derive(Default)]
pub enum ImeState {
    #[default]
    Disabled,
    SetToEnable,
    AboutToEnable,
    Enabled,
}

impl ImeState {
    /// Updates the current IME state, transitioning it to the next
    /// logical state if applicable.
    fn update(&mut self) {
        use ImeState::*;
        match self {
            SetToEnable => *self = AboutToEnable,
            AboutToEnable => *self = Enabled,
            _ => {}
        }
    }

    /// Immediately disables the IME flag.
    fn disable(&mut self) {
        *self = ImeState::Disabled;
    }

    /// Initiates the process to enable the IME flag.
    fn enable(&mut self) {
        use ImeState::*;
        match self {
            Enabled | SetToEnable | AboutToEnable => {}
            _ => *self = SetToEnable,
        }
    }

    fn enable_now(&mut self) {
        *self = ImeState::Enabled;
    }

    /// Checks if the IME flag is currently enabled.
    fn is_enabled(&self) -> bool {
        matches!(self, ImeState::Enabled)
    }
}

#[derive(Default)]
pub struct Cpu {
    // 8-bit registers
    a: u8,
    b: u8,
    c: u8,
    d: u8,
    e: u8,
    h: u8,
    l: u8,
    // Flag register bits
    flag_z: bool,
    flag_n: bool,
    flag_h: bool,
    flag_c: bool,
    // 16-bit registers
    sp: u16,
    pc: u16,
    // Interrupt Master Enable
    ime: ImeState,
    // CPU halted state
    halted: bool,
    stopped: bool,
}

impl Cpu {
    pub fn new() -> Self {
        Cpu::default()
    }

    /// Resets the CPU to its initial state (DMG)
    pub fn set_post_boot_state(&mut self) {
        self.set_af(0x01B0);
        self.set_bc(0x0013);
        self.set_de(0x00D8);
        self.set_hl(0x014D);
        self.sp = 0xFFFE;
        self.pc = 0x0100;
        self.ime = ImeState::Disabled;
        self.halted = false;
        self.stopped = false;
    }

    pub fn step(&mut self, bus: &mut Bus) -> u32 {
        let start = bus.cycles();

        if self.stopped {
            if bus.any_button_pressed() {
                self.stopped = false;
            } else {
                return bus.cycles().wrapping_sub(start);
            }
        }

        self.ime.update();

        let pending = bus.pending_interrupts();
        if pending != 0 {
            self.halted = false;
            if self.ime.is_enabled() {
                self.ime.disable();
                let bit = pending & pending.wrapping_neg(); // lowest pending bit
                bus.set_int_flags(bus.int_flags() & !bit);
                self.push(self.pc, bus); // 2 M-cycles
                self.pc = 0x40 + 8 * bit.trailing_zeros() as u16;
                for _ in 0..3 {
                    bus.tick_m_cycle();
                } // dispatch = 5 M-cycles total
            }
        }

        if self.halted {
            bus.tick_m_cycle();
            return bus.cycles().wrapping_sub(start);
        }
        self.execute(bus);
        bus.cycles().wrapping_sub(start)
    }

    /// Fetches the next byte from memory and increments the program counter.
    pub fn fetch_byte(&mut self, bus: &mut Bus) -> u8 {
        let byte = bus.read_byte(self.pc);
        self.pc = self.pc.wrapping_add(1);
        byte
    }

    /// Fetches the next word (16 bits) from memory and increments the program
    /// counter accordingly.
    pub fn fetch_word(&mut self, bus: &mut Bus) -> u16 {
        let low = self.fetch_byte(bus) as u16;
        let high = self.fetch_byte(bus) as u16;
        (high << 8) | low
    }

    pub fn fetch_bc(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.bc();
        bus.read_byte(addr)
    }

    pub fn fetch_de(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.de();
        bus.read_byte(addr)
    }

    /// Fetches the value at the memory location pointed to by the HL register pair
    /// without modifying HL.
    pub fn fetch_hl(&mut self, bus: &mut Bus) -> u8 {
        let addr = self.hl();
        bus.read_byte(addr)
    }

    /// Fetches the value at the memory location pointed to by the HL register pair
    /// and then increments HL.
    pub fn fetch_hl_inc(&mut self, bus: &mut Bus) -> u8 {
        let value = self.fetch_hl(bus);
        let hl = self.hl().wrapping_add(1);
        self.set_hl(hl);
        value
    }

    /// Fetches the value at the memory location pointed to by the HL register pair
    /// and then decrements HL.
    pub fn fetch_hl_dec(&mut self, bus: &mut Bus) -> u8 {
        let value = self.fetch_hl(bus);
        let hl = self.hl().wrapping_sub(1);
        self.set_hl(hl);
        value
    }

    /// Pushes a word (16 bits) onto the stack and decrements the stack pointer accordingly.
    fn push(&mut self, value: u16, bus: &mut Bus) {
        self.sp = self.sp.wrapping_sub(2);
        bus.write_word(self.sp, value);
    }

    /// Pops a word (16 bits) from the stack and increments the stack pointer accordingly.
    fn pop(&mut self, bus: &mut Bus) -> u16 {
        let value = bus.read_word(self.sp);
        self.sp = self.sp.wrapping_add(2);
        value
    }

    /// Sets the AF register pair and updates the individual flag bits accordingly.
    fn set_af(&mut self, value: u16) {
        self.a = (value >> 8) as u8;
        self.flag_z = (value & 0x80) != 0;
        self.flag_n = (value & 0x40) != 0;
        self.flag_h = (value & 0x20) != 0;
        self.flag_c = (value & 0x10) != 0;
    }

    /// Sets the BC register pair.
    fn set_bc(&mut self, value: u16) {
        self.b = (value >> 8) as u8;
        self.c = value as u8;
    }

    /// Sets the value at the memory location pointed to by the BC register pair.
    fn write_at_bc(&mut self, value: u8, bus: &mut Bus) {
        let addr = self.bc();
        bus.write_byte(addr, value);
    }

    /// Sets the DE register pair.
    fn set_de(&mut self, value: u16) {
        self.d = (value >> 8) as u8;
        self.e = value as u8;
    }

    /// Sets the value at the memory location pointed to by the DE register pair.
    fn write_at_de(&mut self, value: u8, bus: &mut Bus) {
        let addr = self.de();
        bus.write_byte(addr, value);
    }

    /// Sets the HL register pair.
    fn set_hl(&mut self, value: u16) {
        self.h = (value >> 8) as u8;
        self.l = value as u8;
    }

    /// Sets the value at the memory location pointed to by the HL register pair.
    fn write_at_hl(&mut self, value: u8, bus: &mut Bus) {
        let addr = self.hl();
        bus.write_byte(addr, value);
    }

    /// Sets the value at the memory location pointed to by the HL register pair
    /// and then increments HL.
    fn write_at_hl_inc(&mut self, value: u8, bus: &mut Bus) {
        self.write_at_hl(value, bus);
        let hl = self.hl().wrapping_add(1);
        self.set_hl(hl);
    }

    /// Sets the value at the memory location pointed to by the HL register pair
    /// and then decrements HL.
    fn write_at_hl_dec(&mut self, value: u8, bus: &mut Bus) {
        self.write_at_hl(value, bus);
        let hl = self.hl().wrapping_sub(1);
        self.set_hl(hl);
    }

    pub fn f(&self) -> u8 {
        (if self.flag_z { 0x80 } else { 0 })
            | (if self.flag_n { 0x40 } else { 0 })
            | (if self.flag_h { 0x20 } else { 0 })
            | (if self.flag_c { 0x10 } else { 0 })
    }

    pub fn af(&self) -> u16 {
        ((self.a as u16) << 8) | (self.f() as u16)
    }

    /// Gets the value of the HL register pair.
    pub fn hl(&self) -> u16 {
        ((self.h as u16) << 8) | (self.l as u16)
    }

    /// Gets the value of the BC register pair.
    pub fn bc(&self) -> u16 {
        ((self.b as u16) << 8) | (self.c as u16)
    }

    /// Gets the value of the DE register pair.
    pub fn de(&self) -> u16 {
        ((self.d as u16) << 8) | (self.e as u16)
    }

    /// Sets the SP.
    pub fn set_sp(&mut self, value: u16) {
        self.sp = value;
    }

    /// Sets the SP and ticks the bus once. Needed for certain instructions that modify
    /// the SP from a u16 value.
    ///
    /// On real hardware, most instructions that modify the SP actually transfer the lower 8 bits first,
    /// followed by the upper 8 bits, which is handled by our fetch methods.
    pub fn set_sp_and_tick(&mut self, value: u16, bus: &mut Bus) {
        self.sp = value;
        bus.tick_m_cycle();
    }
}
