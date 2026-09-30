# Riddle — art direction: moonlit ink and wash (phase 1, for the owner's approval)

*2026-09-30. The owner's call: a **hybrid pixel–painted** look, leaning a little more **painted**, more **stylistic**,
consistent across sprites, tiles, props, portraits, UI, backdrops and the town. References (for people, never in a
prompt): **Vampire Hunter D: Bloodlust** (gothic drama, moonlit blues and blood reds, long elegant silhouettes, deep
shadow, a fine elegant line) × **Downwell** (a stark limited palette, high contrast, bold readable shapes, one signature
accent) × **1980s anime-magazine watercolour** (washes, paper grain, colour bleed, hand-inked edges, print halftone).
This file governs *look*; `art/ART.md` keeps the pipeline, sizes and readability rules (48 px hero, tag in the silhouette,
chroma key). Phase 2 (regenerating the library) waits for approval. Targets: `art/ui/targets/style/` (§11).*

## 1. The idea in one line

**Ink and moonlight on paper, blocked in on a pixel grid; blood is the only colour that shouts.** The blind raters'
ceiling ("pixel art next to a painted UI", ~6 on every dungeon moment) came from two registers meeting; this guide makes
them one: *everything* — sprite, tile, frame, portrait — is ink line + transparent wash + paper, and *everything* sits on
a visible pixel grid. The painted part is the surface; the pixel part is the structure.

## 2. The palette (exact; shared by `art/art-qc.py --style` and `web/src/render/wash.ts`)

