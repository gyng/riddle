# Cut 21 — A run starts where the lineage is

*Contract, 2026-09-25. Cohort 16 (build 44193ac): 73.0 · 73.5, α 0.938, mean 73.25 — flat. Both
raters lead with the first floors repeating every run and an economy that undercuts them.
`docs/PLATEAU.md` "Cohort 16".*

## Design

### 1. Waystones (core + client; pacing, tension, surprise, progression)

AE: "D1–D7 play out almost the same each time … the first minute of every watch"; AF: "D1–5
repeat the same beats every run". A lineage that banks at D20 still starts at D1.

- **A waystone at each biome's first floor** (D5 Burrows, D9 Fens, D14 Crypt, D19 Foundry, D24
  Deep, D29 Sanctum) is **lit** once the lineage has banked from any floor at or past it.
- **The send chooses where to start**: the camp's `start` tablet (like the cage tablet) offers
  D1 and each lit waystone, with the forecast's move for each (`D9 · bank 62% · ~$180`). The run
  starts on that floor with the heir's kit; floors skipped pay nothing and teach nothing.
- **A start below D1 costs**: the send pays a toll of `$10 × depth` (the ledger names it), so a
  deep start is a bet, and a D1 start stays the free, safe default.
- **Offline** uses the chosen start for every run of the night.
- The situation bands (den D3–5, lock D6–7, captive D9–11, hunger D12) still happen on their
  floors when the run passes them.
- Gate: bots unchanged (they start at D1: DEFAULT, EDITED, … — a `start` is a player choice);
  a test that a waystone lights on a bank at or past it, a start pays its toll and begins on the
  floor; ui.mjs — the tablet and its forecast moves; a measured watch time to the first fight
  from a D9 start vs D1 on a D12+ lineage.

### 2. Supplies are not a leak (core; progression, decisions)

AE: "sells heal potions he finds for $2 while I pay $40"; a strength potion no row drinks
re-bought 16 times.

- **A found supply of a kind the shelf sells goes onto the shelf** at the exit (up to the cap),
  not to salvage — the player's next send packs it for free (`found heal → shelf`).
- **The repeat re-packs only kinds a row can use** (a `drink heal` row → heal; the trait's
  unknowns aside); a kind no row references is not re-bought (`strength · no row`).
- **A stall does not charge the repeat** on top of the loss (AF).
- Gate: tests; the night's spent vs salvage on the cohort-16 sets (reported before/after).

### 3. Words and verdicts (core + client; clarity, failure)

- A death caused by the player's own `drink unknown`/`read unknown` row is `row` (AE saw GAP),
  whatever the row's origin (typed, patched, or preset kept).
- The depth picker offers every depth up to the lineage's best + 2 (AE banked at D20; the picker
  stopped at 12).
- `picked clean`, `restock capped`, `repeat short`: each gets a one-word label change that says
  what it is (`thinned`, `restock ≤ income`) or a tap that opens the ledger line — within the
  copy budget, no sentences.
- The trace's chain explains a row that never fired over the last N ticks (AF: why `hp<30% →
  bank` never fired at 9/42 — the because link on each tick's row reason, not only the last).
- Gate: tests, ui.mjs, copy-lint.

## Gates

| Gate | Bar |
|---|---|
| Waystones light on banks; a start pays its toll and begins on its floor; the tablet's moves | tests, ui.mjs |
| Time to the first fight from a lit waystone vs D1 | measured, reported |
| Found supplies to the shelf; repeat only used kinds; no repeat on a stall | tests, measured |
| `row` for own unknown-drink deaths; the depth picker to best + 2; the words | tests, ui.mjs |
| Bots, dice, stalls on every cohort set | `node tools/gates.mjs --full` |
| Cohort 17: mean ≥ 78; pacing or surprise ≥ 0.8 from both; α ≥ 0.80 | two blind cards |
