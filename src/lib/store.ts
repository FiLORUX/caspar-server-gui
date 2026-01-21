// Zustand store for application state management

import { create } from 'zustand';
import type {
  ConnectionStatus,
  DeckLinkDevice,
  GlobalConfig,
  GuiSettings,
  SystemVersions,
  TabId,
} from './types';
import * as tauri from './tauri';

interface AppState {
  // UI State
  activeTab: TabId;
  setActiveTab: (tab: TabId) => void;

  // Profile State
  profiles: string[];
  activeProfile: string | null;
  currentConfig: GlobalConfig | null;
  configDirty: boolean;
  loadProfiles: () => Promise<void>;
  selectProfile: (name: string) => Promise<void>;
  saveProfile: () => Promise<void>;
  createProfile: (name: string) => Promise<void>;
  deleteProfile: (name: string) => Promise<void>;
  updateConfig: (config: GlobalConfig) => void;

  // Connection State
  connection: ConnectionStatus;
  connect: (host: string, port: number) => Promise<void>;
  disconnect: () => Promise<void>;
  checkConnection: () => Promise<void>;

  // DeckLink State
  deckLinkDevices: DeckLinkDevice[];
  loadDeckLinkDevices: () => Promise<void>;

  // System Info State
  systemVersions: SystemVersions;
  loadSystemVersions: () => Promise<void>;

  // GUI Settings
  settings: GuiSettings | null;
  loadSettings: () => Promise<void>;
  saveSettings: (settings: Partial<GuiSettings>) => Promise<void>;

  // Test Server State
  testServerRunning: boolean;
  testServerUrl: string | null;
  channelsTesting: Set<number>;
  startTestServer: () => Promise<void>;
  stopTestServer: () => Promise<void>;
  testAllChannels: () => Promise<void>;
  stopAllTests: () => Promise<void>;
  testChannel: (channel: number) => Promise<void>;
  stopChannelTest: (channel: number) => Promise<void>;

  // Initialisation
  initialise: () => Promise<void>;
}

