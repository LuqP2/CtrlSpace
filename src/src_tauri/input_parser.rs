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
/// Empirically determined format based on actual USB wired controller data:
/// - Byte 0: Report ID (0x01)
/// - Byte 1: Sequence number
/// - Bytes 2-3: Button/state flags
/// - Bytes 4-7: Timestamp (32-bit LE)
/// - Byte 8: Trigger press flags (0x01=RT, 0x02=LT)
/// - Byte 10: Trackpad flags (0x08=L touch, 0x10=R touch, 0x04=click)
/// - Byte 12: Right trigger analog (0-255)
/// - Byte 13: Left trigger analog (0-255)
/// - Bytes 16-19: Left trackpad X,Y (16-bit LE) OR Stick X,Y when trackpad not touched
/// - Bytes 20-23: Right trackpad X,Y (16-bit LE)
/// - Bytes 48+: Gyroscope/Accelerometer data
pub fn parse_input_report(data: &[u8]) -> Result<ControllerInput, String> {
    if data.len() < 64 {
        return Err(format!("Invalid report size: {} bytes", data.len()));
    }

    // Check report type (should be 0x01 for input reports)
    if data[0] != 0x01 {
        return Err(format!("Invalid report type: 0x{:02x}", data[0]));
    }

    let mut input = ControllerInput::default();

    // Parse buttons (bytes 2-3 contain button state flags)
    // TODO: Need more test data to map individual buttons correctly
    let btn_low = data[2];
    let btn_high = data[3];

    // Byte 8: Trigger press detection
    let trigger_flags = data[8];
    input.buttons.rt = (trigger_flags & 0x01) != 0;
    input.buttons.lt = (trigger_flags & 0x02) != 0;

    // Byte 10: Trackpad touch/click detection
    let trackpad_flags = data[10];
    let lpad_touched = (trackpad_flags & 0x08) != 0;
    let rpad_touched = (trackpad_flags & 0x10) != 0;
    let rpad_clicked = (trackpad_flags & 0x14) == 0x14; // 0x10 | 0x04

    input.buttons.lpad_click = (trackpad_flags & 0x08) != 0 && lpad_touched;
    input.buttons.rpad_click = rpad_clicked;

    // Parse analog triggers (bytes 12-13)
    // Note: Resting values are around 0xe0-0xff, not 0x00!
    input.triggers.right = data[12];
    input.triggers.left = data[13];

    // Parse stick OR left trackpad (bytes 16-19: X,Y as 16-bit LE)
    // When trackpad L is touched (flag 0x08), these are trackpad coordinates
    // Otherwise, these appear to be stick coordinates
    let x1619 = i16::from_le_bytes([data[16], data[17]]);
    let y1619 = i16::from_le_bytes([data[18], data[19]]);

    if lpad_touched {
        input.left_trackpad = TrackpadData {
            x: x1619,
            y: y1619,
            active: true,
        };
        input.stick = StickData { x: 0, y: 0 }; // Stick not used when trackpad active
    } else {
        // When trackpad not touched, these bytes seem to contain stick data
        input.stick.x = x1619;
        input.stick.y = y1619;
        input.left_trackpad = TrackpadData::default();
    }

    // Parse right trackpad (bytes 20-23: X,Y as 16-bit LE)
    let rpad_x = i16::from_le_bytes([data[20], data[21]]);
    let rpad_y = i16::from_le_bytes([data[22], data[23]]);
    input.right_trackpad = TrackpadData {
        x: rpad_x,
        y: rpad_y,
        active: rpad_touched,
    };

    // Parse gyroscope data (bytes 48-55: empirically observed to change with movement)
    if data.len() >= 56 {
        input.gyro.pitch = i16::from_le_bytes([data[48], data[49]]);
        input.gyro.yaw = i16::from_le_bytes([data[50], data[51]]);
        input.gyro.roll = i16::from_le_bytes([data[52], data[53]]);
    }

    // Parse timestamp (bytes 4-7 as u32 LE)
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
