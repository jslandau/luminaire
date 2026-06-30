# Test Requirements: macOS Support

**Design:** `/Users/jlandau/git/luminaire/docs/design-plans/2026-05-14-macos-support.md`
**Phases:** `phase_01.md`, `phase_02.md`, `phase_03.md`
**Generated:** 2026-05-14

---

## Context and Rationalization

Luminaire is a C++17/Qt6 desktop application with **no existing automated test suite** — no GTest, no QTest, no CI harness, and no test target in `CMakeLists.txt`. The entire codebase is a single executable composed of `MainWindow`, `KeyLightAPI`, and `Config`, all of which are tightly coupled to a live Qt GUI event loop, a running system tray, and an external HTTP device (the Elgato Key Light Neo).

The macOS support work being verified falls into three categories, none of which are amenable to traditional unit testing in this repository:

1. **Build system infrastructure (Phase 1):** Outputs are filesystem artifacts (`luminaire.app/`, `Info.plist`, bundle layout). These can be checked with shell assertions but only on a macOS host.
2. **Binary asset generation (Phase 2):** Outputs are PNG and `.icns` files whose correctness is visual. Byte-level checks (file exists, non-trivial size, valid icns magic) are mechanizable; "renders clearly in Finder" is not.
3. **GUI behavior on a running app (Phase 3):** Tray icon clicks, context-menu labels, dark/light mode palette changes, and Dock/Cmd+Tab suppression all require a windowing session, a logged-in macOS user, and (for some criteria) a physically reachable Key Light on the LAN.

Standing up a Qt test harness, a tray-icon GUI test framework (none of the common ones drive `QSystemTrayIcon` reliably on macOS), and a mock HTTP server would constitute a separate, larger project than the macOS port itself. It is therefore deliberately out of scope. The strategy below mechanizes everything that can be mechanized as **shell-script verification** (executed during the phase tasks themselves, per the existing implementation plans) and routes the remainder to a single, explicit **human verification checklist**.

Where an acceptance criterion is split — partly automatable, partly visual — both rows appear.

---

## AC1: cmake --build produces a runnable .app bundle

| AC | Type | Approach | Test artifact / steps |
|----|------|----------|-----------------------|
| AC1.1 `cmake -B build && cmake --build build` creates `build/luminaire.app` | Automated (integration, shell) | Filesystem assertion after build on macOS host | Phase 1, Task 3, Steps 2–3: run `cmake -B build && cmake --build build`; assert `test -d build/luminaire.app/Contents/MacOS` and `test -f build/luminaire.app/Contents/Info.plist`. Existing in `phase_01.md`. |
| AC1.2 `.app` launches by double-clicking in Finder | Human verification | Cannot be driven from CLI without AppleScript GUI scripting (fragile, requires Accessibility permission). Finder double-click invokes LaunchServices, which exercises the same code path as `open build/luminaire.app` — we use `open` as the mechanizable proxy and double-click as the human confirmation. | Human checklist item H1. |
| AC1.3 `.app` can be dragged to `/Applications` and run from there | Human verification | Drag-and-drop is a Finder UI gesture; `cp -R` is a reasonable mechanizable substitute but does not exercise Finder's quarantine attribute handling. | Human checklist item H2. |
| AC1.4 No Dock icon when running | Automated (integration, shell) | Inspect `Info.plist` for `<key>LSUIElement</key><true/>` after build. `LSUIElement=true` is the *cause*; Dock absence is the *effect*. Asserting the cause is reliable; asserting the effect requires querying Dock's running-apps list (`osascript -e 'tell application "System Events" to get name of every process whose background only is false'`), which is brittle. Use plist assertion as primary; reinforce with human spot-check. | Phase 1, Task 3, Step 3: `grep -A1 LSUIElement build/luminaire.app/Contents/Info.plist` must show `<true/>`. Human checklist item H3 for visual confirmation. |
| AC1.5 App does not appear in Cmd+Tab | Automated (cause) + Human (effect) | Same as AC1.4 — `LSUIElement=true` is the documented cause. | Plist assertion (shared with AC1.4) + Human checklist item H4. |

---

## AC2: Menu bar tray icon and interactions

Every AC2 criterion requires a live `QApplication` event loop, a real `QSystemTrayIcon` instance attached to the macOS menu bar, and (for AC2.2 / AC2.4) a connected Elgato Key Light. Qt does not expose a programmatic way to synthesize `QSystemTrayIcon::activated` events from outside the process on macOS, and macOS's menu bar extras are not in the standard accessibility tree, so AppleScript / `cliclick` cannot reliably click them. All AC2 criteria are therefore human-verified.

