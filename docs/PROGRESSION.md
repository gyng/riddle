# Progression — measured, diagnosed, re-tiered

*Proposal, 2026-09-27. No game code changed. Data: `docs/progression/timeline.json` (measured, 24 lineages
× 14 days, `examples/progression.rs`), `walls.json` (the wall probe), `SUMMARY.md` (readable tables),
`proposal.json` (the same per-day shape projected by `project.py`, plus the per-day system table).*

## 1. Diagnosis

**The curve is a day-1 dump, then a flat line.** A fresh dayplayer buys 15–16 unlocks on day 1 (◆25 of
◆30 earned) and 5 on day 2; a rater set buys a median 21.5 on day 1. The catalogue is 44 unlocks for ◆188,
twenty of them ◆0–3 and open on facts every first hour meets. The last *meaningful* unlock (leave-one-out
panel move past its ±) lands on day 2–4 for the dayplayer and day 1–3 for 17 of 18 rater lineages. Days
with an unlock: dayplayer 5 · 6 · 8 (the gate's 6.3), raters 4–7. The reveal ladder is spent the same way:
all 14 steps open by day 1 (oaths day 2) — nothing is left to open in weeks 1–2.

**Stalls are two different things.**
- *Self-imposed* (AU D14, AV D13, AT D11, AR D14, AQ D19: 11–13 days): the set's `depth ≥ N → bank` row.
  The sims never pass N (`beyond` 0.00); moving it to record + 1 passes on 43–100 % of sends (bank −16
  pts). The frontier mark (Cut 15: ◆1 per bank at ≥ best − 1) *pays for staying*: 78 % of rater marks
  (325 of ~415 per lineage) — the "same D13 bank every run" of cohorts 21–23 is the marks-optimal policy.
- *Strength* (dayplayer at D13/D17–18: 7–12 days, counter known; AO/AP/AS at D20–23: 4–8): no probed lever
  moves it — the boss counter written (+0.01), the whole forge (0.00), the exits pushed (0.00 for the
  dayplayer), the other stair at D5/D9/D14/D19 (≈ 0 mean, ≤ 0.47 on single days).

**Unlocks that never matter** (0/18 rater lineages, 0/3 dayplayers): all six condition words, eight of
eleven cards (kite archers, stair dance, corridor fighting, gas step, last stand, pack break, thief guard,
reflect read), the three classes (the sets are fighter sets; nothing makes a second class a way to play),
every automation (quartermaster, auto insure, supply cap, bone sense, incubator), recall sense, phalanx.
What matters: rows 5–8 (12–14/18), vault 3–4 (4–6/18), row9 (2/8), boss focus (2/18). For the dayplayer
only vault2 and party slot 2, once each: extra rows help only a player who has rows worth adding.

**What piles up and when.** Marks: raters from day 3–4 (the catalogue is bought out; frontier income
15–40/day) to a median 287 unspent at day 14, max 859. Gold: raters $251k in, $35k out (forge $22k, done
by day 3–6); $213k median purse at day 14 ($4k in a cohort session is the same slope). The oath board
empties with the catalogue (its rewards *are* catalogue items), so the Cut 28 sink dies on day 2. The
dayplayer is the opposite: income stops (◆0 from day 8: trophies exhausted, no frontier banks), and two of
three end one mark short of `row7` (◆6 of ◆7) for eight days.

**The row pricing oddity.** Marks 2 · 4 · 7 · 11 · **8** · 12 for rows 5–10: Cut 4 priced tier 2 as its own
band (8–12) and let the "3 boss kinds" gate carry the difficulty; the gold ladder (Cut 23) is monotonic in
units (2 · 5 · 10 · 16 · 22 · 30). The cost of row8 (◆11) is also above the ≤ 8 unspent bar — a player who
saves for it fails the bar by construction.

## 2. Principles

1. The wall is the teacher: each tier and each late system opens on a boss *met* or a plateau, not on a
   fact every first hour meets.
2. Marks pay for going deeper and for goals, never for standing still.
3. An unlock that measures ≈ 0 for every lineage is either free vocabulary or a gold convenience.
4. Every price ≤ ◆8 (the unspent bar is then satisfiable), rows monotonic.
5. The late sinks are goals (oaths), not stats: policy stays the lever.

## 3. Marks income

| Change | Evidence | Gate |
|---|---|---|
| **Frontier mark removed**; in its place **the night's mark**: ◆1 per day whose absences brought a send home (bank or return) | frontier = 78 % of rater marks and funds the D13 bank; the dayplayer earns 0.7 in 14 d and 0 from day 8 | dayplayer: ◆ income ≥ 1 on every day; cohort sets: `depth ≥ N → bank` sets' last new best ≥ day 6 |
| Depth, first-bank-per-depth, boss (◆3), trophies, ranks unchanged | 90 % of dayplayer income; spread across the arc | — |
| A pushed floor pays its depth mark as today | push probe: +0.46 beyond | — |

## 4. The catalogue, re-tiered

Tier *k* opens when *k* band bosses have been **met** (their floor reached); tier 1 at the first bank.
Fact gates stay where they are (a card still needs its foe fact).

| Tier (opens) | Marks | Oath rewards (never sold) |
|---|---|---|
| T0 · day 0, free | six condition words (still fact-gated), tame, rogue (first bank), kite archers, stair dance, noise discipline, deep march | — |
| T1 · first bank | row5 3 · throw 3 · vault2 3 | — |
| T2 · Warlord met (D8) | row6 4 · corridor fighting 4 · pack break 4 · thief guard 4 · party-hp word 2 | party slot 2 · verb `hold` · Warlord-burner |
| T3 · Mother met (D13) | row7 5 · ranger 6 · boss focus 4 · vault3 5 · **oath slot 2** 6 | waystone D9 · gas step · party slot 3 |
| T4 · Lich met (D18) | row8 6 · caster 8 · reflect read 5 · vault4 6 | **route 2** (the D9 fork) · last stand · Firebrand |
| T5 · Foundry met (D23) | row9 7 · cadence 5 · lantern rig 5 · recall sense 6 · vault5 7 | **heir pick** (3 traits, not 2) · waystone D14 · party slot 4 |
| T6 · Queen met (D28) | row10 8 · **oath slot 3** 8 | **a set per band** · titles · waystone D19 |
| any tier ≥ 2 | **oath draw ◆2**, repeatable (a fresh standing oath) | — |

- Rows 3 · 4 · 5 · 6 · 7 · 8, monotonic; the gate carries the difficulty, the price the pace (fixes §1's
  oddity and the ≤ 8 conflict). Gold's row ladder stays (units 2 · 5 · 10 · 16 · 22 · 30).
- Condition words become free vocabulary (0/18, ◆10 of day-1 non-decisions). Six paid cards stay paid
  (they carry what rows can't, Cut 23 §3) at ◆4–5 over T2–T5; gas step and last stand become oath rewards.
- Automations leave the marks catalogue for gold (§5): measured ≈ 0 everywhere — chores, by pillar 2.
- Gold buys only rows (the forge ladder) and automations: with $213k purses the climbing gold price of
  any unlock would buy the whole catalogue (≈ $170k) by about day 10.

## 5. Late sinks

- **Marks — oaths.** Oath draws (◆2) and oath slots (T3, T6). The pool's rewards are reserved (the
  table's right column + titles), so the board never empties when the catalogue is bought. Cut 28's
  measure is why this sink *matters*: 19/19 oath best sets differ from bank-optimal by ≥ 2 rows. `bold`
  (record + 1, no return) is the push the frontier mark used to punish.
- **Gold — policy-neutral by construction.** (1) Automations (quartermaster, auto insure, supply cap,
  bone sense, incubator, third tag) at 3–5 forge units, opened at the Lich; (2) an oath is sworn **per
  day** at ¼ of the day's net per slot (Cut 28's ½ a night) (lapses unkept, no refund — a bet the player chose); (3) commissions
  (lineage works: the chronicle's monuments, heir looks, the camp) at 10 units × 1.25ⁿ. None enters a sim
  except the automations, which already measured 0/18. Lever row and KITTED unchanged.

## 6. The system curriculum (owner request)

Systems open one at a time, each on the moment that makes it mean something; no tutorial text (the
reveal *is* the lesson). Today every one is open by the end of day 1 (§1).

| System | Day 0 | Opens on | Why that moment |
|---|---|---|---|
| send, the shaft (D1), forecast headline (`D3 72 %`) | ✓ | — | one number, one button |
| dial (thresholds on the 4 preset rows) | ✓ | — | the first decision is a number |
| edit (verbs/conds: hp · foes · adj; attack · drink · retreat) | | first death | the verdict names a row to change |
| exits (bank · return · rest), loadout | | first gold home | a purse to protect |
| unlocks (next 3 only) | | first mark | |
| reorder, the forecast's vs line | | first plateau (a stall verdict) | order is what the stall verdict blames |
| foe tags + `attack tag:`, cards | | first foe fact | a word for what he met |
| party / taming | | first stray seen | |
| cage | | first cage | |
| boss wall + counters, the divergence scene, the shaft's ends | | the Warlord met (D8) | the first wall the policy must answer |
| forge, waystones / start | | the Warlord slain | gold has a use once the purse is a night deep |
| route (D5 fork) | | the D5 fork seen twice | |
| oaths | | the first plateau or the Warlord met | a goal when stuck |
| automations (gold) | | the Lich met (D18) | chores once there are chores |
| a second class (ranger T3, caster T4) | | bought | |
| route 2 (D9 fork) · heir pick · a set per band | | an oath kept | the late game keeps opening systems |

**Bots.** `metrics::setup` gives every bot the full system set and today's vocabulary (a lineage flag,
`LineageState::systems`, that `Game::new` starts at step 0 and `setup` sets to all), so DEFAULT, EDITED,
RANDOM, PASSIVE, LEARNED, TRIVIAL, KITTED and FULL−Dn play exactly what they play now and their gates are
unchanged. The dayplayer starts at step 0 (it is the fresh player). Systems gate the *editor's*
vocabulary and the camp's tiles; a sim replays whatever the set holds (locks and all, as today).

Projected (`proposal.json` → `systems_by_day`): median open 14 → 17 → 19 on days 1–3, then 19 → 21 by
day 14 (dayplayer); forge day 2–4, route 2 day 9–11, heir pick day 13–14. In-day staging (the first
hours) needs check-in resolution the timeline does not have; the dayplayer's first hour (D6–8, a death,
a bank, a stray) passes every day-1 trigger.

