// Media Scanner version detection
// Checks CasparCG Media Scanner status

use std::net::TcpListener;
use std::time::Duration;

/// The loopback host the media scanner binds to and CasparCG queries. Loopback
/// only — the scanner never needs to be reachable off-box, and binding all
/// interfaces invites clashes with co-hosted services on a shared machine.
pub const HOST: &str = "127.0.0.1";

/// Preferred media-scanner port. Deliberately NOT 8000. That is CasparCG's stock
/// port, but using it here is dangerous: on Windows a loopback bind to
/// 127.0.0.1:8000 SUCCEEDS even while another process already holds 0.0.0.0:8000
/// (a specific address wins over a wildcard without SO_EXCLUSIVEADDRUSE), so the
/// scanner would silently hijack localhost:8000 — which on a shared box can be a
/// live service behind a reverse proxy or tunnel. We never use 8000.
pub const PREFERRED_PORT: u16 = 8010;

/// Choose a free loopback port for the media scanner. A port counts as free only
/// if a WILDCARD (0.0.0.0) bind succeeds. This is the crux: a specific-address
/// bind (127.0.0.1) wrongly succeeds on Windows when another process already
/// holds the same port on 0.0.0.0, so probing the wildcard is what actually
/// detects an in-use port and stops the scanner stealing another service's
/// traffic. The scanner itself still listens on loopback only (`HOST`); the
/// wildcard is used purely to probe.
pub fn pick_port() -> u16 {
    const PROBE_ANY: &str = "0.0.0.0";
    if TcpListener::bind((PROBE_ANY, PREFERRED_PORT)).is_ok() {
        return PREFERRED_PORT;
    }
    // Let the OS hand out a port that is free on every interface.
    TcpListener::bind((PROBE_ANY, 0))
        .ok()
        .and_then(|listener| listener.local_addr().ok())
        .map(|addr| addr.port())
        .unwrap_or(PREFERRED_PORT)
}

/// Get Media Scanner version
///
/// Tries to connect to the scanner's HTTP endpoint to get version info
pub async fn get_scanner_version(scanner_url: Option<&str>) -> Option<String> {
    let default_url = format!("http://{}:{}/version", HOST, PREFERRED_PORT);
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
            port: PREFERRED_PORT,
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
