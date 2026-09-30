# Idle first — start simple, grow four tracks while away, rules last

*Assessment, 2026-09-28. No game code changed. The owner: "rules shouldn't be the main focus… idling is";
"rules and builds should help and be fun but shouldn't be fully necessary"; "rules a late-game unlock for
micro-tweaks; traits/arts as packages of rules"; progression as parallel tracks (character, items, scale,
town); "start simple, and add more simple, coherent, fun systems"; north star: **fun and a sense of
progression**. Inputs: PLAN, AGENTS, PATH_TO_95, PROGRESSION, PLATEAU, FUN_EVAL_IDLE, `eval/presets.json`,
`research/`, 50 blind cards, 23 raters' watch logs (`scratchpad/rater*/run*.json`), the last full gate
table (`scratchpad/gates-merge.log`), `meta.rs`, `facts.rs`, the web sources in §1.*

## 1. What "idling is the focus" means in the genre

| Game | A session | Offline | Automated · chosen | Depth |
|---|---|---|---|---|
| Melvor Idle | set a skill/combat action, leave; swap and bank on return | full rate, **24 h cap** [1] | the loop · action, gear, boss prep | 20+ parallel skills, masteries |
| NGU Idle | allocation burst per rebirth, then idle; rebirth 30 min → 24 h sets cadence [2] | runs closed, no drops | ticks · allocation, rebirth timing | rebirth → Evil → Sadistic; a tab per boss band |
| Idle Slayer | hero runs and fights; you jump for coins [3] | idle income ("punished for idling" = worst reviews) | combat · jumps, upgrades | Ascension → Ultra, zones |
| AFK Arena / Journey | claim the chest, level heroes, push a stage | chest **12 → 24 h** (Journey 16 h) [4] | fights · team, formation | heroes, gear, towers |
| Loop Hero | place tiles, equip, choose when to retreat | none ("you control the adventure, not the hero") [5] | the hero · his world | cards, camp buildings; exits keep 100/60/30 % — Riddle's |
| Cookie Clicker | buy, ascend | **earned**: 5 % → ~91 % [6] | production · purchases | one prestige, achievements |
| Antimatter Dim. | buy, prestige; later configure autobuyers, then scripts | full simulation [7] | autobuyers **earned** from challenges; the Automator (scripts) late | Infinity → Eternity → Reality [U] |
| Legends of Idleon | claim, spend, reassign ~11 characters ("bookkeeping at 11") [8] | **20 % AFK → ~85 %** via upgrades | each character's task · task, map | worlds W1–W7 add systems |
| Soda Dungeon 2 | send the party, come back | runs away | fights · party, gear, resets | **Soda Script** opens with the Arena after 1–3 resets; a read-only default script fires when none does [9] |

[1] wiki.melvoridle.com/w/Offline_Progression · [2] sayolove.github.io/ngu-guide/en/quick-guide · [3] idleslayer.fandom.com/wiki/Rage_Mode ·
[4] appgamer.com/afk-arena/strategy-guide/afk-rewards · [5] en.wikipedia.org/wiki/Loop_Hero, gamerant.com/loop-hero-interview ·
[6] cookieclicker.wiki.gg/wiki/Ascension_guide · [7] antimatter-dimensions.fandom.com/wiki/Autobuyers, …/The_Automator ·
[8] idleon.wiki/wiki/Game_Mechanics_AFK, leetdom.com/reviews/legends-of-idleon · [9] thesovietgaming.com/soda-dungeon-2/scripts-guide,
steamcommunity.com/app/946050/discussions/0/4325098189128485001 · FF12 gambits: 2 slots free, 10 earned (ff12sector.com). [U] unverified.

**What the genre agrees on** (with `research/idle-attraction.md` §2, §6):
1. **Doing nothing still progresses; engaging multiplies** (1.5–3×, Pecorella). Never gate idle below ~50 % of active.
2. **The return is a harvest and a spend** ("log in, spend all your cash, log out"); ~8-min sessions ~5×/day
   (GameAnalytics). The verbs are claim, buy, assign, push — not author.
