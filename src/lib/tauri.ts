// Tauri command wrappers
// Type-safe wrappers for Rust backend commands

import { invoke } from '@tauri-apps/api/core';
import type {
  AmcpResponse,
  CasparConfig,
  DeckLinkDevice,
  DeckLinkStatus,
  GlobalConfig,
  GuiSettings,
  SystemVersions,
  TslDisplay,
} from './types';

// ============================================================================
// Configuration Commands
// ============================================================================

export async function loadCasparConfig(path: string): Promise<CasparConfig> {
  return invoke('load_caspar_config', { path });
}

export async function saveCasparConfig(
  path: string,
  config: CasparConfig
): Promise<void> {
  return invoke('save_caspar_config', { path, config });
}

export async function loadGlobalConfig(path: string): Promise<GlobalConfig> {
  return invoke('load_global_config', { path });
}

export async function saveGlobalConfig(
  path: string,
  config: GlobalConfig
): Promise<void> {
  return invoke('save_global_config', { path, config });
}

export async function exportToCasparXml(config: GlobalConfig): Promise<string> {
  return invoke('export_to_caspar_xml', { config });
}

export async function createDefaultConfig(name: string): Promise<GlobalConfig> {
  return invoke('create_default_config', { name });
}

export async function listProfiles(): Promise<string[]> {
  return invoke('list_profiles');
}

// ============================================================================
// DeckLink Commands
// ============================================================================

export async function listDeckLinkDevices(): Promise<DeckLinkDevice[]> {
  return invoke('list_decklink_devices');
}

export async function getDeckLinkInfo(
  persistentId: string
): Promise<DeckLinkDevice> {
  return invoke('get_decklink_info', { persistentId });
}

export async function setDeckLinkLabel(
  persistentId: string,
  label: string
): Promise<void> {
  return invoke('set_decklink_label', { persistentId, label });
}

export async function setDeckLinkDuplexMode(
  persistentId: string,
  mode: string
): Promise<void> {
  return invoke('set_decklink_duplex_mode', { persistentId, mode });
}

export async function getDeckLinkDriverVersion(): Promise<string | null> {
  return invoke('get_decklink_driver_version');
}

export async function getDeckLinkStatus(index: number): Promise<DeckLinkStatus> {
  return invoke('get_decklink_status', { index });
}

// Direct SDI output test — drives the card's SDI output via the DeckLink SDK,
// bypassing CasparCG's GPU mixer (works even where CasparCG renders black).
// mode: 0 = fill (colour + white digit), 1 = key (white + black digit).
export async function startDeckLinkOutputTest(index: number, mode: number): Promise<void> {
  return invoke('start_decklink_output_test', { index, mode });
}

export async function stopDeckLinkOutputTest(index: number): Promise<void> {
  return invoke('stop_decklink_output_test', { index });
}

// ============================================================================
// AMCP Commands
// ============================================================================

export async function amcpConnect(host: string, port: number): Promise<void> {
  return invoke('amcp_connect', { host, port });
}

export async function amcpDisconnect(): Promise<void> {
  return invoke('amcp_disconnect');
}

export async function amcpIsConnected(): Promise<boolean> {
  return invoke('amcp_is_connected');
}

export async function amcpConnectionInfo(): Promise<[string, number] | null> {
  return invoke('amcp_connection_info');
}

export async function amcpVersion(): Promise<string> {
  return invoke('amcp_version');
}

export async function amcpInfoSystem(): Promise<string> {
  return invoke('amcp_info_system');
}

export async function amcpSendCommand(command: string): Promise<AmcpResponse> {
  return invoke('amcp_send_command', { command });
}

// ============================================================================
// Test Server Commands
// ============================================================================

export async function startTestServer(port?: number): Promise<number> {
  return invoke('start_test_server', { port });
}

export async function stopTestServer(): Promise<void> {
  return invoke('stop_test_server');
}

export async function getTestServerUrl(): Promise<string | null> {
  return invoke('get_test_server_url');
}

export async function testChannel(channel: number): Promise<void> {
  return invoke('test_channel', { channel });
}

export async function stopChannelTest(channel: number): Promise<void> {
  return invoke('stop_channel_test', { channel });
}

export async function testAllChannels(channelCount: number): Promise<void> {
  return invoke('test_all_channels', { channelCount });
}

export async function stopAllChannelTests(channelCount: number): Promise<void> {
  return invoke('stop_all_channel_tests', { channelCount });
}

// ============================================================================
// System Info Commands
// ============================================================================

export async function getNdiVersion(): Promise<string | null> {
  return invoke('get_ndi_version');
}

export async function getScannerVersion(
  url?: string
): Promise<string | null> {
  return invoke('get_scanner_version', { url });
}

export async function getSystemVersions(): Promise<SystemVersions> {
  return invoke('get_system_versions');
}

// ============================================================================
// TSL UMD Tally Monitor Commands
// ============================================================================

export async function startTslMonitor(port?: number): Promise<number> {
  return invoke('start_tsl_monitor', { port });
}

export async function stopTslMonitor(): Promise<void> {
  return invoke('stop_tsl_monitor');
}

export async function getTslDisplays(): Promise<TslDisplay[]> {
  return invoke('get_tsl_displays');
}

export async function tslMonitorPort(): Promise<number | null> {
  return invoke('tsl_monitor_port');
}

// ============================================================================
// CasparCG Server Process Commands
// ============================================================================

export async function startCasparServer(config: GlobalConfig): Promise<void> {
  return invoke('start_caspar_server', { config });
}

export async function stopCasparServer(): Promise<void> {
  return invoke('stop_caspar_server');
}

export async function casparServerRunning(): Promise<boolean> {
  return invoke('caspar_server_running');
}

// ============================================================================
// GUI Settings Commands
// ============================================================================

export async function getGuiSettings(): Promise<GuiSettings> {
  return invoke('get_gui_settings');
}

export async function saveGuiSettings(settings: GuiSettings): Promise<void> {
  return invoke('save_gui_settings', { settings });
}

export async function setCasparPath(path: string): Promise<void> {
  return invoke('set_caspar_path', { path });
}

// ============================================================================
// File Dialog Commands
// ============================================================================

export async function pickFolder(): Promise<string | null> {
  return invoke('pick_folder');
}

export async function pickConfigFile(): Promise<string | null> {
  return invoke('pick_config_file');
}

export async function pickSaveLocation(
  defaultName: string
): Promise<string | null> {
  return invoke('pick_save_location', { defaultName });
}
