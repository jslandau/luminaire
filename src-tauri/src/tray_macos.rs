use crate::icon::{render_lightbulb_icon, ICON_SIZE};
use crate::tray::Theme;
use objc::{sel, sel_impl};
use std::sync::Mutex;
use tauri::image::Image;
use tauri::menu::{Menu, MenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, State};

/// State held for the macOS tray: handles to menu items and current theme.
pub struct TrayState {
    pub current_theme: Theme,
    pub light_on: bool,
    pub connected: bool,
    pub power_on_item: Option<MenuItem<tauri::Wry>>,
    pub power_off_item: Option<MenuItem<tauri::Wry>>,
    pub show_hide_item: Option<MenuItem<tauri::Wry>>,
}

impl Default for TrayState {
    fn default() -> Self {
        Self {
            current_theme: Theme::Light,
            light_on: false,
            connected: false,
            power_on_item: None,
            power_off_item: None,
            show_hide_item: None,
        }
    }
}

/// Get the shared tray state.
fn tray_state(app: &AppHandle) -> State<'_, Mutex<TrayState>> {
    app.state::<Mutex<TrayState>>()
}

/// Render the icon for the current state and theme.
fn render_icon(light_on: bool, theme: Theme) -> Image<'static> {
    let rgba = render_lightbulb_icon(light_on, theme == Theme::Dark);
    Image::new_owned(rgba, ICON_SIZE, ICON_SIZE)
}

/// Setup the macOS tray icon and menu.
pub fn setup_tray(app: &AppHandle) -> tauri::Result<()> {
    // Build menu items
    let power_on = MenuItem::with_id(app, "light_on", "Light On", false, None::<&str>)?;
    let power_off = MenuItem::with_id(app, "light_off", "Light Off", false, None::<&str>)?;
    let show_hide = MenuItem::with_id(app, "show_hide", "Show Window", true, None::<&str>)?;
    let quit = MenuItem::with_id(app, "quit", "Quit", true, None::<&str>)?;

    let menu = Menu::with_items(app, &[&power_on, &power_off, &show_hide, &quit])?;

    // Determine initial theme — default to Light, will correct on first ThemeChanged
    let theme = detect_macos_theme();

    // Store menu item handles in tray state
    {
        let tray = tray_state(app);
        let mut state = tray.lock().unwrap();
        state.current_theme = theme;
        state.power_on_item = Some(power_on);
        state.power_off_item = Some(power_off);
        state.show_hide_item = Some(show_hide);
    }

    let icon = render_icon(false, theme);

    let _tray = TrayIconBuilder::with_id("main")
        .icon(icon)
        .tooltip("Key Light Control")
        .menu(&menu)
        .show_menu_on_left_click(false)
        .on_tray_icon_event(|tray, event| {
            match event {
                TrayIconEvent::Click {
                    button: MouseButton::Left,
                    button_state: MouseButtonState::Up,
                    ..
                } => {
                    let app = tray.app_handle();
                    // Toggle power — fire-and-forget async
                    let app_clone = app.clone();
                    tauri::async_runtime::spawn(async move {
                        let state = app_clone.state::<crate::commands::SharedAppState>();
                        let client = app_clone.state::<crate::commands::SharedClient>();
                        let connected = {
                            let s = state.lock().unwrap();
                            s.connected
                        };
                        if !connected {
                            return;
                        }
                        let new_power = {
                            let s = state.lock().unwrap();
                            !s.light_on
                        };
                        crate::commands::set_power_direct(&app_clone, &client, new_power).await;
                    });
                }
                _ => {}
            }
        })
        .on_menu_event(|app, event| match event.id.as_ref() {
            "light_on" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let client = app_clone.state::<crate::commands::SharedClient>();
                    crate::commands::set_power_direct(&app_clone, &client, true).await;
                });
            }
            "light_off" => {
                let app_clone = app.clone();
                tauri::async_runtime::spawn(async move {
                    let client = app_clone.state::<crate::commands::SharedClient>();
                    crate::commands::set_power_direct(&app_clone, &client, false).await;
                });
            }
            "show_hide" => {
                if let Some(window) = app.get_webview_window("main") {
                    if window.is_visible().unwrap_or(false) {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
            "quit" => {
                app.exit(0);
            }
            _ => {}
        })
        .build(app)?;

    Ok(())
}

