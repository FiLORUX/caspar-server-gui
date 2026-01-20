// CasparCG XML configuration parser and generator
// Handles reading and writing the standard casparcg.config XML format

use quick_xml::events::{BytesDecl, BytesEnd, BytesStart, BytesText, Event};
use quick_xml::{Reader, Writer};
use std::io::Cursor;

use super::schema::*;
use super::GlobalConfigError;

/// Parse a CasparCG XML config file
pub fn parse_caspar_xml(xml: &str) -> Result<CasparConfig, CasparXmlError> {
    let mut reader = Reader::from_str(xml);
    reader.config_mut().trim_text(true);

    let mut config = CasparConfig::default();
    let mut buf = Vec::new();

    // Track current parsing context
    let mut in_paths = false;
    let mut in_channels = false;
    let mut in_channel = false;
    let mut in_consumers = false;
    let mut in_consumer: Option<String> = None;
    let mut in_controllers = false;
    let mut in_tcp = false;
    let mut in_amcp = false;
    let mut in_media_server = false;
    let mut current_element = String::new();
    let mut current_channel: Option<Channel> = None;
    let mut current_consumer: Option<ConsumerBuilder> = None;

    loop {
        match reader.read_event_into(&mut buf) {
            Ok(Event::Start(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();
                current_element = name.clone();

                match name.as_str() {
                    "paths" => in_paths = true,
                    "channels" => in_channels = true,
                    "channel" => {
                        in_channel = true;
                        current_channel = Some(Channel::default());
                    }
                    "consumers" => in_consumers = true,
                    "decklink" => {
                        if in_consumers {
                            in_consumer = Some("decklink".to_string());
                            current_consumer = Some(ConsumerBuilder::DeckLink(DeckLinkConsumer::default()));
                        }
                    }
                    "ndi" => {
                        if in_consumers {
                            in_consumer = Some("ndi".to_string());
                            current_consumer = Some(ConsumerBuilder::Ndi(NdiConsumer::default()));
                        }
                    }
                    "screen" => {
                        if in_consumers {
                            in_consumer = Some("screen".to_string());
                            current_consumer = Some(ConsumerBuilder::Screen(ScreenConsumer::default()));
                        }
                    }
                    "system-audio" => {
                        if in_consumers {
                            in_consumer = Some("system-audio".to_string());
                            current_consumer = Some(ConsumerBuilder::SystemAudio(SystemAudioConsumer::default()));
                        }
                    }
                    "controllers" => in_controllers = true,
                    "tcp" => {
                        if in_controllers {
                            in_tcp = true;
                        }
                    }
                    "amcp" => in_amcp = true,
                    "media-server" => {
                        if in_amcp {
                            in_media_server = true;
                            config.amcp.media_server = Some(MediaServer::default());
                        }
                    }
                    _ => {}
                }
            }
            Ok(Event::End(ref e)) => {
                let name = String::from_utf8_lossy(e.name().as_ref()).to_string();

                match name.as_str() {
                    "paths" => in_paths = false,
                    "channels" => in_channels = false,
                    "channel" => {
                        if let Some(ch) = current_channel.take() {
                            config.channels.push(ch);
                        }
                        in_channel = false;
                    }
                    "consumers" => in_consumers = false,
                    "decklink" | "ndi" | "screen" | "system-audio" => {
                        if in_consumer.is_some() {
                            if let Some(builder) = current_consumer.take() {
                                if let Some(ref mut ch) = current_channel {
                                    ch.consumers.push(builder.into());
                                }
                            }
                            in_consumer = None;
                        }
                    }
                    "controllers" => in_controllers = false,
                    "tcp" => in_tcp = false,
                    "amcp" => in_amcp = false,
                    "media-server" => in_media_server = false,
                    _ => {}
                }
            }
            Ok(Event::Text(ref e)) => {
                let text = e.unescape().map(|s| s.to_string()).unwrap_or_default();

                if in_paths {
                    match current_element.as_str() {
                        "media-path" => config.paths.media = text,
                        "template-path" => config.paths.template = text,
                        "log-path" => config.paths.log = text,
                        "data-path" => config.paths.data = text,
                        "font-path" => config.paths.font = Some(text),
                        _ => {}
                    }
                } else if in_channel && !in_consumers {
                    if current_element == "video-mode" {
                        if let Some(ref mut ch) = current_channel {
                            ch.video_mode = parse_video_mode(&text);
                        }
                    }
                } else if let Some(ref consumer_type) = in_consumer {
                    if let Some(ref mut builder) = current_consumer {
                        apply_consumer_property(builder, consumer_type, &current_element, &text);
                    }
                } else if in_tcp {
                    match current_element.as_str() {
                        "port" => {
                            if let Ok(port) = text.parse() {
                                config.controllers.tcp.port = port;
                            }
                        }
                        "protocol" => config.controllers.tcp.protocol = text,
                        _ => {}
                    }
                } else if in_media_server {
                    if let Some(ref mut ms) = config.amcp.media_server {
                        match current_element.as_str() {
                            "host" => ms.host = text,
                            "port" => {
                                if let Ok(port) = text.parse() {
                                    ms.port = port;
                                }
                            }
                            _ => {}
                        }
                    }
                } else {
                    // Root level elements
                    match current_element.as_str() {
                        "log-level" => config.log_level = Some(text),
                        "log-categories" => config.log_categories = Some(text),
                        "force-deinterlace" => config.force_deinterlace = Some(text == "true"),
                        "auto-deinterlace" => config.auto_deinterlace = Some(text == "true"),
                        "blend-modes" => config.blend_modes = Some(text == "true"),
                        "mixer-latency" => config.mixer_latency = text.parse().ok(),
                        "accelerator" => config.accelerator = Some(text),
                        _ => {}
                    }
                }
            }
            Ok(Event::Eof) => break,
            Err(e) => return Err(CasparXmlError::Parse(format!("Error at position {}: {:?}", reader.error_position(), e))),
            _ => {}
        }
        buf.clear();
    }

    // If no channels were parsed, ensure at least one default exists
    if config.channels.is_empty() {
        config.channels.push(Channel::default());
    }

    Ok(config)
}

/// Generate CasparCG XML from config
pub fn generate_caspar_xml(config: &CasparConfig) -> Result<String, CasparXmlError> {
    let mut writer = Writer::new_with_indent(Cursor::new(Vec::new()), b' ', 2);

    // XML declaration
    writer.write_event(Event::Decl(BytesDecl::new("1.0", Some("utf-8"), None)))?;

    // Root element
    writer.write_event(Event::Start(BytesStart::new("configuration")))?;

    // Paths section
    writer.write_event(Event::Start(BytesStart::new("paths")))?;
    write_element(&mut writer, "media-path", &config.paths.media)?;
    write_element(&mut writer, "template-path", &config.paths.template)?;
    write_element(&mut writer, "log-path", &config.paths.log)?;
    write_element(&mut writer, "data-path", &config.paths.data)?;
    if let Some(ref font) = config.paths.font {
        write_element(&mut writer, "font-path", font)?;
    }
    writer.write_event(Event::End(BytesEnd::new("paths")))?;

    // Log settings
    if let Some(ref level) = config.log_level {
        write_element(&mut writer, "log-level", level)?;
    }
    if let Some(ref categories) = config.log_categories {
        write_element(&mut writer, "log-categories", categories)?;
    }

    // Mixer settings
    if let Some(force) = config.force_deinterlace {
        write_element(&mut writer, "force-deinterlace", if force { "true" } else { "false" })?;
    }
    if let Some(auto) = config.auto_deinterlace {
        write_element(&mut writer, "auto-deinterlace", if auto { "true" } else { "false" })?;
    }
    if let Some(blend) = config.blend_modes {
        write_element(&mut writer, "blend-modes", if blend { "true" } else { "false" })?;
    }
    if let Some(latency) = config.mixer_latency {
        write_element(&mut writer, "mixer-latency", &latency.to_string())?;
    }
    if let Some(ref accel) = config.accelerator {
        write_element(&mut writer, "accelerator", accel)?;
    }

    // Channels section
    writer.write_event(Event::Start(BytesStart::new("channels")))?;
    for channel in &config.channels {
        write_channel(&mut writer, channel)?;
    }
    writer.write_event(Event::End(BytesEnd::new("channels")))?;

    // Controllers section
    writer.write_event(Event::Start(BytesStart::new("controllers")))?;
    writer.write_event(Event::Start(BytesStart::new("tcp")))?;
    write_element(&mut writer, "port", &config.controllers.tcp.port.to_string())?;
    write_element(&mut writer, "protocol", &config.controllers.tcp.protocol)?;
    writer.write_event(Event::End(BytesEnd::new("tcp")))?;
    writer.write_event(Event::End(BytesEnd::new("controllers")))?;

    // AMCP section
    if config.amcp.media_server.is_some() {
        writer.write_event(Event::Start(BytesStart::new("amcp")))?;
        if let Some(ref ms) = config.amcp.media_server {
            writer.write_event(Event::Start(BytesStart::new("media-server")))?;
            write_element(&mut writer, "host", &ms.host)?;
            write_element(&mut writer, "port", &ms.port.to_string())?;
            writer.write_event(Event::End(BytesEnd::new("media-server")))?;
        }
        writer.write_event(Event::End(BytesEnd::new("amcp")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("configuration")))?;

    let result = writer.into_inner().into_inner();
    Ok(String::from_utf8(result)?)
}

fn write_element<W: std::io::Write>(
    writer: &mut Writer<W>,
    name: &str,
    value: &str,
) -> Result<(), CasparXmlError> {
    writer.write_event(Event::Start(BytesStart::new(name)))?;
    writer.write_event(Event::Text(BytesText::new(value)))?;
    writer.write_event(Event::End(BytesEnd::new(name)))?;
    Ok(())
}

fn write_channel<W: std::io::Write>(
    writer: &mut Writer<W>,
    channel: &Channel,
) -> Result<(), CasparXmlError> {
    writer.write_event(Event::Start(BytesStart::new("channel")))?;

    // Video mode - serialize enum properly
    let video_mode_str = serde_json::to_string(&channel.video_mode)
        .map(|s| s.trim_matches('"').to_string())
        .unwrap_or_else(|_| "1080i5000".to_string());
    write_element(writer, "video-mode", &video_mode_str)?;

    // Consumers
    if !channel.consumers.is_empty() {
        writer.write_event(Event::Start(BytesStart::new("consumers")))?;
        for consumer in &channel.consumers {
            write_consumer(writer, consumer)?;
        }
        writer.write_event(Event::End(BytesEnd::new("consumers")))?;
    }

    writer.write_event(Event::End(BytesEnd::new("channel")))?;
    Ok(())
}

fn write_consumer<W: std::io::Write>(
    writer: &mut Writer<W>,
    consumer: &Consumer,
) -> Result<(), CasparXmlError> {
    match consumer {
        Consumer::DeckLink(dl) => {
            writer.write_event(Event::Start(BytesStart::new("decklink")))?;
            write_element(writer, "device", &dl.device.to_string())?;
            if let Some(key) = dl.key_device {
                write_element(writer, "key-device", &key.to_string())?;
            }
            write_element(writer, "embedded-audio", if dl.embedded_audio { "true" } else { "false" })?;
            let latency_str = match dl.latency {
                DeckLinkLatency::Normal => "normal",
                DeckLinkLatency::Low => "low",
                DeckLinkLatency::Default => "default",
            };
            write_element(writer, "latency", latency_str)?;
            let keyer_str = match dl.keyer {
                DeckLinkKeyer::External => "external",
                DeckLinkKeyer::ExternalSeparateDevice => "external_separate_device",
                DeckLinkKeyer::Internal => "internal",
                DeckLinkKeyer::Default => "default",
            };
            write_element(writer, "keyer", keyer_str)?;
            if let Some(key_only) = dl.key_only {
                write_element(writer, "key-only", if key_only { "true" } else { "false" })?;
            }
            writer.write_event(Event::End(BytesEnd::new("decklink")))?;
        }
        Consumer::Ndi(ndi) => {
            writer.write_event(Event::Start(BytesStart::new("ndi")))?;
            write_element(writer, "name", &ndi.name)?;
            write_element(writer, "allow-fields", if ndi.allow_fields { "true" } else { "false" })?;
            writer.write_event(Event::End(BytesEnd::new("ndi")))?;
        }
        Consumer::Screen(scr) => {
            writer.write_event(Event::Start(BytesStart::new("screen")))?;
            write_element(writer, "device", &scr.device.to_string())?;
            write_element(writer, "windowed", if scr.windowed { "true" } else { "false" })?;
            if let Some(w) = scr.width {
                write_element(writer, "width", &w.to_string())?;
            }
            if let Some(h) = scr.height {
                write_element(writer, "height", &h.to_string())?;
            }
            if let Some(x) = scr.x {
                write_element(writer, "x", &x.to_string())?;
            }
            if let Some(y) = scr.y {
                write_element(writer, "y", &y.to_string())?;
            }
            if scr.borderless {
                write_element(writer, "borderless", "true")?;
            }
            if scr.always_on_top {
                write_element(writer, "always-on-top", "true")?;
            }
            if let Some(ref name) = scr.name {
                write_element(writer, "name", name)?;
            }
            writer.write_event(Event::End(BytesEnd::new("screen")))?;
        }
        Consumer::SystemAudio(sa) => {
            writer.write_event(Event::Start(BytesStart::new("system-audio")))?;
            if let Some(ref layout) = sa.channel_layout {
                write_element(writer, "channel-layout", layout)?;
            }
            if let Some(latency) = sa.latency {
                write_element(writer, "latency", &latency.to_string())?;
            }
            writer.write_event(Event::End(BytesEnd::new("system-audio")))?;
        }
    }
    Ok(())
}

/// Parse video mode string to enum
fn parse_video_mode(s: &str) -> VideoMode {
    match s.to_uppercase().as_str() {
        "PAL" => VideoMode::Pal,
        "NTSC" => VideoMode::Ntsc,
        "576P2500" => VideoMode::P576_2500,
        "720P2398" => VideoMode::P720_2398,
        "720P2400" => VideoMode::P720_2400,
        "720P2500" => VideoMode::P720_2500,
        "720P5000" => VideoMode::P720_5000,
        "720P2997" => VideoMode::P720_2997,
        "720P5994" => VideoMode::P720_5994,
        "720P3000" => VideoMode::P720_3000,
        "720P6000" => VideoMode::P720_6000,
        "1080P2398" => VideoMode::P1080_2398,
        "1080P2400" => VideoMode::P1080_2400,
        "1080I5000" => VideoMode::I1080_5000,
        "1080I5994" => VideoMode::I1080_5994,
        "1080I6000" => VideoMode::I1080_6000,
        "1080P2500" => VideoMode::P1080_2500,
        "1080P2997" => VideoMode::P1080_2997,
        "1080P3000" => VideoMode::P1080_3000,
        "1080P5000" => VideoMode::P1080_5000,
        "1080P5994" => VideoMode::P1080_5994,
        "1080P6000" => VideoMode::P1080_6000,
        "1556P2398" => VideoMode::P1556_2398,
        "1556P2400" => VideoMode::P1556_2400,
        "1556P2500" => VideoMode::P1556_2500,
        "2160P2398" => VideoMode::P2160_2398,
        "2160P2400" => VideoMode::P2160_2400,
        "2160P2500" => VideoMode::P2160_2500,
        "2160P2997" => VideoMode::P2160_2997,
        "2160P3000" => VideoMode::P2160_3000,
        "2160P5000" => VideoMode::P2160_5000,
        "2160P5994" => VideoMode::P2160_5994,
        "2160P6000" => VideoMode::P2160_6000,
        _ => VideoMode::I1080_5000, // Default fallback
    }
}

/// Builder enum for constructing consumers during parsing
enum ConsumerBuilder {
    DeckLink(DeckLinkConsumer),
    Ndi(NdiConsumer),
    Screen(ScreenConsumer),
    SystemAudio(SystemAudioConsumer),
}

impl From<ConsumerBuilder> for Consumer {
    fn from(builder: ConsumerBuilder) -> Self {
        match builder {
            ConsumerBuilder::DeckLink(c) => Consumer::DeckLink(c),
            ConsumerBuilder::Ndi(c) => Consumer::Ndi(c),
            ConsumerBuilder::Screen(c) => Consumer::Screen(c),
            ConsumerBuilder::SystemAudio(c) => Consumer::SystemAudio(c),
        }
    }
}

fn apply_consumer_property(builder: &mut ConsumerBuilder, consumer_type: &str, element: &str, value: &str) {
    match (consumer_type, builder) {
        ("decklink", ConsumerBuilder::DeckLink(ref mut dl)) => {
            match element {
                "device" => dl.device = value.parse().unwrap_or(1),
                "key-device" => dl.key_device = value.parse().ok(),
                "embedded-audio" => dl.embedded_audio = value == "true",
                "latency" => dl.latency = match value {
                    "low" => DeckLinkLatency::Low,
                    "default" => DeckLinkLatency::Default,
                    _ => DeckLinkLatency::Normal,
                },
                "keyer" => dl.keyer = match value {
                    "external_separate_device" => DeckLinkKeyer::ExternalSeparateDevice,
                    "internal" => DeckLinkKeyer::Internal,
                    "default" => DeckLinkKeyer::Default,
                    _ => DeckLinkKeyer::External,
                },
                "key-only" => dl.key_only = Some(value == "true"),
                _ => {}
            }
        }
        ("ndi", ConsumerBuilder::Ndi(ref mut ndi)) => {
            match element {
                "name" => ndi.name = value.to_string(),
                "allow-fields" => ndi.allow_fields = value == "true",
                _ => {}
            }
        }
        ("screen", ConsumerBuilder::Screen(ref mut scr)) => {
            match element {
                "device" => scr.device = value.parse().unwrap_or(1),
                "windowed" => scr.windowed = value == "true",
                "width" => scr.width = value.parse().ok(),
                "height" => scr.height = value.parse().ok(),
                "x" => scr.x = value.parse().ok(),
                "y" => scr.y = value.parse().ok(),
                "borderless" => scr.borderless = value == "true",
                "always-on-top" => scr.always_on_top = value == "true",
                "name" => scr.name = Some(value.to_string()),
                _ => {}
            }
        }
        ("system-audio", ConsumerBuilder::SystemAudio(ref mut sa)) => {
            match element {
                "channel-layout" => sa.channel_layout = Some(value.to_string()),
                "latency" => sa.latency = value.parse().ok(),
                _ => {}
            }
        }
        _ => {}
    }
}

/// Errors specific to CasparCG XML parsing/generation
#[derive(Debug, thiserror::Error)]
pub enum CasparXmlError {
    #[error("XML parse error: {0}")]
    Parse(String),
    #[error("XML write error: {0}")]
    Write(#[from] quick_xml::Error),
    #[error("UTF-8 encoding error: {0}")]
    Utf8(#[from] std::string::FromUtf8Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
}

impl From<CasparXmlError> for GlobalConfigError {
    fn from(err: CasparXmlError) -> Self {
        GlobalConfigError::Io(std::io::Error::new(std::io::ErrorKind::InvalidData, err.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let xml = r#"<?xml version="1.0" encoding="utf-8"?>
<configuration>
    <paths>
        <media-path>/data/media/</media-path>
        <template-path>/data/templates/</template-path>
        <log-path>/var/log/</log-path>
        <data-path>/data/</data-path>
    </paths>
    <channels>
        <channel>
            <video-mode>1080i5000</video-mode>
            <consumers>
                <screen>
                    <device>1</device>
                    <windowed>true</windowed>
                </screen>
            </consumers>
        </channel>
    </channels>
    <controllers>
        <tcp>
            <port>5250</port>
            <protocol>AMCP</protocol>
        </tcp>
    </controllers>
</configuration>"#;

        let config = parse_caspar_xml(xml).expect("Failed to parse XML");
        assert_eq!(config.paths.media, "/data/media/");
        assert_eq!(config.channels.len(), 1);
        assert_eq!(config.controllers.tcp.port, 5250);
    }

    #[test]
    fn test_roundtrip() {
        let original = CasparConfig::default();
        let xml = generate_caspar_xml(&original).expect("Failed to generate XML");
        let parsed = parse_caspar_xml(&xml).expect("Failed to parse generated XML");

        assert_eq!(original.channels.len(), parsed.channels.len());
        assert_eq!(original.controllers.tcp.port, parsed.controllers.tcp.port);
    }
}
