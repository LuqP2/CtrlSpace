use std::sync::{Arc, Mutex};
use std::thread;
use std::time::Duration;
use enigo::{Enigo, Key, Keyboard, Settings, Direction};
use once_cell::sync::Lazy;

use crate::src_tauri::input_parser::{ControllerInput, ButtonState};
use crate::src_tauri::steam_controller::SteamControllerManager;

pub struct MapperState {
    pub is_running: bool,
    pub profile: Profile, // To be defined
}

#[derive(Clone, Default)]
pub struct Profile {
    // Simple profile for MVP: a few hardcoded mappings or configurable
    // We'll expand this later via Tauri commands
    pub map_a_to_space: bool,
}

static MAPPER_STATE: Lazy<Arc<Mutex<MapperState>>> = Lazy::new(|| {
    Arc::new(Mutex::new(MapperState {
        is_running: false,
        profile: Profile { map_a_to_space: true },
    }))
});

pub fn start_mapper(sc_manager: Arc<Mutex<Option<SteamControllerManager>>>) {
    let state_clone = MAPPER_STATE.clone();
    
    thread::spawn(move || {
        let mut enigo = Enigo::new(&Settings::default()).unwrap();
        let mut last_state = ButtonState::default();

        loop {
            // Check if mapper is requested to run
            let is_running = {
                state_clone.lock().unwrap().is_running
            };

            if !is_running {
                thread::sleep(Duration::from_millis(50));
                continue;
            }

            // Read controller
            let input_opt = {
                let manager = sc_manager.lock().unwrap();
                manager.as_ref().and_then(|m| m.read_input().ok())
            };

            if let Some(raw_data) = input_opt {
                if let Ok(input) = crate::src_tauri::input_parser::parse_input_report(&raw_data) {
                    let profile = {
                        state_clone.lock().unwrap().profile.clone()
                    };

                    // Execute mapping based on changes
                    
                    // Button A -> Space
                    if profile.map_a_to_space {
                        if input.buttons.a && !last_state.a {
                            let _ = enigo.key(Key::Space, Direction::Press);
                        } else if !input.buttons.a && last_state.a {
                            let _ = enigo.key(Key::Space, Direction::Release);
                        }
                    }

                    // Store last state
                    last_state = input.buttons;
                }
            }

            // High frequency polling (approx ~250Hz = 4ms)
            thread::sleep(Duration::from_millis(4));
        }
    });
}

pub fn set_mapper_running(running: bool) {
    let mut state = MAPPER_STATE.lock().unwrap();
    state.is_running = running;
}
