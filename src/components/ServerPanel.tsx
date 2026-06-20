// Server panel — the operate view.
// Launches casparcg.exe with the active profile, streams its console log into
// the GUI (no separate window), auto-connects AMCP, and offers Start/Stop/Restart
// — the embedded-log model of the classic CasparCG launcher.

import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';

export function ServerPanel() {
  const { currentConfig, connection, connect } = useAppStore();
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [log, setLog] = useState<string[]>([]);
  const logRef = useRef<HTMLDivElement>(null);

  // Subscribe to the live server log emitted by the backend.
  useEffect(() => {
    const unlisten = listen<string>('caspar-log', (event) => {
      setLog((prev) => {
        const next = [...prev, event.payload];
        return next.length > 2000 ? next.slice(-2000) : next;
      });
    });
    return () => {
      unlisten.then((u) => u());
    };
  }, []);

  // Keep the log scrolled to the bottom.
  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [log]);

  // Reflect the live process state.
  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      try {
        const r = await tauri.casparServerRunning();
        if (!cancelled) setRunning(r);
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

  // Poll AMCP until the freshly launched server answers, then connect.
  const autoConnect = async () => {
    for (let i = 0; i < 20; i++) {
      try {
        await connect('localhost', 5250);
        return;
      } catch {
        await new Promise((r) => setTimeout(r, 750));
      }
    }
  };

  const start = async () => {
    setError(null);
    if (!currentConfig) {
      setError('Select a profile first');
      return;
    }
    setLog([]);
    try {
      await tauri.startCasparServer(currentConfig);
      setRunning(true);
      autoConnect();
    } catch (e) {
      setError(String(e));
    }
  };

  const stop = async () => {
    try {
      await tauri.stopCasparServer();
    } catch {
      /* ignore */
    }
    setRunning(false);
  };

  const restart = async () => {
    await stop();
    await new Promise((r) => setTimeout(r, 600));
    await start();
  };

  return (
    <div className="flex flex-col h-full">
      <div className="flex items-center justify-between mb-3">
        <div>
          <h2 className="text-lg font-semibold">Server</h2>
          <p className="text-sm text-[var(--color-text-secondary)]">
            Launch casparcg.exe with the active profile and watch its live log
          </p>
        </div>
        <div className="flex items-center gap-2">
          {running ? (
            <>
              <button
                onClick={restart}
                className="px-3 py-1.5 rounded bg-[var(--color-bg-tertiary)] hover:bg-[var(--color-border)]"
              >
                ↻ Restart
              </button>
              <button
                onClick={stop}
                className="px-3 py-1.5 rounded bg-amber-600 hover:bg-amber-700 text-white"
              >
                ■ Stop
              </button>
            </>
          ) : (
            <button
              onClick={start}
              disabled={!currentConfig}
              title={!currentConfig ? 'Select a profile first' : 'Write config and launch casparcg.exe'}
              className="px-4 py-1.5 rounded bg-emerald-600 hover:bg-emerald-700 text-white disabled:opacity-50"
            >
              ▶ Start Server
            </button>
          )}
        </div>
      </div>

      <div className="flex items-center gap-5 mb-3 text-sm">
        <span className="flex items-center gap-2">
          <span className={`status-dot ${running ? 'connected' : 'disconnected'}`} />
          {running ? 'Server running' : 'Stopped'}
        </span>
        <span className="flex items-center gap-2">
          <span className={`status-dot ${connection.connected ? 'connected' : 'disconnected'}`} />
          {connection.connected
            ? `AMCP connected${connection.version ? ` (${connection.version})` : ''}`
            : 'AMCP not connected'}
        </span>
      </div>

      {error && (
        <div className="mb-3 p-2 rounded bg-red-500/15 text-red-400 text-sm">{error}</div>
      )}

      {/* Embedded live server log */}
      <div
        ref={logRef}
        className="flex-1 min-h-0 overflow-auto rounded bg-black/60 border border-[var(--color-border)] p-3 font-mono text-xs leading-relaxed"
      >
        {log.length === 0 ? (
          <div className="text-[var(--color-text-muted)]">
            No output yet — press Start Server. The CasparCG console log appears here.
          </div>
        ) : (
          log.map((line, i) => {
            const lower = line.toLowerCase();
            const cls = lower.includes('error') || lower.includes('fatal')
              ? 'text-red-400'
              : lower.includes('warning')
                ? 'text-amber-400'
                : 'text-[var(--color-text-secondary)]';
            return (
              <div key={i} className={cls}>
                {line}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
