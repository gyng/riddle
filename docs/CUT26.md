# Cut 26 — The descent forks; the route is a row

*Contract, 2026-09-26. Cohort 21 (build 307dbed): 73.5 · 70.8, α 0.937, mean 72.15 (−2.2).
Feel, expression, pacing, autonomy at 0.6 from both for the eighth cohort. `docs/PATH_TO_95.md`
(the arithmetic, the root causes, P1).*

Every plateau card names the same limit: "one dungeon, one direction (down), goals are depth or
gold" (AG), "the facts pick the row, I only order them" (AO), "the same Warlord on D8 and Bloat
Mother on D13 showed up every run" (AL). One linear descent priced by an exact forecast has one
optimum; every set converges on it. This cut gives each lineage a second defensible optimum at
every band, and makes choosing between them part of the policy the player writes.

## Design

### 1. The forked descent (core; surprise, autonomy, mastery)

- The Warrens (D1–4) stay shared. The base order of the six bands after it is Burrows D5–8,
  Fens D9–13, Crypt D14–18, Foundry D19–23, Deep D24–28, Sanctum D29–33; the bottom stays D34.
- **A fork** at the stairs into a band offers two: the *near* stair (the band's own biome)
  and the *far* stair (the next biome, early). Taking the far stair defers the near biome to the
  next band, which then has no fork (the deferred biome is taken). A route is the base order with
  non-overlapping adjacent swaps: 13 routes, every biome within one band of its own.
- A biome keeps its kinds, traits, situation and boss wherever it sits; stats scale by depth
  (`tier_depth`); the boss sits on the band's last floor and seals it as today (the Warlord at D13
  on a Fens-first route). No biome is ever tuned more than one band from its own (the anti-pillar
  "no HP inflation" — depth breaks programs by traits, not numbers).
- The descent is `f(lineage seed, route)`; floors regenerate per run as now. Grudges, bones and
  counters live on the biome and floor where they happened.
- Facts are per biome and carry across lanes (the Crypt learned at D9 is the Crypt at D14).
- The default route is the base order: DEFAULT, the presets and every existing save play exactly
  as on 307dbed until a route is written.
