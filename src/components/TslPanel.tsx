// TSL UMD tally monitor panel
// Listens for TSL 3.1 UMD tally over UDP and shows the latest state per address.

import { useEffect, useState } from 'react';
import * as tauri from '../lib/tauri';
import type { TslDisplay } from '../lib/types';

const TALLY_COLOURS = ['bg-red-500', 'bg-emerald-500', 'bg-amber-400', 'bg-sky-400'];

export function TslPanel() {
  const [port, setPort] = useState(8900);
  const [boundPort, setBoundPort] = useState<number | null>(null);
  const [displays, setDisplays] = useState<TslDisplay[]>([]);
  const [error, setError] = useState<string | null>(null);

  // Sync the running state on mount (the listener lives in the backend).
  useEffect(() => {
    tauri.tslMonitorPort().then(setBoundPort).catch(() => {});
  }, []);

  // Poll the latest displays while the listener is running.
  useEffect(() => {
    if (boundPort === null) return;
    let cancelled = false;
    const poll = async () => {
      try {
        const next = await tauri.getTslDisplays();
        if (!cancelled) setDisplays(next);
      } catch {
        /* transient — ignore */
      }
    };
    poll();
    const id = setInterval(poll, 1000);
    return () => {
      cancelled = true;
      clearInterval(id);
    };
  }, [boundPort]);

  const start = async () => {
    setError(null);
    try {
      setBoundPort(await tauri.startTslMonitor(port));
    } catch (e) {
      setError(String(e));
    }
  };

  const stop = async () => {
    try {
      await tauri.stopTslMonitor();
    } catch {
      /* ignore */
    }
    setBoundPort(null);
    setDisplays([]);
  };

  return (
    <div>
      <div className="flex items-center justify-between mb-4">
        <div>
          <h2 className="text-lg font-semibold">TSL UMD Tally Monitor</h2>
          <p className="text-sm text-[var(--color-text-secondary)]">
            Listen for TSL 3.1 UMD tally over UDP from a vision mixer or router
          </p>
        </div>
        <div className="flex items-center gap-2">
          <label className="text-sm text-[var(--color-text-muted)]">UDP port</label>
          <input
            type="number"
            value={port}
            min={1}
            max={65535}
            disabled={boundPort !== null}
            onChange={(e) => setPort(parseInt(e.target.value, 10) || 8900)}
            className="w-24 text-sm"
          />
          {boundPort === null ? (
            <button
              onClick={start}
              className="px-4 py-2 bg-emerald-600 hover:bg-emerald-700 text-white rounded transition-colors"
            >
              Start
            </button>
          ) : (
            <button
              onClick={stop}
              className="px-4 py-2 bg-amber-600 hover:bg-amber-700 text-white rounded transition-colors"
            >
              Stop
            </button>
          )}
        </div>
      </div>

      {error && (
        <div className="mb-4 p-3 rounded bg-red-500/15 text-red-400 text-sm">{error}</div>
      )}

      <div className="mb-4 text-sm text-[var(--color-text-muted)]">
        {boundPort !== null
          ? `Listening on UDP ${boundPort} · ${displays.length} display(s) seen`
          : 'Stopped'}
      </div>

      {boundPort !== null && displays.length === 0 ? (
        <div className="panel p-8 text-center text-sm text-[var(--color-text-muted)]">
          Waiting for TSL UMD packets — point your mixer or router at this host's UDP
          port {boundPort}.
        </div>
      ) : (
        <div className="space-y-2">
          {displays.map((d) => (
            <div key={d.address} className="flex items-center gap-3 p-3 panel">
              <span className="px-2 py-0.5 text-xs bg-[var(--color-accent)] text-white rounded">
                {d.address}
              </span>
              <span className="font-medium flex-1">{d.label || '(no label)'}</span>
              <div className="flex gap-1">
                {d.tally.map((on, i) => (
                  <span
                    key={i}
                    title={`Tally ${i + 1}`}
                    className={`w-4 h-4 rounded-full ${
                      on ? TALLY_COLOURS[i] : 'bg-[var(--color-bg-tertiary)]'
                    }`}
                  />
                ))}
              </div>
            </div>
          ))}
        </div>
      )}
    </div>
  );
}
