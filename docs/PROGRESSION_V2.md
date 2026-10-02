# Progression v2 — a curve for months: gates by time, a prestige loop by day 2, automation as rewards

*Design, 2026-09-30. No src changes. The owner: "refine/review and improve the progression curve", after a
comparison with the best idle games: Riddle (a) has no idle floor yet (Cut 30 fixes that), (b) has no early,
repeatable prestige loop (ascension only at D34), (c) opens too much on day 1 for a skilled player (the
curriculum gates by skill, not by time or milestones), (d) doesn't make automation a reward ladder, (e) spans
~2 weeks of content, not months. Inputs: HANDOFF §0, IDLE_FIRST, CUT30, PROGRESSION + `docs/progression/`,
TOWN, `research/`, and a new measurement: the Cut 30 core's checkpoint-2 dayplayer (`cabf89d` + its tree, 8 seeds ×
14 days, IDLE/PICKED/TUNED). Model: `docs/progression/v2_project.py` → `v2_calibration.json`, `v2_projection.json`.*

## 0. Where the curve is today (Cut 30 checkpoint 2, measured)

- **The idle floor works.** IDLE (never picks) reaches D9–10 on day 1 (median), D13 on day 2, D18 on day 3, D23 on
  day 9, D25 on day 14; ~19 runs per 8 h check-in. Steady L5 by day 4; class L10 by day 9.
- **Day 1 is a dump.** One 8 h report announces up to **14 items** (`built blacksmith · built storehouse · +kite
  archers · STEADY L2 · +Guarded · +Hunter · DRILLED · Warlord · built bank`, then `+Bold · +boss focus · +corridor
  fighting · +thief guard · +gas step · +pack break · QUEST DONE`): the six tactics arrive in one beat, the pen opens
  on day 1–2 (the Mother met). By day 3 nothing is left to open but levels.
