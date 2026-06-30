# Luminaire

Control your Elgato Key Light Neo from your desktop.

## What it does

Luminaire is a Rust/Tauri v2 desktop app that controls an Elgato Key Light Neo over its local HTTP API. It runs in your system tray (Linux) or menu bar (macOS) and lets you toggle the light on/off, adjust brightness (0-100), and set color temperature (2900K-7000K warm to cool).

**Key features:**
- Click the tray/menu-bar icon to toggle power
- Show/hide the control window via the tray context menu ("Show Window"/"Hide Window"), or on Linux via middle-click
- Auto-reconnect on startup when you've saved an IP address
- Restores your last brightness and temperature settings
- Direct edit: double-click any value label to type an exact number

## Requirements

- Rust (edition 2021)
- Tauri v2 (CLI: `cargo install tauri-cli` if not already installed)
- Node.js (required by the Tauri CLI tooling)
- An Elgato Key Light Neo on your local network

## Build and install

```bash
# Development mode (hot reload on Rust changes)
cargo tauri dev

# Production build (.app bundle on macOS, binary + .desktop on Linux)
cargo tauri build

# Unit tests
cd src-tauri && cargo test
```

### Linux

`cargo tauri build` produces distributable bundles (deb/rpm/AppImage). For manual installation, the committed `src-tauri/luminaire.desktop` and `src-tauri/icons/icon.svg` are the install sources — the `.desktop` file references the icon by the name `luminaire` (`Icon=luminaire`), not by filename.

### macOS

`cargo tauri build` produces `Luminaire.app` at `src-tauri/target/release/bundle/macos/Luminaire.app`.

> **Note:** `cargo install --path .` (or `cargo build --release`) produces only the raw executable and **does not** create the `.app` bundle. Use `cargo tauri build` to get the bundle.

The app runs as a menu-bar app (`LSUIElement=true`): no Dock icon, no Cmd+Tab entry. To install it, copy `Luminaire.app` to `/Applications`.

Since the app is unsigned, Gatekeeper will block it on first launch. Right-click the app and choose **Open**, or run it from the terminal, then confirm the prompt.

## Using Luminaire

1. Find your light's IP address (check your router or the Elgato Control Center app)
2. Enter the IP address and click Connect
3. Use the sliders to adjust brightness and temperature
4. Click the power button to toggle the light on/off

Luminaire saves your settings automatically. The next time you launch it, it'll reconnect to your light and restore your previous brightness and temperature. Note: existing users upgrading from a previous release will need to re-enter their light's IP address on first launch — the Rust version uses TOML config and does not migrate settings from the earlier version.

### Startup behavior

- **macOS:** starts minimized — tray-only by default. There is no Dock icon.
- **Linux:** shows the control window by default. Pass `--minimized` to start tray-only.
- On both platforms, closing the window hides to the tray (close-to-tray); the app keeps running.

### System tray / menu bar

When you close the window, Luminaire stays running in the background. The icon changes to show the light's state:
- Lit bulb: light is on
- Gray bulb: light is off

#### Linux (system tray)

| Gesture | Action |
|---------|--------|
| Single-click (left) | Toggle light on/off |
| Middle-click | Show/hide control window |
| Right-click | Context menu |

Double-click is **not** available on Linux (ksni/D-Bus limitation); middle-click covers window show/hide.

Context menu items: `Light On`, `Light Off`, `Show Window`/`Hide Window` (label updates with window state), `Exit`. `Light On`/`Light Off` are disabled when not connected.

#### macOS (menu bar)

| Gesture | Action |
|---------|--------|
| Left-click | Toggle light on/off |
| Right-click | Context menu |

Middle-click is not a standard macOS menu-bar gesture and is not wired up. There is no double-click support. Show/hide the window via the context-menu `Show Window`/`Hide Window` item.

Context menu items: `Light On`, `Light Off`, `Show Window`/`Hide Window` (label updates), `Quit`. `Light On`/`Light Off` are disabled when not connected.

The menu-bar icon adapts to macOS dark/light appearance and refreshes on appearance change.

## Troubleshooting

**Can't connect?**
- Verify the IP address (no `http://`, just the numbers like `192.168.1.100`)
- Check that your computer and light are on the same network
- Try pinging the IP address: `ping 192.168.1.100`

**Controls aren't responding?**
- The light polls its state every 5 seconds. Wait a moment if you changed settings from another device.
- After 3 consecutive errors, Luminaire disconnects automatically to prevent hammering a failed connection. Reconnect manually.

## Technical details

Luminaire uses the Elgato HTTP API at `http://{ip}:9123/elgato/lights`. It sends GET requests to read state and PUT requests with JSON payloads to change settings.

The API uses an inverse temperature scale: API value 143 = 7000K (cool), 344 = 2900K (warm). Luminaire handles the conversion so you work with Kelvin values directly.
