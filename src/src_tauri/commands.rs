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

#[derive(Serialize)]
pub struct DetailedDeviceInfo {
    pub vendor_id: u16,
    pub product_id: u16,
    pub product: String,
    pub interface_number: i32,
    pub usage_page: u16,
    pub usage: u16,
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
pub fn list_steam_controller_interfaces() -> Vec<DetailedDeviceInfo> {
    let mut out = vec![];
    if let Ok(api) = hidapi::HidApi::new() {
        for d in api.device_list() {
            if d.vendor_id() == 0x28de {  // Valve
                out.push(DetailedDeviceInfo {
                    vendor_id: d.vendor_id(),
                    product_id: d.product_id(),
                    product: d.product_string().unwrap_or("Unknown").to_string(),
                    interface_number: d.interface_number(),
                    usage_page: d.usage_page(),
                    usage: d.usage(),
                });
            }
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

#[tauri::command]
pub fn read_raw_input_debug() -> Result<String, String> {
    let manager = SC_MANAGER.lock().unwrap();

    match manager.as_ref() {
        Some(m) => {
            match m.read_input() {
                Ok(data) => {
                    // Convert to hex string for debugging
                    let hex: Vec<String> = data.iter().map(|b| format!("{:02x}", b)).collect();
                    Ok(format!("Size: {} bytes\nHex: {}", data.len(), hex.join(" ")))
                }
                Err(e) => Err(e),
            }
        }
        None => Err("Steam Controller manager not initialized".to_string()),
    }
}