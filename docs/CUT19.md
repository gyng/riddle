# Cut 19 — Every lever in sight, every exit a risk

*Contract, 2026-09-24. Cohort 14 (build 205d408): 69.2 · 69.7, α 0.808, mean 69.5 (+1.8).
`docs/PLATEAU.md` "Cohort 14".*

## Design

### 1. The cage is a camp decision (core + client; autonomy, decisions, clarity)

AA: switching the vault's `cage` setting "raised bank-at-D7 from 54 % to 90 %, more than all my
rule edits", found at ~25 min; AA, AB: the in-run sheet "closes before a tap lands".

- The cage pick is a **rule-like camp choice**, shown beside the rule tablets as its own tablet
  (`cage → armour`), with the forecast's delta for each option on its picker (`weapon · armour
  +36% · potion`). It is revealed by the ladder the first time a cage is seen.
- In the run, the cage is a **beat**, not a sheet: `took mail` (the pick) held like a situation
  beat; a tap on the beat within its hold opens the three to override (the world waits while the
  override sheet is open, ≤ 10 s). No sheet the player can miss.
- Gate: ui.mjs — the cage tablet with per-option deltas; a watched cage shows the beat with the
  pick; the override works.

### 2. The way home is part of the run (core; tension, failure)

AB: `hp < 20% → return` made death 1 % and a whole night of 0 deaths banked at D12. A return
is an instant exit at 60 %; nothing stands between the rule and home.

- **A return walks.** `return` now paths to the floor's up-stairs like `bank` does and exits
  there at 60 %; foes can reach him on the way (a hurt hero walking home is the tension the
  genre lives on). `bail` (the player's own button) stays instant.
- The forecast, the verdicts and the stall guard see the walk as any other action.
- Gate: all bot gates hold (DEFAULT dies by D6, EDITED ≥ +15, FULL reaches its depths, dice
  ≤ 5 %, stalls ≤ 1 % on DEFAULT and every cohort set); a test that a return walks and can be
  interrupted by a death; on the cohort-14 sets, the night's death share with a return row is
  > 0 and < the set without it (measured, reported).

### 3. Admin goes (core + client; pacing, progression)

- **The loadout repeats**: the last send's supplies are re-packed at the send by default
  (bought at the shelf's price, a ledger line), `auto: restock` becomes the free default; the
  camp shows `repeat · $120` on the loadout tile and a tap clears it.
- **Restock never spends more than the night brought home**: the offline restock stops at the
  night's income; the report says `restock capped`.
- `+1 row` stays in the unlock list until bought (AA saw it vanish).
- Gate: tests; ui.mjs.

### 4. The death screen agrees with itself (core + client; failure, attribution)

- A death where a row of the player's fired at the end is **not** stamped `gap` when that row
  is the cause (AA: `R2 drink unknown fired` → GAP): the verdict reads `row` (one word: the row
  the player wrote killed him — `R2`), `gap` only when a missing row would have saved him.
- `+ drop one` names the row it would drop (`+ drop R5`) — the least-fired row, still
  overridable by the drop sheet.
- The patch order combines survival and reach: a patch that survives much less (33 % vs 100 %)
  does not outrank on a small reach gain inside the ±; rank by reach only when survival is within
  10 pts.
- Gate: tests, qa.rs (a `row` verdict names a row that fired on the death tick).

### 5. Less repetition (core; surprise, story)

- **Den thefts** thin once the lineage has lost to them: the den's monkeys steal on ≤ 1 in 3
  of the runs that meet them after the lineage has `den` (the thief guard stays the answer).
- **A grudge is avenged once**: `X is avenged` is said on the first kill of that named foe; a
  later kill is `X slain` (no second avenging).
- Gate: tests; the situation gates hold (the den still met by D6, DEFAULT still fails it).

## Gates

| Gate | Bar |
|---|---|
| The cage tablet + per-option deltas; the cage beat and override | ui.mjs |
| A return walks and can die; death share with a return row > 0 on the cohort sets | test, measured |
| Loadout repeats; restock ≤ income; `+1 row` stays | tests, ui.mjs |
| `row` verdict; `+ drop R5`; survival-first ranking | tests, qa.rs |
| Den thefts thin; avenged once | tests |
| Bots, dice, stalls on every cohort set, situations | `node tools/gates.mjs --full` |
| Cohort 15: mean ≥ 75; tension or autonomy ≥ 0.8 from one rater; α ≥ 0.80 | two blind cards |
