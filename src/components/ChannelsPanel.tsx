// Channels configuration panel
// Configure video channels and their consumers. Validation is driven entirely
// by validateConfig(): impossible choices are greyed out, and anything that
// would still be wrong is shown inline so the operator sees it before launch.

import { useState } from 'react';
import { useAppStore } from '../lib/store';
import type {
  Channel,
  Consumer,
  GlobalConfig,
  VideoMode,
  DeckLinkConsumer,
  DeckLinkDevice,
  NdiConsumer,
  ScreenConsumer,
} from '../lib/types';
import {
  VIDEO_MODES,
  DEFAULT_CHANNEL,
  createDefaultDeckLinkConsumer,
  createDefaultNdiConsumer,
  createDefaultScreenConsumer,
  createDefaultSystemAudioConsumer,
} from '../lib/types';
import {
  validateConfig,
  devicesClaimedElsewhere,
  issuesForChannel,
  issuesForConsumer,
  type ValidationIssue,
} from '../lib/validation';

/** Inline list of validation issues, colour-coded by severity. */
function IssueList({ issues }: { issues: ValidationIssue[] }) {
  if (issues.length === 0) return null;
  return (
    <div className="mt-2 space-y-1">
      {issues.map((issue) => (
        <div
          key={issue.id}
          className={`text-xs flex items-start gap-1.5 ${
            issue.severity === 'error' ? 'text-red-400' : 'text-amber-400'
          }`}
        >
          <span aria-hidden>{issue.severity === 'error' ? '✕' : '⚠'}</span>
          <span>{issue.message}</span>
        </div>
      ))}
    </div>
  );
}

export function ChannelsPanel() {
  const {
    currentConfig,
    updateConfig,
    deckLinkDevices,
    connection,
    channelsTesting,
    testAllChannels,
    stopAllTests,
    testChannel,
    stopChannelTest,
  } = useAppStore();

  if (!currentConfig) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--color-text-muted)]">
        Select or create a profile to configure channels
      </div>
    );
  }

  const channels = currentConfig.caspar.channels;
  const anyChannelTesting = channelsTesting.size > 0;
  const issues = validateConfig(currentConfig, deckLinkDevices);

  const handleTestChannels = async () => {
    try {
      if (anyChannelTesting) {
        await stopAllTests();
      } else {
        await testAllChannels();
      }
    } catch (error) {
      console.error('Test channels failed:', error);
    }
  };

  const updateChannels = (newChannels: Channel[]) => {
    const newConfig: GlobalConfig = {
      ...currentConfig,
      caspar: {
        ...currentConfig.caspar,
        channels: newChannels,
      },
    };
    updateConfig(newConfig);
  };

  const addChannel = () => {
    updateChannels([...channels, { ...DEFAULT_CHANNEL, id: crypto.randomUUID() }]);
  };

  const removeChannel = (index: number) => {
    if (channels.length <= 1) return;
    updateChannels(channels.filter((_, i) => i !== index));
  };

  const updateChannel = (index: number, channel: Channel) => {
    const newChannels = [...channels];
    newChannels[index] = channel;
    updateChannels(newChannels);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold">Channel Configuration</h2>
          <p className="text-sm text-[var(--color-text-secondary)]">
            Configure video channels and their output consumers
          </p>
        </div>
        <div className="flex gap-2">
          <button
            onClick={handleTestChannels}
            disabled={!connection.connected}
            className={`px-4 py-2 rounded transition-colors ${
              !connection.connected
                ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] cursor-not-allowed'
                : anyChannelTesting
                  ? 'bg-amber-600 hover:bg-amber-700 text-white'
                  : 'bg-emerald-600 hover:bg-emerald-700 text-white'
            }`}
            title={!connection.connected ? 'Connect to CasparCG server first' : undefined}
          >
            {anyChannelTesting ? '■ Stop Test' : '▶ Test Channels'}
          </button>
          <button
            onClick={addChannel}
            className="px-4 py-2 bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)] transition-colors"
          >
            + Add Channel
          </button>
        </div>
      </div>

      <div className="space-y-4">
        {channels.map((channel, index) => (
          <ChannelCard
            key={channel.id ?? index}
            channel={channel}
            index={index}
            config={currentConfig}
            issues={issues}
            onUpdate={(ch) => updateChannel(index, ch)}
            onRemove={() => removeChannel(index)}
            canRemove={channels.length > 1}
            deckLinkDevices={deckLinkDevices}
            isTesting={channelsTesting.has(index + 1)}
            canTest={connection.connected}
            onToggleTest={async () => {
              try {
                if (channelsTesting.has(index + 1)) {
                  await stopChannelTest(index + 1);
                } else {
                  await testChannel(index + 1);
                }
              } catch (error) {
                console.error('Toggle channel test failed:', error);
              }
            }}
          />
        ))}
      </div>
    </div>
  );
}

