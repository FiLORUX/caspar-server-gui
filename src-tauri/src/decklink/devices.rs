// DeckLink device enumeration and information
// Provides device listing and basic info retrieval

use serde::{Deserialize, Serialize};

/// Information about a DeckLink device
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckLinkDevice {
    /// Device index (1-based, for CasparCG config)
    pub index: u32,
    /// Persistent ID for identifying device across reboots
    pub persistent_id: String,
    /// Model name (e.g., "DeckLink Duo 2")
    pub model_name: String,
    /// Display name (user-configurable)
    pub display_name: String,
    /// Whether device supports duplex mode configuration
    pub supports_duplex: bool,
    /// Current duplex mode if applicable
    pub duplex_mode: Option<String>,
    /// Number of SDI inputs
    pub sdi_inputs: u32,
    /// Number of SDI outputs
    pub sdi_outputs: u32,
    /// Whether device supports internal keying
    pub supports_internal_keying: bool,
    /// Whether device supports external keying
    pub supports_external_keying: bool,
}

impl DeckLinkDevice {
    /// Check if this is a Duo or Quad card (supports duplex)
    pub fn is_multi_port(&self) -> bool {
        self.model_name.contains("Duo") || self.model_name.contains("Quad")
    }
}

/// Error type for DeckLink operations
#[derive(Debug, thiserror::Error)]
pub enum DeckLinkError {
    #[error("DeckLink SDK not available")]
    SdkNotAvailable,
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Failed to enumerate devices: {0}")]
    EnumerationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

/// List all available DeckLink devices
///
/// On systems without DeckLink SDK installed, returns an empty list
#[cfg(feature = "decklink")]
pub fn list_devices() -> Result<Vec<DeckLinkDevice>, DeckLinkError> {
    // This would use the actual decklink-rs crate when compiled with the feature
    // For now, return placeholder that would be filled by the SDK
    Err(DeckLinkError::SdkNotAvailable)
}

#[cfg(not(feature = "decklink"))]
pub fn list_devices() -> Result<Vec<DeckLinkDevice>, DeckLinkError> {
    // Return mock data for development/testing
    // In production, this would return an empty list
    Ok(vec![
        DeckLinkDevice {
            index: 1,
            persistent_id: "0x12345678".to_string(),
            model_name: "DeckLink Duo 2".to_string(),
            display_name: "DeckLink Duo 2 (1)".to_string(),
            supports_duplex: true,
            duplex_mode: Some("half".to_string()),
            sdi_inputs: 2,
            sdi_outputs: 2,
            supports_internal_keying: false,
            supports_external_keying: true,
        },
        DeckLinkDevice {
            index: 2,
            persistent_id: "0x12345679".to_string(),
            model_name: "DeckLink Duo 2".to_string(),
            display_name: "DeckLink Duo 2 (2)".to_string(),
            supports_duplex: true,
            duplex_mode: Some("half".to_string()),
            sdi_inputs: 2,
            sdi_outputs: 2,
            supports_internal_keying: false,
            supports_external_keying: true,
        },
        DeckLinkDevice {
            index: 3,
            persistent_id: "0x87654321".to_string(),
            model_name: "DeckLink Mini Monitor 4K".to_string(),
            display_name: "DeckLink Mini Monitor 4K".to_string(),
            supports_duplex: false,
            duplex_mode: None,
            sdi_inputs: 0,
            sdi_outputs: 1,
            supports_internal_keying: false,
            supports_external_keying: false,
        },
    ])
}

/// Get information about a specific device by persistent ID
pub fn get_device_by_id(persistent_id: &str) -> Result<DeckLinkDevice, DeckLinkError> {
    let devices = list_devices()?;
    devices
        .into_iter()
        .find(|d| d.persistent_id == persistent_id)
        .ok_or_else(|| DeckLinkError::DeviceNotFound(persistent_id.to_string()))
}

/// Get information about a specific device by index
pub fn get_device_by_index(index: u32) -> Result<DeckLinkDevice, DeckLinkError> {
    let devices = list_devices()?;
    devices
        .into_iter()
        .find(|d| d.index == index)
        .ok_or_else(|| DeckLinkError::DeviceNotFound(format!("index {}", index)))
}

/// Get the DeckLink driver version
pub fn get_driver_version() -> Result<Option<String>, DeckLinkError> {
    #[cfg(feature = "decklink")]
    {
        // Would use decklink-rs to get actual version
        Err(DeckLinkError::SdkNotAvailable)
    }

    #[cfg(not(feature = "decklink"))]
    {
        // Return mock version for development
        Ok(Some("12.5.1".to_string()))
    }
}
