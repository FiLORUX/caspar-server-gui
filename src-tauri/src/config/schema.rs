// Configuration schema definitions
// Defines the data structures for CasparCG configuration

use serde::{Deserialize, Serialize};

/// Video mode supported by CasparCG
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum VideoMode {
    #[serde(rename = "PAL")]
    Pal,
    #[serde(rename = "NTSC")]
    Ntsc,
    #[serde(rename = "576p2500")]
    P576_2500,
    #[serde(rename = "720p2398")]
    P720_2398,
    #[serde(rename = "720p2400")]
    P720_2400,
    #[serde(rename = "720p2500")]
    P720_2500,
    #[serde(rename = "720p5000")]
    P720_5000,
    #[serde(rename = "720p2997")]
    P720_2997,
    #[serde(rename = "720p5994")]
    P720_5994,
    #[serde(rename = "720p3000")]
    P720_3000,
    #[serde(rename = "720p6000")]
    P720_6000,
    #[serde(rename = "1080p2398")]
    P1080_2398,
    #[serde(rename = "1080p2400")]
    P1080_2400,
    #[serde(rename = "1080i5000")]
    I1080_5000,
    #[serde(rename = "1080i5994")]
    I1080_5994,
    #[serde(rename = "1080i6000")]
    I1080_6000,
    #[serde(rename = "1080p2500")]
    P1080_2500,
    #[serde(rename = "1080p2997")]
    P1080_2997,
    #[serde(rename = "1080p3000")]
    P1080_3000,
    #[serde(rename = "1080p5000")]
    P1080_5000,
    #[serde(rename = "1080p5994")]
    P1080_5994,
    #[serde(rename = "1080p6000")]
    P1080_6000,
    #[serde(rename = "1556p2398")]
    P1556_2398,
    #[serde(rename = "1556p2400")]
    P1556_2400,
    #[serde(rename = "1556p2500")]
    P1556_2500,
    #[serde(rename = "2160p2398")]
    P2160_2398,
    #[serde(rename = "2160p2400")]
    P2160_2400,
    #[serde(rename = "2160p2500")]
    P2160_2500,
    #[serde(rename = "2160p2997")]
    P2160_2997,
    #[serde(rename = "2160p3000")]
    P2160_3000,
    #[serde(rename = "2160p5000")]
    P2160_5000,
    #[serde(rename = "2160p5994")]
    P2160_5994,
    #[serde(rename = "2160p6000")]
    P2160_6000,
}

impl Default for VideoMode {
    fn default() -> Self {
        VideoMode::I1080_5000
    }
}

impl VideoMode {
    /// Get all available video modes
    pub fn all() -> Vec<VideoMode> {
        vec![
            VideoMode::Pal,
            VideoMode::Ntsc,
            VideoMode::P576_2500,
            VideoMode::P720_2398,
            VideoMode::P720_2400,
            VideoMode::P720_2500,
            VideoMode::P720_5000,
            VideoMode::P720_2997,
            VideoMode::P720_5994,
            VideoMode::P720_3000,
            VideoMode::P720_6000,
            VideoMode::P1080_2398,
            VideoMode::P1080_2400,
            VideoMode::I1080_5000,
            VideoMode::I1080_5994,
            VideoMode::I1080_6000,
            VideoMode::P1080_2500,
            VideoMode::P1080_2997,
            VideoMode::P1080_3000,
            VideoMode::P1080_5000,
            VideoMode::P1080_5994,
            VideoMode::P1080_6000,
            VideoMode::P1556_2398,
            VideoMode::P1556_2400,
            VideoMode::P1556_2500,
            VideoMode::P2160_2398,
            VideoMode::P2160_2400,
            VideoMode::P2160_2500,
            VideoMode::P2160_2997,
            VideoMode::P2160_3000,
            VideoMode::P2160_5000,
            VideoMode::P2160_5994,
            VideoMode::P2160_6000,
        ]
    }

    /// Get display name for the video mode
    pub fn display_name(&self) -> &'static str {
        match self {
            VideoMode::Pal => "PAL (576i50)",
            VideoMode::Ntsc => "NTSC (480i59.94)",
            VideoMode::P576_2500 => "576p25",
            VideoMode::P720_2398 => "720p23.98",
            VideoMode::P720_2400 => "720p24",
            VideoMode::P720_2500 => "720p25",
            VideoMode::P720_5000 => "720p50",
            VideoMode::P720_2997 => "720p29.97",
            VideoMode::P720_5994 => "720p59.94",
            VideoMode::P720_3000 => "720p30",
            VideoMode::P720_6000 => "720p60",
            VideoMode::P1080_2398 => "1080p23.98",
            VideoMode::P1080_2400 => "1080p24",
            VideoMode::I1080_5000 => "1080i50",
            VideoMode::I1080_5994 => "1080i59.94",
            VideoMode::I1080_6000 => "1080i60",
            VideoMode::P1080_2500 => "1080p25",
            VideoMode::P1080_2997 => "1080p29.97",
            VideoMode::P1080_3000 => "1080p30",
            VideoMode::P1080_5000 => "1080p50",
            VideoMode::P1080_5994 => "1080p59.94",
            VideoMode::P1080_6000 => "1080p60",
            VideoMode::P1556_2398 => "1556p23.98 (2K)",
            VideoMode::P1556_2400 => "1556p24 (2K)",
            VideoMode::P1556_2500 => "1556p25 (2K)",
            VideoMode::P2160_2398 => "2160p23.98 (4K)",
            VideoMode::P2160_2400 => "2160p24 (4K)",
            VideoMode::P2160_2500 => "2160p25 (4K)",
            VideoMode::P2160_2997 => "2160p29.97 (4K)",
            VideoMode::P2160_3000 => "2160p30 (4K)",
            VideoMode::P2160_5000 => "2160p50 (4K)",
            VideoMode::P2160_5994 => "2160p59.94 (4K)",
            VideoMode::P2160_6000 => "2160p60 (4K)",
        }
    }
}

