# macOS Support Design

## Summary

Luminaire is a C++17/Qt6 desktop application for controlling an Elgato Key Light Neo over its local HTTP API. Currently it targets Linux only. This design adds macOS support so the app can be built, installed, and run as a native macOS menu bar utility — living exclusively in the menu bar with no Dock icon, no Cmd+Tab entry, and a context menu for configuration.

The approach is deliberately minimal: no new source files, no new classes, and no third-party dependencies beyond Qt6. Three areas change: the CMake build system gains an `if(APPLE)` block that enables bundle packaging and an `Info.plist` that suppresses the Dock icon; two new data files provide the macOS icon asset and bundle metadata; and four small, `#ifdef`-guarded edits to `MainWindow.cpp` isolate Linux/Wayland behavior and add dark-mode-aware tray icon rendering. The Linux build path is left structurally unchanged.

## Definition of Done

- `cmake -B build && cmake --build build` on macOS produces `build/luminaire.app`, launchable from Finder and drag-installable to /Applications
- The app runs as a menu bar-only app (no Dock icon) using QSystemTrayIcon
- Left-click toggles light on/off; right-click shows a context menu with "Show Config", "Light On/Off", and "Quit"
- Right-click "Show Config" opens the existing MainWindow (IP, brightness, temperature)
- The `.app` bundle includes a hand-crafted `.icns` icon
- The Linux build is unaffected; the single CMakeLists.txt handles both platforms
- Wayland-specific code is guarded with `#ifdef Q_OS_LINUX`
- Auto-start on login is out of scope

## Acceptance Criteria

### macos-support.AC1: cmake --build produces a runnable .app bundle
- **macos-support.AC1.1 Success:** `cmake -B build && cmake --build build` on macOS creates `build/luminaire.app`
- **macos-support.AC1.2 Success:** `luminaire.app` launches by double-clicking in Finder
- **macos-support.AC1.3 Success:** `luminaire.app` can be dragged to `/Applications` and run from there
- **macos-support.AC1.4 Failure:** No Dock icon appears when the app is running
- **macos-support.AC1.5 Failure:** App does not appear in Cmd+Tab switcher

### macos-support.AC2: Menu bar tray icon and interactions
- **macos-support.AC2.1 Success:** Tray icon appears in the macOS menu bar when app is running
- **macos-support.AC2.2 Success:** Left-click on tray icon toggles light on/off (when connected)
- **macos-support.AC2.3 Success:** Right-click on tray icon shows context menu with "Show Config", "Light On", "Light Off", "Quit"
- **macos-support.AC2.4 Success:** "Show Config" opens the MainWindow with IP, brightness, and temperature controls
- **macos-support.AC2.5 Success:** "Quit" exits the app
- **macos-support.AC2.6 Edge:** Left-click when not connected does nothing (no crash)

### macos-support.AC3: Tray icon adapts to dark/light mode
- **macos-support.AC3.1 Success:** Icon is visible (non-faint) when light is off and macOS is in dark mode
- **macos-support.AC3.2 Success:** Icon updates when system appearance changes without restarting the app
- **macos-support.AC3.3 Success:** Yellow/lit icon is visible in both light and dark mode when light is on

### macos-support.AC4: Linux build is unaffected
- **macos-support.AC4.1 Success:** `cmake --build` on Linux still produces a plain executable (not a `.app` bundle)
- **macos-support.AC4.2 Success:** Linux `.desktop` and hicolor SVG install rules still function
- **macos-support.AC4.3 Success:** Linux tray behavior (double-click, middle-click show/hide) is unchanged

### macos-support.AC5: Icon asset quality
- **macos-support.AC5.1 Success:** `.app` bundle icon renders clearly in Finder at small, medium, and large icon sizes
- **macos-support.AC5.2 Success:** `data/luminaire.icns` and `data/luminaire.iconset/` are both committed to the repo

## Glossary

