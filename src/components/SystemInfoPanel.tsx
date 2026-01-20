// System information panel
// Shows version info for CasparCG, DeckLink, NDI, and Scanner

import { useState } from 'react';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';

export function SystemInfoPanel() {
  const {
    systemVersions,
    loadSystemVersions,
    connection,
    deckLinkDevices,
  } = useAppStore();
  const [isRefreshing, setIsRefreshing] = useState(false);
  const [xmlPreview, setXmlPreview] = useState<string | null>(null);

  const handleRefresh = async () => {
    setIsRefreshing(true);
    await loadSystemVersions();
    setIsRefreshing(false);
  };

  const handleExportXml = async () => {
    const { currentConfig } = useAppStore.getState();
    if (!currentConfig) {
      alert('No configuration loaded');
      return;
    }

    try {
      const xml = await tauri.exportToCasparXml(currentConfig);
      setXmlPreview(xml);
    } catch (error) {
      alert(`Failed to generate XML: ${error}`);
    }
  };

  const handleSaveXml = async () => {
    const { currentConfig, settings } = useAppStore.getState();
    if (!currentConfig || !settings?.caspar_path) return;

    try {
      const path = `${settings.caspar_path}/casparcg.config`;
      await tauri.saveCasparConfig(path, currentConfig.caspar);
      alert('Configuration saved to casparcg.config');
    } catch (error) {
      alert(`Failed to save: ${error}`);
    }
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold">System Information</h2>
          <p className="text-sm text-[var(--color-text-secondary)]">
            Version information and system status
          </p>
        </div>
        <button
          onClick={handleRefresh}
          disabled={isRefreshing}
          className="px-4 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] rounded hover:bg-[var(--color-border)] transition-colors disabled:opacity-50"
        >
          {isRefreshing ? 'Refreshing...' : 'Refresh'}
        </button>
      </div>

      {/* Version grid */}
      <div className="system-info-grid mb-6">
        <div className="system-info-item">
          <div className="system-info-label">CasparCG Server</div>
          <div
            className={`system-info-value ${
              !connection.version ? 'not-available' : ''
            }`}
          >
            {connection.version || (connection.connected ? 'Unknown' : 'Not connected')}
          </div>
        </div>

        <div className="system-info-item">
          <div className="system-info-label">DeckLink Driver</div>
          <div
            className={`system-info-value ${
              !systemVersions.decklink_version ? 'not-available' : ''
            }`}
          >
            {systemVersions.decklink_version || 'Not installed'}
          </div>
        </div>

        <div className="system-info-item">
          <div className="system-info-label">NDI Tools</div>
          <div
            className={`system-info-value ${
              !systemVersions.ndi_version ? 'not-available' : ''
            }`}
          >
            {systemVersions.ndi_version || 'Not installed'}
          </div>
        </div>

        <div className="system-info-item">
          <div className="system-info-label">Media Scanner</div>
          <div
            className={`system-info-value ${
              !systemVersions.scanner_version ? 'not-available' : ''
            }`}
          >
            {systemVersions.scanner_version || 'Not running'}
          </div>
        </div>
      </div>

      {/* Connection status */}
      <div className="panel mb-6">
        <div className="panel-header">Connection Status</div>
        <div className="panel-content">
          {connection.connected ? (
            <div className="flex items-center gap-3">
              <span className="status-dot connected" />
              <div>
                <div className="font-medium">
                  Connected to {connection.host}:{connection.port}
                </div>
                <div className="text-sm text-[var(--color-text-muted)]">
                  CasparCG Server version {connection.version}
                </div>
              </div>
            </div>
          ) : (
            <div className="flex items-center gap-3">
              <span className="status-dot disconnected" />
              <div>
                <div className="font-medium">Not connected</div>
                <div className="text-sm text-[var(--color-text-muted)]">
                  Click "Connect..." in the status bar to connect to a CasparCG server
                </div>
              </div>
            </div>
          )}
        </div>
      </div>

      {/* Hardware summary */}
      <div className="panel mb-6">
        <div className="panel-header">Hardware Summary</div>
        <div className="panel-content">
          <div className="text-sm">
            <div className="flex justify-between py-2 border-b border-[var(--color-border)]">
              <span className="text-[var(--color-text-secondary)]">DeckLink Cards</span>
              <span>{deckLinkDevices.length}</span>
            </div>
            {deckLinkDevices.map((device) => (
              <div
                key={device.persistent_id}
                className="flex justify-between py-2 border-b border-[var(--color-border)] pl-4"
              >
                <span className="text-[var(--color-text-muted)]">
                  Device {device.index}
                </span>
                <span className="text-[var(--color-text-secondary)]">
                  {device.model_name}
                </span>
              </div>
            ))}
          </div>
        </div>
      </div>

      {/* Export section */}
      <div className="panel">
        <div className="panel-header">Export Configuration</div>
        <div className="panel-content">
          <p className="text-sm text-[var(--color-text-muted)] mb-4">
            Export the current profile to a standard CasparCG configuration file.
          </p>

          <div className="flex gap-2 mb-4">
            <button
              onClick={handleExportXml}
              className="px-4 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] rounded hover:bg-[var(--color-border)] transition-colors"
            >
              Preview XML
            </button>
            <button
              onClick={handleSaveXml}
              className="px-4 py-2 bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)] transition-colors"
            >
              Save to casparcg.config
            </button>
          </div>

          {xmlPreview && (
            <div className="mt-4">
              <div className="flex items-center justify-between mb-2">
                <span className="text-sm font-medium">XML Preview</span>
                <button
                  onClick={() => setXmlPreview(null)}
                  className="text-xs text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
                >
                  Close
                </button>
              </div>
              <pre className="p-4 bg-[var(--color-bg-primary)] rounded text-xs overflow-auto max-h-96 font-mono">
                {xmlPreview}
              </pre>
            </div>
          )}
        </div>
      </div>
    </div>
  );
}
