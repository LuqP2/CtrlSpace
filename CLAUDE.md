# CLAUDE.md - CtrlSpace Developer Guide for AI Assistants

> **Last Updated:** 2025-11-22
> **Version:** 0.1.0
> **Purpose:** Comprehensive guide for AI assistants working with the CtrlSpace codebase

---

## Project Overview

**CtrlSpace** is a desktop application for Steam Controller and Steam Deck input remapping with planned on-screen keyboard functionality. It provides real-time visualization and debugging of controller state through a Tauri-based desktop application.

### Key Capabilities
- Automatic Steam Controller/Deck detection (Vendor ID: 0x28de)
- Raw HID input reading and parsing
- Real-time input visualization (~33Hz polling)
- Lizard Mode control (enable/disable default mouse/keyboard emulation)
- Visual debugging of all controller inputs (buttons, triggers, trackpads, gyro)

### Current Status
- **Implemented:** Device detection, connection management, real-time input parsing, visual debugging
- **Planned:** Input remapping, profile management, on-screen keyboard, persistent configuration

---

## Architecture Overview

### Technology Stack

#### Backend (Rust)
- **Framework:** Tauri 1.6 (Desktop app framework)
- **Language:** Rust Edition 2021 (min version 1.60)
- **Hardware Access:** hidapi 2.6.3 (Cross-platform HID device communication)
- **Serialization:** serde 1.0 + serde_json 1.0 (IPC data serialization)
- **State Management:** once_cell 1.19 (Global singleton pattern)

#### Frontend (TypeScript/React)
- **UI Framework:** React 18.2.0 with TypeScript 5.2.2
- **Build Tool:** Vite 5.2.0 (Fast development and optimized production builds)
- **Styling:** TailwindCSS 3.4.1 (Utility-first CSS)
- **State Management:** Zustand 4.5.0 (Lightweight state management)
- **IPC:** @tauri-apps/api 1.5.3 (Rust ↔ TypeScript communication)
- **Linting:** ESLint 8.57.0 with TypeScript plugins

### Communication Flow
```
Frontend (React)
    ↓ Tauri invoke()
Backend (Rust Commands)
    ↓ SteamControllerManager
HID Device Layer (hidapi)
    ↓ USB HID Protocol
Steam Controller Hardware
```

---

## Directory Structure

```
CtrlSpace/
├── src/                              # Rust backend source
│   ├── main.rs                       # Tauri application entry point
│   └── src_tauri/                    # Backend modules
│       ├── mod.rs                    # Module declarations
│       ├── commands.rs               # Tauri IPC command handlers (10 commands)
│       ├── steam_controller.rs       # Hardware abstraction layer (286 lines)
│       ├── input_parser.rs           # HID report parsing logic (321 lines)
│       └── devices.rs                # Device utilities (placeholder)
│
├── store/                            # Frontend state management
│   └── store.ts                      # Zustand store (profile management)
│
├── styles/                           # CSS styling
│   └── index.css                     # Global styles + Tailwind directives
│
├── App.tsx                           # Main React component (392 lines)
├── main.tsx                          # React entry point
├── index.html                        # HTML template
│
├── vite.config.ts                    # Vite build configuration
├── tsconfig.json                     # TypeScript main configuration
├── tsconfig.node.json                # TypeScript Node.js configuration
├── tailwind.config.js                # TailwindCSS configuration
├── postcss.config.js                 # PostCSS configuration
│
├── Cargo.toml                        # Rust dependencies and metadata
├── Cargo.lock                        # Rust dependency lockfile
├── build.rs                          # Rust build script (Tauri setup)
│
├── package.json                      # Node.js dependencies and scripts
├── package-lock.json                 # Node.js dependency lockfile
├── tauri.conf.json                   # Tauri app configuration
│
├── README.md                         # User-facing documentation
├── CLAUDE.md                         # This file (AI assistant guide)
│
├── icons/                            # Application icons
│   └── icon.ico                      # Windows application icon
│
└── .github/                          # GitHub configuration
    └── copilot-instructions.md       # GitHub Copilot instructions
```

---

## Key Files and Their Purposes

### Rust Backend

#### `src/main.rs` - Application Entry Point
- Configures Tauri application builder
- Registers all IPC command handlers
- Sets up main application window
- **Windows subsystem:** Configured to hide console window in production

#### `src/src_tauri/commands.rs` - IPC Command Handlers (155 lines)
**Commands exposed to frontend:**

