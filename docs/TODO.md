# TODO

## Completed

- [x] Scaffold Tauri 2.0 project with React + TypeScript
- [x] Configure Tailwind CSS 4 with custom broadcast-style variables
- [x] Create Rust backend module structure
- [x] Implement CasparCG XML parser/generator (`quick-xml`)
- [x] Implement Global Config JSON format for profiles
- [x] Implement AMCP TCP client with async commands
- [x] Implement DeckLink device enumeration (mock data)
- [x] Implement system version detection (NDI, Scanner)
- [x] Create React frontend components
- [x] Build Zustand store for state management
- [x] Create TypeScript types matching Rust structures
- [x] Fix Tauri 2.0 command visibility constraints (remove `pub` from commands)
- [x] Fix dialog plugin configuration error
- [x] Verify `pnpm tauri dev` runs successfully

## In Progress

- [ ] Integrate DeckLink SDK for Windows builds
  - [ ] Copy SDK headers to `src-tauri/sdk/decklink/`
  - [ ] Create C wrapper for COM API
  - [ ] Configure `build.rs` for Windows compilation
  - [ ] Update Rust bindings to use real SDK

## Pending

### High Priority (Windows Release)

- [ ] Windows DeckLink SDK integration
- [ ] Test Windows build (`pnpm tauri build --target x86_64-pc-windows-msvc`)
- [ ] Code signing for Windows
- [ ] Installer configuration (NSIS/WiX)

### Medium Priority (Feature Complete)

- [ ] Profile import/export functionality
- [ ] Channel configuration UI (add/remove/reorder)
- [ ] Consumer configuration forms (decklink, ndi, screen)
- [ ] Video mode dropdown with all supported formats
- [ ] AMCP connection status polling
- [ ] Auto-reconnect on connection loss

### Key/Fill Test Pattern Integration

Integrate the `test/key-fill-identifier.html` test pattern into the GUI for one-click channel verification without requiring the CasparCG client.

- [ ] **Test Pattern Server**
  - [ ] Embed static HTTP server in Tauri backend (or use Tauri's asset protocol)
  - [ ] Serve `test/key-fill-identifier.html` at predictable URL
  - [ ] Support dynamic port configuration

- [ ] **Channel Test UI**
  - [ ] Add "Test Channels" button to toolbar/menu
  - [ ] Toggle button sends AMCP commands to all configured channels:
    ```
    CG {ch}-20 ADD 0 "http://localhost:{port}/key-fill-identifier.html?mode=fill&id={ch}" 1
    CG {ch}-19 ADD 0 "http://localhost:{port}/key-fill-identifier.html?mode=key&id={ch}" 1
    MIXER {ch}-19 KEYER 1
    ```
  - [ ] "Stop Test" clears all test layers
  - [ ] Per-channel test toggle in channel list

- [ ] **Audio Ident System**
  - [ ] Implement Web Speech API ident in test pattern
  - [ ] Voice synthesis (rate 0.9, pitch 0.9)
  - [ ] Stereo test tones via Web Audio API with `StereoPannerNode`
  - [ ] Timed announcement schedule:
    | Time | Left Channel | Right Channel |
    |------|--------------|---------------|
    | :00 | Intro + left ident | Intro |
    | :15 | — | Right ident |
    | :30 | Left short | — |
    | :45 | — | Right short |
  - [ ] Enable via `?audio=1` URL parameter
  - [ ] Feed number in announcements matches channel ID

- [ ] **Preview Panel**
  - [ ] Embedded WebView showing preview mode of test pattern
  - [ ] Dropdown to select channel for preview
  - [ ] Visual confirmation without SDI monitoring

### Low Priority (Polish)

- [ ] Application icon and branding
- [ ] Dark/light theme toggle
- [ ] Keyboard shortcuts
- [ ] Undo/redo for config changes
- [ ] Config validation with error highlighting
- [ ] Cross-platform testing (Linux, macOS)

## Known Issues

1. **Unused code warnings** — Many helper functions in DeckLink and system modules are unused until SDK integration is complete

2. **XML parser incomplete** — Currently parses basic structure; needs full consumer type support

3. **AMCP client blocking** — Should add timeout handling for unresponsive servers

## Technical Debt

- [ ] Add proper error types instead of `String` errors
- [ ] Add logging with `log` and `env_logger` crates
- [ ] Add unit tests for config parsing
- [ ] Add integration tests for AMCP client
- [ ] Clean up unused imports and dead code
