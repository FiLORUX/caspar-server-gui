// Server panel — the operate view.
// Launches casparcg.exe with the active profile, streams its console log into
// the GUI (no separate window), auto-connects AMCP, and offers Start/Stop/Restart
// — the embedded-log model of the classic CasparCG launcher.

import { useEffect, useRef, useState } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';
import { validateConfig, errorsOnly } from '../lib/validation';

// CasparCG logs "Failed to enable external/internal keyer" at error level on any
// DeckLink card that has no keyer hardware (e.g. the SDI Micro). It is benign:
// the consumer still initialises and outputs fill, and 2.5.0 offers no keyer
// value that avoids it. Recognise it so the embedded log notes it rather than
// raising a false alarm.
const BENIGN_KEYER_LOG = /failed to enable (external|internal) keyer/i;

export function ServerPanel() {
  const { currentConfig, connection, connect, deckLinkDevices } = useAppStore();
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
        // If the process is gone, the AMCP connection cannot be live — clear any
        // stale "connected (VERSION OK)" so the indicator tells the truth.
        if (!r) {
          const st = useAppStore.getState();
          if (st.connection.connected) {
            st.disconnect();
          }
        }
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

  // Poll AMCP until the freshly launched server answers, then connect — on the
  // port the active profile actually configures, not a hard-coded default.
  const autoConnect = async () => {
    const port =
      useAppStore.getState().currentConfig?.caspar.controllers.tcp.port ?? 5250;
    for (let i = 0; i < 20; i++) {
      try {
        await connect('localhost', port);
        return;
      } catch {
        await new Promise((r) => setTimeout(r, 750));
      }
    }
  };

  // The whole profile is validated up front; launch is refused while any hard
  // error remains, so CasparCG is never handed a config it would reject.
  const configErrors = currentConfig ? errorsOnly(validateConfig(currentConfig, deckLinkDevices)) : [];
  const canStart = !!currentConfig && configErrors.length === 0;

  const start = async () => {
    setError(null);
    // Re-derive from live store state rather than the render-time snapshot, so a
    // launch (including via restart) can never use stale validation results.
    const cfg = useAppStore.getState().currentConfig;
    const devices = useAppStore.getState().deckLinkDevices;
    if (!cfg) {
      setError('Select a profile first');
      return;
    }
    if (errorsOnly(validateConfig(cfg, devices)).length > 0) {
      setError('Resolve the configuration errors below before starting');
      return;
    }
    setLog([]);
    try {
      await tauri.startCasparServer(cfg);
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
              disabled={!canStart}
              title={
                !currentConfig
                  ? 'Select a profile first'
                  : configErrors.length > 0
                    ? 'Resolve configuration errors before starting'
                    : 'Write config and launch casparcg.exe'
              }
              className="px-4 py-1.5 rounded bg-emerald-600 hover:bg-emerald-700 text-white disabled:opacity-50 disabled:cursor-not-allowed"
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

      {!running && configErrors.length > 0 && (
        <div className="mb-3 p-3 rounded bg-red-500/10 border border-red-500/30 text-sm">
          <div className="font-medium text-red-400 mb-1">
            Configuration {configErrors.length === 1 ? 'error' : 'errors'} — fix in the Channels tab before starting:
          </div>
          <ul className="space-y-1">
            {configErrors.map((issue) => (
              <li key={issue.id} className="text-red-300 flex items-start gap-1.5">
                <span aria-hidden>✕</span>
                <span>
                  {issue.channelIndex !== undefined ? `Channel ${issue.channelIndex + 1}: ` : ''}
                  {issue.message}
                </span>
              </li>
            ))}
          </ul>
        </div>
      )}

      {/* Embedded live server log */}
      <div
        ref={logRef}
        className="flex-1 min-h-0 overflow-auto rounded bg-black/60 border border-[var(--color-border)] p-3 font-mono text-xs leading-relaxed"
      >
        {log.length === 0 ? (
          <div className="text-[var(--color-text-muted)]">
            {running
              ? 'Server running — no new console output. (CasparCG logs at startup, then goes quiet when idle.)'
              : 'No output yet — press Start Server. The CasparCG console log appears here.'}
          </div>
        ) : (
          log.map((line, i) => {
            const lower = line.toLowerCase();
            const benignKeyer = BENIGN_KEYER_LOG.test(line);
            const cls = benignKeyer
              ? 'text-amber-400/80'
              : lower.includes('error') || lower.includes('fatal')
                ? 'text-red-400'
                : lower.includes('warning')
                  ? 'text-amber-400'
                  : 'text-[var(--color-text-secondary)]';
            return (
              <div key={i} className={cls}>
                {line}
                {benignKeyer && (
                  <span className="text-[var(--color-text-muted)]">
                    {'  '}— expected on a card with no hardware keyer; fill output is unaffected
                  </span>
                )}
              </div>
            );
          })
        )}
      </div>
    </div>
  );
}
