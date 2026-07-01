use crate::icon::{render_lightbulb_icon, to_argb32, ICON_SIZE};
use ksni::menu::StandardItem;
use ksni::{Icon, MenuItem, ToolTip, Tray, TrayService};
use std::sync::{Arc, Mutex};
use tauri::{AppHandle, Manager};

/// Shared state between the ksni tray and the Tauri runtime.
#[derive(Debug, Clone, Default)]
pub struct TraySharedState {
    pub light_on: bool,
    pub connected: bool,
}

/// The ksni tray implementation for Linux.
pub struct LinuxTray {
    pub state: Arc<Mutex<TraySharedState>>,
    pub app_handle: AppHandle,
}

impl Tray for LinuxTray {
    fn id(&self) -> String {
        "luminaire".to_string()
    }

    fn title(&self) -> String {
        "Luminaire".to_string()
    }

    fn tool_tip(&self) -> ToolTip {
        let light_on = {
            let s = self.state.lock().unwrap();
            s.light_on
        };
        ToolTip {
            icon_name: String::new(),
            icon_pixmap: Vec::new(),
            title: if light_on {
                "Key Light Control - On".to_string()
            } else {
                "Key Light Control".to_string()
            },
            description: String::new(),
        }
    }

    fn icon_pixmap(&self) -> Vec<Icon> {
        let light_on = {
            let s = self.state.lock().unwrap();
            s.light_on
        };

        let rgba = render_lightbulb_icon(light_on, false);
        let argb = to_argb32(&rgba, ICON_SIZE, ICON_SIZE);

        vec![Icon {
            width: ICON_SIZE as i32,
            height: ICON_SIZE as i32,
            data: argb,
        }]
    }

    fn activate(&mut self, _x: i32, _y: i32) {
        // Left-click: toggle light power
        let app = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            let state = app.state::<crate::commands::SharedAppState>();
            let client = app.state::<crate::commands::SharedClient>();
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
            crate::commands::set_power_direct(&app, &client, new_power).await;
        });
    }

    fn secondary_activate(&mut self, _x: i32, _y: i32) {
        // Middle-click: toggle window visibility
        let app = self.app_handle.clone();
        tauri::async_runtime::spawn(async move {
            if let Some(window) = app.get_webview_window("main") {
                if window.is_visible().unwrap_or(false) {
                    let _ = window.hide();
                } else {
                    let _ = window.show();
                    let _ = window.set_focus();
                }
            }
        });
    }

    fn menu(&self) -> Vec<MenuItem<Self>> {
        let connected = {
            let s = self.state.lock().unwrap();
            s.connected
        };

        let window = self.app_handle.get_webview_window("main");
        let is_visible = window
            .map(|w| w.is_visible().unwrap_or(false))
            .unwrap_or(false);

        let show_hide_label = if is_visible {
            "Hide Window"
        } else {
            "Show Window"
        };

        vec![
            MenuItem::Standard(StandardItem {
                label: "Light On".to_string(),
                enabled: connected,
                activate: Box::new(|tray: &mut LinuxTray| {
                    let app = tray.app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let client = app.state::<crate::commands::SharedClient>();
                        crate::commands::set_power_direct(&app, &client, true).await;
                    });
                }),
                ..Default::default()
            }),
            MenuItem::Standard(StandardItem {
                label: "Light Off".to_string(),
                enabled: connected,
                activate: Box::new(|tray: &mut LinuxTray| {
                    let app = tray.app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        let client = app.state::<crate::commands::SharedClient>();
                        crate::commands::set_power_direct(&app, &client, false).await;
                    });
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: show_hide_label.to_string(),
                enabled: true,
                activate: Box::new(|tray: &mut LinuxTray| {
                    let app = tray.app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        if let Some(window) = app.get_webview_window("main") {
                            if window.is_visible().unwrap_or(false) {
                                let _ = window.hide();
                            } else {
                                let _ = window.show();
                                let _ = window.set_focus();
                            }
                        }
                    });
                }),
                ..Default::default()
            }),
            MenuItem::Separator,
            MenuItem::Standard(StandardItem {
                label: "Exit".to_string(),
                enabled: true,
                activate: Box::new(|tray: &mut LinuxTray| {
                    let app = tray.app_handle.clone();
                    tauri::async_runtime::spawn(async move {
                        app.exit(0);
                    });
                }),
                ..Default::default()
            }),
        ]
    }
}

/// Shared handle to the ksni tray service.
static TRAY_HANDLE: std::sync::OnceLock<ksni::Handle<LinuxTray>> = std::sync::OnceLock::new();

/// Setup the Linux tray using ksni.
pub fn setup_tray(app: &AppHandle) -> Result<(), String> {
    let shared_state = Arc::new(Mutex::new(TraySharedState::default()));

    // Store the shared state in Tauri's managed state
    app.manage(shared_state.clone());

    let tray = LinuxTray {
        state: shared_state,
        app_handle: app.clone(),
    };

    let service = TrayService::new(tray);
    let handle = service.handle();
    let _ = TRAY_HANDLE.set(handle);
    service.spawn();

    Ok(())
}

/// Update the tray icon and tooltip based on light state.
pub fn update_tray_icon(app: &AppHandle, light_on: bool) {
    // Update the shared state
    let shared_state = app.state::<Arc<Mutex<TraySharedState>>>();
    {
        let mut s = shared_state.lock().unwrap();
        s.light_on = light_on;
    }

    // Trigger ksni to re-render.
    if let Some(handle) = TRAY_HANDLE.get() {
        handle.update(|_tray| {
            // The update closure just triggers a re-read of icon_pixmap and tool_tip.
        });
    }
}

/// Update menu state based on connection.
pub fn update_menu_state(app: &AppHandle, connected: bool) {
    let shared_state = app.state::<Arc<Mutex<TraySharedState>>>();
    {
        let mut s = shared_state.lock().unwrap();
        s.connected = connected;
    }

    // Trigger ksni to re-read the menu.
    if let Some(handle) = TRAY_HANDLE.get() {
        handle.update(|_tray| {});
    }
}