/// Update the tray icon and tooltip based on light state.
pub fn update_tray_icon(app: &AppHandle, light_on: bool) {
    let theme = {
        let tray = tray_state(app);
        let mut state = tray.lock().unwrap();
        state.light_on = light_on;
        state.current_theme
    };

    let icon = render_icon(light_on, theme);

    if let Some(tray) = app.tray_by_id("main") {
        let _ = tray.set_icon(Some(icon));
        let tooltip = if light_on {
            "Key Light Control - On"
        } else {
            "Key Light Control"
        };
        let _ = tray.set_tooltip(Some(tooltip));
    }
}

/// Update menu item enabled states based on connection state.
pub fn update_menu_state(app: &AppHandle, connected: bool) {
    let (power_on, power_off, show_hide) = {
        let tray = tray_state(app);
        let mut state = tray.lock().unwrap();
        state.connected = connected;
        (
            state.power_on_item.clone(),
            state.power_off_item.clone(),
            state.show_hide_item.clone(),
        )
    };

    if let Some(item) = &power_on {
        let _ = item.set_enabled(connected);
    }
    if let Some(item) = &power_off {
        let _ = item.set_enabled(connected);
    }

    // Update show/hide label based on window visibility
    if let Some(item) = &show_hide {
        let window = app.get_webview_window("main");
        let is_visible = window
            .map(|w| w.is_visible().unwrap_or(false))
            .unwrap_or(false);
        let label = if is_visible { "Hide Window" } else { "Show Window" };
        let _ = item.set_text(label);
    }
}

/// Handle theme change — re-render the tray icon.
pub fn on_theme_changed(app: &AppHandle, theme: Theme) {
    {
        let tray = tray_state(app);
        let mut state = tray.lock().unwrap();
        state.current_theme = theme;
    }
    let light_on = {
        let tray = tray_state(app);
        let state = tray.lock().unwrap();
        state.light_on
    };
    update_tray_icon(app, light_on);
}

/// Detect macOS dark mode at startup via NSApp.effectiveAppearance.
/// Falls back to Light if detection fails.
fn detect_macos_theme() -> Theme {
    // Use objc to query NSApp.effectiveAppearance.name
    // Falls back to Light if detection fails.
    unsafe {
        let cls = match objc::runtime::Class::get("NSApplication") {
            Some(c) => c,
            None => return Theme::Light,
        };
        let app: *mut objc::runtime::Object = objc::msg_send![cls, sharedApplication];
        if app.is_null() {
            return Theme::Light;
        }

        // NSApp.effectiveAppearance
        let appearance: *mut objc::runtime::Object = objc::msg_send![app, effectiveAppearance];
        if appearance.is_null() {
            return Theme::Light;
        }

        // appearance.name returns NSAppearanceName (NSString)
        let name_sel = objc::sel!(name);
        let name_str: *mut objc::runtime::Object = objc::msg_send![appearance, performSelector: name_sel];
        if name_str.is_null() {
            return Theme::Light;
        }

        // Convert NSString to Rust String via UTF8String
        let utf8_sel = objc::sel!(UTF8String);
        let c_str: *const std::os::raw::c_char = objc::msg_send![name_str, performSelector: utf8_sel];
        if c_str.is_null() {
            return Theme::Light;
        }

        let name = std::ffi::CStr::from_ptr(c_str).to_string_lossy().to_string();

        if name.contains("Dark") {
            return Theme::Dark;
        }
    }

    Theme::Light
}
