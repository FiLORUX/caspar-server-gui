// TypeScript type definitions matching Rust backend structures

// ============================================================================
// Video Modes
// ============================================================================

export type VideoMode =
  | 'PAL'
  | 'NTSC'
  | '576p2500'
  | '720p2398'
  | '720p2400'
  | '720p2500'
  | '720p5000'
  | '720p2997'
  | '720p5994'
  | '720p3000'
  | '720p6000'
  | '1080p2398'
  | '1080p2400'
  | '1080i5000'
  | '1080i5994'
  | '1080i6000'
  | '1080p2500'
  | '1080p2997'
  | '1080p3000'
  | '1080p5000'
  | '1080p5994'
  | '1080p6000'
  | '1556p2398'
  | '1556p2400'
  | '1556p2500'
  | '2160p2398'
  | '2160p2400'
  | '2160p2500'
  | '2160p2997'
  | '2160p3000'
  | '2160p5000'
  | '2160p5994'
  | '2160p6000';

export const VIDEO_MODES: { value: VideoMode; label: string }[] = [
  { value: 'PAL', label: 'PAL (576i50)' },
  { value: 'NTSC', label: 'NTSC (480i59.94)' },
  { value: '576p2500', label: '576p25' },
  { value: '720p2398', label: '720p23.98' },
  { value: '720p2400', label: '720p24' },
  { value: '720p2500', label: '720p25' },
  { value: '720p5000', label: '720p50' },
  { value: '720p2997', label: '720p29.97' },
  { value: '720p5994', label: '720p59.94' },
  { value: '720p3000', label: '720p30' },
  { value: '720p6000', label: '720p60' },
  { value: '1080p2398', label: '1080p23.98' },
  { value: '1080p2400', label: '1080p24' },
  { value: '1080i5000', label: '1080i50' },
  { value: '1080i5994', label: '1080i59.94' },
  { value: '1080i6000', label: '1080i60' },
  { value: '1080p2500', label: '1080p25' },
  { value: '1080p2997', label: '1080p29.97' },
  { value: '1080p3000', label: '1080p30' },
  { value: '1080p5000', label: '1080p50' },
  { value: '1080p5994', label: '1080p59.94' },
  { value: '1080p6000', label: '1080p60' },
  { value: '1556p2398', label: '1556p23.98 (2K)' },
  { value: '1556p2400', label: '1556p24 (2K)' },
  { value: '1556p2500', label: '1556p25 (2K)' },
  { value: '2160p2398', label: '2160p23.98 (4K)' },
  { value: '2160p2400', label: '2160p24 (4K)' },
  { value: '2160p2500', label: '2160p25 (4K)' },
  { value: '2160p2997', label: '2160p29.97 (4K)' },
  { value: '2160p3000', label: '2160p30 (4K)' },
  { value: '2160p5000', label: '2160p50 (4K)' },
  { value: '2160p5994', label: '2160p59.94 (4K)' },
  { value: '2160p6000', label: '2160p60 (4K)' },
];

// ============================================================================
// Consumer Types
// ============================================================================

export type DeckLinkLatency = 'normal' | 'low' | 'default';
export type DeckLinkKeyer = 'external' | 'external_separate_device' | 'internal' | 'default';

export interface DeckLinkConsumer {
  type: 'decklink';
  device: number;
  key_device?: number;
  embedded_audio: boolean;
  latency: DeckLinkLatency;
  keyer: DeckLinkKeyer;
  key_only?: boolean;
}

export interface NdiConsumer {
  type: 'ndi';
  name: string;
  allow_fields: boolean;
}

export interface ScreenConsumer {
  type: 'screen';
  device: number;
  windowed: boolean;
  width?: number;
  height?: number;
  x?: number;
  y?: number;
  borderless: boolean;
  always_on_top: boolean;
  name?: string;
}

export interface SystemAudioConsumer {
  type: 'system-audio';
  channel_layout?: string;
  latency?: number;
}

export type Consumer = DeckLinkConsumer | NdiConsumer | ScreenConsumer | SystemAudioConsumer;

// ============================================================================
// CasparCG Configuration
// ============================================================================

export interface Paths {
  media: string;
  template: string;
  log: string;
  data: string;
  font?: string;
}

