# Cut 25 — Rules are the lever; the order is the lesson

*Contract, 2026-09-26. Cohort 20 (build 075d8e2): 74.1 · 74.6, α 0.969, mean 74.35 (+2.7), both
"fun". `docs/PLATEAU.md` "Cohort 20".*

## Design

### 1. Kit helps; rules decide (core; expression, autonomy, mastery)

AN: three forge buys moved bank 40 → 95 %, more than any row.

- **A gate on the lever**: on every cohort set, the whole forge (every step) moves the set's bank
  share by less than the set's best single-row edit (the top offered patch or a one-row change
  from the plateau search) — measured with paired seeds. Tune the kit's numbers (smaller steps,
  armour that blunts not negates, a weapon step that adds accuracy rather than damage) until it
  holds; the forge rows (affordable after an absence, next ≤ 3 nights) and KITTED gates hold.
- The forge's own forecast move keeps showing honestly (`D9 +7`); if a step moves less than its
  ±, it says `≈ ±N`, as now.
- Gate: a metrics row — kit move < best row move on every cohort set; bot gates.

### 2. The order is a verdict (core + client; failure, attribution, mastery)

AM: return row below `attack nearest`, never fired → DICE; AN: own retreat row at 2 hp → DICE.

- **`order`**: a death where a row of the set would have fired (its conds held on the death's
  last N ticks) but a row above it won every tick, and moving it up survives ≥ 50 % of the
  replays and beats base by 15 pts, is `order` (one word, like `row`), naming both rows (`R5 under
  R2`). The patch candidates always include "move Rn above Rm" for every shadowed-while-matched
  row.
- A row that fired on the death tick and whose cut survives is `row` (AN's retreat row) — recheck
  why that death was DICE (the replay's cut candidates must include every own row that fired in
  the last 5 actions).
- Gate: tests; the dice share stays ≤ 5 %; qa.rs — a DICE death has no own row that fired in its
  last 5 actions whose cut survives more.

### 3. No dead watch, no chore loops (core + client; pacing, feel)

- **Chores that repeat**: `pick up ×441` while alert rises 8/8 — find the loop (an item that can't
  be picked up? a pickup that drops and re-picks? a pile respawning?) and fix it; a chore that
  raises alert must not repeat without end. qa.rs: a floor's chore count ≤ 50.
- **Drains count as dead time**: hp or max-hp falling with no foe in view (hunger, poison, a curse)
  plays at the travel rate and shows the cause once (`starving`), not 55 s of numbers.
- **The black frame**: find and fix the empty board frame (camera on a floor not loaded? between
  floors?).
- **A plain 1×**: the mode picker offers `1×` beside fights/fast.
- Gate: fights.mjs — a drain stretch plays ≤ 5 s; the chore invariant; a watch never paints an empty
  board for > 1 frame (sampled).

### 4. Numbers land fast on a deep lineage (client + core; pacing)

AM: 5–9 s of `…` after an edit; ~8 s for forge estimates on a D11 lineage after an absence.

- Measure first on the rater's own state (`scratchpad/raterAM`, a D11 lineage after `absent=8h`,
  headed): first paint, refine, forge. Find what is slow (sims per panel scale with depth? the
  forge/unlock/cage measures hogging the lane? the save sync?) and bring first paint ≤ 1.2 s,
  refine ≤ 3 s, forge ≤ 3 s there.
- Gate: clarity.mjs on a deep saved lineage (a fixture), real wasm, reported.

### 5. Less repetition (core; surprise, story)

- A thief doesn't take the equipped weapon or armour (what he wears is his, not loot); thieves
  take found, then brought-and-unworn, then coins.
- The overnight reel never repeats a line shape more than twice (collapse `×5` into one line with
  a count, pick the most distinct others).
- Gate: tests; measured reel distinctness on the absence.

### 6. Seams (client + core)

- `+1 row ⊘ slay 3 bosses` counts what it says (distinct bosses? say `3 kinds`) — match copy and
  count; the per-heir `RUNS 1 · DEATHS 0` after a death; forge `≈` on every item late (the
  measure depth); the unlock grid never reflows under a tap (fixed cells); a sheet opened from a
  sheet replaces it (one sheet at a time, with a back); the row sheet at the cap says `max`; an
  offline trace shows every blow (14 → 0 is several rows).
- Gate: tests, ui.mjs.

## Gates

| Gate | Bar |
|---|---|
| Whole-forge bank move < best single-row move on every cohort set | metrics row |
| `order` verdict; DICE deaths have no own recent row whose cut survives more; dice ≤ 5 % | tests, qa.rs, gates |
| Chores ≤ 50 per floor; drains ≤ 5 s on the watch; no empty frames; `1×` | qa.rs, fights.mjs |
| Deep-lineage paint ≤ 1.2 s, refine ≤ 3 s, forge ≤ 3 s real wasm | clarity.mjs fixture |
| Worn gear never stolen; reel shapes ≤ 2 repeats | tests, measured |
| Seams | tests, ui.mjs |
| Bots, dice, stalls, dances, no-hp stretches, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 21: mean ≥ 78; expression or pacing 0.8 from both; α ≥ 0.80 | two blind cards |
