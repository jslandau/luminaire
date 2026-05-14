# Human Test Plan: macOS Support

Generated from implementation plan: `docs/implementation-plans/2026-05-14-macos-support/`
HEAD at time of generation: `ba58e88`

## Prerequisites

- macOS 13+ host (Apple Silicon or Intel) with Qt6 installed via Homebrew (`brew install qt6`)
- A separate Linux host with Qt6 (for H14 only)
- A physically reachable Elgato Key Light Neo on the same LAN (for H6, H8, H13)
- Working tree at HEAD `ba58e88`
- Build succeeds: `cd /Users/jlandau/git/luminaire && cmake -B build && cmake --build build` exits 0
- Artifacts present after build:
  - `test -d build/luminaire.app/Contents/MacOS`
  - `test -f build/luminaire.app/Contents/Info.plist`
  - `grep -A1 LSUIElement build/luminaire.app/Contents/Info.plist` shows `<true/>`

## Phase 1: Bundle and Launch (macOS)

| Step | Action | Expected |
|------|--------|----------|
| 1 | In Finder, open `/Users/jlandau/git/luminaire/build/`, double-click `luminaire.app` | App launches. If Gatekeeper blocks ("unidentified developer"), right-click → Open → Open; this is expected for the unsigned local build. (H1 / AC1.2) |
| 2 | Quit the app. Drag `luminaire.app` from `build/` onto `/Applications` in Finder. Launch `/Applications/luminaire.app` | App runs from `/Applications`. (H2 / AC1.3) |
| 3 | With app running, observe the Dock | No Luminaire icon in Dock. (H3 / AC1.4) |
| 4 | With app running, press Cmd+Tab | Luminaire does not appear in the application switcher. (H4 / AC1.5) |

## Phase 2: Tray Icon and Interactions (macOS)

| Step | Action | Expected |
|------|--------|----------|
| 5 | Look at the macOS menu bar (top-right) | A lightbulb icon is visible. (H5 / AC2.1) |
| 6 | Open MainWindow, enter the Key Light IP, click Connect; close window. Single-left-click the menu-bar lightbulb | Physical light toggles; menu-bar icon changes color (yellow when on, gray when off). Repeat once to confirm round-trip. (H6 / AC2.2) |
| 7 | Right-click the menu-bar icon | Popup menu shows, in order: "Light On", "Light Off", separator, "Show Window", separator, "Quit". No "Exit" label. (H7 / AC2.3) |
| 8 | Right-click → "Show Window" | MainWindow appears with IP address field, brightness slider, and temperature slider populated. (H8 / AC2.4) |
| 9 | Right-click → "Quit" | App exits, menu-bar icon disappears. Verify in Terminal: `pgrep -x luminaire` returns nothing (exit code 1). (H9 / AC2.5) |
| 10 | Relaunch `luminaire.app`. Before configuring/connecting, left-click the menu-bar icon | Nothing happens, no crash; `pgrep -x luminaire` still shows the process. (H10 / AC2.6) |

## Phase 3: Dark/Light Mode Adaptation (macOS)

| Step | Action | Expected |
|------|--------|----------|
| 11 | System Settings → Appearance → Dark. Ensure light is off (or not connected) | Menu-bar icon is clearly visible light gray (not blending into dark menu bar). (H11 / AC3.1) |
| 12 | With app running, toggle Appearance: Dark → Light → Dark | Menu-bar icon redraws within ~1 second on each toggle without restarting the app. (H12 / AC3.2) |
| 13 | Click "Light On" (or turn the connected light on). Toggle Appearance between Light and Dark | Yellow lit bulb is clearly visible in both modes. (H13 / AC3.3) |

## Phase 4: Linux Regression (Linux host)

