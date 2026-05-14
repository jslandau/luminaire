# Luminaire

Control your Elgato Key Light Neo from your desktop.

## What it does

Luminaire provides a simple Qt6 GUI to control your Elgato Key Light Neo. It runs in your system tray (Linux) or menu bar (macOS) and lets you toggle the light on/off, adjust brightness (0-100), and set color temperature (2900K-7000K warm to cool).

**Key features:**
- Click the tray/menu-bar icon to toggle power
- Double-click (or middle-click on Linux) to show/hide the control window
- Auto-reconnect on startup when you've saved an IP address
- Restores your last brightness and temperature settings
- Direct edit: double-click any value label to type an exact number

## Requirements

- Qt6 (Widgets, Network)
- CMake 3.16+
- C++17 compiler
- Elgato Key Light Neo on your local network

## Build and install

### Linux

```bash
cmake -B build
cmake --build build

# Run directly
./build/luminaire

# Install system-wide (optional)
sudo cmake --install build
```

After installing system-wide, Luminaire appears in your application menu under Utilities.

### macOS

```bash
cmake -B build
cmake --build build

# Run the app bundle
open ./build/luminaire.app
```

Luminaire runs as a menu-bar app (`LSUIElement=true`) with no Dock icon. There is no system-wide install target on macOS; copy `luminaire.app` to `/Applications` if you want it in Launchpad.

## Using Luminaire

1. Find your light's IP address (check your router or the Elgato Control Center app)
2. Enter the IP address and click Connect
3. Use the sliders to adjust brightness and temperature
4. Click the power button to toggle the light on/off

Luminaire saves your settings automatically. The next time you launch it, it'll reconnect to your light and restore your previous brightness and temperature.

### System tray / menu bar

When you close the window, Luminaire stays running in the background. The icon changes to show the light's state:
- Lit bulb: light is on
- Gray bulb: light is off

#### Linux (system tray)

| Gesture | Action |
|---------|--------|
| Single-click | Toggle light on/off |
| Double-click | Show/hide control window |
| Middle-click | Show/hide control window |
| Right-click | Context menu |

Context menu items: Show/Hide Window, Light On, Light Off, Exit.

#### macOS (menu bar)

| Gesture | Action |
|---------|--------|
| Left-click | Toggle light on/off |
| Double-click | Show/hide control window |
| Right-click | Context menu |

Middle-click is not a standard macOS menu-bar gesture and is not wired up.

Context menu items: Show/Hide Window, Light On, Light Off, Quit.

The menu-bar icon adapts automatically to macOS light/dark appearance.

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
