# CtrlSpace

A desktop app for Steam Controller / Steam Deck input remapping with on-screen keyboard.

## Features

- Automatic device detection (Steam Controller, Deck)
- Basic button mapping to keys
- Mouse emulation from trackpad
- On-Screen Keyboard
- Profiles saved in JSON
- Persistent config

## Tech Stack

- Tauri (Rust backend)
- React + TypeScript (frontend)
- TailwindCSS + shadcn/ui
- Zustand

## Development

### Prerequisites

- Node.js
- Rust
- On Windows: Visual Studio Build Tools with C++ for MSVC linker

### Setup

1. Install dependencies:
   ```
   cd src/frontend
   npm install
   ```

2. Build Rust:
   ```
   cargo build
   ```

3. Run dev:
   ```
   cd src/frontend
   npm run dev
   ```
   In another terminal:
   ```
   cargo tauri dev
   ```

## Build

```
cd src/frontend
npm run build
cargo tauri build
```