| Step | Action | Expected |
|------|--------|----------|
| 14a | On Linux host: `cmake -B build && cmake --build build && ls build/luminaire build/luminaire.app 2>&1` | `build/luminaire` exists; `build/luminaire.app` does not (ls error). (AC4.1) |
| 14b | `cmake --install build --prefix /tmp/lum-install && test -f /tmp/lum-install/share/applications/luminaire.desktop && test -f /tmp/lum-install/share/icons/hicolor/scalable/apps/luminaire.svg && echo OK` | Prints `OK`. (AC4.2) |
| 14c | Run `./build/luminaire`. Middle-click the tray icon. Then double-click the tray icon. Right-click the tray icon | Each of middle-click and double-click toggles the MainWindow show/hide. Right-click menu contains "Exit" (not "Quit"). (H14 / AC4.3) |

## End-to-End: macOS daily-driver scenario

Purpose: Validates the integrated experience a real user would have on macOS after a fresh build — bundle install, persistent config, tray-driven control, and appearance adaptation.

1. Build, drag to `/Applications`, launch from `/Applications`. Confirm no Dock icon, not in Cmd+Tab.
2. Tray right-click → "Show Window" → enter Key Light IP → Connect. Close window (does not quit; persists to tray).
3. Tray left-click toggles the physical light; icon color tracks the light state.
4. Quit app via tray "Quit". Relaunch from `/Applications`. App auto-connects (per `Config` persistence) and the MainWindow brightness/temperature reflect the previous session.
5. Toggle macOS Appearance to Dark; confirm the off-state and on-state menu-bar icons both remain clearly visible.
6. Tray right-click → "Quit"; confirm `pgrep -x luminaire` is empty.

## Traceability

| Acceptance Criterion | Automated Check | Manual Step |
|----------------------|-----------------|-------------|
| AC1.1 .app bundle | `CMakeLists.txt:30-37` + post-build `test -d build/luminaire.app/Contents/MacOS` | Prerequisites |
| AC1.2 Finder launch | — | Phase 1, Step 1 |
| AC1.3 Drag to /Applications | — | Phase 1, Step 2 |
| AC1.4 No Dock icon | `data/Info.plist.in` `LSUIElement=true` | Phase 1, Step 3 |
| AC1.5 No Cmd+Tab | `data/Info.plist.in` `LSUIElement=true` | Phase 1, Step 4 |
| AC2.1 Tray icon present | — | Phase 2, Step 5 |
| AC2.2 Left-click toggles | — | Phase 2, Step 6 |
| AC2.3 Right-click menu labels | `src/MainWindow.cpp:211-219` | Phase 2, Step 7 |
| AC2.4 Show Window opens MainWindow | — | Phase 2, Step 8 |
| AC2.5 Quit exits | — | Phase 2, Step 9 |
| AC2.6 No-op when disconnected | `src/MainWindow.cpp:317` `if (m_connected)` | Phase 2, Step 10 |
| AC3.1 Dark-mode off-icon | `src/MainWindow.cpp:256-258` `QColor(200, 200, 200)` | Phase 3, Step 11 |
| AC3.2 Live appearance update | `src/MainWindow.cpp:235-239` + `changeEvent` line 347 | Phase 3, Step 12 |
| AC3.3 Lit icon both modes | — | Phase 3, Step 13 |
| AC4.1 Linux plain exe | `CMakeLists.txt:30` (MACOSX_BUNDLE only under APPLE) | Phase 4, Step 14a |
| AC4.2 Linux install rules | `CMakeLists.txt:42-55` (install rules under `if(NOT APPLE)`) | Phase 4, Step 14b |
| AC4.3 Linux MiddleClick | `src/MainWindow.cpp:324-325` `#ifndef Q_OS_MACOS` | Phase 4, Step 14c |
| AC5.1 .icns structure | `data/luminaire.icns` (494 KB) + 10 PNGs in `data/luminaire.iconset/` | E2E Step 5 |
| AC5.2 Assets committed | `git ls-files data/luminaire.icns data/luminaire.iconset/` = 11 | Prerequisites |
