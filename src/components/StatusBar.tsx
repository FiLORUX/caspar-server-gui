// Status bar component
// Shows connection status and system information

import { useState } from 'react';
import { useAppStore } from '../lib/store';

export function StatusBar() {
  const {
    connection,
    connect,
    disconnect,
    systemVersions,
    deckLinkDevices,
  } = useAppStore();
  const [showConnectDialog, setShowConnectDialog] = useState(false);
  const [host, setHost] = useState('localhost');
  const [port, setPort] = useState('5250');
  const [isConnecting, setIsConnecting] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const handleConnect = async (e: React.FormEvent) => {
    e.preventDefault();
    setIsConnecting(true);
    setError(null);

    try {
      await connect(host, parseInt(port, 10));
      setShowConnectDialog(false);
    } catch (err) {
      setError(String(err));
    } finally {
      setIsConnecting(false);
    }
  };

  const handleDisconnect = async () => {
    await disconnect();
  };

  // Get primary DeckLink info
  const deckLinkInfo = deckLinkDevices.length > 0
    ? deckLinkDevices[0].model_name
    : null;

  return (
    <>
      <div className="flex items-center h-8 px-4 bg-[var(--color-bg-secondary)] border-t border-[var(--color-border)] text-xs">
        {/* Server status */}
        <div className="flex items-center gap-2 mr-6">
          <span className={`status-dot ${connection.connected ? 'connected' : 'disconnected'}`} />
          <span className="text-[var(--color-text-secondary)]">Server:</span>
          {connection.connected ? (
            <>
              <span className="text-[var(--color-success)]">
                Connected ({connection.version})
              </span>
              <button
                onClick={handleDisconnect}
                className="ml-2 text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
              >
                Disconnect
              </button>
            </>
          ) : (
            <button
              onClick={() => setShowConnectDialog(true)}
              className="text-[var(--color-accent)] hover:text-[var(--color-accent-hover)]"
            >
              Connect...
            </button>
          )}
        </div>

        {/* Divider */}
        <div className="w-px h-4 bg-[var(--color-border)] mx-2" />

        {/* DeckLink info */}
        <div className="flex items-center gap-2 mr-6">
          <span className="text-[var(--color-text-secondary)]">DeckLink:</span>
          <span className={deckLinkInfo ? 'text-[var(--color-text-primary)]' : 'text-[var(--color-text-muted)]'}>
            {deckLinkInfo || 'Not detected'}
          </span>
        </div>

        {/* NDI info */}
        {systemVersions.ndi_version && (
          <>
            <div className="w-px h-4 bg-[var(--color-border)] mx-2" />
            <div className="flex items-center gap-2">
              <span className="text-[var(--color-text-secondary)]">NDI:</span>
              <span className="text-[var(--color-text-primary)]">
                {systemVersions.ndi_version}
              </span>
            </div>
          </>
        )}

        {/* Spacer */}
        <div className="flex-1" />

        {/* App version */}
        <span className="text-[var(--color-text-muted)]">
          CasparCG Server GUI v0.1.0
        </span>
      </div>

      {/* Connect dialog */}
      {showConnectDialog && (
        <div className="fixed inset-0 bg-black/50 flex items-center justify-center z-50">
          <div className="panel w-96">
            <div className="panel-header">Connect to CasparCG Server</div>
            <form onSubmit={handleConnect} className="panel-content">
              <div className="mb-4">
                <label className="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">
                  Host
                </label>
                <input
                  type="text"
                  value={host}
                  onChange={(e) => setHost(e.target.value)}
                  className="w-full"
                  placeholder="localhost"
                />
              </div>

              <div className="mb-4">
                <label className="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">
                  Port
                </label>
                <input
                  type="number"
                  value={port}
                  onChange={(e) => setPort(e.target.value)}
                  className="w-full"
                  placeholder="5250"
                />
              </div>

              {error && (
                <div className="mb-4 p-2 bg-red-900/30 border border-red-500 rounded text-red-300 text-sm">
                  {error}
                </div>
              )}

              <div className="flex justify-end gap-2">
                <button
                  type="button"
                  onClick={() => setShowConnectDialog(false)}
                  className="px-4 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] rounded hover:bg-[var(--color-border)]"
                >
                  Cancel
                </button>
                <button
                  type="submit"
                  disabled={isConnecting}
                  className="px-4 py-2 bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)] disabled:opacity-50"
                >
                  {isConnecting ? 'Connecting...' : 'Connect'}
                </button>
              </div>
            </form>
          </div>
        </div>
      )}
    </>
  );
}
