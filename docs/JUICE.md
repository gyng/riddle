# Juice — lighting, FX and UI motion

*2026-09-25. Brief: make the dungeon and the chrome feel alive ("zhng") with 2D/hybrid art tech, behind a quality setting, without
touching game truth. Everything here reads existing events and the floor the viewer already holds; no crate changed.*

## 1. What the renderer did before

| Area | State before this pass |
|---|---|
| Lights | Up to 12 torches + a hero light as unshadowed quadratic pools in the blit, one global 2-frame flicker; a halo on each flame |
| Grade | Per-biome rgb grade, ambient, lift strength and saturation (`GRADES`), one soft fog falloff past the vision; memory dim 0.68 |
| Particles | None (projectiles are a 2×2 dot; the leash is a dotted arc) |
| Shake | Fight frame only: ±1–2 env texels for 3 ticks on any hurt |
| Hit feedback | A 2-tick flash to 50 % palette-brightest; a 3-texel lunge; a 4-step screen-door dissolve on death; a boss palette flash |
| UI motion | Tile press (2 px + darker), gem pulse, a reveal carve + glint, the row highlight, bar transitions |

## 2. Shortlist, ranked (feel · clarity · cost)

| # | Technique | Feel | Clarity | Cost | Verdict |
|---|---|---|---|---|---|
| 1 | **2D light field with soft shadows** (GPU pass over the wall mask; torches, hero, fire, bolts, hit "pop" lights) + wall-foot AO | high: rooms read as lit spaces, fire lights its surroundings, a hit flashes the room | up: the hero's pool and the fire read; walls ground the floor | ~4 M mask fetches/frame on a 128² map | **built** (med+) |
| 2 | **Hit feedback**: squash & stretch, near-white first flash on foes, popping damage numbers, hit-stop, kick shake | highest per line of code | up: damage is legible as a number; who was hit is obvious | CPU only | **built** (med+) |
| 3 | **Instanced particles**: blood / sparks (constructs, undead) / goo, death dust + soul motes, embers over fire and torches, gas puffs, potion swirl, gold glint, whiffs, dust motes in a lit room | high | neutral (small, short, under the text) | ≤ 900 quads, one draw per tag | **built** (med+; motes high) |
| 4 | **Bloom on emissives** (tag-selected: flames, fire, embers, sparks, souls) | medium-high: fire finally glows | neutral (hot colours only; gas green damped) | 3 passes at ¼ target pixels | **built** (med+) |
| 5 | Death / boss moments: slow-mo, red vignette pulse, desaturation; boss spawn/sight: purple vignette + shake; low-hp heartbeat vignette | high at the moments that matter | up: danger has a colour | uniforms only | **built** (med+; pulses static under reduced motion) |
| 6 | Sprite **rim light** toward the light + stone **relief** (luma-as-height emboss along the light gradient) — the normal-map look without normal maps | medium | neutral | 3–5 extra fetches/px | **built** (high) |
| 7 | **Heat shimmer** over fire (whole-texel wobble, stays pixel-crisp) | medium | neutral | tiny | **built** (high, motion on) |
| 8 | UI motion: screen fade-in, sheet rise-and-settle, panel unfold, press depth on every button family, torchlight sheen over the bar and console, gem rune glow, gold count-up + glint, death banner drop + verdict slam, report lines settle | high for the chrome ("web form" → game) | neutral: nothing moves text the tooling reads | CSS only | **built** (on unless reduced motion / headless) |
| 9 | Codex-generated normal maps for heroes/bosses | medium | neutral | art round-trip per sprite and per look | **deferred**: #6 gives most of it procedurally; revisit with the hero-looks art |
| 10 | Radiance cascades / JFA SDF GI | marginal over #1 at this map size | neutral | high, and needs float targets on phones | **rejected** for now: #1 already shadows through the mask |
| 11 | Palette-aware dithered fog of war | low | down: the second art pass removed dithered fog on purpose ("speckle") | tiny | **rejected** |
| 12 | Parallax layers | low (top-down, no pitch) | — | — | **rejected** |

## 3. How it is built (web/src/render/, web/src/juice.*)

- `quality.ts` — the tier. `low` = the pre-juice blit compiled exactly (`FX` define 0), no particles, numbers, squash, hit-stop.
  Auto: a software GL (SwiftShader / llvmpipe, i.e. the headless client gates) is `low`; a real GPU starts `high` and steps down
  one tier after two 3 s windows with > 30 % of frames over 20 ms. `?fx=low|med|high` or `localStorage["riddle.fx"]` pins it.
  `prefers-reduced-motion` keeps light and bloom and drops shake, hit-stop, slow-mo, squash, rising numbers, pulses, ambient
  particles. `viewer.stats().fx` reports the tier.
- `light.ts` — `LightField`: the wall mask as a linear-filtered data texture (re-uploaded only when the tiles change), a light map
  at 4 texels per tile rendered each frame: ≤ 24 lights, 10 mask samples each along the segment (a soft penumbra), wall-foot AO
  in alpha. The blit multiplies the world by it in place of the torch pools (ambient × 0.66 so the pools read).
- `bloom.ts` — the layers that glow write the target's emissive tag (a = 0.625; the pre-juice blit already treats it as a sprite);
  bright pass → 2 blur passes at half target size → added in the blit (linear, a smooth halo under a crisp image).
