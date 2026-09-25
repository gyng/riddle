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

## 5. Deferred

- Codex normal maps for heroes and bosses (the hero-looks sprites land first).
- Sheet close easing (the sheet is removed synchronously by `ui/sheet.ts`; an exit animation needs a hook there).
- A measured phone GPU number (the timer query never resolves in this harness).
