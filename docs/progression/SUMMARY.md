# Progression timeline — measured (readable summary of `timeline.json`)

*2026-09-27. `examples/progression.rs` on the working tree at 9720ff7 (Cut 28 + uncommitted Cut 28b; two
runs, bit-identical). 24 fresh lineages × 14 days: the dayplayer's policy verbatim (3 seeds, 3 × 8 h
check-ins), the same with the gold sinks (forge steps, oaths; 3 seeds), and the nine rater sets of the last
four cohorts (631fe23 AU AV · 420f27c AS AS-stall AT · 307dbed-ctl AQ AR · 307dbed AO AP; 2 seeds each)
replayed as a goal on the three-absence day (8 h · 20 m · 4 h · 4 h · 7 h 40 m): each check-in the set is
written as far as the lineage can (its rows in order, cut to the row cap, the spine kept), what it needs
is bought first, then the dayplayer's rule (cheapest, down to a reserve of 8), forge steps when the purse
pays them with $150 spare, one oath when the board is open.*

**Measure.** An unlock's move is leave-one-out at the day's end: the lineage's camp panel (50 sims,
paired seeds) with it vs without it. *Meaningful* = bank share or reach at the half-reach floor +≥ 3 pts
past its ±, or mean depth +≥ 0.25 floors past its ±. The 50-sim panel resolves ~±10 pts; "no move" means
nothing that size. The wall probe (`walls.json`) runs on every stalled day.

## Headlines