interface ChannelCardProps {
  channel: Channel;
  index: number;
  config: GlobalConfig;
  issues: ValidationIssue[];
  onUpdate: (channel: Channel) => void;
  onRemove: () => void;
  canRemove: boolean;
  deckLinkDevices: DeckLinkDevice[];
  isTesting: boolean;
  canTest: boolean;
  onToggleTest: () => void;
}

function ChannelCard({
  channel,
  index,
  config,
  issues,
  onUpdate,
  onRemove,
  canRemove,
  deckLinkDevices,
  isTesting,
  canTest,
  onToggleTest,
}: ChannelCardProps) {
  const [isExpanded, setIsExpanded] = useState(true);
  const channelIssues = issuesForChannel(issues, index);

  const addConsumer = (type: Consumer['type']) => {
    let newConsumer: Consumer;
    switch (type) {
      case 'decklink':
        newConsumer = createDefaultDeckLinkConsumer();
        // Default to the first card not already claimed elsewhere, so a fresh
        // consumer starts valid rather than clashing with an existing one.
        {
          const taken = devicesClaimedElsewhere(config, index, channel.consumers.length);
          const free = deckLinkDevices.find((d) => !taken.has(d.index));
          if (free) newConsumer.device = free.index;
        }
        break;
      case 'ndi':
        newConsumer = createDefaultNdiConsumer();
        break;
      case 'screen':
        newConsumer = createDefaultScreenConsumer();
        break;
      case 'system-audio':
        newConsumer = createDefaultSystemAudioConsumer();
        break;
    }
    onUpdate({
      ...channel,
      consumers: [...channel.consumers, newConsumer],
    });
  };

  const updateConsumer = (consumerIndex: number, consumer: Consumer) => {
    const newConsumers = [...channel.consumers];
    newConsumers[consumerIndex] = consumer;
    onUpdate({ ...channel, consumers: newConsumers });
  };

  const removeConsumer = (consumerIndex: number) => {
    onUpdate({
      ...channel,
      consumers: channel.consumers.filter((_, i) => i !== consumerIndex),
    });
  };

  return (
    <div className="panel">
      {/* Channel header */}
      <div className="flex items-center justify-between p-4 border-b border-[var(--color-border)]">
        <div className="flex items-center gap-4">
          <button
            onClick={() => setIsExpanded(!isExpanded)}
            className="text-[var(--color-text-muted)] hover:text-[var(--color-text-primary)]"
          >
            {isExpanded ? '▼' : '▶'}
          </button>
          <span className="font-semibold">Channel {index + 1}</span>

          <select
            value={channel.video_mode}
            onChange={(e) =>
              onUpdate({ ...channel, video_mode: e.target.value as VideoMode })
            }
            className="ml-4"
          >
            {VIDEO_MODES.map((mode) => (
              <option key={mode.value} value={mode.value}>
                {mode.label}
              </option>
            ))}
          </select>

          {/* Test indicator */}
          {isTesting && (
            <span className="px-2 py-0.5 text-xs font-medium bg-emerald-600/20 text-emerald-400 rounded">
              TESTING
            </span>
          )}
        </div>

        <div className="flex items-center gap-2">
          <button
            onClick={onToggleTest}
            disabled={!canTest}
            className={`px-2 py-1 text-xs rounded transition-colors ${
              !canTest
                ? 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-muted)] cursor-not-allowed'
                : isTesting
                  ? 'bg-amber-600/20 text-amber-400 hover:bg-amber-600/30'
                  : 'bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] hover:bg-[var(--color-border)]'
            }`}
            title={!canTest ? 'Connect to CasparCG server first' : isTesting ? 'Stop test' : 'Test channel'}
          >
            {isTesting ? '■ Stop' : '▶ Test'}
          </button>

          {canRemove && (
            <button
              onClick={onRemove}
              className="text-[var(--color-text-muted)] hover:text-red-400"
              title="Remove channel"
            >
              ✕
            </button>
          )}
        </div>
      </div>

      {/* Channel content */}
      {isExpanded && (
        <div className="p-4">
          <div className="flex items-center justify-between mb-3">
            <span className="text-sm font-medium text-[var(--color-text-secondary)]">
              Consumers ({channel.consumers.length})
            </span>
            <div className="flex gap-2">
              <button
                onClick={() => addConsumer('decklink')}
                className="px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] rounded hover:bg-[var(--color-border)]"
              >
                + DeckLink
              </button>
              <button
                onClick={() => addConsumer('ndi')}
                className="px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] rounded hover:bg-[var(--color-border)]"
              >
                + NDI
              </button>
              <button
                onClick={() => addConsumer('screen')}
                className="px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] rounded hover:bg-[var(--color-border)]"
              >
                + Screen
              </button>
              <button
                onClick={() => addConsumer('system-audio')}
                className="px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] rounded hover:bg-[var(--color-border)]"
              >
                + Audio
              </button>
            </div>
          </div>

          <IssueList issues={channelIssues} />

          {channel.consumers.length === 0 ? (
            <div className="text-sm text-[var(--color-text-muted)] text-center py-4">
              No consumers configured. Add a consumer to output video.
            </div>
          ) : (
            <div className="space-y-3 mt-3">
              {channel.consumers.map((consumer, ci) => (
                <ConsumerCard
                  key={ci}
                  consumer={consumer}
                  channelIndex={index}
                  consumerIndex={ci}
                  config={config}
                  issues={issues}
                  onUpdate={(c) => updateConsumer(ci, c)}
                  onRemove={() => removeConsumer(ci)}
                  deckLinkDevices={deckLinkDevices}
                />
              ))}
            </div>
          )}
        </div>
      )}
    </div>
  );
}

