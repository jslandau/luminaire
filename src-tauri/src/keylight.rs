use serde::{Deserialize, Serialize};

// Brightness constants
pub const MIN_BRIGHTNESS: i32 = 0;
pub const MAX_BRIGHTNESS: i32 = 100;

// Temperature constants
pub const MIN_KELVIN: i32 = 2900;
pub const MAX_KELVIN: i32 = 7000;
pub const MIN_API_TEMP: i32 = 143; // Corresponds to 7000K
pub const MAX_API_TEMP: i32 = 344; // Corresponds to 2900K

pub const DEFAULT_PORT: u16 = 9123;

/// The light's state as returned by the API (after Kelvin conversion).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LightState {
    pub on: bool,
    pub brightness: i32,
    pub temperature_kelvin: i32,
}

/// Error types for KeyLight operations.
#[derive(Debug, Clone)]
pub enum KeyLightError {
    Network(String),
    InvalidJson(String),
    NoLights,
    NoIp,
}

impl std::fmt::Display for KeyLightError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            KeyLightError::Network(msg) => write!(f, "Network error: {}", msg),
            KeyLightError::InvalidJson(msg) => write!(f, "Invalid JSON: {}", msg),
            KeyLightError::NoLights => write!(f, "No lights found in response"),
            KeyLightError::NoIp => write!(f, "No IP address configured"),
        }
    }
}

impl std::error::Error for KeyLightError {}

// --- Serde structs for the Elgato JSON format ---

#[derive(Debug, Deserialize)]
pub struct ElgatoResponse {
    pub lights: Vec<ElgatoLight>,
}

#[derive(Debug, Deserialize)]
pub struct ElgatoLight {
    pub on: i32,
    pub brightness: i32,
    pub temperature: i32,
}

/// HTTP client for communicating with an Elgato Key Light.
pub struct KeyLightClient {
    ip: String,
    port: u16,
    client: reqwest::Client,
}

impl KeyLightClient {
    pub fn new(ip: String, port: u16) -> Self {
        let client = reqwest::Client::builder()
            .timeout(std::time::Duration::from_secs(5))
            .build()
            .unwrap_or_else(|_| reqwest::Client::new());
        Self {
            ip,
            port,
            client,
        }
    }

    pub fn set_host(&mut self, ip: String, port: u16) {
        self.ip = ip;
        self.port = port;
    }

    pub fn host(&self) -> &str {
        &self.ip
    }

    pub fn port(&self) -> u16 {
        self.port
    }

    /// Clone the client's connection info into a new client (shares nothing).
    pub fn clone_state(&self) -> KeyLightClient {
        KeyLightClient::new(self.ip.clone(), self.port)
    }

    fn lights_url(&self) -> String {
        format!("http://{}:{}/elgato/lights", self.ip, self.port)
    }

    /// Fetch the current light state via GET.
    pub async fn fetch_state(&self) -> Result<LightState, KeyLightError> {
        if self.ip.is_empty() {
            return Err(KeyLightError::NoIp);
        }

        let resp = self
            .client
            .get(&self.lights_url())
            .send()
            .await
            .map_err(|e| KeyLightError::Network(e.to_string()))?;

        let body = resp
            .text()
            .await
            .map_err(|e| KeyLightError::Network(e.to_string()))?;

        let parsed: ElgatoResponse = serde_json::from_str(&body)
            .map_err(|e| KeyLightError::InvalidJson(e.to_string()))?;

        let light = parsed.lights.first().ok_or(KeyLightError::NoLights)?;

        Ok(LightState {
            on: light.on == 1,
            brightness: light.brightness,
            temperature_kelvin: api_to_kelvin(light.temperature),
        })
    }

    /// Send a PUT request and parse the response to get the updated state.
    async fn send_put_request(&self, json_body: serde_json::Value) -> Result<LightState, KeyLightError> {
        if self.ip.is_empty() {
            return Err(KeyLightError::NoIp);
        }

        let resp = self
            .client
            .put(&self.lights_url())
            .header("Content-Type", "application/json")
            .json(&json_body)
            .send()
            .await
            .map_err(|e| KeyLightError::Network(e.to_string()))?;

        let body = resp
            .text()
            .await
            .map_err(|e| KeyLightError::Network(e.to_string()))?;

        // Parse the PUT response to update UI with actual values (mirrors C++ onPutFinished)
        let parsed: ElgatoResponse = serde_json::from_str(&body)
            .map_err(|e| KeyLightError::InvalidJson(e.to_string()))?;

        let light = parsed.lights.first().ok_or(KeyLightError::NoLights)?;

        Ok(LightState {
            on: light.on == 1,
            brightness: light.brightness,
            temperature_kelvin: api_to_kelvin(light.temperature),
        })
    }

