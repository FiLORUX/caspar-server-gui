// Media Scanner version detection
// Checks CasparCG Media Scanner status

use std::net::TcpListener;
use std::time::Duration;

/// The loopback host the media scanner binds to and CasparCG queries. Loopback
/// only — the scanner never needs to be reachable off-box, and binding all
/// interfaces invites clashes with co-hosted services on a shared machine.
pub const HOST: &str = "127.0.0.1";

/// CasparCG's stock media-scanner port. Tried first so a box where it is free
/// behaves exactly like a stock install and matches operator expectations.
pub const DEFAULT_PORT: u16 = 8000;

/// First fallback when 8000 is taken — common on a shared box where another
/// local web service already owns it. When the scanner cannot bind its port the
/// server reaches the wrong app and every CLS/TLS/THUMBNAIL listing fails with
/// "Invalid Response", so falling back to a free port is what keeps media
/// browsing working.
pub const FALLBACK_PORT: u16 = 8010;

/// Choose a free loopback port for the media scanner. Tries the stock 8000
/// first, then 8010, then lets the OS assign any free port. Binding the same
/// loopback host the scanner is launched with means a successful probe here is a
/// reliable indicator the scanner can bind it a moment later.
pub fn pick_port() -> u16 {
    for candidate in [DEFAULT_PORT, FALLBACK_PORT] {
        if TcpListener::bind((HOST, candidate)).is_ok() {
            return candidate;
        }
    }
    TcpListener::bind((HOST, 0))
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
        .unwrap_or(FALLBACK_PORT)
}

/// Get Media Scanner version
///
/// Tries to connect to the scanner's HTTP endpoint to get version info
pub async fn get_scanner_version(scanner_url: Option<&str>) -> Option<String> {
    let default_url = format!("http://{}:{}/version", HOST, DEFAULT_PORT);
    let url = scanner_url.unwrap_or(&default_url);

    // Try HTTP request with short timeout
    match tokio::time::timeout(
        Duration::from_secs(2),
        fetch_scanner_version(url),
    )
    .await
    {
        Ok(result) => result.ok(),
        Err(_) => None, // Timeout
    }
}

async fn fetch_scanner_version(_url: &str) -> Result<String, ScannerError> {
    // In a full implementation, this would make an HTTP request
    // For now, return a placeholder
    //
    // The actual scanner exposes:
    // GET /version - returns version string
    // GET /status - returns scanning status

    // Mock implementation for development
    Ok("1.1.0".to_string())
}

/// Check if scanner is running
pub async fn is_scanner_running(scanner_url: Option<&str>) -> bool {
    get_scanner_version(scanner_url).await.is_some()
}

/// Scanner HTTP endpoint info
pub struct ScannerEndpoint {
    pub host: String,
    pub port: u16,
}

impl Default for ScannerEndpoint {
    fn default() -> Self {
        Self {
            host: HOST.to_string(),
            port: DEFAULT_PORT,
        }
    }
}

impl ScannerEndpoint {
    pub fn url(&self) -> String {
        format!("http://{}:{}", self.host, self.port)
    }
}

#[derive(Debug, thiserror::Error)]
pub enum ScannerError {
    #[error("Connection failed: {0}")]
    ConnectionFailed(String),
    #[error("Invalid response: {0}")]
    InvalidResponse(String),
    #[error("Scanner not running")]
    NotRunning,
}
