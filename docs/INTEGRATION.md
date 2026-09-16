# Integration — the real game against the wasm engine (2026-09-16)

Scope: make `web/` run end-to-end in the browser against `crates/riddle-wasm` (built into
`web/src/engine/pkg`), fix every wire mismatch, move the engine into a Web Worker, walk the whole
loop with the GPU harness and record what broke. No Rust game rules were changed; no wasm bridge
methods were needed beyond what the core already exposes.

## Architecture after integration

```
main thread                          engine worker (web/src/engine/worker.ts)
  app.ts  ── AsyncEngine proxy ──►    WasmEngine (wasm.ts) over pkg/riddle_wasm.js
  (proxy.ts: one message per call,    JSON strings in/out; calls answered in order
   answered in order; ?engine=fake     (the engine is synchronous, so nothing interleaves)
   wraps the fake in the same shape)
```

- `engine/types.ts` adds `AsyncEngine` (every `Engine` method returning a Promise), `UnlockInfo`,
  `Death.baseline`, the `projectile` event, and the core's interface additions
  (`unlocks() setClass() selectSet()`).
- `engine/proxy.ts`: `workerEngine()` (spawns the module worker, `init` handshake returns the wasm
  version or null → fake fallback) and `asyncify()` for the fake.
- `app.ts`: fully async; the engine's `lineage.sets` are the source of truth for the three set
  tabs (the localStorage blob is now `{v:2, engine, loadout, last_seen}`; v1 blobs still load,
  their client-side sets are ignored). `persist()` debounces `save()` 1 s; `pagehide` /
  `visibilitychange` write the last fetched save string synchronously (a worker cannot be awaited
  there). `busy(label, fn)` shows the thin progress bar (`ui/progress.ts`, indeterminate — a wasm
  call cannot report progress) for `offline`, `forecast`, `verdict`.
