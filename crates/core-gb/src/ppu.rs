pub const SCREEN_WIDTH: usize = 160;
pub const SCREEN_HEIGHT: usize = 144;
pub const SCREEN_SIZE: usize = SCREEN_WIDTH * SCREEN_HEIGHT;

const DOTS_PER_LINE: u16 = 456;

/// STAT register bit masks.
const STAT_LYC_FLAG: u8 = 0x04; // read-only: LY == LYC
const STAT_HBLANK_INT: u8 = 0x08; // mode 0 select
const STAT_VBLANK_INT: u8 = 0x10; // mode 1 select
const STAT_OAM_INT: u8 = 0x20; // mode 2 select
const STAT_LYC_INT: u8 = 0x40; // LYC=LY select
const STAT_WRITABLE: u8 = 0x78; // bits 3-6

// LCDC bit masks.
const LCDC_BG_WINDOW_ENABLE: u8 = 0x01;
const LCDC_OBJ_ENABLE: u8 = 0x02;
const LCDC_OBJ_SIZE: u8 = 0x04; // 0 = 8x8, 1 = 8x16
const LCDC_BG_TILE_MAP: u8 = 0x08;
const LCDC_TILE_DATA_SELECT: u8 = 0x10;
const LCDC_WINDOW_ENABLE: u8 = 0x20;
const LCDC_WINDOW_TILE_MAP: u8 = 0x40;
const LCDC_LCD_ENABLE: u8 = 0x80;

#[derive(Clone, Copy)]
struct SpriteEntry {
    y: u8,
    x: u8,
    tile: u8,
    attrs: u8,
}

/// Interrupts requested by the PPU during a tick.
#[derive(Default, Clone, Copy)]
pub struct PpuInterrupts {
    pub vblank: bool,
    pub stat: bool,
}

/// Classic Game Boy 4-shade greens (ARGB).
const PALETTE: [u32; 4] = [
    0xFFE0F8D0, // 0: Lightest
    0xFF88C070, // 1: Light
    0xFF346856, // 2: Dark
    0xFF081820, // 3: Darkest
];

#[derive(Debug, PartialEq, Eq, Clone, Copy)]
pub enum PpuMode {
    HBlank = 0,
    VBlank = 1,
    OAMSearch = 2,
    PixelTransfer = 3,
}

pub struct Ppu {
    lcdc: u8,                         // 0xFF40 LCD Control
    stat: u8,                         // 0xFF41 LCD Status
    scy: u8,                          // 0xFF42 Scroll Y
    scx: u8,                          // 0xFF43 Scroll X
    ly: u8,                           // 0xFF44 LY register (0..153)
    lyc: u8,                          // 0xFF45 LYC register (0..153)
    bgp: u8,                          // 0xFF47 BG Palette
    obp0: u8,                         // 0xFF48 Object Palette 0
    obp1: u8,                         // 0xFF49 Object Palette 1
    wy: u8,                           // 0xFF4A Window Y position
    wx: u8,                           // 0xFF4B Window X position (+7)
    vram: [u8; 0x2000],               // Video RAM (8KB)
    oam: [u8; 0xA0],                  // Object Attribute Memory (OAM)
    dots: u16,                        // Current dot within scanline (0..456)
    pixel_x: u8,                      // Pixels pushed on current scanline (0..160)
    mode3_delay: u8,                  // Penalty delay before pixel transfer
    window_line: u8,                  // Window's internal line counter
    window_drawn_this_scanline: bool, // Latch for current line
    stat_line: bool,                  // Previous level of the internal STAT interrupt line
    lcd_was_on: bool,                 // To detect LCD off->on transitions
    line_sprites: Vec<SpriteEntry>,   // Up to 10, sorted by priority for this scanline
    pixels: [u32; SCREEN_SIZE],       // Pixel color buffer for the screen
}

