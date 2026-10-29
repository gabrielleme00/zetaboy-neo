use super::Mbc;

pub struct Mbc1 {
    rom: Vec<u8>,
    ram: Vec<u8>,
    ram_enabled: bool,
    bank_low: u8,
    bank_high: u8,
    ram_mode: bool,
}

impl Mbc1 {
    pub fn new(rom: Vec<u8>) -> Self {
        // TODO: read the RAM size from header byte 0x0149 instead of
        // assuming 32 KiB. For now 32 KiB covers most commercial carts.
        Self {
            rom,
            ram: vec![0; 0x8000],
            ram_enabled: false,
            bank_low: 1,
            bank_high: 0,
            ram_mode: false,
        }
    }

    fn num_rom_banks(&self) -> usize {
        (self.rom.len() / 0x4000).max(1)
    }

    fn num_ram_banks(&self) -> usize {
        (self.ram.len() / 0x2000).max(1)
    }

    /// Effective ROM bank for the 0x4000-0x7FFF window. R1 == 0 becomes 1,
    /// then R2 is shifted in as the upper bits. The `%` masks the result to
    /// the actual ROM size, matching MBC1's ignore-extra-bits behavior.
    fn rom_bank(&self) -> usize {
        let low = if self.bank_low == 0 { 1 } else { self.bank_low };
        let bank = ((self.bank_high as usize) << 5) | (low as usize);
        bank % self.num_rom_banks()
    }

    /// Effective ROM bank for the 0x0000-0x3FFF window.
    /// Mode 0: always bank 0. Mode 1: bank (R2 << 5).
    fn rom_bank_zero(&self) -> usize {
        if self.ram_mode {
            ((self.bank_high as usize) << 5) % self.num_rom_banks()
        } else {
            0
        }
    }

    /// Effective RAM bank for the 0xA000-0xBFFF window. R2 is only used
    /// as a RAM bank select in mode 1; in mode 0 it's stuck at 0.
    fn ram_bank(&self) -> usize {
        if self.ram_mode {
            (self.bank_high as usize) % self.num_ram_banks()
        } else {
            0
        }
    }
}

impl Mbc for Mbc1 {
    fn read_byte(&self, address: u16) -> u8 {
        match address {
            0x0000..=0x3FFF => {
                let bank = self.rom_bank_zero();
                self.rom[bank * 0x4000 + address as usize]
            }
            0x4000..=0x7FFF => {
                let bank = self.rom_bank();
                self.rom[bank * 0x4000 + (address as usize - 0x4000)]
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank = self.ram_bank();
                    self.ram[bank * 0x2000 + (address as usize - 0xA000)]
                } else {
                    0xFF
                }
            }
            _ => 0xFF,
        }
    }

    fn write_byte(&mut self, address: u16, value: u8) {
        match address {
            0x0000..=0x1FFF => {
                // RAM enable: low nibble must be 0x0A to enable.
                self.ram_enabled = (value & 0x0F) == 0x0A;
            }
            0x2000..=0x3FFF => {
                // R1: lower 5 bits of the ROM bank number.
                self.bank_low = value & 0x1F;
            }
            0x4000..=0x5FFF => {
                // R2: upper 2 bits (ROM bank in mode 0, RAM bank in mode 1).
                self.bank_high = value & 0x03;
            }
            0x6000..=0x7FFF => {
                // Banking mode select.
                self.ram_mode = (value & 0x01) != 0;
            }
            0xA000..=0xBFFF => {
                if self.ram_enabled {
                    let bank = self.ram_bank();
                    self.ram[bank * 0x2000 + (address as usize - 0xA000)] = value;
                }
            }
            _ => {}
        }
    }
}