- `ui/watch.ts`: the viewer owns the clock (10 ticks/s × speed). The worker is pumped every
  100 ms in 10-tick batches whenever the engine is fewer than 12 ticks ahead of the viewer, so
  the engine never runs more than 22 ticks ahead (under the renderer's 30-tick dead-air jump).
  HUD hp/depth and the ticker are queued by tick and released at the viewer's clock. A `descend`
  splits the batch: the pre-descend events play out, then the new floor snapshot loads and the
  rest is applied. Exits wait for the viewer to drain (≤ 4 s) before the exit sheet / death /
  report. Persist every 5 s during a run (a save is ~1.3 MB: 40 death records dominate).

## Mismatches found and fixed

| # | Mismatch | Fix |
|---|---|---|
| 1 | Client unlock ids (`class:rogue`, `card:*`, `verb:throw`) vs engine ids (`rogue`, `corridor_fighting`, `throw`, …). Class switch was modelled as "buy an owned class". | Camp and report read the engine's `unlocks()` catalogue; `ui/unlocks.ts` only keeps labels and the display chain (row6 hidden until row5 owned). `available` from the engine (prereq + fact gate + affordable) enables the card. Class switch calls `setClass`. Fake engine ids aligned, `unlocks/setClass/selectSet` added to it. |
| 2 | Fractions vs percent. | The client already rendered `x*100`; verified against real numbers (`reach 0.96` → `96%`). Death patches now show `survive%` and the forecast delta; `Death.baseline` shown as a small grey `base 75%` under the patches (copy-tagged `label`). |
| 3 | Ticks. | Trace `t` is ticks; the death table shows the last 5 trace rows (one row per hero action, 10 ticks apart at base speed; chore rows render as `·`). Watch pacing rewritten (above) — the old loop stepped one tick per second at 1×, ten times too slow. |
| 4 | Unknown item kinds `potion` / `scroll`. | `ui/salvage.ts` mirror prices them as their class (8 / 12). |
| 5 | Sets. | `selectSet(i)` on the engine, then `setRules` for the tab; sets adopted from `lineage.sets` on load/offline/import. |
| 6 | Chore rule events. | Ticker shows player rows (`R2 · attack nearest`), traits (`cowardly → retreat`), and one-word `descend` / `pick up`; other chores silent. Engine `callout` events always show. Repeated identical rule callouts within 4 s are deduped. |
| 7 | Worker. | As above. `?engine=fake` wrapped by `asyncify`; the fake's `step` now takes ticks (it sims one action per 10 ticks and scales `t`) so the viewer paces it correctly. |
| 8 | `render/types.ts` local `projectile`. | Moved into `engine/types.ts`; `render/types.ts` is re-exports only. Ev union checked against `wire.rs` — all 22 kinds present. |

Other fixes made while looking at the screenshots:

- `.chip` is a flex container, so the whitespace between label and price collapsed (`dagger$3`,
  `leash$30`) → `gap: .4em`.
- Camp strip overflowed the phone width once the numbers grew (`♟70 cowardly fighter L4 D6 ★8
  $4360 ◆27`), causing horizontal page scroll → strip wraps, 13 px, `#app { overflow-x: hidden }`.
  Class name and level were on separate lines inside the class button → one line (`fighter L4`).
- Egg chips (`◯ goblin archer ranged telegraph g0 $50`) overflowed the phone width → wrap.
- The patch-inserted row's highlight was lost because the camp repaints the editor right after
  mount → highlight survives repaints for 2.4 s.
- Forecast panel was blank until the worker answered (~1 s) → shows the `?` row meanwhile.
- Report: 48 fact chips after 8 h were three phone screens tall → grouped per foe
  (`goblin archer · ranged · telegraph`), items as `heal (green)`, chips 32 px; identical reel
  lines and tamed/hatched chips collapse to `×n`; bests keep only the highest `rank N`,
  `fighter LN`, `DN`; underscores removed everywhere (`goblin_archer` → `goblin archer`).
- Report / death: engine errors (unaffordable unlock, invalid rules) are caught and warned, never
  thrown to the console.

## Measurements

Machine: WSL2, 32 cores, but shared — load average 29–34 during every measurement (other agents
building). GPU harness `tools/browser.mjs` (headed Chromium, D3D12 → RTX 3080).

| What | Result |
|---|---|
| `runOffline(8 h)` in the worker | 9.5 s / 12.4 s (fresh lineage, 62 runs, dev build, headed); 10.5 s headless. Node on the same wasm at the same moment: 5.9 s; on an idle machine earlier in the session: 3.7 s. Absence → report screen end-to-end (navigation + load + batch + verdict) measured 4.2 s and 8.2 s on lineages with ≤ 2 prior runs. |
| `forecast()` in the worker | 1.2–1.9 s (50 sims; 0.5 s native). Main thread stays at 65 fps rAF while it runs. |
| Watch FPS (rAF over 5 s) | 400×800 @3: 33 fps at 1× and 4×. 1280×800 @1: 18–25 fps. The bare `render-demo.html` gives the same numbers (33 / 19) and the wasm engine with the Canvas-2D placeholder view gives 57 fps, so the cap is in the three.js pixel pipeline / GPU present under WSLg, not in the engine or the worker: a CPU profile of the demo shows the main thread 93% idle. Camp screen: 60 fps. |
| `step(10)` round trip | 7 ms (incl. structured clone of a 12 kB snapshot). |
| `save()` | 44 ms for 1.45 MB (worker) — hence the 1 s debounce and 5 s cadence during a run. |

## Walk (all screenshots in `docs/shots/`, `*-phone` = 400×800 @3, `*-wide` = 1280×800 @1)

1. `01-camp-*` fresh lineage: fighter preset (2 rows), forecast `D1 100% · D2+ ?`, causes
   jackal/goblin/ogre from the real 50-sim forecast, unlock cards from `unlocks()`.
2. `02-watch-*`, `03-watch-*`, `04-watch-4x` send → hero moves, tiles reveal, ticker `pick up`,
   `descend`, `R2 · attack nearest`; HUD `24/36 D1 !!`.
3. `04-exit-sheet-*` the run's carried items offered for the vault at the exit (`dagger $3` at the
   death tier), `keep` → `keep()`.
