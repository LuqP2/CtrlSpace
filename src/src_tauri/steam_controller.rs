use hidapi::{HidApi, HidDevice};
use serde::Serialize;
use std::sync::{Arc, Mutex};

// Steam Controller USB IDs
pub const VALVE_VENDOR_ID: u16 = 0x28de;
pub const SC_WIRELESS_PID: u16 = 0x1142; // Wireless dongle
pub const SC_WIRED_PID: u16 = 0x1102;    // Wired connection

#[derive(Debug, Clone, Serialize)]
pub struct SteamControllerInfo {
    pub connected: bool,
    pub connection_type: String,
    pub product_name: String,
    pub serial: String,
}

pub struct SteamControllerManager {
    api: Arc<Mutex<HidApi>>,
    device: Arc<Mutex<Option<HidDevice>>>,
}

impl SteamControllerManager {
    pub fn new() -> Result<Self, String> {
        let api = HidApi::new().map_err(|e| format!("Failed to create HID API: {}", e))?;

        Ok(Self {
            api: Arc::new(Mutex::new(api)),
            device: Arc::new(Mutex::new(None)),
        })
    }

    /// Detect if a Steam Controller is connected
    pub fn detect(&self) -> Option<SteamControllerInfo> {
        let mut api = self.api.lock().unwrap();

        // Refresh device list
        if let Err(e) = api.refresh_devices() {
            eprintln!("Failed to refresh devices: {}", e);
            return None;
        }

        // Look for Steam Controller (wireless or wired)
        for device_info in api.device_list() {
            if device_info.vendor_id() == VALVE_VENDOR_ID {
                let connection_type = match device_info.product_id() {
                    SC_WIRELESS_PID => "Wireless",
                    SC_WIRED_PID => "Wired",
                    _ => continue, // Not a Steam Controller
                };

                return Some(SteamControllerInfo {
                    connected: true,
                    connection_type: connection_type.to_string(),
                    product_name: device_info
                        .product_string()
                        .unwrap_or("Steam Controller")
                        .to_string(),
                    serial: device_info
                        .serial_number()
                        .unwrap_or("Unknown")
                        .to_string(),
                });
            }
        }

        None
    }

    /// Connect to the Steam Controller
    pub fn connect(&self) -> Result<SteamControllerInfo, String> {
        let api = self.api.lock().unwrap();

        // Try to find and open the device
        for device_info in api.device_list() {
            if device_info.vendor_id() == VALVE_VENDOR_ID {
                let pid = device_info.product_id();
                if pid == SC_WIRELESS_PID || pid == SC_WIRED_PID {
                    let device = api
                        .open(VALVE_VENDOR_ID, pid)
                        .map_err(|e| format!("Failed to open device: {}", e))?;

                    let connection_type = if pid == SC_WIRELESS_PID {
                        "Wireless"
                    } else {
                        "Wired"
                    };

                    let info = SteamControllerInfo {
                        connected: true,
                        connection_type: connection_type.to_string(),
                        product_name: device_info
                            .product_string()
                            .unwrap_or("Steam Controller")
                            .to_string(),
                        serial: device_info
                            .serial_number()
                            .unwrap_or("Unknown")
                            .to_string(),
                    };

                    // Store the device
                    let mut device_lock = self.device.lock().unwrap();
                    *device_lock = Some(device);

                    return Ok(info);
                }
            }
        }

        Err("Steam Controller not found".to_string())
    }

    /// Check if currently connected
    pub fn is_connected(&self) -> bool {
        self.device.lock().unwrap().is_some()
    }

    /// Disconnect from the device
    pub fn disconnect(&self) {
        let mut device_lock = self.device.lock().unwrap();
        *device_lock = None;
    }

    /// Get the HID device for reading/writing
    pub fn get_device(&self) -> Arc<Mutex<Option<HidDevice>>> {
        Arc::clone(&self.device)
    }

    /// Read input from the controller (non-blocking)
    pub fn read_input(&self) -> Result<Vec<u8>, String> {
        let device_lock = self.device.lock().unwrap();

        match device_lock.as_ref() {
            Some(device) => {
                let mut buf = vec![0u8; 64];
                match device.read_timeout(&mut buf, 10) {
                    // 10ms timeout
                    Ok(size) => {
                        if size > 0 {
                            buf.truncate(size);
                            Ok(buf)
                        } else {
                            Err("No data available".to_string())
                        }
                    }
                    Err(e) => Err(format!("Read error: {}", e)),
                }
            }
            None => Err("Controller not connected".to_string()),
        }
    }

    /// Read and wait for input (blocking with timeout)
    pub fn read_input_blocking(&self, timeout_ms: i32) -> Result<Vec<u8>, String> {
        let device_lock = self.device.lock().unwrap();

        match device_lock.as_ref() {
            Some(device) => {
                let mut buf = vec![0u8; 64];
                match device.read_timeout(&mut buf, timeout_ms) {
                    Ok(size) => {
                        if size > 0 {
                            buf.truncate(size);
                            Ok(buf)
                        } else {
                            Err("Timeout - no data".to_string())
                        }
                    }
                    Err(e) => Err(format!("Read error: {}", e)),
                }
            }
            None => Err("Controller not connected".to_string()),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_manager_creation() {
        let manager = SteamControllerManager::new();
        assert!(manager.is_ok());
    }

    #[test]
    fn test_detection() {
        let manager = SteamControllerManager::new().unwrap();
        let result = manager.detect();
        // Won't fail even if no controller is present
        println!("Detection result: {:?}", result);
    }
}