- **After day 7 the reports go quiet**: ≤ 2–4 items, a new floor every 1–3 days, gold piling (IDLE $41k by day 11).
  An engaged player reaches D34 ~day 11 (Cut 29's dayplayer ascended then); that is (b), (c), (e) in numbers.

## 1. The target curve

**Cadence targets** (§5's bars): first 10 min ≥ 2 new things; first hour ≥ 6, no gap > 15 min; day 1 ≤ 10 systems
(today 18–21 for a skilled player), ≤ 1 system and ≤ 5 beats a report; days 2–3 something every check-in; week 1 a
system a day; weeks 2–4 something new on ≥ 26/30 days, a system on ≥ 16; months 2+ never a week without. "New" = a
system, building, package or class level, boss met, record, quest, relic, automation rung or prestige.

| Horizon | Character | Items | Scale | Town | Prestige | Automation | Idle-only sees · engaged sees |
|---|---|---|---|---|---|---|---|
| **First 10 min** | Steady warrior, first death → heir 2, class L2–3 | first find → storehouse | 1 hero | camp → blacksmith | — | restock (given) | the same: send, watch, harvest, one spend |
| **First hour** | Warlord met → a second stance (idle: the next report); class L3–4 | forge step 1–2 | first pet (kennel) | — | — | auto-equip the better find | idle: 4–8 things, all from the climb · engaged: 13–15 (watching runs 4× faster) |
| **First session** (~20–90 min) | stance L2 | — | — | — | — | — | nothing more opens: **the session ends on a visible "next"** (`next · tactics · Warlord slain`) |
| **Day 1** | tactics (one), rogue | — | waystones | bank, quest board | — | — | idle: 6 systems · engaged: 7–9 (cap 10); one per report |
| **Days 2–3** | temperament, ranger, tactic slot 2 | — | — | library · drills | **expedition 1** (day 2) | forge standing order, sorter, bank sweep | both: the first reset pays the same day (★, a head start, house 1 next) |
| **Week 1** | **the pen** (age ≥ 72 h), caster | loot properties | **house 1 → a second hero** | tavern, watchtower | expeditions 2–3; base camp | quest claim, stance at the wall (opt-in), kennel, route | idle: 1 expedition, D25 · engaged: 2–3, D23–25 at a higher ★ |
| **Weeks 2–4** | advanced classes, bloodlines | relics, relic sets | houses 2–3, expedition board | hall of heirs | **era 1** (days 17–23), variant 1 | wall edit as a drill (opt-in), auto-expedition, auto-lanes | idle: era 1 day 23 · engaged: day 17–19 |
| **Months 2+** | — | a new biome lane (content) | the company (lanes follow a record) | — | eras 2–10 (a variant each), **dynasty** (~day 45) | auto-era | both: an expedition every ~2 days, an era a week; the gap is content, not gates |

**Benchmarks** (sources at the end; [U] = not verified):

| Horizon | Reference | What it does there | Riddle v2 |
|---|---|---|---|
| First 10 min | AD; Soda Dungeon 2, Cookie Clicker | "click the green button", its designer's regret [A1]; a party/building in minutes [U] | blacksmith at run 1, storehouse by run 3 |
| First hour | Trimps | AutoFight at zone 1 cell 10 (Bloodlust) [T1] | auto-equip at run 6 |
| First hour – day 1 | NGU Idle | rebirths 30 min → 24 h set the cadence [N1]; "ascension in the first half hour is a disaster" (review) [R1] | no reset on day 1; first expedition at 36 h |
| Day 1 | AFK Arena, Melvor | chest 12 → 24 h [K1]; offline capped 24 h [M1]; ~5.3 sessions/day, ~8 min [G1] | 3–6 check-ins; uncapped offline (kept) |
| Days 2–3 | Antimatter Dimensions | first Infinity "a few hours to a couple days" [A2] | expedition 1 on day 2 |
| Days 2–3 | Soda Dungeon 2 | Soda Script after 1–3 resets [S1]; a reset keeps the town and buildings [S2] | the expedition keeps the town (§2) |
| Week 1 | Antimatter Dimensions | first Eternity "a couple days to a week" [A2]; autobuyers earned from challenges [A3] | the pen + 6–9 automation rungs |
| Week 1 | Legends of Idleon | a world boss opens the next world and its systems (W2: alchemy, subclasses) [I1] | houses, tavern, watchtower on expeditions |
| Weeks 2–4 | Antimatter Dimensions | first Reality "a week to a month" [A2] | era 1 on days 17–23 |
| Weeks 2–4 | Loop Hero | the whole campaign 29 h main story (HowLongToBeat) [L1] | the D34 bottom first reached days 17–23 |
| Months 2+ | Antimatter Dimensions | "The End" in 1–3 months [A2]; 4 prestige layers each with its own mechanic (review) [A4] | 3 layers (expedition, era, dynasty) + variants |
| Months 2+ | Idleon, Melvor | W1–W7 [I2]; 99s in months [U] | eras and a content lane per dynasty; the release cadence is the real limit (§6) |

## 2. The prestige loop

**Layer 1 — the expedition** (days 2–3, then every 1–3 days). *Fiction*: the Riddle's shaft shifts; the town sends
a new expedition down a freshly seeded descent (it also answers "the same floors every run"). *Opens*: the Mother
met **and** lineage age ≥ 36 h (fallback age 72 h). *Offered* on the report as one tap with its price in plain
numbers (`expedition · ★+9 · start D8`); never automatic until the player earns and switches on auto-expedition.

| Resets | Kept |
|---|---|
| best depth this expedition, waystones and the start stone, the forge kit (left in the old shaft), the purse (paid into the town: every building's stock grows, so the town track grows on every reset) | town (buildings, bank deposits), bestiary/facts, **drills**, packages and their levels, classes and class levels, pets and the kennel, heirs and chronicle, quests, relics, ★ and every system opened |

- **★ renown**, lifetime-based (Cookie Clicker's cube root, AdCap's square root [R2]): each expedition adds glory
  g(peak) = ⌊peak²/30⌋ (D13 5 · D18 10 · D23 17 · D28 26 · D34 38) and ★ = ⌊3·√G⌋. A same-depth reset pays less
  each time, a deeper one more; a 20 h cooldown means a short reset loop is never optimal (the "5-minute run over
  and over" complaint [R2]).
- **What ★ buys**: climb speed ×(1 + 0.15·√★), +2.5·√(★/5) floors at the same hours (walls yield to time, not HP),
  and a head start at the deepest boss floor ≤ ½ the last peak (⅗ with the base camp): the run-back is one absence,
  not a week ("minutes, not hours" [R2]).
- **A new system per expedition** (the owner's "new system per prestige"): 1 house 1 → a second hero · 2 watchtower
  + base camp + loot properties · 3 house 2 · 4 relics · 5 house 3 + auto-expedition · 6 relic sets · then ★ only
  until the era. **First payoff**: expedition 1 on day 2 (all three profiles); the re-climb passes the old peak within
  a day, and house 1 lands on day 4.
- **Layer 2 — the era** (the existing ascension, re-scoped): the Mirror King slain, ≥ 2 expeditions in this era, ≥ 7
  days since the last era, lineage ≥ 10 days. Resets expeditions' glory to √G; keeps everything else. Each era adds a
  **variant** as a new system (`no_rest`, `short_list`, `bones_only`, `hunted` exist in `engine.rs`; eras 5+
  stack two) and ×1.2 speed. First era: TUNED day 17, PICKED 19, IDLE 23.
- **Layer 3 — the dynasty** (~day 45, after 3 eras): the bloodline (the trait core's parked `bloodline`), the hall
  of heirs' grand look, and a **content lane** (a new biome set) per dynasty — the only layer that needs new content.
- **Nothing punishes absence**: every reset is a choice with its gain shown; an offer never expires or decays; runs
  keep paying while it waits; ★ only grows; the purse goes into the town, not away; a player who never taps still
  climbs to the idle plateau (D25–27 by day 14) and loses nothing by coming back late.

## 3. Automation as a reward ladder

Each rung retires a chore the player has already done by hand (Trimps: AutoFight, AutoUpgrade, AutoJobs by
milestone [T1]; AD: autobuyers from challenges [A3]). Each is named, shown on its building, revocable; the two
policy rungs are opt-in toggles and name their rows (the "never silent" invariant).

| # | Rung (chore retired) | Earned by | Min age | Projected day (IDLE · TUNED) |
|---|---|---|---|---|
| 0 | restock: the quartermaster packs the heal and a drill's item | given (the idle floor) | 0 | 1 · 1 |
| 1 | auto-equip the better find | 6 runs | 30 min | 1 · 1 |
| 2 | forge standing order (the next step when affordable, a reserve kept) | 3 forge steps | 20 h | 1 · 1 |
| 3 | keep/salvage sorter (storehouse full → salvage the worst) | 60 runs | 28 h | 2 · 2 |
| 4 | bank sweep (the purse above a reserve earns) | 3 days of the bank | 48 h | 4 · 4 |
| 5 | quest claim + reroll to one the set can keep | 2 quests done | 60 h | 5 · 5 |
| 6 | stance at the wall: the forecast's best stance swapped in at a wall (opt-in) | expedition 1 + a pick made | 80 h | — · 4 |
| 7 | kennel: field the best pets, hatch eggs | 250 runs | 100 h | 5 · 5 |
| 8 | route and start stone follow the record | expedition 2 | 150 h | 8 · 7 |
| 9 | wall edit as a drill: the wall search's row adopted (opt-in, pen open) | expedition 3 + the pen used | 9 d | — · 10 |
| 10 | auto-expedition: take the offer at a chosen ★ threshold | expedition 5 (or age 21 d) | 13 d | 22 · 15 |
| 11 | lanes: heroes auto-send, each following a record | expedition 5 + 3 heroes (or 22 d) | 16 d | 23 · 17 |
| 12 | auto-era with a chosen variant | era 2 | 35 d | 37 · 36 |

## 4. Gating by time and milestones, never blocking the idle player

Three rules replace "opens on the skill trigger":
1. **Trigger AND age.** A system opens when its trigger has happened *and* the lineage is old enough (lineage age =
   real hours since the lineage began, offline included). An idle player ages at the same rate as an engaged one,
   so the age gate never costs him anything; it only stops a skilled player compressing a week into an afternoon.
2. **One system per check-in** (a check-in = a report of ≥ 1 run). Triggers met early wait in a queue shown as the
   next stage (`next · tactics · tomorrow`); prestige offers jump the queue; stages (levels, bosses, records) are
   never capped. The six tactics become a drip (one per boss slain or per day), not one beat.
3. **A fallback age for every skill trigger**, so no system waits on a skill the idle bot lacks (the pen: the Mother
   met or age 5 d; temperament: heir 3 or 3 d; the expedition 3 d; house 1–3, relics, advanced classes 5–20 d).
   Automation rungs have their own queue (one rung per check-in besides the system): a rung sits on its building.

| System | Trigger | Min age | Cut 30 today | v2 IDLE · TUNED |
|---|---|---|---|---|
| send · watch · report; headline | — | 0 | 0 | 0 |
| blacksmith; storehouse | first gold; first find | 0 | first report | min 10–15 |
| kennel · first pet | first stray (D5) | 15 min | day 1 | day 1 (min 20–30) |
| a second stance | Warlord met | 30 min | day 1 (with 2 more at once) | 2 · 1 |
| tactics (a drip) | Warlord slain | 4 h | day 1, **six at once** | 3 · 1 |
| bank; quest board; rogue; waystones | a night's purse; Warlord slain; first bank; Warlord slain | 8 · 12 · 16 · 20 h | day 1 | 1–3 · 1–2 |
| temperament; library · drills | heir 3; Mother met | 24 · 30 h | day 1–2 | 2–4 · 2 |
| expedition | Mother met | 36 h | — (D34 only) | 2 · 2 |
| ranger; tactic slot 2 | Mother slain; Lich met | 48 · 56 h | day 2–3 | 5 · 3–4 |
| **the pen** | Mother met + expedition 1 | 72 h | **day 1–2** | 4 · 4 |
| house 1 · a second hero; tavern; caster | expedition 1; + 150 runs; Lich slain | 72 · 96 · 96 h | Cut 31 | 4–6 · 4–5 |
| loot properties; watchtower; base camp | expedition 2 | 110–140 h | — | 8–9 · 6 |
| house 2; relics; hall of heirs; advanced classes; house 3 | expeditions 3 · 4; heir 5; exp 4 + Foundry; exp 5 | 8 · 10 · 11 · 14 · 17 d | — | weeks 2–3 |
| era; variants; bloodlines; relic sets; the company; dynasty; a content lane | §2 | 10 d – 60 d | ascension at D34 | weeks 2 – month 3 |

## 5. The model

`python3 docs/progression/v2_project.py --calibrate LOG` reads a verbose Cut 30 dayplayer log into
`v2_calibration.json` (per bot: median best at every 8 h check-in, runs per check-in, what each report announced);
the default run plays the curriculum, the automation ladder and the three layers on a real-time session schedule
(IDLE: a 20-min first session, 3 check-ins a day; PICKED 45 min, 4; TUNED 90 min, 6) over the measured climb, and
writes `v2_projection.json` (per profile: events with times, per-day table, expeditions, cadence, checks). It is a
schedule, not a simulation: the climb is measured; ★, the head start, eras and extra heroes are the §2 knobs.

| Check (bar) | IDLE | PICKED | TUNED | Today (cp2, measured) |
|---|---|---|---|---|
| New things in the first 10 min (≥ 2) · first hour (≥ 6) · longest gap (≤ 15 min) | 5 · 8 · 5 | 5 · 13 · 15 | 5 · 15 · 15 | — (no in-session data) |
| Systems open end of day 1 (≤ 10) · day 3 · 7 · 14 · 30 | 6 · 11 · 18 · 23 · 29 | 7 · 13 · 21 · 24 · 29 | 9 · 13 · 21 · 24 · 29 | ~20 by day 2 (rater 21.5) |
| Most beats in one report after day 1 (≤ 5; prestige days excepted) | 7 | 6 | 5 | 12–14 |
| Days with something new, of 30 (≥ 26) · longest gap (≤ 2 d) | 27 · 1 | 30 · 0 | 28 · 1 | ≤ 5 items/report from day 7 |
| Days a system/rung/prestige opened: of 7 (≥ 6) · of 30 (≥ 16) | 6 · 16 | 7 · 20 | 7 · 23 | days 1–3 only |
| First expedition · second hero · the pen · first D34 = era 1 | day 2 · 4 · 4 · 23 | 2 · 4 · 4 · 19 | 2 · 4 · 4 · 17 | ascension ~day 11 |
| Expeditions by day 30 · eras by day 90 · automation rungs by day 7 / 30 | 4 · 9 · 7 / 10 | 8 · 11 · 9 / 11 | 9 · 11 · 9 / 12 | — |
| 90-day sketch: days with something new · with a system · longest gap (≤ 7 d) | 80 · 45 · 1 | 83 · 62 · 1 | 81 · 64 · 1 | — |

Calibration: IDLE 8 seeds × 14 d; PICKED 8 seeds to day 11 (4 on day 14); TUNED 8 to day 7, 2–3 to day 12 (its wall
search is slow; cut there). Climb: IDLE D9.5·13.5·18, D23 day 9; PICKED D13·16·18.5, D28 day 11; TUNED D28 day 8.

**Findings.** (1) Every bar passes for all three profiles, but only with two things the first draft lacked: the
§4 fallback ages (without them IDLE opened a system on 13–15 of 30 days — its weeks 2–4 waited on expeditions 3–5,
which it takes on days 10, 16, 28) and a rung queue separate from the system queue (with one shared slot IDLE's
rungs waited to days 6–9). (2) The expedition is what keeps weeks 2–4 alive; the Cut 30 curve alone thins to one floor
every 1–3 days after day 7. (3) The first offer must ignore the "stalled" rule (IDLE still gains a floor a check-in on
day 2; with the rule it waited to day 5), and later offers must wait until the old peak is back within 2 floors, or
expeditions go hollow (a D23 → D20 reset before that rule). (4) Engaged play buys less under prestige than on a
fresh climb: era 1 TUNED day 17, PICKED 19, IDLE 23 (1.35×) — the genre's catch-up; Cut 31 should gate ratios per
expedition, not per lifetime. (5) Era days are festivals (up to 9 beats); allowed. (6) Past day 60 the "new" is an
expedition, an era-record or a relic: months need content lanes (§6), not more gates.

## 6. Cuts

**Cut 30 now — hand to the running core agent** (small, all inside its current files; none needs the prestige loop):
1. `systems.rs`: a **min lineage age** per system (the §4 column) beside the trigger, and a **fallback age** for
   skill triggers; `LineageState.age` from the lineage's clock (offline counts).
2. **One system per report**: triggers met wait in a queue (`Lineage.reveal_queue`), shown as the track's next
   stage with its time (`next · tactics · tomorrow`); prestige later jumps it.
3. **The pen** at the Mother met *and* age ≥ 72 h (fallback age 5 d) — it opens on day 1–2 today.
4. **Tactics as a drip**: the first on the Warlord slain, then one per boss slain or per day (today all six in one
   report); the second stance on the Warlord met, the third (`Bold`) a day later.
5. **Report beats capped at 5** (stages first, then the rest as one line `+3 more`); today up to 14.
6. **Package level pace**: the equipped stance reaches L5 on day 4 (gate says day 7) — raise L4/L5 to ~150/400 runs.
7. **dayplayer rows** (reporting now, gated in Cut 31): systems open on day 1 (TUNED ≤ 10), max systems per report
   (1), max beats per report (≤ 5), days with something new (≥ 13/14), longest gap (≤ 1 d); keep the Cut 30 rows.
8. Reserve the save fields so Cut 31 needs no migration: `Lineage.{age_h, reveal_queue, renown, glory,
   expeditions, era_gate}` (defaults; unused).

**Cut 31 — re-scoped: the expedition, a second hero, the ladder** (replaces "houses + tavern → parallel
expeditions"; specialisation forks and weapon properties move to Cut 32):
- Core: the expedition (§2: offer, reset/keep, ★, glory, head start, the purse into the town), house 1 → a second
  hero lane (offline ×N lanes, one aggregated report: the worst death names the verdict), rungs 2–8 (§3), the
  watchtower/base camp/loot-properties rewards, the dayplayer on a 30-day horizon with `--expedition` policies.
- Client: the offer on the report (one tap, the gain in numbers), the town's stock growing on a reset, the
  automation rung on its building, the queue's next stage with its time.
- Gates: first expedition median ≤ day 3 for IDLE, PICKED, TUNED; the re-climb passes the old peak within 24 h on
  ≥ 80 % of expeditions; each profile's peak rises across its first 3 expeditions; IDLE reaches D34 by day 30,
  TUNED ≥ 1.3× sooner (projected 1.35×) and ≥ 1.5× faster per expedition climb; days with something new ≥ 26/30, longest gap ≤ 2 d, a system on ≥ 16/30 days (every profile);
  day-1 systems ≤ 10, one per report; no rung required (leave-one-out: removing a rung never slows below IDLE);
  the Cut 30 idle rows still pass; cohort on three absences: progression and pacing 0.8 from both raters.

**Cut 32** eras as layer 2 (the variants as systems, relics, advanced classes as packages, loot properties) ·
**Cut 33** dynasty, bloodlines, the company, the first content lane. From Cut 32 a content lane every ~2 weeks is
the real long-tail limit (Kongregate: "a rapid release cycle of like every couple days" [R2]).

## Sources

[A1] incrementaldb.com/community/interview/31 (Hevipelle) · [A2] antimatter-dimensions.fandom.com/wiki/Guide,
…/wiki/BubbaCow's_Antimatter_Dimensions_Guide · [A3] antimatter-dimensions.fandom.com/wiki/Autobuyers · [A4] Steam
review 229459473 (research/idle-attraction.md) · [T1] trimps.fandom.com/wiki/Auto_Fight, …/wiki/Auto_Upgrade,
…/wiki/Unlocks · [N1] sayolove.github.io/ngu-guide/en/quick-guide · [R1] Steam review 213486683 · [R2]
research/idle-attraction.md (Pecorella/Kongregate, r/incremental_games 1itf1er, "minutes, not hours") · [K1]
appgamer.com/afk-arena/strategy-guide/afk-rewards · [M1] wiki.melvoridle.com/w/Offline_Progression · [G1]
gameworldobserver.com/2020/12/22/gameanalytics-names-benchmark-engagement-metrics-idle-games · [S1]
thesovietgaming.com/soda-dungeon-2/scripts-guide · [S2] steamcommunity.com/app/946050/discussions/0/2638497042782491559
(a reset keeps town buildings) [U: forum] · [I1] idleon.wiki/wiki/Worlds · [I2] gist.github.com/shnaps/161a370ed795e6141e0553eb30ddc8fa ·
[L1] x.com/HowLongToBeat/status/1650658631916953602 · Cookie Clicker ascension: pocketgamer.com/cookie-clicker/ascension-guide.
