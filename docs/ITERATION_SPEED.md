# Iteration speed — where a cut's time goes, and what to cut next

*Analysis, 2026-09-21, on 4145acc (after 3977696). Measured on this machine (5950X 16C/32T,
RTX 3080 via WSLg). Nothing here weakens a gate; raters stay blind; feel keeps a real browser.*

## 0. Measured today

| Step | Wall | Where it goes |
|---|---|---|
| `cargo test` warm, no edit | 16 s | 0.04 s cargo · 11.5 s core test binary (CPU 171 s / 32) · wasm crate tests |
| core test binary | 11.5 s | **critical path = one test**: `reel_pairs_never_repeat_across_three_absences` 7.2–8.4 s (10 seeds × 6 × 2 h, sequential); then `dice_death_names…` 7.0, `ledger_line_reconciles…` 5.5, `counter_at_the_top…` 5.2, `a_set_that_always_returns…` 4.8 — all seed loops |
| rebuild after a core edit | 13–14 s | one rustc of the 26 k-line crate (lib + lib(test) in parallel, 12–13 s each); a leaf-body edit 2.6 s; `cargo check` 1.5 s |
| fast profile tweaks | none | cgu 256 / `debug = 0` / opt 1: 13.9 / 14.9 / 11.8 s build, opt 1 runs tests 2× slower |
| wasm fast (core edit) | 21 s cargo + 1 s bindgen | vs 15 s after a mtime-only touch |
| clippy warm | 0.1 s (cached) | |
| `tools/gates.mjs` quick | 80–103 s | metrics 49–55 s alone · dayplayer 52 s alone (2 sequential 14-day chains) · **80 s overlapped** (sequential would be 101 s; the 32-thread table starves the chains) |
| metrics `--quick` | 49 s, 1110 CPU-s | CPU-bound (22.7 cores busy on 16 physical). Phases (thread-s, contended): offline 895 (62 %) · end-of-job `forecast` 294 (**20 %**) · verdicts 155 · `death()` 106. FULL bots: 27–46 s per job, 32 jobs = 70 % of the CPU |
| one FULL job alone | 12.5 s | 3 batches 2.2 / 6.2 / 2.0 s: live ticks 5.3 µs · **history clone 52 µs every 10 ticks = 1.07 s of the 2.17 s batch** · stall-verdict sims 4.8 s in batch 2 (cached in batch 3) · verdicts up to 2.9 s each |
| one DEFAULT job alone | 5.7 s | offline 0.66 s (ticks 2.1 µs, history 12 µs/10 ticks = 33 %) · 3 verdicts 0.4 s · 2 `death()` 1.5 s · forecast |
| `run_offline` 8 h native | 0.6–0.8 s | 121 k ticks ≈ 5 µs/tick incl. reports; `seen_pct` 25–34 k calls × 1 µs = 5 % |
| wasm fat-LTO release | 62 s cargo + 42 s wasm-opt | `pnpm build` (tsc + vite 8) **3.3 s**; AGENTS' "~6 min" is not explained by the parts — measured under load? |
| wasm fast vs fat+opt, runtime | **±5 %** | node on the pkg: offline 8 h 0.52 vs 0.54 s · `death()` 0.91 vs 0.94 · forecast 1.31 vs 1.29 · 2.53 vs 1.97 MB |
| `pnpm test` | 27.6 s | bounded by `audio.mjs` 27 s (watches a fake run to its exit at 1×); fights 20.6, qa9 18.2, chain 17.4 |
| playtest walk headed | 33 s | offline 8 h 8.4 s (wasm) · the patch → camp 6.2 s (`death()` deltas in wasm) · 173 `▶▶\|` presses 9 s · fixed waits ~5 s |
| driver op latency | text 15 ms · click 720 ms (600 ms settle) · shot 450 ms | **the hour of a QA session is LLM turns**: qaC 55 min, 68 shots, ~150 calls ≈ 22 s per call, of which < 1 s is the driver |