| Command | Purpose | Return Type |
|---------|---------|-------------|
| `greet(name: String)` | Test command | `String` |
| `list_devices()` | List all HID devices on system | `Vec<DeviceInfo>` |
| `list_steam_controller_interfaces()` | List Valve HID interfaces (0x28de) | `Vec<DeviceInfo>` |
| `ping()` | Connection test | `String` |
| `detect_steam_controller()` | Auto-detect Steam Controller | `Option<SteamControllerInfo>` |
| `connect_steam_controller()` | Initialize connection | `Result<SteamControllerInfo>` |
| `disconnect_steam_controller()` | Close connection | `Result<String>` |
| `is_steam_controller_connected()` | Check connection status | `bool` |
| `read_controller_input()` | Parse and return controller state | `Result<ControllerInput>` |
| `read_raw_input_debug()` | Return raw HID data as hex string | `Result<String>` |

**Global State:**
- `SC_MANAGER`: `Lazy<Arc<Mutex<Option<SteamControllerManager>>>>` - Thread-safe singleton
- Initialized on first access using `once_cell::sync::Lazy`

#### `src/src_tauri/steam_controller.rs` - Hardware Abstraction (286 lines)

**Constants:**
```rust
VALVE_VENDOR_ID: u16 = 0x28de        // Valve Corporation USB vendor ID
SC_WIRELESS_PID: u16 = 0x1142        // Wireless dongle product ID
SC_WIRED_PID: u16 = 0x1102           // Wired/Steam Deck product ID
VENDOR_USAGE_PAGE: u16 = 0xFF00      // Vendor-specific HID interface
```

**SteamControllerManager Core Methods:**
- `detect()` - Enumerate devices, find Valve hardware, filter to vendor-specific interface
- `connect()` - Open HID device, disable Lizard Mode
- `disconnect()` - Re-enable Lizard Mode, close device
- `is_connected()` - Boolean connection state
- `disable_lizard_mode()` - Send feature reports (0x81, 0x87) to disable mouse emulation
- `enable_lizard_mode()` - Send feature report (0x81) to restore default behavior
- `read_input()` - Non-blocking read with 10ms timeout
- `read_input_blocking(timeout_ms)` - Blocking read with custom timeout

**Critical Implementation Details:**
- Uses vendor-specific interface (usage_page=0xFF00) to avoid OS mouse/keyboard interfaces
- Lizard Mode MUST be disabled for raw input; MUST be re-enabled on disconnect
- Thread-safe using `Arc<Mutex<HidDevice>>`
- Includes unit tests for manager creation and detection

#### `src/src_tauri/input_parser.rs` - HID Protocol Parser (321 lines)

**Data Structures:**
```rust
ControllerInput {
    timestamp: u32,        // 32-bit counter from device
    buttons: ButtonState,
    triggers: TriggersData,
    stick: StickData,
    left_trackpad: TrackpadData,
    right_trackpad: TrackpadData,
    gyro: GyroData,
}

ButtonState {
    a, b, x, y: bool,
    lb, rb, lt, rt: bool,
    left_grip, right_grip: bool,
    start, select, steam: bool,
    left_pad_click, right_pad_click: bool,
    stick_click: bool,
}
```

**HID Report Format (64 bytes, reverse-engineered):**
| Bytes | Description | Data Type |
|-------|-------------|-----------|
| 0 | Report ID (always 0x01) | u8 |
| 1 | Sequence number | u8 |
| 2-3 | Button/state flags | u16 LE |
| 4-7 | Timestamp | u32 LE |
| 8 | Trigger press flags (0x01=RT, 0x02=LT) | u8 |
| 10 | Trackpad flags (0x08=L touch, 0x10=R touch, 0x04=click) | u8 |
| 12-13 | Right/Left trigger analog | u8 each |
| 16-17 | Left trackpad/stick X | i16 LE |
| 18-19 | Left trackpad/stick Y | i16 LE |
| 20-21 | Right trackpad X | i16 LE |
| 22-23 | Right trackpad Y | i16 LE |
| 48-49 | Gyro pitch | i16 LE |
| 50-51 | Gyro yaw | i16 LE |
| 52-53 | Gyro roll | i16 LE |

**Parser Function:**
- `parse_input_report(data: &[u8])` - Main parsing logic
- Validates report size (must be 64 bytes)
- Extracts all fields using bit masks and byte offsets
- Returns `Result<ControllerInput, String>`
- Includes unit tests for empty reports, invalid sizes, button parsing

**IMPORTANT:** Button mappings are partially reverse-engineered. Some button positions have TODO comments indicating incomplete mapping.

