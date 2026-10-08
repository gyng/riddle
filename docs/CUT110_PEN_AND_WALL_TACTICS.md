# Cut 110 — tactics answer the walls; the pen opens at the Mother

*2026-10-08. Cohort ad71e72 (57.1 · 56.3, α .61): both raters found Custom rules locked the whole
horizon ("Meet Bloat Mother / 72h elapsed"), expression 0.3 on both cards; A: "only three tactics
existed and nothing addressed the golem wall"; B: "all three lines ended as Guarded fighters with
corridor fighting". Owner: **custom rules are not the player's primary tuning — tactics are, to
reduce busywork**; the pen opens at the Mother met (no age).*

## 1. The pen (`systems::PEN_AGE_H` 72 → 0)

The pen's group opens at the Mother met (one reveal), or at 120 h whatever the climb. The lock's
explanation follows (`pen_needs`: `Meet Bloat Mother · or 120h elapsed`). Optional, late: nothing
in the climb needs it (IDLE/PICKED never write rows).

## 2. Wall tactics (`packages::WALL_TACTICS`)

A wall's counter is a **tactic** that arrives when the wall is met (Warlord slain), beside the drip
(never counted on it), and plays only in its situation:

| tactic | arrives | row |
|---|---|---|
| mirror read (`reflect_read`) | a reflector met (`foe:*:reflect_melee` — the iron golem, the Foundry Master) | `foe reflect_melee → tactic reflect_read`: shoot / burn / step away, never melee |
| quiet steps (`noise_discipline`) | a blinder met (`blind`) | `in deep · hp < 90% → tactic noise_discipline` |
| deep march (`deep_march`) | the Deep entered | `in deep → tactic deep_march` |

The cards already existed as mark unlocks for the pen (meta T4–T5); a player who never writes rows
could not reach them. Gates: `tests_cut30_pkg::wall_tactics_arrive_with_their_wall`; the routine
gate table without lowering a threshold.

## 3. The mirror, softened (`turn::damage_monster`)

Owner: "the golem shouldn't be a hard wall — make the mirror less punishing". A melee blow on a
`reflect_melee` foe (iron golem, warden, Foundry Master) sends **a third back** (at least 1) and the
rest **lands** — was: the whole blow back, nothing landed, so a melee hero could not hurt a golem at
all. `reflected!` still reads it; mirror read (shoot · burn · step away) stays the better answer.
Tests: `iron_golem_reflects_melee_but_not_arrows` (lands, at a price, less back than lands), the
Sentinel riposte (one riposte, no recursion; new numbers).

## 4. Deviation: `Workers pay the away player` (dayplayer, routine full)

After §3 the row fails: D18 48 vs 48 h · mean best **28.86 vs 29.33** (16 seeds; 29.06 vs 29.64 at 40).
It passed before (24 vs 48 h · 28.50 vs 28.11). Bisect (targeted row, fresh): the generator, the pen age
and the Legacy-in-rest fix do not move it; the softened mirror does (whole reflection again: 27.79 vs
27.23, PASS) — both away players go deeper, the one doing every chore by hand at its check-in more so.
Worker ablation: only the armourer's found weapons matter (without them the two tie at 40 seeds); a
clearer-edge rule (115 %, 130 %), wield-only and no-slower-arm filters did not move it. No worker change
honestly restores a strict win, and none was taken to pass the row. **Recorded deviation** (AGENTS.md
"record a deviation with a reason"), owner-directed design; the row's threshold is unchanged. `Workers
never cost the daily player a floor` passes (30.31 vs 30.43). Follow-up: a worker that pays the away
player at a wall (e.g. the drillmaster wearing a wall tactic as it arrives, announced and revocable).

Update 2026-10-08 (Cut 112 apprentice order, Cut 113 late walls): the row reads 28.25 vs 28.36 after
Cut 112, then **27.71 vs 28.04** after Cut 113's harder late bosses (both away players slow at the King;
the one forging by hand at its check-ins keeps its edge). Still a recorded deviation, threshold
unchanged; its fix is Cut 114 §3 (a worker that pays at a wall).