- **`.app` bundle**: A macOS application package — a directory with a `.app` suffix that Finder presents as a single launchable file. Contains the executable, resources, and metadata under a standard folder layout.
- **`Info.plist`**: The macOS bundle metadata file (XML property list). Tells the OS the app's name, identifier, icon filename, and behavioral flags such as `LSUIElement`.
- **`LSUIElement`**: An `Info.plist` key that, when set to `true`, marks an app as an "agent" — it runs without a Dock icon and does not appear in the Cmd+Tab application switcher.
- **`CFBundleIdentifier`**: A reverse-DNS string (e.g. `com.luminaire.app`) that uniquely identifies the app bundle to macOS and its subsystems.
- **`.icns`**: Apple's multi-resolution icon container format. Holds the same icon image at several sizes (16 px through 512 px) and Retina (@2x) variants in a single file.
- **iconset**: A folder named `<name>.iconset/` containing PNG files at the required sizes. `iconutil -c icns` converts the folder into a `.icns` file.
- **`iconutil`**: macOS command-line tool (ships with Xcode) that converts a `.iconset/` folder to `.icns` and vice versa.
- **`rsvg-convert`**: A command-line SVG rasterizer (from the `librsvg` library) used here to export the existing `luminaire.svg` to PNG at each required icon size.
- **`QSystemTrayIcon`**: The Qt class that manages a system tray / menu bar icon. Used on both Linux and macOS; behavior and click semantics differ between platforms.
- **`QEvent::PaletteChange`**: A Qt event delivered to widgets when the system color palette changes — used here to detect macOS dark/light mode switches at runtime.
- **`Q_OS_MAC` / `Q_OS_LINUX`**: Qt preprocessor macros defined on their respective platforms. Used with `#ifdef` to compile platform-specific code paths.
- **`MACOSX_BUNDLE`**: A CMake target property that, when set to `TRUE`, instructs CMake to package the executable inside a `.app` bundle on macOS.
- **`MACOSX_BUNDLE_INFO_PLIST`**: A CMake variable pointing to an `Info.plist.in` template; CMake performs variable substitution and writes the result into the bundle.
- **`MACOSX_PACKAGE_LOCATION`**: A CMake source-file property that specifies where within the `.app` bundle a resource file is copied (e.g. `"Resources"` places it in `Contents/Resources/`).
- **`macdeployqt`**: A Qt tool that copies all required Qt frameworks into a `.app` bundle so the app is self-contained and can run on machines without Qt installed. Not used here (personal-use only).
- **Gatekeeper**: macOS security feature that blocks apps from unidentified developers. On an unsigned app, right-click → Open bypasses the initial block.
- **Wayland**: A Linux display server protocol used on modern Linux desktops (e.g. KDE Plasma). The existing codebase has Wayland-specific window-activation workarounds that are guarded with `#ifdef Q_OS_LINUX`.
- **Retina (@2x)**: Apple's term for high-DPI displays. Icon assets are supplied at double resolution (e.g. `icon_256x256@2x.png` = 512 px) so macOS can use them on Retina screens without upscaling.
- **`NSHighResolutionCapable`**: An `Info.plist` key that opts the app into high-DPI rendering on Retina displays instead of pixel-doubled blurry rendering.
- **Homebrew**: The macOS package manager used to install Qt6 as a development dependency.

## Architecture

Luminaire is a single-process Qt6/C++17 desktop app. No new components are introduced. macOS support is added through build system changes, two new data files, and targeted platform guards in existing source files.

Changes touch four areas:

**CMakeLists.txt** — an `if(APPLE)` block adds `MACOSX_BUNDLE TRUE` to the target, bundles `data/luminaire.icns` as a resource, and sets `MACOSX_BUNDLE_INFO_PLIST` to `data/Info.plist.in`. Linux-only install rules (`.desktop`, hicolor SVG icon) are wrapped in `if(NOT APPLE)`.

**data/Info.plist.in** (new) — macOS bundle metadata: `LSUIElement=true` suppresses the Dock icon and Cmd+Tab entry; `CFBundleIconFile`, `CFBundleName`, `CFBundleIdentifier` (`com.luminaire.app`), `CFBundleShortVersionString`, and `NSHighResolutionCapable=true` are set via CMake variable substitution.

**data/luminaire.icns** (new, hand-crafted) — produced once from `data/luminaire.iconset/` PNG exports of the existing `data/luminaire.svg`. Sizes: 16, 32, 128, 256, 512 px plus @2x Retina variants. Converted with `iconutil -c icns`. Both the iconset folder and `.icns` are committed; the build never runs `iconutil`.