### Frontend

#### `App.tsx` - Main React Component (392 lines)

**State Management:**
```typescript
const [controllerInfo, setControllerInfo] = useState<any>(null)
const [isConnected, setIsConnected] = useState(false)
const [error, setError] = useState<string | null>(null)
const [input, setInput] = useState<any>(null)
const [isPolling, setIsPolling] = useState(false)
```

**Effects:**
1. **Connection Status Polling (1000ms interval):**
   - Continuously checks `is_steam_controller_connected()`
   - Updates connection status and re-detects if disconnected

2. **Input Polling (30ms interval when connected):**
   - Calls `read_controller_input()` at ~33Hz
   - Updates input state for real-time visualization
   - Only active when `isConnected && isPolling`

**Command Handlers:**
- `detectController()` - Invokes `detect_steam_controller`
- `connectController()` - Invokes `connect_steam_controller`
- `disconnectController()` - Invokes `disconnect_steam_controller`
- `testRawInput()` - Debug command for raw HID data
- `listInterfaces()` - Lists all Valve HID interfaces

**UI Components:**
- **Connection Status Panel:** LED indicator + controller info
- **Action Buttons:** Detect, Connect, Disconnect, Debug commands
- **Input Visualization Panels:**
  1. Buttons (16-button grid)
  2. Analog Triggers (progress bars)
  3. Analog Stick (2D visualizer)
  4. Left Trackpad (2D position + touch indicator)
  5. Right Trackpad (2D position + touch indicator)
  6. Gyroscope (raw pitch/yaw/roll values)

**Helper Components:**
- `ButtonIndicator`: Colored circle (green=pressed, gray=released)
- `ProgressBar`: Horizontal bar for analog values (0-255)
- `StickVisualizer`: 2D crosshair on grid (range ±32768)
- `TrackpadVisualizer`: Conditional rendering when touch detected

**Styling:** TailwindCSS utility classes, dark mode (gray-900 background)

#### `store/store.ts` - Zustand State Store (15 lines)

**Current Implementation:**
```typescript
interface AppState {
  profiles: any[]
  currentProfile: any
  setProfiles: (profiles: any[]) => void
  setCurrentProfile: (profile: any) => void
}
```

**Status:** Minimal implementation prepared for future profile management. Currently not actively used in App.tsx.

**TODO:** Replace `any` types with proper interfaces for Profile data.

### Configuration Files

#### `tauri.conf.json` - Tauri Configuration
- **Build:** Runs `npm run build` before production, `npm run dev` before dev
- **DevPath:** `http://localhost:1420` (Vite dev server)
- **DistDir:** `../dist` (production build output)
- **Window:** 1200x800px (min 800x600), titled "CtrlSpace - Steam Controller Manager"
- **Allowlist:** Restricted (only dialog.message, dialog.ask, shell.open)
- **Bundle ID:** `com.ctrlspace.app`
- **CSP:** Disabled (null) - Consider enabling for security

#### `vite.config.ts` - Vite Build Configuration
- **Dev Server:** Port 1420 (strict mode - fails if port unavailable)
- **Environment Variables:** `VITE_*` and `TAURI_*` prefixes
- **Build Target:** ES2021
- **Minification:** esbuild in production, disabled in debug
- **Sourcemaps:** Generated for debug builds
- **clearScreen:** false (prevents obscuring Rust compiler errors)

#### `Cargo.toml` - Rust Project Configuration
```toml
[package]
name = "ctrlspace"
version = "0.1.0"
edition = "2021"
rust-version = "1.60"

[dependencies]
tauri = { version = "1.6", features = ["shell-open"] }
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
hidapi = "2.6.3"
once_cell = "1.19"

[build-dependencies]
tauri-build = { version = "1.5", features = [] }
```

**Key Dependencies:**
- `tauri` - Desktop app framework
- `hidapi` - Cross-platform HID access (critical for hardware)
- `serde` - Serialization for IPC
- `once_cell` - Global state initialization

---

## Development Workflows

### Initial Setup

1. **Prerequisites:**
   - Node.js (v18+ recommended)
   - Rust (1.60+)
   - **Windows:** Visual Studio Build Tools with C++ for MSVC linker
   - **Linux:** libusb, libudev development headers
   - **macOS:** Xcode Command Line Tools

2. **Frontend Setup:**
   ```bash
   cd /home/user/CtrlSpace
   npm install
   ```

3. **Backend Setup:**
   ```bash
   cargo build
   ```

