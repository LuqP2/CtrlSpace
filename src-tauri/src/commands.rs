use serde::Serialize;

#[derive(Serialize)]
pub struct DeviceInfo { pub vendor_id: u16, pub product_id: u16, pub product: String }

#[tauri::command]
pub fn list_devices() -> Vec<DeviceInfo> {
    let mut out = vec![];
    if let Ok(api) = hidapi::HidApi::new() {
        for d in api.device_list() {
            out.push(DeviceInfo {
                vendor_id: d.vendor_id(),
                product_id: d.product_id(),
                product: d.product_string().unwrap_or_default(),
            });
        }
    }
    out
}

#[tauri::command]
pub fn ping() -> &'static str { "ok" }
