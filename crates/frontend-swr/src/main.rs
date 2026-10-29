use core_common::{ControllerState, SystemCore};
use core_gb::{GameBoy, SCREEN_HEIGHT, SCREEN_WIDTH};

use minifb::{Key, Scale, ScaleMode, Window, WindowOptions};

fn main() {
    let arg = std::env::args()
        .nth(1)
        .expect("Please provide the path to the ROM file.");
    let rom = std::fs::read(arg).expect("Failed to read the ROM file.");
    let mut gb = GameBoy::new(rom);
    gb.set_post_boot_state();

    let mut has_audio = false;
    match audio_cpal::CpalAudioSink::new() {
        Ok(sink) => {
            gb.set_audio_sink(Box::new(sink));
            has_audio = true;
        }
        Err(e) => {
            eprintln!("Warning: Failed to initialize audio sink: {e}");
        }
    }

    let mut window = Window::new(
        "zetaboy-neo",
        SCREEN_WIDTH,
        SCREEN_HEIGHT,
        WindowOptions {
            resize: true,
            scale: Scale::X4,
            scale_mode: ScaleMode::AspectRatioStretch,
            ..WindowOptions::default()
        },
    )
    .unwrap_or_else(|e| {
        panic!("{}", e);
    });

    if !has_audio {
        window.set_target_fps(60);
    }

    let mut input = ControllerState::default();

    while window.is_open() {
        if has_audio {
            while !gb.audio_ready_for_frame() {
                std::thread::sleep(std::time::Duration::from_millis(1));
            }
        }
        update_controller_state(&window, &mut input);
        gb.set_input(&input);
        gb.step_frame();
        window
            .update_with_buffer(gb.frame_buffer(), SCREEN_WIDTH, SCREEN_HEIGHT)
            .unwrap();
    }
}

fn update_controller_state(window: &Window, controller: &mut ControllerState) {
    controller.up = window.is_key_down(Key::Up);
    controller.down = window.is_key_down(Key::Down);
    controller.left = window.is_key_down(Key::Left);
    controller.right = window.is_key_down(Key::Right);
    controller.a = window.is_key_down(Key::Z);
    controller.b = window.is_key_down(Key::X);
    controller.start = window.is_key_down(Key::Enter);
    controller.select = window.is_key_down(Key::Tab);
}
