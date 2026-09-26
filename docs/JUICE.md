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
