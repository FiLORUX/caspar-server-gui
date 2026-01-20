// AMCP TCP client
// Handles connection and communication with CasparCG server

use serde::{Deserialize, Serialize};
use std::sync::Arc;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::TcpStream;
use tokio::sync::Mutex;

/// AMCP client for communicating with CasparCG server
pub struct AmcpClient {
    stream: Option<Arc<Mutex<TcpStream>>>,
    host: String,
    port: u16,
}

impl AmcpClient {
    /// Create a new disconnected client
    pub fn new() -> Self {
        Self {
            stream: None,
            host: String::new(),
            port: 0,
        }
    }

    /// Check if connected
    pub fn is_connected(&self) -> bool {
        self.stream.is_some()
    }

    /// Get connection info
    pub fn connection_info(&self) -> Option<(String, u16)> {
        if self.is_connected() {
            Some((self.host.clone(), self.port))
        } else {
            None
        }
    }

    /// Connect to CasparCG server
    pub async fn connect(&mut self, host: &str, port: u16) -> Result<(), AmcpError> {
        let addr = format!("{}:{}", host, port);
        let stream = TcpStream::connect(&addr).await.map_err(|e| {
            AmcpError::Connection(format!("Failed to connect to {}: {}", addr, e))
        })?;

        self.stream = Some(Arc::new(Mutex::new(stream)));
        self.host = host.to_string();
        self.port = port;

        Ok(())
    }

    /// Disconnect from server
    pub async fn disconnect(&mut self) {
        self.stream = None;
        self.host.clear();
        self.port = 0;
    }

    /// Send a command and receive response
    pub async fn send_command(&self, command: &str) -> Result<AmcpResponse, AmcpError> {
        let stream = self.stream.as_ref().ok_or(AmcpError::NotConnected)?;
        let mut stream = stream.lock().await;

        // Send command with CRLF terminator
        let cmd = format!("{}\r\n", command);
        stream.write_all(cmd.as_bytes()).await.map_err(|e| {
            AmcpError::Send(format!("Failed to send command: {}", e))
        })?;

        // Read response
        let mut reader = BufReader::new(&mut *stream);
        let mut response_line = String::new();
        reader.read_line(&mut response_line).await.map_err(|e| {
            AmcpError::Receive(format!("Failed to read response: {}", e))
        })?;

        // Parse response code
        let response_line = response_line.trim();
        let (code, message) = parse_response_line(response_line)?;

        // Check if multi-line response
        if code >= 200 && code < 300 {
            // Success with possible data
            if response_line.ends_with(" OK") || message.is_empty() {
                return Ok(AmcpResponse {
                    code,
                    message: message.to_string(),
                    data: None,
                });
            }

            // Read multi-line data until empty line
            let mut data = Vec::new();
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).await.map_err(|e| {
                    AmcpError::Receive(format!("Failed to read data: {}", e))
                })?;

                let line = line.trim_end();
                if line.is_empty() {
                    break;
                }
                data.push(line.to_string());
            }

            Ok(AmcpResponse {
                code,
                message: message.to_string(),
                data: if data.is_empty() { None } else { Some(data.join("\n")) },
            })
        } else {
            // Error response
            Ok(AmcpResponse {
                code,
                message: message.to_string(),
                data: None,
            })
        }
    }
}

impl Default for AmcpClient {
    fn default() -> Self {
        Self::new()
    }
}

/// Parse first line of AMCP response
fn parse_response_line(line: &str) -> Result<(u16, &str), AmcpError> {
    // Format: "CODE MESSAGE" or "CODE"
    let parts: Vec<&str> = line.splitn(2, ' ').collect();

    let code = parts.first()
        .ok_or_else(|| AmcpError::Protocol("Empty response".to_string()))?
        .parse::<u16>()
        .map_err(|_| AmcpError::Protocol(format!("Invalid response code: {}", line)))?;

    let message = parts.get(1).unwrap_or(&"");

    Ok((code, message))
}

/// AMCP response from server
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AmcpResponse {
    pub code: u16,
    pub message: String,
    pub data: Option<String>,
}

impl AmcpResponse {
    /// Check if response indicates success
    pub fn is_success(&self) -> bool {
        self.code >= 200 && self.code < 300
    }

    /// Check if response indicates error
    pub fn is_error(&self) -> bool {
        self.code >= 400
    }
}

/// AMCP client errors
#[derive(Debug, thiserror::Error)]
pub enum AmcpError {
    #[error("Not connected to server")]
    NotConnected,
    #[error("Connection error: {0}")]
    Connection(String),
    #[error("Send error: {0}")]
    Send(String),
    #[error("Receive error: {0}")]
    Receive(String),
    #[error("Protocol error: {0}")]
    Protocol(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_response_line() {
        let (code, msg) = parse_response_line("200 OK").unwrap();
        assert_eq!(code, 200);
        assert_eq!(msg, "OK");

        let (code, msg) = parse_response_line("201 VERSION OK").unwrap();
        assert_eq!(code, 201);
        assert_eq!(msg, "VERSION OK");

        let (code, msg) = parse_response_line("404 ERROR").unwrap();
        assert_eq!(code, 404);
        assert_eq!(msg, "ERROR");
    }
}
