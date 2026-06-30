use crate::app_state::{AppState, MAX_CONSECUTIVE_ERRORS};
use crate::config::Config;
use crate::keylight::{KeyLightClient, LightState};
use std::sync::Mutex;
use tauri::{AppHandle, Emitter, Manager, State};

/// The shared mutable state managed by Tauri.
pub type SharedAppState = Mutex<AppState>;
pub type SharedConfig = Mutex<Config>;
pub type SharedClient = Mutex<Option<KeyLightClient>>;

/// Payload for the `state-received` event.
#[derive(Clone, serde::Serialize)]
pub struct StateReceivedPayload {
    pub on: bool,
    pub brightness: i32,
    pub temperature: i32,
}

/// Payload for the `connection-succeeded` event.
#[derive(Clone, serde::Serialize)]
pub struct ConnectionSucceededPayload {
    pub ip: String,
}

/// Payload for the `error` event.
#[derive(Clone, serde::Serialize)]
pub struct ErrorPayload {
    pub message: String,
    pub consecutive_errors: u32,
    pub disconnected: bool,
}

/// Payload for the `status-update` event.
#[derive(Clone, serde::Serialize)]
pub struct StatusUpdatePayload {
    pub text: String,
    pub color: String,
}

/// Connect to the light at the given IP.
/// Saves IP, fetches state, and on success restores saved brightness/temperature.
#[tauri::command]
pub async fn connect(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    config: State<'_, SharedConfig>,
    client: State<'_, SharedClient>,
    ip: String,
) -> Result<LightState, String> {
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.begin_connect(&ip);
    }

    // Save the IP immediately
    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        cfg.ip_address = ip.clone();
        cfg.save();
    }

    // Create the KeyLight client
    let kc = KeyLightClient::new(ip.clone(), crate::keylight::DEFAULT_PORT);
    {
        let mut c = client.lock().map_err(|e| e.to_string())?;
        *c = Some(kc);
    }

    // Fetch the current state
    let kc_ref = {
        let c = client.lock().map_err(|e| e.to_string())?;
        c.as_ref()
            .ok_or("No client configured")?
            .clone_state()
    };

    match kc_ref.fetch_state().await {
        Ok(light_state) => {
            // Mark connected
            {
                let mut s = state.lock().map_err(|e| e.to_string())?;
                s.mark_connected();
                s.on_state_received(light_state.on, light_state.brightness, light_state.temperature_kelvin);
            }

            // Emit connection-succeeded immediately (before restore PUTs)
            app
                .emit("connection-succeeded", ConnectionSucceededPayload { ip: ip.clone() })
                .ok();

            // Update tray menu (enable power items, update show/hide label)
            crate::tray::update_tray_menu(&app, true);
            crate::tray::update_tray(&app, light_state.on);

            // Emit the initial state-received
            app
                .emit("state-received", StateReceivedPayload {
                    on: light_state.on,
                    brightness: light_state.brightness,
                    temperature: light_state.temperature_kelvin,
                })
                .ok();

            // Restore saved brightness/temperature if available (AC2.4)
            // Use a single set_state PUT when both values are present to avoid races.
            // Each restore PUT emits state-received on response (AC1.6).
            let saved_brightness = {
                let cfg = config.lock().map_err(|e| e.to_string())?;
                cfg.brightness
            };
            let saved_temperature = {
                let cfg = config.lock().map_err(|e| e.to_string())?;
                cfg.temperature
            };

            if saved_brightness >= 0 && saved_temperature >= 0 {
                // Both saved: single combined PUT via set_state
                let app_clone = app.clone();
                let kc_clone = kc_ref.clone_state();
                let on_state = light_state.on;
                tokio::spawn(async move {
                    match kc_clone.set_state(on_state, saved_brightness, saved_temperature).await {
                        Ok(updated) => {
                            update_state_and_emit(&app_clone, &kc_clone, updated).await;
                        }
                        Err(e) => {
                            handle_error_spawned(&app_clone, &e).await;
                        }
                    }
                });
            } else if saved_brightness >= 0 {
                let app_clone = app.clone();
                let kc_clone = kc_ref.clone_state();
                tokio::spawn(async move {
                    match kc_clone.set_brightness(saved_brightness).await {
                        Ok(updated) => {
                            update_state_and_emit(&app_clone, &kc_clone, updated).await;
                        }
                        Err(e) => {
                            handle_error_spawned(&app_clone, &e).await;
                        }
                    }
                });
            } else if saved_temperature >= 0 {
                let app_clone = app.clone();
                let kc_clone = kc_ref.clone_state();
                tokio::spawn(async move {
                    match kc_clone.set_temperature(saved_temperature).await {
                        Ok(updated) => {
                            update_state_and_emit(&app_clone, &kc_clone, updated).await;
                        }
                        Err(e) => {
                            handle_error_spawned(&app_clone, &e).await;
                        }
                    }
                });
            }

            Ok(light_state)
        }
        Err(e) => {
            // Connection failed
            let should_disconnect = {
                let mut s = state.lock().map_err(|e| e.to_string())?;
                s.on_error()
            };

            let msg = e.to_string();
            if should_disconnect {
                app
                    .emit("error", ErrorPayload {
                        message: msg.clone(),
                        consecutive_errors: MAX_CONSECUTIVE_ERRORS,
                        disconnected: true,
                    })
                    .ok();
            } else {
                let count = {
                    let s = state.lock().map_err(|e| e.to_string())?;
                    s.consecutive_errors
                };
                app
                    .emit("error", ErrorPayload {
                        message: msg.clone(),
                        consecutive_errors: count,
                        disconnected: false,
                    })
                    .ok();
            }

            Err(msg)
        }
    }
}