| AC | Type | Approach | Test artifact / steps |
|----|------|----------|-----------------------|
| AC2.1 Tray icon appears in menu bar | Human verification | macOS menu-bar extras are not scriptable. | Human checklist H5. |
| AC2.2 Left-click toggles light on/off when connected | Human verification | Requires real Key Light on LAN; tray click is unsynthesizable. | Human checklist H6. |
| AC2.3 Right-click shows context menu with "Show Config", "Light On", "Light Off", "Quit" | Mostly human; partial automated | Menu text is set in code at `src/MainWindow.cpp` `setupTrayIcon()`; a `grep` check ensures the labels exist in source. Visual menu rendering is human. **Discrepancy note:** the design says "Show Config" but the implementation in `phase_03.md` uses the existing label "Show Window". Resolve before sign-off (treat current behavior as the answer; update design if intentional). | Automated: `grep -q '"Quit"' src/MainWindow.cpp && grep -q '"Light On"' src/MainWindow.cpp && grep -q '"Light Off"' src/MainWindow.cpp`. Human checklist H7. |
| AC2.4 "Show Config" opens MainWindow with IP/brightness/temperature | Human verification | Tray menu interaction. | Human checklist H8. |
| AC2.5 "Quit" exits the app | Human verification | Tray menu interaction; could be partially mechanized by sending `SIGTERM`, but that does not exercise the `QAction → QApplication::quit` connection that is actually under test. | Human checklist H9. |
| AC2.6 Left-click when not connected does nothing (no crash) | Human verification | Source-level inspection (`grep -A2 'reason == QSystemTrayIcon::Trigger' src/MainWindow.cpp` must show the `if (m_connected)` guard) is a useful smoke check but does not prove no-crash at runtime. | Automated grep + Human checklist H10. |

---

## AC3: Tray icon adapts to dark/light mode

All three criteria are inherently visual: "visible (non-faint)", "updates when system appearance changes", and "yellow/lit icon is visible" are properties of rendered pixels in the macOS menu bar. Programmatically toggling appearance via `defaults write -g AppleInterfaceStyle Dark` (and `killall cfprefsd`) plus reading the rendered tray pixel back is theoretically possible but requires screen-capture + image comparison infrastructure that does not exist in this repo and would itself need golden images that depend on macOS version.

| AC | Type | Approach | Test artifact / steps |
|----|------|----------|-----------------------|
| AC3.1 Off-state icon visible in dark mode | Automated (source) + Human (visual) | Source check: `createLightbulbIcon` must contain `QColor(200, 200, 200)` inside an `#ifdef Q_OS_MACOS` block. Visual confirmation is human. | Automated grep; Human checklist H11. |
| AC3.2 Icon updates on system appearance change without app restart | Automated (source) + Human (live) | Source check: `setupTrayIcon` must connect `QStyleHints::colorSchemeChanged` to `updateTrayIcon`, and `changeEvent` must handle `QEvent::PaletteChange`. Live appearance-toggle response is human. | Automated grep for `colorSchemeChanged` and `PaletteChange`; Human checklist H12. |
| AC3.3 Lit (yellow) icon visible in both modes | Human verification | Visual. | Human checklist H13. |

---

## AC4: Linux build is unaffected

| AC | Type | Approach | Test artifact / steps |
|----|------|----------|-----------------------|
| AC4.1 Linux `cmake --build` produces plain executable (not `.app`) | Automated (integration, shell) | On a Linux host: `cmake -B build && cmake --build build && test -f build/luminaire && test ! -d build/luminaire.app`. | Run as part of Linux verification pass (Phase 1, Task 3, Step 5). |
| AC4.2 Linux `.desktop` and hicolor SVG install rules still function | Automated (integration, shell) | On a Linux host: `cmake --install build --prefix /tmp/lum-install && test -f /tmp/lum-install/share/applications/luminaire.desktop && test -f /tmp/lum-install/share/icons/hicolor/scalable/apps/luminaire.svg`. | Linux verification pass; not currently in any phase task — add to the AC4 sign-off step. |
| AC4.3 Linux tray behavior (double-click, middle-click show/hide) unchanged | Automated (source) + Human (runtime) | Source check: `onTrayActivated` must still contain the `MiddleClick` branch outside the `#ifdef Q_OS_MACOS` guard, and `setupTrayIcon` on Linux must use the label `"Exit"`. Runtime click verification is human (and was working before this change — regression risk is low). | Automated grep for `MiddleClick` inside `#ifndef Q_OS_MACOS` block; Human checklist H14 (Linux only). |

---

## AC5: Icon asset quality

