use serde::{Deserialize, Serialize};

/// Maximum consecutive errors before full disconnect.
pub const MAX_CONSECUTIVE_ERRORS: u32 = 3;

/// Snapshot of the app state sent to the frontend via the get_state command.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppStateSnapshot {
    pub connected: bool,
    pub light_on: bool,
    pub brightness: i32,
    pub temperature: i32,
    pub ip: String,
    pub consecutive_errors: u32,
}

/// The application's internal state machine.
/// Tracks connection status, light state, and error count.
#[derive(Debug, Clone)]
pub struct AppState {
    pub connected: bool,
    pub light_on: bool,
    pub consecutive_errors: u32,
    pub brightness: i32,
    pub temperature: i32,
    pub ip: String,
}

impl Default for AppState {
    fn default() -> Self {
        Self {
            connected: false,
            light_on: false,
            consecutive_errors: 0,
            brightness: 50,
            temperature: 4950,
            ip: String::new(),
        }
    }
}

impl AppState {
    pub fn snapshot(&self) -> AppStateSnapshot {
        AppStateSnapshot {
            connected: self.connected,
            light_on: self.light_on,
            brightness: self.brightness,
            temperature: self.temperature,
            ip: self.ip.clone(),
            consecutive_errors: self.consecutive_errors,
        }
    }

    /// Called when a connect attempt begins.
    pub fn begin_connect(&mut self, ip: &str) {
        self.ip = ip.to_string();
        self.consecutive_errors = 0;
    }

    /// Called when the connection succeeds.
    pub fn mark_connected(&mut self) {
        self.connected = true;
        self.consecutive_errors = 0;
    }

    /// Called on disconnect.
    pub fn disconnect(&mut self) {
        self.connected = false;
        self.consecutive_errors = 0;
        self.light_on = false;
    }

    /// Called when a state update is received from the light.
    /// Resets the error counter.
    pub fn on_state_received(&mut self, on: bool, brightness: i32, temperature: i32) {
        self.light_on = on;
        self.brightness = brightness;
        self.temperature = temperature;
        self.consecutive_errors = 0;
    }

    /// Called when an error occurs.
    /// Returns true if the error threshold was reached (should disconnect).
    pub fn on_error(&mut self) -> bool {
        self.consecutive_errors += 1;
        if self.consecutive_errors >= MAX_CONSECUTIVE_ERRORS {
            self.connected = false;
            self.light_on = false;
            true
        } else {
            false
        }
    }

    /// Check if the light toggle should proceed.
    pub fn can_toggle_power(&self) -> bool {
        self.connected
    }
}

// --- Unit tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_state() {
        let state = AppState::default();
        assert!(!state.connected);
        assert!(!state.light_on);
        assert_eq!(state.consecutive_errors, 0);
    }

    #[test]
    fn test_connect_disconnect_cycle() {
        let mut state = AppState::default();
        state.begin_connect("192.168.1.50");
        assert_eq!(state.ip, "192.168.1.50");
        assert_eq!(state.consecutive_errors, 0);

        state.mark_connected();
        assert!(state.connected);

        state.disconnect();
        assert!(!state.connected);
        assert!(!state.light_on);
        assert_eq!(state.consecutive_errors, 0);
    }

    #[test]
    fn test_error_counter_resets_on_state() {
        let mut state = AppState::default();
        state.mark_connected();

        // Two errors
        state.on_error();
        state.on_error();
        assert_eq!(state.consecutive_errors, 2);
        assert!(state.connected); // still connected

        // State received resets counter
        state.on_state_received(true, 50, 4500);
        assert_eq!(state.consecutive_errors, 0);
        assert!(state.light_on);
    }

    #[test]
    fn test_error_counter_auto_disconnect_at_3() {
        let mut state = AppState::default();
        state.mark_connected();

        let r1 = state.on_error();
        assert!(!r1);
        assert_eq!(state.consecutive_errors, 1);
        assert!(state.connected);

        let r2 = state.on_error();
        assert!(!r2);
        assert_eq!(state.consecutive_errors, 2);
        assert!(state.connected);

        let r3 = state.on_error();
        assert!(r3);
        assert_eq!(state.consecutive_errors, 3);
        assert!(!state.connected);
        assert!(!state.light_on);
    }

    #[test]
    fn test_can_toggle_power() {
        let mut state = AppState::default();
        assert!(!state.can_toggle_power());

        state.mark_connected();
        assert!(state.can_toggle_power());

        state.disconnect();
        assert!(!state.can_toggle_power());
    }
}