**src/MainWindow.cpp / MainWindow.h** — four targeted changes:
- `showWindow()`: Wayland window-activation block wrapped in `#ifdef Q_OS_LINUX`
- `setupTrayIcon()`: "Exit" label becomes "Quit" on macOS via `#ifdef Q_OS_MAC`
- `onTrayActivated()`: `MiddleClick` branch wrapped in `#ifndef Q_OS_MAC`
- New `changeEvent(QEvent*)`: responds to `QEvent::PaletteChange` by calling `updateTrayIcon(m_lightOn)` to redraw with current theme
- `createLightbulbIcon(bool on)`: when `on == false`, checks dark mode via `qApp->palette().color(QPalette::Window).lightness() < 128` and uses `QColor(200,200,200)` instead of the current dark gray so the off-state icon is visible on a dark menu bar

**src/main.cpp** — no changes.

## Existing Patterns

The codebase has no prior macOS-specific code. All existing platform-specific behavior targets Linux/Wayland via comments but no `#ifdef` guards. This design introduces the first platform guards, following Qt's standard convention (`Q_OS_LINUX`, `Q_OS_MAC`).

CMake structure follows the existing single-file pattern; the `if(APPLE)` block extends it rather than splitting into separate files.

## Implementation Phases

<!-- START_PHASE_1 -->
### Phase 1: CMake and Info.plist
**Goal:** Make `cmake --build` on macOS produce a valid `.app` bundle skeleton.

**Components:**
- `CMakeLists.txt` — `if(APPLE)` block with `MACOSX_BUNDLE TRUE`, `MACOSX_BUNDLE_INFO_PLIST`, icns source file with `MACOSX_PACKAGE_LOCATION "Resources"`, `if(NOT APPLE)` guard on Linux install rules
- `data/Info.plist.in` — new file with `LSUIElement`, `CFBundleIconFile`, `CFBundleName`, `CFBundleIdentifier`, `CFBundleShortVersionString`, `NSHighResolutionCapable`

**Dependencies:** None (first phase); requires Qt6 installed via Homebrew and a placeholder `.icns` file for the build to succeed (even a 1×1 PNG renamed `.icns` works at this stage)

**Done when:** `cmake -B build && cmake --build build` on macOS produces `build/luminaire.app`; app launches from Finder; no Dock icon appears; Cmd+Tab does not show the app
<!-- END_PHASE_1 -->

<!-- START_PHASE_2 -->
### Phase 2: Icon Assets
**Goal:** Produce a production-quality `.icns` for the bundle.

**Components:**
- `data/luminaire.iconset/` — PNG exports of `data/luminaire.svg` at 16, 32, 128, 256, 512 px and @2x variants (10 files total), produced with `rsvg-convert` or Inkscape
- `data/luminaire.icns` — generated once via `iconutil -c icns data/luminaire.iconset -o data/luminaire.icns`, committed to the repo

**Dependencies:** Phase 1 (bundle structure must exist to verify icon appears correctly)

**Done when:** App icon appears correctly in Finder, the menu bar tray icon renders cleanly at menu bar size, and `data/luminaire.icns` is committed alongside `data/luminaire.iconset/`
<!-- END_PHASE_2 -->

<!-- START_PHASE_3 -->
### Phase 3: Source Code Guards and Dark Mode Adaptation
**Goal:** Guard Linux-specific code paths and make the tray icon adapt to macOS dark/light mode.

**Components:**
- `src/MainWindow.cpp` — four changes: `#ifdef Q_OS_LINUX` around Wayland activation block in `showWindow()`; `#ifdef Q_OS_MAC` for "Quit" label in `setupTrayIcon()`; `#ifndef Q_OS_MAC` around `MiddleClick` branch in `onTrayActivated()`; new `changeEvent(QEvent*)` implementation; dark mode branch in `createLightbulbIcon(bool on)`
- `src/MainWindow.h` — `changeEvent(QEvent*)` declaration in `protected` section

**Dependencies:** Phase 1 (app must be running on macOS to verify tray behavior)

**Done when:** App builds and runs cleanly on both macOS and Linux; tray icon is visible in both light and dark mode on macOS; "Quit" appears in the macOS context menu; "Exit" remains on Linux
<!-- END_PHASE_3 -->

## Additional Considerations

**`macdeployqt`:** For personal use, Qt frameworks are found via `DYLD_LIBRARY_PATH` or rpath from the Homebrew Qt installation, so the bare `.app` bundle from `cmake --build` runs on the build machine without running `macdeployqt`. Distributing to another machine would require `macdeployqt` to bundle Qt frameworks inside the `.app` — out of scope here.

**Signing/notarization:** Not required for personal use. The app will trigger Gatekeeper on first launch; right-click → Open bypasses this.
