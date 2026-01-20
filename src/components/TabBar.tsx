// Tab bar component
// Navigation between different configuration panels

import { useAppStore } from '../lib/store';
import type { TabId } from '../lib/types';

const TABS: { id: TabId; label: string }[] = [
  { id: 'paths', label: 'Paths' },
  { id: 'channels', label: 'Channels' },
  { id: 'decklink', label: 'DeckLink' },
  { id: 'system', label: 'System Info' },
];

export function TabBar() {
  const { activeTab, setActiveTab, currentConfig } = useAppStore();

  return (
    <div className="flex border-b border-[var(--color-border)] bg-[var(--color-bg-secondary)]">
      {TABS.map((tab) => (
        <button
          key={tab.id}
          className={`tab-button ${activeTab === tab.id ? 'active' : ''}`}
          onClick={() => setActiveTab(tab.id)}
          disabled={!currentConfig && tab.id !== 'system'}
        >
          {tab.label}
        </button>
      ))}

      {/* Spacer */}
      <div className="flex-1" />

      {/* Profile name display */}
      {currentConfig && (
        <div className="flex items-center px-4 text-sm text-[var(--color-text-secondary)]">
          <span className="text-[var(--color-text-muted)] mr-2">Profile:</span>
          <span className="font-medium">{currentConfig.name}</span>
        </div>
      )}
    </div>
  );
}