/// Disconnect from the light.
#[tauri::command]
pub async fn disconnect(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    client: State<'_, SharedClient>,
) -> Result<(), String> {
    {
        let mut s = state.lock().map_err(|e| e.to_string())?;
        s.disconnect();
    }
    {
        let mut c = client.lock().map_err(|e| e.to_string())?;
        *c = None;
    }

    app.emit("status-update", StatusUpdatePayload {
        text: "Disconnected".to_string(),
        color: "gray".to_string(),
    }).ok();

    // Update tray menu (disable power items) and tray icon
    crate::tray::update_tray_menu(&app, false);
    crate::tray::update_tray(&app, false);

    Ok(())
}

/// Toggle light power. No-op if not connected.
#[tauri::command]
pub async fn toggle_power(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    client: State<'_, SharedClient>,
) -> Result<(), String> {
    let (can_toggle, new_power) = {
        let s = state.lock().map_err(|e| e.to_string())?;
        (s.can_toggle_power(), !s.light_on)
    };

    if !can_toggle {
        return Ok(());
    }

    let kc = {
        let c = client.lock().map_err(|e| e.to_string())?;
        c.as_ref().ok_or("Not connected")?.clone_state()
    };

    match kc.set_power(new_power).await {
        Ok(updated) => {
            update_state_and_emit(&app, &kc, updated).await;
        }
        Err(e) => {
            handle_error(&app, &state, &e).await;
        }
    }

    Ok(())
}

/// Set brightness (clamped to [0, 100]). Persists and emits state-received.
#[tauri::command]
pub async fn set_brightness(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    config: State<'_, SharedConfig>,
    client: State<'_, SharedClient>,
    value: i32,
) -> Result<(), String> {
    let kc = {
        let c = client.lock().map_err(|e| e.to_string())?;
        c.as_ref().ok_or("Not connected")?.clone_state()
    };

    // Save to config immediately
    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        cfg.brightness = value;
        cfg.save();
    }

    match kc.set_brightness(value).await {
        Ok(updated) => {
            update_state_and_emit(&app, &kc, updated).await;
        }
        Err(e) => {
            handle_error(&app, &state, &e).await;
        }
    }

    Ok(())
}

