# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

**Freshness:** 2026-05-14

## Build & Development Commands

```bash
# Configure and build
cmake -B build
cmake --build build

# Run the application (Linux)
./build/luminaire

# Run the application (macOS)
open ./build/luminaire.app

# Install system-wide (Linux only, requires sudo)
sudo cmake --install build
```

**Dependencies:** Qt6 (Widgets, Network)

**Platforms:** Linux (KDE/Wayland tested) and macOS. On macOS, the build produces a `luminaire.app` bundle with embedded `Info.plist` (from `data/Info.plist.in`) and `LSUIElement=true` (menu-bar/tray app, no Dock icon). `cmake --install` is Linux-only; on macOS the `.app` bundle is the deliverable.

## Architecture

C++17/Qt6 application for controlling an Elgato Key Light Neo via its HTTP API.

### Source Files
- `src/main.cpp` - Entry point, creates QApplication and MainWindow
- `src/MainWindow.h/.cpp` - Main UI with IP config, power toggle, brightness/temperature sliders, system tray integration. Includes KDE Plasma/Wayland compatibility for window management (Linux) and dark-mode-aware tray icon (macOS). Platform-specific behavior is guarded by `Q_OS_LINUX` / `Q_OS_MACOS`. Overrides `changeEvent` to refresh the tray icon on macOS palette/appearance changes.
- `src/KeyLightAPI.h/.cpp` - Async HTTP client using QNetworkAccessManager. Communicates with light at `http://{ip}:9123/elgato/lights`. Defines constants for brightness/temperature ranges.
- `src/Config.h/.cpp` - Persists settings (IP address, brightness, temperature) via QSettings with explicit sync() calls

### Data Files
- `data/luminaire.desktop` - Freedesktop desktop entry for app menu integration (Linux)
- `data/luminaire.svg` - Application icon (lightbulb, Linux)
- `data/Info.plist.in` - macOS bundle Info.plist template (configured by CMake via `MACOSX_BUNDLE_*` properties; sets `LSUIElement=true`)
- `data/luminaire.icns` - macOS app icon (built from `data/luminaire.iconset/`)
- `data/luminaire.iconset/` - Source PNGs (16/32/128/256/512 @1x and @2x) used to generate the `.icns`

### Features
- **Power Toggle:** Single button shows ON (green) / OFF (red), click to toggle
- **System Tray:**
  - Minimizes to tray on close, lightbulb icon reflects light state (lit when on, gray when off)
  - Single-click tray icon toggles light power on/off
  - Double-click tray icon shows/hides window (all platforms); middle-click also shows/hides on Linux (not wired on macOS, where middle-click is not a standard menu-bar gesture)
  - Right-click context menu provides "Show/Hide Window", "Light On/Off", and "Exit" (Linux) / "Quit" (macOS) actions
  - Menu items update dynamically based on state
  - macOS: tray icon adapts to system dark/light appearance and refreshes on appearance changes (via `QStyleHints::colorSchemeChanged` and `QEvent::PaletteChange`)
- **Auto-connect:** Reconnects automatically on startup if IP address is saved
- **Settings Restore:** Brightness/temperature restored to last-used values on connect
- **Periodic Refresh:** Polls light state every 5 seconds to stay in sync with external changes
- **Error Resilience:** Tracks consecutive errors (max 3) before disconnecting, prevents refresh timer from hammering failed connections
- **Direct Edit:** Double-click brightness/temperature labels to type exact values
- **Keyboard:** Enter in IP field triggers connect
- **Wayland Compatibility:** Enhanced window show/hide behavior for KDE Plasma on Wayland (Linux-only code path, guarded by `Q_OS_LINUX`)

### Constants (KeyLightAPI.h)
- **Brightness:** MIN_BRIGHTNESS=0, MAX_BRIGHTNESS=100
- **Temperature (Kelvin):** MIN_KELVIN=2900, MAX_KELVIN=7000
- **Temperature (API values):** MIN_API_TEMP=143 (7000K), MAX_API_TEMP=344 (2900K)
- **Error Handling:** MAX_CONSECUTIVE_ERRORS=3 (MainWindow.h)

### Elgato API
GET/PUT to `/elgato/lights` with JSON body:
```json
{"numberOfLights":1,"lights":[{"on":0|1,"brightness":0-100,"temperature":143-344}]}
```
Temperature mapping: 143=7000K, 344=2900K (inverse relationship). Use `kelvinToApi()` and `apiToKelvin()` conversion functions.
