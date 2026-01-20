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
    /// User-assigned device label
    pub device_label: Option<String>,
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
    /// Whether device supports capture
    pub supports_capture: bool,
    /// Whether device supports playback
    pub supports_playback: bool,
    /// Maximum audio channels supported
    pub max_audio_channels: u32,
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
    #[error("DeckLink not initialised")]
    NotInitialised,
    #[error("DeckLink COM initialisation failed")]
    ComFailed,
    #[error("DeckLink drivers not installed")]
    NoDriver,
    #[error("Device not found: {0}")]
    DeviceNotFound(String),
    #[error("Failed to enumerate devices: {0}")]
    EnumerationFailed(String),
    #[error("Configuration error: {0}")]
    ConfigError(String),
}

// FFI bindings for DeckLink C wrapper
#[cfg(feature = "decklink")]
mod ffi {
    use std::os::raw::c_char;

    pub const DECKLINK_MAX_STRING_LENGTH: usize = 256;

    // Error codes from C wrapper
    pub const DECKLINK_OK: i32 = 0;
    pub const DECKLINK_ERROR_NOT_INITIALISED: i32 = -1;
    pub const DECKLINK_ERROR_COM_FAILED: i32 = -2;
    pub const DECKLINK_ERROR_NO_DRIVER: i32 = -3;
    pub const DECKLINK_ERROR_INVALID_INDEX: i32 = -4;
    pub const DECKLINK_ERROR_QUERY_FAILED: i32 = -5;

    // IO support flags
    pub const DECKLINK_IO_SUPPORT_CAPTURE: u32 = 1 << 0;
    pub const DECKLINK_IO_SUPPORT_PLAYBACK: u32 = 1 << 1;

    // Video connection flags
    pub const DECKLINK_VIDEO_CONNECTION_SDI: u32 = 1 << 0;

    #[repr(C)]
    #[derive(Debug)]
    pub struct DeckLinkDeviceInfo {
        pub index: i32,
        pub display_name: [c_char; DECKLINK_MAX_STRING_LENGTH],
        pub model_name: [c_char; DECKLINK_MAX_STRING_LENGTH],
        pub device_label: [c_char; DECKLINK_MAX_STRING_LENGTH],
        pub persistent_id: i64,
        pub device_group_id: i64,
        pub sub_device_index: i32,
        pub num_sub_devices: i32,
        pub video_input_connections: u32,
        pub video_output_connections: u32,
        pub audio_input_connections: u32,
        pub audio_output_connections: u32,
        pub io_support: u32,
        pub supports_internal_keying: bool,
        pub supports_external_keying: bool,
        pub supports_dual_link_sdi: bool,
        pub supports_quad_link_sdi: bool,
        pub supports_idle_output: bool,
        pub max_audio_channels: i32,
    }

    impl Default for DeckLinkDeviceInfo {
        fn default() -> Self {
            Self {
                index: 0,
                display_name: [0; DECKLINK_MAX_STRING_LENGTH],
                model_name: [0; DECKLINK_MAX_STRING_LENGTH],
                device_label: [0; DECKLINK_MAX_STRING_LENGTH],
                persistent_id: -1,
                device_group_id: -1,
                sub_device_index: 0,
                num_sub_devices: 0,
                video_input_connections: 0,
                video_output_connections: 0,
                audio_input_connections: 0,
                audio_output_connections: 0,
                io_support: 0,
                supports_internal_keying: false,
                supports_external_keying: false,
                supports_dual_link_sdi: false,
                supports_quad_link_sdi: false,
                supports_idle_output: false,
                max_audio_channels: 0,
            }
        }
    }

    extern "C" {
        pub fn decklink_init() -> i32;
        pub fn decklink_cleanup();
        pub fn decklink_get_device_count(count: *mut i32) -> i32;
        pub fn decklink_get_device_info(index: i32, info: *mut DeckLinkDeviceInfo) -> i32;
        pub fn decklink_get_api_version(version: *mut c_char, max_length: i32) -> i32;
    }

    /// Convert a C string buffer to a Rust String
    pub fn cstr_to_string(buf: &[c_char]) -> String {
        let bytes: Vec<u8> = buf
            .iter()
            .take_while(|&&c| c != 0)
            .map(|&c| c as u8)
            .collect();
        String::from_utf8_lossy(&bytes).into_owned()
    }
}

#[cfg(feature = "decklink")]
use std::sync::Once;

#[cfg(feature = "decklink")]
static INIT: Once = Once::new();

#[cfg(feature = "decklink")]
static mut INIT_RESULT: i32 = ffi::DECKLINK_ERROR_NOT_INITIALISED;

/// Initialise the DeckLink SDK (call once at startup)
#[cfg(feature = "decklink")]
pub fn init() -> Result<(), DeckLinkError> {
    unsafe {
        INIT.call_once(|| {
            INIT_RESULT = ffi::decklink_init();
        });

        match INIT_RESULT {
            ffi::DECKLINK_OK => Ok(()),
            ffi::DECKLINK_ERROR_COM_FAILED => Err(DeckLinkError::ComFailed),
            _ => Err(DeckLinkError::SdkNotAvailable),
        }
    }
}

#[cfg(not(feature = "decklink"))]
pub fn init() -> Result<(), DeckLinkError> {
    Ok(())
}

