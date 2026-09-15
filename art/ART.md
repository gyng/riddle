# Riddle — art bible

This file is the authority on *look*. `art/manifest.json` lists every keyed asset and its
drawable brief; `art/make_tiles.py` is the authority on environment tiles. Nothing here
overrides game rules (`docs/CUT1.md`). Derived from `../tacticalswap/docs/ART.md` (the
two-register rule) and `research/art-tech.md` §4–5 (two-density compositing).

## Direction

**Paper cut-outs on a pixel diorama.** The dungeon is an 8×8-texel, ≤ 8-colour pixel world
(Downwell register: flat fills, Bayer dither, everything has a dark edge). The hero, monsters,
bosses and summons are **loose 1980s watercolour-anime key art** stamped onto that world at
twice the environment's pixel density. The mismatch is deliberate and framed as such
(Paper Mario): ink outline dilated to one env-texel, contact shadow under every sprite,
snapped movement at 8–12 fps. If it reads as intent it is style; if it is half-hidden it is a
bug — so we never soften either register toward the other.

### Register 1 — keyed character sprites (codex, `art/generated/`)

Loose transparent watercolour and gouache washes **inside bold, clean, confident dark ink
contour lines** — the anime cel principle. The wash may bleed past or stop short of the line;
the line itself is deliberate, dark and continuous around the outer silhouette and the major
interior forms.

Riddle-specific readability rules (hard):

- **Target size is tiny.** A hero reads at **48 px tall** on a phone; monsters at **32–48 px**;
  bosses ~64 px; summons ~24 px. Judge every sprite at that size, not at paint size.
- **Silhouettes are simple.** One big readable mass, one gesture, 3–4 signature details, drop the
  rest. No dense interior rendering; large simple wash shapes, white/ivory paper in every lit
  area, ONE flat indigo-umber shadow mass on the shadow side.
- **Ink line is 4–8 px at 1024 px generation size** (≈ 1 px at 48 px after the box downscale;
  `pack.py` then dilates it by 2 px on the 2× master so it survives as ≥ 1 env texel).
- **The tag lives in the silhouette.** Each monster's learnable behaviour must be visible in its
  outline at 48 px: archer *holds a drawn bow*, conjurer *raises a staff with floating blades*,
  bloat is *a swollen sac*, jelly is *a translucent blob*, monkey *clutches a stolen bag*, ogre is
  *huge and wide*, eel is *a long S-curve*, wraith is *tattered and floating*, captive is
  *in chains*, jackal is *low and lean*, ghoul is *hunched and clawed*, skeleton is *all bone*.
  Bosses are **larger and crowned/marked** (crown, skull-crown, matriarch's spines).
- **Upper-left key light, always.** Lit side near paper-white; shadow side one flat wash.
- **`#0000FF` chroma key** background, flat, edge to edge. Subject far from pure blue: any blue
  material uses **cyan/teal mid-tones (high green) and blue-black depths (all channels low)**;
  every pixel inside the silhouette must sit ≥ 150 RGB-distance from (0,0,255). Ultramarine
  and royal blue are forbidden on the subject.
- **No floor, no contact shadow, no cast-shadow pool, no scenery, no glow halo, no text.** The
  renderer owns grounding and light.
- One subject per sprite, centred, ~10 % margin, full body, nothing cropped. Facing **right**
  (the renderer flips for left).
- Unmistakably ANIME for the two heroes and the captive: expressive anime faces, large eyes,
  hair as flowing masses, 1980s OVA character design. Monsters are anime-bestiary creatures,
  not D&D illustration or western storybook.

Style vocabulary for prompts (never name living artists; describe the look):

- "loose transparent watercolour and gouache on cold-press paper, washes left wet and unblended"
- "pigment blooms, granulation, dry-brush edges, white paper showing through"
- "bold clean confident dark ink contour lines, anime cel principle, flat stylized shapes"
- "1980s Japanese fantasy anime key visual / OVA box art"
- "chiaroscuro: single key light upper-left, shadow as one flat indigo-umber wash"
- "muted field, jewel-bright accents"
- Never: text, logos, signatures, watermarks, photorealism, airbrushed volume, dense noisy
  detail, 3D-render shading, cel-shading gradients, modern digital gradients, game-asset gloss,
  pixel art (sprites are painted; the *environment* is pixel art).

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

Sprites are tinted 40–60 % toward the active biome palette at runtime (tint LUT, not a hard
quantise). So sprite hues must be **clean** (not muddy mid-greys) and values **high-contrast**
(paper-white lit side, deep shadow mass); a muddy sprite becomes invisible after tinting.
`pack.py` also pre-quantises each master to ≤ 32 colours (median cut, no dither); the biome ramps ship in `atlas.json` `meta.palettes` for the runtime tint pass.

## Readability constraints (summary)

- Read at 48 px (hero) / 32–48 px (monster) on a phone. Composite over the real tiles before
  accepting.
- Tag visible in silhouette. Bosses bigger and crowned/marked.
- Ink outline continuous, dark, 4–8 px at 1024 px.
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
