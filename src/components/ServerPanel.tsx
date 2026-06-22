// Server panel — the operate view.
// Launches casparcg.exe with the active profile, streams its console log into
// the GUI (no separate window), auto-connects AMCP, and offers Start/Stop/Restart
// — the embedded-log model of the classic CasparCG launcher.

import { useEffect, useRef, useState } from 'react';
import { useAppStore } from '../lib/store';
import * as tauri from '../lib/tauri';
import { validateConfig, errorsOnly } from '../lib/validation';

// CasparCG logs "Failed to enable external/internal keyer" at error level on any
// DeckLink card that has no keyer hardware (e.g. the SDI Micro). It is benign:
// the consumer still initialises and outputs fill, and 2.5.0 offers no keyer
// value that avoids it. Recognise it so the embedded log notes it rather than
// raising a false alarm.
const BENIGN_KEYER_LOG = /failed to enable (external|internal) keyer/i;

// Deterministic log colouring. Rather than substring-matching "error" anywhere
// in a line — which mis-colours an info line that merely mentions an error — we
// key off the structured level token each source actually emits: CasparCG's
// "[timestamp] [level] …", the scanner's pino "level": <number>, and our own
// "[launcher]" prefix. Same line in, same colour out, every time.
function classifyLogLine(line: string): { cls: string; keyerNote?: boolean } {
  // The launcher's own orchestration lines — distinct from server output.
  if (line.startsWith('[launcher]')) {
    const lower = line.toLowerCase();
    const bad =
      lower.includes('giving up') || lower.includes('failed') || lower.includes('crashed');
    return { cls: bad ? 'text-red-400' : 'text-cyan-400' };
  }
  // The no-keyer "error" CasparCG logs on cards without keyer hardware is benign.
  if (BENIGN_KEYER_LOG.test(line)) {
    return { cls: 'text-amber-400/80', keyerNote: true };
  }
  // Scanner lines are pino JSON: { "level": <number> }. 50+ = error/fatal,
  // 40 = warning, below = info/debug.
  if (line.startsWith('[scanner]')) {
    const m = line.match(/"level":\s*(\d+)/);
    if (m) {
      const level = parseInt(m[1], 10);
      if (level >= 50) return { cls: 'text-red-400' };
      if (level >= 40) return { cls: 'text-amber-400' };
    }
    return { cls: 'text-[var(--color-text-muted)]' };
  }
  // CasparCG server lines — colour by the level token, not the message body.
  const m = line.match(/\]\s*\[(trace|debug|info|warning|error|fatal)\]/i);
  if (m) {
    switch (m[1].toLowerCase()) {
      case 'fatal':
      case 'error':
        return { cls: 'text-red-400' };
      case 'warning':
        return { cls: 'text-amber-400' };
      case 'trace':
      case 'debug':
        return { cls: 'text-[var(--color-text-muted)]' };
      default:
        return { cls: 'text-[var(--color-text-secondary)]' };
    }
  }
  return { cls: 'text-[var(--color-text-secondary)]' };
}

export function ServerPanel() {
  const {
    currentConfig,
    connection,
    connect,
    deckLinkDevices,
    serverLog,
    clearServerLog,
    scannerEndpoint,
  } = useAppStore();
  const [running, setRunning] = useState(false);
  const [error, setError] = useState<string | null>(null);
  // This host's primary IPv4 — what the operator points a remote client at.
  const [primaryIp, setPrimaryIp] = useState<string | null>(null);
  const logRef = useRef<HTMLDivElement>(null);
  // Tracks the previous poll's running state so we can detect the moment the
  // server comes up (first Start or a supervised restart) and reconnect AMCP.
  const prevRunning = useRef(false);

  // The log is captured app-level into the store (so it survives tab switches and
  // a crash); just keep it scrolled to the bottom here.
  useEffect(() => {
    if (logRef.current) {
      logRef.current.scrollTop = logRef.current.scrollHeight;
    }
  }, [serverLog]);

  // Resolve the host's primary IP once, so the panel can show the operator the
  // exact endpoint to connect a remote client to.
  useEffect(() => {
    tauri.getPrimaryIp().then(setPrimaryIp).catch(() => setPrimaryIp(null));
  }, []);

  // Reflect the live process state, and keep the AMCP link in step with it.
  useEffect(() => {
    let cancelled = false;
    const poll = async () => {
      try {
        const r = await tauri.casparServerRunning();
        if (cancelled) return;
        setRunning(r);
        const st = useAppStore.getState();
        if (!r) {
          // Process gone (Stop, crash, or mid-restart) — a live AMCP link cannot
          // exist, so clear any stale "connected (VERSION OK)". The scanner
          // endpoint is event-driven (cleared by the launcher on stop), so it is
          // deliberately left alone here to survive a server-only restart.
          if (st.connection.connected) {
            st.disconnect();
          }
        } else if (!prevRunning.current && !st.connection.connected) {
          // Server just came up — first Start or a supervised restart — so
          // re-establish AMCP without the operator touching anything.
          autoConnect();
        }
        prevRunning.current = r;
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
    clearServerLog();
    try {
      await tauri.startCasparServer(cfg);
      setRunning(true);
      // We drive the connect here, so mark this rising edge as handled — the
      // poll's reconnect detector should only fire for a supervised restart
      // (where start() is not called), not for this manual Start/Restart.
      prevRunning.current = true;
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

      {/* Connection endpoints. A remote operator's client connects over AMCP to
          this server's address on the configured port — that is the figure to
          relay. The media scanner is loopback-internal (the server proxies
          CLS/TLS/THUMBNAIL to it); its port is shown only so a non-stock value
          is visible when diagnosing media browsing. */}
      <div className="flex flex-wrap items-center gap-x-6 gap-y-1 mb-3 text-xs text-[var(--color-text-secondary)]">
        <span>
          {primaryIp ? (
            <>
              Client →{' '}
              <span className="font-mono text-[var(--color-text-primary)]">
                {primaryIp}:{currentConfig?.caspar.controllers.tcp.port ?? 5250}
              </span>{' '}
              (AMCP)
            </>
          ) : (
            <>
              Client → AMCP port{' '}
              <span className="font-mono text-[var(--color-text-primary)]">
                {currentConfig?.caspar.controllers.tcp.port ?? 5250}
              </span>{' '}
              on this server's address
            </>
          )}
        </span>
        {scannerEndpoint && (
          <span className={scannerEndpoint.isDefault ? '' : 'text-amber-400'}>
            Media scanner{' '}
            <span className="font-mono">
              {scannerEndpoint.host}:{scannerEndpoint.port}
            </span>{' '}
            {scannerEndpoint.isDefault ? '(internal)' : '(internal — stock 8000 was busy)'}
          </span>
        )}
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
        {serverLog.length === 0 ? (
          <div className="text-[var(--color-text-muted)]">
            {running
              ? 'Server running — no new console output. (CasparCG logs at startup, then goes quiet when idle.)'
              : 'No output yet — press Start Server. The CasparCG console log appears here.'}
          </div>
        ) : (
          serverLog.map((line, i) => {
            const { cls, keyerNote } = classifyLogLine(line);
            return (
              <div key={i} className={cls}>
                {line}
                {keyerNote && (
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
