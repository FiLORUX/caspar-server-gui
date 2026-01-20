// Setup wizard for first-time configuration
// Prompts user to select CasparCG installation path

import { useState } from 'react';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';

export function SetupWizard() {
  const { saveSettings, loadProfiles, createProfile } = useAppStore();
  const [path, setPath] = useState('');
  const [error, setError] = useState<string | null>(null);
  const [isSubmitting, setIsSubmitting] = useState(false);

  const handleBrowse = async () => {
    const selected = await tauri.pickFolder();
    if (selected) {
      setPath(selected);
      setError(null);
    }
  };

  const handleSubmit = async (e: React.FormEvent) => {
    e.preventDefault();
    if (!path) {
      setError('Please select the CasparCG installation folder');
      return;
    }

    setIsSubmitting(true);
    setError(null);

    try {
      await tauri.setCasparPath(path);
      await saveSettings({ caspar_path: path });
      await loadProfiles();

      // Create a default profile
      await createProfile('Default');
    } catch (err) {
      setError(String(err));
    } finally {
      setIsSubmitting(false);
    }
  };

  return (
    <div className="flex items-center justify-center min-h-screen bg-[var(--color-bg-primary)] p-8">
      <div className="w-full max-w-lg">
        <div className="panel">
          <div className="panel-header">
            Welcome to CasparCG Server GUI
          </div>
          <div className="panel-content">
            <p className="text-[var(--color-text-secondary)] mb-6">
              To get started, please select your CasparCG Server installation folder.
              This is the folder containing <code className="text-[var(--color-accent)]">casparcg.exe</code>.
            </p>

            <form onSubmit={handleSubmit}>
              <div className="mb-4">
                <label className="block text-sm font-medium text-[var(--color-text-secondary)] mb-2">
                  CasparCG Installation Path
                </label>
                <div className="flex gap-2">
                  <input
                    type="text"
                    value={path}
                    onChange={(e) => setPath(e.target.value)}
                    placeholder="C:\CasparCG"
                    className="flex-1"
                    readOnly
                  />
                  <button
                    type="button"
                    onClick={handleBrowse}
                    className="px-4 py-2 bg-[var(--color-bg-tertiary)] text-[var(--color-text-primary)] rounded hover:bg-[var(--color-border)] transition-colors"
                  >
                    Browse...
                  </button>
                </div>
              </div>

              {error && (
                <div className="mb-4 p-3 bg-red-900/30 border border-red-500 rounded text-red-300 text-sm">
                  {error}
                </div>
              )}

              <div className="flex justify-end">
                <button
                  type="submit"
                  disabled={!path || isSubmitting}
                  className="px-6 py-2 bg-[var(--color-accent)] text-white rounded hover:bg-[var(--color-accent-hover)] transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  {isSubmitting ? 'Setting up...' : 'Continue'}
                </button>
              </div>
            </form>

            <div className="mt-6 pt-6 border-t border-[var(--color-border)]">
              <h3 className="text-sm font-medium text-[var(--color-text-secondary)] mb-2">
                What happens next?
              </h3>
              <ul className="text-sm text-[var(--color-text-muted)] space-y-1">
                <li>• A profiles folder will be created alongside CasparCG</li>
                <li>• You can create and manage multiple configuration profiles</li>
                <li>• Export configurations to standard casparcg.config format</li>
              </ul>
            </div>
          </div>
        </div>
      </div>
    </div>
  );
}
