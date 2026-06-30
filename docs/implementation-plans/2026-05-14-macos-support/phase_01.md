# macOS Support Implementation Plan

**Goal:** Make `cmake --build` on macOS produce a valid `.app` bundle skeleton with Dock-icon suppression.

**Architecture:** Add an `if(APPLE)` block to CMakeLists.txt that enables MACOSX_BUNDLE, bundles the `.icns` as a Resources file, and sets MACOSX_BUNDLE_INFO_PLIST to a new `data/Info.plist.in` template. Linux install rules are wrapped in `if(NOT APPLE)`. No source files change in this phase.

**Tech Stack:** CMake 3.16+, Qt6 (Widgets, Network), macOS Info.plist XML

**Scope:** Phase 1 of 3 from original design

**Codebase verified:** 2026-05-14

---

## Acceptance Criteria Coverage

### macos-support.AC1: cmake --build produces a runnable .app bundle
- **macos-support.AC1.1 Success:** `cmake -B build && cmake --build build` on macOS creates `build/luminaire.app`
- **macos-support.AC1.4 Failure:** No Dock icon appears when the app is running
- **macos-support.AC1.5 Failure:** App does not appear in Cmd+Tab switcher

---

<!-- START_TASK_1 -->
### Task 1: Create placeholder `data/luminaire.icns`

**Verifies:** None (infrastructure prerequisite for CMake bundle)

**Files:**
- Create: `data/luminaire.icns`

**Step 1: Create the placeholder**

```bash
printf '\x00' > data/luminaire.icns
```

This is intentionally minimal — just enough for CMake to bundle a file named `luminaire.icns` into `Contents/Resources/`. macOS will silently fall back to a generic icon until Phase 2 replaces this with a real `.icns`. Do not evaluate AC5 (icon quality) or expect the app icon to appear in Finder until Phase 2 is complete.

**Step 2: Verify it exists**

```bash
ls -la data/luminaire.icns
```

Expected: file exists

**Step 3: Commit**

```bash
git add data/luminaire.icns
git commit -m "chore: add placeholder .icns for CMake bundle (replaced in Phase 2)"
```
<!-- END_TASK_1 -->

<!-- START_TASK_2 -->
### Task 2: Create `data/Info.plist.in`

**Verifies:** None (infrastructure)

**Files:**
- Create: `data/Info.plist.in`

**Step 1: Create the file with this exact content**

```xml
<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
    <key>CFBundleDevelopmentRegion</key>
    <string>en</string>
    <key>CFBundleExecutable</key>
    <string>${MACOSX_BUNDLE_EXECUTABLE_NAME}</string>
    <key>CFBundleIdentifier</key>
    <string>@MACOSX_BUNDLE_GUI_IDENTIFIER@</string>
    <key>CFBundleInfoDictionaryVersion</key>
    <string>6.0</string>
    <key>CFBundleName</key>
    <string>@MACOSX_BUNDLE_BUNDLE_NAME@</string>
    <key>CFBundlePackageType</key>
    <string>APPL</string>
    <key>CFBundleShortVersionString</key>
    <string>@MACOSX_BUNDLE_SHORT_VERSION_STRING@</string>
    <key>CFBundleVersion</key>
    <string>@MACOSX_BUNDLE_BUNDLE_VERSION@</string>
    <key>CFBundleIconFile</key>
    <string>luminaire</string>
    <key>LSUIElement</key>
    <true/>
    <key>NSHighResolutionCapable</key>
    <true/>
    <key>NSPrincipalClass</key>
    <string>NSApplication</string>
</dict>
</plist>
```

Note: `CFBundleIconFile` is hardcoded to `luminaire` (no `.icns` extension — macOS appends it automatically). `LSUIElement <true/>` suppresses the Dock icon and Cmd+Tab entry.

**Step 2: Commit**

```bash
git add data/Info.plist.in
git commit -m "chore: add macOS Info.plist.in bundle metadata"
```
<!-- END_TASK_2 -->

