# art/ — Riddle art pipeline

`art/ART.md` is the style authority (two registers: keyed watercolour-anime sprites, hand-authored
8×8 pixel tiles). `art/manifest.json` lists every keyed asset and its brief.

## Pipeline

```text
manifest.json ──make_prompts.py──▶ prompts/batchN.txt ──codex exec──▶ generated/<id>.png
make_tiles.py ───────────────────────────────────────────────────────▶ tiles/<biome>_<tile>.png, <biome>_bones_0/1, gas_*, fire_*, item glyphs, _sheet.png
review.py      per-sprite QC (corners, flood-fill near-key, band census) + qc/review.png at runtime size over real tiles
pack.py        key → crop → box-resample to master_h → 2 px ink dilate+darken → quantise ≤32 → shelf-pack
               ▶ web/public/art/atlas.png + atlas.json  { "frames": { id: {x,y,w,h} }, "meta": {...} }  and title.png
art-qc.py      shipping gate (drift, dimensions, gutters, key spill, outline, tile colour counts, freshness)
```

Re-pack from scratch:

```sh
python3 art/make_tiles.py && python3 art/pack.py && python3 art/art-qc.py
```

Regenerate a sprite (example: one retry batch of two ids):

```sh
python3 art/make_prompts.py 5 wraith captive        # writes art/prompts/batch5.txt
mkdir -p art/logs && setsid nohup codex exec --cd /home/g/p/riddle --skip-git-repo-check \
  --sandbox workspace-write -c approval_policy="never" - < art/prompts/batch5.txt > art/logs/batch5.log 2>&1 &
# poll `ls -la --time-style=+%T art/generated` until mtimes are quiescent, then:
python3 art/review.py wraith captive && python3 art/pack.py && python3 art/art-qc.py
```

Runtime contract (renderer): sprite frames are `master_h` px tall = **2× the runtime sprite-texel
height** (`meta.sprites[id].texel_h`); draw at half size with nearest sampling (option C, two densities
in one target) or as-is on a 1× target. Tiles are 8×8 keyed `<biome>_<tile>`; overlays `gas_0/1`,
`fire_0/1`; item glyphs `potion scroll weapon armour gold bones`; props `<biome>_bones_0/1` (alpha,
drawn in the biome ramp, frame 1 = glint), `<biome>_shrine_0/1` (frame 1 = light lifted one texel),
`<biome>_vault` / `<biome>_vault_open`, `<biome>_nest_0/1` (frame 0 eyes closed, 1 open) — all alpha
props over floor. `meta.palettes` carries the six 8-colour biome ramps
(`warrens fens crypt foundry deep sanctum`) for the palette/tint pass plus `boss_flash` (crimson→white-hot, same 8-index shape; CUT2 §7 one-shot
palette flash on boss sight — no tiles are authored in it).

## Operational rules (paid for in blood, tacticalswap + this round)

- Run every codex batch **detached** (`setsid nohup … &`); a session memory monitor kills long
  foreground background tasks. Up to three batches concurrently.
- Codex file writes are **not synchronised with exit**: poll `art/generated/` mtimes to quiescence.
- Codex's built-in `image_gen` returns transparent-background PNGs by default even when asked for a
  flat `#0000FF` fill, and its self-check then "retries once with the blue emphasised". Both are fine:
  `pack.py` accepts authored alpha (transparent corner) or a blue key. What is NOT fine is a
  flatten-to-black (`RGB`, corners `(0,0,0)`): `review.py` flags it (`corners=False`) and it must be
  regenerated.
- Codex leaves `_inspection_*.png` scratch files in `art/generated/` during self-checks; they are
  gitignored and `art-qc.py` treats any unregistered PNG there as a failure — delete before packing.

## Automated shipping QC

`python3 art/art-qc.py` after every pack. It fails on manifest/source/atlas drift, unregistered
sources in `art/generated/`, keyed sources under 1024 px short edge, wrong frame heights, missing
gutters/overlaps, residual pure-key pixels, blue semi-transparent spill, implausible alpha, lost
outlines (bright edge pixels), tiles over 8 colours, and a stale atlas. Subjective gates
(too rendered, no ink line, tag unreadable at 48 px) are judged on `art/qc/review.png`.