### Development Mode

**Two-terminal workflow:**

**Terminal 1 - Frontend Dev Server:**
```bash
npm run dev
```
- Starts Vite on http://localhost:1420
- Hot module replacement (HMR) enabled
- Auto-reloads on TypeScript/React changes

**Terminal 2 - Tauri Dev Server:**
```bash
cargo tauri dev
```
- Compiles Rust backend
- Launches desktop window
- Watches Rust files for changes (requires restart)
- Opens Chromium DevTools for debugging

**Alternative (Single Command):**
```bash
npm run tauri:dev
```
- Runs both frontend and backend together

### Production Build

```bash
npm run build          # TypeScript check + Vite build
cargo tauri build      # Rust compilation + bundling
```

**Output Location:**
- **Linux:** `src-tauri/target/release/bundle/`
- **Windows:** `src-tauri/target/release/bundle/msi/` or `.exe`
- **macOS:** `src-tauri/target/release/bundle/dmg/` or `.app`

### Testing

**Rust Unit Tests:**
```bash
cargo test
```
- Runs tests in `steam_controller.rs` and `input_parser.rs`
- Currently 5 unit tests (manager creation, detection, parser edge cases)

**Frontend Testing:**
- **No test framework configured** (Jest/Vitest not installed)
- Manual testing via UI
- Consider adding testing library for future work

**Linting:**
```bash
npm run lint
```
- ESLint with TypeScript rules
- Max warnings: 0 (strict mode)

### Debugging

**Frontend:**
- Open DevTools in Tauri window (Ctrl+Shift+I / Cmd+Option+I)
- Console logs show Tauri invoke errors
- React DevTools available

**Rust:**
- Use `println!` macros with emoji prefixes:
  - ✅ Success messages
  - ❌ Error messages
  - 🔍 Debug/info messages
- Check terminal running `cargo tauri dev` for output
- Use `read_raw_input_debug` command for HID hex dumps

**HID Protocol Debugging:**
1. Connect controller
2. Click "Test Raw Input" button
3. Examine hex output in console
4. Compare with byte offsets in `input_parser.rs` documentation

---

## Code Conventions and Patterns

### Rust Conventions

**Naming:**
- `snake_case` for functions, variables, modules
- `PascalCase` for structs, enums, traits
- `SCREAMING_SNAKE_CASE` for constants

**Error Handling:**
```rust
// Command functions return Result<T, String>
#[tauri::command]
fn example_command() -> Result<SomeData, String> {
    some_operation()
        .map_err(|e| format!("Failed: {}", e))?;
    Ok(result)
}
```

**Shared State Pattern:**
```rust
use once_cell::sync::Lazy;
use std::sync::{Arc, Mutex};

static GLOBAL_STATE: Lazy<Arc<Mutex<Option<Manager>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

// In commands:
let mut manager = GLOBAL_STATE.lock().unwrap();
```

**Serialization:**
```rust
#[derive(Debug, Serialize, Deserialize)]
struct DataStruct {
    field: Type,
}
```

### TypeScript/React Conventions

**Naming:**
- `camelCase` for functions, variables
- `PascalCase` for components, interfaces
- `UPPER_CASE` for constants

**Tauri Invocation:**
```typescript
import { invoke } from '@tauri-apps/api/tauri'

// Strongly typed invocation
const result = await invoke<ReturnType>('command_name', {
  arg: value
})

// Error handling
try {
  const data = await invoke<Data>('command')
  setData(data)
} catch (err) {
  setError(String(err))
}
```

**State Management:**
```typescript
// Local state for UI
const [state, setState] = useState<Type>(initialValue)

// Global state for cross-component data (future)
import { useStore } from './store/store'
const profiles = useStore(state => state.profiles)
```

**Component Structure:**
```typescript
function Component() {
  // State declarations
  const [state, setState] = useState()

  // Effects
  useEffect(() => {
    // Effect logic
    return () => cleanup()
  }, [dependencies])

  // Handlers
  const handleAction = async () => {
    // Handler logic
  }

  // Render
  return (
    // JSX
  )
}
```

### Git Commit Conventions

**Semantic Prefixes:**
- `feat:` - New features
- `fix:` - Bug fixes
- `perf:` - Performance improvements
- `refactor:` - Code restructuring without behavior change
- `test:` - Adding/updating tests
- `docs:` - Documentation changes
- `chore:` - Build process, dependencies, tooling
- `debug:` - Temporary debugging commits