interface ConsumerCardProps {
  consumer: Consumer;
  channelIndex: number;
  consumerIndex: number;
  config: GlobalConfig;
  issues: ValidationIssue[];
  onUpdate: (consumer: Consumer) => void;
  onRemove: () => void;
  deckLinkDevices: DeckLinkDevice[];
}

function ConsumerCard({
  consumer,
  channelIndex,
  consumerIndex,
  config,
  issues,
  onUpdate,
  onRemove,
  deckLinkDevices,
}: ConsumerCardProps) {
  const typeLabels: Record<Consumer['type'], string> = {
    decklink: 'DeckLink',
    ndi: 'NDI',
    screen: 'Screen',
    'system-audio': 'System Audio',
  };

  const consumerIssues = issuesForConsumer(issues, channelIndex, consumerIndex);
  const hasError = consumerIssues.some((i) => i.severity === 'error');

  return (
    <div className={`consumer-card ${hasError ? 'ring-1 ring-red-500/40' : ''}`}>
      <div className="consumer-card-header">
        <span className="consumer-type-badge">{typeLabels[consumer.type]}</span>
        <button
          onClick={onRemove}
          className="text-[var(--color-text-muted)] hover:text-red-400"
          title="Remove consumer"
        >
          ✕
        </button>
      </div>

      {consumer.type === 'decklink' && (
        <DeckLinkConsumerForm
          consumer={consumer}
          channelIndex={channelIndex}
          consumerIndex={consumerIndex}
          config={config}
          onUpdate={(c) => onUpdate(c)}
          devices={deckLinkDevices}
        />
      )}

      {consumer.type === 'ndi' && (
        <NdiConsumerForm consumer={consumer} onUpdate={(c) => onUpdate(c)} />
      )}

      {consumer.type === 'screen' && (
        <ScreenConsumerForm consumer={consumer} onUpdate={(c) => onUpdate(c)} />
      )}

      {consumer.type === 'system-audio' && (
        <div className="text-sm text-[var(--color-text-muted)]">
          System audio output (uses default audio device)
        </div>
      )}

      <IssueList issues={consumerIssues} />
    </div>
  );
}