/// Set temperature in Kelvin. Persists and emits state-received.
#[tauri::command]
pub async fn set_temperature(
    app: AppHandle,
    state: State<'_, SharedAppState>,
    config: State<'_, SharedConfig>,
    client: State<'_, SharedClient>,
    kelvin: i32,
) -> Result<(), String> {
    let kc = {
        let c = client.lock().map_err(|e| e.to_string())?;
        c.as_ref().ok_or("Not connected")?.clone_state()
    };

    // Save to config immediately
    {
        let mut cfg = config.lock().map_err(|e| e.to_string())?;
        cfg.temperature = kelvin;
        cfg.save();
    }

    match kc.set_temperature(kelvin).await {
        Ok(updated) => {
            update_state_and_emit(&app, &kc, updated).await;
        }
        Err(e) => {
            handle_error(&app, &state, &e).await;
        }
    }

    Ok(())
}

/// Get the current app state snapshot.
#[tauri::command]
pub async fn get_state(state: State<'_, SharedAppState>) -> Result<crate::app_state::AppStateSnapshot, String> {
    let s = state.lock().map_err(|e| e.to_string())?;
    Ok(s.snapshot())
}

/// Set power directly (used by tray menu items "Light On" / "Light Off").
pub async fn set_power_direct(
    app: &AppHandle,
    client: &State<'_, SharedClient>,
    on: bool,
) {
    let connected = {
        let state = app.state::<SharedAppState>();
        let s = state.lock().unwrap();
        s.connected
    };

    if !connected {
        return;
    }

    let kc = {
        let c = client.lock().unwrap();
        match c.as_ref() {
            Some(kc) => kc.clone_state(),
            None => return,
        }
    };

    match kc.set_power(on).await {
        Ok(updated) => {
            update_state_and_emit(app, &kc, updated).await;
        }
        Err(e) => {
            let state = app.state::<SharedAppState>();
            handle_error(app, &state, &e).await;
        }
    }
}

// --- Helper functions ---

/// Update AppState from a LightState and emit state-received.
/// Also updates the tray icon. Guards against stale responses after disconnect (H2).
async fn update_state_and_emit(
    app: &AppHandle,
    _kc: &KeyLightClient,
    state_data: LightState,
) {
    let was_in_error;
    let connected;
    let ip;
    {
        let state = app.state::<SharedAppState>();
        let guard = state.lock();
        if let Ok(mut s) = guard {
            // Guard: don't apply state if disconnected (stale response from before disconnect)
            if !s.connected {
                return;
            }
            was_in_error = s.consecutive_errors > 0;
            connected = s.connected;
            ip = s.ip.clone();
            s.on_state_received(state_data.on, state_data.brightness, state_data.temperature_kelvin);
        } else {
            return;
        }
    }

    // If we were in an error state, restore the "Connected" status (M1)
    if was_in_error && connected {
        let _ = app.emit("status-update", StatusUpdatePayload {
            text: format!("Connected to {}", ip),
            color: "green".to_string(),
        });
    }

    // Emit state-received to frontend
    let _ = app.emit("state-received", StateReceivedPayload {
        on: state_data.on,
        brightness: state_data.brightness,
        temperature: state_data.temperature_kelvin,
    });

    // Update tray icon
    crate::tray::update_tray(app, state_data.on);
}

/// Handle an error from a spawned (fire-and-forget) task.
/// Routes through the standard handle_error path.
async fn handle_error_spawned(app: &AppHandle, error: &crate::keylight::KeyLightError) {
    let state = app.state::<SharedAppState>();
    handle_error(app, &state, error).await;
}

/// Handle an error: increment counter, emit error event, possibly disconnect.
async fn handle_error(
    app: &AppHandle,
    state: &State<'_, SharedAppState>,
    error: &crate::keylight::KeyLightError,
) {
    let (should_disconnect, count) = {
        let mut s = state.lock().unwrap();
        let d = s.on_error();
        (d, s.consecutive_errors)
    };

    let msg = error.to_string();
    if should_disconnect {
        app.emit("error", ErrorPayload {
            message: msg,
            consecutive_errors: count,
            disconnected: true,
        }).ok();
        crate::tray::update_tray_menu(app, false);
        crate::tray::update_tray(app, false);
    } else {
        app.emit("error", ErrorPayload {
            message: msg,
            consecutive_errors: count,
            disconnected: false,
        }).ok();
    }
}


