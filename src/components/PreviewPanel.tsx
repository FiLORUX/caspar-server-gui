// Key/Fill preview panel.
// Renders the identifier test pattern in the app's own webview via the built-in
// test HTTP server — the same page CasparCG would play to SDI, but visible
// in-app without an SDI monitor and independent of CasparCG's HTML/CEF module.

import { useEffect, useState } from 'react';
import { useAppStore } from '../lib/store';

type PreviewMode = 'preview' | 'fill' | 'key';

export function PreviewPanel() {
  const { currentConfig, testServerUrl, testServerRunning, startTestServer } = useAppStore();
  const [channel, setChannel] = useState(1);
  const [mode, setMode] = useState<PreviewMode>('preview');
  const [error, setError] = useState<string | null>(null);

  const channelCount = currentConfig?.caspar.channels.length ?? 1;

  // The preview only needs the local test pattern server — not CasparCG.
  useEffect(() => {
    if (!testServerRunning) {
      startTestServer().catch((e) => setError(String(e)));
    }
  }, [testServerRunning, startTestServer]);

  const src = testServerUrl
    ? `${testServerUrl}/index.html?mode=${mode}&id=${channel}`
    : null;

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-3">
        <div>
          <h2 className="text-lg font-semibold">Key / Fill Preview</h2>
          <p className="text-sm text-[var(--color-text-secondary)]">
            The identifier pattern rendered in-app — the same page CasparCG plays to SDI,
            no monitor needed and independent of the server
          </p>
        </div>
        <div className="flex items-center gap-2">
          <select
            value={mode}
            onChange={(e) => setMode(e.target.value as PreviewMode)}
            className="text-sm"
          >
            <option value="preview">Preview (fill + key)</option>
            <option value="fill">Fill</option>
            <option value="key">Key</option>
          </select>
          <select
            value={channel}
            onChange={(e) => setChannel(parseInt(e.target.value, 10))}
            className="text-sm"
          >
            {Array.from({ length: channelCount }, (_, i) => i + 1).map((c) => (
              <option key={c} value={c}>
                Channel {c}
              </option>
            ))}
          </select>
        </div>
      </div>

      {error && (
        <div className="mb-3 p-2 rounded bg-red-500/15 text-red-400 text-sm">{error}</div>
      )}

      <div className="flex-1 min-h-0 rounded border border-[var(--color-border)] overflow-hidden bg-black">
        {src ? (
          <iframe
            key={src}
            src={src}
            title="Key/Fill preview"
            className="w-full h-full border-0"
          />
        ) : (
          <div className="flex items-center justify-center h-full text-sm text-[var(--color-text-muted)]">
            Starting preview server…
          </div>
        )}
      </div>

      <p className="mt-2 text-xs text-[var(--color-text-muted)]">
        To push this to SDI, start the server and use Test on the Channels tab. The SDI
        output also needs a free DeckLink output (unplug any input feeding a single-connector card).
      </p>
    </div>
  );
}
