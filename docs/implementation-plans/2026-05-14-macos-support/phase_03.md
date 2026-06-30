# macOS Support Implementation Plan

**Goal:** Guard Linux-specific code paths and make the tray icon adapt to macOS dark/light mode at runtime.

**Architecture:** Four targeted edits to `src/MainWindow.cpp` and one to `src/MainWindow.h`: wrap Wayland activation in `#ifdef Q_OS_LINUX`, use "Quit" label on macOS, guard MiddleClick with `#ifndef Q_OS_MACOS`, connect `QStyleHints::colorSchemeChanged` for reliable dark mode detection, add `changeEvent` as a fallback, and add a dark-mode branch in `createLightbulbIcon`. Uses `Q_OS_MACOS` (not deprecated `Q_OS_MAC`) and `QStyleHints::colorScheme()` (Qt 6.5+).

**Tech Stack:** Qt6 C++17 — QStyleHints, QSystemTrayIcon, QPainter, Qt preprocessor macros

**Scope:** Phase 3 of 3 from original design

**Codebase verified:** 2026-05-14

---

## Acceptance Criteria Coverage

### macos-support.AC2: Menu bar tray icon and interactions
- **macos-support.AC2.3 Success:** Right-click on tray icon shows context menu with "Show Config", "Light On", "Light Off", "Quit"
- **macos-support.AC2.5 Success:** "Quit" exits the app
- **macos-support.AC2.6 Edge:** Left-click when not connected does nothing (no crash)

### macos-support.AC3: Tray icon adapts to dark/light mode
- **macos-support.AC3.1 Success:** Icon is visible (non-faint) when light is off and macOS is in dark mode
- **macos-support.AC3.2 Success:** Icon updates when system appearance changes without restarting the app
- **macos-support.AC3.3 Success:** Yellow/lit icon is visible in both light and dark mode when light is on

### macos-support.AC4: Linux build is unaffected
- **macos-support.AC4.3 Success:** Linux tray behavior (double-click, middle-click show/hide) is unchanged

---

<!-- START_SUBCOMPONENT_A (tasks 1-4) -->

<!-- START_TASK_1 -->
### Task 1: Add `changeEvent` declaration to `MainWindow.h`

**Verifies:** macos-support.AC3.2

**Files:**
- Modify: `src/MainWindow.h:22-24`

**Step 1: In the `protected:` section (lines 22–24), add the `changeEvent` override.**

Current lines 22–24:
```cpp
protected:
    bool eventFilter(QObject *obj, QEvent *event) override;
    void closeEvent(QCloseEvent *event) override;
```

Replace with:
```cpp
protected:
    bool eventFilter(QObject *obj, QEvent *event) override;
    void closeEvent(QCloseEvent *event) override;
    void changeEvent(QEvent *event) override;
```

**Step 2: Commit**

```bash
git add src/MainWindow.h
git commit -m "feat: declare changeEvent override for macOS dark mode adaptation"
```
<!-- END_TASK_1 -->

<!-- START_TASK_2 -->
### Task 2: Add `#include <QStyleHints>` and guard Wayland code in `showWindow()`

**Verifies:** macos-support.AC4.3

**Files:**
- Modify: `src/MainWindow.cpp:1-13` (includes block)
- Modify: `src/MainWindow.cpp:157-176` (showWindow)

**Step 1: Add `#include <QStyleHints>` to the includes block.**

Current includes (lines 1–13):
```cpp
#include "MainWindow.h"
#include "Config.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGroupBox>
#include <QIntValidator>
#include <QEvent>
#include <QMouseEvent>
#include <QCloseEvent>
#include <QApplication>
#include <QPainter>
#include <QTimer>
#include <QDebug>
```

Replace with (add `<QStyleHints>` after `<QApplication>`):
```cpp
#include "MainWindow.h"
#include "Config.h"
#include <QVBoxLayout>
#include <QHBoxLayout>
#include <QGroupBox>
#include <QIntValidator>
#include <QEvent>
#include <QMouseEvent>
#include <QCloseEvent>
#include <QApplication>
#include <QStyleHints>
#include <QPainter>
#include <QTimer>
#include <QDebug>
```

**Step 2: Wrap Wayland-specific block in `showWindow()` with `#ifdef Q_OS_LINUX`.**

