/// Distinct button flags for the internal button state. The bit positions
/// here are arbitrary — `read` maps them onto P1's lower nibble based on
/// which group is currently selected. Note that, for example, `BTN_A` and
/// `BTN_RIGHT` are different flags even though both surface as P1 bit 0
/// when their respective groups are selected.
pub mod buttons {
    pub const BTN_RIGHT: u8 = 0x01;
    pub const BTN_LEFT: u8 = 0x02;
    pub const BTN_UP: u8 = 0x04;
    pub const BTN_DOWN: u8 = 0x08;
    pub const BTN_A: u8 = 0x10;
    pub const BTN_B: u8 = 0x20;
    pub const BTN_SELECT: u8 = 0x40;
    pub const BTN_START: u8 = 0x80;
}

pub use buttons::*;

/// P1 group-select lines. These are active low: writing a 0 to bit 5
/// selects the action group, writing a 0 to bit 4 selects the direction
/// group. Both can be selected at once.
const P1_SELECT_ACTION: u8 = 0x20; // bit 5
const P1_SELECT_DIRECTION: u8 = 0x10; // bit 4

/// Game Boy joypad (P1/JOYP at 0xFF00).
pub struct Joypad {
    /// Bitmask of pressed buttons, using the BTN_* flags above.
    buttons: u8,
    /// Last value written to P1: only bits 4-5 are meaningful.
    select: u8,
}

impl Joypad {
    pub fn new() -> Self {
        Self {
            buttons: 0,
            select: 0,
        }
    }

    /// Reads P1.
    ///
    /// * Bits 7-6 always read as 1 on DMG.
    /// * Bits 5-4 echo the last written select lines.
    /// * Bits 3-0 read 0 for each pressed button in a selected group,
    ///   1 otherwise. When neither group is selected, all four read 1.
    pub fn read(&self) -> u8 {
        let mut lower = 0x0F;

        if self.select & P1_SELECT_ACTION == 0 {
            if self.buttons & BTN_A != 0 {
                lower &= !0x01;
            }
            if self.buttons & BTN_B != 0 {
                lower &= !0x02;
            }
            if self.buttons & BTN_SELECT != 0 {
                lower &= !0x04;
            }
            if self.buttons & BTN_START != 0 {
                lower &= !0x08;
            }
        }
        if self.select & P1_SELECT_DIRECTION == 0 {
            if self.buttons & BTN_RIGHT != 0 {
                lower &= !0x01;
            }
            if self.buttons & BTN_LEFT != 0 {
                lower &= !0x02;
            }
            if self.buttons & BTN_UP != 0 {
                lower &= !0x04;
            }
            if self.buttons & BTN_DOWN != 0 {
                lower &= !0x08;
            }
        }

        0xC0 | (self.select & 0x30) | lower
    }

    /// Writes P1. Only bits 5-4 (the group-select lines) are stored; bits
    /// 0-3 and 6-7 are ignored. A write that switches a group on while a
    /// button in it is already held can itself pull a line low, but we
    /// only surface IRQs from `set_button` — see the note below.
    pub fn write(&mut self, value: u8) {
        self.select = value & 0x30;
    }

    /// Updates a button's pressed state.
    ///
    /// Returns `true` if this caused a high-to-low transition on any
    /// *currently selected* P1 input line. That is the condition for a
    /// joypad interrupt, so the caller can do:
    ///
    /// ```ignore
    /// if joypad.set_button(BTN_A, true) {
    ///     cpu.request_interrupt(Interrupt::Joypad);
    /// }
    /// ```
    pub fn set_button(&mut self, button: u8, pressed: bool) -> bool {
        let before = self.read() & 0x0F;
        if pressed {
            self.buttons |= button;
        } else {
            self.buttons &= !button;
        }
        let after = self.read() & 0x0F;
        // High -> low on any selected input line.
        (before & !after) != 0
    }
}