**Examples from History:**
```
feat: enable real-time visual mode with corrected parser
fix: disable auto-polling to prevent crash on connect
perf: reduce latency for real-time input
test: try reading data without disabling Lizard Mode
```

---

## Common Development Tasks

### Adding a New Tauri Command

1. **Define command in `src/src_tauri/commands.rs`:**
```rust
#[tauri::command]
fn new_command(arg: String) -> Result<ReturnType, String> {
    // Implementation
    Ok(result)
}
```

2. **Register in `src/main.rs`:**
```rust
fn main() {
    tauri::Builder::default()
        .invoke_handler(tauri::generate_handler![
            // ... existing commands,
            new_command
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
```

3. **Call from frontend:**
```typescript
const result = await invoke<ReturnType>('new_command', {
  arg: value
})
```

### Modifying the HID Parser

**CRITICAL:** The HID protocol is reverse-engineered. Changes require careful validation.

1. **Locate parser in `src/src_tauri/input_parser.rs`**
2. **Reference the byte offset documentation** (lines 1-40)
3. **Test changes with `read_raw_input_debug` command**
4. **Add unit test for new parsing logic:**
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_new_field_parsing() {
        let data = vec![/* test data */];
        let result = parse_input_report(&data);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().new_field, expected_value);
    }
}
```
5. **Run `cargo test` before committing**

### Adding UI Visualization Component

1. **Create component in `App.tsx`:**
```typescript
function NewVisualizer({ data }: { data: DataType }) {
  return (
    <div className="p-4 bg-gray-800 rounded">
      {/* Component JSX */}
    </div>
  )
}
```

2. **Add to main render:**
```typescript
{input && (
  <NewVisualizer data={input.newField} />
)}
```

3. **Style with TailwindCSS classes** (avoid custom CSS)

### Implementing Profile Management

**Current state:** Zustand store structure exists but unused.

**Implementation steps:**
1. **Define Profile interface in `store/store.ts`:**
```typescript
interface Profile {
  id: string
  name: string
  mappings: ButtonMapping[]
  // ... other fields
}
```

2. **Add Tauri commands for persistence:**
```rust
#[tauri::command]
fn save_profile(profile: Profile) -> Result<(), String> {
    // Save to JSON file
}

#[tauri::command]
fn load_profiles() -> Result<Vec<Profile>, String> {
    // Load from JSON files
}
```

3. **Update Zustand store types** (replace `any`)
4. **Add UI for profile selection/editing**

---

## Critical Domain Knowledge

### Lizard Mode

**What is it:**
- Steam Controller's default firmware behavior
- Emulates mouse + keyboard for basic functionality
- Left trackpad = mouse, triggers = clicks, buttons = keyboard keys

**Why disable it:**
- Prevents interference with raw input reading
- Allows full control over all inputs
- Required for custom remapping

**How it works:**
```rust
// Disable (on connect):
// Send feature report: [0x81]
// Send feature report: [0x87, 0x03, 0x08, 0x07, 0x00]

// Enable (on disconnect):
// Send feature report: [0x81]
```

**CRITICAL:** Always re-enable Lizard Mode on disconnect! Otherwise, controller may become unusable for normal Steam operation.

### Polling Strategy

**Current implementation:** 30ms interval = ~33Hz

**Rationale:**
- Balance between responsiveness and CPU usage
- Matches typical game input polling rates
- HID read timeout: 10ms (non-blocking)

**Alternatives considered:**
- Faster (10ms): Higher CPU usage, minimal benefit
- Slower (60ms): Noticeable input lag
- Event-driven: Not supported by hidapi (HID is poll-based)

**Performance notes:**
- Recent commit (perf: reduce latency) optimized polling loop
- Frontend polling uses `useEffect` cleanup to prevent memory leaks

### HID Protocol Reverse Engineering

**Status:** Partially documented, ongoing discovery.

**Known:**
- Report structure (64 bytes)
- Major input fields (buttons, triggers, trackpads, gyro)
- Timestamp counter
- Touch detection flags

**Unknown/TODO:**
- Some button bit positions (see TODO comments in parser)
- Haptic feedback commands
- LED control
- Battery status (if applicable)
- Additional sensor data

**Validation method:**
1. Use `read_raw_input_debug` to capture hex data
2. Press specific buttons/move specific inputs
3. Identify changed bytes
4. Update parser with new mappings
5. Add unit tests

**Reference resources:**
- Valve's open-source Steam Controller driver (Linux kernel)
- Community reverse engineering docs (Reddit, GitHub)
- HID descriptor parsing (if accessible via `hidapi`)

### Thread Safety in Tauri

**Problem:** Tauri commands execute on multiple threads simultaneously.

**Solution:** Use `Arc<Mutex<T>>` for shared state.

**Example:**
```rust
static MANAGER: Lazy<Arc<Mutex<Option<SteamControllerManager>>>> =
    Lazy::new(|| Arc::new(Mutex::new(None)));

