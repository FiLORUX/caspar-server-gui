// DeckLink configuration panel
// Shows installed DeckLink devices and allows configuration

import { useEffect, useState } from 'react';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';
import type { DeckLinkDevice, DeckLinkStatus } from '../lib/types';

export function DeckLinkPanel() {
  const { deckLinkDevices, loadDeckLinkDevices, currentConfig, updateConfig } = useAppStore();
  const [statuses, setStatuses] = useState<Record<string, DeckLinkStatus>>({});
  // Direct SDI test live per device: 1-based index -> mode (0 = fill, 1 = key).
  const [testing, setTesting] = useState<Map<number, number>>(new Map());

  // Start (or switch) the test on a device in the given mode. Switching mode on a
  // running test stops it first, since a card can only be opened once.
  const startTest = async (device: DeckLinkDevice, mode: number) => {
    if (testing.get(device.index) === mode) return;
    try {
      if (testing.has(device.index)) {
        await tauri.stopDeckLinkOutputTest(device.index);
      }
      await tauri.startDeckLinkOutputTest(device.index, mode);
      setTesting((prev) => new Map(prev).set(device.index, mode));
    } catch (error) {
      console.error('Direct SDI output test failed:', error);
      alert(`Direct SDI output test failed: ${error}`);
    }
  };

  const stopTest = async (device: DeckLinkDevice) => {
    try {
      await tauri.stopDeckLinkOutputTest(device.index);
    } catch (error) {
      console.error('Stop SDI test failed:', error);
    } finally {
      setTesting((prev) => {
        const next = new Map(prev);
        next.delete(device.index);
        return next;
      });
    }
  };

  // Stop every running test when the panel unmounts, so cards are released.
  useEffect(() => {
    return () => {
      for (const index of testing.keys()) {
        tauri.stopDeckLinkOutputTest(index).catch(() => {});
      }
    };
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  // The direct SDI test needs exclusive access to the card, so it cannot run
  // while CasparCG does. Poll the server state to gate the test button.
  const [serverRunning, setServerRunning] = useState(false);
  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      try {
        const r = await tauri.casparServerRunning();
        if (!cancelled) setServerRunning(r);
      } catch {
        /* ignore */
      }
    };
    poll();
    const id = setInterval(poll, 1500);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, []);

  // Poll live signal status (~2 s). IDeckLinkStatus is passive, so this does not
  // disturb capture/playback; it just reflects input/reference lock in the UI.
  useEffect(() => {
    if (deckLinkDevices.length === 0) return;
    let cancelled = false;
    const poll = async () => {
      const entries = await Promise.all(
        deckLinkDevices.map(async (d) => {
          try {
            return [d.persistent_id, await tauri.getDeckLinkStatus(d.index)] as const;
          } catch {
            return null;
          }
        })
      );
      if (cancelled) return;
      setStatuses((prev) => {
        const next = { ...prev };
        for (const entry of entries) {
          if (entry) next[entry[0]] = entry[1];
        }
        return next;
      });
    };
    poll();
    const id = setInterval(poll, 2000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [deckLinkDevices]);

  const handleRefresh = async () => {
    await loadDeckLinkDevices();
  };

  const handleSetDuplexMode = async (device: DeckLinkDevice, mode: string) => {
    try {
      await tauri.setDeckLinkDuplexMode(device.persistent_id, mode);
      await loadDeckLinkDevices();
    } catch (error) {
      console.error('Failed to set duplex mode:', error);
      alert(`Failed to set duplex mode: ${error}`);
    }
  };

  const handleWriteLabel = async (device: DeckLinkDevice, label: string) => {
    try {
      await tauri.setDeckLinkLabel(device.persistent_id, label);
      await loadDeckLinkDevices();
    } catch (error) {
      console.error('Failed to write label to device:', error);
      alert(`Failed to write label to device: ${error}`);
    }
  };

  const updateDeviceLabel = (persistentId: string, label: string) => {
    if (!currentConfig) return;

    const existingDevices = currentConfig.decklink.devices;
    const existingIndex = existingDevices.findIndex(
      (d) => d.persistent_id === persistentId
    );

    let newDevices;
    if (existingIndex >= 0) {
      newDevices = [...existingDevices];
      newDevices[existingIndex] = { ...newDevices[existingIndex], label };
    } else {
      const device = deckLinkDevices.find((d) => d.persistent_id === persistentId);
      if (!device) return;
      newDevices = [
        ...existingDevices,
        {
          persistent_id: persistentId,
          model_name: device.model_name,
          label,
        },
      ];
    }

    updateConfig({
      ...currentConfig,
      decklink: {
        ...currentConfig.decklink,
        devices: newDevices,
      },
    });
  };

  const getDeviceLabel = (device: DeckLinkDevice): string => {
    if (!currentConfig) return '';
    const configDevice = currentConfig.decklink.devices.find(
      (d) => d.persistent_id === device.persistent_id
    );
    return configDevice?.label || '';
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold">DeckLink Devices</h2>
          <p className="text-sm text-[var(--color-text-secondary)]">
            Configure Blackmagic DeckLink capture and output devices
          </p>
        </div>
        <button
          onClick={handleRefresh}
          className="px-4 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] rounded hover:bg-[var(--color-border)] transition-colors"
        >
          Refresh Devices
        </button>
      </div>

      {deckLinkDevices.length === 0 ? (
        <div className="panel">
          <div className="p-8 text-center">
            <div className="text-4xl mb-4">🎬</div>
            <h3 className="text-lg font-medium mb-2">No DeckLink Devices Detected</h3>
            <p className="text-sm text-[var(--color-text-muted)] max-w-md mx-auto">
              Make sure Blackmagic Desktop Video software is installed and your DeckLink
              cards are properly connected.
            </p>
          </div>
        </div>
      ) : (
        <div className="space-y-4">
          {deckLinkDevices.map((device) => (
            <DeckLinkDeviceCard
              key={device.persistent_id}
              device={device}
              label={getDeviceLabel(device)}
              status={statuses[device.persistent_id]}
              onLabelChange={(label) => updateDeviceLabel(device.persistent_id, label)}
              onDuplexModeChange={(mode) => handleSetDuplexMode(device, mode)}
              onWriteLabel={() => handleWriteLabel(device, getDeviceLabel(device))}
              liveMode={testing.has(device.index) ? (testing.get(device.index) as number) : -1}
              onStart={(mode) => startTest(device, mode)}
              onStop={() => stopTest(device)}
              serverRunning={serverRunning}
            />
          ))}
        </div>
      )}

      <div className="mt-6 p-4 bg-[var(--color-bg-tertiary)] rounded text-sm text-[var(--color-text-muted)]">
        <strong className="text-[var(--color-text-secondary)]">Note:</strong> Duplex mode
        changes require a system restart to take effect. The device indices shown here
        correspond to the device numbers used in the channel configuration.
      </div>
    </div>
  );
}

interface DeckLinkDeviceCardProps {
  device: DeckLinkDevice;
  label: string;
  status?: DeckLinkStatus;
  onLabelChange: (label: string) => void;
  onDuplexModeChange: (mode: string) => void;
  onWriteLabel: () => void;
  liveMode: number; // -1 = not testing, 0 = fill live, 1 = key live
  onStart: (mode: number) => void;
  onStop: () => void;
  serverRunning: boolean;
}

function DeckLinkDeviceCard({
  device,
  label,
  status,
  onLabelChange,
  onDuplexModeChange,
  onWriteLabel,
  liveMode,
  onStart,
  onStop,
  serverRunning,
}: DeckLinkDeviceCardProps) {
  const isTesting = liveMode >= 0;
  // The card can only be driven by one owner. While CasparCG runs it holds the
  // card, so starting the direct SDI test is unavailable until the server stops.
  const testBlocked = serverRunning && !isTesting;
  return (
    <div className="decklink-card">
      <div className="flex items-start justify-between mb-4">
        <div>
          <div className="flex items-center gap-2">
            <span className="decklink-model">{device.model_name}</span>
            <span className="px-2 py-0.5 text-xs bg-[var(--color-accent)] text-white rounded">
              Device {device.index}
            </span>
            {isTesting && (
              <span className="px-2 py-0.5 text-xs font-medium bg-emerald-600/20 text-emerald-400 rounded">
                {liveMode === 1 ? 'KEY' : 'FILL'} SDI TEST LIVE
              </span>
            )}
          </div>
          <div className="decklink-id mt-1">{device.persistent_id}</div>
        </div>
        {/* Direct SDI test: Fill and Key each start that mode (switching is one
            click); Stop releases the card. Fill/Key need the server stopped. */}
        <div className="flex items-center gap-2 text-sm">
          <button
            onClick={() => onStart(0)}
            disabled={testBlocked}
            title={
              testBlocked
                ? 'Stop the CasparCG server first — it holds the card.'
                : "Output this card's colour field with its number (fill), straight to SDI."
            }
            className={`px-3 py-1.5 rounded transition-colors ${
              testBlocked
                ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] cursor-not-allowed'
                : liveMode === 0
                  ? 'bg-emerald-600 text-white'
                  : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]'
            }`}
          >
            ▶ Fill
          </button>
          <button
            onClick={() => onStart(1)}
            disabled={testBlocked}
            title={
              testBlocked
                ? 'Stop the CasparCG server first — it holds the card.'
                : 'Output the matching key (white field, black number) straight to SDI.'
            }
            className={`px-3 py-1.5 rounded transition-colors ${
              testBlocked
                ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] cursor-not-allowed'
                : liveMode === 1
                  ? 'bg-emerald-600 text-white'
                  : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]'
            }`}
          >
            ▶ Key
          </button>
          <button
            onClick={onStop}
            disabled={!isTesting}
            title={isTesting ? 'Stop the SDI test and release the card.' : undefined}
            className={`px-3 py-1.5 rounded transition-colors ${
              isTesting
                ? 'bg-amber-600 hover:bg-amber-700 text-white'
                : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] cursor-not-allowed'
            }`}
          >
            ■ Stop
          </button>
        </div>
      </div>

      <div className="grid grid-cols-2 gap-4">
        {/* Label */}
        <div>
          <label className="block text-sm text-[var(--color-text-secondary)] mb-1">
            Label
          </label>
          <div className="flex gap-2">
            <input
              type="text"
              value={label}
              onChange={(e) => onLabelChange(e.target.value)}
              placeholder="e.g., Graphics Fill"
              className="flex-1 text-sm"
            />
            <button
              onClick={onWriteLabel}
              title="Write this label to the card's NVRAM (persists across reboots, visible to Desktop Video and CasparCG)"
              className="px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] rounded hover:bg-[var(--color-border)] whitespace-nowrap"
            >
              ⤓ To device
            </button>
          </div>
        </div>

        {/* Duplex Mode */}
        {device.supports_duplex && (
          <div>
            <label className="block text-sm text-[var(--color-text-secondary)] mb-1">
              Duplex Mode
            </label>
            <select
              value={device.duplex_mode || 'half'}
              onChange={(e) => onDuplexModeChange(e.target.value)}
              className="w-full text-sm"
            >
              <option value="half">Half Duplex (Independent I/O)</option>
              <option value="full">Full Duplex (Key/Fill pair)</option>
            </select>
          </div>
        )}
      </div>

      {/* Capabilities */}
      <div className="mt-4 flex flex-wrap gap-2">
        {device.input_connectors.length > 0 && (
          <span className="px-2 py-1 text-xs bg-[var(--color-bg-primary)] rounded">
            In: {device.input_connectors.join(', ')}
          </span>
        )}
        {device.output_connectors.length > 0 && (
          <span className="px-2 py-1 text-xs bg-[var(--color-bg-primary)] rounded">
            Out: {device.output_connectors.join(', ')}
          </span>
        )}
        {device.supports_internal_keying && (
          <span className="px-2 py-1 text-xs bg-[var(--color-bg-primary)] rounded">
            Internal Keying
          </span>
        )}
        {device.supports_external_keying && (
          <span className="px-2 py-1 text-xs bg-[var(--color-bg-primary)] rounded">
            External Keying
          </span>
        )}
      </div>

      {/* Live signal status (IDeckLinkStatus, polled) */}
      {status && (
        <div className="mt-3 flex flex-wrap gap-2 text-xs">
          <span
            className={`px-2 py-1 rounded ${
              status.input_signal_locked
                ? 'bg-emerald-600/20 text-emerald-400'
                : 'bg-[var(--color-bg-primary)] text-[var(--color-text-muted)]'
            }`}
          >
            Input: {status.input_signal_locked ? status.input_display_mode ?? 'locked' : 'no signal'}
          </span>
          <span
            className={`px-2 py-1 rounded ${
              status.reference_signal_locked
                ? 'bg-emerald-600/20 text-emerald-400'
                : 'bg-[var(--color-bg-primary)] text-[var(--color-text-muted)]'
            }`}
          >
            Ref:{' '}
            {status.reference_signal_locked
              ? `${status.reference_display_mode ?? 'locked'}${
                  status.reference_type ? ` (${status.reference_type})` : ''
                }`
              : 'none'}
          </span>
        </div>
      )}
    </div>
  );
}