- Gate: tests (13 routes enumerate; every route reaches D34's stairs; replay hash per route); a
  307dbed save's next 10 sends are hash-identical.

### 2. The route is policy (core + client; expression, autonomy, attribution)

- The rule set carries a **route**: one choice per fork, shown above the rows as a chip line
  (`⑂ D5 fens · D14 crypt`), exported with the rows, restored with sets 1–3. It is written, never
  steered: the hero takes the set's stair; with no choice written, the near stair.
- A fork appears in the editor only after the hero has seen it (a fact: `fork D5`, learned the
  first time a hero stands on D4's two stairs). No tutorial text; the callout on the watch is
  `TWO STAIRS` and the chronicle line names the stair taken.
- **`in: <biome>`** is a new cond, open once the biome has been entered, so a set can carry rows
  for both lanes — at the cost of rows under the cap of 8.
- The forecast prices the set's route. The fork chip's sheet is an option tablet (first-pass
  sims, like forge/cage/start): both stairs for the current set (`fens D8 61% · burrows D8 34%`);
  `vs sent` reads a route change like any edit.
- Waystones light per route prefix; the start sheet lists lit (lane, depth) pairs.
- Offline: the night plays the active set's route; the report names each band's lane
  (`D5–8 · the Fens`), the reel credits `R3 · in: fens` rows as it credits any row.
- Copy: chips are biome names; the callout ≤ 3 words; no new sentences (copy-lint).
- Gate: tests; ui.mjs (the fork chip appears only after `fork D5`; a route edit reprices; export
  round-trips the route); copy-lint.

### 3. Lanes that want different sets (core tuning; expression, decisions)

- A fork is a decision only if the two stairs reward different programs. Tune (traits, kinds,
  situations, boss counters — never raw stats) until, at the D5 and D9 forks, **no set dominates
  both lanes**: the best set for each lane (the plateau search, seeded from EDITED and from every
  cohort set) loses ≥ 15 pts of reach-the-band's-end on the other lane, paired seeds.
- Both lanes are viable: EDITED's best set per lane reaches the band's end ≥ 50 %; neither
  lane's gold/hr for its own best set exceeds the other's by more than 1.5×.
- A biome's first boss kill pays marks wherever it sits (first kills already pay); the lane not
  taken is a visible frontier (`crypt · D9 · ?` on the shaft), so the second lane is the next goal.
- Gate: a metrics row per fork (per-lane best sets, cross-lane loss, gold/hr ratio);
  rule-set diversity (distinct multisets ÷ clears) ≥ 0.5 over the agent sets' D13 clears.

### 4. The first fork arrives in the first hour (core; pacing, surprise)

- A fresh lineage with the preset plus one offered patch sees the D5 fork (D4's stairs) on ≥ 50 %
  of sends within its first six sends (raters reach D5–D8 in their first 20 minutes today; keep it so).
- The D9 fork is reached by the cohort sets' lineages before the absence ends.
- Gate: metrics (first-hour agent: sends to first fork seen, forks seen by the end of the night).

### 5. Speed and the gate table (core; clarity, pacing)

- The fork tablet adds one option to the first pass; Cut 25's bars hold on the D11 fixture: first
  paint ≤ 1.2 s, refine ≤ 3 s, forge ≤ 3 s, real wasm.
- The gate table samples routes as it samples verdicts: each seed plays the base route and one
  swapped route (rotating which fork per seed), so every route is covered across 30 seeds without
  doubling the full table's time (≤ 4.5 min).
- Gate: clarity.mjs fixture; `node tools/gates.mjs --full` wall time.

### 6. Seams from cohort 21 (client + core)

- `GAP` beside `unpatched 10/12`: a death most replays survive unpatched reads `dice`-leaning —
  make the stamp and the counts agree (AO).
- A drive-off at 25/36 hp `driven $0 · $297 lost` opens its verdict (AP).
- A locked cond cannot enter a row unmarked (AP: `R3 descend · locked cond` found only in a trace).
- `survives 12/12 · reach D5 −76` — print reach moves as from→to, not a signed delta (AP).
- The `repeat` bubble never covers a button; the forge's second tap stays in place (AO).
- Gate: tests, ui.mjs, qa.rs (a stamp never contradicts its own replay counts).

## Gates

| Gate | Bar |
|---|---|
| 13 routes; default = base order; 307dbed saves hash-identical for 10 sends; replay hash per route | tests |
| Fork seen before shown; route exported, forecast, repriced; `in: <biome>` fact-gated | tests, ui.mjs |
| No set dominates both lanes at D5 and D9 (cross-lane loss ≥ 15 pts, paired) | metrics row |
| Both lanes viable (≥ 50 % band end); gold/hr ratio ≤ 1.5× | metrics row |
| Rule-set diversity across clears ≥ 0.5 | metrics row |
| First fork within six sends on ≥ 50 % of fresh lineages; D9 fork seen by the end of the night | metrics |
| Bots on every sampled route: DEFAULT dies ≤ D6 on ≥ 80 %; EDITED − DEFAULT ≥ 15; RANDOM, PASSIVE lose; LEARNED ≤ +2 floors; dice ≤ 5 %; cohort-set stalls 0 | `node tools/gates.mjs --full` |
| D11 fixture: paint ≤ 1.2 s, refine ≤ 3 s, forge ≤ 3 s; full gate table ≤ 4.5 min | clarity.mjs, timing |
| Seams; copy budgets | tests, ui.mjs, qa.rs, copy-lint |
| Cohort 22 (horizon as cohort 21): mean ≥ 77; expression 0.8 from ≥ 1; autonomy 0.8 from ≥ 1; surprise 0.8 from both; no axis under 0.6; criterion fun ≥ 6 from ≥ 1; α ≥ 0.80 | two blind cards |

## Not in this cut

- Solved floors folding in the watch and the waystone start economy (PATH P2), the divergence
  replay on an edit (P4) — Cut 27. Oaths (P3) — Cut 28.
- The eval track (PATH E: RATING.md's audio line, three simulated absences, a human listening
  pass) runs as a control cohort on 307dbed, in parallel, and never mixes into cohort 22's α.

## Risks, recorded before the build

- **A lane that is simply better** (one optimum again, one stair deep): the cross-lane gate
  catches it; tune traits, not stats. If no tuning holds at D9 in the budget, ship the D5 fork
  only and record it.
- **Bots on an early Fens** (DEFAULT meets gas and water at D5): DEFAULT must still die by D6 —
  if the Fens-first route lets DEFAULT die *earlier*, that holds the gate; if a lane lets
  PASSIVE survive deeper, the lane is wrong.
- **Forecast noise across lanes**: the tablet uses paired seeds per lane; a stair whose move is
  inside its ± reads `≈` as every option does.
- **Attribution**: a death on the far lane is the route's choice — the verdict may name the route
  (`route` as a row-like cause, `D5 fens`) when the near lane's replays survive ≥ 50 % and beat
  base by 15 pts, as `order` does for rows.