4. `05-death-*` `goblin · D2 · 3 over · 1 unknown unused · gap`, last-5 trace with tick `t`,
   three patches with survive % and forecast delta, `base 75%`.
5. `06-camp-patched-*` tapping a patch inserted `hp < 50% → shield bash` at R1, highlighted;
   forecast re-ran (busy bar `FORECAST` visible top-right).
6. `08-busy-*` reload after 8 h of simulated absence (`last_seen` shifted): blank page + bar
   `OFFLINE`; `09-report-*` 69 runs · 69 deaths · D6 · ◆+27, learned, bests, xp `fighter +1138 ·
   L4 ↑3`, deaths, salvaged, renown `+7391 · ★8 ↑8`, pending, reel; `open` → `10-worst-death-*`.
7. `11-camp-rich-*` forecast to D7 with `D8+ ?` (`known_to = best+1`), supplies catalogue now
   lists identified potions; `12-camp-unlocked-phone` after buying `row5` and `throw`
   (max_rows 5, `throw caustic/confusion/fire` in the vocabulary); tab 2 selected and back.
8. Companions: `13-camp-tame-rules-phone` (`foe hp < 25% → tame nearest` + rest/retreat set after
   buying `tame` and `party_slot_2`), a second 8 h absence → `14-report-pets-phone` tamed ×5,
   hatched ×3, lost ×5 (eggs), best D16; `15-camp-party-phone` kennel cards, eggs with `$50`
   hatch; `16-ledger-phone`; `17-companion-rules-phone` (`foes ≥ 1 → shoot`, `adjacent ≥ 1 →
   attack`, 2/2 rows); `21-watch-party-phone` run resumed mid-D4 with a companion in the party.
9. `18-forge-phone`, `19-class-sheet-phone` (rogue greyed until unlocked), `20-settings-phone`
   (`wasm 0.1.0`), `22-watch-skipped-phone` (▶▶| ×4), pause/resume stable, `23-exit-sheet-bail-phone`
   → `24-report-after-bail-phone` (`1 run · 0 deaths · +63 xp · Returned with 226 loot`).
10. `25-prod-watch-phone` production build via `vite preview` (worker + wasm in the SW precache,
    service worker registered); `26-fake-watch-phone` `?engine=fake`.

Console: zero errors or warnings on every step (page errors, worker errors and console
`error`/`warning` were collected; the only entry ever seen came from the test harness's own
`sessionStorage` init script on `about:blank`).

`pnpm -s build`, `tsc`, `node tools/copy-lint.mjs`: clean.

## What looked wrong as a player (not fixed here — game/renderer tracks)

- **Watch, phone**: the map is tiny. A tile is 16 CSS px, the visible floor sits in the middle
  third of a 400×800 screen with black around it; the hero sprite is two tiles tall, which reads
  well, but the rooms and corridors are hard to read at arm's length. Wide is fine.
- **Watch**: corrected 2026-09-16 (docs/RENDER_PERF.md): 59–60 fps at 400×800@3, 0.7 ms CPU per frame; the earlier 33 fps was the WSLg present ceiling, which scales with window width (1280 wide caps at ~29). The rest of this bullet is historical: JS idle — the
  pixel pipeline's cost is on the GPU/present side; worth a look before phones.
- **Death screen**: the DEFAULT hero sits at 1–3 HP for 4–5 actions "attacking nearest" with
  `1 unknown unused` / `5 unknown unused` in its pack, and the patches offered are `hp < 30% →
  descend`, `hp < 30% → bank`, `always → read fear`, `always → attack nearest` (+5%,
  survive 100%, baseline 75–100%). `hp < 30% → drink unknown` never appears although it is the
  obvious fix and in the vocabulary. Rows with no condition and a 0% delta read as noise.
- **Report after 8 h with the preset**: 69 runs, 69 deaths — the DEFAULT set never banks or
  returns, so every heir dies; the first thing a new player sees after coming back is a 100%
  death rate. Reel: the same "Near death, then the floor survived." five times (now collapsed to
  ×5 in the client, but the sifter picks one pattern).