/// DeckLink latency mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "lowercase")]
pub enum DeckLinkLatency {
    #[default]
    Normal,
    Low,
    Default,
}

/// DeckLink keyer mode
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Default)]
#[serde(rename_all = "snake_case")]
pub enum DeckLinkKeyer {
    #[default]
    External,
    ExternalSeparateDevice,
    Internal,
    Default,
}

/// DeckLink consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeckLinkConsumer {
    pub device: u32,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_device: Option<u32>,
    #[serde(default)]
    pub embedded_audio: bool,
    #[serde(default)]
    pub latency: DeckLinkLatency,
    #[serde(default)]
    pub keyer: DeckLinkKeyer,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub key_only: Option<bool>,
}

impl Default for DeckLinkConsumer {
    fn default() -> Self {
        Self {
            device: 1,
            key_device: None,
            embedded_audio: true,
            latency: DeckLinkLatency::default(),
            keyer: DeckLinkKeyer::default(),
            key_only: None,
        }
    }
}

/// NDI consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NdiConsumer {
    pub name: String,
    #[serde(default)]
    pub allow_fields: bool,
}

impl Default for NdiConsumer {
    fn default() -> Self {
        Self {
            name: "CasparCG".to_string(),
            allow_fields: true,
        }
    }
}

/// Screen consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScreenConsumer {
    #[serde(default = "default_screen_device")]
    pub device: u32,
    #[serde(default = "default_true")]
    pub windowed: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub width: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub height: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub x: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub y: Option<i32>,
    #[serde(default)]
    pub borderless: bool,
    #[serde(default)]
    pub always_on_top: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
}

fn default_screen_device() -> u32 {
    1
}

fn default_true() -> bool {
    true
}

impl Default for ScreenConsumer {
    fn default() -> Self {
        Self {
            device: 1,
            windowed: true,
            width: None,
            height: None,
            x: None,
            y: None,
            borderless: false,
            always_on_top: false,
            name: None,
        }
    }
}

/// System audio consumer configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct SystemAudioConsumer {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub channel_layout: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub latency: Option<u32>,
}

/// Consumer type enumeration
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
pub enum Consumer {
    DeckLink(DeckLinkConsumer),
    Ndi(NdiConsumer),
    Screen(ScreenConsumer),
    #[serde(rename = "system-audio")]
    SystemAudio(SystemAudioConsumer),
}

/// Channel configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Channel {
    #[serde(default)]
    pub video_mode: VideoMode,
    #[serde(default)]
    pub consumers: Vec<Consumer>,
}

impl Default for Channel {
    fn default() -> Self {
        Self {
            video_mode: VideoMode::default(),
            consumers: vec![],
        }
    }
}

/// Path configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Paths {
    #[serde(default)]
    pub media: String,
    #[serde(default)]
    pub template: String,
    #[serde(default)]
    pub log: String,
    #[serde(default)]
    pub data: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub font: Option<String>,
}

/// TCP controller configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TcpController {
    #[serde(default = "default_tcp_port")]
    pub port: u16,
    #[serde(default = "default_protocol")]
    pub protocol: String,
}

fn default_tcp_port() -> u16 {
    5250
}

fn default_protocol() -> String {
    "AMCP".to_string()
}

impl Default for TcpController {
    fn default() -> Self {
        Self {
            port: 5250,
            protocol: "AMCP".to_string(),
        }
    }
}

/// Controllers configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Controllers {
    #[serde(default)]
    pub tcp: TcpController,
}

/// AMCP media server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MediaServer {
    #[serde(default = "default_localhost")]
    pub host: String,
    #[serde(default = "default_media_server_port")]
    pub port: u16,
}

fn default_localhost() -> String {
    "localhost".to_string()
}

fn default_media_server_port() -> u16 {
    8000
}

impl Default for MediaServer {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 8000,
        }
    }
}

/// AMCP configuration
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct AmcpConfig {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub media_server: Option<MediaServer>,
}

/// Main CasparCG configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CasparConfig {
    #[serde(default)]
    pub paths: Paths,
    #[serde(default)]
    pub channels: Vec<Channel>,
    #[serde(default)]
    pub controllers: Controllers,
    #[serde(default)]
    pub amcp: AmcpConfig,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_level: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub log_categories: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub force_deinterlace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auto_deinterlace: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub blend_modes: Option<bool>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub mixer_latency: Option<u32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub accelerator: Option<String>,
}

impl Default for CasparConfig {
    fn default() -> Self {
        Self {
            paths: Paths::default(),
            channels: vec![Channel::default()],
            controllers: Controllers::default(),
            amcp: AmcpConfig::default(),
            log_level: None,
            log_categories: None,
            force_deinterlace: None,
            auto_deinterlace: None,
            blend_modes: None,
            mixer_latency: None,
            accelerator: None,
        }
    }
}
