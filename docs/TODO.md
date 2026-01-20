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