#[tauri::command]
fn command() -> Result<(), String> {
    let mut manager = MANAGER.lock()
        .map_err(|_| "Lock failed")?;
    // Mutex held, safe to mutate
}
```

**Avoid:**
- Holding locks across `await` points (not applicable in sync commands)
- Nested locks (deadlock risk)
- Long-running operations while holding lock (blocks other commands)

### Cross-Platform Considerations

**hidapi behavior differences:**
- **Windows:** May require admin privileges for some devices
- **Linux:** Requires udev rules for non-root access
- **macOS:** Permissions dialog on first access

**Recommended udev rule (Linux):**
```
# /etc/udev/rules.d/99-steam-controller.rules
SUBSYSTEM=="hidraw", ATTRS{idVendor}=="28de", MODE="0666"
SUBSYSTEM=="usb", ATTRS{idVendor}=="28de", MODE="0666"
```

**Testing checklist:**
- Test on all target platforms
- Verify permissions handling
- Check USB hotplug behavior
- Validate path separators in file operations (if adding persistence)

---

## Testing Strategy

### Current Test Coverage

**Rust (5 unit tests):**
- `steam_controller::tests::test_manager_creation` - Verifies manager instantiation
- `steam_controller::tests::test_detection` - Tests device enumeration
- `input_parser::tests::test_parse_empty_report` - Validates zero-filled report
- `input_parser::tests::test_parse_invalid_size` - Checks error handling for wrong size
- `input_parser::tests::test_parse_buttons` - Validates button bit extraction

**Frontend:**
- No automated tests (Jest/Vitest not configured)
- Manual testing via UI

### Recommended Testing Additions

**Rust:**
1. **Command tests:** Mock Tauri context, test all IPC commands
2. **Parser tests:** Add tests for all known input combinations
3. **Integration tests:** Test full connect → read → disconnect flow
4. **Error handling:** Test device disconnection mid-read

**Frontend:**
1. **Install testing library:**
   ```bash
   npm install -D @testing-library/react @testing-library/jest-dom vitest
   ```
2. **Component tests:** Visualizer components with mock data
3. **Hook tests:** Custom hooks for polling logic
4. **Integration tests:** Tauri command invocations with mocks

**Manual Testing Checklist:**
- [ ] Device detection with controller connected
- [ ] Device detection with controller disconnected
- [ ] Connect while powered off (should fail gracefully)
- [ ] Disconnect while reading (should not crash)
- [ ] Real-time input for all 16 buttons
- [ ] Analog trigger full range (0-255)
- [ ] Trackpad touch detection and positioning
- [ ] Gyroscope movement
- [ ] USB hotplug (connect/disconnect during app runtime)
- [ ] Multiple connection attempts
- [ ] Lizard Mode re-enabled after disconnect

---

## Troubleshooting Guide

### Common Issues

**1. "Device not detected" but controller is connected**

**Causes:**
- Controller not on vendor-specific interface
- Wrong product ID (check if wireless vs wired)
- Permissions issue (Linux/macOS)

**Debug steps:**
```typescript
// In frontend:
await invoke('list_devices')  // Check if device appears at all
await invoke('list_steam_controller_interfaces')  // Verify interface filtering
```

**Fix:**
- Linux: Install udev rules, replug device
- Windows: Try running as administrator
- Check `VALVE_VENDOR_ID` and PID constants match your device

---

**2. "Failed to read input" or immediate disconnection**

**Causes:**
- Lizard Mode not properly disabled
- Wrong HID interface opened
- Device firmware in unexpected state

**Debug steps:**
```typescript
// Check raw data:
await invoke('read_raw_input_debug')
```

**Fix:**
- Disconnect and reconnect controller
- Restart Steam (if running)
- Power cycle controller (hold Steam button + Select for 10s)

---

**3. Input values frozen or incorrect**

**Causes:**
- Parser byte offset error
- Report format changed (firmware update?)
- Endianness issue

**Debug steps:**
1. Capture raw hex with `read_raw_input_debug`
2. Press one button at a time, note which bytes change
3. Compare with `input_parser.rs` byte offset documentation
4. Update parser if mismatch found

---

**4. Frontend not connecting to backend**

**Causes:**
- Vite dev server not running on port 1420
- Tauri build error (silent failure)

**Debug steps:**
```bash
# Check Vite:
curl http://localhost:1420