- **Difficulty tail**: a 4-row set (`tame` / `hp<50 & foes≥1 → retreat` / `hp<90 → rest` /
  `attack nearest`) reached **D16 (the ending) in the second 8 h batch** of the very first
  lineage; `rest` is that strong. There is no ending screen yet: `lineage.ended` is set and the
  camp just carries on.
- **Gold**: $4,360 after 8 h, $9,040 after 16 h, and the only sinks are $30–60 supplies and $50
  hatches; the vault kept `mail` under `keep_pref: best_weapon`.
- **Bests list**: `rank 1 … rank 8`, `fighter L2 … L4` as separate lines (collapsed client-side
  now); `first kill: spectral_blade` for a summon is a "best" too.
- **Offline boot**: the page is blank except for the 3 px bar for up to ~10 s. A player may
  think it hung; a `runs so far` count would need chunked batches from the core.
- **Lost companions** are listed by name (`◯ Zelim`) while tamed/hatched are listed by kind.

## Follow-ups (same day, after commit 8499740)

1. **Phone map scale.** `ui/viewer.ts` passes `baseTexels: 150` when the canvas's short side is
   < 600 CSS px (else the renderer's default 200). At 400×800 @3: k = 8, a tile is 21 CSS px,
   ~19 tiles across, hero ≈ 1/12 of the height. Desktop unchanged. Re-shot: `02-watch-phone.png`.
2. **Chunked offline.** `App.runOfflineChunked` calls `runOffline` per slice and merges the
   reports (`mergeReports`: sums runs/deaths/xp/renown/marks/salvaged, unions learned/bests,
   concatenates tamed/hatched/lost/found, reel = top 5 by score, worst_death = deeper (ties: later),
   live = last, pending = last slice's (it is a state, not a delta), sampled = any). The camp
   renders first with the last state, `inert` and dimmed, and the bar reads `RUNS N · BEST Dk`
   after each slice (`27-offline-progress-phone.png`); the forecast is deferred until the batch is
   done so it does not queue ahead of the first slice.
   *Slice size.* Every slice pays the worst-death verdict + four patch forecasts inside the core's
   `report()` (~2.3 s in wasm on this loaded machine). With the requested ≤ 30-minute slices an 8 h
   absence took **46–50 s** (16 slices) against ~10 s for one call, so the slice is
   `clamp(elapsed / 6, 30 min, 2 h)`: 1 h → 2 slices, 3.3 s; 8 h → 80-min slices, **16.3 s**
   total, first count at 2.9 s; 24 h → 2 h slices, 51 s (12 slices). Recommendation for the core: a
   `run_offline` variant (or flag) that skips the verdict, then 30-minute slices cost nothing extra
   and the count can tick every ~0.6 s.
3. **Ending.** `ui/ending.ts`: `/art/title.png` full-bleed (`object-fit: cover`), `♟heir Dbest`,
   four tiles (runs · deaths · facts · renown), one button `again`. Shown whenever the app would
   show the camp and `lineage.ended` is true (so the report's `camp` button lands on it). The core
   has no ascend/carry-over method, so `again` = `newLineage(fresh seed)` with the three rule sets
   re-applied via `selectSet`/`setRules` (facts, classes, gold, marks, kennel reset). The wire
   `Lineage` has no run counter, so "runs" is counted client-side (`runsSeen`, in the save blob):
   watched runs + offline `report.runs` since the lineage began. Reached in the test by a 3-row
   `retreat / rest / attack` set in the second 8 h batch: `28-ending-phone.png`,
   `28-ending-wide.png`, `29-camp-after-again-phone.png` (heir 1, same 3 rows, 0 facts).
4. **Names.** Kennel/party cards and the companion rule sheet show `kind name L g` (`jackal
   Zelim`). Reports from a watched run list tamed and lost companions as `kind · name` (name from
   the snapshot entity), rendered kind + small name. Offline reports keep what the core sends:
   `tamed`/`hatched` are kinds, `lost` are names only — pairing them needs the core to emit
   `kind · name` (or both fields) in `ReturnReport`.

`pnpm -s build`, `tsc`, `node tools/copy-lint.mjs` clean; zero console errors on the ending,
again, and chunked-offline walks.
