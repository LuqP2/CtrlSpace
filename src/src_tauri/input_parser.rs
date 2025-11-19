use serde::Serialize;
use std::fmt;

/// Button bit flags for Steam Controller
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Buttons {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub lb: bool,         // Left bumper
    pub rb: bool,         // Right bumper
    pub lt: bool,         // Left trigger click
    pub rt: bool,         // Right trigger click
    pub lgrip: bool,      // Left grip
    pub rgrip: bool,      // Right grip
    pub start: bool,
    pub select: bool,     // Back button
    pub steam: bool,      // Steam button
    pub lpad_click: bool, // Left trackpad click
    pub rpad_click: bool, // Right trackpad click
    pub stick_click: bool,
}

impl Default for Buttons {
    fn default() -> Self {
        Self {
            a: false,
            b: false,
            x: false,
            y: false,
            lb: false,
            rb: false,
            lt: false,
            rt: false,
            lgrip: false,
            rgrip: false,
            start: false,
            select: false,
            steam: false,
            lpad_click: false,
            rpad_click: false,
            stick_click: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TrackpadData {
    pub x: i16,
    pub y: i16,
    pub active: bool,
}

impl Default for TrackpadData {
    fn default() -> Self {
        Self {
            x: 0,
            y: 0,
            active: false,
        }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct StickData {
    pub x: i16,
    pub y: i16,
}

impl Default for StickData {
    fn default() -> Self {
        Self { x: 0, y: 0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct TriggersData {
    pub left: u8,
    pub right: u8,
}

impl Default for TriggersData {
    fn default() -> Self {
        Self { left: 0, right: 0 }
    }
}

#[derive(Debug, Clone, Copy, Serialize)]
pub struct GyroData {
    pub pitch: i16,
    pub yaw: i16,
    pub roll: i16,
}

impl Default for GyroData {
    fn default() -> Self {
        Self {
            pitch: 0,
            yaw: 0,
            roll: 0,
        }
    }
}

/// Complete input state from Steam Controller
#[derive(Debug, Clone, Serialize)]
pub struct ControllerInput {
    pub buttons: ButtonState,
    pub left_trackpad: TrackpadData,
    pub right_trackpad: TrackpadData,
    pub stick: StickData,
    pub triggers: TriggersData,
    pub gyro: GyroData,
    pub timestamp: u32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ButtonState {
    pub a: bool,
    pub b: bool,
    pub x: bool,
    pub y: bool,
    pub lb: bool,
    pub rb: bool,
    pub lt: bool,
    pub rt: bool,
    pub lgrip: bool,
    pub rgrip: bool,
    pub start: bool,
    pub select: bool,
    pub steam: bool,
    pub lpad_click: bool,
    pub rpad_click: bool,
    pub stick_click: bool,
}

impl Default for ControllerInput {
    fn default() -> Self {
        Self {
            buttons: ButtonState::default(),
            left_trackpad: TrackpadData::default(),
            right_trackpad: TrackpadData::default(),
            stick: StickData::default(),
            triggers: TriggersData::default(),
            gyro: GyroData::default(),
            timestamp: 0,
        }
    }
}

impl Default for ButtonState {
    fn default() -> Self {
        Self {
            a: false,
            b: false,
            x: false,
            y: false,
            lb: false,
            rb: false,
            lt: false,
            rt: false,
            lgrip: false,
            rgrip: false,
            start: false,
            select: false,
            steam: false,
            lpad_click: false,
            rpad_click: false,
            stick_click: false,
        }
    }
}

impl fmt::Display for ControllerInput {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "Input[buttons={:?}, lpad=({},{}), rpad=({},{}), stick=({},{}), triggers=({},{})]",
            self.buttons,
            self.left_trackpad.x,
            self.left_trackpad.y,
            self.right_trackpad.x,
            self.right_trackpad.y,
            self.stick.x,
            self.stick.y,
            self.triggers.left,
            self.triggers.right
        )
    }
}

/// Parse HID report from Steam Controller
///
/// The Steam Controller sends 64-byte HID reports with the following structure:
/// - Bytes 0-1: Report type
/// - Bytes 2-9: Button states (bit flags)
/// - Bytes 10-11: Left trigger analog value
/// - Bytes 12-13: Right trigger analog value
/// - Bytes 14-17: Left trackpad X, Y coordinates
/// - Bytes 18-21: Right trackpad X, Y coordinates
/// - Bytes 22-25: Stick X, Y coordinates
/// - Bytes 26-31: Gyroscope data (pitch, yaw, roll)
/// - Remaining: Timestamp and other data
pub fn parse_input_report(data: &[u8]) -> Result<ControllerInput, String> {
    if data.len() < 64 {
        return Err(format!("Invalid report size: {} bytes", data.len()));
    }

    // Check report type (should be 0x01 for input reports)
    if data[0] != 0x01 {
        return Err(format!("Invalid report type: 0x{:02x}", data[0]));
    }

    let mut input = ControllerInput::default();

    // Parse buttons (bytes 8-10 contain button flags)
    // Button layout based on Steam Controller protocol
    let btn1 = data[8];
    let btn2 = data[9];
    let btn3 = data[10];

    input.buttons.rt = (btn1 & 0x01) != 0;
    input.buttons.lt = (btn1 & 0x02) != 0;
    input.buttons.rb = (btn1 & 0x04) != 0;
    input.buttons.lb = (btn1 & 0x08) != 0;
    input.buttons.y = (btn1 & 0x10) != 0;
    input.buttons.b = (btn1 & 0x20) != 0;
    input.buttons.x = (btn1 & 0x40) != 0;
    input.buttons.a = (btn1 & 0x80) != 0;

    input.buttons.lpad_click = (btn2 & 0x02) != 0;
    input.buttons.rpad_click = (btn2 & 0x04) != 0;
    input.buttons.stick_click = (btn2 & 0x40) != 0;

    input.buttons.lgrip = (btn3 & 0x01) != 0;
    input.buttons.rgrip = (btn3 & 0x02) != 0;
    input.buttons.start = (btn3 & 0x04) != 0;
    input.buttons.steam = (btn3 & 0x08) != 0;
    input.buttons.select = (btn3 & 0x10) != 0;

    // Parse analog triggers (bytes 11-12)
    input.triggers.left = data[11];
    input.triggers.right = data[12];

    // Parse stick position (bytes 16-19: X as i16, Y as i16)
    input.stick.x = i16::from_le_bytes([data[16], data[17]]);
    input.stick.y = i16::from_le_bytes([data[18], data[19]]);

    // Parse left trackpad (bytes 20-25)
    let lpad_x = i16::from_le_bytes([data[20], data[21]]);
    let lpad_y = i16::from_le_bytes([data[22], data[23]]);
    input.left_trackpad = TrackpadData {
        x: lpad_x,
        y: lpad_y,
        active: lpad_x != 0 || lpad_y != 0, // Simple touch detection
    };

    // Parse right trackpad (bytes 26-31)
    let rpad_x = i16::from_le_bytes([data[26], data[27]]);
    let rpad_y = i16::from_le_bytes([data[28], data[29]]);
    input.right_trackpad = TrackpadData {
        x: rpad_x,
        y: rpad_y,
        active: rpad_x != 0 || rpad_y != 0,
    };

    // Parse gyroscope data (bytes 32-37: pitch, yaw, roll as i16)
    if data.len() >= 38 {
        input.gyro.pitch = i16::from_le_bytes([data[32], data[33]]);
        input.gyro.yaw = i16::from_le_bytes([data[34], data[35]]);
        input.gyro.roll = i16::from_le_bytes([data[36], data[37]]);
    }

    // Parse timestamp (bytes 4-7 as u32)
    input.timestamp = u32::from_le_bytes([data[4], data[5], data[6], data[7]]);

    Ok(input)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_empty_report() {
        let mut data = vec![0u8; 64];
        data[0] = 0x01; // Set valid report type

        let result = parse_input_report(&data);
        assert!(result.is_ok());

        let input = result.unwrap();
        assert_eq!(input.stick.x, 0);
        assert_eq!(input.stick.y, 0);
        assert!(!input.buttons.a);
    }

    #[test]
    fn test_invalid_report_size() {
        let data = vec![0u8; 32]; // Too small
        let result = parse_input_report(&data);
        assert!(result.is_err());
    }

    #[test]
    fn test_button_parsing() {
        let mut data = vec![0u8; 64];
        data[0] = 0x01;
        data[8] = 0x80; // A button pressed

        let result = parse_input_report(&data).unwrap();
        assert!(result.buttons.a);
        assert!(!result.buttons.b);
    }
}
