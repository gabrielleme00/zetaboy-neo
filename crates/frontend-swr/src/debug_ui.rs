use core_gb::Ppu;
use minifb::{Scale, ScaleMode, Window, WindowOptions};

pub const TILE_VIEWER_W: usize = 128; // 16 tiles * 8px
pub const TILE_VIEWER_H: usize = 192; // 24 tiles * 8px (384 tiles total)

pub const MAP_VIEWER_W: usize = 256; // 32 tiles * 8px
pub const MAP_VIEWER_H: usize = 256; // 32 tiles * 8px

pub fn open_debug_window(
    title: &str,
    width: usize,
    height: usize,
    scale: Scale,
) -> minifb::Result<Window> {
    Window::new(
        title,
        width,
        height,
        WindowOptions {
            resize: true,
            scale,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
}

/// Decodes one pixel from a tile starting at VRAM address `tile_addr` (0x8000..=0x97F0).
fn sample_tile_pixel(
    vram: &[u8; 0x2000],
    tile_addr: u16,
    px: usize, // 0..8
    py: usize, // 0..8
) -> u8 {
    let offset = (tile_addr - 0x8000) as usize + py * 2;
    let byte1 = vram[offset];
    let byte2 = vram[offset + 1];
    let bit_idx = (7 - (px & 7)) as u8;

    let low_bit = (byte1 >> bit_idx) & 0x01;
    let high_bit = (byte2 >> bit_idx) & 0x01;
    (high_bit << 1) | low_bit
}

/// Renders all 384 tiles in 0x8000..=0x97FF as a 16x24 grid (128x192).
pub fn render_tile_viewer(ppu: &Ppu, buf: &mut [u32]) {
    let vram = &ppu.vram;
    let cols = TILE_VIEWER_W / 8; // 16

    for y in 0..TILE_VIEWER_H {
        let tile_y = y / 8;
        let py = y % 8;

        for x in 0..TILE_VIEWER_W {
            let tile_x = x / 8;
            let px = x % 8;

            let tile_index = (tile_y * cols + tile_x) as u16; // 0..384
            let tile_addr = 0x8000 + tile_index * 16;

            let color_num = sample_tile_pixel(vram, tile_addr, px, py);
            buf[y * TILE_VIEWER_W + x] = ppu.bg_shade_color(color_num);
        }
    }
}

/// Renders the active 32x32 background tilemap (256x256) using LCDC.3 and LCDC.4.
pub fn render_tilemap_viewer(ppu: &Ppu, buf: &mut [u32]) {
    let vram = &ppu.vram;
    let map_base = ppu.bg_map_base();

    for y in 0..MAP_VIEWER_H {
        let map_y = y / 8; // 0..32
        let py = y % 8;

        for x in 0..MAP_VIEWER_W {
            let map_x = x / 8; // 0..32
            let px = x % 8;

            let map_offset = (map_base - 0x8000) as usize + map_y * 32 + map_x;
            let raw_tile_id = vram[map_offset];
            let tile_addr = ppu.get_tile_data_addr(raw_tile_id);

            let color_num = sample_tile_pixel(vram, tile_addr, px, py);
            buf[y * MAP_VIEWER_W + x] = ppu.bg_shade_color(color_num);
        }
    }
}