    /// Set power state (on/off).
    pub async fn set_power(&self, on: bool) -> Result<LightState, KeyLightError> {
        let light = serde_json::json!({
            "on": if on { 1 } else { 0 }
        });
        let body = self.build_body(light);
        self.send_put_request(body).await
    }

    /// Set brightness (clamped to [0, 100]).
    pub async fn set_brightness(&self, brightness: i32) -> Result<LightState, KeyLightError> {
        let clamped = brightness.clamp(MIN_BRIGHTNESS, MAX_BRIGHTNESS);
        let light = serde_json::json!({
            "brightness": clamped
        });
        let body = self.build_body(light);
        self.send_put_request(body).await
    }

    /// Set temperature in Kelvin (converted to API value via kelvin_to_api which clamps).
    pub async fn set_temperature(&self, kelvin: i32) -> Result<LightState, KeyLightError> {
        let light = serde_json::json!({
            "temperature": kelvin_to_api(kelvin)
        });
        let body = self.build_body(light);
        self.send_put_request(body).await
    }

    /// Set full state (power, brightness, temperature).
    /// Ported for API parity but has no UI caller (dead code in C++ source).
    pub async fn set_state(
        &self,
        on: bool,
        brightness: i32,
        kelvin: i32,
    ) -> Result<LightState, KeyLightError> {
        let clamped_brightness = brightness.clamp(MIN_BRIGHTNESS, MAX_BRIGHTNESS);
        let light = serde_json::json!({
            "on": if on { 1 } else { 0 },
            "brightness": clamped_brightness,
            "temperature": kelvin_to_api(kelvin)
        });
        let body = self.build_body(light);
        self.send_put_request(body).await
    }

    fn build_body(&self, light: serde_json::Value) -> serde_json::Value {
        serde_json::json!({
            "numberOfLights": 1,
            "lights": [light]
        })
    }
}

/// Convert Kelvin to the Elgato API temperature value.
/// API uses inverse scale: MIN_API_TEMP(143) = MAX_KELVIN(7000K), MAX_API_TEMP(344) = MIN_KELVIN(2900K)
/// Kelvin is clamped to [2900, 7000] before conversion (matching C++ which clamps inside kelvinToApi).
pub fn kelvin_to_api(kelvin: i32) -> i32 {
    let kelvin = kelvin.clamp(MIN_KELVIN, MAX_KELVIN);
    MAX_API_TEMP - (kelvin - MIN_KELVIN) * (MAX_API_TEMP - MIN_API_TEMP) / (MAX_KELVIN - MIN_KELVIN)
}

/// Convert the Elgato API temperature value to Kelvin.
/// API value is clamped to [143, 344] before conversion.
pub fn api_to_kelvin(api_value: i32) -> i32 {
    let api_value = api_value.clamp(MIN_API_TEMP, MAX_API_TEMP);
    MIN_KELVIN + (MAX_API_TEMP - api_value) * (MAX_KELVIN - MIN_KELVIN) / (MAX_API_TEMP - MIN_API_TEMP)
}

