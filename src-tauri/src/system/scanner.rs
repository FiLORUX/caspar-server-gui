// Media Scanner version detection
// Checks CasparCG Media Scanner status

use std::time::Duration;

/// Get Media Scanner version
///
/// Tries to connect to the scanner's HTTP endpoint to get version info
pub async fn get_scanner_version(scanner_url: Option<&str>) -> Option<String> {
    let url = scanner_url.unwrap_or("http://localhost:8000/version");

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
            host: "localhost".to_string(),
            port: 8000,
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
