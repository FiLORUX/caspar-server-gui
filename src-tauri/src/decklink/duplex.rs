// DeckLink duplex mode configuration
// Handles duplex mode for Duo 2 and Quad 2 cards

use serde::{Deserialize, Serialize};

use super::{DeckLinkDevice, DeckLinkError};

/// Duplex mode configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum DuplexMode {
    /// Full duplex: SDI 1+2 as key/fill pair
    Full,
    /// Half duplex: SDI 1 and SDI 2 as independent I/O
    Half,
}

impl DuplexMode {
    /// Get display name for the mode
    pub fn display_name(&self) -> &'static str {
        match self {
            DuplexMode::Full => "Full Duplex (Key/Fill pair)",
            DuplexMode::Half => "Half Duplex (Independent I/O)",
        }
    }

    /// Get description for the mode
    pub fn description(&self) -> &'static str {
        match self {
            DuplexMode::Full => {
                "SDI 1 and SDI 2 operate as a key/fill pair for external keying. \
                 This provides the best performance for graphics playout."
            }
            DuplexMode::Half => {
                "SDI 1 and SDI 2 operate as independent inputs/outputs. \
                 This allows simultaneous input and output on the same card."
            }
        }
    }
}

impl std::fmt::Display for DuplexMode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            DuplexMode::Full => write!(f, "full"),
            DuplexMode::Half => write!(f, "half"),
        }
    }
}

impl std::str::FromStr for DuplexMode {
    type Err = DeckLinkError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "full" => Ok(DuplexMode::Full),
            "half" => Ok(DuplexMode::Half),
            _ => Err(DeckLinkError::ConfigError(format!(
                "Invalid duplex mode: {}. Expected 'full' or 'half'",
                s
            ))),
        }
    }
}

/// Get the current duplex mode for a device
#[cfg(feature = "decklink")]
pub fn get_duplex_mode(_device: &DeckLinkDevice) -> Result<Option<DuplexMode>, DeckLinkError> {
    // Would use decklink-rs to get actual mode
    Err(DeckLinkError::SdkNotAvailable)
}

#[cfg(not(feature = "decklink"))]
pub fn get_duplex_mode(device: &DeckLinkDevice) -> Result<Option<DuplexMode>, DeckLinkError> {
    if !device.supports_duplex {
        return Ok(None);
    }

    // Return mock mode for development
    Ok(device.duplex_mode.as_ref().and_then(|s| s.parse().ok()))
}

/// Set the duplex mode for a device
///
/// Note: This requires administrator privileges on most systems
/// and a system restart to take effect
#[cfg(feature = "decklink")]
pub fn set_duplex_mode(
    _persistent_id: &str,
    _mode: DuplexMode,
) -> Result<(), DeckLinkError> {
    // Would use decklink-rs to set actual mode
    Err(DeckLinkError::SdkNotAvailable)
}

#[cfg(not(feature = "decklink"))]
pub fn set_duplex_mode(
    _persistent_id: &str,
    _mode: DuplexMode,
) -> Result<(), DeckLinkError> {
    // Mock implementation for development
    // In production, this would use the DeckLink SDK
    Ok(())
}

/// Check if duplex mode change requires restart
pub fn requires_restart_for_mode_change() -> bool {
    // DeckLink SDK requires restart for duplex mode changes
    true
}

/// Connector mapping for multi-port cards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectorMapping {
    /// SDI 1 direction
    pub sdi1: ConnectorDirection,
    /// SDI 2 direction
    pub sdi2: ConnectorDirection,
    /// SDI 3 direction (for Quad cards)
    pub sdi3: Option<ConnectorDirection>,
    /// SDI 4 direction (for Quad cards)
    pub sdi4: Option<ConnectorDirection>,
}

/// Direction of a connector
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum ConnectorDirection {
    Input,
    Output,
}

impl ConnectorMapping {
    /// Create default mapping for half-duplex Duo 2
    pub fn duo2_half_duplex() -> Self {
        Self {
            sdi1: ConnectorDirection::Output,
            sdi2: ConnectorDirection::Output,
            sdi3: None,
            sdi4: None,
        }
    }

    /// Create default mapping for full-duplex Duo 2
    pub fn duo2_full_duplex() -> Self {
        Self {
            sdi1: ConnectorDirection::Output,
            sdi2: ConnectorDirection::Output,
            sdi3: None,
            sdi4: None,
        }
    }
}