<!-- START_TASK_3 -->
### Task 3: Update `CMakeLists.txt` with macOS bundle block

**Verifies:** macos-support.AC1.1, macos-support.AC1.4, macos-support.AC1.5

**Files:**
- Modify: `CMakeLists.txt`

**Step 1: Replace the entire file with this content**

The current file is 34 lines. Replace it entirely:

```cmake
cmake_minimum_required(VERSION 3.16)
project(luminaire VERSION 1.0.0 LANGUAGES CXX)

set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)
set(CMAKE_AUTOMOC ON)

find_package(Qt6 REQUIRED COMPONENTS Widgets Network)

if(APPLE)
    set_source_files_properties(data/luminaire.icns PROPERTIES
        MACOSX_PACKAGE_LOCATION "Resources"
    )
endif()

add_executable(luminaire
    src/main.cpp
    src/KeyLightAPI.cpp
    src/Config.cpp
    src/MainWindow.cpp
    $<$<BOOL:${APPLE}>:data/luminaire.icns>
)

target_include_directories(luminaire PRIVATE src)
target_link_libraries(luminaire PRIVATE Qt6::Widgets Qt6::Network)

if(APPLE)
    set_target_properties(luminaire PROPERTIES
        MACOSX_BUNDLE TRUE
        MACOSX_BUNDLE_BUNDLE_NAME "Luminaire"
        MACOSX_BUNDLE_GUI_IDENTIFIER "com.luminaire.app"
        MACOSX_BUNDLE_SHORT_VERSION_STRING "${PROJECT_VERSION}"
        MACOSX_BUNDLE_BUNDLE_VERSION "${PROJECT_VERSION}"
        MACOSX_BUNDLE_INFO_PLIST "${CMAKE_SOURCE_DIR}/data/Info.plist.in"
    )
endif()

# Installation
include(GNUInstallDirs)

if(NOT APPLE)
    install(TARGETS luminaire
        RUNTIME DESTINATION ${CMAKE_INSTALL_BINDIR}
    )

    install(FILES data/luminaire.desktop
        DESTINATION ${CMAKE_INSTALL_DATADIR}/applications
    )

    install(FILES data/luminaire.svg
        DESTINATION ${CMAKE_INSTALL_DATADIR}/icons/hicolor/scalable/apps
    )
endif()
```

Note: The generator expression `$<$<BOOL:${APPLE}>:data/luminaire.icns>` conditionally adds the `.icns` to the source list only on Apple. `MACOSX_BUNDLE_SHORT_VERSION_STRING` and `MACOSX_BUNDLE_BUNDLE_VERSION` expand to `1.0.0` from the `project()` call.

**Step 2: Configure and build**

```bash
cmake -B build
cmake --build build
```

Expected on macOS: Creates `build/luminaire.app/`

**Step 3: Verify bundle structure**

```bash
ls build/luminaire.app/Contents/
```
Expected output includes: `Info.plist  MacOS/  Resources/`

```bash
ls build/luminaire.app/Contents/Resources/
```
Expected: `luminaire.icns`

```bash
cat build/luminaire.app/Contents/Info.plist
```
Expected: contains `LSUIElement` with `<true/>`, `CFBundleIdentifier` as `com.luminaire.app`, `CFBundleName` as `Luminaire`.

**Step 4: Launch and verify Dock/Cmd+Tab suppression**

```bash
open build/luminaire.app
```

Expected: App launches. No icon appears in the Dock. App does not appear in Cmd+Tab. (The window itself may still appear — tray-only behavior is finalized in Phase 3.)

**Step 5: Verify Linux build is unaffected (if Linux is available)**

On a Linux machine or CI environment:
```bash
cmake -B build && cmake --build build
ls build/luminaire
```

Expected: produces a plain `luminaire` executable (not a `.app` directory). The `.desktop` and `.svg` install rules remain intact.

**Step 6: Commit**

```bash
git add CMakeLists.txt
git commit -m "feat: add macOS .app bundle support to CMakeLists.txt"
```
<!-- END_TASK_3 -->
