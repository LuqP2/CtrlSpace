#![cfg_attr(
    all(not(debug_assertions), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod src_tauri;

fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            src_tauri::commands::greet,
            src_tauri::commands::list_devices,
            src_tauri::commands::list_steam_controller_interfaces,
            src_tauri::commands::ping,
            src_tauri::commands::detect_steam_controller,
            src_tauri::commands::connect_steam_controller,
            src_tauri::commands::disconnect_steam_controller,
            src_tauri::commands::is_steam_controller_connected,
            src_tauri::commands::read_controller_input,
            src_tauri::commands::read_raw_input_debug,
            src_tauri::commands::start_mapper,
            src_tauri::commands::stop_mapper
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}