Current `showWindow()` (lines 157–176):
```cpp
void MainWindow::showWindow()
{
    // Clear minimized state if present
    if (isMinimized()) {
        setWindowState(windowState() & ~Qt::WindowMinimized);
    }

    show();

    // On Wayland (especially KDE Plasma), window activation is restricted.
    // We need to use a combination of techniques to bring window to front.
    setWindowState(Qt::WindowActive);
    raise();
    activateWindow();

    // Force focus - helps on some platforms
    setFocus();

    updateShowHideAction();
}
```

Replace with:
```cpp
void MainWindow::showWindow()
{
    // Clear minimized state if present
    if (isMinimized()) {
        setWindowState(windowState() & ~Qt::WindowMinimized);
    }

    show();

#ifdef Q_OS_LINUX
    // On Wayland (especially KDE Plasma), window activation is restricted.
    // We need to use a combination of techniques to bring window to front.
    setWindowState(Qt::WindowActive);
    raise();
    activateWindow();

    // Force focus - helps on some platforms
    setFocus();
#endif

    updateShowHideAction();
}
```

**Step 3: Commit**

```bash
git add src/MainWindow.cpp
git commit -m "feat: guard Wayland window-activation code with Q_OS_LINUX"
```
<!-- END_TASK_2 -->

<!-- START_TASK_3 -->
### Task 3: Update `setupTrayIcon()`, `onTrayActivated()`, and `createLightbulbIcon()`

**Verifies:** macos-support.AC2.3, macos-support.AC2.5, macos-support.AC2.6, macos-support.AC3.1, macos-support.AC3.3, macos-support.AC4.3

**Files:**
- Modify: `src/MainWindow.cpp:200-232` (setupTrayIcon)
- Modify: `src/MainWindow.cpp:234-277` (createLightbulbIcon)
- Modify: `src/MainWindow.cpp:291-310` (onTrayActivated)

**Step 1: Replace `setupTrayIcon()` (lines 200–232).**

Replace the entire method:
```cpp
void MainWindow::setupTrayIcon()
{
    m_trayIcon = new QSystemTrayIcon(this);
    m_trayIcon->setToolTip("Key Light Control");

    // Create tray menu
    m_trayMenu = new QMenu(this);

    m_trayPowerOnAction = m_trayMenu->addAction("Light On");
    m_trayPowerOffAction = m_trayMenu->addAction("Light Off");
    m_trayMenu->addSeparator();
    m_trayShowHideAction = m_trayMenu->addAction("Show Window");
    m_trayMenu->addSeparator();
#ifdef Q_OS_MACOS
    QAction *exitAction = m_trayMenu->addAction("Quit");
#else
    QAction *exitAction = m_trayMenu->addAction("Exit");
#endif

    m_trayIcon->setContextMenu(m_trayMenu);

    // Connect tray actions
    connect(m_trayPowerOnAction, &QAction::triggered, this, [this]() { m_api->setPower(true); });
    connect(m_trayPowerOffAction, &QAction::triggered, this, [this]() { m_api->setPower(false); });
    connect(m_trayShowHideAction, &QAction::triggered, this, [this]() {
        qDebug() << "Show/Hide Window clicked - isVisible:" << isVisible() << "isMinimized:" << isMinimized();
        toggleWindow();
    });
    connect(exitAction, &QAction::triggered, qApp, &QApplication::quit);

    connect(m_trayIcon, &QSystemTrayIcon::activated, this, &MainWindow::onTrayActivated);

#ifdef Q_OS_MACOS
    connect(qApp->styleHints(), &QStyleHints::colorSchemeChanged, this, [this](Qt::ColorScheme) {
        updateTrayIcon(m_lightOn);
    });
#endif

    updateTrayIcon(false);
    updateTrayActions();
    updateShowHideAction();
    m_trayIcon->show();
}
```

**Step 2: Replace the color lines in `createLightbulbIcon()` (line 243–244).**

Current lines 243–244:
```cpp
    QColor bulbColor = on ? QColor(255, 220, 80) : QColor(128, 128, 128);
    QColor outlineColor = on ? QColor(200, 160, 40) : QColor(80, 80, 80);
```

