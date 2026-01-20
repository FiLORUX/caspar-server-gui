// Paths configuration panel
// Configure media, template, log, and data paths

import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';
import type { GlobalConfig, Paths } from '../lib/types';

export function PathsPanel() {
  const { currentConfig, updateConfig } = useAppStore();

  if (!currentConfig) {
    return (
      <div className="flex items-center justify-center h-full text-[var(--color-text-muted)]">
        Select or create a profile to configure paths
      </div>
    );
  }

  const paths = currentConfig.caspar.paths;

  const updatePath = (key: keyof Paths, value: string) => {
    const newConfig: GlobalConfig = {
      ...currentConfig,
      caspar: {
        ...currentConfig.caspar,
        paths: {
          ...currentConfig.caspar.paths,
          [key]: value,
        },
      },
    };
    updateConfig(newConfig);
  };

  const handleBrowse = async (key: keyof Paths) => {
    const selected = await tauri.pickFolder();
    if (selected) {
      updatePath(key, selected);
    }
  };

  return (
    <div className="max-w-3xl">
      <h2 className="text-lg font-semibold mb-4">Path Configuration</h2>
      <p className="text-[var(--color-text-secondary)] mb-6">
        Configure the paths that CasparCG Server uses for media, templates, logs, and data files.
      </p>

      <div className="space-y-4">
        <PathInput
          label="Media Path"
          description="Location of video and image media files"
          value={paths.media}
          onChange={(v) => updatePath('media', v)}
          onBrowse={() => handleBrowse('media')}
        />

        <PathInput
          label="Template Path"
          description="Location of HTML and Flash templates"
          value={paths.template}
          onChange={(v) => updatePath('template', v)}
          onBrowse={() => handleBrowse('template')}
        />

        <PathInput
          label="Log Path"
          description="Where server logs are written"
          value={paths.log}
          onChange={(v) => updatePath('log', v)}
          onBrowse={() => handleBrowse('log')}
        />

        <PathInput
          label="Data Path"
          description="Location for data files and datasets"
          value={paths.data}
          onChange={(v) => updatePath('data', v)}
          onBrowse={() => handleBrowse('data')}
        />

        <PathInput
          label="Font Path"
          description="Custom font directory (optional)"
          value={paths.font || ''}
          onChange={(v) => updatePath('font', v)}
          onBrowse={() => handleBrowse('font')}
          optional
        />
      </div>

      {/* Controller settings */}
      <div className="mt-8 pt-8 border-t border-[var(--color-border)]">
        <h3 className="text-md font-semibold mb-4">Controller Settings</h3>

        <div className="grid grid-cols-2 gap-4 max-w-md">
          <div>
            <label className="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">
              TCP Port
            </label>
            <input
              type="number"
              value={currentConfig.caspar.controllers.tcp.port}
              onChange={(e) => {
                const newConfig: GlobalConfig = {
                  ...currentConfig,
                  caspar: {
                    ...currentConfig.caspar,
                    controllers: {
                      ...currentConfig.caspar.controllers,
                      tcp: {
                        ...currentConfig.caspar.controllers.tcp,
                        port: parseInt(e.target.value, 10) || 5250,
                      },
                    },
                  },
                };
                updateConfig(newConfig);
              }}
              className="w-full"
              min={1}
              max={65535}
            />
          </div>

          <div>
            <label className="block text-sm font-medium text-[var(--color-text-secondary)] mb-1">
              Protocol
            </label>
            <select
              value={currentConfig.caspar.controllers.tcp.protocol}
              onChange={(e) => {
                const newConfig: GlobalConfig = {
                  ...currentConfig,
                  caspar: {
                    ...currentConfig.caspar,
                    controllers: {
                      ...currentConfig.caspar.controllers,
                      tcp: {
                        ...currentConfig.caspar.controllers.tcp,
                        protocol: e.target.value,
                      },
                    },
                  },
                };
                updateConfig(newConfig);
              }}
              className="w-full"
            >
              <option value="AMCP">AMCP</option>
            </select>
          </div>
        </div>
      </div>
    </div>
  );
}

interface PathInputProps {
  label: string;
  description: string;
  value: string;
  onChange: (value: string) => void;
  onBrowse: () => void;
  optional?: boolean;
}

function PathInput({
  label,
  description,
  value,
  onChange,
  onBrowse,
  optional,
}: PathInputProps) {
  return (
    <div className="panel">
      <div className="p-4">
        <div className="flex items-start justify-between mb-2">
          <div>
            <label className="block text-sm font-medium text-[var(--color-text-primary)]">
              {label}
              {optional && (
                <span className="ml-2 text-xs text-[var(--color-text-muted)]">(optional)</span>
              )}
            </label>
            <p className="text-xs text-[var(--color-text-muted)]">{description}</p>
          </div>
        </div>

        <div className="flex gap-2">
          <input
            type="text"
            value={value}
            onChange={(e) => onChange(e.target.value)}
            className="flex-1"
            placeholder={optional ? 'Not set' : 'Select a folder...'}
          />
          <button
            type="button"
            onClick={onBrowse}
            className="px-4 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] rounded hover:bg-[var(--color-border)] transition-colors"
          >
            Browse...
          </button>
        </div>
      </div>
    </div>
  );
}
