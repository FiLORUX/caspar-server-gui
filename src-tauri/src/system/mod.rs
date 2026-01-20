// System information module
// Detects installed software versions and hardware

pub mod ndi;
pub mod scanner;

pub use ndi::get_ndi_version;
pub use scanner::get_scanner_version;

use serde::{Deserialize, Serialize};

/// Combined system information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemVersions {
    /// CasparCG server version (from AMCP)
    pub caspar_version: Option<String>,
    /// DeckLink driver version
    pub decklink_version: Option<String>,
    /// NDI Tools version
    pub ndi_version: Option<String>,
    /// Media Scanner version
    pub scanner_version: Option<String>,
}

impl Default for SystemVersions {
    fn default() -> Self {
        Self {
            caspar_version: None,
            decklink_version: None,
            ndi_version: None,
            scanner_version: None,
        }
    }
}

/// Collect all system version information
pub async fn collect_system_info() -> SystemVersions {
    SystemVersions {
        caspar_version: None, // Set by AMCP connection
        decklink_version: crate::decklink::get_driver_version().ok().flatten(),
        ndi_version: get_ndi_version(),
        scanner_version: get_scanner_version(None).await,
    }
}
