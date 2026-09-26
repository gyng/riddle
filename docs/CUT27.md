# Cut 27 — The watch opens at doubt; the edit is a scene

*Contract, 2026-09-26. Cohort 22 (build 420f27c, three absences): 68.1 · 69.2, α 0.966, mean
68.65 (+2.95 over the control's 65.7). `docs/PATH_TO_95.md` P2 + P4; `docs/PLATEAU.md` "Cohort 22".*

## Design

### 1. Solved floors fold (core + client; pacing, surprise, tension, feel) — P2

AS/AT: D1–8 every run is a commute (theft, shrine, cage, two stairs, the Warlord routine).

- The sim still plays every floor (economy, determinism unchanged). A floor the active set clears
  ≥ 95 % (paired forecast, per floor) is **folded**: the watch plays it as one interstitial line
  (`D1–6 · 100% · +$84 · stolen heal`) and opens at the first floor below 95 %. A folded floor's
  beats that change state (a theft, a find, a death-adjacent hp dip, a first) still show as chips
  on the fold line; nothing that changes the lineage is hidden.
- `▶▶|` and modes are unchanged below the fold; a tap on the fold line plays the folded floors.
- A lit waystone start is never worse on gold/hr than D1 for a set that clears the band ≥ 95 % (the
  skipped floors' gold is paid at the start as `+$N passage` when the set would have cleared them).
- Gate: fights.mjs — watched time on floors forecast ≥ 95 % ≤ 5 s per floor; the fold line lists
  every state change of its floors (test); metrics — waystone gold/hr ≥ D1 gold/hr on cohort sets.

### 2. The edit is a scene (core + client; feel, mastery, clarity) — P4

AS: "turn a chip, read the number"; AT: many edits `≈`.

- On each edit whose paired move is ≥ 5 pts (or crosses a death), the core finds on a paired seed
  the first tick where the new set fires a different row than the sent set, and returns both
  branches' next few seconds (`Divergence {seed, tick, depth, sent_row, new_row, sent_end,
  new_end}`). The camp plays it in the forecast pane as a 3–5 s before/after on the renderer
  (`R5 now` → `lives · D9` vs `dies · D7`). The number stays; the scene carries *why*.
- An edit inside its ± shows the divergence too when one exists (`≈ ±6 · R3 fires 4× more`), so `≈`
  says what changed.
- Gate: divergence found for ≥ 90 % of ≥ 5-pt edits on cohort sets (metrics); scene ≤ 1.5 s after
  the refine on the D11 fixture (clarity.mjs, real wasm); ui.mjs.

### 3. Bank and return trade (core; decisions)

AO (21), AS (22): bank beats return on every number.

- A return keeps 60 % but walks from anywhere; a bank keeps 100 % but must reach the floor's up
  stairs. Measure on the cohort sets why bank dominates (the walk rarely fails? bank rows fire at
  the same moments?) and tune so each wins somewhere: e.g. a return keeps what's carried and heals
  the heir's start; a bank exposes the carry on the walk (thieves/ambush pressure scale with the
  carry). Never raw stats.
- Gate: a metrics row — on the cohort sets, the best set with a return row beats its bank-swapped
  twin on ≥ 1 of (gold/hr, death share, reach) by ≥ 10 pts, and vice versa.

### 4. Loops and the gem (core + client; failure, clarity)

- `R8 pack ↔ pick up` (AS, D5, 12 turns among 4 foes): the pack-break card and the pick-up chore
  alternate — fix the cause (a chore must not run with foes adjacent; a card row that drops what the
  chore picks up); AS's set is in eval/cards and the stall gate must catch it before the fix.
- The stall screen's gem goes through the same guard as the death gem (whole-run measure landed,
  never a harming patch); the gem's pre-selection is the best whole-run patch, and the shown order
  matches it (AS: `cut R1` 1/12 above a 12/12).
- Gate: qa.rs (the gem on every screen never harms; the gem is the first shown); stalls 0 on AS's set.

### 5. Seams

- A drive-off whose counter row is already in the set says `order` (`R7 under R2`), not `counter
  unwritten` (AT).
- A death after the player removed the row that answered its cause (the bloat row) is not `dice`
  (AT) — the removed row is a candidate patch (`restore R4`).
- `R1 fired 0 of 8 runs` vs a trace crediting it (AS): one count.
- A pet `gone wild`: its line says why (`left at 30% hp` or its cause) (AT).
- Gate: tests, qa.rs.

## Gates

| Gate | Bar |
|---|---|
| Folded floors ≤ 5 s each on the watch; fold line carries every state change; waystone gold/hr ≥ D1 | fights.mjs, tests, metrics |
| Divergence found ≥ 90 % of ≥ 5-pt edits; scene ≤ 1.5 s after refine on the D11 fixture | metrics, clarity.mjs |
| Bank and return each win somewhere by ≥ 10 pts on the cohort sets | metrics row |
| AS's loop fixed (stalls 0 on its set); the gem never harms on any screen; gem == first shown | qa.rs, gates |
| Seams | tests, qa.rs |
| Bots, dice, stalls, dances, no-hp stretches, lever, lanes, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 23 (three absences): mean ≥ 72 (control 65.7); pacing 0.8 from ≥ 1; feel 0.8 from ≥ 1; α ≥ 0.80 | two blind cards |