Replace with:
```cpp
#ifdef Q_OS_MACOS
    const bool darkMode = QGuiApplication::styleHints()->colorScheme() == Qt::ColorScheme::Dark;
    QColor bulbColor = on ? QColor(255, 220, 80) : (darkMode ? QColor(200, 200, 200) : QColor(128, 128, 128));
    QColor outlineColor = on ? QColor(200, 160, 40) : (darkMode ? QColor(150, 150, 150) : QColor(80, 80, 80));
#else
    QColor bulbColor = on ? QColor(255, 220, 80) : QColor(128, 128, 128);
    QColor outlineColor = on ? QColor(200, 160, 40) : QColor(80, 80, 80);
#endif
```

In dark mode, the off-state uses `QColor(200,200,200)` (light gray) so the icon is clearly visible against a dark menu bar. `QGuiApplication` is accessible because `QApplication` (already included) inherits from it — no additional include needed.

**Step 3: Replace `onTrayActivated()` (lines 291–310) to guard MiddleClick.**

Replace the entire method:
```cpp
void MainWindow::onTrayActivated(QSystemTrayIcon::ActivationReason reason)
{
    qDebug() << "Tray activated with reason:" << reason;

    if (reason == QSystemTrayIcon::Trigger) {
        // Single-click: toggle light power
        qDebug() << "Single-click detected - toggling light";
        if (m_connected) {
            m_api->setPower(!m_lightOn);
        }
    } else if (reason == QSystemTrayIcon::DoubleClick) {
        // Double-click: show/hide window
        qDebug() << "Double-click detected - toggling window";
        toggleWindow();
#ifndef Q_OS_MACOS
    } else if (reason == QSystemTrayIcon::MiddleClick) {
        // Middle-click: show/hide window (more reliable on Linux)
        qDebug() << "Middle-click detected - toggling window";
        toggleWindow();
#endif
    }
}
```

**Step 4: Commit**

```bash
git add src/MainWindow.cpp
git commit -m "feat: macOS tray icon guards - Quit label, no MiddleClick, dark mode icon"
```
<!-- END_TASK_3 -->

<!-- START_TASK_4 -->
### Task 4: Implement `changeEvent()` in `MainWindow.cpp`

**Verifies:** macos-support.AC3.2

**Files:**
- Modify: `src/MainWindow.cpp` (insert after `closeEvent` at line 321)

**Step 1: After the closing brace of `closeEvent` (line 321), insert the `changeEvent` implementation.**

After line 321, add:
```cpp

void MainWindow::changeEvent(QEvent *event)
{
#ifdef Q_OS_MACOS
    if (event->type() == QEvent::PaletteChange) {
        updateTrayIcon(m_lightOn);
    }
#endif
    QWidget::changeEvent(event);
}
```

Note: `QStyleHints::colorSchemeChanged` (connected in `setupTrayIcon`) is the primary trigger for live dark/light mode switches on macOS. This `changeEvent` override handles `PaletteChange` as a fallback for any other palette-driven updates. Always call `QWidget::changeEvent(event)` to preserve base class behavior.

**Step 2: Build**

```bash
cmake --build build
```

Expected: builds without errors or warnings on macOS.

**Step 3: Manual verification on macOS**

1. Run: `open build/luminaire.app`
2. Verify tray icon appears in the menu bar (AC2.1)
3. Right-click tray icon — menu must show "Quit" (not "Exit") (AC2.3, AC2.5)
4. Left-click tray icon when not connected — must do nothing (no crash) (AC2.6)
5. Enter an IP, connect to a real Key Light:
   - Left-click tray icon — light toggles on/off (AC2.2)
   - Right-click → "Show Config" — MainWindow appears (AC2.4)
6. System Settings → Appearance → switch Dark ↔ Light
7. Tray icon must update without restarting: light gray when off + dark mode; yellow when on (AC3.1, AC3.2, AC3.3)

**Step 4: Build on Linux (if available) to verify AC4**

```bash
cmake --build build
./build/luminaire
```

Expected: builds and runs; right-click shows "Exit" (not "Quit"); middle-click still toggles window.

**Step 5: Commit**

```bash
git add src/MainWindow.cpp
git commit -m "feat: implement changeEvent for palette-change tray icon refresh"
```
<!-- END_TASK_4 -->

<!-- END_SUBCOMPONENT_A -->
