mod nombc;
mod mbc1;

use nombc::NoMbc;
use mbc1::Mbc1;

pub struct Cart {
    mbc: Box<dyn Mbc>,
}

impl Cart {
    pub fn new(rom: Vec<u8>) -> Self {
        let mbc = match rom[0x147] {
            0x00 => Box::new(NoMbc::new(rom)) as Box<dyn Mbc>,
            0x01..=0x03 => Box::new(Mbc1::new(rom)) as Box<dyn Mbc>,
            other => panic!("Unsupported cartridge type {other:#04x}")
        };
        Self { mbc }
    }

    pub fn read_byte(&self, address: u16) -> u8 {
        self.mbc.read_byte(address)
    }

    pub fn write_byte(&mut self, address: u16, value: u8) {
        self.mbc.write_byte(address, value);
    }
}

trait Mbc {
    fn read_byte(&self, address: u16) -> u8;
    fn write_byte(&mut self, address: u16, value: u8);
}
