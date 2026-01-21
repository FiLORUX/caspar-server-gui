// Channels configuration panel
// Configure video channels and their consumers

import { useState } from 'react';
import { useAppStore } from '../lib/store';
import type {
  Channel,
  Consumer,
  GlobalConfig,
  VideoMode,
  DeckLinkConsumer,
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
    updateChannels([...channels, { ...DEFAULT_CHANNEL }]);
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
            key={index}
            channel={channel}
            index={index}
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
  onUpdate: (channel: Channel) => void;
  onRemove: () => void;
  canRemove: boolean;
  deckLinkDevices: { index: number; display_name: string }[];
  isTesting: boolean;
  canTest: boolean;
  onToggleTest: () => void;
}

function ChannelCard({
  channel,
  index,
  onUpdate,
  onRemove,
  canRemove,
  deckLinkDevices,
  isTesting,
  canTest,
  onToggleTest,
}: ChannelCardProps) {
  const [isExpanded, setIsExpanded] = useState(true);

  const addConsumer = (type: Consumer['type']) => {
    let newConsumer: Consumer;
    switch (type) {
      case 'decklink':
        newConsumer = createDefaultDeckLinkConsumer();
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

          {channel.consumers.length === 0 ? (
            <div className="text-sm text-[var(--color-text-muted)] text-center py-4">
              No consumers configured. Add a consumer to output video.
            </div>
          ) : (
            <div className="space-y-3">
              {channel.consumers.map((consumer, ci) => (
                <ConsumerCard
                  key={ci}
                  consumer={consumer}
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
  onUpdate: (consumer: Consumer) => void;
  onRemove: () => void;
  deckLinkDevices: { index: number; display_name: string }[];
}

function ConsumerCard({ consumer, onUpdate, onRemove, deckLinkDevices }: ConsumerCardProps) {
  const typeLabels: Record<Consumer['type'], string> = {
    decklink: 'DeckLink',
    ndi: 'NDI',
    screen: 'Screen',
    'system-audio': 'System Audio',
  };

  return (
    <div className="consumer-card">
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
    </div>
  );
}

interface DeckLinkConsumerFormProps {
  consumer: DeckLinkConsumer;
  onUpdate: (consumer: DeckLinkConsumer) => void;
  devices: { index: number; display_name: string }[];
}

function DeckLinkConsumerForm({ consumer, onUpdate, devices }: DeckLinkConsumerFormProps) {
  return (
    <div className="grid grid-cols-2 gap-3 text-sm">
      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Device</label>
        <select
          value={consumer.device}
          onChange={(e) => onUpdate({ ...consumer, device: parseInt(e.target.value, 10) })}
          className="w-full text-sm"
        >
          {devices.length > 0 ? (
            devices.map((d) => (
              <option key={d.index} value={d.index}>
                {d.display_name}
              </option>
            ))
          ) : (
            <option value={consumer.device}>Device {consumer.device}</option>
          )}
        </select>
      </div>

      <div>
        <label className="block text-[var(--color-text-muted)] mb-1">Key Device</label>
        <select
          value={consumer.key_device ?? ''}
          onChange={(e) =>
            onUpdate({
              ...consumer,
              key_device: e.target.value ? parseInt(e.target.value, 10) : undefined,
            })
          }
          className="w-full text-sm"
        >
          <option value="">None</option>
          {devices.map((d) => (
            <option key={d.index} value={d.index}>
              {d.display_name}
            </option>
          ))}
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
          onChange={(e) => onUpdate({ ...consumer, keyer: e.target.value as DeckLinkConsumer['keyer'] })}
          className="w-full text-sm"
        >
          <option value="external">External</option>
          <option value="external_separate_device">External (Separate Device)</option>
          <option value="internal">Internal</option>
          <option value="default">Default</option>
        </select>
      </div>

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