## Per-asset notes (round 1, 2026-09-16 — 21 keyed ids + title, 22/22 accepted)

Four codex batches (`prompts/batch1..4.txt`, 6/6/6/4 ids, three concurrent then the fourth),
model `gpt-6-astra` via the built-in `image_gen` tool. ~6 min per batch; all finals landed
within a minute of exit. Every batch first returned transparent-alpha PNGs, and codex itself
regenerated them "with the blue emphasised" — final sources are all opaque `#0000FF`, all
1024×1024 (title 1536×1024). Codex also self-corrected one asset with lettering (captive) before
saving. Zero retries were needed from our side. QC: `review.py` corner exactness (24/24 exact),
flood-fill over near-key, band census, then the runtime-size composite sheet `qc/review.png`
over the warrens/fens/crypt floor+wall tiles, judged by eye.

Reading of the flood-fill numbers: `enclosed_pure` is pure key inside genuine silhouette
openings (bow-and-arm gap on the archer = 128k px, warlord's raised arm, conjurer's staff arms)
and keys to transparent — correct. `enclosed_near` is the anti-aliased ramp around those same
openings; nothing on any subject sat in the 30–130 band, because every prompt spelled out the
colour science (teal mid-tones, blue-black depths, ≥ 150 floor).

- **hero_fighter / hero_rogue** — first try. Unmistakably 80s OVA (large eyes, hair masses);
  fighter's round shield + short sword and the rogue's low diagonal crouch are distinct at 48 px
  on all three biomes. Fighter's 15k enclosed-pure px = the gap between shield and body.
- **rat / jackal** — first try. Horizontal creatures: at texel_h 32 the rat is 64 sprite-texels
  wide (= 4 tiles); the renderer may draw rats at 0.75 scale if that crowds corridors. Jackal's
  stretched-dash-with-ears silhouette reads "fast".
- **goblin / goblin_archer / goblin_conjurer** — first try. Cleaver / drawn bow (arc wider than
  the body) / raised staff with two hovering ivory blades all read at 40 px; the three are
  distinct from each other by silhouette alone.
- **monkey** — first try; the clutched ochre sack and the question-mark tail are the two things
  that survive the downscale, which is exactly the tag.
- **ogre** — first try. Broad mass, club overhead. Dull grey-umber as briefed; the darkened
  inner edge from `pack.py` keeps it separated from the warrens floor.
- **bloat / pink_jelly / eel** — first try. Sac with tendrils, translucent dome, S-curve: the
  three "no-face" monsters are the most legible sprites in the set at 32–40 px.
- **skeleton** — first landing was an RGB flatten onto BLACK (corners (0,0,0)); codex's own
  retry replaced it with a blue-keyed source before exit. Gappy ribcage reads as "all bone".
- **ghoul / wraith** — first try. Ghoul leans forward with two claw shapes leading; wraith has
  no feet and a torn hem. Both are low-value sprites; watch them on the crypt palette after the
  runtime tint — the 2 px ink ring is what keeps them off the wall.
