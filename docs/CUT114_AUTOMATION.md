# Cut 114 — automation that serves the player (queued after Cut 113)

*2026-10-08, owner: "follow up progression improvements with automation improvements specifically".
Today (`tree.rs`, docs/AUTOMATION_TREE.md): eleven workers on one trunk; a chore done by hand 2–3 times
(or a fallback age, 24–44 h) lights its worker; one hire at a time for a share of a forge unit; ranks
II–III after 4 · 8 · 12 days; each worker's first act is announced and the worker can be paused.
Cohorts c4705f9 · 1fb7786: the apprentice "spent −$24 500 of my gold without asking" (both raters);
"most deaths came offline under drilled/default rules"; "every return asked forge + legacy + tactics +
unlock + worker taps ×3 bloodlines"; and `Workers pay the away player` fails by half a floor
(Cut 110 §4, a recorded deviation).*

## Principle

A worker carries out an order the player gave and reports what it did — never a silent policy
(AGENTS.md: policy is chosen, drilled or written, never silent; nothing punishes absence).

## Queue

1. **Every worker takes a standing order** (Cut 112 starts with the apprentice): one chip of two or three
   words each — apprentice `all spare · half · off`; clerk `keep N nights`; armourer `wear finds · keep
   forged`; guide `frontier · safe · off`; keeper `keep pref`. Default orders leave the returning player a
   purse and a decision. Gate: IDLE bars unchanged (its defaults); PICKED's orders measured on the
   paired panel; copy budgets.
2. **A worker's ledger on every return**: per worker what it did and at what cost (`apprentice · sword +3 ·
   mail +2 · −$X`, `clerk · banked $Y`, `guide · start D19`), one line each, folded under the report's
   head. Gate: every worker act in the absence appears (count = `tree.acts` delta); gold reconciles.
3. **Workers that pay at a wall** — the Cut 110 §4 deviation's fix: the drillmaster swaps a known wall's
   counter tactic into a slot while the sends meet that wall and swaps the player's tactic back after
   (announced `DRILLMASTER · mirror read`, revocable, never over a pen row). Gate: `automation-pays`
   PASS without a lowered bar; `workers-daily` holds.
4. **One order for every bloodline** (with Cut 113 §7): an order set on one bloodline offers `same for all`;
   hires stay per town. Gate: taps per three-bloodline check-in ↓ (ui QA script).
5. **Ranks you choose**: at rank II a worker offers one of two perks (apprentice `cheaper steps · keeps a
   reserve`; scout `shorter rest · safer start`) instead of a fixed bonus. Gate: neither perk dominates on
   the dayplayer's AWAY and DAILY bots.
6. **What comes next, visible**: the town's next worker shows its chore count (`forge 2/3`) and its
   fallback (`or 30 h`) on its post; a lit hire glints once (≤ 3 markers). Gate: cut30town's marker and
   fold budgets.
7. **After the clear**: a late worker (the herald's rank III or a new `chronicler`) carries the post-clear
   Ascension sends the owner opted into, so a cleared dungeon keeps an away loop. Gate: no auto-ascension
   without an explicit order; IDLE never slays the King in a fortnight (holds).

### §3 landed (2026-10-09, blind 77030eb): the scout's order at a wall

A (77030eb): "the 20m absence came back with a D28 run and $10 949; the 8h absence came back with $1 496, four
dead heirs and the best depth unchanged". B: twice `stuck · went home` at `alert 8/8 · Path blocked`, no trace or fix.

- **Scout · at a wall** (`StandingSwitches.wall`, `WALL_ORDERS` `bank · carry · push`, default `bank`;
  `tree::{note_wall, wall_hold, strength}`): two heirs dead in a row on one floor within two of the record make a
  wall (a send past it forgets it). `bank`: the scout's sends bank at the stairs to it (`before Queen · banked`,
  the run never sets foot on it) while the hero is no stronger than at the last death (forge steps, class level,
  Legacy, worn packages and levels, rules); every `WALL_RETRY` (3) held sends, or as soon as he is stronger, one
  tries it again, its haul carried home from the stairs (secured as a record's). `carry`: every send's haul is
  carried home from the stairs and the heir goes on. `push`: as before. A send by hand always pushes; a paused
  scout carries no order. The report's scout line names it (`sent 9 · banked before Queen ×6`, `carried $X ·
  Queen`), announced as a first act. Harness (literal) lineages and sims never carry it.
- **A floor given up carries the stall's record**: the idle floor's walk home from a floor the guard stopped
  twice (Cut 30 §1, `Run.bail`; `engine::gave_up`) now gets `trace::stall_record` — the guard's moment as the
  cause, the trace, the patches that leave the floor (a replay that only gives the floor up again is not the loop
  broken), `went home` as its margin, linked from the runs log (`death_id`); its exit keeps the return's share.

Measured (16 seeds × 14 days, `automation-pays` mean best AWAY vs AWAY − nodes · D18 h):

| order | without the wake's temperament keep | with it (the shared tree, 2026-10-09) |
|---|---|---|
| `push` (no order) | 28.61 vs 28.88 · 48 vs 48 | 27.09 vs 27.26 · 72 vs 60 |
| `bank` (default) | 28.58 vs 28.78 · 48 vs 48 | 27.04 vs 27.25 · 72 vs 48 |
| `carry` | 28.65 vs 28.88 · 48 vs 48 | 27.20 vs 27.33 · 72 vs 60 |

Under `bank`, without the temperament keep, every IDLE bar holds (D8 16/16 · D13 day 1.3 · D23 16/16 by day 4.0 ·
stall 4 d · gold 14/14 · King 0/16 · stages 10 · idle-delta +0 h · hands-idle PASS), `workers-daily` 29.70 vs
29.35 PASS, and the metrics table all PASS. The client's temperament keep (packages.rs `wake`) alone fails
IDLE D23 (3/16), the IDLE stall (11 d) and `Quests keepable` (worst 0 %), with the order `push` as with `bank`.

Probe (`examples/wall_absence.rs`, 36 walled AWAY camps from days 3 · 6 · 8): an 8 h absence hauls **$12 041
under `bank`** vs $9 529 pushing (+26 %), deaths 3.9 vs 8.2, new floors +0.50 vs +0.69; `carry` $11 213, 9.0
deaths, +0.69. A 20 m absence out-paid the 8 h one on 1/36 camps under every order (the rater's case is a 20 m
run that set a record: its carry was secured by the record's checkpoints).

`automation-pays` still **fails** under every order: the gap is days 1–4 (mean best day 1 12.7 vs 14.0, day 3
20.3 vs 22.1 — the first session's hires against the forge), before the deep walls; at the walls (days 5–9) AWAY
is level or ahead. The deep walls are day-gated (a scar and the drill per day met), so no wall order moves the
fortnight's depth; the order pays the absence instead. Cut 110 §4's deviation stands, threshold unchanged.

Each item lands with its gate, the routine full gate table without a lowered threshold, the full client
suite, then a fresh blind pair.
