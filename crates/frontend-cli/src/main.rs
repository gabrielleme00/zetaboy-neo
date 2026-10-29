use core_common::SystemCore;

fn main() {
    let arg = std::env::args().nth(1).expect("Please provide the path to the ROM file.");
    let rom = std::fs::read(arg).expect("Failed to read the ROM file.");
    let mut gb = core_gb::GameBoy::new(rom);
    gb.set_post_boot_state();
    loop {
        gb.step();
    }
}
