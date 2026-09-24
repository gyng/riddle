# Riddle — art bible

This file is the authority on *look*. `art/manifest.json` lists every keyed asset and its
drawable brief; `art/make_tiles.py` is the authority on environment tiles. Nothing here
overrides game rules (`docs/CUT1.md`). Derived from `../tacticalswap/docs/ART.md` (the
two-register rule) and `research/art-tech.md` §4–5 (two-density compositing).

## Direction

**Painted sprites on a dressed pixel diorama** (revised 2026-09-24, the art pass against
`art/ui/targets/watch.png`). The dungeon is a 16×16-texel, ≤ 8-colour-per-biome pixel world
(register 3, below) dressed with torches, barrels, crates, banners, moss and blood, lit by torch
pools. The hero, monsters, bosses and summons are **crisp painted RPG character sprites with a
watercolour-inspired fill texture**, drawn at twice the environment's texel density with a dark
ink ring and a contact shadow.

Why the change (keyed register v1 → v2): the v1 register was loose 1980s watercolour-anime key
art, box-downscaled to 32–64 texels. The user judged the result "just too far" from the target
mockups: at game size the washes turned to mush, the thin interior line work vanished, the pale
paper-white lit side read washed-out against the dark dungeon, and the tall anime proportions put
the readable part (the face and weapon) in a few texels. The target's sprites are chunky, crisply
outlined and high in value contrast; v2 keeps a watercolour *texture* inside the fills (so the set
still has a painted, hand-made surface, and the tint LUT still has clean hues to work with) and
takes everything else — proportions, outline, shading, palette — from the target. Manifest ids,
sizes (`master_h`/`texel_h`) and the pack pipeline are unchanged.

### Register 1 — keyed character sprites, v2 (codex, `art/generated/`)

A **crisp top-down dark-fantasy RPG character sprite**, like the characters in `watch.png`,
painted at high resolution:

- **Proportions:** chunky — the head about one third of the height, stocky torso, short planted
  legs, hands and weapon drawn LARGE; 3/4 view from slightly above, facing **right**.
- **Outline:** one thick, clean, continuous **near-black** outline round the silhouette (12–16 px
  at 1024 px ≈ 1 sprite texel after the downscale; `pack.py` still dilates a 2 px ink ring on
  the master), thinner dark lines on the major interior forms. Hard edges; nothing feathered.
- **Shading:** three tones per material (lit / mid / one shadow), key light **upper-left**,
  strong value contrast. Inside each tone a subtle watercolour wash texture (granulation, pigment
  variation, a few dry-brush flecks) — never a smooth digital gradient, never a loose wet wash, no
  white paper showing.
- **Palette:** grounded earthy dark fantasy — moss/olive greens, crimson and oxblood cloth, warm
  leather, dull steel, bone ivory — with one or two saturated accents; clean hues (the runtime
  tints sprites 30 % toward the biome ramp).
- **Faces:** small and simple (two dark eyes, a brow); heroes keep a gentle anime flavour (hair as a
  few big locks), monsters grotesque-but-charming like the target's goblins.

Riddle-specific readability rules (hard, unchanged):

- **Target size is tiny.** A hero reads at **48 px tall** on a phone; monsters at **32–48 px**;
  bosses ~64 px; summons ~24 px. Judge every sprite at that size, not at paint size.
- **Silhouettes are simple.** One big readable mass, one gesture, 3–4 signature details.
- **The tag lives in the silhouette.** Each monster's learnable behaviour must be visible in its
  outline at 48 px: archer *holds a drawn bow*, conjurer *raises a staff with floating blades*,
  bloat is *a swollen sac*, jelly is *a translucent blob*, monkey *clutches a stolen bag*, ogre is
  *huge and wide*, eel is *a long S-curve*, wraith is *tattered and floating*, captive is
  *in chains*, jackal is *low and lean*, ghoul is *hunched and clawed*, skeleton is *all bone*,
  the captain *raises a war horn*. Bosses are **larger and crowned/marked**.
- **`#0000FF` chroma key** background, flat, edge to edge. Subject far from pure blue: any blue
  material uses **cyan/teal mid-tones (high green) and blue-black depths (all channels low)**;
  every pixel inside the silhouette must sit ≥ 150 RGB-distance from (0,0,255).
- **No floor, no contact shadow, no cast-shadow pool, no scenery, no glow halo, no text.** The
  renderer owns grounding and light. One subject, centred, ~10 % margin, full body.

Style vocabulary for prompts (never name living artists or games; describe the look; the full
preamble is `make_prompts.py` HEADER):

- "crisp top-down dark-fantasy RPG character sprite, chunky proportions, head a third of the height"
- "one thick clean continuous near-black outline, hard crisp edges"
- "three tones per material, key light upper-left, a subtle watercolour wash texture inside each tone"
- "grounded earthy palette: moss greens, crimson cloth, warm leather, dull steel, bone"
- Never: text, logos, signatures, watermarks, photorealism, 3D-render shading, airbrushed volume,
  glossy game-asset rendering, dense noisy detail, stair-stepped fake pixel art, loose
  watercolour-anime illustration (the v1 register).

### Register 3 — the 16×16 environment (codex pixel art → ramp index, `art/make_env.py`)

