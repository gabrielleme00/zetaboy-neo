use super::Mbc;

pub struct NoMbc {
    rom: Vec<u8>,
}

impl NoMbc {
    pub fn new(rom: Vec<u8>) -> Self {
        Self { rom }
    }
}

impl Mbc for NoMbc {
    fn read_byte(&self, address: u16) -> u8 {
        if address < 0x8000 {
            self.rom[(address as usize) % self.rom.len()]
        } else {
            0xFF
        }
    }

    fn write_byte(&mut self, _address: u16, _value: u8) {}
}