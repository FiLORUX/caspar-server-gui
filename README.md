# CasparCG Server GUI

A modern, cross-platform desktop application for configuring CasparCG Server 2.5.0 with integrated DeckLink SDK support.

Built with **Tauri 2.0** (Rust backend) and **React + TypeScript + Tailwind CSS** (frontend).

## Features

- **Visual Configuration Editor** — Edit CasparCG configuration with a modern UI
- **Profile Management** — Save, load, and switch between multiple configuration profiles
- **DeckLink Integration** — Enumerate devices, configure duplex modes, set labels
- **AMCP Client** — Connect to running CasparCG server for live status
- **System Information** — Display versions for CasparCG, DeckLink, NDI, Scanner
- **Cross-Platform** — Windows (primary), macOS, Linux

## Screenshot

```
┌────────────────────────────────────────────────────────────────────────┐
│  CasparCG Server GUI                                    [_] [□] [X]   │
├────────────────────────────────────────────────────────────────────────┤
│ ┌──────────────┐ ┌────────────────────────────────────────────────────┐│
│ │  PROFILES    │ │ [Paths] [Channels] [DeckLink] [System Info]       ││
│ │              │ ├────────────────────────────────────────────────────┤│
│ │ ▸ Studio A   │ │                                                    ││
│ │   Studio B   │ │  Active panel content                              ││
│ │   Backup     │ │                                                    ││
│ │              │ │                                                    ││
│ │ [+ New]      │ │                                                    ││
│ │ [Import]     │ │                                                    ││
│ └──────────────┘ └────────────────────────────────────────────────────┘│
├────────────────────────────────────────────────────────────────────────┤
│ Server: ● Connected (2.5.0) | DeckLink: Duo 2 | NDI: 6.0.1           │
└────────────────────────────────────────────────────────────────────────┘
```

## Quick Start

### Prerequisites

- [Node.js](https://nodejs.org/) 18+ with pnpm
- [Rust](https://rustup.rs/) 1.70+
- [Tauri CLI](https://tauri.app/start/prerequisites/)

### Development

```bash
# Install dependencies
pnpm install

# Run in development mode
pnpm tauri dev

# Build for production
pnpm tauri build
```

### Windows Build (with DeckLink SDK)

```bash
# Build with DeckLink support (requires SDK)
cargo build --release --features decklink
```

## Project Structure

```
caspar-server-gui/
├── src/                       # React frontend
│   ├── App.tsx               # Main application component
│   ├── components/           # UI components
│   │   ├── SetupWizard.tsx   # First-run setup
│   │   ├── ProfileSidebar.tsx
│   │   ├── TabBar.tsx
│   │   ├── StatusBar.tsx
│   │   ├── PathsPanel.tsx
│   │   ├── ChannelsPanel.tsx
│   │   ├── DeckLinkPanel.tsx
│   │   └── SystemInfoPanel.tsx
│   ├── lib/
│   │   ├── types.ts          # TypeScript type definitions
│   │   ├── tauri.ts          # Tauri command wrappers
│   │   └── store.ts          # Zustand state management
│   └── styles/
│       └── main.css          # Tailwind + custom CSS
├── src-tauri/                 # Rust backend
│   ├── src/
│   │   ├── lib.rs            # Tauri commands (29 commands)
│   │   ├── main.rs           # Entry point
│   │   ├── amcp/             # AMCP TCP client
│   │   ├── config/           # Config parsing/generation
│   │   ├── decklink/         # DeckLink SDK integration
│   │   └── system/           # System version detection
│   ├── Cargo.toml
│   └── tauri.conf.json
└── package.json
```

## Configuration Format

The GUI uses a **Global Config** JSON format that wraps CasparCG configuration with additional metadata:

```json
{
  "version": "1.0",
  "name": "Studio A - Main Playout",
  "created": "2026-01-20T12:00:00Z",
  "modified": "2026-01-20T14:30:00Z",
  "caspar": {
    "paths": {
      "media": "/data/media/",
      "template": "/data/templates/",
      "log": "/var/log/casparcg/",
      "data": "/data/casparcg/"
    },
    "channels": [
      {
        "videoMode": "1080i5000",
        "consumers": [
          {
            "type": "decklink",
            "device": 1,
            "embeddedAudio": true
          }
        ]
      }
    ]
  },
  "decklink": {
    "devices": [
      {
        "persistentId": "0x12345678",
        "modelName": "DeckLink Duo 2",
        "label": "Graphics Fill",
        "duplexMode": "half"
      }
    ]
  }
}
```

## Tauri Commands

The Rust backend exposes 29 commands:

| Category | Commands |
|----------|----------|
| **Config** | `load_caspar_config`, `save_caspar_config`, `load_global_config`, `save_global_config`, `export_to_caspar_xml`, `create_default_config`, `list_profiles` |
| **DeckLink** | `list_decklink_devices`, `get_decklink_info`, `set_decklink_label`, `set_decklink_duplex_mode`, `get_decklink_driver_version` |
| **AMCP** | `amcp_connect`, `amcp_disconnect`, `amcp_is_connected`, `amcp_connection_info`, `amcp_version`, `amcp_info_system`, `amcp_send_command` |
| **System** | `get_ndi_version`, `get_scanner_version`, `get_system_versions` |
| **Settings** | `get_gui_settings`, `save_gui_settings`, `set_caspar_path` |
| **Dialogs** | `pick_folder`, `pick_config_file`, `pick_save_location` |

## Technology Stack

| Component | Technology |
|-----------|------------|
| Desktop Framework | Tauri 2.0 |
| Frontend | React 19, TypeScript 5.7 |
| Styling | Tailwind CSS 4 |
| State Management | Zustand 5 |
| XML Parsing | quick-xml 0.37 |
| Async Runtime | Tokio |
| DeckLink | Conditional compilation with feature flag |

## Licence

MIT