# Check Rust compilation:
cargo build
```

**Fix:**
- Ensure both `npm run dev` and `cargo tauri dev` are running
- Check terminal for Rust compiler errors
- Restart both processes

---

**5. "Lock failed" errors in commands**

**Causes:**
- Mutex poisoned (panic while holding lock)
- Deadlock (rare with current architecture)

**Debug steps:**
- Check Rust terminal for panic messages
- Restart application

**Fix:**
- Identify panic source, add error handling
- Consider using `try_lock()` for non-critical operations

---

### Performance Issues

**Symptom:** High CPU usage during polling

**Solutions:**
- Increase polling interval (currently 30ms)
- Implement adaptive polling (faster when input changing, slower when idle)
- Move parsing to Rust (already done)
- Debounce frontend updates

**Symptom:** UI lag or stuttering

**Solutions:**
- Use `React.memo()` for visualizer components
- Batch state updates
- Profile with React DevTools Performance tab
- Consider Web Workers for heavy computation (if adding remapping logic)

---

## Future Development Roadmap

### Planned Features (from README)

1. **Input Remapping**
   - Define button → key mappings
   - Trackpad → mouse configuration
   - Trigger → analog input mapping
   - Multi-button combinations

2. **On-Screen Keyboard**
   - Trackpad navigation
   - Button-based text input
   - Word prediction (optional)

3. **Profile Management**
   - Save/load profiles (JSON format)
   - Profile switching via UI or hotkey
   - Per-application profiles (auto-switch)

4. **Persistent Configuration**
   - Settings storage (probably JSON in app data directory)
   - Remember last connected device
   - UI preferences (theme, layout)

5. **Advanced Features**
   - Haptic feedback control
   - LED customization (if supported)
   - Macro recording
   - Dead zone configuration
   - Sensitivity curves

### Technical Debt

1. **Type Safety:**
   - Replace `any` types in `store.ts`
   - Add TypeScript interfaces for all IPC data structures
   - Generate TypeScript types from Rust structs (consider `ts-rs` crate)

2. **Error Handling:**
   - Consistent error types (use `thiserror` crate)
   - User-friendly error messages in UI
   - Error logging/reporting

3. **Testing:**
   - Set up frontend test framework
   - Increase Rust test coverage to >80%
   - Add integration tests
   - CI/CD pipeline with automated testing

4. **Documentation:**
   - API documentation for Rust (cargo doc)
   - Component documentation for React (Storybook?)
   - User manual
   - Video tutorials

5. **Code Quality:**
   - Enable more Clippy lints
   - Set up pre-commit hooks
   - Code coverage reporting
   - Performance benchmarking

6. **Security:**
   - Enable CSP in `tauri.conf.json`
   - Review allowlist permissions
   - Input validation for all commands
   - Secure profile storage (encrypt if contains sensitive data)

---

## Working with This Codebase as an AI Assistant

### Before Making Changes

1. **Read relevant files first:**
   - Never modify code you haven't read
   - Use Read tool before Edit tool
   - Understand context before proposing changes

2. **Check existing patterns:**
   - Follow established naming conventions
   - Match existing error handling style
   - Use same state management patterns

3. **Validate assumptions:**
   - Test commands work before modifying
   - Verify HID protocol assumptions with raw data
   - Check git history for context on recent changes

### When Adding Features

1. **Start with planning:**
   - Use TodoWrite to break down tasks
   - Identify affected files
   - Consider impact on existing functionality

2. **Incremental implementation:**
   - Smallest possible changes first
   - Test after each step
   - Commit working states frequently

3. **Testing requirements:**
   - Add unit tests for new Rust code
   - Manual test UI changes
   - Update this CLAUDE.md if architecture changes

### When Fixing Bugs

1. **Reproduce first:**
   - Understand the failure case
   - Check if issue exists in current code
   - Identify root cause before fixing

2. **Minimal changes:**
   - Fix only what's broken
   - Avoid "drive-by refactoring"
   - Don't add features while fixing bugs

3. **Verification:**
   - Test fix resolves issue
   - Ensure no regressions
   - Add test to prevent future regression

### Communication with Users

1. **Be specific about file locations:**
   - Use `file_path:line_number` format
   - Example: "Button parsing happens in `src/src_tauri/input_parser.rs:156`"

2. **Explain technical decisions:**
   - Why this approach vs alternatives
   - Trade-offs considered
   - Potential future issues

3. **Ask for clarification when needed:**
   - HID protocol details are reverse-engineered (incomplete)
   - Profile format not yet defined
   - UI/UX preferences vary

### Don't Do These Things

❌ **Don't add unnecessary abstraction** - Keep it simple
❌ **Don't over-engineer** - Solve current problem, not hypothetical futures
❌ **Don't break Lizard Mode handling** - Critical for device usability
❌ **Don't commit without testing** - Especially HID protocol changes
❌ **Don't ignore Rust compiler warnings** - They're usually right
❌ **Don't use placeholder types (`any`)** - Define proper interfaces
❌ **Don't hardcode values** - Use constants with clear names
❌ **Don't disable security features** - CSP should be enabled eventually

### Do These Things

✅ **Do read existing code first**
✅ **Do follow established patterns**
✅ **Do add tests for new functionality**
✅ **Do update documentation**
✅ **Do use semantic commit messages**
✅ **Do validate HID changes with raw data**
✅ **Do handle errors gracefully**
✅ **Do clean up old TODO comments when completing tasks**

---

## Quick Reference

### File Paths for Common Tasks

| Task | Primary Files |
|------|---------------|
| Add/modify Tauri command | `src/src_tauri/commands.rs`, `src/main.rs` |
| Fix input parsing | `src/src_tauri/input_parser.rs` |
| Hardware interaction | `src/src_tauri/steam_controller.rs` |
| UI changes | `App.tsx` |
| Add visualizer component | `App.tsx` (inline components) |
| State management | `store/store.ts` |
| Styling | `styles/index.css`, `tailwind.config.js` |
| Build configuration | `vite.config.ts`, `Cargo.toml`, `tauri.conf.json` |
| Dependencies | `package.json`, `Cargo.toml` |

### Command Cheatsheet

```bash
# Development
npm install                  # Install frontend dependencies
cargo build                  # Build Rust backend
npm run dev                  # Start Vite dev server
cargo tauri dev              # Run Tauri in dev mode
npm run tauri:dev            # Run both (may not work on all systems)

