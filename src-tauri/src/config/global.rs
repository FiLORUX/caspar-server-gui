// Global configuration format
// Wraps CasparCG config and DeckLink device settings in a unified JSON format

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

use super::{CasparConfig, DeckLinkKeyer, DeckLinkLatency, VideoMode};

/// Connector mapping for DeckLink cards with multiple SDI ports
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorMode {
    Input,
    Output,
}

/// DeckLink device configuration stored in global config
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckLinkDeviceConfig {
    /// Persistent ID used to identify the card across reboots
    pub persistent_id: String,
    /// Model name (e.g., "DeckLink Duo 2")
    pub model_name: String,
    /// User-assigned label (e.g., "Graphics Fill")
    #[serde(skip_serializing_if = "Option::is_none")]
    pub label: Option<String>,
    /// Duplex mode for cards that support it (full/half)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub duplex_mode: Option<String>,
    /// Connector mapping for multi-port cards
    #[serde(skip_serializing_if = "Option::is_none")]
    pub connector_mapping: Option<std::collections::HashMap<String, ConnectorMode>>,
}

/// DeckLink section of global config
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct DeckLinkConfig {
    #[serde(default)]
    pub devices: Vec<DeckLinkDeviceConfig>,
}

/// Global configuration format that wraps everything
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Config format version
    pub version: String,
    /// User-friendly profile name
    pub name: String,
    /// Creation timestamp
    pub created: DateTime<Utc>,
    /// Last modification timestamp
    pub modified: DateTime<Utc>,
    /// CasparCG server configuration
    pub caspar: CasparConfig,
    /// DeckLink device configuration
    #[serde(default)]
    pub decklink: DeckLinkConfig,
}

impl Default for GlobalConfig {
    fn default() -> Self {
        Self {
            version: "1.0".to_string(),
            name: "Default Profile".to_string(),
            created: Utc::now(),
            modified: Utc::now(),
            caspar: CasparConfig::default(),
            decklink: DeckLinkConfig::default(),
        }
    }
}

impl GlobalConfig {
    /// Create a new global config with the given name
    pub fn new(name: impl Into<String>) -> Self {
        let now = Utc::now();
        Self {
            version: "1.0".to_string(),
            name: name.into(),
            created: now,
            modified: now,
            caspar: CasparConfig::default(),
            decklink: DeckLinkConfig::default(),
        }
    }

    /// Update the modified timestamp
    pub fn touch(&mut self) {
        self.modified = Utc::now();
    }

