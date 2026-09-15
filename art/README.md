# art/ — Riddle art pipeline

`art/ART.md` is the style authority (two registers: keyed watercolour-anime sprites, hand-authored
8×8 pixel tiles). `art/manifest.json` lists every keyed asset and its brief.

## Pipeline

```text
manifest.json ──make_prompts.py──▶ prompts/batchN.txt ──codex exec──▶ generated/<id>.png
make_tiles.py ───────────────────────────────────────────────────────▶ tiles/<biome>_<tile>.png, gas_*, fire_*, item glyphs, _sheet.png
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
`fire_0/1`; item glyphs `potion scroll weapon armour gold`. `meta.palettes` carries the three 8-colour
biome ramps for the palette/tint pass.

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

## For the next batch

- The full ART.md preamble as the first block + per-asset "tag in the silhouette" line + the
  colour-science paragraph produced 21/21 acceptable keyed sprites with no manual retry. Keep
  all three; drop none.
- Codex's `image_gen` defaults to transparent output; either accept it (pack.py does) or let
  codex's own corner self-check retry. Do not fight it in the prompt beyond one sentence.
- Watch for RGB flatten-to-black on any retry; `review.py` `corners=False` catches it.
- Horizontal animals (rat, jackal, eel) get wide at a given `texel_h`; set `master_h` by the
  longest axis you want, not the height, if a future creature is very long.
