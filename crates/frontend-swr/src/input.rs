use core_common::ControllerState;
use gilrs::{Axis, Button, Gilrs};
use minifb::{Key, Window};

const STICK_THRESHOLD: f32 = 0.5;

#[derive(Default, Debug, Clone, Copy)]
pub struct DebugActions {
    pub toggle_pause: bool,
    pub step_frame: bool,
    pub toggle_tile_viewer: bool,
    pub toggle_map_viewer: bool,
}

pub struct InputHandler {
    gilrs: Option<Gilrs>,
}

impl InputHandler {
    pub fn new() -> Self {
        Self {
            gilrs: Gilrs::new().ok(),
        }
    }

    /// Collects edge-triggered debug actions across all currently open windows.
    pub fn get_debug_actions(&self, windows: &[&Window]) -> DebugActions {
        let mut actions = DebugActions::default();

        for win in windows {
            for key in win.get_keys_pressed(minifb::KeyRepeat::No) {
                match key {
                    Key::P => actions.toggle_pause = true,
                    Key::N => actions.step_frame = true,
                    Key::F1 => actions.toggle_tile_viewer = true,
                    Key::F2 => actions.toggle_map_viewer = true,
                    _ => {}
                }
            }
        }

        actions
    }

    pub fn get_controller(&mut self, window: &Window) -> ControllerState {
        let mut state = Self::read_keyboard(window);

        if let Some(gilrs) = &mut self.gilrs {
            while let Some(event) = gilrs.next_event() {
                #[cfg(debug_assertions)]
                println!("{:?}", event.event);
            }

            for (_id, gamepad) in gilrs.gamepads() {
                if !gamepad.is_connected() {
                    continue;
                }

                let stick_x = gamepad.value(Axis::LeftStickX);
                let stick_y = gamepad.value(Axis::LeftStickY);

                state.a |= gamepad.is_pressed(Button::South);
                state.b |= gamepad.is_pressed(Button::East);
                state.start |= gamepad.is_pressed(Button::Start);
                state.select |= gamepad.is_pressed(Button::Select);

                state.up |= gamepad.is_pressed(Button::DPadUp) || stick_y > STICK_THRESHOLD;
                state.down |= gamepad.is_pressed(Button::DPadDown) || stick_y < -STICK_THRESHOLD;
                state.left |= gamepad.is_pressed(Button::DPadLeft) || stick_x < -STICK_THRESHOLD;
                state.right |= gamepad.is_pressed(Button::DPadRight) || stick_x > STICK_THRESHOLD;
            }
        }

        state
    }

    fn read_keyboard(window: &Window) -> ControllerState {
        let mut extra = std::collections::HashMap::new();

        extra.insert("example".to_string(), window.is_key_down(Key::Z));

        ControllerState {
            a: window.is_key_down(Key::Z),
            b: window.is_key_down(Key::X),
            start: window.is_key_down(Key::Enter),
            select: window.is_key_down(Key::Tab),
            up: window.is_key_down(Key::Up),
            down: window.is_key_down(Key::Down),
            left: window.is_key_down(Key::Left),
            right: window.is_key_down(Key::Right),
        }
    }
}
