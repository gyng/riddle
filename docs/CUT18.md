# Cut 18 — The watch keeps its peaks; the screen never contradicts itself

*Contract, 2026-09-24. Cohort 13 (build 32971ad): 67.3 · 68.1, α 0.863, mean 67.7 — flat. The
frame (Cut 17) did not move feel or aesthetic; both raters look at the watch for the hour and
lead with its timing, then with walls and contradictions. `docs/PLATEAU.md` "Cohort 13".*

## Design

### 1. The watch's timing (client; feel, pacing)

Y: "`GOBLIN WARLORD DOWN` went by in about a second"; "each cage held the watch about 30 s …
about 90 s of one 5-minute watch"; "`fast` mode was slower than `fights`". Z: "every cage sits on
choose one for about 30 s unless you tap".

- **Peaks hold in wall time.** A beat (boss down, the break, bank/return, a situation) holds
  ≥ 2.5 s of wall time whatever the mode's rate, with the frame on it; the ticker never
  replaces a beat's line inside that time.
- **The cage waits 6 s**, not 30, then the preference picks (the bar shows 6 s); ▶▶| picks at
  once. Watching it is optional, never a toll.
- **`fast` is never slower than `fights`**: across a run, `fast` spends ≤ the wall time of
  `fights` on every segment (dead stretches 16×, fights 4×, beats as above), measured by a test
  that plays the same fake run in both modes.
- Gate: fights.mjs — a boss-down beat is on screen ≥ 2.5 s in both modes; clarity.mjs — the
  cage closes by 6.5 s untouched; a `fast` run's wall time ≤ the `fights` run's on the same world.

### 2. The fight is legible (render + client; feel, aesthetic)

Y: "monsters stack on one tile and the warlord sprite hid my hero completely". Z: "sprites
overlap, and stacked callouts (`R4 HIT RANGED` over `ARCHER DRAWS`) hide the fight".

- **The hero is always drawn on top** and never covered: a sprite whose rect overlaps the
  hero's by > 30 % is drawn behind him and at 60 % alpha where it overlaps; a boss is drawn at
  its size but offset so the hero's head stays clear.
- **One line of callout at a time over the fight**: the rule callout and a telegraph never
  stack; the telegraph (the threat) wins the line, the rule shows on the ticker.
- Gate: fights.mjs — with a boss adjacent, the hero's rect is ≥ 70 % unoccluded
  (`debugRects`); two callouts on one tick draw one line over the fight.

### 3. A wall says it is a wall (core + client; clarity)

Z: "`D9 0%` for every rule set, with no reason given, until I met the Goblin Warlord"; Y the same.

- `ForecastDepth.wall`: when reach falls to ≤ 5 % at a depth whose floor above holds a boss
  that seals the stairs, the forecast names him (`goblin_warlord`); the shaft's notch reads
  `D9 · warlord` and the panel's row `D9 0% · warlord wall`. When the lineage knows the counter,
  the `try:` line already follows it (Cut 10 §2).
- Gate: a test — a D8 lineage without the counter row gets `wall` on D9; with it, none when
  reach clears 5 %.

### 4. No contradiction on a screen (core + client; clarity, failure)

- **`drink ✗ no item` with `heal potion ×4` in FOUND** (Z): reproduce; if the heals were found
  unidentified the reason must say `unknown` (`drink ✗ unknown`), not `no item`.
- **A stall's cause names the loop** (Y: "the jackal, no path" while the trace shows the gas
  retreat ping-ponging): when the stall's last window alternates between two rows, the cause is
  those rows (`R5 retreat ↔ descend`), and the verdict's first patch addresses that row.
- **The report after a stall** counts it (`RUNS 1 · STALLED 1`, never `DEATHS 0` alone) (Y).
- **`APPLY` does not send** (Z): it applies the top patch and lands on the camp with the tablet
  lit; the player sends.
- Gate: tests for each; ui.mjs for APPLY.

### 5. The middle is not inert (client + core; decisions, progression)

Y, Z: "every card: unlock read `reach ~0`"; "gold piled up to $2016 with almost nothing to
spend it on".

- The unlock tile shows both prices (`◆3 · $450`) — the gold path is visible without opening
  the panel; a tile affordable with gold glows like one affordable with marks.
- A card's reach is measured with the card's rows in their best position (the top), not at a
  fixed row, and a card whose best reach is within its ± reads `~0` *and* names its situation
  (`kite archers · vs archers`) so the player knows when it matters.
- Gate: ui.mjs — the tile's two prices; a test that a card's measured row position is its best.

## Gates

| Gate | Bar |
|---|---|
| Beats ≥ 2.5 s wall time; the cage ≤ 6.5 s; `fast` ≤ `fights` in wall time | fights.mjs, clarity.mjs |
| Hero ≥ 70 % unoccluded beside a boss; one callout line over a fight | fights.mjs |
| `ForecastDepth.wall` names the sealing boss | test |
| No `no item` beside a found-unknown; stall cause names a row loop; stall tallied; APPLY doesn't send | tests, ui.mjs |
| Unlock tiles carry both prices; cards measured at their best row | ui.mjs, test |
| All bots, dice, stalls ≤ 1 % on DEFAULT and every cohort set | `node tools/gates.mjs --full` |
| Cohort 14: mean ≥ 75; feel ≥ 0.8 from one rater; α ≥ 0.80 | two blind cards |