- **captive** — first try (after codex's internal text fix). Kneeling, manacled, chain across the
  wrists and pooling at the knees. At 48 px the chain reads as a dark band at the wrists plus the
  kneeling pose; acceptable, but if playtest says "who is that", the retry wording is "chain as
  ONE thick dark loop crossing the whole figure".
- **boss_goblin_warlord / boss_bloat_mother / boss_lich** — first try. Iron crown + red banner;
  spine crown + budding spawn + maw; skull crown + skull staff + teal-white flame. All clearly
  "the big crowned one" next to their biome's rank-and-file at 64 px.
- **spectral_blade / spectral_hound** — first try. Blade packs to 20×48 (a vertical sliver — by
  design); hound's wisping hindquarters survive at 24 px.
- **title** — first try, no text. Fighter and rogue from behind at a stair mouth, warren stone
  giving way to teal fen light and an indigo crypt hall below. Note the key-art heroes are
  looser interpretations than the sprites (long blue-black hair, blond rogue); acceptable for a
  splash, regenerate if the camp screen shows both side by side.

`pack.py`: 21 keyed masters (heights 48/64/80/96/128 per manifest `master_h`) + 33 tiles →
54 frames in a 512×382 `atlas.png`; `title.png` 1536×1024. `art-qc.py` PASS.

## Per-asset notes (round 2 / Cut 2, 2026-09-16 — 2 keyed ids + bones prop, 2/2 accepted)

One codex batch (`prompts/batch5.txt`, 2 ids, log `logs/batch5.log`), same `gpt-6-astra` /
`image_gen`, ~12 min wall clock (the first ~10 min is codex reading its image skill; both finals
landed together and codex exited ~2 min later). Zero retries. Codex resampled both to 1024×1024
(Lanczos) and ran its own corner/near-blue self-check before saving; opaque `#0000FF` sources.

- **hero_ranger** — first try. Hood up, long braid, full quiver over the right shoulder, longbow
  held out front with a nocked arrow; deep green + leather. At 48 px the bow arc plus the hood
  and cloak triangle read "ranger" on all three biomes; the quiver survives as a bump above the
  shoulder. `enclosed_pure` 62k px = the bow-and-arm openings (same as goblin_archer);
  `enclosed_near` 1132 is the ≤ 3 px anti-aliased fringe around those openings (1124 of 1132
  within 3 px of enclosed pure; 8 px of true subject spill in a 1 M px image). Packs to 69×96.
- **hero_caster** — first try. Soft hood over long pale flowing hair, layered dusty violet +
  ivory robes with a wide hem, tall gnarled staff with a single ember-orange crystal painted as
  a flat jewel (no glow), open left hand at the hip, no hat, no beard. At 48 px the staff's
  orange tip and the robe column read "caster"; hue-adjacent to the fighter's crimson tunic but
  the silhouettes (shield bulk vs staff column) never confuse. `enclosed_near` 486 → 468 fringe,
  18 px spill. Packs to 54×96.
- **bones prop** (`make_tiles.py`) — `<biome>_bones_0/1` are ramp-indexed 8×8 with alpha
  (drawn on nothing, so they sit over floor or floor_alt): skull upper-left (ramp-7 crown, two
  ramp-1 sockets, ramp-6 tooth row), one long bone diagonal to the lower-left, a chip at the
  bottom-right, ramp-0 edge throughout. Frame 1 adds one ramp-7 sparkle texel above the bone and
  lifts the near socket to ramp-3 — a 2-frame "glint" the viewer can flip at ~2 Hz. 4 colours per
  frame. The generic `bones` item glyph is frame 0 in the item register's own bone/ink palette.
  Checked at 14× and at 2× over floor/floor_alt on all three ramps (`tiles/_sheet.png`, "over
  floor" rows now carry `bones`, `bones_0`, `bones_1`).
- **boss_flash** — a fourth ramp in `atlas.json` `meta.palettes` (crimson-black → white-hot),
  same 8-index shape as a biome ramp so the tint pass can swap to it for the CUT2 §7 one-shot
  flash. No tiles are authored in it; `make_tiles.build()` iterates `PALETTES` only.

`pack.py`: 23 keyed masters + 40 tiles → 63 frames in a 512×366 `atlas.png`. `art-qc.py` PASS
(63 frames, 0 warnings, 0 failures). `qc/review.png` regenerated for all 23 ids (heroes are rows 1–4).

## Per-asset notes (round 3 / Cut 3, 2026-09-16 — three tile sets)

`make_tiles.py` now carries six biome ramps and per-biome overrides for *any* tile (`tpl()`:
a template is a row list for every biome or `{biome: rows, "*": default}`); dither codes moved
to a `DITHER` table. `_sheet.png` is three rows per biome (tiles, over-floor, a 5×4 sample room
with every special floor + a bones pile) — the old single "room" row overlapped its rooms.

| biome | ramp (0 darkest → 7 lightest) | what each index is for |
|---|---|---|
| foundry | `#120b08 #2c1a12 #4e2a18 #8a3f1c #5b5a5e #c2622a #9a9598 #f4c040` | 2 dark-rust floor, 3/5 oranges, 4/6 iron greys, 7 the one hot yellow |
| deep | `#030306 #0c0e1a #181b30 #1e2340 #2c3560 #5a5f78 #b8b09a #ece4cc` | 2 near-black floor, 3/4 blue-black walls, 6/7 bone highlights |
| sanctum | `#1a1c24 #3c404e #666a78 #9a9aa0 #c9a84a #d9d4c6 #ebe6d8 #fbf7ee` | 0–3 slate shadows, 4 the one gold, 5–7 pale stone (floor field is 5, not 2) |

Ramps stay monotone in luminance so the tint pass can index by value; all three ship in
`atlas.json meta.palettes` next to the first three and `boss_flash`.

- **foundry** — floor: dark rust field with one orange fleck and iron grit; floor_alt: a diagonal
  crack with one ember (5) in it. Wall: riveted iron plates (4) with light rivets (6) at the plate
  corners, a dark seam and one rust patch (3). Door: iron plates, dark seams, light knob. **Water is a
  molten channel** (`f` = 50 % dither of the two oranges, hot-yellow dashes) — it still reads "the
  light special floor" by value, and the foundry has no water creature. Stairs and chasm default.
- **deep** — floor: near-black (2) with the fewest marks of any biome (vision 4 — the floor must
  not fight the dark); floor_alt: a wet pool (3) and one bone chip (6). Wall: blue-black cave rock
  with vertical cracks. **stairs_down's top step is bone (6)** so the stairs are findable on the
  near-black floor; **chasm lip is `d`** (25 % ink-blue over blue-black) because "light falls in
  from above" does not apply and the floor-vs-chasm gap is only ~20 luminance — judged in the
  sample room: the chasm still reads as the hole. Water default (ink-blue dither, bone dashes).
- **sanctum** — floor: pale dressed flagstones, field 5 with light-slate mortar (3) and one lit
  texel (6) per slab; floor_alt: a cracked slab with a gold inlay fleck (4). Wall: white marble
  courses with a **gold trim course (4) at mid-height** (a frieze every 8 texels when stacked).
  **Water is a still reflecting pool** (`e` = slate greys, white dashes). Door default (pale planks
  over gold, i.e. a gilded door); stairs_down's top step comes out gold, stairs_up pale/pale/gold.
  Bones props on the pale floor survive only thanks to their ramp-0 rim — checked in the room.
- Every tile ≤ 8 colours by construction (ramp-indexed); `make_tiles.py` counts and fails if not.

## Per-asset notes (round 3 / Cut 3, 2026-09-16 — 14 monsters + 3 bosses, 17/17 accepted, 2 retried)

Three concurrent codex batches, one per biome (`prompts/batch6..8.txt`, 6/6/5 ids, logs
`logs/batch6..8.log`), same `gpt-6-astra` / `image_gen`; launched 12:00, all finals quiescent by
12:09, every source opaque `#0000FF` 1024×1024. Codex used its own single retry on seven ids
(golem's first landing was the RGB flatten-to-black; warden/echo/mirror_king/smith/eel/queen for
"small blue patches") and kept the echo's first version when the retry came back transparent. One
critique retry from our side (`batch9.txt`, forge_imp + siren, ~5 min): originals archived as
`archive/superseded/<id>_r1.png`, the sharper briefs are now the manifest descriptions. The
monster list is the CUT3 table's 14 kinds (the track brief says "15" but enumerates the same 14).

Census after the retries: 17/17 corners exact; every `<-- CHECK` is fringe. Splitting
`enclosed_near` into ≤ 3 px-of-an-opening fringe vs true subject spill: golem 696/3, foundry
master 899/0, lurker 701/0, troll 507/3, shade 204/16, queen 397/219, warden 570/89, echo 538/0,
mirror king 642/0, imp (retry) 36 total, siren (retry) 381/11, smith 114/26. The queen's 219 and
warden's 89 are scattered shadow flecks in 1 M px sources — sub-texel after the box downscale, and
`art-qc.py` reports zero semi-transparent blue in the packed frames. `qc/review.png` now
composites over all six biomes (review.py reads `make_tiles.PALETTES`).

- **iron_golem** (85×96) — codex's own retry after a flatten-to-black. Stepped block of riveted
  rust plates, small visor head; the right-angle silhouette is nothing like the ogre. Accept.
- **forge_imp** (34×56) — RETRIED. Round 1 was a flame-blob with the bottle lost in it (the
  brief's "flame tuft of hair" became a body of fire). Retry wording that worked: "the body is
  flesh, NOT made of fire", the bottle "pale bone-white glass ... separated from the body by a
  full ink contour ... the palest, brightest thing in the sprite". Now a solid orange imp holding
  a pale flask out front; at 28 px the flask is its own pale bump. Accept.
- **bell_sentinel** (42×80) — first try. Conical bronze bell for a head over a slim iron body, rod
  raised; the bell is the top of the silhouette on every biome. Accept.
- **slag_crawler** (121×48) — first try. Low horizontal dash, cracked crust, ember dots trailing
  off the tail. Widest monster in the set (121 px master = 7.5 tiles at runtime; the renderer may
  scale it 0.75 like the rat). Accept.
- **smith** (57×80) — first try. Square hammer raised over the shoulder, long apron rectangle;
  reads "hammer and apron", distinct from goblins and from the golem. Accept.
- **boss_foundry_master** (112×128) — first try. Rivet-spiked helm crown, giant hammer overhead,
  apron over plates, anvil chest. "The big crowned iron smith" next to the golem. Accept.
- **lurker** (59×80) — first try. Blank eyeless brow, craned neck, arms hanging past the feet.
  Most legible of the deep set. Accept.
- **deep_eel** (113×48) — first try. Thin three-wave ribbon with a frill; clearly not the fens
  eel's thick S at 32 px. Accept.
- **cave_troll** (96×96) — first try. Boulder back, head sunk low, knuckles at the ankles, no
  weapon — the hunched counterpart to the upright ogre. Accept.
- **siren** (54×80) — RETRIED. Round 1 stood with hands clasped and head bowed under the hood:
  "hooded robe", not "singer". Retry wording: "head THROWN BACK — chin up, face to the ceiling,
  mouth wide open — hood fallen back", voice-arcs "as thick as the contour line and a head-width
  wide", "Not a hooded figure looking down". Now the throat and the three arcs are in the
  silhouette. Accept.
- **mirror_shade** (88×96) — first try. Flat blue-black hero silhouette (sword low, half-cloak)
  with thin cyan-white edge light; no face. On the deep floor it is a dark shape on near-black
  and survives only by the ink ring + highlights — the intended "shadow" read; watch it after
  the runtime tint. Accept.
- **boss_lurker_queen** (117×128) — first try. Bone-spine crown, four arms, swollen belly,
  hatchlings on the flanks. Accept.
- **warden** (53×96) — first try. Tower shield forward, bow arc and quiver above the far
  shoulder; both bumps survive at 48 px. Accept.
- **acolyte** (70×80) — first try. Folded at the waist, both arms out with open palms; distinct
  from the siren's thrown-back head. Accept.
- **echo** (65×80) — first try (codex kept its first version after a transparent retry).
  Doubled walking figure, two heads and two sets of legs, rear copy paler. Accept.
- **sentinel** (56×80) — first try. Tapered monolith with one large gold-irised eye; no limbs.
  Not confusable with the bell sentinel. Accept.
- **boss_mirror_king** (93×128) — first try. Tall spiked gold crown, robe, sceptre topped with a
  round mirror — the boss tag reads. Hue drift: codex painted a pale ivory-and-gold king rather
  than the briefed dark-mirror body (the shade is dark, the king is not), so the "hero's dark
  reflection" idea is carried by the shade alone. Acceptable for the sanctum palette (it sits
  with the warden/acolyte); regenerate only if the ending screen wants the pair to rhyme.

`pack.py`: 40 keyed masters + 70 tiles → 110 frames in a 512×641 `atlas.png`; six biome ramps +
`boss_flash` in `meta.palettes`. `art-qc.py` PASS (110 frames, 0 warnings, 0 failures).
`qc/review.png` regenerated for the 17 new ids over all six biomes; `tiles/_sheet.png` has the
three new biome rows. Note for the client track: `web/src/render/palette.ts` hard-codes the
first three ramps; the new three are in `atlas.json meta.palettes` (`foundry`, `deep`, `sanctum`).

## Per-asset notes (round 4 / Cut 5, 2026-09-16 — three situation props, all six ramps)

`make_tiles.py` only, no codex. CUT5 §4's situations get one 8×8 prop each, drawn on
transparent so they overlay `floor`/`floor_alt`, ramp-indexed so every biome gets them for free
(the brief asks for Warrens/Fens/Crypt; `build()` iterates `PALETTES`, so foundry/deep/sanctum
ship too, with per-biome overrides where a ramp's index roles differ). Rim rule: ramp-0 wherever
the outer silhouette meets the floor; a lone lit texel (the shrine light) is exempt like the
bones sparkle. Judged at 4× on `tiles/_sheet.png` (the "over floor" rows now end
`shrine_0 shrine_1 vault vault_open nest_0 nest_1`; the sample room is 7×4 and carries a shrine,
a closed and an open vault, and both nest frames next to walls) and at 12× over floor.

- **shrine** (`<biome>_shrine_0/1`, 6 colours; sanctum 5) — a 5-wide stele (face 5, one lit
  glyph texel 6 over a carved 3) on a 7-wide plinth (4), full ramp-0 rim, and one ramp-7 texel
  floating above = the altar light. Frame 1 moves that texel one row up (2 Hz flip = a flicker;
  the light is the only thing that changes, so the renderer can also hold frame 0 for an
  unlit/used shrine). Sanctum: pale stele (6) with the one gold (4) as the glyph on a slate (3)
  plinth — face 5 would have matched the floor field. Foundry: iron stele (4) with an ember
  glyph (5) on a rust (3) plinth so the stele is not an orange block on rust.
- **vault** (`<biome>_vault`, 4 colours; `<biome>_vault_open`, 3) — a front-on cage: ramp-0
  posts and rails, a lit top rail (4, 5 ends), two ramp-4 iron bars, and a 2×2 lock plate (6)
  with one dark keyhole texel under a dark cross rail. The floor shows through four gaps. A
  first pass with ramp-0 bars read as a strongbox; lit bars between dark posts read "gate" at
  4×, which is why the bars are 4 not 0 (they sit in enclosed openings, so the outer rim is
  still all ramp-0). `vault_open` keeps the frame and rail and folds both bars flush against
  the posts, leaving a 4×5 window of open floor — "open" reads by the empty interior alone.
  On the deep ramp the 0-vs-2 post/floor gap is ~20 luminance; the lit rail and plate carry
  it (deep is outside the CUT5 biome list anyway).
- **nest** (`<biome>_nest_0/1`, 6 colours; sanctum 5) — a low straw mound, six texels tall
  (a seven-tall dome read as a head at 12×), body 4 with a lit patch 5 upper-right, a shadowed
  3 band at the base, two bone chips (7 then 6, diagonal, so they read as one bone) lower-left,
  ramp-0 rim. Frame 0: two ramp-3 slits in the lit patch (eyes closed, a sleeping den). Frame 1:
  the same two texels ramp-0 (eyes open). The renderer should flip to frame 1 on `on_see: nest`
  / when the den wakes rather than blink at 2 Hz. Sanctum: pale cloth-and-bone mound (6/7) with
  a gold fleck (4) for the bone chip; the default 4 would have been the sanctum's gold.

`pack.py`: 40 keyed masters + 106 tiles → 146 frames in a 512×650 `atlas.png` (was 110 / 641).
`art-qc.py` PASS (146 frames, 0 warnings, 0 failures). Frame ids for the client:
`<biome>_shrine_0`, `<biome>_shrine_1`, `<biome>_vault`, `<biome>_vault_open`, `<biome>_nest_0`,
`<biome>_nest_1` for each of `warrens fens crypt foundry deep sanctum`. The stray is a monster
sprite (a previous heir's companion), not a prop — nothing authored here.

## Art pass (2026-09-24) — register 3 environment, keyed register v2, portraits

Gap list: `art/ui/ART_GAP.md`. Style authority updated in `ART.md` (keyed register v2 and the reason; register 3).

- **Register 3 (environment, 24 ids `env_*`)** — batches 10–12 (8/8/8) + retry 13. Codex painted each as a tiny pixel
  tile enlarged to 1024 and grid-resampled it itself (~6 KB sources). `make_env.py` → 125 tiles (6 biomes × 20 ramp
  classes + 5 hue assets; the Burrows is re-ramped at runtime). First landing 18/24; retried `floor_1`, `floor_3` (too
  flat — "mortar lines the darkest value, plainly visible"), `crate` (blob → "top face + X brace"), `bones` (blue-grey
  shading inside the key), `water` (noise → "separated ripple dashes"), `banner` (a thin stick → "wide cloth, sigil,
  swallowtail"); 6/6 accepted on the retry (the banner's sigil came out dark, not bone-white). Originals in
  `archive/superseded/env_*_r1.png`. `torch_1` is derived (flame lifted a texel, two warm levels swapped).
- **Keyed register v2 (41 ids + new `goblin_captain`)** — batches 14–21 (heroes + ogre; D1 cast; captain/monkey/
  bloat/warlord; D9+ cast; foundry/deep/sanctum). v1 masters kept as `archive/superseded/<id>_v1.png`. Codex self-retried
  the goblin once. Our retry (batch 22): `siren` (came back hooded again — "NO HOOD, head tilted back, three sound-arcs")
  and `mirror_shade` (lost its sword); both accepted after. Known weak: `forge_imp`'s flask is red, not bone-white;
  `rat` is compact (3/4 pose) inside its 16-texel box.
- **Portraits (11)** — `prompts/ui_portraits.txt` → `art/ui/portraits/*.png` (1024 opaque squares; the CSS circle
  masks them): `hero_{fighter,rogue,ranger,caster}`, `pet_{rat,jackal,monkey,goblin}`, `captive`,
  `boss_goblin_{captain,warlord}`; 11/11 first try. `tools/ui-skin.py` packs them to `web/public/ui/portraits/*.webp`
  (256 px, q88, 200 KB total) and lists them in `skin.json`. The death variant is a CSS treatment
  (`.portrait.dead .face.painted`: desaturated, sepia, a blood tint), not a painting.

## Second art pass (2026-09-24) — situation props and items at 16×16, four sprite fixes

- **Batch 23** (`env_shrine env_vault env_vault_open env_nest`), **24** (`env_item_{potion,scroll,weapon,armour,gold}`),
  **25** (`rat goblin_captain boss_goblin_warlord forge_imp`), three concurrent, ~7 min; **26** retry (`env_shrine`: a
  lopsided stele that read as a boot → "wide, squat, symmetrical altar"; `env_nest`: "a nearly black burrow hole").
  Codex returned a mix of transparent and near-blue backgrounds; `key_source` takes both.
- `make_env.py`: shrine is a hue asset (5 colours + ink) with two candle flames painted by rule (`shrine_frames`, frame 1
  swaps core/body and lifts the tip), copied to every biome's `<biome>_env_shrine_<f>`; vault/vault_open/nest are ramp
  classes (nest frame 1 = two ramp-7 eye texels in the hollow); items are hue assets `env_item_<category>`.
- Sprites: rat side-on, long body (texel_h 20); captain/warlord leaf-green skin ("NOT grey, NOT olive-grey" in the brief);
  forge imp's flask ivory and empty. `review.py`: corners exact; captain's `enclosed_near` 204 is horn/arm fringe.
  All accepted first try. Superseded: `archive/superseded/<id>_v2r1.png`, `env_{shrine,nest}_r1.png`.

## For the next batch

- The full ART.md preamble as the first block + per-asset "tag in the silhouette" line + the
  colour-science paragraph produced 21/21 acceptable keyed sprites with no manual retry. Keep
  all three; drop none.
- Codex's `image_gen` defaults to transparent output; either accept it (pack.py does) or let
  codex's own corner self-check retry. Do not fight it in the prompt beyond one sentence.
- Watch for RGB flatten-to-black on any retry; `review.py` `corners=False` catches it.
- Horizontal animals (rat, jackal, eel, slag_crawler, deep_eel) get wide at a given `texel_h`; set
  `master_h` by the longest axis you want, not the height, if a future creature is very long.
- When a brief says a creature has a flame/glow *detail*, say what the body is made of in the
  same sentence ("the body is flesh, NOT made of fire") — round 3's imp became a fireball.
- A pose tag ("singing", "bowed") needs the head direction spelled out ("chin up, face to the
  ceiling"); "hooded" alone pulls the head down into the hood.
- Batches of six, three concurrent, took ~9 min wall clock end to end; a two-id critique retry
  ~5 min. Codex's own single retry now covers most colour-science misses, so check
  `review.py` for `corners=False` and judge the sheet before deciding on a retry.

## Hero looks (2026-09-25) — class × male / female / cat, sprites and portraits

`prompts/hero_looks.txt` (index) → batches `hero_looks_a.txt` (fighter female/cat, rogue ×3), `hero_looks_b.txt`
(ranger male/cat, caster ×3), `hero_looks_p.txt` (8 portraits), three concurrent, ~8 min; retries `hero_looks_r1.txt`
(`hero_ranger_female`: the old ranger read as a boy at 48 px, its braid hidden in the hood → "a long braid OUT of the hood,
behind the back") and `hero_looks_r1p.txt` (portrait `hero_caster_male`: the old long-haired caster read androgynous → short
hair, stubble). Each prompt lets Codex read the class's accepted master as its reference; the look's tag is in the silhouette
(man: short hair; woman: a ponytail/braid shape behind the head; cat-folk: two tall ears + a curving tail). 16/18 first try.
Reused as-is: sprite `hero_fighter_male` (= the v2 fighter), portraits `hero_fighter_male`, `hero_rogue_female`,
`hero_ranger_female`. `hero_<class>` stays as the default-look alias (a copy of fighter/caster `_male`, rogue/ranger
`_female`, `Class::default_look`); the replaced v2 masters are `archive/superseded/hero_{rogue,ranger,caster}_v2.png` and
`archive/superseded/portrait_hero_caster_v1.png`. Ids: `hero_<class>_<look>` in the atlas and in `ui/portraits/`; the client
falls back to `hero_<class>`, then the procedural silhouette (`render/look.ts`, `ui/frame.ts paintFace`).

## Per-biome tile sets (juice pass 2, 2026-09-26) — 82 ids, 6 biomes

`prompts/batch30..35.txt` (one biome per batch: burrows, fens, crypt, foundry, deep, sanctum; 13–14 ids each, `make_prompts.py`
adds a biome paragraph — "a DIFFERENT PLACE at a glance, not the same flagstones recoloured"), three concurrent, ~9 min per round.
80/82 accepted first try; retry `batch36` (`env_sanctum_crate`: a red-brown jumble → "a pale ivory chest, one gold band, no red";
`env_burrows_pot`: tiny and all black → "LARGE, a grey rim highlight, a ladle"), both accepted; originals in
`archive/superseded/env_*_r1.png`. Judged on per-biome sheets (source · converted · sample room) and in the headed render-demo.
Known weak: the Fens' mud floors are busy at 4×; the Burrows' loot sack reads as a lump.


## The fork's two lanes (juice pass 3, 2026-09-26) — Burrows and Fens tiles redrawn

`prompts/batch40` (Fens, 11 tiles) and `batch41` (Burrows, 10 tiles + the loot sack), two concurrent, ~12 min; the briefs pin
rows and name the palette's colours (`make_env.FREE_PALETTES`, a palette paragraph from `make_prompts.py`). Codex drew most tiles as
16×16 grids and snapped them to the palette itself. Retries (`batch42`/`43`/`44`, whose header was hand-edited to *prefer* drawing
the pinned grid pixel by pixel with PIL): Fens floors ×4 (every plank's lit row + joints read as bricks → long boards, one joint,
2–3 lit pixels), water / wall top / door / stairs (image_gen speckle → flat areas); Burrows floor_1 (a dotted root → one solid 2-px
root), floor_3 (speckle), floor_2 (claw marks read as letters → pebbles only), wall_face_0 (speckle twice → roots at pinned
coordinates). All accepted; `FREE_SWAP` evens the Fens boards' remaining lit ends and joints. Superseded: `archive/superseded/
env_*_j2.png` (juice pass 2's drawings), `env_*_j3r1.png` / `env_burrows_wall_face_0_j3r2.png` (this round's rejects). Judged at the
watch's zoom in the headed render-demo (`scratchpad/juice3/biomes-before-after.png`) and a real D5 watch.