export const useAppStore = create<AppState>((set, get) => ({
  // UI State
  activeTab: 'paths',
  setActiveTab: (tab) => set({ activeTab: tab }),

  // Profile State
  profiles: [],
  activeProfile: null,
  currentConfig: null,
  configDirty: false,

  loadProfiles: async () => {
    try {
      const profiles = await tauri.listProfiles();
      set({ profiles });
    } catch (error) {
      console.error('Failed to load profiles:', error);
      set({ profiles: [] });
    }
  },

  selectProfile: async (name) => {
    const { settings } = get();
    if (!settings?.caspar_path) return;

    try {
      const path = `${settings.caspar_path}/caspar-gui-profiles/${name}.json`;
      const config = await tauri.loadGlobalConfig(path);
      set({ activeProfile: name, currentConfig: config, configDirty: false });
    } catch (error) {
      console.error('Failed to load profile:', error);
    }
  },

  saveProfile: async () => {
    const { activeProfile, currentConfig, settings } = get();
    if (!activeProfile || !currentConfig || !settings?.caspar_path) return;

    try {
      const path = `${settings.caspar_path}/caspar-gui-profiles/${activeProfile}.json`;
      await tauri.saveGlobalConfig(path, currentConfig);
      set({ configDirty: false });
    } catch (error) {
      console.error('Failed to save profile:', error);
      throw error;
    }
  },

  createProfile: async (name) => {
    const { settings, loadProfiles } = get();
    if (!settings?.caspar_path) return;

    try {
      const config = await tauri.createDefaultConfig(name);
      const path = `${settings.caspar_path}/caspar-gui-profiles/${name}.json`;
      await tauri.saveGlobalConfig(path, config);
      await loadProfiles();
      set({ activeProfile: name, currentConfig: config, configDirty: false });
    } catch (error) {
      console.error('Failed to create profile:', error);
      throw error;
    }
  },

  deleteProfile: async (name) => {
    // TODO: Implement delete with confirmation
    console.log('Delete profile:', name);
  },

  updateConfig: (config) => {
    set({ currentConfig: config, configDirty: true });
  },

  // Connection State
  connection: { connected: false },

  connect: async (host, port) => {
    try {
      await tauri.amcpConnect(host, port);
      const version = await tauri.amcpVersion();
      set({
        connection: { connected: true, host, port, version },
      });
    } catch (error) {
      console.error('Failed to connect:', error);
      set({ connection: { connected: false } });
      throw error;
    }
  },

  disconnect: async () => {
    try {
      await tauri.amcpDisconnect();
      set({ connection: { connected: false } });
    } catch (error) {
      console.error('Failed to disconnect:', error);
    }
  },

  checkConnection: async () => {
    try {
      const connected = await tauri.amcpIsConnected();
      if (connected) {
        const info = await tauri.amcpConnectionInfo();
        if (info) {
          const [host, port] = info;
          const version = await tauri.amcpVersion();
          set({ connection: { connected: true, host, port, version } });
          return;
        }
      }
      set({ connection: { connected: false } });
    } catch {
      set({ connection: { connected: false } });
    }
  },

  // DeckLink State
  deckLinkDevices: [],

  loadDeckLinkDevices: async () => {
    try {
      const devices = await tauri.listDeckLinkDevices();
      set({ deckLinkDevices: devices });
    } catch (error) {
      console.error('Failed to load DeckLink devices:', error);
      set({ deckLinkDevices: [] });
    }
  },

  // System Info State
  systemVersions: {},

  loadSystemVersions: async () => {
    try {
      const versions = await tauri.getSystemVersions();
      set({ systemVersions: versions });
    } catch (error) {
      console.error('Failed to load system versions:', error);
    }
  },

  // GUI Settings
  settings: null,

  loadSettings: async () => {
    try {
      const settings = await tauri.getGuiSettings();
      set({ settings });
    } catch (error) {
      console.error('Failed to load settings:', error);
    }
  },

  saveSettings: async (partial) => {
    const { settings } = get();
    if (!settings) return;

    const newSettings = { ...settings, ...partial };
    try {
      await tauri.saveGuiSettings(newSettings);
      set({ settings: newSettings });
    } catch (error) {
      console.error('Failed to save settings:', error);
      throw error;
    }
  },

  // Test Server State
  testServerRunning: false,
  testServerUrl: null,
  channelsTesting: new Set<number>(),

  startTestServer: async () => {
    try {
      const port = await tauri.startTestServer();
      const url = await tauri.getTestServerUrl();
      set({ testServerRunning: true, testServerUrl: url });
      console.log(`Test server started on port ${port}`);
    } catch (error) {
      console.error('Failed to start test server:', error);
      throw error;
    }
  },

  stopTestServer: async () => {
    try {
      await tauri.stopTestServer();
      set({ testServerRunning: false, testServerUrl: null });
    } catch (error) {
      console.error('Failed to stop test server:', error);
      throw error;
    }
  },

  testAllChannels: async () => {
    const { currentConfig, testServerRunning, startTestServer } = get();
    if (!currentConfig) return;

    // Ensure test server is running
    if (!testServerRunning) {
      await startTestServer();
    }

    const channelCount = currentConfig.caspar.channels.length;
    try {
      await tauri.testAllChannels(channelCount);
      // Mark all channels as testing
      const testingChannels = new Set<number>();
      for (let i = 1; i <= channelCount; i++) {
        testingChannels.add(i);
      }
      set({ channelsTesting: testingChannels });
    } catch (error) {
      console.error('Failed to test channels:', error);
      throw error;
    }
  },

  stopAllTests: async () => {
    const { currentConfig } = get();
    if (!currentConfig) return;

    const channelCount = currentConfig.caspar.channels.length;
    try {
      await tauri.stopAllChannelTests(channelCount);
      set({ channelsTesting: new Set<number>() });
    } catch (error) {
      console.error('Failed to stop channel tests:', error);
      throw error;
    }
  },

  testChannel: async (channel: number) => {
    const { testServerRunning, startTestServer, channelsTesting } = get();

    // Ensure test server is running
    if (!testServerRunning) {
      await startTestServer();
    }

    try {
      await tauri.testChannel(channel);
      const newTesting = new Set(channelsTesting);
      newTesting.add(channel);
      set({ channelsTesting: newTesting });
    } catch (error) {
      console.error(`Failed to test channel ${channel}:`, error);
      throw error;
    }
  },

  stopChannelTest: async (channel: number) => {
    const { channelsTesting } = get();

    try {
      await tauri.stopChannelTest(channel);
      const newTesting = new Set(channelsTesting);
      newTesting.delete(channel);
      set({ channelsTesting: newTesting });
    } catch (error) {
      console.error(`Failed to stop channel ${channel} test:`, error);
      throw error;
    }
  },

  // Initialisation
  initialise: async () => {
    const { loadSettings, loadProfiles, loadDeckLinkDevices, loadSystemVersions } = get();

    // Load settings first
    await loadSettings();

    // If CasparCG path is set, load profiles
    const currentSettings = get().settings;
    if (currentSettings?.caspar_path) {
      await loadProfiles();

      // Select last profile if set
      if (currentSettings.last_profile) {
        const { selectProfile } = get();
        await selectProfile(currentSettings.last_profile);
      }

      // Try auto-connect if server was running
      if (
        currentSettings.last_server_was_running &&
        currentSettings.last_host &&
        currentSettings.last_port
      ) {
        const { connect } = get();
        try {
          await connect(currentSettings.last_host, currentSettings.last_port);
        } catch {
          // Auto-connect failed, that's fine
        }
      }
    }

    // Load DeckLink devices and system versions
    await loadDeckLinkDevices();
    await loadSystemVersions();
  },
}));
