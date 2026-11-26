mod debug_ui;
mod input;

use std::error::Error;
use std::path::PathBuf;
use std::thread;
use std::time::Duration;

use minifb::{Scale, ScaleMode, Window, WindowOptions};

use audio_cpal::CpalAudioSink;
use core_common::SystemCore;
use core_gb::{GameBoy, SCREEN_HEIGHT, SCREEN_WIDTH};
use debug_ui::{
    MAP_VIEWER_H, MAP_VIEWER_W, TILE_VIEWER_H, TILE_VIEWER_W, open_debug_window,
    render_tile_viewer, render_tilemap_viewer,
};
use input::InputHandler;

fn main() -> Result<(), Box<dyn Error>> {
    let rom_path = parse_rom_path()?;
    let rom = std::fs::read(&rom_path)?;

    let mut gb = GameBoy::new(rom);
    gb.set_post_boot_state();

    let has_audio = attach_audio(&mut gb);
    let mut input = InputHandler::new();
    let mut debug_windows = DebugWindows::new();
    let mut paused = false;

    let mut game_window = create_window("zetaboy-neo", has_audio)?;
    while game_window.is_open() {
        pace_frame(&gb, has_audio);

        let mut windows: Vec<&Window> = vec![&game_window];
        windows.extend(debug_windows.open_windows());

        let debug_actions = input.get_debug_actions(&windows);
        if debug_actions.toggle_pause {
            paused = !paused;
        }
        if debug_actions.step_frame {
            gb.step_frame();
        }
        if debug_actions.toggle_tile_viewer {
            debug_windows.toggle_tile_viewer()?;
        }
        if debug_actions.toggle_map_viewer {
            debug_windows.toggle_map_viewer()?;
        }

        let controller = input.get_controller(&game_window);
        gb.set_input(&controller);

        if !paused {
            gb.step_frame();
        }

        game_window.update_with_buffer(gb.frame_buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)?;
        debug_windows.update(&gb)?;
    }

    Ok(())
}

fn parse_rom_path() -> Result<PathBuf, Box<dyn Error>> {
    std::env::args()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "Usage: zetaboy-neo <path-to-rom>".into())
}

fn attach_audio(gb: &mut GameBoy) -> bool {
    match CpalAudioSink::new() {
        Ok(sink) => {
            gb.set_audio_sink(Box::new(sink));
            true
        }
        Err(err) => {
            eprintln!("Warning: Failed to initialize audio sink: {err}");
            false
        }
    }
}

fn create_window(title: &str, has_audio: bool) -> Result<Window, Box<dyn Error>> {
    let mut window = Window::new(
        title,
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )?;

    if !has_audio {
        window.set_target_fps(60);
    }

    Ok(window)
}

fn pace_frame(gb: &GameBoy, has_audio: bool) {
    if has_audio {
        while !gb.audio_ready_for_frame() {
            thread::sleep(Duration::from_micros(250));
        }
    }
}

struct DebugWindows {
    tile_viewer: Option<Window>,
    tile_buf: Vec<u32>,
    map_viewer: Option<Window>,
    map_buf: Vec<u32>,
}

impl DebugWindows {
    fn new() -> Self {
        Self {
            tile_viewer: None,
            tile_buf: vec![0; TILE_VIEWER_W * TILE_VIEWER_H],
            map_viewer: None,
            map_buf: vec![0; MAP_VIEWER_W * MAP_VIEWER_H],
        }
    }

    fn open_windows(&self) -> Vec<&Window> {
        let mut wins = Vec::with_capacity(2);
        if let Some(win) = &self.tile_viewer {
            if win.is_open() {
                wins.push(win);
            }
        }
        if let Some(win) = &self.map_viewer {
            if win.is_open() {
                wins.push(win);
            }
        }
        wins
    }

    fn toggle_tile_viewer(&mut self) -> minifb::Result<()> {
        if self.tile_viewer.is_some() {
            self.tile_viewer = None;
        } else {
            self.tile_viewer = Some(open_debug_window(
                "VRAM Tile Viewer (128x192)",
                TILE_VIEWER_W,
                TILE_VIEWER_H,
                Scale::X4,
            )?);
        }
        Ok(())
    }

    fn toggle_map_viewer(&mut self) -> minifb::Result<()> {
        if self.map_viewer.is_some() {
            self.map_viewer = None;
        } else {
            self.map_viewer = Some(open_debug_window(
                "Tilemap Viewer (256x256)",
                MAP_VIEWER_W,
                MAP_VIEWER_H,
                Scale::X2,
            )?);
        }
        Ok(())
    }

    fn update(&mut self, gb: &GameBoy) -> minifb::Result<()> {
        if let Some(win) = &mut self.tile_viewer {
            if !win.is_open() {
                self.tile_viewer = None;
            } else {
                render_tile_viewer(gb.ppu(), &mut self.tile_buf);
                win.update_with_buffer(&self.tile_buf, TILE_VIEWER_W, TILE_VIEWER_H)?;
            }
        }

        if let Some(win) = &mut self.map_viewer {
            if !win.is_open() {
                self.map_viewer = None;
            } else {
                render_tilemap_viewer(gb.ppu(), &mut self.map_buf);
                win.update_with_buffer(&self.map_buf, MAP_VIEWER_W, MAP_VIEWER_H)?;
            }
        }

        Ok(())
    }
}
