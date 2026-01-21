# TSG Key/Fill Channel Identifier

> Professional broadcast test pattern for CasparCG, vMix, and OBS key/fill output verification.

## Overview

This HTML-based test graphic enables rapid identification and pairing verification of key/fill SDI outputs from DeckLink cards. Designed for broadcast environments with multiple Quad 2 cards where up to 16 key/fill pairs need systematic identification.

### Key Features

- **Three viewing modes:** Fill, Key, and Preview (combined simulation)
- **Channel fingerprinting:** Unique visual per channel (1–8) for instant mismatch detection
- **Premultiplied alpha design:** Mathematically correct for hardware keying
- **Audio ident system:** Speech synthesis with stereo test tones
- **Zero dependencies:** Pure HTML5/CSS3/ES2023

---

## URL Parameters

```
?mode=<mode>&id=<channel>
```

| Parameter | Values | Default | Description |
|-----------|--------|---------|-------------|
| `mode` | `preview`, `fill`, `key` | `preview` | Output mode |
| `id` | `1`–`16` | `1` | Channel/feed identifier |

### Examples

```bash
# Preview mode (combined simulation) — default
http://localhost:9966/key-fill-identifier.html

# Channel 5, fill output
http://localhost:9966/key-fill-identifier.html?mode=fill&id=5

# Channel 5, key output
http://localhost:9966/key-fill-identifier.html?mode=key&id=5

# Channel 12 preview
http://localhost:9966/key-fill-identifier.html?id=12
```

---

## Output Modes

### Preview Mode (Default)

Simulates the combined key/fill output as it would appear through a downstream keyer or DeckLink hardware.

- **Background:** Checkerboard pattern (indicates transparency)
- **FILL text:** Visible in accent colour
- **KEY text:** Visible in gradient (simulates keyed reveal)
- **✓ PAIRED:** Text revealed (only visible when combined)

Use this mode for verification in a web browser before deploying to CasparCG.

### Fill Mode

RGB colour output for the fill channel. Renders:

- Colourful graphics, textures, and gradients
- "FILL" label prominently displayed
- Rotating arc animation (fill-specific motion)
- Channel ID with decorative rings
- EBU colour bars strip

### Key Mode

Luminance matte output for the key channel. Renders:

- Pure white elements on transparent background
- "KEY" label as white matte
- Scanning bar animation (key-specific motion)
- Channel fingerprint as white segments

---

## Visual Components

### Channel Fingerprint

An 8-segment circular indicator providing instant visual verification of channel pairing.

```
        1
      8   2
    7   •   3
      6   4
        5
```

- Each channel (1–8) activates its corresponding segment
- Channels 9–16 wrap to segments 1–8
- **Correct pairing:** Fill and key show identical active segment
- **Mismatch:** Different segments highlighted — immediately obvious

The rotating radar arm also starts at a channel-specific angle, providing secondary verification through animation phase alignment.

### Header Labels

| Mode | Left Box | Right Box |
|------|----------|-----------|
| Fill | "CH 01 › FILL" (cyan) | Subtle gradient background |
| Key | Solid white (alpha) | "CH 01 › KEY" (white matte) |
| Preview | "CH 01 › FILL" (cyan) | "CH 01 › KEY" (gradient text) |

### Motion Panels

Two distinct animations ensure temporal verification:

1. **Fill Sync:** Rotating concentric arcs (visible in fill/preview)
2. **Key Sync:** Horizontal scanning bar (matte in key, revealed in preview)

### PAIRED Reveal

Text that appears **only** when fill and key are correctly combined:

- **Fill mode:** Invisible (colour matches animated gradient)
- **Key mode:** White matte text
- **Combined:** "✓ PAIRED" revealed in gradient colours

This is the "magic" verification — if you see "✓ PAIRED", your key/fill routing is correct.

---

## CasparCG Integration

### Basic AMCP Commands

```
# Load fill on layer 20, key on layer 19
CG 1-20 ADD 0 "http://localhost:9966/key-fill-identifier.html?mode=fill&id=1" 1
CG 1-19 ADD 0 "http://localhost:9966/key-fill-identifier.html?mode=key&id=1" 1

# Enable keying (layer 19 keys layer 20)
MIXER 1-19 KEYER 1
```

