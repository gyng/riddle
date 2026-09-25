# Cut 24 — Every minute moves; every run is its own

*Contract, 2026-09-26. Cohort 19 (build 4b15a61): 71.4 · 71.9, α 0.905, mean 71.65 (+2.7).
`docs/PLATEAU.md` "Cohort 19".*

## Design

### 1. A fight that can't progress ends (core + client; pacing, feel, failure)

AL: the Warlord > 4 min at 1× on `attack nearest`, boss bar full; AK: ~100 s of retreat ↔ pack
break against archers.

- **Core**: a fight where neither side's hp has moved for N actions (the hero's blows all shrugged
  — a boss's immunity, a counter unwritten; or a two-row dance) ends: a boss that can't be hurt
  wins its fight (the hero is driven off the floor: a `return`-tier exit with the verdict
  `no counter` naming the boss's trait and the fact to learn), a dance hands the floor to the next
  row per the guard. Tune N so a fight that is merely long (both bars moving) never trips.
- **Client**: the watch never shows > 15 s of real time without an hp change, a kill, a pickup or
  a descent: dead stretches in any mode play at the travel rate (the `fights` mode included).
- Gate: a metrics row — the longest no-hp-change stretch per send on every cohort set (p99 ≤ 60
  actions), and the qaL/raterAL-before-boss-row set reproduces AL's loop and now ends (add AL's
  pre-boss set from scratchpad/raterAL/notes.txt to eval/cards); fights.mjs — no 15 s dead watch.

### 2. Runs differ (core; surprise, story, pacing)

AK: the same jackal on D3, the shrine line, identical summaries.

- **Named foes rotate**: a named foe appears on a floor for a lineage at most every third run
  unless it holds a grudge (killed an heir); a grudge's name returns until avenged, then retires
  for good (`Ulak is avenged` then Ulak again: fix — AK).
- **Floor events vary**: the shrine and the other one-line floor events draw from a pool of ≥ 3
  variants each, no repeat within 3 runs; one new event kind per biome band.
- **End-of-run summary** names what was new this run (a first, a record, a named kill, a find
  kind never seen) before the counts; a run with nothing new says the one thing that differed.
- Gate: measured — distinct reel lines per 10 runs on the cohort sets (report before/after;
  target +50 %); tests for the grudge retirement.

### 3. The forge keeps its price (core; progression, fairness)

AL: the same step dearer as the lineage deepens; a thief took the bought leather.

- A step's price is fixed per step (set from the lineage's best when that ladder was first shown,
  or a flat table) — never rises with a deeper best.
- Kit items are never stolen, never salvaged, never dropped (they carry the kit ids).
- The forge's move reads in the same form as edits (`D9 +7 · death −7`), no bare `death +7`.
- Gate: tests; the forge metrics rows (affordable after an absence; next ≤ 3 nights) hold.

### 4. The forecast leaves room to think (core + client; mastery, expression, pacing)

AK: "the live forecast solves most edits". AL: own rows `≈`.

- The first pass paints ≤ 1 s and the refine ≤ 3 s in real wasm; edits in quick succession do not
  queue refines (only the latest).
- A move inside its own ± reads `≈ ±N` with the N, so a small real move is visibly unresolved, not
  "no change".
- Gate: clarity.mjs timing in real wasm (reported); ui.mjs.

### 5. Seams (client + core)

- The keep sheet keeps what was tapped (both raters): the tapped chip is kept, never salvaged.
- The reel credits the row that fired at the low point (`R4 read teleport saved him`).
- The forecast's boss floor is the floor the boss is met on.
- Gate: tests; qa.rs (a reel's saved-by row fired on the low tick).

## Gates

| Gate | Bar |
|---|---|
| Longest no-hp stretch p99 ≤ 60 actions on every cohort set; AL's loop ends; no 15 s dead watch | metrics, fights.mjs |
| Distinct reel lines per 10 runs +50 %; grudge retires when avenged | measured, tests |
| Forge price fixed per step; kit never stolen; forge rows hold | tests, metrics |
| First paint ≤ 1 s, refine ≤ 3 s real wasm; `≈ ±N` | clarity.mjs, ui.mjs |
| Keep sheet keeps the tapped; reel's saved-by row; boss floor | tests, qa.rs |
| Bots, dice, stalls, dances, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 20: mean ≥ 76; pacing and surprise 0.8 from one rater; α ≥ 0.80 | two blind cards |
