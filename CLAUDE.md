# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Build & Development Commands

```bash
# Configure and build
cmake -B build
cmake --build build

# Run the application
./build/luminaire

# Install system-wide (requires sudo)
sudo cmake --install build
```

**Dependencies:** Qt6 (Widgets, Network)

## Architecture

C++17/Qt6 application for controlling an Elgato Key Light Neo via its HTTP API.

### Source Files
- `src/main.cpp` - Entry point, creates QApplication and MainWindow
- `src/MainWindow.h/.cpp` - Main UI with IP config, power toggle, brightness/temperature sliders, system tray integration. Includes KDE Plasma/Wayland compatibility for window management.
- `src/KeyLightAPI.h/.cpp` - Async HTTP client using QNetworkAccessManager. Communicates with light at `http://{ip}:9123/elgato/lights`. Defines constants for brightness/temperature ranges.
- `src/Config.h/.cpp` - Persists settings (IP address, brightness, temperature) via QSettings with explicit sync() calls

### Data Files
- `data/luminaire.desktop` - Freedesktop desktop entry for app menu integration
- `data/luminaire.svg` - Application icon (lightbulb)

### Features
- **Power Toggle:** Single button shows ON (green) / OFF (red), click to toggle
- **System Tray:**
  - Minimizes to tray on close, lightbulb icon reflects light state (lit when on, gray when off)
  - Single-click tray icon toggles light power on/off
  - Middle-click (or double-click) tray icon shows/hides window
  - Right-click context menu provides "Show/Hide Window", "Light On/Off", and "Exit" actions
  - Menu items update dynamically based on state
- **Auto-connect:** Reconnects automatically on startup if IP address is saved
- **Settings Restore:** Brightness/temperature restored to last-used values on connect
- **Periodic Refresh:** Polls light state every 5 seconds to stay in sync with external changes
- **Error Resilience:** Tracks consecutive errors (max 3) before disconnecting, prevents refresh timer from hammering failed connections
- **Direct Edit:** Double-click brightness/temperature labels to type exact values
- **Keyboard:** Enter in IP field triggers connect
- **Wayland Compatibility:** Enhanced window show/hide behavior for KDE Plasma on Wayland

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
