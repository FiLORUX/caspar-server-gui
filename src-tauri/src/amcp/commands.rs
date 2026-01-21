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

// Test pattern layers - high numbers to avoid conflicts with production content
const TEST_FILL_LAYER: u32 = 20;
const TEST_KEY_LAYER: u32 = 19;

impl AmcpClient {
    // ═══════════════════════════════════════════════════════════════
    // CHANNEL TEST COMMANDS
    // Load fill/key test patterns for visual verification
    // ═══════════════════════════════════════════════════════════════

    /// Start a channel test by loading fill/key identifier patterns
    ///
    /// Loads the test pattern on layers 19 (key) and 20 (fill), with keyer enabled.
    /// The test URL should point to the HTTP server serving the test patterns.
    pub async fn start_channel_test(
        &self,
        channel: u32,
        test_server_url: &str,
    ) -> Result<(), AmcpError> {
        // Build URLs for fill and key modes
        let fill_url = format!(
            "[HTML] {}/key-fill-identifier.html?mode=fill&id={}",
            test_server_url, channel
        );
        let key_url = format!(
            "[HTML] {}/key-fill-identifier.html?mode=key&id={}",
            test_server_url, channel
        );

        // Load fill on layer 20
        let fill_cmd = format!("PLAY {}-{} {}", channel, TEST_FILL_LAYER, fill_url);
        let response = self.send_command(&fill_cmd).await?;
        if !response.is_success() {
            return Err(AmcpError::Protocol(format!(
                "Failed to load fill pattern: {} {}",
                response.code, response.message
            )));
        }

        // Load key on layer 19
        let key_cmd = format!("PLAY {}-{} {}", channel, TEST_KEY_LAYER, key_url);
        let response = self.send_command(&key_cmd).await?;
        if !response.is_success() {
            return Err(AmcpError::Protocol(format!(
                "Failed to load key pattern: {} {}",
                response.code, response.message
            )));
        }

        // Enable keyer on layer 19 to use it as external key for layer 20
        let keyer_cmd = format!("MIXER {}-{} KEYER 1", channel, TEST_KEY_LAYER);
        let response = self.send_command(&keyer_cmd).await?;
        if !response.is_success() {
            return Err(AmcpError::Protocol(format!(
                "Failed to enable keyer: {} {}",
                response.code, response.message
            )));
        }

        Ok(())
    }

    /// Stop a channel test by clearing the test layers
    pub async fn stop_channel_test(&self, channel: u32) -> Result<(), AmcpError> {
        // Clear layer 20 (fill)
        let clear_fill = format!("CLEAR {}-{}", channel, TEST_FILL_LAYER);
        self.send_command(&clear_fill).await?;

        // Clear layer 19 (key)
        let clear_key = format!("CLEAR {}-{}", channel, TEST_KEY_LAYER);
        self.send_command(&clear_key).await?;

        Ok(())
    }

    /// Stop all channel tests (useful when disconnecting or cleaning up)
    pub async fn stop_all_channel_tests(&self, channel_count: u32) -> Result<(), AmcpError> {
        for channel in 1..=channel_count {
            // Ignore errors - channel might not have a test running
            let _ = self.stop_channel_test(channel).await;
        }
        Ok(())
    }

    // ═══════════════════════════════════════════════════════════════
    // INFO AND VERSION COMMANDS
    // ═══════════════════════════════════════════════════════════════

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