| | dayplayer (3) | rater sets (18) |
|---|---|---|
| Days with ≥ 1 unlock (bar ≥ 10) | 5 · 6 · 8 (mean 6.3 — the gate's own number) | 4–7 |
| Unlocks bought on day 1 | 15–16 (◆25) | median 21.5 (◆32–65) |
| Last *meaningful* unlock | day 2 · — · day 4 | day 1–3 (one lineage day 9) |
| Last new best | day 2 · 7 · 5 | day 1–13; 10 of 18 by day 3 |
| Longest best-depth stall (bar ≤ 3) | 12 · 7 · 9 days (D17–18, counter known) | 4–13 (AU AV AT AQ AR: 11–13) |
| Marks earned, 14 d | ~72 (30 on day 1, **0 from day 8**) | median 388, **78 % frontier banks** |
| Marks unspent, worst check-in after day 2 (bar ≤ 8) | 6–8 | median 287 at day 14, max 859 |
| Gold at day 14 | $1.7k–5.1k | median $213k (in $251k, spent $35k: forge $22k, unlock buys $5k, restock) |
| Forge complete (w3 a4 p4) | never (plain) | day 3–6 |
| Class L10 | never (L4–6) | day 4–8 |
| Reveal ladder | all 14 steps by day 1 | all by day 1, oaths day 2 |

## Marks by source (per lineage, 14 days)

- dayplayer: lifetime trophies 22 · new depth 17 · biome studied 9 · run trophies 8 · bosses 6 · first bank
  per depth 5 · ranks 4 · frontier banks 0.7.
- raters: **frontier banks 325** · lifetime trophies 22 · ranks 20 · depth 17 · first bank per depth 10 ·
  studied 8 · run trophies 8 · bosses 7 · mastery 2. A rater set that banks at its record pays ◆1 a bank:
  15–40 marks a day from day 4 with nothing left to buy.

## Unlocks bought → meaningful (leave-one-out, day's end)

- raters: row5 14/18 · row6 14/18 · row7 14/18 · row8 12/18 · vault3 6/18 · vault4 4/18 · row9 2/8 · boss
  focus 2/18 · throw 1/18 · vault2 1/18 · vault5 1/10 · lantern rig 1/6 · forge day-buys 11/74; **0/18**: all
  six condition words, kite archers, stair dance, corridor fighting, gas step, last stand, pack break, thief
  guard, bone sense, supply cap, quartermaster, auto insure, rogue, ranger, caster (0/16), recall sense,
  reflect read (0/9), phalanx.
- dayplayer: vault2 1/3, party slot 2 1/3; everything else 0 (rows 5–6 included: its rows are not better
  rows). Two of three dayplayers end holding ◆6 beside `row7` at ◆7 with no income — one mark short for
  eight days.

## The walls (`walls.json`)

- Rater sets with a `depth ≥ N → bank` row (AU D14, AV D13, AT D11, AR D14, AQ D19) never pass N in the
  sims (`beyond` 0.00). Moving that row to record + 1 passes the record on **43–100 %** of sends (mean +0.46
  over 200 probes, bank −16 pts). The stall is the exit row, and the frontier mark pays for keeping it.
- The boss's counter written: +0.01 mean; the whole forge: 0.00; routes (the other stair at D5/D9/D14/D19):
  ≈ 0 mean, up to 0.47 on single days for AS/AP/AO at D19–22.
- The dayplayer at D13/D17–18: every probe (counter, forge, push, routes) 0.00–0.18: a strength wall.

## Per-day tables

### dayplayer (3 lineages) — medians (max)

| day | best | new best | ◆ earned (frontier) | ◆ spent | ◆ unspent max | $ end | $ max | unlocks bought | meaningful | sinks: forge · restock |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 8 | 3/3 | 30 (0) | 25 | 7 (8) | 754 | 1083 | 16 | 0/3 | 0 · 1012 |
| 2 | 8 | 1/3 | 7 (1) | 9 | 5 (8) | 1213 | 1778 | 5 | 1/3 | 0 · 1675 |
| 3 | 12 | 1/3 | 5 (0) | 8 | 4 (6) | 1628 | 1932 | 2 | 0/3 | 0 · 0 |
| 4 | 13 | 2/3 | 4 (0) | 5 | 5 (5) | 1680 | 2298 | 1 | 1/3 | 0 · 0 |
| 5 | 17 | 1/3 | 3 (0) | 0 | 5 (6) | 1684 | 3110 | 0 | 0/3 | 0 · 62 |
| 6 | 17 | 0/3 | 0 (0) | 0 | 2 (6) | 1237 | 4099 | 0 | 0/3 | 0 · 124 |
| 7 | 17 | 1/3 | 4 (0) | 0 | 6 (7) | 952 | 4099 | 0 | 0/3 | 0 · 248 |
| 8 | 17 | 0/3 | 0 (0) | 0 | 6 (6) | 716 | 3962 | 0 | 0/3 | 0 · 236 |
| 9 | 17 | 0/3 | 0 (0) | 0 | 6 (6) | 1168 | 3962 | 0 | 0/3 | 0 · 75 |
| 10 | 17 | 0/3 | 0 (0) | 0 | 6 (6) | 2199 | 3962 | 0 | 0/3 | 0 · 0 |
| 11 | 17 | 0/3 | 0 (0) | 0 | 3 (6) | 3763 | 3887 | 0 | 0/3 | 0 · 124 |
| 12 | 17 | 0/3 | 0 (0) | 0 | 3 (6) | 2361 | 3763 | 1 | 0/3 | 0 · 2100 |
| 13 | 17 | 0/3 | 0 (0) | 0 | 5 (6) | 1937 | 5101 | 0 | 0/3 | 0 · 75 |
| 14 | 17 | 0/3 | 0 (0) | 0 | 5 (6) | 1725 | 3662 | 0 | 0/3 | 0 · 249 |

### dayplayer+sinks (3 lineages) — medians (max)

| day | best | new best | ◆ earned (frontier) | ◆ spent | ◆ unspent max | $ end | $ max | unlocks bought | meaningful | sinks: forge · restock |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 8 | 3/3 | 29 (0) | 25 | 7 (8) | 306 | 684 | 15 | 0/3 | 600 · 1050 |
| 2 | 8 | 1/3 | 8 (1) | 9 | 6 (8) | 496 | 699 | 4 | 0/3 | 600 · 600 |
| 3 | 11 | 3/3 | 13 (1) | 11 | 7 (7) | 496 | 1333 | 4 | 0/3 | 0 · 562 |
| 4 | 13 | 2/3 | 7 (0) | 9 | 3 (6) | 503 | 664 | 2 | 0/3 | 0 · 0 |
| 5 | 13 | 1/3 | 8 (0) | 6 | 3 (5) | 477 | 1678 | 1 | 0/3 | 0 · 174 |
| 6 | 13 | 1/3 | 0 (0) | 0 | 2 (5) | 496 | 1271 | 0 | 0/3 | 0 · 0 |
| 7 | 13 | 0/3 | 0 (0) | 0 | 2 (4) | 477 | 2428 | 0 | 0/3 | 0 · 0 |
| 8 | 13 | 0/3 | 0 (0) | 0 | 2 (4) | 496 | 1044 | 0 | 0/3 | 0 · 62 |
| 9 | 13 | 1/3 | 0 (0) | 0 | 2 (4) | 637 | 1044 | 0 | 0/3 | 0 · 0 |
| 10 | 13 | 0/3 | 0 (0) | 0 | 2 (4) | 496 | 1163 | 0 | 0/3 | 0 · 75 |
| 11 | 13 | 0/3 | 1 (0) | 0 | 2 (5) | 932 | 1250 | 0 | 0/3 | 0 · 0 |
| 12 | 13 | 0/3 | 0 (0) | 0 | 2 (5) | 700 | 862 | 0 | 0/3 | 0 · 75 |
| 13 | 13 | 0/3 | 0 (0) | 0 | 4 (5) | 700 | 1262 | 0 | 0/3 | 0 · 310 |
| 14 | 13 | 0/3 | 0 (0) | 0 | 4 (5) | 700 | 1610 | 0 | 0/3 | 0 · 434 |

### rater (18 lineages) — medians (max)

| day | best | new best | ◆ earned (frontier) | ◆ spent | ◆ unspent max | $ end | $ max | unlocks bought | meaningful | sinks: forge · restock |
|---|---|---|---|---|---|---|---|---|---|---|
| 1 | 13 | 18/18 | 59 (5) | 52 | 8 (8) | 714 | 1517 | 21.5 | 15/18 | 2400 · 4837 |
| 2 | 13.5 | 7/18 | 15.5 (6.5) | 18 | 7 (8) | 1451 | 2924 | 5 | 4/18 | 5100 · 6967 |
| 3 | 14 | 8/18 | 15.5 (9.5) | 9 | 7 (51) | 2197.5 | 9173 | 2 | 4/18 | 8100 · 9517 |
| 4 | 14.5 | 3/18 | 19.5 (16.5) | 0 | 21 (127) | 4478.5 | 30229 | 0 | 0/18 | 4800 · 5588.5 |
| 5 | 15.5 | 2/18 | 20 (17) | 0 | 33 (186) | 15573.5 | 48487 | 0 | 0/18 | 0 · 1541 |
| 6 | 16 | 4/18 | 20.5 (17.5) | 0 | 60.5 (263) | 35792 | 71726 | 0 | 0/18 | 0 · 311.5 |
| 7 | 17.5 | 2/18 | 21 (19) | 0 | 69 (335) | 53418.5 | 96027 | 0 | 0/18 | 0 · 329.5 |
| 8 | 18.5 | 3/18 | 21.5 (19.5) | 0 | 89.5 (410) | 75694.5 | 131104 | 0 | 0/18 | 0 · 336 |
| 9 | 19 | 2/18 | 33.5 (31.5) | 0 | 123 (488) | 94042.5 | 192830 | 0 | 1/18 | 0 · 411 |
| 10 | 19 | 1/18 | 31.5 (30.5) | 0 | 154.5 (558) | 120398.5 | 251492 | 0 | 0/18 | 0 · 417.5 |
| 11 | 19 | 1/18 | 41 (40) | 0 | 197 (629) | 145954 | 313510 | 0 | 0/18 | 0 · 460 |
| 12 | 19 | 0/18 | 23 (22) | 0 | 220 (704) | 168677 | 372350 | 0 | 0/18 | 0 · 404.5 |
| 13 | 19 | 1/18 | 21 (21) | 0 | 241 (782) | 191425.5 | 432006 | 0 | 0/18 | 0 · 330 |
| 14 | 19 | 0/18 | 35 (34.5) | 0 | 287.5 (859) | 213172.5 | 485115 | 0 | 0/18 | 0 · 385.5 |