impl Ppu {
    pub fn new() -> Self {
        Self {
            lcdc: 0,
            stat: PpuMode::OAMSearch as u8,
            scy: 0,
            scx: 0,
            ly: 0,
            lyc: 0,
            bgp: 0,
            obp0: 0,
            obp1: 0,
            wy: 0,
            wx: 0,
            vram: [0; 0x2000],
            oam: [0; 0xA0],
            dots: 0,
            pixel_x: 0,
            mode3_delay: 12,
            window_line: 0,
            window_drawn_this_scanline: false,
            stat_line: false,
            lcd_was_on: false,
            line_sprites: Vec::with_capacity(10),
            pixels: [0xFFFFFFFF; SCREEN_SIZE],
        }
    }

    pub fn frame_buffer(&self) -> &[u32; SCREEN_SIZE] {
        &self.pixels
    }

    pub fn is_lcd_enabled(&self) -> bool {
        (self.lcdc & LCDC_LCD_ENABLE) != 0
    }

    fn window_enabled(&self) -> bool {
        (self.lcdc & LCDC_WINDOW_ENABLE) != 0
    }

    fn mode(&self) -> PpuMode {
        match self.stat & 0x03 {
            0 => PpuMode::HBlank,
            1 => PpuMode::VBlank,
            2 => PpuMode::OAMSearch,
            3 => PpuMode::PixelTransfer,
            _ => unreachable!(),
        }
    }

    fn set_mode(&mut self, mode: PpuMode) {
        self.stat = (self.stat & !0x03) | (mode as u8);
    }

    /// Refreshes the LY=LYC flag, recomputes the STAT interrupt line, and
    /// returns true if the line just went from low to high.
    fn update_stat_line(&mut self) -> bool {
        let coincidence = self.ly == self.lyc;
        if coincidence {
            self.stat |= STAT_LYC_FLAG;
        } else {
            self.stat &= !STAT_LYC_FLAG;
        }

        let line = (coincidence && self.stat & STAT_LYC_INT != 0)
            || match self.mode() {
                PpuMode::HBlank => self.stat & STAT_HBLANK_INT != 0,
                PpuMode::VBlank => self.stat & STAT_VBLANK_INT != 0,
                PpuMode::OAMSearch => self.stat & STAT_OAM_INT != 0,
                PpuMode::PixelTransfer => false, // mode 3 has no select bit
            };

        let rising = line && !self.stat_line;
        self.stat_line = line;
        rising
    }

    /// Advances the PPU by 1 M-cycle (4 dots) and returns requested interrupts.
    pub fn tick_m_cycle(&mut self) -> PpuInterrupts {
        let mut irq = PpuInterrupts::default();
        for _ in 0..4 {
            let r = self.tick_dot();
            irq.vblank |= r.vblank;
            irq.stat |= r.stat;
        }
        irq
    }