### Multiple Channels

```bash
# Channel 1 (DeckLink output 1)
CG 1-20 ADD 0 "http://...?mode=fill&id=1" 1
CG 1-19 ADD 0 "http://...?mode=key&id=1" 1
MIXER 1-19 KEYER 1

# Channel 2 (DeckLink output 2)
CG 2-20 ADD 0 "http://...?mode=fill&id=2" 1
CG 2-19 ADD 0 "http://...?mode=key&id=2" 1
MIXER 2-19 KEYER 1
```

### Runtime Updates

The template supports CG UPDATE for live parameter changes:

```
CG 1-20 UPDATE 0 "{\"id\":5}"
CG 1-19 UPDATE 0 "{\"id\":5}"
```

---

## vMix Integration

Create two Browser inputs:

1. **Fill input:** `http://localhost:9966/key-fill-identifier.html?mode=fill&id=1`
2. **Key input:** `http://localhost:9966/key-fill-identifier.html?mode=key&id=1`

Configure as external key/fill pair in vMix output settings.

---

## OBS Integration

Add two Browser sources:

1. **Fill source:** URL with `?mode=fill&id=1`
2. **Key source:** URL with `?mode=key&id=1`

Use the Lua or Python scripting API to route as key/fill if supported by your output plugin.

---

## Audio Ident System

The identifier includes an optional audio ident programme using Web Speech API and Web Audio API.

### Features

- **Speech synthesis:** Voice announcements via Web Speech API
- **Test tone:** 1 kHz sine wave at -18 dBFS (EBU reference level)
- **Stereo verification:** Left/right channel identification
- **Timed sequence:** Announcements at :00, :15, :30, :45 each minute

### Announcement Schedule

| Time | Left Channel | Right Channel |
|------|--------------|---------------|
| :00 | Intro + "Feed N. Left-hand audio channel..." | Intro (simultaneous) |
| :15 | — | "Feed N. Right-hand audio channel..." |
| :30 | "Feed N. Left channel." | — |
| :45 | — | "Feed N. Right channel." |

### Technical Notes

- Speech synthesis outputs to both channels (browser limitation)
- Test tones use `StereoPannerNode` for true L/R separation
- Enable via `?audio=1` parameter

---

## Premultiplied Alpha Explained

This test pattern is designed with premultiplied alpha mathematics in mind:

```
Combined_RGB = Fill_RGB × Key_Luminance
Combined_Alpha = Key_Luminance
```

Every pixel is considered for three viewing contexts:

1. **Fill monitor:** What RGB values are present?
2. **Key monitor:** What luminance (alpha) values are present?
3. **Combined output:** Fill × Key multiplication result

### Design Principles

- Where key is **white** (1.0): Fill colour passes through completely
- Where key is **black** (0.0): Output is transparent
- Where key is **grey** (0.5): Fill colour at 50% opacity

The "✓ PAIRED" text exploits this:
- Fill has the text in a colour matching the gradient (invisible)
- Key has the text as white matte
- Combined: gradient colour shows through the text shape

---

## Troubleshooting

### Fingerprint Mismatch

If fill shows segment 1 but key shows segment 4:
- Check SDI cable routing
- Verify CasparCG channel configuration
- Confirm DeckLink device mapping

### No Alpha (Solid Background)

- Ensure browser/CasparCG supports transparent rendering
- Check `background: transparent` is not overridden
- Verify key output is connected to keyer input

### Animation Out of Sync

If radar arms are not aligned between fill and key:
- Channels are mismatched (fingerprint will also differ)
- Or: Pages loaded at different times (refresh both simultaneously)

---

## Files

| File | Purpose |
|------|---------|
| `key-fill-identifier.html` | Main test pattern (recommended) |
| `key-fill-tester.html` | Legacy version |
| `key-fill-tester.md` | This documentation |

---

## Version History

- **2.0.0** (2026-01) — Complete rewrite with fingerprint system, preview mode, modern URL params
- **1.0.0** (2025-12) — Initial key/fill test pattern

---

*TSG Key/Fill Identifier — Part of the CasparCG Server GUI project*