## 7. Projection

`project.py` keeps what was measured (depth curve, bosses, trophies, banks, gold brought home) and
recomputes what the proposal changes. Two assumptions, both from measurements: a sworn oath is kept every
2nd day with one slot, 4 days in 5 with two (Cut 28: each pool oath keepable ≥ 20 %/night by some set,
3–4 nights a day); a stalled player pushes (the plateau note, the `bold` oath), and a push breaks the
stall the next day only where the probe measured it passing the record on ≥ 10 % of sends.

| Bar | Measured | Projected |
|---|---|---|
| Days with ≥ 1 unlock (marks unlock, oath kept, or system opened) ≥ 10/14 | dayplayer 6.3 mean; raters 4–7 | dayplayer 12 · 12 · 12 (content only 11–12); raters mean 11.8, min 9 (AT: its D11 set meets the Mother late) |
| Marks unspent ≤ 8 at any check-in after day 2 | dayplayer 6–8; raters ≤ 859 | dayplayer ≤ 8; raters ≤ 7.8 |
| Longest best-depth stall ≤ 3 days | dayplayer 7–12; raters 4–13 | raters: 13/18 ≤ 3 (every `depth ≥ N` set); AO 8 · 4, AP 6 · 6, AS s2 5; **dayplayer 7–12 unchanged** |
| Purse ≤ max(3 days' net, 10 units) after day 3 | raters $213k median | raters ≤ 1.4 days' net |

**The dayplayer's stall does not move.** Nothing probed breaks its D13/D17–18 wall; the marks it would
buy (rows) do not help its rows. Making the stall bar a real gate needs one more measured lever before it
can pass: **E1** — at a wall held 2 days, offer the forecast's best one-row edit from the vocabulary
the lineage owns (the plateau search `oath_lib` already runs) and measure whether the dayplayer's taking
it clears ≥ 10 % beyond; if no row does, the Crypt band needs a ramp (content), recorded as such.

## 8. Gates

| # | Gate | Where | Bar |
|---|---|---|---|
| G1 | Days with ≥ 1 unlock, oath kept or system opened | dayplayer, now a hard bar | mean ≥ 10/14; and **content only** (no system opens) ≥ 8 on every seed |
| G2 | Longest best-depth stall, counter known | dayplayer, hard bar | ≤ 3 days (fails until E1 lands; recorded, not weakened) |
| G3 | Marks unspent after day 2 | dayplayer + progression.rs cohort sets | ≤ 8 |
| G4 | A meaningful unlock (leave-one-out) in every 4-day window | progression.rs cohort sets | ≥ 3 of the 4 windows on ≥ 70 % of lineages |
| G5 | No marks for standing still | progression.rs cohort sets | frontier source = 0; `depth ≥ N` sets' last new best ≥ day 6 |
| G6 | Purse bounded by sinks | progression.rs cohort sets | ≤ max(3 days' net, 10 units) after day 3 |
| G7 | Every price ≤ ◆8, rows monotonic | `meta.rs` test | — |
| G8 | Bots with the full system set | metrics | DEFAULT ≤ D6, EDITED +15, RANDOM/PASSIVE lose, LEARNED ≤ +2, KITTED ≤ D8 — unchanged |
| G9 | Policy stays the lever with every tier and oath reward owned | metrics lever row | whole forge + every reward < best row move |
| G10 | Oath sets differ, keepable | metrics (Cut 28) | ≥ 2 rows · ≥ 20 %/night, now for the reserved pool |

The dayplayer's `--gate` and `tools/gates.mjs` drop the "informational until M7" carve-out (the descent is
34 floors; the bars fail for the reasons above, not for missing content): G1–G3 become hard bars.

## 9. Risks

- The rater replay is an expert who knows the late set: it front-loads everything; real first days are
  slower, so day-1 purchase counts are an upper bound.
- The projection holds depth to the measured curve; a player who pushes earns more depth marks and opens
  tiers sooner — the measured `unspent` headroom (≤ 7.8 against 8) is thin.
- Oath keep rates are the Cut 28 floor, not a measured player's; a player who never swears loses 5–7
  unlock days (the content-only half of G1 guards it).
- Free condition words enlarge day-0 vocabulary; the curriculum's edit step (hp · foes · adj first) keeps
  the first sheet small.
