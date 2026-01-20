# Implementation Details

This document describes the technical implementation of the CasparCG Server GUI.

## Architecture Overview

The application follows a clean separation between the Rust backend (Tauri) and React frontend:

```
┌─────────────────────────────────────────────────────────┐
│                    React Frontend                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │  Components │  │   Zustand   │  │   Tauri.ts  │     │
│  │   (UI)      │  │   (Store)   │  │  (Commands) │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
└────────────────────────┬────────────────────────────────┘
                         │ invoke()
┌────────────────────────┴────────────────────────────────┐
│                    Rust Backend                          │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐     │
│  │   Config    │  │   DeckLink  │  │    AMCP     │     │
│  │  (XML/JSON) │  │   (SDK)     │  │   (TCP)     │     │
│  └─────────────┘  └─────────────┘  └─────────────┘     │
│  ┌─────────────┐  ┌─────────────┐                      │
│  │   System    │  │  AppState   │                      │
│  │ (Versions)  │  │  (Shared)   │                      │
│  └─────────────┘  └─────────────┘                      │
└─────────────────────────────────────────────────────────┘
```

## Rust Backend Modules

### `lib.rs` — Main Entry Point

Contains all 29 Tauri commands and the `AppState` struct:

```rust
pub struct AppState {
    pub amcp_client: Arc<Mutex<amcp::AmcpClient>>,
    pub gui_settings: Arc<Mutex<GuiSettings>>,
}
```

**Important**: Commands in `lib.rs` must NOT be `pub` due to Tauri 2.0's macro constraints. This prevents name collision errors (E0255).

### `config/` — Configuration Management

| File | Purpose |
|------|---------|
| `schema.rs` | Type definitions: `VideoMode`, `Consumer`, `Channel`, `CasparConfig` |
| `caspar.rs` | XML parser/generator using `quick-xml` |
| `global.rs` | Global config (JSON) and GUI settings |

**VideoMode enum** supports all standard broadcast formats:

```rust
pub enum VideoMode {
    Pal,           // 576i50
    Ntsc,          // 480i5994
    _720p5000,     // 720p50
    _1080i5000,    // 1080i50
    _1080p5000,    // 1080p50
    _2160p5000,    // 4K 50p
    // ... and more
}
```

### `amcp/` — AMCP Protocol Client

Async TCP client for communicating with CasparCG server:

```rust
pub struct AmcpClient {
    stream: Option<TcpStream>,
    host: Option<String>,
    port: Option<u16>,
}
```

Supports commands:
- `VERSION` — Get server version
- `INFO SYSTEM` — Get system information
- Raw command passthrough

### `decklink/` — DeckLink SDK Integration

| File | Purpose |
|------|---------|
| `devices.rs` | Device enumeration, model detection |
| `duplex.rs` | Duplex mode configuration (Duo 2/Quad 2) |

**Conditional compilation** for SDK availability:

```rust
#[cfg(feature = "decklink")]
pub fn list_devices() -> Result<Vec<DeckLinkDevice>, DeckLinkError> {
    // Real SDK implementation
}

#[cfg(not(feature = "decklink"))]
pub fn list_devices() -> Result<Vec<DeckLinkDevice>, DeckLinkError> {
    // Mock data for development
    Ok(vec![
        DeckLinkDevice {
            device_index: 0,
            persistent_id: "0x12345678".to_string(),
            model_name: "DeckLink Duo 2".to_string(),
            // ...
        },
    ])
}
```

### `system/` — System Version Detection

| File | Purpose |
|------|---------|
| `ndi.rs` | NDI Tools version detection (platform-specific) |
| `scanner.rs` | CasparCG Scanner version via HTTP |

## Frontend Architecture

### State Management (Zustand)

Central store in `src/lib/store.ts`:

```typescript
interface AppStore {
  // GUI state
  isSetupComplete: boolean;
  casparPath: string | null;
  activeTab: TabId;

  // Configuration
  currentConfig: GlobalConfig | null;
  profiles: string[];
  activeProfile: string | null;

  // Connection
  isConnected: boolean;
  serverVersion: string | null;

  // DeckLink
  deckLinkDevices: DeckLinkDevice[];

  // System
  systemVersions: SystemVersions | null;

  // Actions
  setCasparPath: (path: string) => Promise<void>;
  loadProfile: (name: string) => Promise<void>;
  // ...
}
```

### Components

| Component | Responsibility |
|-----------|----------------|
| `SetupWizard` | First-run CasparCG path selection |
| `ProfileSidebar` | Profile list, create/delete/rename |
| `TabBar` | Panel tab navigation |
| `PathsPanel` | Media/template/log/data path editing |
| `ChannelsPanel` | Channel and consumer configuration |
| `DeckLinkPanel` | DeckLink device list and settings |
| `SystemInfoPanel` | Version information display |
| `StatusBar` | Connection status, quick info |

### Tauri Command Wrappers

Type-safe wrappers in `src/lib/tauri.ts`:

```typescript
export async function loadCasparConfig(path: string): Promise<CasparConfig> {
  return invoke('load_caspar_config', { path });
}
```

## Configuration Formats

### Global Config (JSON)

Wraps CasparCG config with metadata and DeckLink settings:

```json
{
  "version": "1.0",
  "name": "Profile Name",
  "created": "ISO8601",
  "modified": "ISO8601",
  "caspar": { /* CasparConfig */ },
  "decklink": {
    "devices": [ /* DeckLinkDevice[] */ ]
  }
}
```

### CasparCG Config (XML)

Standard `casparcg.config` format:

```xml
<?xml version="1.0" encoding="utf-8"?>
<configuration>
  <paths>
    <media-path>./media/</media-path>
    <template-path>./templates/</template-path>
  </paths>
  <channels>
    <channel>
      <video-mode>1080i5000</video-mode>
      <consumers>
        <decklink>
          <device>1</device>
        </decklink>
      </consumers>
    </channel>
  </channels>
</configuration>
```

## Profile Storage

Profiles are stored alongside the CasparCG installation:

```
C:\CasparCG\
├── casparcg.exe
├── casparcg.config           # Active config (exported XML)
├── caspar-gui-profiles/      # GUI profiles folder
│   ├── Default.json          # Profile 1
│   ├── Studio A.json         # Profile 2
│   └── Backup.json           # Profile 3
└── ...
```

GUI settings (last profile, window state) stored in OS-specific app data:
- Windows: `%APPDATA%\com.thast.caspar-server-gui`
- macOS: `~/Library/Application Support/com.thast.caspar-server-gui`
- Linux: `~/.config/com.thast.caspar-server-gui`

## Build Configuration

### Cargo.toml Features

```toml
[features]
default = []
decklink = []  # Enable real DeckLink SDK integration
```

### Tauri Configuration

Key settings in `tauri.conf.json`:

```json
{
  "productName": "CasparCG Server GUI",
  "identifier": "com.thast.caspar-server-gui",
  "app": {
    "windows": [{
      "title": "CasparCG Server GUI",
      "width": 1200,
      "height": 800,
      "minWidth": 900,
      "minHeight": 600
    }]
  }
}
```

## Error Handling

All Tauri commands return `Result<T, String>`:

```rust
#[tauri::command]
async fn load_caspar_config(path: String) -> Result<CasparConfig, String> {
    let content = std::fs::read_to_string(&path)
        .map_err(|e| format!("Failed to read file: {}", e))?;

    parse_caspar_xml(&content)
        .map_err(|e| format!("Failed to parse config: {}", e))
}
```

Frontend handles errors in the store:

```typescript
try {
  const config = await loadCasparConfig(path);
  set({ currentConfig: config, error: null });
} catch (error) {
  set({ error: String(error) });
}
```
