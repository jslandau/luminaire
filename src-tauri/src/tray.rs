use tauri::AppHandle;

/// Current theme tracking for icon rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Theme {
    Light,
    Dark,
}

/// Update the tray icon based on light state.
/// Called from the command layer whenever a state update is received.
pub fn update_tray(app: &AppHandle, light_on: bool) {
    #[cfg(target_os = "macos")]
    {
        crate::tray_macos::update_tray_icon(app, light_on);
    }
    #[cfg(target_os = "linux")]
    {
        crate::tray_linux::update_tray_icon(app, light_on);
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = app;
        let _ = light_on;
    }
}

/// Update the tray menu state (enabled/disabled items, show/hide label).
/// Called when connection state changes.
pub fn update_tray_menu(app: &AppHandle, connected: bool) {
    #[cfg(target_os = "macos")]
    {
        crate::tray_macos::update_menu_state(app, connected);
    }
    #[cfg(target_os = "linux")]
    {
        crate::tray_linux::update_menu_state(app, connected);
    }
    #[cfg(not(any(target_os = "macos", target_os = "linux")))]
    {
        let _ = app;
        let _ = connected;
    }
}
