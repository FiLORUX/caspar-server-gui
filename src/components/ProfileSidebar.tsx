// Profile sidebar component
// Lists and manages configuration profiles

import { useState } from 'react';
import { useAppStore } from '../lib/store';

export function ProfileSidebar() {
  const {
    profiles,
    activeProfile,
    configDirty,
    selectProfile,
    saveProfile,
    createProfile,
  } = useAppStore();
  const [isCreating, setIsCreating] = useState(false);
  const [newProfileName, setNewProfileName] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleSelectProfile = async (name: string) => {
    if (configDirty) {
      // TODO: Show confirmation dialog
      const confirm = window.confirm(
        'You have unsaved changes. Do you want to save before switching profiles?'
      );
      if (confirm) {
        await saveProfile();
      }
    }
    await selectProfile(name);
  };

  const handleCreateProfile = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!newProfileName.trim()) {
      setError('Please enter a profile name');
      return;
    }

    if (profiles.includes(newProfileName.trim())) {
      setError('A profile with this name already exists');
      return;
    }

    try {
      await createProfile(newProfileName.trim());
      setNewProfileName('');
      setIsCreating(false);
      setError(null);
    } catch (err) {
      setError(String(err));
    }
  };

  return (
    <div className="w-56 bg-[var(--color-bg-secondary)] border-r border-[var(--color-border)] flex flex-col">
      {/* Header */}
      <div className="p-3 border-b border-[var(--color-border)]">
        <h2 className="text-xs font-semibold text-[var(--color-text-secondary)] uppercase tracking-wider">
          Profiles
        </h2>
      </div>

      {/* Profile list */}
      <div className="flex-1 overflow-auto">
        {profiles.length === 0 ? (
          <div className="p-4 text-sm text-[var(--color-text-muted)] text-center">
            No profiles yet
          </div>
        ) : (
          <div className="py-1">
            {profiles.map((name) => (
              <div
                key={name}
                className={`profile-item flex items-center justify-between ${
                  activeProfile === name ? 'active' : ''
                }`}
                onClick={() => handleSelectProfile(name)}
              >
                <span className="truncate">{name}</span>
                {activeProfile === name && configDirty && (
                  <span className="w-2 h-2 rounded-full bg-[var(--color-warning)]" title="Unsaved changes" />
                )}
              </div>
            ))}
          </div>
        )}
      </div>

      {/* Actions */}
      <div className="p-3 border-t border-[var(--color-border)]">
        {isCreating ? (
          <form onSubmit={handleCreateProfile}>
            <input
              type="text"
              value={newProfileName}
              onChange={(e) => setNewProfileName(e.target.value)}
              placeholder="Profile name..."
              className="w-full mb-2 text-sm"
              autoFocus
            />
            {error && (
              <p className="text-xs text-red-400 mb-2">{error}</p>
            )}
            <div className="flex gap-2">
              <button
                type="submit"
                className="flex-1 px-2 py-1 text-xs bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)]"
              >
                Create
              </button>
              <button
                type="button"
                onClick={() => {
                  setIsCreating(false);
                  setNewProfileName('');
                  setError(null);
                }}
                className="flex-1 px-2 py-1 text-xs bg-[var(--color-bg-tertiary)] text-[var(--color-text-secondary)] rounded hover:bg-[var(--color-border)]"
              >
                Cancel
              </button>
            </div>
          </form>
        ) : (
          <div className="flex flex-col gap-2">
            <button
              onClick={() => setIsCreating(true)}
              className="w-full px-3 py-2 text-sm bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] rounded hover:bg-[var(--color-border)] transition-colors"
            >
              + New Profile
            </button>
            {activeProfile && configDirty && (
              <button
                onClick={() => saveProfile()}
                className="w-full px-3 py-2 text-sm bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)] transition-colors"
              >
                Save Changes
              </button>
            )}
          </div>
        )}
      </div>
    </div>
  );
}