    fn tick_dot(&mut self) -> PpuInterrupts {
        let mut irq = PpuInterrupts::default();

        if !self.is_lcd_enabled() {
            self.lcd_was_on = false;
            self.dots = 0;
            self.ly = 0;
            self.window_line = 0;
            self.window_drawn_this_scanline = false;
            self.stat_line = false;
            self.stat &= !STAT_LYC_FLAG;
            self.set_mode(PpuMode::HBlank);
            return irq;
        }

        if !self.lcd_was_on {
            self.lcd_was_on = true;
            self.dots = 0;
            self.ly = 0;
            self.window_line = 0;
            self.set_mode(PpuMode::OAMSearch);
        }

        self.dots += 1;

        match self.mode() {
            PpuMode::OAMSearch => {
                if self.dots == 80 {
                    self.set_mode(PpuMode::PixelTransfer);
                    self.mode3_delay = 12;
                    self.pixel_x = 0;
                    self.scan_sprites();
                }
            }
            PpuMode::PixelTransfer => {
                if self.mode3_delay > 0 {
                    self.mode3_delay -= 1;
                } else {
                    if self.pixel_x < 160 {
                        self.render_pixel(self.pixel_x, self.ly);
                        self.pixel_x += 1;
                    }
                    if self.pixel_x >= 160 {
                        self.set_mode(PpuMode::HBlank);
                    }
                }
            }
            PpuMode::HBlank => {
                if self.dots >= DOTS_PER_LINE {
                    self.dots = 0;

                    if self.window_drawn_this_scanline {
                        self.window_line = self.window_line.wrapping_add(1);
                    }
                    self.window_drawn_this_scanline = false;

                    self.ly += 1;
                    if self.ly == 144 {
                        self.set_mode(PpuMode::VBlank);
                        irq.vblank = true;
                    } else {
                        self.set_mode(PpuMode::OAMSearch);
                    }
                }
            }
            PpuMode::VBlank => {
                if self.dots >= DOTS_PER_LINE {
                    self.dots = 0;
                    self.ly += 1;
                    if self.ly >= 154 {
                        self.ly = 0;
                        self.window_line = 0;
                        self.window_drawn_this_scanline = false;
                        self.set_mode(PpuMode::OAMSearch);
                    }
                }
            }
        }

        // Evaluate *after* mode/LY changes so the line reflects the new state
        // on the same dot.
        irq.stat = self.update_stat_line();
        irq
    }
}

/// PPU register and memory access methods.
impl Ppu {
    /// Reads a byte from a PPU register.
    pub fn reg_read_byte(&self, address: u16) -> u8 {
        match address {
            0xFF40 => self.lcdc,
            0xFF41 => self.stat | 0x80,
            0xFF42 => self.scy,
            0xFF43 => self.scx,
            0xFF44 => self.ly,
            0xFF45 => self.lyc,
            0xFF47 => self.bgp,
            0xFF48 => self.obp0,
            0xFF49 => self.obp1,
            0xFF4A => self.wy,
            0xFF4B => self.wx,
            _ => 0xFF,
        }
    }

    /// Writes a byte to a PPU register.
    pub fn reg_write_byte(&mut self, address: u16, value: u8) {
        match address {
            0xFF40 => self.lcdc = value,
            0xFF41 => self.stat = (self.stat & !STAT_WRITABLE) | (value & STAT_WRITABLE),
            0xFF42 => self.scy = value,
            0xFF43 => self.scx = value,
            0xFF44 => {}
            0xFF45 => self.lyc = value,
            0xFF47 => self.bgp = value,
            0xFF48 => self.obp0 = value,
            0xFF49 => self.obp1 = value,
            0xFF4A => self.wy = value,
            0xFF4B => self.wx = value,
            _ => {}
        }
    }

    /// Reads a byte from VRAM, respecting the PPU mode lock.
    pub fn vram_read_byte(&self, address: u16) -> u8 {
        if self.mode() == PpuMode::PixelTransfer {
            0xFF // VRAM is locked during Mode 3
        } else {
            self.vram[(address & 0x1FFF) as usize]
        }
    }

    /// Writes a byte to VRAM, respecting the PPU mode lock.
    pub fn vram_write_byte(&mut self, address: u16, value: u8) {
        if self.mode() != PpuMode::PixelTransfer {
            self.vram[(address & 0x1FFF) as usize] = value;
        }
    }

    pub fn oam_read_byte(&self, address: u16) -> u8 {
        let idx = (address & 0xFF) as usize;
        if idx >= self.oam.len()
            || matches!(self.mode(), PpuMode::OAMSearch | PpuMode::PixelTransfer)
        {
            0xFF
        } else {
            self.oam[idx]
        }
    }

    pub fn oam_write_byte(&mut self, address: u16, value: u8) {
        let idx = (address & 0xFF) as usize;
        if idx < self.oam.len()
            && !matches!(self.mode(), PpuMode::OAMSearch | PpuMode::PixelTransfer)
        {
            self.oam[idx] = value;
        }
    }

    pub fn vram_dma_read(&self, address: u16) -> u8 {
        self.vram[(address & 0x1FFF) as usize]
    }

