// AMCP command implementations
// High-level wrappers for common AMCP commands

use super::{AmcpClient, AmcpError, AmcpResponse};
use serde::{Deserialize, Serialize};

/// System information from CasparCG INFO SYSTEM command
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemInfo {
    pub version: Option<String>,
    pub channels: u32,
}

impl AmcpClient {
    /// Get CasparCG server version
    pub async fn version(&self) -> Result<String, AmcpError> {
        let response = self.send_command("VERSION").await?;

        if response.is_success() {
            // Version is typically in the data section
            Ok(response.data.unwrap_or_else(|| response.message.clone()))
        } else {
            Err(AmcpError::Protocol(format!(
                "VERSION command failed: {} {}",
                response.code, response.message
            )))
        }
    }

    /// Get server version for a specific component
    pub async fn version_component(&self, component: &str) -> Result<String, AmcpError> {
        let cmd = format!("VERSION {}", component);
        let response = self.send_command(&cmd).await?;

        if response.is_success() {
            Ok(response.data.unwrap_or_else(|| response.message.clone()))
        } else {
            Err(AmcpError::Protocol(format!(
                "VERSION {} command failed: {} {}",
                component, response.code, response.message
            )))
        }
    }

    /// Get system information
    pub async fn info_system(&self) -> Result<String, AmcpError> {
        let response = self.send_command("INFO SYSTEM").await?;

        if response.is_success() {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(AmcpError::Protocol(format!(
                "INFO SYSTEM command failed: {} {}",
                response.code, response.message
            )))
        }
    }

    /// Get channel information
    pub async fn info_channel(&self, channel: u32) -> Result<String, AmcpError> {
        let cmd = format!("INFO {}", channel);
        let response = self.send_command(&cmd).await?;

        if response.is_success() {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(AmcpError::Protocol(format!(
                "INFO {} command failed: {} {}",
                channel, response.code, response.message
            )))
        }
    }

    /// Get template host information
    pub async fn info_template(&self, channel: u32, layer: u32) -> Result<String, AmcpError> {
        let cmd = format!("INFO {}-{}", channel, layer);
        let response = self.send_command(&cmd).await?;

        if response.is_success() {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(AmcpError::Protocol(format!(
                "INFO {}-{} command failed: {} {}",
                channel, layer, response.code, response.message
            )))
        }
    }

    /// Get server paths
    pub async fn info_paths(&self) -> Result<String, AmcpError> {
        let response = self.send_command("INFO PATHS").await?;

        if response.is_success() {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(AmcpError::Protocol(format!(
                "INFO PATHS command failed: {} {}",
                response.code, response.message
            )))
        }
    }

    /// Get server configuration
    pub async fn info_config(&self) -> Result<String, AmcpError> {
        let response = self.send_command("INFO CONFIG").await?;

        if response.is_success() {
            Ok(response.data.unwrap_or_default())
        } else {
            Err(AmcpError::Protocol(format!(
                "INFO CONFIG command failed: {} {}",
                response.code, response.message
            )))
        }
    }

    /// Ping the server to check connectivity
    pub async fn ping(&self) -> Result<bool, AmcpError> {
        // Send an innocuous command to test connection
        let response = self.send_command("VERSION").await?;
        Ok(response.is_success())
    }

    /// Restart the server (requires appropriate permissions)
    pub async fn restart(&self) -> Result<(), AmcpError> {
        let response = self.send_command("RESTART").await?;

        if response.is_success() {
            Ok(())
        } else {
            Err(AmcpError::Protocol(format!(
                "RESTART command failed: {} {}",
                response.code, response.message
            )))
        }
    }
}

/// Parse VERSION response to extract version string
pub fn parse_version_response(data: &str) -> Option<String> {
    // CasparCG returns version in various formats
    // Try to extract the version number
    let trimmed = data.trim();

    // If it contains newlines, take first line
    let first_line = trimmed.lines().next()?;

    // Remove any trailing comments or extra info
    Some(first_line.trim().to_string())
}

/// Parse INFO SYSTEM response (XML format)
pub fn parse_system_info(xml: &str) -> Option<SystemInfo> {
    // Basic parsing - the actual response is XML
    // For now just extract some key values
    let version = extract_xml_value(xml, "version");
    let channels = extract_xml_value(xml, "channels")
        .and_then(|s| s.parse().ok())
        .unwrap_or(0);

    Some(SystemInfo { version, channels })
}

/// Simple XML value extraction (not a full parser)
fn extract_xml_value(xml: &str, tag: &str) -> Option<String> {
    let open_tag = format!("<{}>", tag);
    let close_tag = format!("</{}>", tag);

    let start = xml.find(&open_tag)? + open_tag.len();
    let end = xml[start..].find(&close_tag)? + start;

    Some(xml[start..end].trim().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_version_response() {
        assert_eq!(
            parse_version_response("2.5.0"),
            Some("2.5.0".to_string())
        );
        assert_eq!(
            parse_version_response("2.5.0\nmore data"),
            Some("2.5.0".to_string())
        );
    }

    #[test]
    fn test_extract_xml_value() {
        let xml = "<system><version>2.5.0</version><channels>2</channels></system>";
        assert_eq!(extract_xml_value(xml, "version"), Some("2.5.0".to_string()));
        assert_eq!(extract_xml_value(xml, "channels"), Some("2".to_string()));
    }
}