3. **Scripting is a late, optional layer over a default that already plays** (Soda Script, AD's Automator, FF12).
4. **Automation is a reward**; **systems unfold one at a time** (the top-rated driver after offline gains).
5. **Walls yield to time**; smart play gets there sooner (Kongregate, *Math of Idle Games III*). Hard walls are opt-in.
6. **Parallel lanes** give every return several things to read. Failures: "a wallpaper simulator, no stakes"
   (Legends of Dragaea), coarse AI sliders "neither predictable nor expressive" (Dungeon Team), currency sprawl (LBR).

Riddle's research chose the opposite of 5 ("hard walls the AI cannot pass until its brain is rewritten").

## 2. Where Riddle is today (evidence)

**Not engaging does not progress.** The shipped fighter is two rows (`hp < 30% → drink heal`, `foes ≥ 1 →
attack nearest`), no exit. Last full gate table: DEFAULT dies on 30/30 seeds, best D7.5, **0 gold and 0 xp
over 8 h** — itself a gate row. ~20 gate rows guarantee non-engagement fails: DEFAULT ≤ D8, LEARNED ≤ +2,
PETS ≤ D8, LEVELLED ≤ D9, TRIVIAL, KITTED ≤ D8 (+4 variants), FULL−D23/28/33 held, DEFAULT fails den/lock/
captive/hunger, the lever row (forge < the best row edit) and its oath twin; Cut 30's draft adds a trait lever.
Even the patch-taking dayplayer stalls **13 days** (Cut 29 recorded 8); only writing a row (E1) broke it.

**Time.** 50 cards: median 55 active min, 14 own edits (0.24/min, the first after run 1), 14 runs watched,
16 offline. Watching is **40 %** of active time (23 raters with logs, 13–62 %); the other ~60 % is camp —
editing, forecast-reading, forge, oaths, unlocks. Post-absence check-ins 5–22 min. (Agent raters over-state
authoring.) **The absence** (AX, 8 h): 16 runs, 14 banks at D10, 0 deaths, +$2,703, ◆26; depth never passes the
set's own bank row; a 20-min absence returns 0–1 runs; purses reach $4–12k with nothing to buy.

**Valued** (every cohort): the death naming the row, the forecast re-pricing an edit, the overnight plateau
note, the Warlord arc, oaths. **Marked down**: pacing 0.6 ("admin heavier than the decisions"), expression
0.6 ("the facts pick the row"), the same floors every run.

**The scorer** puts 12 of 37 weight on the rule layer (decisions, expression, failure, attribution at 3).
Re-scored — idler · idle-rpg · idle-roguelike: Melvor 70.0 · 65.4 · **60.0**; Cookie 75.5 · 62.7 · **54.1**;
Loop Hero 72.0 · 73.8 · 74.1; Riddle AW 73.0 · 73.8 · 75.1. The preset pays for authorship; `idle-rpg` ("the
agent plays; the player configures") is the idle-first lens.

## 3. Today's progression, in plain words (cut29-wip)

- **Minute 0**: one fighter, two rules, send. He dies around floor 4–7.
- **First death**: you may change his rules; the death screen says which rule failed and suggests fixes with odds.
- **First gold home**: exits (stairs keep 100 %, walking home 60 %, death 30 %), a 3-item loadout, a 1-item vault.
- **Records** earn marks (◆) that buy more rule lines (4 → 10), actions (throw), vault space — six tiers that open as you meet each boss.
- **Meeting monsters** teaches words for rules and pre-written rule cards. **A stray** → taming; pets fight, level, die, leave eggs to breed.
- **Floor 5** forks into two biomes; **floor 8** is the Warlord, a wall until a rule answers him; bosses every 5 floors to the bottom (D34).
- **Forge**: gold buys weapon/armour steps and a bigger pack; waystones start runs deeper.
- **Classes**: rogue at the first bank, ranger at the D13 boss, caster at D18; class levels 1–10 add actions.
  **Heirs** inherit everything and carry a temperament. **Oaths**: self-set challenges paying party slots, a route, a trait pick.
- Ranks, trophies, the bestiary and the chronicle run throughout.

**How it feels, measured**: nearly all of it opens on **day 1** (a rater buys ~21 unlocks, the dayplayer 15);
nothing new after day 2–3; depth moves only when you rewrite rules; gold piles up. Only rows 5–8 and vault space
ever mattered — 8 of 11 cards, the 3 extra classes and every automation measured 0/18.

## 4. Options (judged by fun and felt progression)

**A — Rules become an occasional tactics layer (1–2 cuts).** A good default set ships and improves itself
(a counter met twice becomes a *drilled* row; the wall search's row is adopted after 2 stalled days unless
refused). Felt progression: depth moves while away, but only one thing grows — the rule list. Risk: a slower Riddle.

**B — An idle-first spine (4–6 cuts).** Parallel heirs, collections, prestige, automation bought with gold;
strength passes any wall with time; rules shrink to tuning. Felt progression: high. Axes: progression, return ↑↑;
decisions, failure, attribution ↓. Risk: discards what raters praise; Melvor-shaped games score ~60 on this preset.

**C — Hybrid: send, leave, return, spend (2 cuts).** The camp opens on the harvest (≤ 3 taps); walls soften to
speed bumps (§6); forecast and verdicts stay for whoever opens them. Return, pacing, progression ↑; expression
stays 0.6 without a middle layer.

**D — Packages first, the editor late (the owner's option; +2–3 cuts on C).** A **package** is a named
bundle of rows the engine already runs: a stance/art (`Guarded` = heal at 30 %, return at 20 %, retreat from a
telegraph), a temperament (`Reckless` = no exits, attack the strongest, +1 atk), a tactic (today's cards).
- *Composition*: slots stance · temperament · tactics 1 → 3; priority fixed by kind (own rows, late > drilled
  counters > stance > temperament > tactics > chores); each row has a role (exit, heal, target, move); within a
  role the higher slot wins and the loser shows greyed (`Reckless: no exit · Guarded wins`). Packages compile to
  a plain `RuleSet` with provenance, so sim, trace, replay hash and forecast are untouched.
- *Levels are the idle engine*: a package levels from runs it fired in, offline too (`Guarded` L3 adds
  `telegraph → retreat`, L5 banks at record − 1). Policy improves, yet every row is written by us and equipped by the player.
- *Feedback*: the forecast prices a swap or level on the same paired panel; a verdict names `package · row`;
  patches are *swap*, *level* or *equip* (rows once the pen opens); `forecastMove` gains a `package` part.
- *Traits (Cut 30)* become temperament packages: 1–2 rows the name promises + the generator's gift and cost.
  A modifier that "never acts" is invisible to an idle player; the prototype found only row-coupled gifts made
  builds anyway. *Class arts (Cut 31)* become stances. *Cards are already packages* (`meta::unlock_rows`).
- *Save migration*: an old set becomes one `custom` package with the pen open; card rows become tactics.
- Axes: decisions ↑ (a priced pick, like Loop Hero's cards), expression ↑ (3–5 packages have several optima),
  clarity and pacing ↑, attribution and failure hold (chosen, named). Risk: coarse presets — every package shows its rows.

## 5. Invariants re-framed: "engaging beats not engaging; not engaging still progresses"

| Today | Replace with |
|---|---|
| DEFAULT dies by D6/D8; yields 0 over 8 h | **IDLE** (never edits or picks, card 1 on every wake) meets the Warlord day 1, the Mother by day 4, D23 by day 12 (14-day dayplayer `--idle`); stall ≤ 4 d; gold every day |
| EDITED − DEFAULT ≥ 15 pts | **PICKED** (the forecast's top package swap each check-in) reaches each wall ≥ 1.5× sooner than IDLE; **TUNED** (+ rows) ≥ 1.5× sooner than PICKED from D13 |
| LEARNED ≤ +2; FULL−Dn never passes Dn | facts, drills, levels drive IDLE; IDLE never out-paces PICKED; an equipped counter passes Dn ≥ 3 days before the drill |
| RANDOM/PASSIVE lose every seed | RANDOM slower than IDLE to D13 on ≥ 80 %; PASSIVE retired (the stance can't be emptied) |
| Lever: forge < the best row edit | **No system required, each adds value**: leave-one-out on PICKED/TUNED — each system moves time-to-wall past ±; none > 60 % of the TUNED − IDLE gap |
| Every death names a rule; policy is written | every death names its cause and cheapest lever (spend · package · row · time); policy is chosen or written, never adopted silently; dice ≤ 5 % |

## 6. The walls that need a written rule today → speed bumps

| Wall | Needs today | Idle path |
|---|---|---|
| Warlord D8 (`shields up`) | `attack boss` | counter learned at the first meeting, drilled at the second; `Boss Focus` sooner |
| Mother D13 (`heals in gas`) | `throw fire` + a fire potion | drill + the quartermaster packs the counter's item |
| Lich D18 (`endless dead`) | `attack summoned` | drill, class levels |
| Foundry D21–23 (`reflects blows`) + the strength wall | `reflect read` | drill, scars, forge steps allowed to count |
| Lurker Queen D28 (`brood shields`) | `read silence`; the scroll is crowded out of a full pack (205 floors had one, 0 heroes held it) | the quartermaster reserves a slot for a known counter item |
| Mirror King D33 (`mirrors verbs`) | the `cadence` card | drill; the late pen pays off here |

**Scars**: each meeting leaves the boss −5 % hp for the lineage (cap −30 %, cleared when slain), deterministic
and shown (`Warlord · scarred ×3`). Time becomes progress with no HP inflation.

## 7. The proposed tracks (the owner's shape; start simple)

**Day 0 is almost nothing**: send, watch, come back, spend gold at the blacksmith. Every later system is
simple alone, readable in one look, feeds the others, and arrives as a stage on one of four tracks. Idle runs pay
into all four (gold, xp, facts, loot, records). Each track shows its **next stage and trigger**. ✓ exists in code
(maybe under another name) · ◐ partly · + new. Days are rough targets for a player who only idles and spends.

| Track | Stage (opens on · adds to idle · day) |
|---|---|
| **Character** | ✓ one warrior on the school stance (start · progresses alone · 0) → ◐ stances/arts from cards and masteries (Warlord met · a priced pick, levels offline · 0–1) → ✓ pets: tame, party (first stray · a second fighter · 1) → ✓ classes rogue/ranger/caster (first bank · Mother · Lich · new verb ladders · 1/3/6) → ◐ temperaments = traits as packages (heir 3 · a pick per heir · 2) → + advanced classes, Cut 31 forks (class L3 · a stance per fork · 4+) → ✓ **the pen**: rows, order, thresholds (Mother met or a 3-day stall · 1.5×+ on late walls · 4–6) |
| **Items** | ✓ pack of 3, auto-equip, restock (start · 0) → ✓ vault/keep (first find · 0) → ✓ blacksmith steps (first gold · a stronger hero, allowed to pass walls · 0) → ◐ salvage to materials (blacksmith · every run converts · 1) → + loot with properties: reach, cleave, quiet (D13 · builds to equip · 3) → + relics, a set per biome (first boss slain · small permanent gifts, a ledger · 4+) |
| **Scale** | ✓ one hero (0) → ✓ party slots 2–4 (tames, milestones · 1–3) → ✓ routes and waystones (Warlord slain · pick a lane · 1–2) → + a second hero on a parallel expedition (first house · **×2 runs per absence** · 3–4) → + an expedition board, 3–4 heroes, one aggregated report (houses 2–3 · ×3–4 · 6–10) → + a company: auto-send lanes that follow a record (tavern 2 · no taps per lane · 10+) → + ascension (the bottom · a new system each · 14+) |
| **Town** | ✓ campfire, vault, ledger, chronicle tiles (0) → ✓ blacksmith = the forge (first gold · 0) → ✓ kennel (first tame · pet slots, incubation · 1) → + **bank** (a purse ≥ one night's net · ~2 %/night interest capped at 3 nights' net: a sink that pays back, ends $12k purses · 1–2) → + houses 1–3 (banks at D13/18/23 · a hero slot each · 3+) → ◐ tavern (first house · hire heroes; rumours = today's bounties, one goal at a time · 4) → ◐ library/shrine (first counter · drills faster, package levels · 3) → ◐ watchtower (the first fork · forecast detail, the next band's foes · 5) → ◐ hall of heirs (heir 5 · bloodlines, titles · 6+) |

The tracks feed each other: town buildings open character/scale stages (houses → heroes, kennel → pets, library →
package levels); items make each hero stronger; scale multiplies idle output that pays for the town. Buildings are
policy-neutral: output, convenience or visibility, never a required answer. **Art (Codex)**: the camp vista (the
Warrens stair, `art/ui/backdrops`) becomes a town strip with building sprites in 3 stages each (9 × 3 = 27) and a
hero idle per house, in the ART.md register with primitive fallbacks; only `forge_imp` exists today.

## 8. Current systems: keep, simplify, defer, cut

| System | Call | One line |
|---|---|---|
| Send, watch (replay viewer, fold, callouts), the overnight report | keep | the core loop; the report leads with what grew on each track |
| Exits 100/60/30 %, heirs and lineage, facts/bestiary, taming/party, vault, chronicle/reel, boss arcs | keep | simple, fun alone, already coherent |
| Forecast | simplify | one headline (`reach D9 72 %`) and a price on a package swap; the panel, `vs` line and `forecastMove` parts go late |
| Death screen, verdicts, patches | simplify | cause + one cheapest lever (spend · package · wait); trace, `gap/dice/order`, replays and patch odds come with the pen |
| Tactic cards, masteries, traits, class arts | simplify | all become packages with one slot each |
| Forge, loadout, keep sheet, cage, insure | simplify | the blacksmith building + standing orders; no per-send taps |
| Marks catalogue (44 unlocks, tiers T0–T6) | simplify | records open track stages; no shop of rows and words |
| Oaths | simplify | a tavern rumour: one goal, one reward, no slots, draws, prices or commissions |
| Waystones, forks | simplify | a lane per hero on the scale track, via the watchtower |
| Rule editor (rows, conds, verbs, reorder), trace, divergence scene, plateau search | defer | the pen: the character track's last stage |
| Breeding tags, trait twists/blood/bloodline, oath slots, route 2, ascension | defer | late stages once the tracks are full |
| Meters (Cut 29) | defer | a report tab, then the pen's companion |
| Paid condition words, row purchases, gold automations as unlocks, commissions, oath draws | cut | chores or catalogue filler; buildings and the pen absorb them |

## 9. Recommendation

**C with D at its core, laid out as the four tracks, day 0 almost nothing.** Idle drives every track; packages
are the character track's mid game; the pen is its last stage. It best serves the north star: something grows on
every return on several tracks, and each next stage is visible. It keeps what raters praise (named deaths, the
priced choice, the plateau note, goals) and drops what they mark down (authoring workload, a single optimum,
admin). A alone is one track; B discards the half that works. Score cohorts on `idle-roguelike` (continuity) and
`idle-rpg` (the idle-first lens); re-weight only after re-running the calibration cards (FUN_EVAL §6).

**Progression feel** (what an IDLE player sees grow; a gate — every check-in grows ≥ 1 track, every day opens a stage):

| | Character | Items | Scale | Town |
|---|---|---|---|---|
| First hour | L1 → L3, first pet, Warlord met | first kept sword, blacksmith step 1 | 1 hero + 1 pet | camp → blacksmith |
| First day | first stance + a level, rogue, Warlord drilled | steps 2–3, vault 2 | party 2, a waystone | kennel, bank |
| First week | temperaments, ranger, caster, the pen at the Mother | loot properties, relics begin | 2–3 heroes, one aggregated report | houses 1–2, tavern, library; D18–23 |

**Cut 29 — finish, trim.** Keep the lazy wall search (a ship blocker; it becomes the drill source), the Queen pack
fix as quartermaster behaviour, mergeReports, the keep-sheet skip, standing orders, tap-is-a-decision, the systems
mechanism (its table re-ordered: packages before the pen). Stop: tuning the dayplayer bars against the
rule-writing model, editor-speed work, the telegraph patch rule (item 2), tier re-gating (item 5), the oath
slots/draws/commissions client, meters beyond the report.
**Cut 30 (traits) — pause its direction.** "A trait never acts; only rows choose" and its lever gates encode
rules-first. Keep its data shape, wake cards, reveal, facts, counterfactual; re-scope traits as temperament packages.

### Draft — Cut 30 · Idle first: the hero climbs on his own; four tracks; packages before the pen

*Contract draft. Evidence: DEFAULT yields 0 over 8 h (a gate); the dayplayer stalls 13 days; camp is ~60 % of
active time; everything opens on day 1 and nothing after day 3; pacing and expression 0.6 on every plateau card.*

#### 1. The idle floor (core)
- The school stance `Steady` (heal at 30 %, return at 20 %, bank at record); drills (a counter met twice
  enters the stance as a named, revocable row, announced once); scars; the quartermaster packs known counter
  items; a 20-min absence returns ≥ 1 run.
- Gate: the IDLE rows (§5); the replay hash; dice ≤ 5 %.

#### 2. Packages (core + client)
- `Package { kind: stance | temperament | tactic, rows by level, gift?, cost? }` → `RuleSet` with provenance;
  slots stance · temperament · tactics 1 → 3; role shadowing shown; levels from runs, offline included.
- 4 stances (with the class masteries), 8 temperaments (Cut 30's heads), 6 re-tuned tactics — few and simple.
- One-line forecast price on a swap; the verdict names `package · row`; the lever offered is swap/level/equip.
- Old saves: a `custom` package, the pen open.
- Gate: tests (compile order, provenance in the trace, shadowing); qa.rs (forecastMove parts sum); metrics —
  each package is best for ≥ 1 wall on ≥ 1 seed set, none best for all.

#### 3. Day 0 and the town, stage 1 (client + core)
- Day 0 shows send, the watch, the report, the blacksmith; nothing else.
- The camp becomes a town strip (campfire, vault, ledger, chronicle, blacksmith, kennel, bank); a tracks panel
  shows each track's stage, next stage and trigger; the pen opens at the Mother or a 3-day stall.
- Gate: ui.mjs (day 0 surfaces ≤ 4; each trigger opens its stage; the next stage always shown); a scripted IDLE
  check-in ≤ 3 taps.

#### 4. Measure (core)
- dayplayer `--idle`, `--picked`, `--tuned` over 14 days; stage opens per track per day; leave-one-out per system.

| Gate | Bar |
|---|---|
| IDLE: Warlord day 1, Mother ≤ day 4, D23 ≤ day 12, stall ≤ 4 d, gold every day | dayplayer `--idle` |
| PICKED ≥ 1.5× IDLE; TUNED ≥ 1.5× PICKED from D13 | dayplayer |
| Every IDLE check-in grows ≥ 1 track; ≥ 1 stage opens per day for 14 days | dayplayer |
| No system required; each adds value; none > 60 % of the gap; bank ≤ 10 % of daily income | metrics |
| Cohort 25 (three absences): return, pacing, progression 0.8 from both; ≥ 74 on both presets | two blind cards |

**Next**: Cut 31 houses + tavern → parallel expeditions and the aggregated report (one verdict per absence, the
worst death; offline ticks ×N ≈ 0.4 s per lane per 8 h). Cut 32 advanced classes and loot properties as packages.
Cut 33 relics, library, watchtower.

**Deviations to record.** The ~20 retired gate rows are *replaced*, not weakened. AGENTS.md "Hard invariants" and
PLAN.md's anti-pillar "no calm mode… the policy is optional" change only with the owner's sign-off.