The art pass's environment, drawn over register 2 wherever it is loaded (register 2 stays the
fallback and the source of situation props and item glyphs). Codex paints each tile/decal/prop as
a tiny pixel-art tile enlarged to 1024 px (manifest `bg: env` opaque tile, `env_keyed` on
`#0000FF`); `make_env.py` box-downscales it to its `texels` box (16×16; torch 16×24; banner
16×20), cuts luminance at **fixed quantiles** into a per-class ramp-index list (so the four floor
variants share one value distribution and tile together, and every biome gets the drawing in its
own 8-colour ramp; `BIOME_REMAP` re-roles the sanctum's pale field and the foundry's oranges) and
writes `<biome>_env_<name>.png`. Blood, torch flames and banners are **hue assets** (`env_<name>`,
≤ 8 colours, one file for every biome) drawn sprite-tagged so red and flame keep their hue. Tiles
are still 8×8 *world* quads — two texels per env texel, the sprite density.

Classes (index lists darkest→lightest): floors `1 2 3 4` (field on 2, lit edges rare), wall
face `0–5` (lit capstone ledge over courses), wall top `1 2 3` (the renderer rims each side that
meets open floor), door = portcullis `0–5`, stairs, water, chasm; decals moss/crack/rubble;
props barrel/crate/pot/bones with a ramp-0 rim.

Renderer contract (`web/src/render/index.ts`, render-only, seeded by tile position): a wall
whose tile below is seen open ground draws its **face**, any other wall its **top**; floors pick a
variant by hash; torches on every ~5th face over floor (2 frames, 180 ms), banners on a few;
barrels/crates/pots in room corners; moss (more near walls), cracks, rubble, blood, bones as
decals. Torches feed up to 12 lights to the blit: a value lift before the quantise (the pool
climbs the ramp through the dither) and a small warm tint after it; away from a torch world
tiles sit at 0.78 (a ramp step down) and are not dithered.

### Register 2 — environment tiles (hand-authored pixel art, `art/tiles/`)

8×8 texels, ≤ 8 colours from the biome palette, drawn procedurally-but-designed by
`art/make_tiles.py` (no random noise: every texel is placed by rule, dither is ordered Bayer).
Downwell-style: mostly flat fills with a little dither, **1-px dark rim on every wall** so tiles
and sprites share the "everything has a dark edge" rule. Floors are quiet and low-contrast (they
must never compete with a sprite); walls carry the rim and a brick/rock texture; water and chasm
are the two "special" floors and read by value (water lighter and dithered, chasm near-black).
Overlays `gas`/`fire` are 2-frame 8×8 sprites with alpha; item glyphs are 8×8 single-object
silhouettes with a dark edge.

Biome palettes (8 colours each; index 0 is the darkest, 7 the lightest):

| biome | mood | ramp |
|---|---|---|
| warrens | olive / umber / bone — dry earth, goblin burrows | `#14120d #2e2a1c #4a4326 #6b6a2f #8c7a3c #b09a5a #d4c58a #efe6c0` |
| fens | teal / moss / slate — wet, gas, cold | `#0c1416 #1a2b2e #24443f #2f6a5a #4d8a72 #6f9f8a #9dbfa8 #d6e6da` |
| crypt | indigo / ash / bone — undead, still, cold | `#0b0a14 #1c1a30 #33304f #4f4d6d #77738c #a39fae #d3cfc9 #f1ede0` |

### Palette unification (runtime)

Sprites are tinted 30 % toward the active biome palette at runtime (`blit.ts` `uTint`; a blend,
not a hard quantise). So sprite hues must be **clean** (not muddy mid-greys) and values
**high-contrast** (lit tone vs one deep shadow tone); a muddy sprite becomes invisible after tinting.
`pack.py` also pre-quantises each master to ≤ 32 colours (median cut, no dither); the biome ramps ship in `atlas.json` `meta.palettes` for the runtime tint pass.

## Readability constraints (summary)

- Read at 48 px (hero) / 32–48 px (monster) on a phone. Composite over the real tiles before
  accepting.
- Tag visible in silhouette. Bosses bigger and crowned/marked.
- Ink outline continuous, near-black, 12–16 px at 1024 px (v2; v1 was 4–8 px).
- Chroma: corners exactly `#0000FF`; zero near-key (RGB-distance 30–130) pixels inside the
  subject; blue-black depths and teal mid-tones only.
- No floor / shadow pool / text / glow.

## Technical

- PNG, sRGB. Generation size: units **1024 px on the short edge** (1024×1024 square, or
  1024×1536 portrait for tall subjects); title key art **1536×1024**, no text.
- One file per manifest id at `art/generated/<id>.png`; filenames match ids exactly. No
  unregistered PNG may remain there; rejected work goes to `art/archive/superseded/`.
- `pack.py`: chroma-key → RGBA, crop to alpha bounds, box-resample to the master height
  (hero 96, monsters 64–96, bosses 128, summons 48 — 2× the runtime sprite-texel height so the
  renderer downscales nearest by 2 or uses as-is), dilate the outline 2 px and darken, quantise,
  then pack sprites + tiles + overlays + glyphs into `web/public/art/atlas.png` +
  `atlas.json` (`{ "frames": { id: {x,y,w,h} } }`, 1-px gutters). Title → `web/public/art/title.png`.
- Missing art must never block the game: the renderer keeps primitive fallbacks per id.

## Provenance

All generated output is treated as original work. Prompts describe era/medium/look only — no
artist names, no franchise names, no copyrighted characters, no real people.