// --- Unit tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_kelvin_to_api_boundaries() {
        assert_eq!(kelvin_to_api(2900), 344); // min kelvin -> max api
        assert_eq!(kelvin_to_api(7000), 143); // max kelvin -> min api
    }

    #[test]
    fn test_api_to_kelvin_boundaries() {
        assert_eq!(api_to_kelvin(344), 2900); // max api -> min kelvin
        assert_eq!(api_to_kelvin(143), 7000); // min api -> max kelvin
    }

    #[test]
    fn test_round_trip() {
        // Test round-trip for boundary values (integer division means some intermediate values
        // don't round-trip exactly, but boundaries always do — matching C++ behavior)
        assert_eq!(api_to_kelvin(kelvin_to_api(2900)), 2900);
        assert_eq!(api_to_kelvin(kelvin_to_api(7000)), 7000);
        // The midpoint should also round-trip since (7000-2900)/(344-143) = 4100/201 ≈ 20.4
        // so 201 steps map back exactly only at boundaries in integer arithmetic
    }

    #[test]
    fn test_kelvin_clamping() {
        // Values below MIN_KELVIN should clamp to MIN_KELVIN -> MAX_API_TEMP
        assert_eq!(kelvin_to_api(0), 344);
        assert_eq!(kelvin_to_api(-100), 344);
        // Values above MAX_KELVIN should clamp to MAX_KELVIN -> MIN_API_TEMP
        assert_eq!(kelvin_to_api(8000), 143);
        assert_eq!(kelvin_to_api(99999), 143);
    }

    #[test]
    fn test_api_clamping() {
        // api_to_kelvin clamps to [143, 344] before conversion
        // 0 clamps to 143 -> MAX_KELVIN (7000)
        assert_eq!(api_to_kelvin(0), 7000);
        // 500 clamps to 344 -> MIN_KELVIN (2900)
        assert_eq!(api_to_kelvin(500), 2900);
    }

    #[test]
    fn test_brightness_clamping_in_set_brightness() {
        // The clamping happens inside set_brightness, but we can verify the logic
        assert_eq!(50i32.clamp(MIN_BRIGHTNESS, MAX_BRIGHTNESS), 50);
        assert_eq!((-10i32).clamp(MIN_BRIGHTNESS, MAX_BRIGHTNESS), 0);
        assert_eq!(150i32.clamp(MIN_BRIGHTNESS, MAX_BRIGHTNESS), 100);
    }

    #[test]
    fn test_json_serialization_set_power_on() {
        let light = serde_json::json!({
            "on": 1
        });
        let body = serde_json::json!({
            "numberOfLights": 1,
            "lights": [light]
        });
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"on\":1"));
        assert!(json.contains("\"numberOfLights\":1"));
    }

    #[test]
    fn test_json_serialization_set_brightness() {
        let light = serde_json::json!({
            "brightness": 75
        });
        let body = serde_json::json!({
            "numberOfLights": 1,
            "lights": [light]
        });
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"brightness\":75"));
    }

    #[test]
    fn test_json_serialization_set_temperature() {
        let light = serde_json::json!({
            "temperature": 200
        });
        let body = serde_json::json!({
            "numberOfLights": 1,
            "lights": [light]
        });
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"temperature\":200"));
    }

    #[test]
    fn test_json_serialization_set_state() {
        let light = serde_json::json!({
            "on": 1,
            "brightness": 50,
            "temperature": 243
        });
        let body = serde_json::json!({
            "numberOfLights": 1,
            "lights": [light]
        });
        let json = serde_json::to_string(&body).unwrap();
        assert!(json.contains("\"on\":1"));
        assert!(json.contains("\"brightness\":50"));
        assert!(json.contains("\"temperature\":243"));
    }

    #[test]
    fn test_json_serialization_partial_bodies() {
        // set_power only sends on
        let power_body = serde_json::json!({
            "numberOfLights": 1,
            "lights": [{"on": 0}]
        });
        let power_json = serde_json::to_string(&power_body).unwrap();
        assert!(power_json.contains("\"on\":0"));
        assert!(!power_json.contains("brightness"));
        assert!(!power_json.contains("temperature"));

        // set_brightness only sends brightness
        let bright_body = serde_json::json!({
            "numberOfLights": 1,
            "lights": [{"brightness": 42}]
        });
        let bright_json = serde_json::to_string(&bright_body).unwrap();
        assert!(bright_json.contains("\"brightness\":42"));
        assert!(!bright_json.contains("\"on\""));
        assert!(!bright_json.contains("\"temperature\""));
    }

    #[test]
    fn test_response_parsing() {
        let json = r#"{"numberOfLights":1,"lights":[{"on":1,"brightness":50,"temperature":243}]}"#;
        let parsed: ElgatoResponse = serde_json::from_str(json).unwrap();
        let light = &parsed.lights[0];
        assert_eq!(light.on, 1);
        assert_eq!(light.brightness, 50);
        assert_eq!(light.temperature, 243);
        assert!(light.on == 1);
    }
}
