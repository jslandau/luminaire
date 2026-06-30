pub mod app_state;
pub mod commands;
pub mod config;
pub mod icon;
pub mod keylight;
pub mod tray;

#[cfg(target_os = "macos")]
pub mod tray_macos;

#[cfg(target_os = "linux")]
pub mod tray_linux;

use std::sync::Mutex;

use app_state::AppState;
use commands::{SharedAppState, SharedClient, SharedConfig};
use config::Config;
use keylight::KeyLightClient;
use tauri::{Emitter, Manager};

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    let config = Config::load();
    let saved_ip = config.ip_address.clone();

    tauri::Builder::default()
        .plugin(tauri_plugin_shell::init())
        .manage(Mutex::new(AppState::default()))
        .manage(Mutex::new(config.clone()))
        .manage(Mutex::new(None::<KeyLightClient>))
        .setup(move |app| {
            // Setup platform-specific tray
            #[cfg(target_os = "macos")]
            {
                // Manage tray state before setup_tray (which reads it)
                app.manage(std::sync::Mutex::new(tray_macos::TrayState::default()));
                tray_macos::setup_tray(app.handle())?;
            }

            #[cfg(target_os = "linux")]
            {
                tray_linux::setup_tray(app.handle()).map_err(|e| {
                    tauri::Error::Anyhow(anyhow::anyhow!("{}", e))
                })?;
            }

            // Auto-connect if we have a saved IP (AC2.3)
            if !saved_ip.is_empty() {
                let app_handle = app.handle().clone();
                let ip = saved_ip.clone();
                tauri::async_runtime::spawn(async move {
                    // Small delay to let the frontend load
                    tokio::time::sleep(std::time::Duration::from_millis(500)).await;
                    let state = app_handle.state::<SharedAppState>();
                    let config = app_handle.state::<SharedConfig>();
                    let client = app_handle.state::<SharedClient>();
                    let _ = commands::connect(
                        app_handle.clone(),
                        state,
                        config,
                        client,
                        ip,
                    )
                    .await;
                });
            }

            // Start the periodic refresh timer
            let app_handle = app.handle().clone();
            tauri::async_runtime::spawn(async move {
                refresh_loop(app_handle).await;
            });

            // Determine initial window visibility (AC8.1, AC8.2)
            #[cfg(target_os = "macos")]
            {
                // macOS: start minimized (tray-only) by default
                if let Some(window) = app.get_webview_window("main") {
                    let _ = window.hide();
                }
            }

            #[cfg(target_os = "linux")]
            {
                // Linux: show window by default; --minimized flag hides it
                let minimized = std::env::args().any(|a| a == "--minimized");
                if let Some(window) = app.get_webview_window("main") {
                    if minimized {
                        let _ = window.hide();
                    } else {
                        let _ = window.show();
                    }
                }
            }

            Ok(())
        })
        .on_window_event(|window, event| {
            match event {
                // Close-to-tray: intercept window close -> hide instead of quit (AC8.3)
                tauri::WindowEvent::CloseRequested { api, .. } => {
                    api.prevent_close();
                    let _ = window.hide();
                    #[cfg(target_os = "macos")]
                    crate::tray_macos::update_show_hide_label(window.app_handle());
                }
                // macOS: Theme change detection (AC5.5)
                #[cfg(target_os = "macos")]
                tauri::WindowEvent::ThemeChanged(theme) => {
                    let app_theme = match theme {
                        tauri::Theme::Dark => tray::Theme::Dark,
                        _ => tray::Theme::Light,
                    };
                    let app = window.app_handle();
                    tray_macos::on_theme_changed(app, app_theme);
                }
                #[cfg(not(target_os = "macos"))]
                _ => {}
                #[cfg(target_os = "macos")]
                _ => {}
            }
        })
        .invoke_handler(tauri::generate_handler![
            commands::connect,
            commands::disconnect,
            commands::toggle_power,
            commands::set_brightness,
            commands::set_temperature,
            commands::get_state,
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}

/// Periodic refresh loop — polls the light state every 5 seconds while connected (AC4.3).
async fn refresh_loop(app: tauri::AppHandle) {
    loop {
        tokio::time::sleep(std::time::Duration::from_secs(5)).await;

        let connected = {
            let state = app.state::<SharedAppState>();
            let s = state.lock().unwrap();
            s.connected
        };

        if !connected {
            continue;
        }

        // Get the client
        let kc = {
            let client = app.state::<SharedClient>();
            let c = client.lock().unwrap();
            c.as_ref().map(|kc| kc.clone_state())
        };

        let kc = match kc {
            Some(kc) => kc,
            None => continue,
        };

        match kc.fetch_state().await {
            Ok(light_state) => {
                // Update state and emit
                let was_in_error;
                let ip;
                {
                    let state = app.state::<SharedAppState>();
                    let guard = state.lock();
                    if let Ok(mut s) = guard {
                        was_in_error = s.consecutive_errors > 0;
                        ip = s.ip.clone();
                        s.on_state_received(
                            light_state.on,
                            light_state.brightness,
                            light_state.temperature_kelvin,
                        );
                    } else {
                        continue;
                    }
                }

                // If recovering from an error, restore the "Connected" status (M1)
                if was_in_error {
                    let _ = app.emit("status-update", commands::StatusUpdatePayload {
                        text: format!("Connected to {}", ip),
                        color: "green".to_string(),
                    });
                }

                let _ = app.emit("state-received", commands::StateReceivedPayload {
                    on: light_state.on,
                    brightness: light_state.brightness,
                    temperature: light_state.temperature_kelvin,
                });

                // Update tray icon
                tray::update_tray(&app, light_state.on);
            }
            Err(e) => {
                // Handle error
                let (should_disconnect, count) = {
                    let state = app.state::<SharedAppState>();
                    let mut s = state.lock().unwrap();
                    let d = s.on_error();
                    (d, s.consecutive_errors)
                };

                let msg = e.to_string();
                if should_disconnect {
                    let _ = app.emit("error", commands::ErrorPayload {
                        message: msg,
                        consecutive_errors: count,
                        disconnected: true,
                    });
                    tray::update_tray_menu(&app, false);
                    tray::update_tray(&app, false);
                } else {
                    let _ = app.emit("error", commands::ErrorPayload {
                        message: msg,
                        consecutive_errors: count,
                        disconnected: false,
                    });
                }
            }
        }
    }
}