export interface Channel {
  video_mode: VideoMode;
  consumers: Consumer[];
}

export interface TcpController {
  port: number;
  protocol: string;
}

export interface Controllers {
  tcp: TcpController;
}

export interface MediaServer {
  host: string;
  port: number;
}

export interface AmcpConfig {
  media_server?: MediaServer;
}

export interface CasparConfig {
  paths: Paths;
  channels: Channel[];
  controllers: Controllers;
  amcp: AmcpConfig;
  log_level?: string;
  log_categories?: string;
  force_deinterlace?: boolean;
  auto_deinterlace?: boolean;
  blend_modes?: boolean;
  mixer_latency?: number;
  accelerator?: string;
}

// ============================================================================
// Global Configuration
// ============================================================================

export type ConnectorMode = 'input' | 'output';

export interface DeckLinkDeviceConfig {
  persistent_id: string;
  model_name: string;
  label?: string;
  duplex_mode?: string;
  connector_mapping?: Record<string, ConnectorMode>;
}

export interface DeckLinkConfig {
  devices: DeckLinkDeviceConfig[];
}

export interface GlobalConfig {
  version: string;
  name: string;
  created: string;
  modified: string;
  caspar: CasparConfig;
  decklink: DeckLinkConfig;
}

// ============================================================================
// GUI Settings
// ============================================================================

export interface GuiSettings {
  caspar_path?: string;
  last_profile?: string;
  last_host?: string;
  last_port?: number;
  last_server_was_running: boolean;
  window_width?: number;
  window_height?: number;
}

// ============================================================================
// DeckLink Devices
// ============================================================================

export interface DeckLinkDevice {
  index: number;
  persistent_id: string;
  model_name: string;
  display_name: string;
  device_label?: string;
  supports_duplex: boolean;
  duplex_mode?: string;
  sdi_inputs: number;
  sdi_outputs: number;
  supports_internal_keying: boolean;
  supports_external_keying: boolean;
  supports_capture: boolean;
  supports_playback: boolean;
  max_audio_channels: number;
}

// ============================================================================
// System Info
// ============================================================================

export interface SystemVersions {
  caspar_version?: string;
  decklink_version?: string;
  ndi_version?: string;
  scanner_version?: string;
}

// ============================================================================
// AMCP
// ============================================================================

export interface AmcpResponse {
  code: number;
  message: string;
  data?: string;
}

// ============================================================================
// UI State
// ============================================================================

export type TabId = 'paths' | 'channels' | 'decklink' | 'system';

export interface ConnectionStatus {
  connected: boolean;
  host?: string;
  port?: number;
  version?: string;
}

// ============================================================================
// Default Values
// ============================================================================

export const DEFAULT_PATHS: Paths = {
  media: '',
  template: '',
  log: '',
  data: '',
};

export const DEFAULT_CHANNEL: Channel = {
  video_mode: '1080i5000',
  consumers: [],
};

export const DEFAULT_CASPAR_CONFIG: CasparConfig = {
  paths: DEFAULT_PATHS,
  channels: [DEFAULT_CHANNEL],
  controllers: {
    tcp: {
      port: 5250,
      protocol: 'AMCP',
    },
  },
  amcp: {},
};

export const DEFAULT_GLOBAL_CONFIG: Omit<GlobalConfig, 'created' | 'modified'> = {
  version: '1.0',
  name: 'Default Profile',
  caspar: DEFAULT_CASPAR_CONFIG,
  decklink: {
    devices: [],
  },
};

export function createDefaultDeckLinkConsumer(): DeckLinkConsumer {
  return {
    type: 'decklink',
    device: 1,
    embedded_audio: true,
    latency: 'normal',
    keyer: 'external',
  };
}

export function createDefaultNdiConsumer(): NdiConsumer {
  return {
    type: 'ndi',
    name: 'CasparCG',
    allow_fields: true,
  };
}

export function createDefaultScreenConsumer(): ScreenConsumer {
  return {
    type: 'screen',
    device: 1,
    windowed: true,
    borderless: false,
    always_on_top: false,
  };
}

export function createDefaultSystemAudioConsumer(): SystemAudioConsumer {
  return {
    type: 'system-audio',
  };
}