    /// Load from JSON file
    pub fn load_from_file(path: &std::path::Path) -> Result<Self, GlobalConfigError> {
        let content = std::fs::read_to_string(path)?;
        let config: GlobalConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save to JSON file, with embedded `_`-prefixed documentation.
    pub fn save_to_file(&self, path: &std::path::Path) -> Result<(), GlobalConfigError> {
        let content = serde_json::to_string_pretty(&self.documented_value()?)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Build the JSON value for this profile with embedded documentation: an
    /// `_about` note and an `_allowed` map of the accepted values for each
    /// enumerated field, generated from the application's own definitions so the
    /// lists can never drift from what the loader accepts. Keys starting with `_`
    /// are ignored when the profile is read back.
    fn documented_value(&self) -> Result<serde_json::Value, GlobalConfigError> {
        let mut value = serde_json::to_value(self)?;
        if let serde_json::Value::Object(map) = &mut value {
            map.insert(
                "_about".to_string(),
                serde_json::Value::String(
                    "CasparCG Server GUI profile. Keys starting with \"_\" are \
                     documentation only and are ignored on load; \"_allowed\" lists \
                     the accepted values for each enumerated field."
                        .to_string(),
                ),
            );
            map.insert("_allowed".to_string(), allowed_values());
        }
        Ok(value)
    }
}

/// Serialise a set of enum values to their on-the-wire string form.
fn allowed_strings<T: Serialize>(values: &[T]) -> Vec<String> {
    values
        .iter()
        .filter_map(|v| serde_json::to_value(v).ok())
        .filter_map(|v| v.as_str().map(str::to_string))
        .collect()
}

/// The accepted values for every enumerated profile field, generated from the
/// schema where the variants iterate cleanly. Consumer types and duplex modes
/// are small fixed sets (data-carrying enums) and are listed explicitly.
fn allowed_values() -> serde_json::Value {
    serde_json::json!({
        "caspar.channels[].video_mode": allowed_strings(&VideoMode::all()),
        "caspar.channels[].consumers[].type": ["decklink", "ndi", "screen", "system-audio"],
        "caspar.channels[].consumers[] (decklink).keyer": allowed_strings(&[
            DeckLinkKeyer::External,
            DeckLinkKeyer::ExternalSeparateDevice,
            DeckLinkKeyer::Internal,
            DeckLinkKeyer::Default,
        ]),
        "caspar.channels[].consumers[] (decklink).latency": allowed_strings(&[
            DeckLinkLatency::Normal,
            DeckLinkLatency::Low,
            DeckLinkLatency::Default,
        ]),
        "decklink.devices[].duplex_mode": ["full", "half"],
    })
}

/// GUI settings stored separately from profiles
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuiSettings {
    /// Path to CasparCG installation directory
    #[serde(skip_serializing_if = "Option::is_none")]
    pub caspar_path: Option<String>,
    /// Last used profile name
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_profile: Option<String>,
    /// Last AMCP host
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_host: Option<String>,
    /// Last AMCP port
    #[serde(skip_serializing_if = "Option::is_none")]
    pub last_port: Option<u16>,
    /// Whether server was running when GUI last closed
    #[serde(default)]
    pub last_server_was_running: bool,
    /// Window width
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_width: Option<u32>,
    /// Window height
    #[serde(skip_serializing_if = "Option::is_none")]
    pub window_height: Option<u32>,
}

impl Default for GuiSettings {
    fn default() -> Self {
        Self {
            caspar_path: None,
            last_profile: None,
            last_host: Some("localhost".to_string()),
            last_port: Some(5250),
            last_server_was_running: false,
            window_width: None,
            window_height: None,
        }
    }
}

impl GuiSettings {
    /// Get the path to the GUI settings file
    pub fn settings_path() -> Option<std::path::PathBuf> {
        dirs::config_dir().map(|p| p.join("caspar-server-gui").join("settings.json"))
    }

    /// Load settings from file
    pub fn load() -> Self {
        Self::settings_path()
            .and_then(|path| std::fs::read_to_string(&path).ok())
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
    }

    /// Save settings to file
    pub fn save(&self) -> Result<(), GlobalConfigError> {
        let path = Self::settings_path().ok_or_else(|| {
            GlobalConfigError::Io(std::io::Error::new(
                std::io::ErrorKind::NotFound,
                "Could not determine config directory",
            ))
        })?;

        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)?;
        }

        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(&path, content)?;
        Ok(())
    }

    /// Get the profiles directory based on CasparCG path
    pub fn profiles_dir(&self) -> Option<std::path::PathBuf> {
        self.caspar_path
            .as_ref()
            .map(|p| std::path::PathBuf::from(p).join("caspar-gui-profiles"))
    }
}

/// Errors that can occur when working with global config
#[derive(Debug, thiserror::Error)]
pub enum GlobalConfigError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("JSON parse error: {0}")]
    Json(#[from] serde_json::Error),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn documented_profile_round_trips() {
        let cfg = GlobalConfig::new("Test Profile");
        let value = cfg.documented_value().unwrap();

        // Documentation is embedded and generated from the schema.
        assert!(value.get("_about").is_some());
        assert!(value["_allowed"]["caspar.channels[].video_mode"]
            .as_array()
            .map(|modes| modes.iter().any(|v| v == "1080i5000"))
            .unwrap_or(false));

        // The `_`-prefixed notes are ignored on load, leaving the profile intact.
        let json = serde_json::to_string(&value).unwrap();
        let loaded: GlobalConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(loaded.name, "Test Profile");
        assert_eq!(loaded.version, cfg.version);
    }
}
