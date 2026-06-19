# DeckLink Hardware Verification

This document records the first end-to-end verification of the DeckLink SDK
integration against real Blackmagic hardware, the latent defects it uncovered,
and how to reproduce the checks. Until this exercise the `decklink` feature had
**never been compiled** (there was no `target/` directory), so the integration
was untested by definition.

## 1. Test rig

Verified on 2026-06-19, on the Windows host that carries the card:

| Component | Value |
|-----------|-------|
| Card | Blackmagic **DeckLink SDI Micro** (1× SDI in, 1× SDI out) |
| Persistent ID | `0xD82C98C0` |
| Custom label | `Thast NUC` (set via Desktop Video, read back correctly) |
| Desktop Video driver | **16.0.1** (active, COM-registered) |
| Bundled SDK headers | 15.3 — older than the runtime, binds forward-compatibly |
| C++ toolchain | Visual Studio 2017 Build Tools, MSVC 14.16 |
| IDL compiler | MIDL from Windows SDK 10.0.17763 |
| Rust | 1.96.0, `stable-x86_64-pc-windows-msvc` |

> A stale second install (`Program Files\Blackmagic Design\Desktop Video\`,
> 16.0.1's predecessor 12.2.2) also exists on disk but is **not** the registered
> COM server. Always read the API version at runtime rather than trusting a file
> on disk — see defect 4.

### Keying reality for the SDI Micro

The card reports `supports_internal_keying = false` **and**
`supports_external_keying = false`, confirmed against the live COM API. It has a
single SDI output, so it physically cannot emit a separate fill/key pair for a
downstream hardware keyer. For key/fill workflows on this card you therefore
either:

- output the **composite** (CasparCG mixes fill and key internally to one SDI), or
- output **fill only** or **key only** (`key_only` on the DeckLink consumer), or
- pair **two** single-output devices via the consumer's `key_device` +
  `keyer = external_separate_device` — the configuration schema already supports
  this.

## 2. Verification chain

Two independent layers were exercised, both against the physical card.

### 2.1 Standalone C++ COM probe

The project's own `decklink_wrapper.cpp` was compiled unchanged with MIDL + `cl`
and linked against the registered Desktop Video runtime, with a small `main()`
that prints every field the GUI wrapper surfaces, plus the live API version. This
isolates the C/COM layer from Rust. Output:

```
RUNTIME_API_VERSION=16.0.1
COMPILE_TIME_API_VERSION=16.0.1      (after defect 4 fixed)
DEVICE_COUNT=1
model_name=DeckLink SDI Micro
display_name / device_label="Thast NUC"
persistent_id=0xD82C98C0
video_in=0x1 video_out=0x1  (1× SDI in, 1× SDI out)
io_support=0x3              (capture + playback)
internal_key=0 external_key=0  dual=0 quad=0 idle=1 max_audio=16
```

### 2.2 Real Rust FFI (`cargo test`)

A hardware-in-the-loop integration test (`src-tauri/tests/decklink_hw.rs`)
enumerates through the exact production FFI path the Tauri commands use:

```
DeckLink API version: Some("16.0.1")
Enumerated 1 DeckLink device(s):
  [1] DeckLink SDI Micro | id=0xD82C98C0 label=Some("Thast NUC")
      sdi_in=1 sdi_out=1 int_key=false ext_key=false duplex=false max_audio=16
test enumerate_decklink_hardware ... ok
All 8 worker threads enumerated 1 device(s) consistently
test enumerate_decklink_from_many_threads ... ok
```

The second test spawns eight worker threads to mirror how Tauri dispatches
commands across a tokio thread pool, proving the COM-apartment fix (defect 5).

## 3. Defects found and fixed

| # | Defect | Effect | Fix |
|---|--------|--------|-----|
| 1 | `decklink_wrapper.cpp` referenced `BLACKMAGIC_DECKLINK_API_VERSION_STRING` without including `DeckLinkAPIVersion.h` | `decklink` build failed to compile | Added the include |
| 2 | `build.rs` compiled `decklink_wrapper.cpp` but **not** the MIDL-generated `DeckLinkAPI_i.c` | link failed: `unresolved external symbol CLSID_CDeckLinkIterator`, … | Added `out_dir/DeckLinkAPI_i.c` to the `cc` build |
| 3 | Bundle resource `../test` and the test-server path pointed at a non-existent `test/` dir / `key-fill-identifier.html` | `tauri build` aborted (`resource path ..\test doesn't exist`); test pattern would 404 | Repointed to the real `key-fill-identifier/` folder serving `index.html` |
| 4 | `decklink_get_api_version()` returned the **compile-time** SDK constant (15.3) | GUI reported the wrong driver version (live driver is 16.0.1) | Query `IDeckLinkAPIInformation::GetString(BMDDeckLinkAPIVersion)` at runtime, fall back to the constant |
| 5 | COM initialised once on the first thread only; commands run on arbitrary tokio workers | Risk of `CO_E_NOTINITIALIZED` / non-deterministic enumeration failures in the running app | `ComApartment` RAII guard initialises COM per FFI call (balanced), verified across 8 threads |

Defects 1–3 each independently broke the build, which is why the feature had
never compiled before.

## 4. Reproducing the checks

Prerequisites: Blackmagic Desktop Video installed, a DeckLink device present,
Rust MSVC toolchain, and a Visual Studio Build Tools + Windows SDK (for MIDL).
Run from a developer command prompt (so `vcvars` sets the SDK include paths that
MIDL needs):

```bat
cd src-tauri
cargo test --features decklink --test decklink_hw -- --nocapture
```

Without the `decklink` feature the same test runs against the mock device list,
so it stays green in CI on machines with no card.

## 5. Known limitations (not blocking)

- **Duplex get/set** is unimplemented for real hardware (`get_duplex_mode` /
  `set_duplex_mode` return `SdkNotAvailable` under the feature; the wrapper
  exposes no duplex entry points). It is irrelevant to the single-port SDI Micro
  but is the headline feature for Duo/Quad cards and needs
  `IDeckLinkProfileManager` to be wired through the wrapper.
- **Device-label write** (`set_decklink_label`) is a no-op; the GUI persists
  labels in the profile instead. Writing to the card's NVRAM would use
  `IDeckLinkConfiguration::SetString` + `WriteConfigurationToPreferences`.
- **`count_sdi_connections`** only checks the SDI bit and returns at most 1, so
  port counts on multi-port cards are not exact. Correct for the SDI Micro.

These could not be exercised on the available single-port, no-keyer card and are
deliberately left documented rather than implemented blind.
