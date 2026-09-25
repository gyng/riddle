# Cut 22 — The gold loop pays

*Contract, 2026-09-25. Cohort 17 (build 6da3ed0): 70.3 · 71.9, α 0.968, mean 71.1 (−2.15). Three
sinks added by Cuts 19–21 (the repeat restock, the waystone toll, a D1 thief taking the heal just
bought) made the economy a treadmill. `docs/PLATEAU.md` "Cohort 17".*

## Design

### 1. A safe run nets gold (core; progression, tension, return)

AG: "+$49 returned · −$120 spent"; AH: "−$170 spent against +$47 returned … a treadmill".

- **The economy has a gate**: over 8 h on each cohort set (17 sets in `eval/cards/`), a set that
  banks or returns its runs **nets positive gold per send** after its supplies, tolls and thefts
  (the dayplayer-style measure in `examples/metrics.rs`: net gold per send ≥ +$20 on every cohort
  set whose runs mostly come home). Tune prices to meet it, don't weaken it:
  - **Supply prices scale with the lineage**: a potion costs `$10 + 2 × best depth` (D8 → $26,
    D20 → $50) and a scroll 1.5× that, so a D8 lineage's heal is not a D20 lineage's.
  - **The repeat restock re-buys only what was used** (a potion drunk, a scroll read) — an unused
    packed supply is already back on the shelf; a stolen one is re-bought only once a night.
  - **The toll is a fraction of the start's reward**: `$5 × depth` (was $10), paid once a night.
- Gate: the new metrics row; `DEFAULT yields 0 xp/gold` holds; EDITED banks ≥ 3 per 8 h holds.

### 2. Thieves take what was found, not what was bought (core; tension, fairness)

AH: "monkeys steal the heal potion on D1 almost every run … 8 seconds after I paid $40".

- A thief's pick order: found items first (the run's own loot), then gold-worth, and a packed
  supply only when the pack holds nothing else. A D1 monkey with an empty-handed hero steals a
  coin pile's worth instead (`−$6 stolen`).
- Gate: measured bought-supply thefts per run on the cohort sets (report before/after; target
  ≤ 0.1).

### 3. The forecast answers the question the player asked (core + client; decisions, mastery)

AH: "most edits moved the forecast less than its ±10–13 error, so I couldn't tell good from bad".

- **An edit shows its paired delta**: the camp keeps the previous set's panel and shows, per
  depth and on the gems, the edit's move (`D8 +6`) — paired seeds make that move far tighter than
  either absolute bar. When the move is inside its own paired ± it reads `≈` (no call).
- **Absolute numbers stay as they are**; the delta is a second, small line under the shaft
  (`vs last · D8 +6 · bank +4`), cleared on the next edit.
- Gate: a test that the paired delta's ± is ≤ half the absolute ± on the cohort sets; ui.mjs for
  the line.

### 4. Verdicts and labels (core + client; failure, clarity)

- An **attack row** that chased into a death (the killing action was that row's, and cutting or
  narrowing it survives ≥ 50 % and beats base by 15 pts) is `row` too (AH's R4 `attack ranged`).
- The start picker's options show the **death share** beside the bank change (`D9 · bank +3 ·
  death 61%`) (AG); deltas always carry a sign and a `Δ` look (`+3`), never read as a chance.
- Gate: tests; ui.mjs.

## Gates

| Gate | Bar |
|---|---|
| Net gold per send ≥ +$20 on every mostly-homecoming cohort set (8 h) | metrics row |
| Bought-supply thefts per run ≤ 0.1 on the cohort sets | measured |
| Paired edit delta ± ≤ half the absolute ± | test |
| Attack-row `row` verdicts; start picker's death share; signed deltas | tests, ui.mjs |
| Bots, dice, stalls, DEFAULT yields 0, EDITED banks ≥ 3 | `node tools/gates.mjs --full` |
| Cohort 18: mean ≥ 76; progression 0.8 from both; α ≥ 0.80 | two blind cards |
