# DeckLink SDK Integration

This directory contains the Blackmagic DeckLink SDK files and a C wrapper for Rust FFI.

## Structure

```
decklink/
├── include/           # SDK IDL files and headers
│   ├── DeckLinkAPI.idl
│   ├── DeckLinkAPIVersion.h
│   └── ... (other IDL files)
├── wrapper/           # C wrapper for Rust FFI
│   ├── decklink_wrapper.h
│   └── decklink_wrapper.cpp
└── README.md
```

## Build Requirements

### Windows

1. **Visual Studio 2019 or later** with C++ workload
2. **Windows SDK** (includes MIDL compiler)
3. **DeckLink Desktop Video** drivers installed

The build process:
1. `build.rs` runs MIDL to compile IDL files into C++ headers
2. `cc` crate compiles the C++ wrapper with MSVC
3. Rust links against the resulting static library

### macOS / Linux

The wrapper compiles as a stub that returns `DECKLINK_ERROR_NO_DRIVER`.
The GUI falls back to mock device data on non-Windows platforms.

## SDK Version

- **DeckLink SDK**: 15.3
- **API Version**: `0x0f030000`

## Licence

The DeckLink SDK is provided by Blackmagic Design under their EULA:
https://www.blackmagicdesign.com/EULA/DeckLinkSDK

The C wrapper code is MIT licensed.

## Usage from Rust

```rust
#[cfg(feature = "decklink")]
extern "C" {
    fn decklink_init() -> i32;
    fn decklink_cleanup();
    fn decklink_get_device_count(count: *mut i32) -> i32;
    fn decklink_get_device_info(index: i32, info: *mut DeckLinkDeviceInfo) -> i32;
}
```

See `src-tauri/src/decklink/devices.rs` for the full implementation.