    /// DMA write: bypasses the mode-based OAM lock.
    pub fn oam_dma_write(&mut self, index: u8, value: u8) {
        if (index as usize) < self.oam.len() {
            self.oam[index as usize] = value;
        }
    }
}

/// PPU rendering methods.
impl Ppu {
    /// Renders a single pixel at the given (x, y) coordinates, taking into account
    /// the background, window, and sprite layers, and writes the final color to the
    /// PPU's pixel buffer.
    pub fn render_pixel(&mut self, x: u8, y: u8) {
        if x >= 160 || y >= 144 {
            return;
        }

        let bg_num = self.bg_color_number(x, y);
        let bg_rgb = PALETTE[((self.bgp >> (bg_num * 2)) & 0x03) as usize];

        let final_color = self.sprite_pixel_at(x, y, bg_num).unwrap_or(bg_rgb);

        let buffer_index = (y as usize * SCREEN_WIDTH) + x as usize;
        self.pixels[buffer_index] = final_color;
    }

    /// Raw (pre-BGP) 0-3 BG/window color number. 0 when LCDC.0 disables both layers.
    fn bg_color_number(&mut self, x: u8, y: u8) -> u8 {
        if self.lcdc & LCDC_BG_WINDOW_ENABLE == 0 {
            return 0;
        }

        let window_active =
            self.window_enabled() && y >= self.wy && (x as u16 + 7) >= self.wx as u16;

        if window_active {
            self.window_drawn_this_scanline = true; // Set latch

            let win_x = (x as u16 + 7 - self.wx as u16) as u8;
            let map_base = if self.lcdc & LCDC_WINDOW_TILE_MAP != 0 {
                0x9C00
            } else {
                0x9800
            };

            let window_row = if self.wx == 166 {
                self.window_line.wrapping_add(1)
            } else {
                self.window_line
            };
            self.tile_pixel_color_index(map_base, win_x, window_row)
        } else {
            let world_x = x.wrapping_add(self.scx);
            let world_y = y.wrapping_add(self.scy);
            let map_base = if self.lcdc & LCDC_BG_TILE_MAP != 0 {
                0x9C00
            } else {
                0x9800
            };

            self.tile_pixel_color_index(map_base, world_x, world_y)
        }
    }

    /// Fetches the raw 2-bit color index for a pixel within a tile map,
    /// given the map base address and the pixel's world (map-space)
    /// coordinates. The tile-data area is selected by LCDC.4, which is
    /// shared by both the BG and the window.
    fn tile_pixel_color_index(&self, map_base: u16, world_x: u8, world_y: u8) -> u8 {
        // Identify 8x8 tile position in the 32x32 tile map.
        let tile_x = (world_x / 8) as u16;
        let tile_y = (world_y / 8) as u16;

        let tile_map_addr = map_base + (tile_y * 32) + tile_x;
        let tile_index = self.vram[(tile_map_addr - 0x8000) as usize];

        // Locate tile data address in VRAM. LCDC.4 selects between the
        // unsigned 0x8000 base and the signed 0x9000 base.
        let tile_data_addr: u16 = if (self.lcdc & LCDC_TILE_DATA_SELECT) != 0 {
            0x8000 + (tile_index as u16 * 16)
        } else {
            let signed_index = tile_index as i8 as i16;
            (0x9000i32 + (signed_index * 16) as i32) as u16
        };

        // Fetch the 2 bytes for the current 8-pixel scanline row.
        let row_in_tile = (world_y % 8) as u16;
        let byte1 = self.vram[(tile_data_addr + (row_in_tile * 2) - 0x8000) as usize];
        let byte2 = self.vram[(tile_data_addr + (row_in_tile * 2) + 1 - 0x8000) as usize];

        // Extract 2-bit color index for this exact pixel (MSB first).
        let bit_idx = 7 - (world_x % 8);
        color_index(byte1, byte2, bit_idx)
    }