- `fx.ts` — `Juice`: `ReplayState.onEvent` (a new hook, silent while a seek or a skip replays the past) queues events; each frame
  they become particles, numbers, pop lights, squash, hit-stop / slow-mo (the viewer clock's rate), a kick shake, vignette pulses.
- `blit.ts` — `FX` 1: light field, AO, sprites tinted by the light's hue, bloom, vignette, desaturation; `FX` 2: + rim light,
  relief, heat shimmer.
- `web/src/juice.ts` + `juice.css` — `html[data-juice=on]` (off under reduced motion and in automation on a software GL; `?juice=`
  pins). The purse count-up is a CSS `@property` counter on a pseudo-element over the real `$N` (transparent while counting), so
  the DOM text is always the true sum.
- Art never blocks: particles use a generated white texel and the bitmap font; nothing needs a new asset. Missing torch art → no
  torch lights (the hero, fire and pops still light the field).
- Debug/measure: `render-demo.html?busy=1&biome=foundry&depth=20&speed=4&texels=120[&fight=0][&seek=T][&fx=]`;
  `window.__riddleSyncTiming = true` drains the GPU each frame so `stats().cpuMs` includes the GPU (measurement only).

## 4. Numbers

See the report in the handback / `scratchpad/juice/*/frames.json` (headed GPU, 400 × 800). The sync-timed pipeline cost on this box
is dominated by the WSLg readback round trip (~8 ms floor in both tiers), so the rAF interval p50/p95 is the frame-time claim.

## 5. Deferred (pass 1) → what pass 2 did

- ~~Codex normal maps for heroes and bosses~~ → **derived** normal maps (pass 2, §6.2): the sprite's own silhouette and shading give the
  volume; a Codex round trip per sprite and per look was not needed for the light pass to read them.
- ~~Sheet close easing~~ → a ghost of the closing panel eases out (§6.3); `ui/sheet.ts` still removes the sheet at once.
- A measured phone GPU number (the timer query never resolves in this harness) — still deferred.

## 6. Pass 2 (2026-09-26): biomes, boss moments, normals, the Cut 27 surfaces, sound

Brief: the biggest feel gain first — biomes that look like different places, the pass-1 deferred list, sound with variety and a
limiter (verified objectively: an agent cannot listen), the perf budget kept (p95 ≤ 16.7 ms on the GPU at high, `low` unchanged).

### 6.1 Per-biome tile sets (art/, `art/ART.md` register 3b)

Every biome drew the same tiles recoloured. 82 Codex drawings (6 biomes × floors ×4, wall faces ×2, wall top, door, stairs ×2, three
props, and water where the material differs; the Warrens keep watch.png's set) → `make_env.py` → the same `<biome>_env_<name>` ids the
renderer already keys by biome (not depth: the route puts a biome at different depths). A per-biome drawing keeps its own value
structure (the shared set's fixed quantiles sprayed a checker or a rivet grid into speckle). The Burrows gets its own art and ramp
(`atlas.ts` no longer overwrites loaded Burrows tiles with the Warrens' recolour). Fallbacks unchanged: shared drawing → 8×8 register →
procedural. Shots: `scratchpad/juice2/before/biome-*.png` vs `after/biome-*.png` (zoomed: `after-grid.png`), QC sheets `qc-*.png`.

### 6.2 Derived sprite normals (render/normals.ts, high)

Each sprite texel's height is a dome over its chamfer distance to the silhouette's edge (a 4-texel bevel) plus 35 % of its luma; the
normal is the gradient. Derived on the CPU once the sprite sheet settles (~20 ms per 512² derivation, debounced 400 ms after the sheet
changes; a new sprite draws unshaded until then), drawn by a twin of the entity layer (same instances, squash, flip, dissolve) into a
target the size of the main one; the blit's `FX 2` path shades sprite texels by N·L toward the light field's gradient (a soft upper-left
key where the field is flat), scaled by the light's strength, on top of the rim light. One extra draw call; nothing at `low`/`med`.

### 6.3 Motion

- **Boss moments** (render/fx.ts): the entrance (a spawn, a `see`, or — new — the viewer's first frame with the boss in view): a ring of
  dust, rising violet motes, a violet pop light, a 3-texel shake, a 140 ms held breath; the break (`<boss> breaks`): a spark fountain, an
  ember ring, an orange flash, a 110 ms hit-stop; the fall: two rings (dust, gold), a soul plume, a white-gold flash, 1.2 s of slow-mo at
  0.3, a 4-texel shake. Each has its own sound (§7). Shots: `scratchpad/juice2/{before,after}/boss-*-{entrance,break,down}.png`;
  `render-demo.html?boss=1&biome=crypt` plays all three.
- **Sheet close**: `sheetGhost` (juice.ts) clones the closing panel — inert, aria-hidden, no studs — at its rect for 150 ms (drop and
  fade); the sheet itself leaves the DOM at once, so every tool that reads it sees it gone.
- **The fold line** (Cut 27 §1): the head settles in (letter-spacing closes), its chips pop in one by one, the ▸ breathes, docking
  slides it up (fill `backwards`, so `.faded` still dims it). **The divergence scene** (Cut 27 §2): it opens like a lens (a clip from
  its middle), each branch's tag slides in, the frame warms on the edited branch, the end word slams and rings in its colour, the stills
  step in, the line under `vs sent` glints when it lands. All CSS in `juice.css`, gated on `data-juice=on` and reduced motion; no copy.

## 7. Sound (web/src/audio.ts) — the audit and pass 2

**Before**: 9 cues (hit, slay, rule, telegraph, three exits, level, unlock) + the camp drone; one timbre per cue, fixed pitch (every hit
at a given damage identical, every kill identical), straight into the destination (no bus, no limiter), no ambience, nothing on the
chrome, the boss moments or the Cut 27 surfaces.

**After** (the Cut 10 contract kept — ≤ 2 oscillators and one gain envelope per cue, ≤ 200 ms, the death note ≤ 1 s; the gate
`web/tests/audio.mjs` now also covers every new cue, 8 consecutive strikes never identical, the bed up in a run and gone after):
- a noise burst per cue (a shared white-noise buffer at a random offset through one biquad — no second gain);
- per foe family (flesh · bone · ooze · metal · spirit, `familyOf`) timbres for the hero's blows (`strike`), the blows he takes (`hit`,
  whose thud still falls with the damage) and the kills (`slay`); every combat cue jittered (pitch ±6–9 %, filter ±20 %, length ±12 %,
  noise slice);
- boss moments `boss_in` (a sub drop under a saw), `boss_break` (a steel crack), `boss_down` (a falling boom);
- the chrome (read off the DOM in juice.ts, no screen module touched): `click` on any button, `edit` on a rule's chip, `buy` when the
  purse falls, `verdict` with the death word's slam; the Cut 27 surfaces: `fold`, `scene` per branch, `scene_end` (resolved / not);
- per-biome **ambience beds** in the watch (`audio.bed(biome)` from the HUD's snapshot; a 6 s seamless two-pole noise texture with a
  slow breath and a faint tone, rendered once per biome, plus one-shots on a Poisson clock — drips, skitters, bubbles, clanks, crackle,
  wind, a far bell, chimes — each synthesised fresh with its own pitch and pan; buffers only, never an oscillator);
- a master bus into a **soft limiter** (a WaveShaper: linear to −3 dBFS, a tanh shoulder that never passes 0.93). A
  DynamicsCompressorNode was tried first and measured: its detector clamped every cue's onset by 6–7 dB (it flattened the transients
  the hits are made of) and added 1.7 dB of makeup gain.

### 7.1 Measurements (OfflineAudioContext, the game's own chain; `node scratchpad/juice2/audio-measure.mjs`)

Six seeded renders per cue at 48 kHz; loudness is K-weighted mean square over the cue's own length (a short-term LUFS stand-in, not
BS.1770-gated); centroid is magnitude-weighted (and power-weighted, which follows the body of the sound rather than its noise floor);
"min Δ" is the smallest max-abs sample difference between consecutive renders (0 = identical: the musical cues are fixed on purpose).

| cue | peak dBFS | RMS dBFS | loudness | clipped | pre-limiter peak | centroid Hz (mag · pow) | ± centroid | min Δ |
|---|---|---|---|---|---|---|---|---|
| `hit:flesh` | -10 | -24.5 | -24.6 | 0 | -10.4 | 2729 · 200 | 190 | 0.3205 |
| `hit:bone` | -9.3 | -22.3 | -22.3 | 0 | -10.1 | 4397 · 636 | 346 | 0.3385 |
| `hit:ooze` | -9.8 | -23.1 | -23.5 | 0 | -10 | 532 · 217 | 137 | 0.3315 |
| `hit:metal` | -8.1 | -21.6 | -21.4 | 0 | -8.5 | 8473 · 942 | 295 | 0.4556 |
| `hit:spirit` | -9.6 | -22.8 | -23.2 | 0 | -9.8 | 1454 · 351 | 169 | 0.3371 |
| `strike:flesh` | -10.7 | -24.7 | -25.1 | 0 | -12.2 | 1166 · 139 | 1535 | 0.0731 |
| `strike:bone` | -11 | -25.3 | -24.9 | 0 | -11.2 | 4715 · 1176 | 142 | 0.3443 |
| `strike:ooze` | -15.9 | -26.3 | -26.8 | 0 | -16.9 | 588 · 294 | 219 | 0.0696 |
| `strike:metal` | -9.5 | -27.3 | -26.7 | 0 | -10.8 | 9685 · 1680 | 135 | 0.3581 |
| `strike:spirit` | -15.3 | -27.1 | -27.5 | 0 | -15.9 | 1559 · 713 | 229 | 0.1699 |
| `slay:flesh` | -11.3 | -21.6 | -21.8 | 0 | -11.4 | 3319 · 597 | 80 | 0.3011 |
| `slay:bone` | -18.5 | -27.6 | -27 | 0 | -19.1 | 4556 · 1532 | 179 | 0.1874 |
| `slay:ooze` | -13.4 | -24.4 | -24.9 | 0 | -14.6 | 640 · 211 | 181 | 0.1091 |
| `slay:metal` | -8.9 | -26.7 | -26.2 | 0 | -9.1 | 8607 · 1857 | 233 | 0.2366 |
| `slay:spirit` | -16.8 | -29.4 | -29.1 | 0 | -17.3 | 6318 · 1393 | 74 | 0.0995 |
| `rule` | -18.6 | -35.4 | -33.3 | 0 | -18.6 | 4278 · 2349 | 451 | 0.0423 |
| `telegraph` | -17.8 | -27.4 | -27.8 | 0 | -18.2 | 1173 · 624 | 50 | 0.0092 |
| `exit_bank` | -9.5 | -22.3 | -22.9 | 0 | -9.5 | 768 · 334 | 0 | 0 |
| `exit_return` | -10.5 | -23.2 | -23.8 | 0 | -10.5 | 752 · 324 | 0 | 0 |
| `exit_death` | -14 | -29.5 | -29 | 0 | -14 | 188 · 68 | 0 | 0 |
| `level` | -19.9 | -25.3 | -25.1 | 0 | -19.9 | 4328 · 1189 | 0 | 0 |
| `unlock` | -19.9 | -25.3 | -25 | 0 | -19.9 | 4568 · 1387 | 0 | 0 |
| `buy` | -17.7 | -30.4 | -28.8 | 0 | -17.7 | 5460 · 2469 | 131 | 0.0918 |
| `boss_in` | -5.4 | -16 | -15.6 | 0 | -5.6 | 3021 · 208 | 75 | 0.114 |
| `boss_break` | -2.5 | -16 | -14.6 | 0 | -3 | 9216 · 3849 | 131 | 0.8464 |
| `boss_down` | -6.2 | -22.8 | -22.9 | 0 | -6.2 | 2625 · 347 | 77 | 0.1188 |
| `click` | -17.8 | -37.8 | -36.4 | 0 | -18.7 | 7693 · 2051 | 300 | 0.0811 |
| `edit` | -18.3 | -35.5 | -36.1 | 0 | -19.3 | 2500 · 539 | 479 | 0.037 |
| `verdict` | -8.5 | -20.1 | -20 | 0 | -8.7 | 2930 · 177 | 103 | 0.0663 |
| `fold` | -21.9 | -36.5 | -36.9 | 0 | -23 | 2204 · 708 | 55 | 0.0215 |
| `scene` | -21.9 | -34.1 | -34.6 | 0 | -21.9 | 3392 · 487 | 70 | 0.0325 |
| `scene_end:up` | -17.3 | -28.9 | -29.4 | 0 | -17.3 | 1299 · 674 | 0 | 0 |
| `scene_end:down` | -16.4 | -28.6 | -29.2 | 0 | -16.4 | 1038 · 527 | 0 | 0 |

| bed (12 s) | peak dBFS | RMS dBFS | loudness | clipped | centroid Hz (mag · pow) |
|---|---|---|---|---|---|
| warrens | -22.6 | -43.1 | -43.8 | 0 | 3410 · 335 |
| burrows | -21.3 | -42.6 | -42.9 | 0 | 3765 · 305 |
| fens | -22.7 | -43.3 | -43.5 | 0 | 3856 · 689 |
| crypt | -25.6 | -42.6 | -42.9 | 0 | 3099 · 408 |
| foundry | -22.1 | -42.4 | -43.1 | 0 | 3858 · 251 |
| deep | -22.9 | -43.1 | -43.8 | 0 | 3946 · 236 |
| sanctum | -30.5 | -44.9 | -43.5 | 0 | 4997 · 1863 |

- **Clipping**: 0 clipped samples in every cue and every bed. Worst case, three heavy cues at one instant (the burst cap: `boss_down`
  + a 24-damage hit + `boss_break`): −0.6 dBFS summed before the limiter; after it ≤ 0.93 by construction.
- **Hierarchy**: the boss moments −15…−23, fights −21…−29, the telegraph/rule/UI −28…−37, the beds −43…−44 (≈ 15–20 dB under a
  fight: audible between blows, gone under them).
- **Family spread**: the five families' `strike` centroids span 588–9685 Hz (power-weighted 139–1680 Hz); `slay` 640–8607 Hz.
- **Variety**: a live run of 12 strikes through the real object (Math.random jitter) logged 12 signatures, 0 consecutive identical;
  jittered cues' renders differ by 0.02–0.85 full scale between consecutive seeds.
- The listening pass a person does once is `eval/AUDIO.md`.

## 8. Numbers (pass 2)

Headed GPU (D3D12 RTX 3080), 400 × 800 at the Windows scale (1.5), fx `high`, on a quiet machine (load < 8), before = a HEAD worktree
(the pass-1 renderer and atlas) and after = this pass, back to back (`node scratchpad/juice2/measure.mjs <label> --port N [--wasm]`;
`scratchpad/juice2/perf-*/frames.json`). rAF is the frame-time claim; "sync" drains the GPU each frame (measurement only; its ~8 ms
floor is the WSLg readback).

| scene | rAF p50 / p95 before | after | sync p50 / p95 before | after | draw calls before → after |
|---|---|---|---|---|---|
| busy D5 Burrows 1× | 16.7 / 16.7 | 16.7 / 16.8 | 7.7 / 11.7 | 9.3 / 12.5 | 17 → 17 |
| busy D5 Burrows 4× | 16.7 / 16.8 | 16.7 / 16.8 | 8.4 / 12.3 | 9.1 / 13.0 | 16 → 17 |
| busy D20 Foundry 1× | 16.7 / 16.8 | 16.7 / 16.7 | 8.7 / 12.3 | 8.3 / 11.8 | 17 → 17 |
| busy D20 Foundry 4× | 16.7 / 16.8 | 16.7 / 16.8 | 8.8 / 11.9 | 8.8 / 12.2 | 16 → 17 |
| busy D30 Sanctum 4× | 16.7 / 16.7 | 16.7 / 16.8 | 8.4 / 12.1 | 9.4 / 13.5 | 17 → 17 |
| boss moments, Crypt 1× | 16.7 / 16.8 | 16.7 / 16.8 | 9.0 / 12.2 | 9.4 / 14.6 | 16 → 17 |
| the watch (fake engine) 1× | 16.7 / 16.8 | 16.7 / 16.8 | 9.3 / 14.8 | 10.9 / 14.9 | 15 → 17 |
| the watch (fake engine) 4× | 16.7 / 16.8 | 16.7 / 16.7 | 8.8 / 12.4 | 9.4 / 13.5 | 16 → 16 |
| the real watch (wasm) 1× | — | 16.7 / 16.8 | — | 8.2 / 15.1 | 14 |
| the real watch (wasm) 4× | — | 16.7 / 16.8 | — | 11.6 / 17.2 | 17 |

Every scene holds vsync at p95 (16.8 ms is one 60 Hz interval). The normal pass costs one draw call and ~0.5–1 ms of the drained
pipeline; its CPU derivation (~20 ms per 512² sheet) runs once the sheet settles, never per frame. `low` is untouched: the blit's
`FX 0` path, the headless gates' tier, compiles exactly as before (the normals, the new particles and the chrome's motion are `high`/
`med`/`data-juice=on` only; the sounds are not visual). Under load (72 on 32 cores, another agent's metrics job) the before and the
after both fell to 33 ms p95 — those runs are not the claim.

Client gates: the full suite ran 24/27 under that load (clarity's real-wasm edit timings, cut25's dark-stretch, qaj's cage beat); each
of the three passed on a quiet re-run (`clarity: ok (67)`, `cut25: ok (22)`, `qaj: ok (34)`); `audio: ok (69)`, `fights: ok (47)`,
`looks: ok (15)`, `cut16`, `cut27: ok (35)`. The headed walk (`playtest.mjs --seed 3001 --headed`) is clean.

## 9. Pass 3 (2026-09-26): the fork's two lanes, the boss's fall toned

- **Burrows vs Fens** (they meet at the D5 fork): both redrawn by Codex in a fixed per-biome palette with hue roles and converted
  by nearest colour (`art/make_env.py` `FREE_PALETTES`; `art/ART.md` register 3b) — the Fens a grey plank boardwalk over dark water
  with teal pools and reeds, a log palisade standing in water, open bog water; the Burrows packed warm-ochre earth with a root vein,
  root-veined earth banks and timber shoring. Burrows ramp ochre (was red clay, `palette.ts`); grades (`blit.ts` `GRADES`) Burrows
  ochre at full saturation, Fens cool; the Fens' light teal (`index.ts` `LIGHT_TINT`). Shots: `scratchpad/juice3/biomes-before-after.png`
  (headed GPU, 1.5×, the watch's zoom), a real D5 Burrows watch `scratchpad/juice3/after/watch-D5.png`.
- **The boss's fall** (`render/fx.ts`): the cream plume, the white-gold radius-8 light and the 1.1 s cream flash washed the boss into a
  pale blob before he fell. Now in the boss's own colour (`BOSS_COLS`: warlord green, bloat mother goo, lich teal, foundry master
  ember, lurker queen violet, mirror king gold): a dust ring, a small coloured ring and ten motes rising off him, a radius-3.5 pop for
  420 ms, a 0.3 flash for 480 ms, slow-mo 0.3 for 700 ms (was 1200), a 3-texel shake; the killing blow flashes him at 0.2
  (`index.ts` `BOSS_FALL_FLASH`; the slow-mo stretched the 0.5 flash). The break's pop is radius 3 / 380 ms, its flash 0.3 / 420 ms.
- **The break's cream cloud** was the hero, struck in the same tick, flashed toward the ramp's brightest with his ink gone: a sprite's
  flash now spares its ink and darkest shading (`layers.ts`: the mix scales by `smoothstep(0.06, 0.3, luma)`), so any struck sprite
  reads as itself lit up. Shots: `scratchpad/juice3/boss-death-before-after.png` (Burrows and Fens; break · fall · +250 ms · +750 ms).
- **Numbers**: rAF p50/p95 16.7/16.7–16.8 in every scene of `measure.mjs` (headed D3D12, high, 1.5×; `scratchpad/juice3/perf-after`),
  as in pass 2; draw calls 17–18. Client gates fights, looks, cut13, cut16, cut22, cut25, cut26, qaj pass (fights/cut13/cut25/qaj
  failed differently under load 30–78 and passed alone on a quiet window).

## 10. The gfx eval (2026-09-27): a blind-rated harness, six rounds, the desktop frame, context loss

Brief (`scratchpad/queued/gfx-eval-brief.md`): an eval harness with blind visual raters, then improve → re-rate rounds until the mean is
≥ 8.0 with no moment under 7.0, or stop after three consecutive rounds of < +0.2 and name what blocks the next gain. It stopped on that rule:
**4.90 → 5.97** (min 3.65 → 5.00), rounds 4–6 at −0.04, −0.02, +0.19.

### 10.1 The harness (tools/)

- `tools/gfx-round.sh <out>` — one round on a **frozen server** (a fresh no-HMR Vite, `web/tests/vite.test.config.ts`: another agent's edit
  reloaded the walk's pages mid-capture twice) → `tools/gfx-eval.mjs`: headed Chromium on the GPU, the same walk every round — the real
  engine (seed 4242: a run watched at `normal`, `?absent=8h` → the report, the worst death, the camp with four rules, an edit and its scene,
  the why sheet, the forecast, the oath board), the fake engine with `?fake_god=1` (new dev knob, `engine/fake.ts`: the hero never drops under
  1 hp) for the Fens at D6 and a fight, `render-demo.html?boss=1` for the boss's entrance, break and fall, and 1440 × 900 for desktop
  camp / watch / death / report / why sheet. Per moment: `<name>.png`; `<name>-strip.png`, 4 frames 180 ms apart from the compositor's
  screencast (CDP `Page.startScreencast`, timestamped — screenshots took 100–300 ms each under load and missed openings), anchored on the
  tap for sheets (a DOM click: Playwright's actionability waits pushed the opening out of the window); `frames.json` (rAF p50/p95/p99, the
  viewer's cpuMs); `anims.json` (every CSS animation running after the trigger: name, duration, delay, easing); `layout.json` (the owner's
  check (a): console, portrait and gem on screen, nothing over the gem at its centre and four inner points, no horizontal scroll).
- `rater/`: every shot under a neutral name + `prompt.txt` (`tools/gfx-rater-prompt.txt`): an art director's brief, the four targets, 0–10 on
  readability · hierarchy · target · motion · juice with one quoted reason each and three changes. Two fresh general-purpose raters a round,
  never reused (A–N); `scratchpad/gfx-eval/score.py` averages them. The two raters' means agreed within 0.52 every round.
- `tools/gfx-audio.mjs` — the §7 measurement as a tool (OfflineAudioContext through the game's chain).
- `web/tests/layout.mjs` — the IA checks as a gate (phone: every screen, every sheet, the scene; desktop: three columns, console ≥ 90 %,
  a sheet beside its tablet). `web/tests/ctxloss.mjs` — the context-loss must-fix (§10.4).

### 10.2 Scores (mean of the two raters' five criteria; `scratchpad/gfx-eval/round*/`)

| moment | r0 | r1 | r2 | r3 | r4 | r5 | r6 (final) | Δ |
|---|---|---|---|---|---|---|---|---|
| watch-warrens | 5.35 | 5.20 | 5.25 | 5.55 | 5.50 | 5.50 | 5.75 | +0.40 |
| report | 5.45 | 5.60 | 5.90 | 6.15 | 5.40 | 6.30 | 5.35 | -0.10 |
| death | 5.95 | 6.80 | 6.65 | 6.70 | 6.90 | 6.90 | 7.20 | +1.25 |
| camp | 5.35 | 5.70 | 5.55 | 5.15 | 6.15 | 5.80 | 5.40 | +0.05 |
| scene | 4.80 | 5.80 | 5.40 | 5.15 | 6.40 | 4.50 | 5.45 | +0.65 |
| edit | 4.55 | 5.40 | 5.60 | 5.90 | 5.45 | 6.30 | 6.10 | +1.55 |
| forecast | 4.75 | 5.30 | 5.75 | 6.30 | 5.75 | 6.40 | 6.35 | +1.60 |
| oaths | 4.75 | 4.60 | 5.20 | 5.90 | 5.80 | 6.10 | 6.25 | +1.50 |
| watch-fens | 3.65 | 4.60 | 4.80 | 5.00 | 4.80 | 3.90 | 5.05 | +1.40 |
| fight | 4.80 | 5.10 | 5.35 | 5.90 | 5.15 | 5.40 | 5.55 | +0.75 |
| boss-in | 5.85 | 6.50 | 6.40 | 6.95 | 5.80 | 6.70 | 7.00 | +1.15 |
| boss-break | 5.45 | 5.70 | 5.15 | 6.10 | 6.20 | 6.10 | 6.80 | +1.35 |
| boss-fall | 5.15 | 5.70 | 5.65 | 6.35 | 6.05 | 5.60 | 6.65 | +1.50 |
| d-watch | 3.85 | 4.40 | 4.85 | 5.15 | 6.15 | 5.50 | 5.90 | +2.05 |
| d-report | 4.65 | 4.60 | 5.35 | 5.50 | 5.20 | 5.80 | 5.00 | +0.35 |
| d-death | 5.20 | 5.80 | 5.60 | 6.10 | 6.75 | 6.20 | 6.60 | +1.40 |
| d-camp | 4.55 | 6.10 | 5.60 | 5.75 | 5.65 | 5.50 | 5.75 | +1.20 |
| d-edit | 4.05 | 4.50 | 5.00 | 5.45 | 5.35 | 5.60 | 5.35 | +1.30 |
| **mean** | **4.90** | **5.41** | **5.50** | **5.84** | **5.80** | **5.78** | **5.97** | **+1.08** |
| min | 3.65 | 4.40 | 4.80 | 5.00 | 4.80 | 3.90 | 5.00 | |

Before/after: `scratchpad/gfx-eval/sheet.png` (phone, round 0 over round 6), `sheet-desktop.png`.

### 10.3 What each round changed (all kept: each round's losses were strip timing or a regression fixed the round after)

- **Round 1** — render: the callout and the fight caption are a DOM iron plate with a notch (`tags.ts` `Plate`; `debugText` boxes kept), the
  unexplored void a dark Voronoi rock that fades from the lit edge (`blit.ts`, FX ≥ 1 only), ambient 0.66 → 0.95 of the field, a wider warmer
  hero light, the Warrens' saturation 0.55 → 0.78, damage numbers at full size, slash crescents on landed blows, the Burrows/Warrens/Crypt
  water redrawn from the Fens' art in their ramps (it was a 1-texel checker), glowing gas puffs (the 55 % screen-door read as a checker), the
  phone watch zoomed (`PHONE_TEXELS` 120 → 100, desktop 112). Fake engine: `R2 attack` callouts in the core's order (`attack attack nearest`),
  `blade` drawn as `spectral_blade` (a magenta fallback jug). Art (Codex, QC'd): 13 action icons `v_*`, the death and report backdrops.
  CSS: the gilded primary plaque, amber reach troughs, the scene's iron frame, embers, the vista's fire, the banner's sway and seal glint.
- **Round 2** — the action's icon plaque on every rule tablet, the why sheet, the fix tablets and the desktop's read-only tablets; the forecast's
  try row back in ink on parchment; the primary as a dark iron plaque with rivets and gilt letters (the gold face drowned its word at 1440);
  the report's plaques stamp in; the seal slams with a dust ring; name plates capped to the hostiles nearest the hero; the ambience bed
  ducks 9 dB under a boss's entrance, break or fall, the verdict and a run's end (`audio.ts`, gated in `audio.mjs`); the desktop rules column
  lights the rule that acted.
- **Round 3** — no black frames on a frame cut (`CUT_FRAMES` 2 → 0); the hero's hurt flash 0.5 → 0.28; sheets and panels unroll from their
  top edge (they faded from 30–40 % opacity: "a double exposure"); the oath board deals its tablets in; rune plates in empty command slots (at QA K's ≤ 0.2 opacity, `ui.mjs`);
  the stock in sunk wells (and the frame's grid column pinned to the viewport: the wells' min-content once pushed the console 6 px off
  screen — the harness's layout check caught it).
- **Round 4** — art (Codex): the carved pillar (a hooded knight's alcove, banner, sconce) behind the desktop's side columns and the braziers
  at the vista's feet; the edit's scene is quiet (one name, no plate over the hero); a boss keeps off the hero (`BOSS_COVER` 0.1); the guard's
  break splits his shield over him (`tags.ts` `shatter`); the shaft a size up.
- **Round 5** — a boss drops into his arena (`fx.ts` `lift`, 3 ticks, then the landing squash); a small foe steps out from behind the hero;
  gas tiles drift as smaller blobs inside denser puffs; the camera leans toward the seen ground when no foe frames it; glints and mist on
  visible water; sheets without the backdrop blur (it cost their first frame ~0.2 s).
- **Round 6** — damage numbers are outlined DOM glyphs (the bitmap `80` read as `$0`); a plate that would climb over more than two other plates
  is left off (a crowd keeps three rows of names); a small view (< 320 CSS px: the scene) lets the hero take 2/5 of its height; the viewer waits
  ≤ 1.5 s for the atlas before its first frame (the scene showed the primitive fallback); rising sparks off the braziers.

### 10.4 The must-fix and the IA additions

- **WebGL context loss** (`render/index.ts`, new `render/view2d.ts`, `render/fallback.ts`): `webglcontextlost` is prevented, the view drops to a
  Canvas-2D drawing of the same replay state (atlas tiles and sprites, hero light) while the clock, `▶▶|`, the beats and the exit run on;
  `webglcontextrestored` rebuilds (a new GPU timer, textures and the light mask re-uploaded, normals re-derived) and hides the 2D view. A
  renderer that cannot be created mounts the 2D viewer with the full Viewer API (the old placeholder had no clock: `▶▶|` died on it). Disposal
  now forces the context loss, so the scene's and the stills' contexts are freed at once (Chrome drops the oldest past ~16 — a likely cause
  of AW's). `web/tests/ctxloss.mjs`: loss mid-watch → 2D view, ticks advance, restore → GL again; loss → `▶▶|` reaches the exit; a page with no
  WebGL at all → the run reaches its exit.
- **Desktop** (`web/src/wide.css`, `ui/frame.ts` `wideCols`, `ui/sheet.ts` `placeWide`): ≥ 1024 px the frame is one grid edge to edge — the
  rules column left, the well centre, the shaft right with the meters' slot under it, the console across the bottom, the top bar unchanged;
  sheets open beside their tablet. Phones are untouched (everything inside the media query).
- **Mobile**: the layout gate holds the console, portrait and gem visible and uncovered on every screen, sheet and the scene.

### 10.5 Audio (`scratchpad/gfx-eval/round6/audio.json`)

Unchanged cues measure as §7.1 (0 clipped samples; the three-cue stack −0.6 dBFS before the limiter; 12 live strikes, 0 consecutive
identical). New: the bed ducks 9 dB (40 ms down, held 0.6 s, back over 0.9 s) under `boss_in`, `boss_break`, `boss_down`, `verdict`, `exit_*`.

### 10.6 Frame times

Captured on a machine another agent held at load 30–70 on 32 cores for the whole session, so no number here is a clean claim: round 0 (load
13–30) read p50/p95 16.7/16.7–16.8 on every phone moment and the desktop watch; round 1 (load 15–20, after the ember layer was moved to a
compositor transform — a blended background-position animation had taken the camp's p95 to 33 ms) read 16.7/16.7–16.8 on every phone moment,
the desktop watch 16.7/33.4; round 6 (load 40–69) reads 16.7/16.7–16.8 on the camp, death, Fens, fight and boss, 33/50–83 on the Warrens watch,
the report and the desktop watch — and the untouched `low` tier read the same 33/50 at 1440 × 900 in the same minute, so load, not the tier,
sets those. The `low` tier's shader path is unchanged (the rock is `FX > 0`). A quiet-window run of `tools/gfx-round.sh` is owed.

### 10.7 What blocks the next gain (why rounds 4–6 stalled)

1. **Sprite scale vs tile scale** (every rater, every round: "the hero is 2–3 tiles tall, hides the foe he fights", "sprites at twice the
   tiles' pixel density"). The sprites are authored at 2× the tiles; shrinking them in the renderer breaks the pixel grid. Needs an art call
   (re-author the bestiary and heroes at tile density, a Codex batch of ~60) or a design call on the dungeon's zoom.
2. **The black of the unexplored floor** ("half the view is void"). Fog of war is game truth; the rock texture and the camera's lean helped
   (+0.40 on the Warrens watch). More needs a design call (a lit fog texture art pass, or a tighter frame per room).
3. **Motion on still screens** (camp, report, desktop: 2.5–4 every round whatever the flicker, embers, sparks). Four frames 180 ms apart do
   not show ambient life at half size; the raters score these screens as "frozen". Needs either larger ambient motion (a feel risk the owner
   should rule on) or a rubric change for screens that should be still.
4. **The report's arrival** (motion 2–3): its plaques render ~0.6 s after the screen mounts (the report's own async content), so the strip
   is a dim, empty parchment. A client/engine timing fix (report.ts).
5. **Copy-bound asks** the copy law forbids a visual pass to answer: `BROKEN`/`SLAIN` plates, the report's four-line head, the D8 row's
   words, `fired 19/13754 turns`, the fix tablets' three numbers, `highlig…` truncated in the console. Needs the copy owner.
6. **Fake-engine moments**: the Fens and the fight are shot from the fake engine (the real one never reached D5 in a scripted run); its
   crowds pile on the hero. A dev fixture (a real save that starts at D6) would rate the real game.

### 10.8 Rounds 7–9 (2026-09-28, resumed from the HANDOFF list): sprite scale, real saves, arrivals, the scroll — stopped again on the rule

| moment | r6 | r7 | r8 | r9 |
|---|---|---|---|---|
| watch-warrens | 5.75 | 5.60 | 6.25 | 6.20 |
| report | 5.35 | 6.10 | 5.90 | 6.10 |
| death | 7.20 | 6.90 | 6.95 | 7.05 |
| camp | 5.40 | 6.00 | 5.90 | 6.40 |
| scene | 5.45 | 5.00 | 5.50 | 5.85 |
| edit | 6.10 | 6.20 | 6.25 | 6.10 |
| forecast | 6.35 | 6.40 | 6.25 | 6.10 |
| oaths | 6.25 | 5.80 | 5.85 | 5.50 |
| watch-fens | 5.05 | 5.70 | 5.25 | 5.55 |
| fight | 5.55 | 5.30 | 6.55 | 6.70 |
| boss-in | 7.00 | 6.80 | 6.80 | 6.90 |
| boss-break | 6.80 | 6.40 | 6.00 | 5.60 |
| boss-fall | 6.65 | 6.80 | 6.40 | 6.10 |
| d-watch | 5.90 | 5.90 | 6.35 | 5.75 |
| d-report | 5.00 | 6.20 | 5.60 | 5.75 |
| d-death | 6.60 | 6.70 | 6.50 | 6.25 |
| d-camp | 5.75 | 6.70 | 6.15 | 6.35 |
| d-edit | 5.35 | 5.80 | 6.15 | 5.85 |
| **mean** | **5.97** | **6.13** | **6.14** | **6.12** |
| min | 5.00 | 5.00 | 5.25 | 5.50 |

Raters Q R (r7), S T (r8), U V (r9), fresh each round; the two raters' means agreed within 0.3 every round. The rounds moved +0.16, +0.01 and −0.02,
so the stop rule applies again. The Fens and fight moments are real saves from r7 on, and the boss demo runs at the phone's 100 texels, so
those rows are not strictly comparable with r0–r6.

- **Sprite scale** (the #1 ask, rounds 0–6): `SPRITE_SCALE` (`render/palette.ts`, dev `?sprite=`) multiplies every entity's runtime size, cut
  down by area on the sprite grid (`atlas.ts putDown`, the procedural fallbacks included); `HERO_TEXELS` and `view2d.ts` follow it (the 2D
  view had drawn sprites at 2×: `texel_h` now comes from the atlas meta). A blind pick (raters O and P, four candidates: 1.0 at 100 texels,
  0.62 at 80, 0.5 at 66, 0.5 at 100) put **0.5 at the watch's zoom unchanged** first for both ("one world at one density"). The fight
  frame's zoom is ≤ 1.25× the map's (2× smeared the half-size sprites). The desktop's `DESK_TEXELS` is 112 → 84, and a view under 320 CSS px
  (the edit's scene) shows ~half the texels. After a hero-cover push, a stack keeps its fan (fights.mjs's stack checks).
  "Scale" was no longer named in r8–r9.
- **Real saves** (`tools/gfx-eval.mjs phoneDeep`): `web/tests/fixtures/deep.json` imported, `setStart(9)` gives the Fens at D9 and `setStart(5)`
  gives a fight at D6, shot in the fight frame with a foe in view and no title card. `--q` / `--qdemo` pass dev params. `d-edit`'s strip is
  now its opening.
- **The report's arrival**: the strip was a dim, empty parchment because the GPU rasters a new screen's first frame for ~0.5 s on this box
  (LoAF: one 450–500 ms render frame; trace: `RasterDecoderImpl::DoEndRasterCHROMIUM` 400+ ms on the GPU process; no main-thread long
  task). The document timeline does not advance meanwhile, so every fade-from-0 arrival sits invisible, then lands already finished. A
  screen now gets `.arrived` two frames after it mounts (`juice.ts arrivals`). The report's plaques and lines draw at rest and hop or nudge
  once the frame is presented. Plaques are visible at 33 ms (they were at 0 opacity until ~600 ms). The oath board's opening has the same
  stall: its strip showed "only the camp dimming" in r9 (motion 3.5–4.5).
- **The lit fog edge** (`blit.ts`, FX > 0): the rock beside explored ground takes the light of the seen texels next to it (8 directions,
  1–3 half-tiles out; only drawn world texels lend light, so an unseen room never shows through). A 1-texel dark outline goes round every
  sprite. The Fens grade is darker and less saturated.
- **UI**: the report is a hanging scroll (`art/ui/frames/scroll.png`, Codex, QC'd; 9-slice rolls in `tools/ui-skin.py` → `skin-scroll`).
  Sheets carry riveted iron corner brackets and a deep shadow. A rule's ordinal sits in a recessed stud. Oath seals sit whole inside the card,
  rewards in icon plaques, a short purse in red. The four-stat topbar draws a size down at ≤ 440 px (a D11 save ran `best` under the gear).
  The death and report wells fade at their foot instead of cutting a text line. `by heirs 2–14` stays on one line. Damage numbers start over
  the shoulder, not the face.
- **Still screens**: live flame tongues and a stepped fire-glow pool on the vista's braziers (`i.j-flame`/`i.j-glow`, container units,
  transforms and opacity only), and embers a size up and quicker. Camp motion went 4 → 5.5 at best, but the desktop camp and report still
  read "static" at the strip's half size.
- **Harness**: `tools/gfx-round.sh` leaked its Vite server every round (`setsid` forked, and the trap's group kill missed it). The subshell
  now `exec`s it.

Frame times: a quiet window at the start of the session (load 0.5; `round7-base`) read p50/p95 **16.7/16.7–16.8 on every phone moment**
(watch, Fens, fight, boss, camp, report, death). The desktop watch read 33/33.5 as the load rose to 23. Rounds 7–9 were captured at load
35–100 (other agents' gates) and read 33/50–67; the viewer's CPU stays at 1.6–1.9 ms, and the added shader work (edge light, outline) runs on
a ≤ 204 × 294 target. A quiet-window desktop measurement is still owed.

**What blocks the next gain** (the same plateau: ~6.1, with rater noise about ±0.2):
1. **Art that CSS cannot make**, asked for by every rater: corpse and death sprites for bosses ("a dark smear"), translucent spectral wraiths
   ("grey static blobs": the renderer has no per-sprite alpha, since alpha is the layer tag), props in rooms (barrels, banners, bones), a
   literal shield-shatter graphic with shards, a painted backdrop behind the report's scroll (candles, helmet, gold) as in `report.png`,
   killer portraits. This needs a Codex batch of ~15–25 assets plus renderer hooks (a death pose frame, an ethereal flag).
2. **The GPU raster stall on this box** hides every opening animation (report, oath board, sheets on desktop: motion 3.5–5). A real device
   may not stall. The rubric measures it here, so either rate on a machine without the WSL D3D12 raster path, or pre-raster (keep screens
   mounted and hidden) — a client architecture call.
3. **Layout and IA asks** outside the gfx remit: the report's nine `OPENED` chips before the stat plaques (hierarchy 5–5.5 every round);
   the forecast as the camp's hex-gem shaft, not web bars; the empty `?` reach column on the desktop report; the scene overlaying the rules
   mid-row; the desktop death banner's `D8 · NO` wrap; the WHY sheet anchored to its tablet.
4. **Boss break and fall** fell 6.8 → 5.6 and 6.65 → 6.1 at half-size sprites: the break's shield and the fall read small. This wants
   bigger boss sprites (a boss-only scale, e.g. 0.75) and the break/slain plates. The copy owner must approve `BROKEN`/`SLAIN` as words.

### 10.9 Rounds 10+ (2026-09-28, unblocked by the coordinator: an art batch, a boss-only scale, `.arrived` everywhere, `BROKEN`/`SLAIN`)

- **Art (Codex, each QC'd by eye; art-qc 0/0)**: six boss death poses (`boss_<kind>_dead`, 1536×1024 masters → atlas `ent:<kind>_dead`), six
  room props (`env_skulls`, `env_chest`, `env_rack`, `env_statue` in each biome's ramp; `env_brazier`, `env_candles` keep their fire —
  `make_env.py` HUE), the boss's painted shield and four shards (`art/ui/fx` → `web/public/ui/fx`, `ui-skin.py` `fx`), and a painted room
  behind the report's scroll (banner, sword and helmet on gold at the left, a torch, candle, books and skull at the right). Killer
  portraits are cut from the monsters' own masters (`tools/foe-portraits.py` → `web/public/ui/foes`, skin.json `foes`: 37 heads).
- **Renderer**: bosses at `BOSS_SCALE` 0.75 (`palette.ts spriteScale`; the atlas, the procedural fallback and the 2D view); a fallen
  boss lies in his death pose from 4 ticks after the blow for the rest of the floor (state keeps his body; no art → he dissolves as before);
  ethereal kinds (wraith, mirror shade, echo, siren, spectral blade and hound) draw in their own layer (`layers.ts` ghost: a 50/50 blend
  whose alpha stays the sprite tag, cold-tinted, bobbing) with a faint cold light in the field — FX > 0 only, `low` unchanged; the new props
  dress corners, north walls and big rooms, a brazier and candles light the field.
- **UI**: `BROKEN` and `SLAIN` stamps slam over the boss (`tags.ts stamp`), the break throws the painted shield's shards; the killer's
  portrait hangs in an iron medallion on the death banner; sheets and the oath board are drawn at rest in their first frame and settle / deal
  in once it is presented (`.arrived` on `.sheet-wrap`, juice.ts).
- **Harness**: the boss stills are shot before their strips (the strip's 1.5 s outlived the shatter and the stamp).

| moment | r9 | r10 | r11 |
|---|---|---|---|
| watch-warrens | 6.20 | 5.65 | 6.10 |
| report | 6.10 | 6.45 | 6.60 |
| death | 7.05 | 6.90 | 7.25 |
| camp | 6.40 | 6.00 | 6.15 |
| scene | 5.85 | 5.55 | 6.00 |
| edit | 6.10 | 6.05 | 6.35 |
| forecast | 6.10 | 5.95 | 6.30 |
| oaths | 5.50 | 6.05 | 6.65 |
| watch-fens | 5.55 | 5.20 | 5.35 |
| fight | 6.70 | 6.05 | 6.30 |
| boss-in | 6.90 | 6.90 | 7.05 |
| boss-break | 5.60 | 6.35 | 6.90 |
| boss-fall | 6.10 | 6.65 | 6.95 |
| d-watch | 5.75 | 5.55 | 5.90 |
| d-report | 5.75 | 5.85 | 6.15 |
| d-death | 6.25 | 6.35 | 6.95 |
| d-camp | 6.35 | 5.95 | 6.45 |
| d-edit | 5.85 | 5.85 | 5.90 |
| **mean** | **6.12** | **6.07** | **6.41** |

Raters W, X. The targeted moments rose (boss break +0.75, fall +0.55, oaths +0.55, report +0.35); the watch moments fell on capture
variance (the run's frame differs each round: W and X both saw "the top half black void" and a "hero smudge" in this round's D1 frame).

**Round 11** (raters Y, Z; **6.41**, +0.34, the session's best — lowest 5.35, the Fens): the world's contrast per biome (`blit.ts CONTRAST`,
FX > 0: the Fens 0.6, the Warrens 0.78 — "the floor is louder than the actors"), the rock mass kept in view further out (the void's
floor 0.15 → 0.3, fade over 11 tiles), the hero's hurt flash 0.28 → 0.16, the edit's scene in the console's riveted bezel with a vignette,
the death backdrop lifted, the report's deepest plaque gilt. Death 7.25, boss entrance 7.05, fall 6.95, break 6.90, oaths 6.65. The fights
suite's `fast: the break is a beat` check fails only when run beside three other suites (load); alone it passes (48/48).

**Rounds 12–14** (raters AA AB · AC AD · AE AF): 6.16 · 6.28 · **6.50** (the session's best; lowest 5.60, the Fens).
- r12: a drifting ground mist (value noise in world space; the Fens 0.42, the Crypt and the Deep lighter; FX > 0), ethereal foes drawn
  violet-white at ~2× so a 50/50 blend still reads over the Fens' teal, damage numbers step clear of the callout and name plates, the scene
  inset zoomed further (a view < 320 CSS px shows 0.45× the texels).
- r13: every sheet and camp panel is the report's parchment scroll (`skin-scroll`: rolls across, ragged sides, a drop shadow); the Fens'
  floor contrast 0.45.
- r14: the report's deepest floor is the hero plaque (first, across the row, a size up, gilt — the other six pair up, no empty slot); the
  oath cards drop in (the round-13 tilt read "skewed, glitchy"), their seals press less; `measuring` fits inside the APPLY gem; desktop
  embers a size up (the strip is a quarter size at 1440).

| moment | r11 | r12 | r13 | r14 |
|---|---|---|---|---|
| watch-warrens | 6.10 | 5.70 | 5.70 | 6.20 |
| report | 6.60 | 6.30 | 6.45 | 6.70 |
| death | 7.25 | 6.90 | 7.05 | 7.10 |
| camp | 6.15 | 6.25 | 5.95 | 6.25 |
| scene | 6.00 | 6.00 | 6.05 | 6.05 |
| edit | 6.35 | 6.20 | 6.40 | 6.75 |
| forecast | 6.30 | 5.75 | 6.15 | 6.35 |
| oaths | 6.65 | 5.75 | 5.95 | 6.30 |
| watch-fens | 5.35 | 5.25 | 5.65 | 5.60 |
| fight | 6.30 | 6.40 | 6.15 | 6.50 |
| boss-in | 7.05 | 7.05 | 7.10 | 7.15 |
| boss-break | 6.90 | 6.65 | 6.65 | 6.85 |
| boss-fall | 6.95 | 6.35 | 6.80 | 6.90 |
| d-watch | 5.90 | 5.45 | 5.70 | 6.00 |
| d-report | 6.15 | 5.95 | 5.70 | 6.20 |
| d-death | 6.95 | 6.60 | 6.85 | 6.85 |
| d-camp | 6.45 | 6.45 | 6.40 | 6.65 |
| d-edit | 5.90 | 5.95 | 6.35 | 6.60 |
| **mean** | **6.41** | **6.16** | **6.28** | **6.50** |

**Rounds 15–17** (raters AG AH · AI AJ · AK AL): 6.26 · 6.43 · 6.49 — stopped on the rule (three rounds under +0.2 after r14's 6.50).
- r15 (reverted in part): a Codex redraw of the Fens floor as two calm planks per tile took the Fens to 5.00 ("hero and floor one murky
  teal") — reverted, as was an edge-only hurt vignette; kept: the shield splits from 20 % of its fall, over the boss's head, BROKEN above it.
- r16: the forecast's bars as brass troughs with a molten fill and a hot tip (cut29.css had thinned them to 4 px), a gilt iron EDIT, the
  oath rewards a size up, the hero's own light warm in every biome and a touch brighter, a darker Fens room, a white burst on a boss's
  killing blow.
- r17: in the Fens a sprite takes less of the light's hue (0.15, was 0.5: "a teal hero on a teal floor"); two first-time captions under
  neighbouring top-bar stats take two rows.

| moment | r14 | r15 | r16 | r17 |
|---|---|---|---|---|
| watch-warrens | 6.20 | 5.65 | 5.85 | 5.85 |
| report | 6.70 | 6.45 | 6.80 | 6.60 |
| death | 7.10 | 6.75 | 7.15 | 7.10 |
| camp | 6.25 | 6.05 | 6.25 | 6.25 |
| scene | 6.05 | 6.10 | 6.10 | 6.05 |
| edit | 6.75 | 6.55 | 6.40 | 6.70 |
| forecast | 6.35 | 6.10 | 6.35 | 6.90 |
| oaths | 6.30 | 6.10 | 6.40 | 6.15 |
| watch-fens | 5.60 | 5.00 | 5.30 | 5.40 |
| fight | 6.50 | 6.15 | 6.65 | 6.40 |
| boss-in | 7.15 | 7.10 | 7.45 | 7.40 |
| boss-break | 6.85 | 6.65 | 6.70 | 6.90 |
| boss-fall | 6.90 | 6.75 | 7.05 | 7.00 |
| d-watch | 6.00 | 5.70 | 5.85 | 5.95 |
| d-report | 6.20 | 6.10 | 6.20 | 6.45 |
| d-death | 6.85 | 6.50 | 6.45 | 6.75 |
| d-camp | 6.65 | 6.65 | 6.45 | 6.55 |
| d-edit | 6.60 | 6.35 | 6.35 | 6.40 |
| **mean** | **6.50** | **6.26** | **6.43** | **6.49** |

Session arc: 5.97 (r6) → 6.49 (r17), best 6.50 (r14); lowest 5.00 → 5.40. Moments at or over 7: death 7.10, boss entrance 7.40, boss fall
7.00. Frame times: see §10.8 (phone 16.7/16.7–16.8 on a quiet box); the added shader work (ghost layer, mist, contrast, edge light) runs on a
≤ 204 × 294 target and the viewer's CPU stays ~1.6–1.9 ms.

**What blocks the next gain (the bar is 8.0 / no moment < 7.0; the remaining asks are the same every round):**
1. **The watch's own look (5.4–6.2, the floor of the table)**: raters want the painted target's lit stone rooms — "the floor is flat sepia
   mush", "the Fens a teal soup", "half the view black void". The 8×8-texel tile register at the pick raters' zoom cannot carry that
   detail; a zoom/letterbox that frames the seen rooms, or a new tile register (painted 16×16 at 1:1), is a design call. A first Codex
   redraw of the Fens floor (calmer planks) scored worse and was reverted.
2. **IA on every UI screen** (hierarchy 5.5–6.5; Cut 29 client): the report's two-line title and four sub-lines, the `OPENED` chips cut off
   at the fold, the forecast's D8 row, the scene inset overlapping the rules, the WHY sheet not anchored to its tablet, the desktop meters'
   clipped text ('trav', '3 hit'), empty console slots.
3. **Still screens' motion** (desktop camp/report/death 4.5–5.5): the strip at quarter size cannot see ambient CSS motion; the camp is
   being replaced by the Cut 30 town, so no more effort there.
4. **Art the raters keep naming**: a boss HP bar under the name plate, a loot/coin burst on a boss kill, a stagger pose, killer portraits
   in the forecast's KILLERS line, death-scene sprites behind the banner (the killer medallion helped: desktop death target 7.5–8).

### 10.10 Rounds 18–20 (2026-09-28): the painted 16×16 tile register (the owner's call) and the Cut 29 client's UI list — stopped on the rule

**The register** (`art/painted.py`): 15 pieces per biome (4 floors, 2 wall faces, wall top, door, stairs down/up, water, chasm, barrel,
crate, pot) × 7 biomes = 105 Codex paintings in the target's lit-stone language (the Burrows warm earth and timber, the Fens a cool
teal boardwalk — kept apart; the Crypt blue granite, the Foundry soot and ember seams, the Deep navy cave rock, the Sanctum cream marble),
each QC'd by eye as a sample room at game zoom (`scratchpad/gfx-eval/paint/room.py`). Conversion (`make_env.py` → `painted.convert_all`):
a box downscale to 16×16, a small local-contrast lift, ≤ 24 colours per tile (art-qc reads `art/tiles/_painted.json` for that cap), written
over the ramp register's `<biome>_env_<name>` — the renderer's ids are unchanged, the 8-colour ramp tiles stay the fallback for any piece
not painted, and a tile stays 16 texels on the sprite grid (1:1 with the sprites; the hero 1.5 tiles at SPRITE_SCALE 0.5, bosses 0.75 —
no zoom change was needed). The wall tops were redrawn once as lit capstone masonry (the first pass, "the darkest", read as void); the
renderer darkens them a step under the floor and lights a one-texel bevel on every edge that meets open ground. The Warrens' grade is now
neutral (the old one was tuned for the ramp), the ramp decals at 40 %, the Fens' torches lantern-warm and its mist lighter.

**UI and asks**: the report's headline balanced to its width, its other lines one paragraph, OPENED as small plaques; the forecast's try row
keeps only whose floor it is (the pill tighter) and KILLERS carry the foes' portraits (unmet ones stay faceless); the scene plays over the
phone's vista, clear of the rules; the desktop meters wrap; no empty console slots on desktop; a boss's name plate carries a long framed
HP bar; the break is a stagger (a big squash-reel) and the shield halves fly from 6 %; the fall throws twice the coins; a one-texel walking
bob; fewer blood drops off the hero; frames and backdrops decode at boot. (The WHY sheet was already anchored by the Cut 29 client.)

| moment | r17 | r18 | r19 | r20 |
|---|---|---|---|---|
| watch-warrens | 5.85 | 5.70 | 5.65 | 6.20 |
| watch-fens | 5.40 | 5.40 | 5.45 | 5.70 |
| forecast | 6.90 | 7.05 | 7.05 | 7.10 |
| death | 7.10 | 6.95 | 7.10 | 7.20 |
| **mean** | **6.49** | **6.31** | **6.39** | **6.23** |

Raters AM AN · AO AP · AQ AR2. The dungeon moments moved (the Warrens watch 5.85 → 6.20, the Fens 5.40 → 5.70: "painted walls with lit
capstones now read as built") but the mean did not: three rounds under +0.2, so the eval stops again.

**What blocks the next gain** (every rater, every round):
1. **Motion on the still screens** scores 3–4.5 on desktop (camp, report, death) — the strip cannot see ambient CSS at quarter size; the
   camp becomes the Cut 30 town.
2. **The oath board** (5.7): "three identical cards, big empty wood" — a layout/content change (reward art per oath) the board's owner
   should take.
3. **The painted shield over pixel sprites** ("a style clash", "hides the boss"): a pixel-register shield at sprite scale.
4. **Coins as square motes** ("debug pixels, not coins"): a coin sprite for the particle system.
5. **The dungeon**: raters now ask for set dressing density and wall shadows (the rooms read "bare grey tile fields"), not the tile
   register itself.

### 10.11 Round 21 and the motion-aware harness (2026-09-28; stopped here for the owner's pause)

**The harness now sees motion** (`tools/gfx-eval.mjs`, `tools/gfx-rater-prompt.txt`): the screencast runs at full device resolution and each
moment gets a `-motion.png` — its first frame beside (phone) or above (desktop) the same frame dimmed with every pixel that changed within
~1 s in red — and the prompt lists what each moment is meant to move (`moves: …`). Round 20's build re-rated under it (raters AS, AT) is the
new baseline: **6.74** (the old harness read 6.23) — death 7.90, boss entrance 7.70, forecast 7.40, report 7.10, desktop camp 7.10; rounds
before 20b are not comparable with it.

**Round 21** (raters AU, AV): **6.68** (−0.06 on the baseline; rater spread 6.49 · 6.87). Committed (c6008ad): five more painted props per
biome (skulls, chest, rack, statue, bones — 35 Codex pieces), denser dressing at corners and walls, bones on open floor, a deeper wall-foot
shadow (AO 0.32 → 0.5, the wall above weighted 2.2); the boss's shield and shards cut to the sprites' pixel register (`_px`, pixelated);
the loot burst's coins a spinning 6×6 coin sprite; a prop-conversion fix (a wrapped rim drew a stray dash over every prop). The oath board's
reward vignettes were built and then withdrawn (the board is redesigned as a quest board in Cut 30); their Codex sources stay uncommitted
in `art/ui/oath/` for that cut.

| moment | 20b (baseline) | 21 |
|---|---|---|
| watch-warrens | 5.60 | 6.05 |
| watch-fens | 5.40 | 5.50 |
| fight | 6.50 | 6.70 |
| boss-break | 6.90 | 6.30 |
| boss-fall | 7.00 | 7.05 |
| death | 7.90 | 7.55 |
| **mean** | **6.74** | **6.68** |

**Next steps** (in order): (1) the Fens (5.50) — raters still read a "flat undressed teal field": dress it with its own props (reeds,
ruins, water edges) and warm the hero's pool further; (2) the boss break fell with the pixel shield ("shards hard to parse", "the shield over
the boss muddles him") — offset the split above him, bigger halves in clear arcs, a crack flash; (3) the camera framing on early floors
("the void eats 40 %"); (4) the IA asks the raters repeat (the report's OPENED chips, the scene inset over the route row, desktop meters);
(5) the oath board → Cut 30's quest board.