interface DeckLinkConsumerFormProps {
  consumer: DeckLinkConsumer;
  channelIndex: number;
  consumerIndex: number;
  config: GlobalConfig;
  onUpdate: (consumer: DeckLinkConsumer) => void;
  devices: DeckLinkDevice[];
}

function DeckLinkConsumerForm({
  consumer,
  channelIndex,
  consumerIndex,
  config,
  onUpdate,
  devices,
}: DeckLinkConsumerFormProps) {
  const selectedDevice = devices.find((d) => d.index === consumer.device);
  const claimedElsewhere = devicesClaimedElsewhere(config, channelIndex, consumerIndex);
  const deviceListed = devices.some((d) => d.index === consumer.device);
  const usesKeyDevice = consumer.keyer === 'external_separate_device';

  return (
    <div className="grid grid-cols-2 gap-3 text-sm">
      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Device</label>
        <select
          value={consumer.device}
          onChange={(e) => onUpdate({ ...consumer, device: parseInt(e.target.value, 10) })}
          className="w-full text-sm"
        >
          {devices.map((d) => {
            const taken = claimedElsewhere.has(d.index);
            return (
              <option key={d.index} value={d.index} disabled={taken}>
                {d.display_name}
                {taken ? ' — in use' : ''}
              </option>
            );
          })}
          {/* Keep the configured value visible and selectable even when the card
              is absent (or nothing was enumerated), so the value never silently
              snaps to another device. */}
          {!deviceListed && (
            <option value={consumer.device}>
              Device {consumer.device}
              {devices.length > 0 ? ' — not detected' : ''}
            </option>
          )}
        </select>
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Key Device</label>
        <select
          value={consumer.key_device ?? ''}
          disabled={!usesKeyDevice}
          onChange={(e) =>
            onUpdate({
              ...consumer,
              key_device: e.target.value ? parseInt(e.target.value, 10) : undefined,
            })
          }
          className="w-full text-sm disabled:opacity-50"
          title={usesKeyDevice ? undefined : 'Only used with External (Separate Device) keying'}
        >
          <option value="">None</option>
          {devices.map((d) => {
            const isFill = d.index === consumer.device;
            const taken = claimedElsewhere.has(d.index);
            return (
              <option key={d.index} value={d.index} disabled={isFill || taken}>
                {d.display_name}
                {isFill ? ' — fill device' : taken ? ' — in use' : ''}
              </option>
            );
          })}
        </select>
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Latency</label>
        <select
          value={consumer.latency}
          onChange={(e) => onUpdate({ ...consumer, latency: e.target.value as DeckLinkConsumer['latency'] })}
          className="w-full text-sm"
        >
          <option value="normal">Normal</option>
          <option value="low">Low</option>
          <option value="default">Default</option>
        </select>
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Keyer</label>
        <select
          value={consumer.keyer}
          onChange={(e) => {
            const keyer = e.target.value as DeckLinkConsumer['keyer'];
            // The key device is only meaningful for external_separate_device;
            // drop it when switching away so the saved profile stays clean.
            onUpdate({
              ...consumer,
              keyer,
              key_device: keyer === 'external_separate_device' ? consumer.key_device : undefined,
            });
          }}
          className="w-full text-sm"
        >
          <option value="default">Default (plain fill, no keying)</option>
          <option
            value="external"
            disabled={selectedDevice ? !selectedDevice.supports_external_keying : false}
          >
            External
            {selectedDevice && !selectedDevice.supports_external_keying
              ? ' — not supported by this card'
              : ''}
          </option>
          {/* External (Separate Device) uses two independent fill outputs — fill
              on this device, key on the key-device — so it needs no hardware
              keyer and is the fill+key route for cards without one. It does,
              however, need a second card; greyed out when only one is detected. */}
          <option value="external_separate_device" disabled={devices.length === 1}>
            External (Separate Device)
            {devices.length === 1 ? ' — needs a second card' : ''}
          </option>
          <option
            value="internal"
            disabled={selectedDevice ? !selectedDevice.supports_internal_keying : false}
          >
            Internal
            {selectedDevice && !selectedDevice.supports_internal_keying
              ? ' — not supported by this card'
              : ''}
          </option>
        </select>
      </div>

      {selectedDevice && (
        <div className="col-span-2 text-xs text-[var(--color-text-muted)]">
          {selectedDevice.model_name} keying:{' '}
          {selectedDevice.supports_internal_keying || selectedDevice.supports_external_keying
            ? [
                selectedDevice.supports_internal_keying && 'internal',
                selectedDevice.supports_external_keying && 'external',
              ]
                .filter(Boolean)
                .join(', ')
            : 'none — single output, fill or key only (pair a separate key device for fill+key)'}
        </div>
      )}

      <div className="col-span-2 flex items-center gap-4">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={consumer.embedded_audio}
            onChange={(e) => onUpdate({ ...consumer, embedded_audio: e.target.checked })}
          />
          <span className="text-[var(--color-text-muted)]">Embedded Audio</span>
        </label>
      </div>
    </div>
  );
}