    /// Returns the final color of the highest-priority opaque sprite pixel at
    /// this x, or None if no sprite pixel is visible (no sprite, all transparent,
    /// or the winning sprite is behind a non-zero BG pixel). Transparent pixels
    /// (color 0) let lower-priority sprites show through.
    fn sprite_pixel_at(&self, x: u8, y: u8, bg_num: u8) -> Option<u32> {
        let height: u8 = if self.lcdc & LCDC_OBJ_SIZE != 0 {
            16
        } else {
            8
        };

        for sprite in &self.line_sprites {
            let sx = sprite.x as i16 - 8;
            let sy = sprite.y as i16 - 16;
            if (x as i16) < sx
                || (x as i16) >= sx + 8
                || (y as i16) < sy
                || (y as i16) >= sy + height as i16
            {
                continue;
            }

            let y_flip = sprite.attrs & 0x40 != 0;
            let x_flip = sprite.attrs & 0x20 != 0;
            let palette = if sprite.attrs & 0x10 != 0 {
                self.obp1
            } else {
                self.obp0
            };

            let mut row = (y as i16 - sy) as u8; // 0..height
            if y_flip {
                row = height - 1 - row;
            }
            let tile = if height == 16 {
                if row < 8 {
                    sprite.tile & 0xFE
                } else {
                    sprite.tile | 0x01
                }
            } else {
                sprite.tile
            };
            let row_in_tile = (row % 8) as u16;

            let mut col = (x as i16 - sx) as u8; // 0..8
            if x_flip {
                col = 7 - col;
            }

            let tile_data_addr = 0x8000u16 + (tile as u16 * 16);
            let byte1 = self.vram[(tile_data_addr + row_in_tile * 2 - 0x8000) as usize];
            let byte2 = self.vram[(tile_data_addr + row_in_tile * 2 + 1 - 0x8000) as usize];
            let color_num = color_index(byte1, byte2, 7 - col);

            if color_num == 0 {
                continue; // transparent: fall through to a lower-priority sprite
            }

            let behind_bg = sprite.attrs & 0x80 != 0;
            if behind_bg && bg_num != 0 {
                return None; // BG wins
            }

            let mapped = (palette >> (color_num * 2)) & 0x03;
            return Some(PALETTE[mapped as usize]);
        }
        None
    }

    /// Selects up to 10 sprites intersecting the current scanline, in
    /// draw priority order (lowest X first; ties keep OAM index order,
    /// since sort_by is stable).
    fn scan_sprites(&mut self) {
        self.line_sprites.clear();
        if self.lcdc & LCDC_OBJ_ENABLE == 0 {
            return;
        }
        let height: i16 = if self.lcdc & LCDC_OBJ_SIZE != 0 {
            16
        } else {
            8
        };
        let ly = self.ly as i16;

        for i in 0..40 {
            if self.line_sprites.len() == 10 {
                break;
            }
            let base = i * 4;
            let sy = self.oam[base];
            let top = sy as i16 - 16;
            if ly >= top && ly < top + height {
                self.line_sprites.push(SpriteEntry {
                    y: sy,
                    x: self.oam[base + 1],
                    tile: self.oam[base + 2],
                    attrs: self.oam[base + 3],
                });
            }
        }

        // Sort the selected sprites by their X coordinate to determine draw priority
        self.line_sprites.sort_by_key(|s| s.x);

        // Approximate mode-3 penalty: ~6 dots per sprite that can appear on screen
        let penalty = self.line_sprites.iter().filter(|s| s.x < 168).count() as u8 * 6;
        self.mode3_delay += penalty; // max 12 + 60, fits in u8
    }
}

/// Extracts the 2-bit color index from two bytes for a specific bit position.
fn color_index(byte1: u8, byte2: u8, bit_idx: u8) -> u8 {
    let low_bit = (byte1 >> bit_idx) & 0x01;
    let high_bit = (byte2 >> bit_idx) & 0x01;
    (high_bit << 1) | low_bit
}