/// List all available DeckLink devices
#[cfg(feature = "decklink")]
pub fn list_devices() -> Result<Vec<DeckLinkDevice>, DeckLinkError> {
    // Ensure SDK is initialised
    init()?;

    unsafe {
        // Get device count
        let mut count: i32 = 0;
        let result = ffi::decklink_get_device_count(&mut count);

        match result {
            ffi::DECKLINK_OK => {}
            ffi::DECKLINK_ERROR_NOT_INITIALISED => return Err(DeckLinkError::NotInitialised),
            ffi::DECKLINK_ERROR_NO_DRIVER => return Err(DeckLinkError::NoDriver),
            _ => {
                return Err(DeckLinkError::EnumerationFailed(format!(
                    "Failed to get device count (error {})",
                    result
                )))
            }
        }

        // Enumerate devices
        let mut devices = Vec::with_capacity(count as usize);

        for i in 0..count {
            let mut info = ffi::DeckLinkDeviceInfo::default();
            let result = ffi::decklink_get_device_info(i, &mut info);

            if result == ffi::DECKLINK_OK {
                let display_name = ffi::cstr_to_string(&info.display_name);
                let model_name = ffi::cstr_to_string(&info.model_name);
                let device_label = ffi::cstr_to_string(&info.device_label);

                // Count SDI connections
                let sdi_inputs = count_sdi_connections(info.video_input_connections);
                let sdi_outputs = count_sdi_connections(info.video_output_connections);

                // Determine duplex support (Duo/Quad cards)
                let supports_duplex =
                    model_name.contains("Duo") || model_name.contains("Quad");

                let device = DeckLinkDevice {
                    index: (i + 1) as u32, // 1-based for CasparCG
                    persistent_id: format!("0x{:08X}", info.persistent_id),
                    model_name,
                    display_name,
                    device_label: if device_label.is_empty() {
                        None
                    } else {
                        Some(device_label)
                    },
                    supports_duplex,
                    duplex_mode: if supports_duplex {
                        Some("half".to_string()) // Default assumption
                    } else {
                        None
                    },
                    sdi_inputs,
                    sdi_outputs,
                    supports_internal_keying: info.supports_internal_keying,
                    supports_external_keying: info.supports_external_keying,
                    supports_capture: (info.io_support & ffi::DECKLINK_IO_SUPPORT_CAPTURE) != 0,
                    supports_playback: (info.io_support & ffi::DECKLINK_IO_SUPPORT_PLAYBACK) != 0,
                    max_audio_channels: info.max_audio_channels as u32,
                };

                devices.push(device);
            }
        }

        Ok(devices)
    }
}

#[cfg(feature = "decklink")]
fn count_sdi_connections(connections: u32) -> u32 {
    if (connections & ffi::DECKLINK_VIDEO_CONNECTION_SDI) != 0 {
        1 // At least 1, could count more accurately with additional flags
    } else {
        0
    }
}

#[cfg(not(feature = "decklink"))]
pub fn list_devices() -> Result<Vec<DeckLinkDevice>, DeckLinkError> {
    // Return mock data for development/testing
    Ok(vec![
        DeckLinkDevice {
            index: 1,
            persistent_id: "0x12345678".to_string(),
            model_name: "DeckLink Duo 2".to_string(),
            display_name: "DeckLink Duo 2 (1)".to_string(),
            device_label: None,
            supports_duplex: true,
            duplex_mode: Some("half".to_string()),
            sdi_inputs: 2,
            sdi_outputs: 2,
            supports_internal_keying: false,
            supports_external_keying: true,
            supports_capture: true,
            supports_playback: true,
            max_audio_channels: 16,
        },
        DeckLinkDevice {
            index: 2,
            persistent_id: "0x12345679".to_string(),
            model_name: "DeckLink Duo 2".to_string(),
            display_name: "DeckLink Duo 2 (2)".to_string(),
            device_label: None,
            supports_duplex: true,
            duplex_mode: Some("half".to_string()),
            sdi_inputs: 2,
            sdi_outputs: 2,
            supports_internal_keying: false,
            supports_external_keying: true,
            supports_capture: true,
            supports_playback: true,
            max_audio_channels: 16,
        },
        DeckLinkDevice {
            index: 3,
            persistent_id: "0x87654321".to_string(),
            model_name: "DeckLink Mini Monitor 4K".to_string(),
            display_name: "DeckLink Mini Monitor 4K".to_string(),
            device_label: Some("PGM Output".to_string()),
            supports_duplex: false,
            duplex_mode: None,
            sdi_inputs: 0,
            sdi_outputs: 1,
            supports_internal_keying: false,
            supports_external_keying: false,
            supports_capture: false,
            supports_playback: true,
            max_audio_channels: 8,
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

/// Get the DeckLink API version
#[cfg(feature = "decklink")]
pub fn get_api_version() -> Result<Option<String>, DeckLinkError> {
    init()?;

    unsafe {
        let mut version = [0i8; 32];
        let result = ffi::decklink_get_api_version(version.as_mut_ptr(), 32);

        if result == ffi::DECKLINK_OK {
            Ok(Some(ffi::cstr_to_string(&version)))
        } else {
            Ok(None)
        }
    }
}

#[cfg(not(feature = "decklink"))]
pub fn get_api_version() -> Result<Option<String>, DeckLinkError> {
    // Return mock version for development
    Ok(Some("15.3 (mock)".to_string()))
}

/// Get the DeckLink driver version (Desktop Video version)
pub fn get_driver_version() -> Result<Option<String>, DeckLinkError> {
    // Driver version detection would require additional SDK calls
    // For now, use API version as a proxy
    get_api_version()
}