interface NdiConsumerFormProps {
  consumer: NdiConsumer;
  onUpdate: (consumer: NdiConsumer) => void;
}

function NdiConsumerForm({ consumer, onUpdate }: NdiConsumerFormProps) {
  return (
    <div className="grid grid-cols-2 gap-3 text-sm">
      <div className="col-span-2">
        <label className="block text-[var(--color-text-muted)] mb-1">NDI Name</label>
        <input
          type="text"
          value={consumer.name}
          onChange={(e) => onUpdate({ ...consumer, name: e.target.value })}
          className="w-full text-sm"
          placeholder="CasparCG"
        />
      </div>

      <div className="col-span-2">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={consumer.allow_fields}
            onChange={(e) => onUpdate({ ...consumer, allow_fields: e.target.checked })}
          />
          <span className="text-[var(--color-text-muted)]">Allow Fields (interlaced output)</span>
        </label>
      </div>
    </div>
  );
}

interface ScreenConsumerFormProps {
  consumer: ScreenConsumer;
  onUpdate: (consumer: ScreenConsumer) => void;
}

function ScreenConsumerForm({ consumer, onUpdate }: ScreenConsumerFormProps) {
  return (
    <div className="grid grid-cols-2 gap-3 text-sm">
      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Screen Device</label>
        <input
          type="number"
          value={consumer.device}
          onChange={(e) => onUpdate({ ...consumer, device: parseInt(e.target.value, 10) || 1 })}
          className="w-full text-sm"
          min={1}
        />
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Window Name</label>
        <input
          type="text"
          value={consumer.name || ''}
          onChange={(e) => onUpdate({ ...consumer, name: e.target.value || undefined })}
          className="w-full text-sm"
          placeholder="Optional"
        />
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Width</label>
        <input
          type="number"
          value={consumer.width || ''}
          onChange={(e) => onUpdate({ ...consumer, width: parseInt(e.target.value, 10) || undefined })}
          className="w-full text-sm"
          placeholder="Auto"
        />
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Height</label>
        <input
          type="number"
          value={consumer.height || ''}
          onChange={(e) => onUpdate({ ...consumer, height: parseInt(e.target.value, 10) || undefined })}
          className="w-full text-sm"
          placeholder="Auto"
        />
      </div>

      <div className="col-span-2 flex flex-wrap items-center gap-4">
        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={consumer.windowed}
            onChange={(e) => onUpdate({ ...consumer, windowed: e.target.checked })}
          />
          <span className="text-[var(--color-text-muted)]">Windowed</span>
        </label>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={consumer.borderless}
            onChange={(e) => onUpdate({ ...consumer, borderless: e.target.checked })}
          />
          <span className="text-[var(--color-text-muted)]">Borderless</span>
        </label>

        <label className="flex items-center gap-2 cursor-pointer">
          <input
            type="checkbox"
            checked={consumer.always_on_top}
            onChange={(e) => onUpdate({ ...consumer, always_on_top: e.target.checked })}
          />
          <span className="text-[var(--color-text-muted)]">Always on Top</span>
        </label>
      </div>
    </div>
  );
}
