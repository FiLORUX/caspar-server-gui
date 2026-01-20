# Roadmap

## Version 0.1.0 — MVP (Current)

**Goal**: Functional configuration editor with mock DeckLink support

### Completed
- [x] Project scaffolding (Tauri 2.0, React, TypeScript, Tailwind)
- [x] Rust backend with 29 Tauri commands
- [x] CasparCG XML config parsing and generation
- [x] Global config JSON format for profiles
- [x] AMCP TCP client for server communication
- [x] DeckLink module structure with mock data
- [x] React UI components (panels, sidebar, status bar)
- [x] Zustand state management
- [x] Development build working (`pnpm tauri dev`)

### Remaining
- [ ] Basic UI polish and layout fixes
- [ ] Profile save/load functionality testing

---

## Version 0.2.0 — Windows Release

**Goal**: Standalone Windows application with real DeckLink support

### DeckLink SDK Integration
- [ ] Copy DeckLink SDK headers to project
- [ ] Create C wrapper for Windows COM API
- [ ] Configure `build.rs` for Windows builds
- [ ] Implement real device enumeration
- [ ] Implement duplex mode configuration
- [ ] Test on Windows with actual DeckLink hardware

### Windows Build
- [ ] Production build (`pnpm tauri build`)
- [ ] Code signing certificate
- [ ] NSIS or WiX installer
- [ ] Auto-update support (Tauri updater)

### Target: ~15 MB standalone installer

---

## Version 0.3.0 — Feature Complete

**Goal**: Full CasparCG configuration support

### Configuration Editor
- [ ] Complete channel editor (add/remove/reorder)
- [ ] Video mode selector with all formats
- [ ] Consumer configuration forms:
  - [ ] DeckLink consumer (device, keyer, latency)
  - [ ] NDI consumer (name, allow fields)
  - [ ] Screen consumer (device, windowed, size)
  - [ ] System audio consumer
- [ ] Path editor with folder browser
- [ ] Controller settings (TCP port)
- [ ] AMCP media server settings

### Profile Management
- [ ] Profile rename
- [ ] Profile duplicate
- [ ] Profile import/export
- [ ] Export to `casparcg.config` XML

### AMCP Features
- [ ] Connection status polling
- [ ] Auto-reconnect
- [ ] Server restart command
- [ ] Live channel info display

---

## Version 0.4.0 — Cross-Platform

**Goal**: macOS and Linux support

### macOS
- [ ] DeckLink SDK integration (Objective-C)
- [ ] DMG installer
- [ ] Code signing and notarisation

### Linux
- [ ] DeckLink SDK integration (C++)
- [ ] AppImage or Flatpak
- [ ] Desktop integration

---

## Version 1.0.0 — Production Ready

**Goal**: Stable release for broadcast environments

### Polish
- [ ] Application icon and branding
- [ ] Splash screen
- [ ] About dialog with version info
- [ ] Keyboard shortcuts
- [ ] Context menus
- [ ] Tooltips and help text

### Quality
- [ ] Unit test coverage >80%
- [ ] Integration tests
- [ ] Cross-platform CI/CD
- [ ] Documentation
- [ ] User guide

### Performance
- [ ] Startup time <2 seconds
- [ ] Memory usage <100 MB
- [ ] Installer size <20 MB

---

## Future Considerations

### Potential Features
- Template browser with preview
- Media browser with thumbnails
- Real-time playback monitoring
- Multi-server management
- OSC controller support
- Preset configurations (newsroom, sports, etc.)

### Integrations
- CasparCG HTML templates
- Graphics pack management
- Companion integration
- vMix/OBS switcher support

---

## Release Targets

| Version | Target Date | Focus |
|---------|-------------|-------|
| 0.1.0 | January 2026 | MVP development build |
| 0.2.0 | February 2026 | Windows standalone release |
| 0.3.0 | March 2026 | Feature complete |
| 0.4.0 | April 2026 | Cross-platform |
| 1.0.0 | Q2 2026 | Production ready |
