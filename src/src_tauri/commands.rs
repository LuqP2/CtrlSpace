use serde::Serialize;
use std::sync::{Arc, Mutex};
use once_cell::sync::Lazy;
use super::steam_controller::{SteamControllerManager, SteamControllerInfo};
use super::input_parser::{parse_input_report, ControllerInput};

#[derive(Serialize)]
pub struct DeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
}

// Global Steam Controller manager
static SC_MANAGER: Lazy<Arc<Mutex<Option<SteamControllerManager>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
pub fn greet(name: &str) -> String {
    format!("Hello, {}!", name)
}

#[tauri::command]
pub fn list_devices() -> Vec<DeviceInfo> {
    let mut out = vec![];
    if let Ok(api) = hidapi::HidApi::new() {
        for d in api.device_list() {
            out.push(DeviceInfo {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                product: d.product_string().unwrap_or("Unknown").to_string(),
            });
        }
    }
    out
}

#[tauri::command]
pub fn ping() -> &'static str {
    "ok"
}

// Steam Controller Commands

#[tauri::command]
pub fn detect_steam_controller() -> Option<SteamControllerInfo> {
    // Initialize manager if not already done
    {
        let mut manager = SC_MANAGER.lock().unwrap();
        if manager.is_none() {
            *manager = SteamControllerManager::new().ok();
        }
    }

    // Detect controller
    let manager = SC_MANAGER.lock().unwrap();
    manager.as_ref().and_then(|m| m.detect())
}

#[tauri::command]
pub fn connect_steam_controller() -> Result<SteamControllerInfo, String> {
    // Initialize manager if not already done
    {
        let mut manager = SC_MANAGER.lock().unwrap();
        if manager.is_none() {
            *manager = SteamControllerManager::new().ok();
        }
    }

    // Connect to controller
    let manager = SC_MANAGER.lock().unwrap();
    match manager.as_ref() {
        Some(m) => m.connect(),
        None => Err("Failed to initialize Steam Controller manager".to_string()),
    }
}

#[tauri::command]
pub fn disconnect_steam_controller() -> bool {
    let manager = SC_MANAGER.lock().unwrap();
    if let Some(m) = manager.as_ref() {
        m.disconnect();
        true
    } else {
        false
    }
}

#[tauri::command]
pub fn is_steam_controller_connected() -> bool {
    let manager = SC_MANAGER.lock().unwrap();
    manager.as_ref().map(|m| m.is_connected()).unwrap_or(false)
}

#[tauri::command]
pub fn read_controller_input() -> Result<ControllerInput, String> {
    let manager = SC_MANAGER.lock().unwrap();

    match manager.as_ref() {
        Some(m) => {
            let raw_data = m.read_input()?;
            parse_input_report(&raw_data)
        }
        None => Err("Steam Controller manager not initialized".to_string()),
    }
}