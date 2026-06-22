# CasparCG Server GUI

A Windows desktop app for running and configuring CasparCG Server 2.5.0. It is a
profile-based configuration editor and a launcher: it starts and supervises the
server and its media scanner, streams the console log into the window, and speaks
AMCP to the running instance. It also exposes the DeckLink SDK directly for device
setup and an SDI output test that drives the card itself, bypassing the GPU mixer
(useful when the mixer renders black, e.g. on GPUs CasparCG cannot drive).

Built with **Tauri 2** (Rust) and **React + TypeScript + Tailwind CSS**.

> **Platform:** Windows. The DeckLink integration and the process supervision use
> Windows-only facilities, and the app is only built and tested on Windows. The
> Tauri/React shell is portable in principle, but macOS/Linux are not supported.

## Features

- **Profiles** — save, load and switch CasparCG configurations. Channel
  validation is the single source of truth and blocks launching an invalid
  config (impossible options are greyed out, Start is gated on errors).
- **Launch & supervise** — Start/Stop/Restart `casparcg.exe`; the launcher also
  runs the media scanner and keeps both alive: it restarts the scanner if it
  dies and restarts the server on a crash, with a crash-loop guard so an
  unrenderable config cannot thrash. The console log is embedded and
  colour-coded by severity; AMCP reconnects automatically after a restart.
- **DeckLink** — enumerate devices, set duplex mode and persistent labels, read
  live signal status, and run a direct-SDK **SDI test** (Fill / Key / Stop) that
  works even where CasparCG's GPU mixer outputs black.
- **AMCP** — auto-connect to the running server for version and status, and send
  ad-hoc commands.
- **Media scanner** — launched alongside the server on a free loopback port
  (never 8000), with the matching `<amcp><media-server>` written into the
  config so CLS/TLS/THUMBNAIL listings work.
- **TSL UMD** — tally monitor.
- **System info** — versions for CasparCG, the DeckLink driver, NDI and the
  scanner; the Server panel shows the host's primary IP and AMCP port for
  connecting a remote client.

Panels: Server, Paths, Channels, Preview, DeckLink, System, TSL.

## Quick start

### Requirements

- [Node.js](https://nodejs.org/) 18+ with pnpm
- [Rust](https://rustup.rs/) (stable, MSVC toolchain)
- [Tauri prerequisites](https://tauri.app/start/prerequisites/) for Windows
  (WebView2, Visual Studio Build Tools)
- The Blackmagic **DeckLink SDK** is required only for the `decklink` feature.

### Development

```bash
pnpm install
pnpm tauri dev
```

### Build

```bash
# Installer without DeckLink support
pnpm tauri build --bundles nsis

# With DeckLink support (requires the SDK and the MSVC toolchain)
pnpm tauri build --features decklink --bundles nsis
```

The DeckLink integration is gated behind the `decklink` Cargo feature and
compiled in only when requested.

## Project structure

```
caspar-server-gui/
├── src/                        # React + TypeScript front end
│   ├── App.tsx                 # Shell: tabs, app-level log/event listeners
│   ├── components/             # Server, Paths, Channels, Preview, DeckLink,
│   │                           #   System, TSL panels + setup wizard, profile
│   │                           #   sidebar, tab bar, status bar
│   ├── lib/                    # types, Tauri wrappers, Zustand store, validation
│   └── styles/
├── src-tauri/                  # Rust back end
│   ├── src/
│   │   ├── lib.rs              # Tauri commands + server/scanner supervisor
│   │   ├── main.rs             # Entry point
│   │   ├── amcp/               # AMCP TCP client
│   │   ├── config/             # Global Config <-> casparcg.config (XML)
│   │   ├── decklink/           # DeckLink SDK: enumeration, status, SDI test
│   │   ├── http_server/        # Local test server for the preview/colour test
│   │   ├── system/             # version + primary-IP detection
│   │   └── tsl/                # TSL UMD tally monitor
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## Configuration format

The GUI stores each profile as a **Global Config** JSON file that wraps the
CasparCG configuration with metadata, and generates a standard `casparcg.config`
(XML) from it at launch:

```json
{
  "version": "1.0",
  "name": "Studio A - Main Playout",
  "created": "2026-01-20T12:00:00Z",
  "modified": "2026-01-20T14:30:00Z",
  "caspar": {
    "paths": {
      "media": "C:\\Users\\Operator\\Videos",
      "template": "template/",
      "log": "log/",
      "data": "data/"
    },
    "channels": [
      {
        "videoMode": "1080i5000",
        "consumers": [
          { "type": "decklink", "device": 1, "embeddedAudio": true }
        ]
      }
    ]
  },
  "decklink": {
    "devices": [
      {
        "persistentId": "0x12345678",
        "modelName": "DeckLink SDI Micro",
        "label": "Graphics Fill",
        "duplexMode": "half"
      }
    ]
  }
}
```

## Tauri commands

The Rust backend exposes Tauri commands grouped by area: configuration and
profiles, server/scanner process control, DeckLink (enumeration, labels, duplex,
status, SDI test), AMCP, the preview test server, the TSL UMD monitor, system
info and primary-IP, GUI settings, and file dialogs. The authoritative list is
the `generate_handler!` block in `src-tauri/src/lib.rs`.

## Technology stack

| Component | Technology |
|-----------|------------|
| Desktop framework | Tauri 2 |
| Frontend | React 19, TypeScript 5.8 |
| Styling | Tailwind CSS 4 |
| State management | Zustand 5 |
| XML | quick-xml |
| Async runtime | Tokio |
| DeckLink | direct SDK access behind the `decklink` Cargo feature |

## Licence

MIT

---

David Thåst · [thåst.se](https://xn--thst-roa.se) · [FiLORUX](https://github.com/FiLORUX)
