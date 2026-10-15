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

## 6. A second DeckLink device: ATEM Mini Extreme ISO G2 over Thunderbolt

Verified 2026-09-28 on the same rig, with a Blackmagic ATEM Mini Extreme ISO G2
attached over Thunderbolt. Its link enumerates entirely under `PCI\VEN_BDBD`
(Blackmagic's PCI vendor ID), parented off an Intel PCI-to-PCI Bridge
(Thunderbolt tunnel), with its AVStream/audio children under the
`DECKLINK\AVSTREAM` / `DECKLINK\AUDIO` namespace — the same Desktop Video driver
stack as the DeckLink SDI Micro, and **zero USB enumeration** anywhere in the
chain. Do not assume a Thunderbolt-attached ATEM is "just a USB capture device"
from its connector shape alone; check the PnP bus topology.

Re-running the `dlprobe` enumeration tool with the ATEM attached now reports:

```
COUNT_RESULT=0 DEVICE_COUNT=2
=== DEVICE 0 === DeckLink SDI Micro       — internal_key=0 external_key=0
=== DEVICE 1 === ATEM Mini Extreme ISO G2 — internal_key=0 external_key=1
```

Unlike the SDI Micro, the ATEM's Thunderbolt link reports **`external_key=1`** —
genuine external-keyer hardware support. This was confirmed live, not just via
enumeration: starting CasparCG 2.5.0 with

```xml
<decklink>
  <device>2</device>
  <keyer>external</keyer>
</decklink>
```

produces, with no error:

```
ATEM Mini Extreme ISO G2 [2-2|1080p5000] Enabled external keyer.
ATEM Mini Extreme ISO G2 [2-2|1080p5000] Initialized.
```

Contrast this with the SDI Micro, which always logs `Failed to enable external
keyer` for the same `keyer` setting (expected and benign — see the keyer
behaviour notes referenced in `ServerPanel.tsx`). A real key+fill channel is
therefore achievable on this rig with **zero extra hardware**: no
`external_separate_device` pairing, no second physical card, just the ATEM's
own Thunderbolt device index with `keyer=external`. CasparCG treats an
omitted keyer the same as `default`, which also resolves to an "external"
attempt internally — so `keyer=default` on this device produces the identical
result as `keyer=external`.

Device numbering follows SDK enumeration order, not any fixed ID: the SDI
Micro enumerated first (CasparCG `device=1`) and the ATEM second
(`device=2`) in this test. Re-run `dlprobe` (or the GUI's own DeckLink panel)
to confirm numbering rather than assuming it before adding or removing
Blackmagic hardware.

## 7. GPU adapter selection for the OpenGL mixer

This NUC is an Intel "Hades Canyon" hybrid-graphics board: an Intel HD
Graphics 630 integrated GPU alongside a discrete AMD Radeon RX Vega M GH, both
real, present, and healthy PCI devices (`PCI\VEN_8086&DEV_591B` and
`PCI\VEN_1002&DEV_694C` respectively — verify with `Get-PnpDevice -Class
Display`, not just `Win32_VideoController`, if a "does it even have an Intel
GPU" question comes up again).

CasparCG's OpenGL device (`Initializing OpenGL Device (sfml)` in the log) is
created via classic `wgl`, not DXGI. Windows' per-application **"Graphics
preference"** setting (`HKCU\Software\Microsoft\DirectX\UserGpuPreferences`,
surfaced in Settings → System → Display → Graphics) only steers **DXGI**
adapter enumeration (Direct3D/UWP apps) — it does **not** affect which
adapter a classic `wgl`-based OpenGL context binds to. Setting it to "Power
saving" for `casparcg.exe` on this box was confirmed to have **no effect**:
the log still reports `Initialized OpenGL 4.5.0 Core Profile Context ...
ATI Technologies Inc.` — the mixer still lands on the AMD Vega chip, with its
documented tendency to render GPU-mixed channels black on some drivers.

The correct lever on an AMD hybrid-graphics system is **AMD's own Radeon
Software (Adrenalin / "CNext")**, which is installed and running on this box
(`RadeonSoftware.exe`) and exposes a genuine per-application Graphics profile
(Power Saving ↔ High Performance) that does influence legacy OpenGL adapter
binding. This was identified but not yet applied/verified end-to-end in this
session — do that (Radeon Software → Gaming → add `casparcg.exe` → Power
Saving → confirm the log now shows `Intel(R) HD Graphics 630` as the OpenGL
vendor, then check real channel output, not just the log line) before relying
on it for a show.

## 8. DeckLink Quad 2 — native hardware key/fill consumes a non-adjacent device index

Verified 2026-10-12/13 on a production playout rig (`samson`), a single
Blackmagic **DeckLink Quad 2** card (one physical card, **not** two — see the
corrected-mistake note below): 8 independently-configurable bi-directional
3G-SDI connectors, SDK-enumerated as devices 1–8. Official spec, for when this
needs re-verifying rather than re-guessing:

- [Markertek: DeckLink Quad 2, 8 Channel Capture and Playback Card](https://www.markertek.com/product/bmd-bdlkdvqd2/blackmagic-design-bdlkdvqd2-decklink-quad-3g-sdi-8-channel-capture-and-playback-card)
- [DeckLink Quad 2 official techspecs PDF](https://www.syntex.cz/media/document/3606/decklink-quad-2-techspecs.pdf)

**The card ships with connectors 1+2 pre-configured in Blackmagic Desktop
Video Setup as a dedicated "key/fill" profile pair** — genuine hardware
premultiplied key+fill with no drift (the headline feature of the Quad 2/Duo 2
generation over the old Quad 1, which needed external genlock referencing to
avoid key/fill drift). The remaining connectors (3–8) are a "single i/o"
profile: freely, individually selectable as input or output by whichever
application opens them — **direction is decided entirely in software by the
consumer that opens the device, there is no separate per-port direction
toggle to set in Desktop Video Setup for those**.

To avoid mixing the two numbering schemes: physical connectors 1+2 are the
key/fill pair, and on this card those are **SDK devices 1 and 5** (physical
connector 2 is SDK device 5). SDK device numbers are enumeration order, not
connector order, so never translate between them by eye.

The trap: opening SDK device **1** as a plain `<decklink><device>1</device>`
CasparCG consumer (no `key_device`, `keyer=default`) is sufficient on its own
to drive the full hardware key+fill pair — the card automatically also claims
its physical companion connector to carry the key signal. That companion
connector is **not** SDK device 2 (the next sequential index); on this card it
enumerates as **SDK device 5**. Device numbering follows SDK enumeration
order, not physical connector order or any fixed ID — confirmed again here,
same as §6 above.

Consequence: assigning SDK device 5 to any *other* CasparCG channel — even
with nothing else running, after killing every known competing process
(`tsg-lineup-tauri.exe`, `BMDStreamingServer.exe`), after a full Windows
reboot — fails every single time with:

```
Exception: .../decklink_consumer.cpp(820): Throw in function enable_video
DeckLink Quad 2 [4-5|1080p5000] Could not enable primary video output.
```

This is **not** the documented process-contention failure mode (that one is
real too — see §"competing processes" below — but produces the *identical*
error string, so don't assume you've found the same bug twice just because
the message matches). Here the device is correctly, permanently unavailable
because it is already hardware-claimed as device 1's key companion the moment
device 1's consumer opens. No amount of killing other processes, no reboot,
fixes it — because nothing is wrongly holding it. The fix is channel
reassignment, not more troubleshooting: give device 1 (and only device 1) to
the channel that actually wants hardware key+fill, and never assign its
hidden companion index to anything else.

**Two already-documented, genuinely different causes share this exact log
line** — grep the day's log for "Could not enable" and check which applies
before assuming either:

- `tsg-lineup-tauri.exe` / `BMDStreamingServer.exe` (Desktop Video Run key) /
  `vMixService` holding the card from a previous session — kill the competing
  process, not the card. See `ti2026-playout-rig` session notes.
- A card's native key/fill pairing claiming a non-adjacent companion index
  (this section) — reassign channels, don't keep restarting.

**Corrected mistake, left in deliberately:** earlier in the same session this
was misdiagnosed as "two physical Quad 2 cards" because Windows PnP shows two
top-level `PCI\VEN_BDBD&DEV_A13F` device nodes (`Get-PnpDevice -Class
Unknown,Other,Media | Where InstanceId -match VEN_BDBD`). Two top-level PCI
device nodes with the same device ID does **not** mean two physical cards —
multi-port professional capture cards routinely expose multiple PCI
functions from one physical board for bandwidth reasons. The official spec
(8 connectors, one card) is the authority here; PCI topology alone is not
enough to infer physical card count, the same lesson as §6/§7 above applied
to a new way of getting it wrong.

## 9. Quad 2 key output — what was observed, and what was not measured

Observed on `samson` (2026-10-14), channel 1 on SDK device 1 with
`<keyer>default</keyer>`:

- `INFO` reports `keyer=external`, `key-only=false` for channel 1, and the log
  shows `Enabled external keyer` for device 1 only. Every other channel logs
  `Failed to enable external keyer`, which is expected on devices without a
  keyer (see the keyer-behaviour note: `default` resolves to `external` in
  2.5.0, so there is no config value that skips the call).
- The `key-fill-identifier` pattern (`mode=straight`) rendered without errors
  on layer 30 of channel 1.
- The operator first saw no signal at all on the connector they took to be the
  key output, then later reported key appearing on the connector that
  previously showed fill. Which connector carries which signal, and whether
  the pair is reversed from the assumption, was **not resolved or measured**.
  Desktop Video Setup also showed device 1 as an input at one point; Caspar
  opens no inputs, so this is presumably the bidirectional connector display,
  but it was not confirmed.

Treat the polarity of the pair as unverified until it is read on a scope or a
multiviewer tile. Do not infer it from the SDK index.

## 10. Playing from the file server through a symlink

Goal: occasionally play material straight from the file server without copying
it, by exposing a server folder inside the CasparCG media path.

Findings (verified 2026-10-14):

- Directory **junctions** cannot target a UNC path. A directory **symbolic
  link** can, provided `fsutil behavior query SymlinkEvaluation` reports
  "Local to remote symbolic links are enabled" (the default).
- PowerShell's `New-Item -ItemType SymbolicLink` opens the target first and
  fails with "Access is denied" from an SSH session. Create the link with
  `CreateSymbolicLinkW` (flag `1`, directory) or `cmd /c mklink /D`, neither of
  which validates the target. Beware non-ASCII characters in the target path
  when passing it through a console code page; build such strings from
  `[char]` code points.
- SMB credentials are per logon session. `cmdkey /add` is refused over SSH
  ("Credentials cannot be saved from this logon session"); run it in the
  interactive desktop session of the account that runs `casparcg.exe`. The
  entry then lives in Credential Manager and is picked up by Caspar. An
  SSH session still gets "Access is denied" afterwards, which is expected.
- To test access as the interactive user from SSH, run a one-shot scheduled
  task with `/IT` as that user, write the result to a file, read it, and delete
  the task.
- The media scanner follows the link. A 45-file, roughly 1 TB folder appeared
  in `CLS` within about 25 seconds, and `CLS` latency stayed at 320-330 ms
  before, during and after indexing. Playback of the large files over the
  link was **not** tested.
- Remove a link with `rmdir <link>` only. Never use `rd /s` or
  `Remove-Item -Recurse` on it, which would follow the link.
- Copying a Mac-authored folder brings `._*` AppleDouble files with it
  (hidden, so `Get-ChildItem` without `-Force` undercounts). Strip them before
  they reach a playout media folder.
- A folder the scanner has open cannot be renamed ("Access is denied"), even
  though CasparCG already lists its contents. Rename media folders before they
  are first indexed, or while the scanner is idle.

## 11. Operational notes from the same live session

- **Two CasparCG instances.** The 09:03 start followed a crash in the HTML
  module (`html.cpp(326): [cef_executor] Could not post task`, then
  `terminate`). `casparcg.exe` child processes (CEF helpers) share the image
  name; identify an instance by the process whose parent is not another
  `casparcg.exe`. A second instance can come from a manual double-click racing
  the launcher's supervisor; the cause on the day was not proven.
- **Conductor does not run on the playout machine.** It runs on the control
  PC (ports 7777 operator surface, 7778 multiviewer). Nothing listens on 7777
  on the playout machine; connection refused there is normal.
- **Per-session SMB mappings.** A drive letter mapped in an interactive
  session reports "Unavailable" from an SSH session on the same machine.
- **Disk headroom on the control PC** matters when staging media there; check
  free space before copying and remove the staging copy once the destination
  is verified byte-for-byte (file count and total bytes).
