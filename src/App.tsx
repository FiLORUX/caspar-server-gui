import { useEffect, useState } from 'react';
import { useAppStore } from './lib/store';
import { ProfileSidebar } from './components/ProfileSidebar';
import { TabBar } from './components/TabBar';
import { PathsPanel } from './components/PathsPanel';
import { ChannelsPanel } from './components/ChannelsPanel';
import { DeckLinkPanel } from './components/DeckLinkPanel';
import { SystemInfoPanel } from './components/SystemInfoPanel';
import { StatusBar } from './components/StatusBar';
import { SetupWizard } from './components/SetupWizard';

function App() {
  const { activeTab, settings, initialise } = useAppStore();
  const [isLoading, setIsLoading] = useState(true);

  useEffect(() => {
    initialise().finally(() => setIsLoading(false));
  }, [initialise]);

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-screen bg-[var(--color-bg-primary)]">
        <div className="text-center">
          <div className="w-12 h-12 border-4 border-[var(--color-accent)] border-t-transparent rounded-full animate-spin mx-auto mb-4" />
          <p className="text-[var(--color-text-secondary)]">Loading...</p>
        </div>
      </div>
    );
  }

  // Show setup wizard if CasparCG path not configured
  if (!settings?.caspar_path) {
    return <SetupWizard />;
  }

  return (
    <div className="flex flex-col h-screen bg-[var(--color-bg-primary)]">
      {/* Main content area */}
      <div className="flex flex-1 overflow-hidden">
        {/* Profile sidebar */}
        <ProfileSidebar />

        {/* Main panel area */}
        <div className="flex-1 flex flex-col overflow-hidden">
          {/* Tab bar */}
          <TabBar />

          {/* Active panel content */}
          <div className="flex-1 overflow-auto p-4">
            {activeTab === 'paths' && <PathsPanel />}
            {activeTab === 'channels' && <ChannelsPanel />}
            {activeTab === 'decklink' && <DeckLinkPanel />}
            {activeTab === 'system' && <SystemInfoPanel />}
          </div>
        </div>
      </div>

      {/* Status bar */}
      <StatusBar />
    </div>
  );
}

export default App;
