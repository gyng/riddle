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