# Production
npm run build                # Build frontend
cargo tauri build            # Build production app

# Testing
cargo test                   # Run Rust unit tests
npm run lint                 # Lint TypeScript/React

# Debugging
cargo check                  # Fast compile check
cargo clippy                 # Rust linter
npm run preview              # Preview production build
```

### Key Constants

```rust
// Rust (src/src_tauri/steam_controller.rs)
VALVE_VENDOR_ID: 0x28de      // Valve USB vendor ID
SC_WIRELESS_PID: 0x1142      // Wireless dongle
SC_WIRED_PID: 0x1102         // Wired controller
VENDOR_USAGE_PAGE: 0xFF00    // Vendor-specific interface

// Frontend (App.tsx)
CONNECTION_POLL_INTERVAL: 1000ms
INPUT_POLL_INTERVAL: 30ms
```

### Important Byte Offsets (HID Report)

| Offset | Field | Type |
|--------|-------|------|
| 0 | Report ID | u8 (always 0x01) |
| 1 | Sequence | u8 |
| 2-3 | Button flags | u16 LE |
| 4-7 | Timestamp | u32 LE |
| 12-13 | Triggers (R, L) | u8 each |
| 16-19 | Left pad/stick (X, Y) | i16 LE each |
| 20-23 | Right pad (X, Y) | i16 LE each |
| 48-53 | Gyro (pitch, yaw, roll) | i16 LE each |

---

## Conclusion

This guide provides comprehensive context for AI assistants working with CtrlSpace. The codebase is relatively small and focused, making it approachable for modifications. Key complexity areas are:

1. **HID Protocol:** Reverse-engineered, incomplete documentation
2. **Thread Safety:** Proper Mutex usage critical for stability
3. **Lizard Mode:** Must be handled correctly to avoid breaking controller
4. **Real-time Performance:** Polling strategy affects responsiveness

When in doubt, prioritize:
- Stability over features
- Simplicity over abstraction
- Testing over speed
- Documentation over cleverness

Good luck! 🎮

---

**Document Maintenance:**
- Update this file when adding new major features
- Document new HID protocol discoveries
- Keep command reference current
- Add troubleshooting entries for common issues
- Update roadmap as features complete

**Last reviewed:** 2025-11-22
**Next review:** After profile management implementation or major architecture changes
