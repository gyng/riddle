# Cut 15 — The two currencies meet a decision

*Contract, 2026-09-23. Cohort 11 (build 8231993): 66.5 · 70.8, α 0.897, mean 68.7.
`docs/PLATEAU.md` "Cohort 11": Cut 14's lapses are gone from the cards; both raters found the
economy first. Progression fell 0.8 → 0.6 from both.*

## Design

### 1. A bank near the frontier pays a mark (core; progression, tension, return)

U: "returns paid ◆0 while deaths paid ◆+7/+8, so playing safe stalled the unlock track"; V:
"+1 row costs ◆11 and most returns pay ◆0". Marks come from new depths, first kills and
trophies; a set that banks safely earns none after its first week, so the idle loop's safe
play and its progression pull apart.

- **Frontier banks.** A bank from depth ≥ `best_depth − 1` pays ◆1 (on top of the new-best
  marks it may already earn). A bank shallower than that pays nothing; a return pays nothing.
  So the safe set's night earns marks only if it banks near where the lineage has been —
  "bank at D8" earns, "bank at D3" does not. The exit line names it (`· ◆+1 frontier`).
- **Offline counts the same.** An 8 h night of 16 frontier banks is ◆16; that is the point, and
  the catalogue has to absorb it: the gate is the dayplayer's.
- Gate: `DEFAULT yields 0 xp/gold` holds (DEFAULT dies); EDITED's marks per 8 h printed;
  the dayplayer's `Marks unspent at any check-in after day 2 ≤ 8` holds; `Days with ≥ 1
  unlock` moves toward its 10/14 (informational, recorded).

### 2. Gold buys what marks buy, at a price that climbs (core + client; progression, decisions)

U: "$600+ and nothing worth buying"; V: "$1488 banked with nothing worth buying".

- Every unlock in the catalogue can be bought with gold instead of marks at
  `GOLD_PER_MARK × cost × (1 + gold_buys / 4)` (`GOLD_PER_MARK` = 150; the fourth gold buy
  costs double the first). Marks stay the cheap path; gold is the patient one. The unlock sheet
  shows both prices (`◆3 · $450`); a buy with gold is a ledger line (`unlock throw −$450`).
- Gate: a test that a gold buy spends gold, not marks, and raises the next gold price; the
  wire invariant `an available unlock buys` covers the gold path; the dayplayer's bars as §1.

### 3. A patch never takes a row silently (client; attribution, expression)

U: "the caster patch replaced my drink unknown row"; V: "tapping a patch silently deleted my
least-fired row, which was my D8 bank row." Cut 14 §4's `↑ R3` is withdrawn.

- On a full set a patch chip reads `+ drop one`; the tap opens a sheet of the set's own rows,
  each with its fired count (`R5 bank · fired 0/1`), the least-fired marked; the player taps
  the row to drop, and the patch goes in at its measured place. Cancel leaves the set whole.
- Gate: patch-overflow.mjs: a full set's patch opens the sheet; no row leaves before a row is
  tapped; the tapped row is the one gone.

### 4. The watch's peaks are on screen (client; feel, pacing, story)

U: "the Goblin Warlord kill reached me only as ticker text in a few seconds"; "many 1× frames
were a black floor card"; "the fights/fast chips never showed which speed I was in". V:
"`CAPTIMONKEY`"; "empty stretches of up to 11 seconds where the hero only picks things up".

- **A boss kill is a beat** (as the bank is, Cut 14 §3): the fight frame holds `SCENE_MS` on the
  kill with `BOSS DOWN` (the boss's name) as the callout, in `fights` and `fast`.
- **The floor card at 1× is short**: ≤ 1.2 s of wall time, and never over a fight or a beat.
- **The speed chip shows its rate** as a small number on the lit chip (`fast 16`, `fights 2`
  while a fight plays): no word, the digits.
- **Name tags never overlap**: a second tag on a row that would intersect moves a row down
  (the map frame and the fight frame alike).
- **A chore stretch runs at the mode's flat rate** in `fights` too (the pick-up chore is dead
  air; only a fight, a beat or a near hold slows the clock).
- Gate: fights.mjs: a fake boss kill opens the frame with `BOSS DOWN`; two tags on intersecting
  rects draw on two rows; clarity.mjs: the lit chip carries its rate; the floor card ≤ 1.2 s.

### 5. The cage waits for the player who is watching (client + core; autonomy, story)

V: "the cage's choose one sheet expired before I could tap it every time."

- While a watched run's vault sheet is open, the world waits for the tap (the pump does not
  step the engine), up to 30 s of wall time; then the preference picks as now. Offline and in
  sims the engine's 50-tick grace is unchanged (determinism: the choice is an input, as
  `choose` already is).
- Gate: clarity.mjs: with the sheet open for 8 s the frontier does not move and the chips
  still take a tap.

### 6. Small (core)

- `hazard first` on every row of a death footer reads as nothing (V): the footer says it once
  (`all rows · in gas`), with the hazard's name.
- `R1 retreat caught him.` on a run he survived (V): a sealed low point's turn beat reads
  `R1 retreat saved him.` when the resolution is `lived` / a home exit.
- A `below_bar` candidate that survives 0 % is not shown (U: `survives 0% · base 0%`): the
  dice screen shows only candidates that survive > 0.

## Gates

| Gate | Bar |
|---|---|
| Frontier banks pay ◆1; returns and shallow banks pay 0 | test |
| Gold buys any unlock at the climbing price; ledger line; marks untouched | test, `examples/qa.rs` |
| Marks unspent after day 2 ≤ 8; days with ≥ 1 unlock (informational, recorded) | dayplayer |
| A full set's patch asks which row to drop | patch-overflow.mjs |
| Boss kill beat; tags never overlap; the chip's rate; floor card ≤ 1.2 s | fights.mjs, clarity.mjs |
| A watched cage waits for the tap | clarity.mjs |
| Bots hold: DEFAULT dies by D6, EDITED ≥ +15, RANDOM/PASSIVE lose, LEARNED ≤ 2 floors, dice ≤ 5 %, stalls ≤ 1 % on DEFAULT and every cohort set, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 12: mean ≥ 75; progression ≥ 0.8 from both; α ≥ 0.80 | two blind cards |

## Tracks

- **Core**: §1, §2 (engine, wire, `buy_unlock` with a `gold` flag), §6, their tests.
- **Client**: §2 sheet, §3, §4, §5, their tests.
- **Coordinator**: QA round, cohort 12.