| name | hex | role |
|---|---|---|
| INK | `#0d0c14` | outlines, the deepest shadow, the void (blue-black, never neutral black) |
| UMBRA | `#1c1b2b` | shadow |
| DUSK | `#2b3350` | mid-shadow (takes the place's tint) |
| MOON | `#4d6c99` | the moonlit mid-tone (takes the place's tint) |
| MIST | `#a4bcd6` | moonlit highlight, rims, fog, ghosts, slash arcs |
| BONE | `#eadfc5` | paper, skin light, bone, the moon, text on dark |
| BLOOD | `#c01530` | **the signature accent**: the hero's cloak, blood, danger, the one primary button |
| CLOT | `#5c0b1c` | dark blood, the accent's shadow, the death banner's folds |
| EMBER | `#e8923a` | a flame and its small pool only (torch, forge, candle) — never a room's cast |
| GILT | `#b89448` | worn gold trim on metal and the "recommended" edge — sparingly |

**Per-place tint rule**: a place names one tint; **only DUSK and MOON lean toward it** (DUSK 60 %, MOON 40 % toward
tint × 1.7). INK, UMBRA, MIST, BONE, BLOOD, EMBER never move, so every place is recognisably the same world.
Warrens `#4a3b2c` umber · Burrows `#5a4527` ochre · Fens `#2d5752` teal · Crypt `#2f2c4f` indigo · Foundry `#5a2a1e` rust ·
Deep `#1f2e4f` navy · Sanctum `#6b6048` pale gold · Town `#3a4a3a` moss. A monster may carry one local hue (a goblin's
DUSK-green skin, a slime's teal) inside the tint's family; a second saturated hue is a phase-2 exception to argue for.

## 3. Values and contrast

- **~60 % of a frame is INK/UMBRA.** Light is a *shape*: a moon shaft, a torch pool, a rim — not an ambient fill.
- Every frame spans INK to MIST (L* p5 ≤ 14, p99 ≥ 62). Mid-tones carry the place; the extremes carry the drama.
- **No warm overall cast** (mid-tone b* ≤ +12). Warmth lives inside EMBER pools only; the old umber/amber world goes.
- **BLOOD ≤ ~8 % of a frame** (QC bar 12 %), present on every frame that has a hero or a danger (≥ 0.2 %).
- Value separates actor from ground: a sprite's lit side is ≥ 2 value steps off the floor under it; its INK line does the rest.

## 4. Line and edge — how "pixel" survives

- **Ink**: one blue-black INK contour round every silhouette, thicker on the shadow side, thinner on the lit side; a few
  fine hatching strokes in shadows. On sprites the contour is ≥ 1 sprite texel (12–16 px at 1024, as ART.md v2).
- **The grid**: forms are blocked in chunky square clusters with **stepped edges** — the pixel grid is visible at a
  glance. Sprites stay at the current density (hero `texel_h` 48, `master_h` 96; tiles 16×16 on the sprite grid).
- **Inside the grid, paint**: each cluster is a transparent wash, not a flat pixel colour — granulation, pigment pooling
  darker at the wash's edge, a little bleed across a boundary. Tiles are **painted washes on a pixel grid**: 16×16, ≤ 24
  colours (the painted register's cap), the wash kept by the downscale rather than quantised to a ramp.
- Silhouettes: **long and elegant** (hero head ≈ 1/6.5 of height, long trailing cloaks and coats) but one bold mass
  with one gesture — still readable at 48 px. The hero sheet's 48 px read (sheet.png) holds the cloak and sword but loses
  the face: phase 2 keeps a BONE face patch ≥ 2 texels and a MIST rim on the sword so the long figure still reads.
  This moves the hero away from ART.md's chunky 1/3-head proportion (§12).

## 5. Texture

- **Paper grain**: warm BONE paper shows through the lightest washes; a fine grain over the whole frame (screen-fixed,
  like print). **Watercolour bloom**: soft blotches and edge darkening in mid-tones. **Halftone**: a 45° dot screen
  only inside the shadow band (not the deepest INK, never on lit ground). Texture must never beat an actor: it lives in
  the ground and the frame, at low amplitude on sprites.

## 6. Light

Cold **moonlight from above** (MIST rims on top edges, shafts through cracks and windows) is the key; small **EMBER torch
pools** are the warm counterpoint and the only warm light. The hero keeps a small light of his own (MIST, not amber).
Ghosts and magic glow MIST; fire and forge glow EMBER; nothing glows BLOOD except a hit flash.

## 7. UI, frames, type

- Iron and stone become **ink and wash**: INK plates, MIST moonlit bevel on the top edge, GILT rivets, hatching for
  hammered texture — never glossy metal. Parchment is BONE paper with bloom stains and an inked, deckled edge.
- The primary gem is **BLOOD** (was amber); secondary gems MOON; danger gauges BLOOD, health MIST.
- Icons: BONE ink drawings on dark tiles, one weight, no gradients.
- Type unchanged in role: a tall carved gothic/serif display face (Cinzel / IM Fell) in BONE for titles and verdicts, the
  condensed sans for numbers, the pixel face for in-world callouts. The copy budget (≤ 3 words) is untouched.

## 8. The Codex preamble (every future asset prompt starts with this block, verbatim)

The full text is the `STYLE PREAMBLE` block of `art/prompts/style_targets.txt`; phase 2 moves it into `make_prompts.py`
`HEADER` (replacing the v2 sprite style paragraph, keeping ART.md's chroma-key and readability paragraphs after it). Its
parts: gothic moonlit hybrid leaning painted · late-1980s anime-magazine watercolour printed on paper, over a coarse
visible pixel grid · washes with granulation, pooling, bleed, paper grain, halftone in the deepest shadows · hand-inked
blue-black contour, thick shadow side · elegant long silhouettes, one bold readable mass · the palette by name and hex ·
the tint rule · 60 % shadow, moonlight above, EMBER pools, BLOOD ≤ 8 % · NEVER: text, logos, photorealism, glossy 3D,
airbrushed gradients, lens flare, a warm brown/amber cast, rainbow saturation, noisy detail, flat vector, chibi.
Prompts describe the look only: no artist, film or game names (ART.md provenance).

## 9. The renderer prototype — `?look=wash`

`web/src/render/wash.ts`, off by default, sticky via localStorage `riddle.look` (`?look=off` clears). Canvas: the blit's
last step (`LOOK` define, compiled only when FX > 0: the `low` tier's shader is byte-identical): the frame onto the §2 ramp
with the place's tint after a night curve (luminance^1.3: the mid-tones sink; a fifth of the painted hue kept; saturated reds and flame keep theirs), 30 % toward six value bands,
an INK line on source-luminance edges, wash edge-darkening/bloom from a 3-texel neighbourhood, world-anchored granulation
on the sprite-texel grid, screen paper grain, BONE in the lights, the halftone in the shadow band. ~9 texture reads per
device pixel, no extra pass. Chrome: an SVG gradient-map filter (the same ramp, reds kept) on the DOM chrome plus a paper
grain + bloom overlay (multiply). Headed GPU, 400×800 @2×, fx high: rAF p50/p95 16.7/16.8 ms on the Warrens, Fens, boss
and death, the same as without it. It is a *direction check*, not the phase-2 look: it cannot redraw a chunky sprite long,
turn an amber gem red, or give a tile a wash it never had — phase 2's art does that.

## 10. Reviewing consistency

**Automated** — `python3 art/art-qc.py --style <png>…` (the place's tint read from the file name): hard fails on contrast
(L* p5 ≤ 14, p99 ≥ 62), warm cast (mid-tone b* ≤ +12), the blood budget (0.2–12 %), off-palette hues (chroma > 22 outside
±24° of the palette's hues and the tint: ≤ 15 %); warns on shadow share (< 40 % on a frame) and mean ΔE to the palette
(> 14). On 2026-09-30 every style target passes; `targets/watch.png` and round 26's Warrens watch fail cast (+18, +17)
and contrast — the check tells the two directions apart. Phase 2 runs it on every sprite composited over its biome floor,
every tile sheet and every UI frame, and adds it to `art-qc.py`'s default run for new-register ids.

**By eye** (every asset, at game size over the real floor, beside a style target):
1. Does it read in one glance at 48 px (hero) / 32–48 (foe) — one mass, the tag in the silhouette?
2. INK contour closed; thick side away from the moon?
3. Pixel grid visible (stepped edges) *and* washes inside the clusters (not flat fills, not a smooth gradient)?
4. Only palette names + the place's tint? BLOOD only where it means something? No warm brown cast?
5. Lit from above by the moon; any warm light has a visible EMBER source?
6. Paper/grain/halftone present but quieter than the actor?
7. Same family as its neighbours: place a new piece in the contact sheet beside the six targets — the odd one out loses.

## 11. Style targets (Codex, `art/ui/targets/style/`, prompts `art/prompts/style_targets.txt`)

`watch_warrens` · `watch_fens` (a fight, two places, one frame) · `boss` (the Warlord, a crypt hall) · `death` (YOU DIED,
three patch strips, a BLOOD APPLY gem) · `hero_sheet` (the warrior: man / woman / cat-folk, 48 px read copies, a portrait)
· `ui_sheet` (bar, console, parchment, tiles, the BLOOD SEND gem, gauges, icons) · `town` (the hub at night). Contact
sheet with the prototype's before/after: `scratchpad/style/sheet.png`.

## 12. What changes in phase 2 (for the owner to approve)

1. The bestiary and heroes redrawn under §8 (~70 sprites; the long-silhouette hero replaces ART.md's 1/3-head rule, the
   48 px read kept). 2. The painted tile register (105 pieces) redrawn in §2 with the tint rule. 3. UI frames, icons, gems
   (amber → BLOOD) and backdrops; portraits. 4. The town set (`docs/TOWN.md`) drawn in this style from the start.
5. The renderer's grades (`blit.ts GRADES`, torch colours) re-tuned to moonlight, the wash pass kept only for paper grain
   and halftone once the art carries the palette itself. Do/don't, in short — **do**: moon from above, INK everywhere, one
   BLOOD thing per frame, visible grid, washes inside it. **Don't**: amber rooms, glossy metal, smooth gradients, a second
   accent colour, texture louder than the actor, text in art.
