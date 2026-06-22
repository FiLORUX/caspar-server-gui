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

    /// Start a channel test by putting a solid colour fill on the channel.
    ///
    /// This deliberately does NOT use the HTML identifier: CasparCG's CEF/HTML
    /// producer crashes the server outright on some setups (the process vanishes
    /// right after the PLAY is acknowledged), which made "Test" kill the server
    /// and show nothing. The colour producer is a plain, CEF-free path that
    /// reliably reaches the SDI output, so Test answers the only question it needs
    /// to — "is this channel actually producing output?" — without risking the
    /// server. The HTML identifier still renders in the in-app Preview tab, which
    /// uses the system webview rather than CasparCG's CEF.
    ///
    /// `test_server_url` is unused now but kept in the signature for callers.
    pub async fn start_channel_test(
        &self,
        channel: u32,
        _test_server_url: &str,
    ) -> Result<(), AmcpError> {
        // Magenta — unmistakably a test signal, and easy to spot on a scope.
        let cmd = format!("PLAY {}-{} #FFFF00FF", channel, TEST_FILL_LAYER);
        let response = self.send_command(&cmd).await?;
        if !response.is_success() {
            return Err(AmcpError::Protocol(format!(
                "Failed to load test pattern: {} {}",
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