## 0b. Round 2 — 2026-09-24, on 2b86cac

*Measured A/B: HEAD's binaries built in a scratch worktree and run back to back with the new ones
under the same load. The box was shared with a client agent (load 20–40), so absolute walls read
high; the ratios are the result.*

| Step | Before (HEAD) | After | How (all bit-identical: `Replay hash` a6cf8d205957f07e both sides, the full table's text identical but for its timing lines) |
|---|---|---|---|
| `node tools/gates.mjs --full --fresh` | 4 m 05 s (3 m 35 s on a quieter box) | **2 m 25 s** | the table is the critical path (qa and the dayplayer finish well before it) |
| metrics full (table) | 244 s | **145 s** | bot jobs 3841 → 2381 thread-s (FULL jobs 82 % of it) |
| metrics `--quick` | 52–54 s, 1210 CPU-s | **33–34 s**, 828 CPU-s | |
| qa leg (`examples/qa.rs`) | 116–134 s beside the table · 49 s alone (4 threads, 185 CPU-s) | **76–89 s** · **36 s** alone (138 CPU-s) | engine only; see "qa" below |
| core test binary | 15.9–19.5 s wall, 395–419 CPU-s | **9.2–11.2 s**, 245–273 CPU-s | engine + seed loops on threads |
| `run_offline` 8 h, DEFAULT seed 1 (109 k ticks, reports incl.) | 1.15–1.27 s (≈ 11 µs/tick) | **0.75–0.85 s** (≈ 7.3 µs/tick) | |
| verdict (mean of 15, seed 1) | 0.197 s | **0.118 s** | |
| `death()` with deltas (table's mean of 116) | 1.54–1.63 s | **1.07–1.24 s** | |
| one FULL job (seed 1, single-threaded) | 11.1 s | **7.9 s** | |
| sim panel (4 × 50 sims to D8, the paired-forecast test's work) | 2.9 s | 2.6 s | |

What moved it, in order of yield:

1. **The chores' floods stop at their answer** (`tiles.rs` `bfs_nearest`, `bfs_parent_to`). `nearest_tile`
   (explore, items, den gold) wanted the least-distance tile, then the least index, and flooded the whole floor to
   find it; the flood now stops once the first layer holding one is complete (every tile of that distance is
   discovered then, so the choice and the parents along its path are the full flood's). Walking to a known goal
   (`path_step`, the stairs, a hunt, the bones) stops when the goal is discovered (a tile's parent is final at
   discovery). Six `hero_bfs(run) → step_towards(goal)` sites use it (`ai::hero_path_to`). FULL job −15 %.
2. **The end-of-job `known_to` forecast at `MIN_SIMS`** (§3.2's proposal): `known_to` is the panel's range
   (`best_depth + 1`), not a sim result; 15–17 % of a job's CPU. The gate reads the same field.
3. **One job pool** in `metrics.rs`: the counter trials and the cohort-set stall jobs join the bot jobs' queue
   (longest first), so they fill the cores while the last FULL jobs finish instead of running as phases after
   them (−9 s of phases). `gates.mjs` gives the table every core but one (the dayplayer is ~1 min alone, no longer
   the critical path); `METRICS_PHASES=1` prints phase and per-job walls.
4. **Precomputed line-of-sight walks** (`tiles::los_table`): `update_vision`'s 225 Bresenham walks per hero move
   read their offsets from a per-radius table (the walk depends only on the ends' difference) and index the tiles
   directly (both ends in bounds ⇒ every point is). Sim panels −5–9 %.
5. Smaller, each measured or plainly non-negative: the BFS generic over its `blocked` closure (the `&|_| false`
   floods — `hero_dist`, the generator's — no longer call through a `dyn` per tile; −2.5 % on sim panels); the
   visited test first in the flood's neighbour loop (−3 %); `max_rows` one `range` descent instead of six string
   lookups on every `ctx()`; `seen_pct` one pass; `situations_seen` skips 32-tile blocks with nothing in view;
   `approach` reads the cached hero field instead of copying 13 KB per monster per tick.
6. **Tests** (§3.4): the ten longest seed loops run a thread per seed (`tests::par_seeds`, results in seed order,
   a panicking seed re-raised with its own message; the two "first seed that has X" loops go 8 seeds at a time and
   keep the first in order). Each asserts what it asserted. The suite is CPU-bound now (≈ 250 CPU-s on 32
   threads), not critical-path-bound: the engine wins are what shortened it. Where the CPU goes:
   `paired_forecast_one_notch_edit_is_a_small_delta` 90 CPU-s (30 seeds × 2 × 50 sims to D8 — the contract's
   own sample), `forecast_pm_and_refine_share_the_panel_seeds` 37, `dice_death_names_an_alternative…` 30,
   `the_nights_ledger_reconciles…` 18; nothing else over 13.

The inner-loop check for this kind of work is new: `cargo run --profile fast -p riddle-core --example fingerprint`
(8 seeds × {DEFAULT, a finished FULL lineage} × 8 h, three `death()`s and the forecast, serialised and hashed; ~8 s
on the cores) — the same printout before and after a change, in a tenth of the table's time. The table's replay
hash stays the gate. The wasm `pkg/` was not rebuilt in this round (web/ belonged to another agent): the changes are
bit-identical, so the next `tools/wasm.sh` brings the client the faster forecast and `death()` with no other effect.

**qa (examples/qa.rs).** The brief's guess ("40 real sends per seed") is out of date: `SENDS` is 6 per set
(18 per seed) beside a 20-sim panel per set. Phase CPU per seed (30 seeds, before): the forecast-vs-sends block 32 %
(three 20-sim panels + 18 sends), the night 22 %, the two full forecasts 13 % + ~12 %, `death()`s 20 %. It was never
the critical path (116 s beside a 215–244 s table) and costs ~3 % of the gate's CPU; its sends are the invariant's
evidence (the forecast machinery must not grade itself), so it was left as it is and got the engine's −25 %.

**Not worth doing (measured or argued):** `#[inline]` on the tiny map helpers (−2.6 %: the `fast` profile's
crate-local ThinLTO already inlines them); a coordinate array beside the flood's queue instead of the division
(slower: the second allocation); merging `sheets`-style probes into the page itself (a click that mutates cannot
be undone — hence the throwaway copy). **Next, in order:** the history ring (`Run` clone every 10 live ticks is
8.5 % of a FULL job plus its drop — facts behind an `Arc` with a serde shim keeps the save's JSON, ~3 % of the
table); the stall-verdict sims (31 % of a FULL job, the report's content — only a faster tick helps); the flood's
inner loop (20 % self: offsets instead of the multiply, u32 queue).

## 1. The evaluation loop (the afternoon)

Today: QA round (2 players, parallel, ~55 min) → triage + fixes → reship → QA round 2 →
blind cohort (2 raters, ~60 min) → score. ≈ 3.5–4 h. The QA reports are the evidence that a
large share is mechanical: of the 26 fixed rows in `eval/qa/952e306.triage.md`, 14 are numbers
that do not reconcile, duplicated or misspelt strings, titleless sheets, or inert controls.

### 1.1 Mechanical QA pass — do now

Two layers, both before any LLM plays:

- **Wire invariants** (`examples/qa.rs`, native, 30 seeds in ~30 s, parallel): play the protocol
  through the engine (send → step to the exit → `death()` → apply a patch → buy → `set_rules`
  → `run_offline` → report) and assert what the QA players reconciled by hand: gold header ==
  ledger sum; `RUNS == deaths + banked + returned + stalled`; `ends` sum to 1 ± rounding and
  `death 0 ⇒ no killers`; a learned flavour never prints as `?`; a gate's requirement names a
  fact the ledger does not already hold; camp supplies == the next run's `brought`; no patch
  already in the set; the refund of a dropped supply equals its price; `worth` == the exit
  sheet's prices. Each cohort's triage adds rows (it is the same move as the Cut 11 `because`
  gate).
- **Screen lint** (`web/tests/screens.mjs`, headless, ~2–3 min): the playtest walk extended to
  the QA brief's full itinerary (four sends in `fights` / `fast` with `▶▶|`, every death and
  every because-link, four hand edits, every affordable buy and its sheet, drop / reorder rows,
  supplies, vault pref, every camp and report sheet, `?absent=8h`, the report line by line).
  Per dump: no `undefined` / `NaN` / `[object`; no segment repeated within a line (`returned
  $20 · returned $20`); every visible enabled button changes the text hash or is a documented
  toggle; every sheet has a title; numbers parsed from the header, the ledger and the report
  agree; `tools/copy-lint.mjs` over the *live* text (composed strings escape the source lint).
  `web/tests/qa9.mjs` (31 checks born from the last QA list) is this file's seed.

Gain: ~half of a QA list found in 3 min instead of 55; the first LLM QA round starts on a build
without them; a regression pass on the reship is free. Cost: 1–2 days for the first version,
then ~30 min per cohort to encode the new lapses. Risk: none to the numbers (QA is evidence,
not score). It cannot judge copy that "could not be explained" — that stays with the LLM.

### 1.2 Transcript ("screen-reader") QA — do now, alongside 1.1

The screen-lint walk already yields ~60 innerText dumps with timestamps and screenshots. A QA
agent reads the transcript in one turn (~30 k tokens) and files lapses, then spends 10–20
interactive turns reproducing the suspicious ones. ~10 min per player instead of 55; four
players on four seeds for the price of one today. What it loses: the player's own decisions
(the walk's edits are scripted — keep one interactive player per cohort for that) and
timing-bound lapses (a callout that persists, a repaint a second late, an inert `▶▶|`) — the
walk mitigates with periodic HUD dumps during watches and a per-dump timestamp. Cost: half a
day (the walk is 1.1's; the brief is a paragraph). Risk: none; QA is not scored.

### 1.3 Blind rater on a transcript — next cut, as a predictor only

Text carries clarity, attribution, decisions, expression; it does not carry feel, pacing,
audio, and the rater's agency is part of the fun. The blind card is the precommitted instrument
behind eight cohorts at 74–76; changing its medium breaks the series. So: a transcript rater
is a *leading indicator* (§1.5), never the cohort. Calibrate first: the eight rated builds are
commits — walk each in a worktree, rate the transcripts N times, check the mean tracks the
blind means (a 5-point drift like the Loop Hero re-anchor rule). Cost: a day plus tokens; the
walk and brief exist by then. Risk: a predictor that does not track is worse than none — the
calibration is the gate on using it.

### 1.4 Compressing the horizon — nothing to compress

The waits are already small: the absence is 8 s of wasm, a `death()` 1–6 s, a forecast 1–2 s.
"40 minutes" is not a clock for an agent; it is ~150 turns × 22 s. The 1× watches are ~10
turns of `shot` / `wait` each — a driver `watch` op (record HUD text + a frame every N s at
the chosen speed for M s, return a contact sheet) makes them one turn and keeps the rater's
speed choice. `?speed=` stays a dev tool: a rater watching at 4× is rating a different game.

### 1.5 A fun-eval harness — next cut

`tools/fun-eval.mjs --build <commit> --n 8 --mode transcript|api [--blind]`: N agent sessions
(`claude -p` with the brief, a seed and a port each), cards collected, `eval/score.sh` +
`fun-reliability.mjs`, prints mean · sd · α · per-axis distribution and the quoted lapses.
Transcript mode ≈ 0.1 M tokens per card, minutes; api mode (§2) ≈ 5 M tokens, ~20 min, in
parallel. Use: the distribution over 8 seeds tells a plateau from a seed; the blind cohort of
two on the GPU stays the decision. Cost: a day once §1.2 and §2 exist.

## 2. A tool server for the players (MCP or the driver's HTTP) — do now, with two profiles

*Done (driver ops, 2026-09-22): `state`, `act` (settles on `!busy`, not 600 ms) and `watch`
(lines that appeared per sample + evenly spaced shots) in `tools/driver.mjs`; the briefs name
them. 2026-09-24: `send_and_watch` (tap send, tap the rater's mode, sample the lines that
appeared every N ms, shots every M ms, optional `▶▶|` taps at the player's cadence; stops at a
sheet over the watch or when the watch's own controls are gone — the death / report screen —
and returns that screen's text, its buttons, the open sheet and an end shot; ~9–13 s for a
`fast` + `▶▶|` send, one turn where it took 10–15) and `sheets` (every sheet a screen opens,
tapped in a throwaway copy of the page — the same saved lineage in a second context, its save
stamped "seen now" so the reload runs no absence — the main page untouched; 4 copies side by
side; a control that changed the screen instead is listed with the lines it added and its copy
rebuilt; ~30–70 s for a camp's 22 controls; controls are found by the `.sheet-wrap` they open,
not by label, so the reskin does not break it). Both read only the page (the rater profile);
the briefs name them. The profiles as two servers: still open.*

The driver is already an HTTP/JSON tool; each op costs a Bash turn and a JSON blob in context.
The transport is not the cost, the op granularity is. Compound ops, exposed as MCP tools
(`.mcp.json`, ~150 lines over `driver.mjs`) or as new driver ops:

- `state` → screen, text, buttons, engine busy, in one call (today: 3).
- `act(label)` → click, settle on `!engineBusy` + a frame (not a fixed 600 ms), return the new
  text and a shot path (today: click, wait, text, shot = 4).
- `send_and_watch(mode, speed, until)` → interstitials, HUD dumps every N s, the exit / death
  text, every because-link's text, the trace (today: 10–15 turns per run).
- `sheets()` → every camp / report sheet's text in one call (today: ~20 turns).
- `edit(rules_text)` for the QA profile only; a rater edits by hand (the editor is rated).
- **QA profile** adds the wire: `game.lineage()`, `game.forecast()`, `death(id)` JSON beside the
  screen — screen-vs-wire reconciliation is the strongest lapse detector there is.
- **Rater profile** exposes only what the phone shows (text, pixels, buttons) — the wire leaks
  `dice`, hidden facts, the forecast's internals, and clarity is rated on the copy.

Turn estimate from the QA notes: runs 60 → 15, edits 20 → 8 (rater) / 4 (QA), buys and
sheets 20 → 3, report 15 → 3, setup 20 → 10: **~150 → 45–60 turns, ~55 → ~20 min** per
player, and fewer `getByText` misfires. Fidelity: unchanged for raters if screenshots stay
mandatory at the brief's five moments and `send_and_watch` honours the rater's speed; the
rater still decides every edit, patch and buy. Cost: a day. Risk: a rater profile that grows
a wire field by accident — keep the profiles two servers, not a flag.

## 3. Build and verify

### 3.1 Preview ship — do now

`tools/ship.sh --preview`: `tools/wasm.sh` (fast) + `pnpm build` (3 s) + preview restart
≈ 20–25 s instead of the fat-LTO + wasm-opt path (62 + 42 s here; "~6 min" in AGENTS). The
engine's results are the same source, integer-deterministic; runtime within ±5 % (measured);
the only loss is 0.55 MB on a local preview. Use it for every QA round; record the recipe in
the cohort's Build field; keep `--ship` for the blind cohort if the series' Build field should
stay literal (it costs 2 min once per cut). Add a wasm replay-hash check (node on the pkg,
`runOffline(1800)` + save, the same FNV as `metrics.rs`) so wasm and native are compared on
every ship, which today they are not.

### 3.2 The gate table's CPU — do now (bit-identical by construction)

*Done 2026-09-24 (§0b): the `known_to` forecast at `MIN_SIMS`; one pool for every job; the table on
every core but one (the dayplayer is no longer the critical path).*

- **The end-of-job forecast** (`r.known_to_ok`, 20 % of the quick table's CPU) checks
  `known_to == best_depth + 1`, which `forecast_with` assigns (forecast.rs:172). Keep the gate,
  run it on `forecast_with(g, rules, MIN_SIMS)`: same field, 1/10 the sims. Quick 49 → ~40 s,
  full ~150 → ~120 s.
- **Table + dayplayer overlap**: the dayplayer is the critical path (52 s alone, 80 s under the
  table's 32 threads; its chains need 2–3 uncontended cores). Give the table
  `available − seeds − 1` threads and start the dayplayer first: ≈ 58 s for the pair (estimate).
- **FULL jobs** (27–46 s contended, 32 of them) are a third history clones (§4.1) and 40 %
  stall-verdict sims that no gate row reads but that are the report's content — leave the sims,
  fix the clone; with §4.1 a FULL job ≈ 9 s.

### 3.3 Cache the table by binary hash — do now (an hour)

`gates.mjs`: sha1 of `target/fast/examples/{metrics,dayplayer}` + the presets → the printed
table under `target/gates/<hash>.txt`; a hit reprints and re-checks `gates: all PASS`. The
binary hash covers every input exactly (sources, deps, rustc). Client-only commits — most of
the last twenty — skip the 2.5-min full table entirely; `verify --full` on a client cut ≈ 45 s.
"Only changed subsystems" is the unsafe form of the same idea: any core edit can move any bot.

### 3.4 Split the seed loops in the slow tests — do now (an hour)

*Done 2026-09-24 (§0b): ten loops on `tests::par_seeds`. The suite turned out CPU-bound, so the
engine work did most of the shortening.*

The test binary is critical-path bound (7–8 s in one test; the CPU sum is 69 s / 32 cores = 2 s).
Run each seed loop over `std::thread::scope` (or split per seed): ~11.5 → ~4 s per
`cargo test`, so `verify --quick` after a core edit ≈ 28 → 20 s. Determinism is per seed.

### 3.5 Profile and layout — no / later

Profile tweaks measured nothing (§0). The 13 s floor after a core edit is one rustc over one
crate: `engine.rs` / `ai.rs` / `turn.rs` are 3.3 k / 3.7 k / 1.9 k lines and an edit in `tick`
re-optimises its whole CGU; a leaf edit is 2.6 s. Splitting the giants into modules would cut
the common case toward ~5 s — a refactor for a quiet week, not a cut. Moving `tests.rs` out
of the lib does not help (the lib(test) build is the same rustc). In the inner loop, `cargo
check` (1.5 s) answers type errors before the 13 s.

## 4. The core sim (bit-identical: nothing below is read by `tick`; the replay-hash gate and
the 3977696 fingerprint method prove it)

### 4.1 The history ring — do now

Every 10 ticks `Run` + `facts` are cloned: 12 µs shallow, **52 µs on deep floors** (33 % of a
DEFAULT batch's live time, 49 % of a FULL batch's). The clone by field, FULL: trace 12.8 µs
(16 `TraceTurn` with Strings), facts 10.8 (a `BTreeSet<String>` of every fact, cloned 12 k times
per 8 h while it changes < 100 times), kills + notes + verb_ring 8.2 (Strings that grow with the
run), monsters 2.9, dist + maps + sets 2.6, hero + items 1.7, floor 0.5. Fixes in order of
yield: facts behind an `Arc` bumped on change (serde `rc` keeps the save's shape); intern the
kill / note kinds (an id, the String at read time); keep the checkpoint's `trace` only as far
as `verdict` reads it. Expected 52 → ~15 µs: `run_offline` 8 h −20–25 %, a FULL job −25 %,
the table's CPU −12 %, the dayplayer chain −6–8 s. Note `history` is in the save (31 Runs in
every localStorage write); `#[serde(skip)]` + a save version is a separate, larger saving.

### 4.2 `seen_pct` — do now (ten lines)

*2026-09-24: one fused pass (no counters: `Map`'s fields are public and written in many places, a
counter would be a second truth). The chores' floods, not this, were the tick's cost (§0b).*

Two full passes over the tiles per call, 25–34 k calls per 8 h (5 % of live time). Keep
`seen_passable` and `passable` counters on the map, bumped where `seen` flips.

### 4.3 What is left — next

`turn::tick` is 2.1 µs shallow, 5.3 µs deep: vision (225 LOS rays per tick), monster pathing,
the BFS on a hero move. A flame graph needs `perf_event_paranoid ≤ 1` (one `sudo`; samply is
installed) — do that before guessing. Verdicts on deep floors reach 2.9 s (the D28+ candidates
× 20 replays of long runs) against a 0.4 s bar measured as a mean of 12 on shallow ones; it is
not a gate problem, it is the FULL jobs' tail.

## 5. Everything else

- `pnpm test` is one test: `audio.mjs` watches a fake run to its exit at 1× (27 s); the cue
  checks do not need real time — `speed=4` or `▶▶|` after the first cues → ~10 s, suite ~20 s.
- The driver's fixed settles (600 ms per click, 1.5 s per goto) are ~15 % of a walk; settle on
  `!engineBusy` + one frame. Small.
- Two raters headed at once: the QA players already run in parallel headless; two GPU windows
  at 400×800×3 should hold 60 fps on the 3080 — run `screen-time.mjs` once with two windows
  open before the next cohort to know, then run the blind pair concurrently (60 → 60 min, but
  the afternoon's serial tail halves).
- Vite full reloads from a `pkg/` rebuild kill walks; build into a staging dir and `mv` (one
  event), or run client tests against a preview build. Minor; the retry covers it.
- `verify --quick` runs tsc and copy-lint after the tests; run them alongside (−3 s).
- Native `run_offline` 8 h is 0.6 s; the client's absence takes 8.4 s (single-threaded wasm,
  quick slices + one `death()`). A rater sees that as a progress bar, not a lapse — fine.

## Ranked top 5

1. **Mechanical QA + transcript QA** (§1.1, §1.2): ~half a QA list in 3 min, a QA round 55 →
   ~10–20 min, a free regression pass on the reship. The afternoon → ~1.5–2 h.
2. **Compound ops with two profiles** (§2): ~150 → ~50 turns per player, QA and blind alike;
   screen-vs-wire reconciliation for QA; no change to what a rater sees.
3. **Preview ship** (§3.1): 2–6 min → ~20 s per reship, runtime within 5 %, plus a wasm/native
   replay-hash check the loop lacks today.
4. **Gate table: cheap `known_to`, history-ring snapshot, `seen_pct`, thread split** (§3.2,
   §4.1, §4.2): quick 80 → ~45 s, full 2.5 → ~1.5 min, `verify --full` ~4 → ~2.5 min.
5. **Binary-hash table cache + seed-loop split** (§3.3, §3.4): client-only `verify --full`
   ≈ 45 s; `cargo test` 16 → ~8 s.

**Order.** Ship the two afternoon-scale wins first because the cohort is the gate for the cut:
the screen-lint walk with its wire invariants (§1.1) and the compound driver ops in two
profiles (§2), then run the next QA round as transcript-first with one interactive player. In
the same day, the preview ship (§3.1) so the reship is seconds. Then the engine work, all
bit-identical and fingerprinted: the `known_to` forecast, the `Arc` facts and interned strings
in the history ring, the `seen_pct` counters, the table's thread split; land the binary-hash
cache and the seed-loop split with them. Leave the transcript *rater* and the fun-eval harness
for the next cut, and only after calibrating them against the eight cohorts on record.
