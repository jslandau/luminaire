# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Freshness:** 2026-06-30

## Build & Development Commands

```bash
# Run in development mode (hot reload on Rust changes)
cargo tauri dev

# Build a production bundle (.app on macOS, binary on Linux)
cargo tauri build

# Run unit tests
cd src-tauri && cargo test
```

**Dependencies:** Rust (edition 2021), Tauri v2, Node.js (for Tauri CLI). No Qt dependencies.

**Platforms:** Linux (KDE/Wayland tested) and macOS. On macOS, the build produces a `Luminaire.app` bundle with `LSUIElement=true` (menu-bar/tray app, no Dock icon). On Linux, the binary is the deliverable; a `.desktop` file and SVG icon are provided for installation.

## Architecture

Rust/Tauri v2 application for controlling an Elgato Key Light Neo via its HTTP API. The backend is pure Rust; the frontend is a small HTML/CSS/JS bundle served by Tauri's built-in webview (no bundler required).

### Source Files

#### Backend (`src-tauri/src/`)
- `main.rs` - Entry point, calls `luminaire_lib::run()`
- `lib.rs` - Tauri app setup, window event handling (close-to-tray, theme change), periodic refresh loop, auto-connect on startup
- `keylight.rs` - Async HTTP client using `reqwest`. Communicates with the light at `http://{ip}:9123/elgato/lights`. Defines constants for brightness/temperature ranges, Kelvin↔API conversion, and the `KeyLightClient` with `fetch_state`, `set_power`, `set_brightness`, `set_temperature`, `set_state` methods. Each PUT method parses the response and returns updated `LightState`.
- `config.rs` - Persists settings (IP address, brightness, temperature) to `config.toml` in the platform config directory (`directories` crate). Uses -1 sentinel for unset brightness/temperature.
- `app_state.rs` - State machine: connection status, light on/off, error counter (max 3), brightness, temperature. Provides `AppState` and `AppStateSnapshot` types.
- `commands.rs` - Tauri command bridge: `connect`, `disconnect`, `toggle_power`, `set_brightness`, `set_temperature`, `get_state`. Emits events to frontend: `state-received`, `connection-succeeded`, `error`, `status-update`.
- `icon.rs` - Programmatic tray icon renderer. Draws a lightbulb (glow, bulb, screw base) to an RGBA pixel buffer. Provides `render_lightbulb_icon(on, dark_mode)` and `to_argb32()` for ksni.
- `tray.rs` - Platform-agnostic tray interface. Dispatches to `tray_macos` or `tray_linux` based on `#[cfg(target_os)]`.
- `tray_macos.rs` - macOS tray via Tauri's built-in `TrayIconBuilder`. Left-click toggles power (`show_menu_on_left_click(false)`), right-click opens context menu. Dark-mode detection via `objc` querying `NSApp.effectiveAppearance`. Theme change re-renders the icon.
- `tray_linux.rs` - Linux tray via `ksni` (StatusNotifierItem over D-Bus). Left-click (`activate`) toggles power, middle-click (`secondary_activate`) toggles window visibility, right-click opens context menu. No double-click support (D-Bus limitation).

#### Frontend (`src/`)
- `index.html` - UI layout: IP input + Connect button, status label, power toggle, brightness slider, temperature slider
- `main.js` - Frontend logic: invokes Tauri commands, listens for events, manages drag/edit state for sliders
- `style.css` - Styling for the webview

### Data Files (`src-tauri/`)
- `Info.plist` - macOS bundle Info.plist (sets `LSUIElement=true`, `CFBundleIdentifier=com.luminaire.app`)
- `icons/` - Application icons (32x32, 128x128, 128x128@2x PNGs, .icns, .svg, .ico)
- `luminaire.desktop` - Freedesktop desktop entry for Linux
- `capabilities/default.json` - Tauri v2 capabilities/permissions for the frontend

### Legacy Data (`data/`)
- `data/luminaire.svg` - Original application icon source
- `data/luminaire.desktop` - Original desktop entry
- `data/luminaire.iconset/` - Original source PNGs for icon generation
- `data/luminaire.icns` - Original macOS app icon

**Note:** Settings from the previous C++/Qt version (QSettings) are **not migrated**. The Rust version uses TOML-based config in the platform config directory. Existing users will need to re-enter their IP address on first launch.

### Features
- **Power Toggle:** Single button shows ON (green) / OFF (red), click to toggle
- **System Tray:**
  - Minimizes to tray on close, lightbulb icon reflects light state (lit when on, gray when off)
  - Left-click tray icon toggles light power on/off
  - Middle-click tray icon shows/hides window (Linux only; macOS uses menu item)
  - Right-click context menu provides "Light On", "Light Off", "Show/Hide Window", and "Quit" (macOS) / "Exit" (Linux)
  - Menu items update dynamically based on connection state
  - macOS: tray icon adapts to system dark/light appearance and refreshes on appearance changes
  - Linux: double-click is not available (ksni/D-Bus limitation); middle-click covers the same action
- **Auto-connect:** Reconnects automatically on startup if IP address is saved
- **Settings Restore:** Brightness/temperature restored to last-used values on connect
- **Periodic Refresh:** Polls light state every 5 seconds to stay in sync with external changes
- **Error Resilience:** Tracks consecutive errors (max 3) before disconnecting, prevents refresh timer from hammering failed connections
- **Direct Edit:** Double-click brightness/temperature labels to type exact values
- **Keyboard:** Enter in IP field triggers connect
- **Platform Behavior:**
  - macOS: app starts minimized (tray-only) by default (LSUIElement, no Dock icon)
  - Linux: shows window by default; `--minimized` flag hides it
  - Close-to-tray on both platforms

### Constants (`src-tauri/src/keylight.rs`)
- **Brightness:** MIN_BRIGHTNESS=0, MAX_BRIGHTNESS=100
- **Temperature (Kelvin):** MIN_KELVIN=2900, MAX_KELVIN=7000
- **Temperature (API values):** MIN_API_TEMP=143 (7000K), MAX_API_TEMP=344 (2900K)
- **Error Handling:** MAX_CONSECUTIVE_ERRORS=3 (`src-tauri/src/app_state.rs`)

### Elgato API
GET/PUT to `/elgato/lights` with JSON body:
```json
{"numberOfLights":1,"lights":[{"on":0|1,"brightness":0-100,"temperature":143-344}]}
```
Temperature mapping: 143=7000K, 344=2900K (inverse relationship). Use `kelvin_to_api()` and `api_to_kelvin()` conversion functions.
