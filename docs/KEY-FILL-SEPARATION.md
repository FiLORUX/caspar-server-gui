# Key and Fill Separation

How the test pattern makes elements appear on one signal and not the other, why
the numbers are what they are, and how to re-measure them. Every figure below was
measured on a live SDI chain, not derived from the page.

## The identity the whole thing rests on

SDI fill and key are **premultiplied** (shaped). For a pixel of straight colour
`C` at alpha `a`:

```
fill = C × a          key = a
```

This was confirmed end to end rather than assumed. `calibration.html` band 2 pins
colour at 255 and steps only alpha; measured on the fill output it produced

```
0  15  32  48  66  82 100 117 133 151 167 185 201 218 235 252
```

identical to band 1's grey staircase. Fill tracks `255 × a`, so the chain is
premultiplied. Straight alpha would have left band 2 flat at 252.

Compositing a colour `C` at alpha `a` over a backdrop whose premultiplied fill is
`F` and alpha is `A` gives

```
fill_out = C×a + F×(1−a)          key_out = a + A×(1−a)
```

Both separation techniques fall straight out of those two lines.

## Technique 1 — fill only, by an opaque plate

**Goal:** obvious on fill, invisible on key.

An element is invisible on the key exactly when its alpha matches its
surroundings. The robust way to guarantee that is an **opaque** plate: a fully
opaque layer occludes whatever sits beneath it, so alpha is pinned at exactly 1
across the whole block regardless of what the background is doing. Only the
colour then varies inside it.

Used by the numerals and the left header box:

| Element | Colour | Alpha | Fill | Key |
|---|---|---|---|---|
| Plate | 38 | 1.00 | 38 | 255 |
| Numerals | 255 | 1.00 | 255 | 255 |

Any glow must go. `text-shadow` spills alpha past the plate edge and prints a
halo on the key, which is precisely the separation the plate exists to remove.
Inside a clipped opaque box a glow is harmless, because alpha there is already 1.

## Technique 2 — key only, by matching the premultiplied fill

**Goal:** invisible on fill, readable on key.

Here stacking is exploited rather than fought. From the compositing line above,
`fill_out = F` for **any** alpha precisely when `C = F`. So painting an element
in the backdrop's own premultiplied fill value leaves the fill untouched, while
its alpha necessarily rises and lifts the key.

Used by the ident line and the right header box:

| Element | Colour | Alpha | Fill | Key |
|---|---|---|---|---|
| Bed | 140 | 0.70 | 98 | 179 |
| Glyphs | 98 | 1.00 | 98 | 255 |

Matching the **premultiplied fill** rather than the colour is what makes this
survive anti-aliasing: every partial-coverage edge pixel is a blend of two
surfaces that already share a fill value, so the blend carries that fill too.
Edges soften on the key and stay invisible on the fill. Matching colour instead
leaves glyph pixels several levels proud of the backdrop.

## The stacking trap

A semi-transparent element drawn **on top of** a bed can never match that bed's
alpha. Compositing gives `a_total = a_patch + a_bed(1 − a_patch)`, and solving
`a_total = a_bed` yields `a_patch = 0` — which takes the element's fill to zero
with it. Two layers of alpha 0.70 composite to 0.91, not 0.70.

So a fill-only element must **replace** the bed, not cover it. That is why the
level ladder is built from sibling rectangles rather than overlays, and why the
coincidence chevrons cannot be made fill-only in an alpha rig at all — a
`clip-path` shape cannot tile a bed. The chevrons are a registration instrument
only; their full three-state behaviour needs a two-output rig with separate
`mode=fill` and `mode=key` renders.

## Why the decorative wash is hidden in straight mode

`.fill-texture` and `.fill-sweep` paint animated diagonals across the whole
raster. A key-only bed must be semi-transparent for its alpha difference to
exist, so whatever lies behind leaks through at `(1 − a)` and drags the fill with
it. With an animated wash behind, that leak varies per frame and the null never
holds. Key mode already hides both; straight mode now does the same, which puts
every instrument on a backdrop of known zero. Measured spread on the header boxes
fell from tens of levels to 3 once the wash was gone.

## Measured on the live chain

Read from the ATEM multiviewer's TB Fill and TB Key tiles.

| Element | TB Fill | TB Key | Verdict |
|---|---|---|---|
| Numerals plate | 35 / 35 / 252, spread 217 | 254 / 254 / 254, spread 0 | fill only |
| Left header box | 16 / 18 / 159, spread 143 | 251 / 254 / 254, spread 3 | fill only |
| Right header box | 93 / 96 / 96, spread 3 | 173 / 178 / 247, spread 74 | key only |
| Ladder BED | 96 | 178 | reference |
| Ladder FILL ONLY | 177 | 179 | vanishes on key |
| Ladder KEY ONLY | 96 | 254 | vanishes on fill |
| Ladder BOTH | 252 | 254 | visible on both |

`FILL ONLY` sits 1 level from the bed on the key; `KEY ONLY` sits 0 levels from
the bed on the fill. The residual few levels are the multiviewer's own rescaling,
not leakage.

## Re-measuring

1. Play `calibration.html` on the channel.
2. Capture the multiviewer **without disturbing playout**: the DeckLink driver
   exposes the card's input as a DirectShow source named `Blackmagic WDM Capture`,
   which is a separate path from the output the server holds.

   ```
   ffmpeg -f dshow -rtbufsize 200M -pixel_format uyvy422 -video_size 1920x1080 \
          -framerate 50 -i video="Blackmagic WDM Capture" -frames:v 1 -y grab.png
   ```

   The bundled `ffmpeg.exe` has no `decklink` demuxer, so `dshow` is the route.
3. The multiviewer is an exact 4×4 grid: each source is 480×270 at scale 0.25, so
   the chart's 120px column grid lands on 30px columns.

**Do not re-derive these values from a PNG snapshot of the channel.** PNG stores
straight alpha by specification, so `ADD 1 IMAGE` un-premultiplies on the way out
and every sample it writes looks straight regardless of what leaves on SDI.
Reading `C=255` beside `a=179` in such a file evidences the file format and
nothing else. An earlier revision of the level ladder was rebuilt on exactly that
misreading and left `KEY ONLY` sitting 41 levels off the bed on the fill signal.
The authority is a scope or a multiviewer on the real outputs.

A channel snapshot is still useful for fine detail the multiviewer cannot
resolve — the ident line is 13px tall, which is 3px in a tile — provided the
premultiplied fill is reconstructed as `C × a / 255` from the straight values.