| AC | Type | Approach | Test artifact / steps |
|----|------|----------|-----------------------|
| AC5.1 `.icns` renders clearly at small/medium/large in Finder | Automated (file format) + Human (visual) | Automatable: `file data/luminaire.icns` reports `Mac OS X icon`; `iconutil -c iconset data/luminaire.icns -o /tmp/roundtrip` succeeds; the round-tripped iconset contains 10 PNGs at the expected pixel dimensions (`sips -g pixelWidth -g pixelHeight` on each). This proves *structural* quality but not *visual* clarity (e.g. a blank white square would pass). Visual clarity is human. | Phase 2, Task 3 Step 2 (file size > 1 byte), plus extended assertion: `sips -g pixelWidth data/luminaire.iconset/icon_512x512@2x.png` returns `1024`. Human checklist H15. |
| AC5.2 `data/luminaire.icns` and `data/luminaire.iconset/` both committed | Automated (integration, shell) | `git ls-files data/luminaire.icns data/luminaire.iconset/ | wc -l` must report ≥ 11 (1 icns + 10 PNGs). | Run after Phase 2, Task 3, Step 4. |

---

## Human Verification Checklist

Execute on a macOS host (Apple Silicon or Intel) running macOS 13+ with Qt6 installed via Homebrew, after all three phases are complete. Some items additionally require a physically reachable Elgato Key Light Neo on the local network.

**Setup:**
1. `cd /Users/jlandau/git/luminaire && cmake -B build && cmake --build build`
2. `open build/luminaire.app`

**Checklist:**

- [ ] **H1 (AC1.2):** Open Finder, navigate to `build/`, double-click `luminaire.app`. App launches without an "unidentified developer" hard-block. (Right-click → Open the first time if Gatekeeper intervenes; this is expected for an unsigned build.)
- [ ] **H2 (AC1.3):** Drag `luminaire.app` from `build/` onto `/Applications` in Finder. Launch from `/Applications`. App runs.
- [ ] **H3 (AC1.4):** While the app is running, look at the Dock. No Luminaire icon appears.
- [ ] **H4 (AC1.5):** While the app is running, press Cmd+Tab. Luminaire is not in the application switcher.
- [ ] **H5 (AC2.1):** A lightbulb icon is visible in the macOS menu bar (top-right area).
- [ ] **H6 (AC2.2):** With a Key Light configured and connected, single-left-click the menu-bar icon. The physical light toggles on/off and the icon color updates (yellow when on, gray when off).
- [ ] **H7 (AC2.3):** Right-click the menu-bar icon. The popup menu contains, in order: "Light On", "Light Off", separator, "Show Window" (or "Show Config" if implementation is updated to match design), separator, "Quit". No "Exit" label appears.
- [ ] **H8 (AC2.4):** Right-click → "Show Window" (or "Show Config"). The MainWindow appears with IP address field, brightness slider, and temperature slider.
- [ ] **H9 (AC2.5):** Right-click → "Quit". The app exits; the menu-bar icon disappears; `pgrep -x luminaire` returns nothing.
- [ ] **H10 (AC2.6):** Relaunch the app. Before entering an IP / connecting, left-click the menu-bar icon. Nothing happens; no crash; app stays running.
- [ ] **H11 (AC3.1):** System Settings → Appearance → Dark. Disconnect from the light (or leave off). The menu-bar icon is light gray and clearly visible against the dark menu bar (not nearly invisible).
- [ ] **H12 (AC3.2):** With the app running, toggle System Settings → Appearance between Light and Dark. The menu-bar icon updates within ~1 second without restarting the app.
- [ ] **H13 (AC3.3):** Turn the connected light on (or click "Light On"). The yellow lit-icon is clearly visible in both Light and Dark appearance.
- [ ] **H14 (AC4.3, Linux only):** On a Linux host, build and run `./build/luminaire`. Middle-click the tray icon → window shows/hides. Double-click the tray icon → window shows/hides. Right-click → menu contains "Exit" (not "Quit").
- [ ] **H15 (AC5.1):** In Finder, view `build/luminaire.app` in Icon view at multiple sizes (Cmd+J → Icon size slider). The lightbulb design is recognizable at 32px, 128px, and 512px without blurring or aliasing artifacts.

---

## Summary

| Category | Count | Notes |
|----------|-------|-------|
| Fully automated | 5 | AC1.1, AC4.1, AC4.2, AC5.2, partial AC1.4/AC1.5 (plist) |
| Hybrid (source/struct automated + visual human) | 6 | AC2.3, AC2.6, AC3.1, AC3.2, AC4.3, AC5.1 |
| Human only | 8 | AC1.2, AC1.3, AC2.1, AC2.2, AC2.4, AC2.5, AC3.3, AC1.4/AC1.5 visual |

No new test infrastructure is added in this work. All automated checks are inline shell assertions executed as part of each phase's existing task steps (or appended where noted). Human verification is consolidated into the single checklist above (H1–H15), executed once after Phase 3 completes.
