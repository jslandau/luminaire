use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

/// Persisted application settings.
/// A brightness/temperature of -1 means "unset, skip" (matching C++ sentinel semantics).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub ip_address: String,
    pub brightness: i32,
    pub temperature: i32,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            ip_address: String::new(),
            brightness: -1,
            temperature: -1,
        }
    }
}

impl Config {
    /// Get the path to the config directory.
    /// macOS: ~/Library/Application Support/Luminaire
    /// Linux: ~/.config/Luminaire
    fn config_dir() -> Option<PathBuf> {
        directories::ProjectDirs::from("", "Luminaire", "Luminaire")
            .map(|dirs| dirs.config_dir().to_path_buf())
    }

    fn config_path() -> Option<PathBuf> {
        Self::config_dir().map(|dir| dir.join("config.toml"))
    }

    /// Load settings from disk. Returns defaults if the file doesn't exist.
    pub fn load() -> Self {
        let path = match Self::config_path() {
            Some(p) => p,
            None => return Self::default(),
        };

        if !path.exists() {
            return Self::default();
        }

        match fs::read_to_string(&path) {
            Ok(contents) => toml::from_str(&contents).unwrap_or_default(),
            Err(_) => Self::default(),
        }
    }

    /// Save settings to disk immediately (TOML serialize + write + flush).
    pub fn save(&self) {
        let dir = match Self::config_dir() {
            Some(d) => d,
            None => return,
        };

        // Ensure the directory exists
        if let Err(e) = fs::create_dir_all(&dir) {
            eprintln!("Failed to create config directory: {}", e);
            return;
        }

        let path = dir.join("config.toml");
        let contents = match toml::to_string_pretty(self) {
            Ok(c) => c,
            Err(e) => {
                eprintln!("Failed to serialize config: {}", e);
                return;
            }
        };

        match fs::File::create(&path) {
            Ok(mut file) => {
                if let Err(e) = file.write_all(contents.as_bytes()) {
                    eprintln!("Failed to write config: {}", e);
                } else if let Err(e) = file.flush() {
                    eprintln!("Failed to flush config: {}", e);
                }
            }
            Err(e) => eprintln!("Failed to create config file: {}", e),
        }
    }
}

// --- Unit tests ---

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_values() {
        let config = Config::default();
        assert_eq!(config.ip_address, "");
        assert_eq!(config.brightness, -1);
        assert_eq!(config.temperature, -1);
    }

    #[test]
    fn test_serialization_round_trip() {
        let config = Config {
            ip_address: "192.168.1.100".to_string(),
            brightness: 75,
            temperature: 4500,
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.ip_address, "192.168.1.100");
        assert_eq!(deserialized.brightness, 75);
        assert_eq!(deserialized.temperature, 4500);
    }

    #[test]
    fn test_sentinel_values_round_trip() {
        let config = Config {
            ip_address: "".to_string(),
            brightness: -1,
            temperature: -1,
        };
        let toml_str = toml::to_string_pretty(&config).unwrap();
        let deserialized: Config = toml::from_str(&toml_str).unwrap();
        assert_eq!(deserialized.brightness, -1);
        assert_eq!(deserialized.temperature, -1);
    }
}
