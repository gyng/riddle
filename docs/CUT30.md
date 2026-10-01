# Cut 30 — Idle first: the hero climbs on his own

*Contract, 2026-09-30. The idle-first pivot's first cut (owner decisions, `docs/HANDOFF.md` §0). Design:
`docs/IDLE_FIRST.md` (recommendation C + D, §5 invariants, §6 walls, §7 tracks, §8 keep/simplify/defer/cut),
`docs/TOWN.md` (the hub; targets `art/ui/targets/town/`), the paused trait core (`docs/CUT30-traits-superseded.md`
status, `crates/riddle-core/src/traits.rs`). Evidence: DEFAULT yields 0 gold and 0 xp over 8 h (a gate row); the
dayplayer stalls 8–13 days without writing a row; camp is ~60 % of active time; everything opens on day 1 and
nothing after day 3; pacing and expression 0.6 on every plateau card; oaths unclear to the owner after a 9/9 agent
probe. North star: **fun and a sense of progression**. Start simple.*

**Precondition.** Cut 29 is finished as written and merged (the owner's order). Its gates stay as written until this
cut's merge; the rows this cut retires (§6) retire with it, not before.

## Design

### 1. The idle floor — DEFAULT progresses on its own (core; progression, return, pacing)

- **The school stance `Steady`** replaces the two-row fighter preset as every new lineage's set: `hp < 30% → drink
  heal`, `hp < 20% → return`, `depth ≥ best → bank`, `foes ≥ 1 → attack nearest` (the rows it compiles to, §2). It
  banks or returns on its own, so an unattended night pays gold and xp.
- **Drills.** A band boss's counter the lineage knows (the fact, e.g. `goblin_warlord:shields`) enters the set as a
  named row at the boss's **second meeting** (`drill · attack boss`), announced once (`DRILLED`), shown on the stance,
  revocable with one tap. Drills are the idle path through every wall in IDLE_FIRST §6 (Warlord, Mother, Lich,
  Foundry `reflect read`, Queen `read silence`, Mirror King `cadence`); the Cut 29 wall search (`wall.rs`) supplies a
  wall's drill when no fact-counter exists.
- **Scars.** Each meeting leaves the boss −5 % max hp for the lineage (cap −30 %, cleared when slain); deterministic,
  shown on the boss plate and the shaft (`Warlord · scarred ×3`). Time becomes progress without HP inflation.
- **The quartermaster packs known counters.** When a drill needs an item (`throw fire` at the Mother, the silence
  scroll at the Queen), the restock reserves a pack slot for it before any other supply.
- **Short absences pay.** A 20-min absence returns ≥ 1 run (today 0–1).
- Gate: dayplayer `--idle` rows (table); tests (drill at the second meeting, revocation persists, scars capped and
  cleared, the reserved slot, the replay hash with drills and scars); dice ≤ 5 %.

### 2. Packages v1 — pre-written rule bundles that level (core + client; decisions, expression, clarity)

- **Shape.** `Package { id, kind: stance | tactic | temperament, rows_by_level: [[Row]; 5], gift?, cost? }`.
  Equipped packages **compile to a plain `RuleSet` with provenance** (`provenance.rs`), so the sim, trace, replay
  hash and forecast are untouched. Compile order is fixed by kind: pen rows (once open) › drills › stance guard rows
  (heal, exit) › tactic › temperament › stance fallback (`attack nearest`) › chores. Each row has a role (exit, heal,
  target, move); a row a same-role row above it always pre-empts is shown greyed with the winner (`Bold: no exit ·
  Guarded wins`).
- **Slots v1**: stance ×1 (never empty), tactic ×1 (a second at a stage, §4), temperament ×1 (from heir 3).
- **The set v1 (few, simple, each ≤ 6 rows at L5)**:
  - Stances ×4 — `Steady` (the school), `Guarded` (heal 40 %, return 30 %, retreat from a telegraph), `Bold` (heal
    25 %, no return, bank at record + 2, attack lowest), `Hunter` (bank at record, attack boss/summoned first). Each
    arrives on a stage trigger (§4), free; equipping is free and instant in camp.
  - Tactics ×6 — today's cards re-tuned as packages (`meta::unlock_rows` already is one): `boss focus`, `corridor
    fighting`, `kite archers`, `thief guard`, `gas step`, `pack break`. Owned cards convert.
  - Temperaments ×4 — the trait engine's four mapped shapes (`traits::upgrade`: cowardly → `skittish`, brave →
    `unbowed`, greedy → `light hands`, curious → `iron gut`), each **1–2 rows the name promises + its gift and cost**.
    The old per-floor overrides in `turn::choose_and_act` are removed: a temperament acts only through its rows.
- **Levels are the idle engine.** A package gains a run for every run one of its rows fired (offline included); L2 · L3
  · L4 · L5 at 5 · 20 · 60 · 150 runs (tunable; the gate is the felt pace). Each level adds or tunes a row, written
  by us (`Steady` L3 `telegraph → retreat`, L5 banks at record − 1 when hurt). A level is a report line and a beat
  (`STEADY L3`).
- **Reused from the trait core** (built, inert today): the parts and lexicon name and describe temperaments (chip
  `skittish · slow`, formula `[hurt] → quick +3 · slow`, stamp, `?` until learned); **fact learning** (a gift's size
  becomes a fact on its first live turn, `trait:<head>`); **inheritance and the wake's three cards** (heir 3: three
  temperament cards with distinct `when`s, a send without a pick takes card 1); **ladder arrival** (systems
  `traits` → `temperament`, heir 3 or a death past D5); **neutral bots** (`traits::neutral` for the §6 bots that
  measure without one); **old-save mapping** (`traits::upgrade`). The generator's random draw stays off (the table
  ships the 4 mapped shapes only); blood, fade, twists and bloodline wait (NOT in Cut 30).
- **Feedback, simplified.** The forecast shows one headline (`reach D9 72 %`) and prices a swap or level on the same
  paired panel (`Guarded · death −8`); `forecastMove` gains a `package` part. The verdict names `package · row`
  (`Steady · return at 20%`). Before the pen opens, the death screen shows the cause and **one cheapest lever**
  (spend · package · wait); trace, `gap/dice`, replays and patch odds come with the pen.
- **The pen is a late stage.** The rule editor (rows, conds, verbs, reorder) opens at the Mother met or a 3-day
  stall; until then no rule tablets on the home screen. Opened, the player's rows sit above the packages. The editor
  itself is unchanged. Old saves: the set becomes one `custom` package, the pen open; card rows become tactics.
- Gate: tests (compile order, provenance in the trace, shadowing, levels from offline runs, save mapping, no
  `acting_row == -1` from a temperament over 30 seeds × 8 h); qa.rs (`forecastMove` parts incl. `package` sum to
  the whole ± noise; the verdict names a package row when one acted); metrics (table).

### 3. The town hub v1 (client + core + art; return, progression, aesthetic)

- **Day 0 is a camp** (`phone_day0.png`): campfire, one tent (= the hero: portrait, class, look), a supply crate
  (= the pack), the **dungeon mouth** (a cave, two torches: tap = send) and the lone hero; the SEND gem stays in the
  console. Nothing else is interactive.
- **Buildings v1**, each appearing on the scene when built (scaffold → built, dust and a glint, no sentence):
  **blacksmith** (first gold home: the forge), **storehouse** (first find kept: the vault), **kennel** (first tame:
  the party, cage), **bank** (a purse ≥ one night's net: deposits earn ~2 %/night, capped at 3 nights' net). One
  look per building in v1, stepped to look 2–3 by its level (forge steps, bank cap) where the art exists.
- The **next building** shows as a staked plot with a small marker (its trigger on tap); unbuilt plots are grass.
- **What moves**: at SEND the hero walks from the tent to the mouth and goes in (then the watch opens; no camera dive
  in v1). On return the absence's runs walk out of the mouth as parties (one per run, ≤ 3 on screen, ~20 s total, a
  sack sized by its gold, a glowing chest for a find, a pet behind for a tame; a death walks nobody out). A hero at
  home loops tent → forge → fire; pets follow or lie in the kennel; smoke and forge sparks (fx); day/night from the
  local clock with the light field. Markers ≤ 3 (coin over the bank, `!` rune, sword over the forge). Walkers are
  cosmetic, seeded by lineage id + day, never read back.
- **Tech**: a `town` scene in the existing three.js pixel renderer (`render/town.ts`: tile map, named plots, a path
  graph; `townState(lineage, absence)` → buildings, walkers, markers), DOM hit targets over each building (≥ 44 px, a
  hidden label), the Canvas-2D fallback (`view2d.ts`) drawing the same scene. The console's command card becomes
  the **building bar** (one tile per built building, same panel, shared badges). Panels stay UI.md sheets anchored
  above their building. Existing tiles with no building yet (chronicle, ledger, packages, quests) stay on the bar.
- **Core**: `Lineage.town` (buildings, levels, the next plot and its trigger), the bank (deposit, interest, cap),
  heroes' home/away. Nothing else.
- Gate: ui.mjs / `cut30.mjs` (day 0 ≤ 4 interactive surfaces: mouth, crate, tent, gem; each trigger builds its
  building; the next plot always shown until the v1 set is built; mouth tap = send; ≤ 12 elements above the fold
  at 400 × 800 at every stage); GPU harness 60 fps with 24 walkers (stress), ≤ 30 fps after 10 s without input, 0
  when hidden; ctxloss.mjs draws the town; bank tests (interest capped, never negative, deterministic).

### 4. The tracks panel (client + core; progression, clarity)

- One panel, four rows — **character · items · scale · town** — each: the current stage (icon + ≤ 2 words), a
  progress bar when the next trigger is numeric, and **next: stage · trigger** (`next · kennel · first tame`). Opened
  from the portrait-mini in the top bar; the report leads with what grew on each track.
- **Stages v1** (the ones that exist after this cut; IDLE_FIRST §7 has the rest):
  - character: warrior on `Steady` → a second stance (Warlord met) → a tactic (Warlord slain) → pets (first stray) →
    rogue/ranger/caster (first bank · Mother · Lich) → a temperament (heir 3) → a second tactic slot (Lich met) →
    **the pen** (Mother met or a 3-day stall).
  - items: pack of 3 → storehouse → blacksmith steps → a counter packed (first drill with an item).
  - scale: one hero → party slot 2 → waystones (Warlord slain) → party slots 3–4.
  - town: camp → blacksmith → storehouse → kennel → bank.
- **Core**: `Lineage.tracks` (per track: stage, next, trigger, progress) and `ReturnReport.grew` (what grew per
  track: xp, a level, a package level, gold, a building, a best), both derived from state (`systems.rs` gains the
  stage table; its order puts packages before the pen).
- Gate: ui.mjs (four tracks, a next stage on every track until its v1 stages are done); dayplayer (every IDLE
  check-in grows ≥ 1 track; stage days, table).

### 5. Oaths → a simple quest board (core + client; goals, clarity) — or deferred

- **One quest at a time**, on a notice board by the mouth (a prop; a tile on the building bar). It shows **one plain
  goal line** (`reach D10 · no return`, ≤ 5 words), **the reward as a picture** (the parked vignettes in
  `art/ui/oath/`: card, heir, route, row, slot, title, verb, waystone), and **a progress bar**. Kept → the reward, a
  `QUEST DONE` beat and a report line; a new quest the next day (one free swap a day).
- **No stake**: no price, no forswear, no slots, draws or commissions (Cut 28/29 mechanics removed from play; a sworn
  oath's stake is refunded in full on load, kept rewards stay).
- Arrives late on the ladder: the Warlord slain. Goals come from the oath pool made plain; a constraint goal wants a
  different package (`no return` → `Bold`), so a quest is a reason to swap, never a requirement.
- **Deferred instead** if the owner's check (§8) fails twice: the board is hidden, the system `quests` stays closed,
  and Cut 31 decides whether it lives in the tavern (rumours) or goes.
- Gate: tests (progress, one-a-day swap, refund on load); metrics (each quest completable ≥ 20 %/night by some package
  set with the pen closed); the owner's check (§8).

### 6. The replacement invariants and gates (core + docs)

The idle gates **replace** the "not engaging fails" rows (the owner's decision, HANDOFF §0) — replaced, not weakened.
**New bots** (dayplayer, 14 days, 3 check-ins a day, 8 seeds; `--idle`, `--picked`, `--tuned`):
- **IDLE** (replaces DEFAULT): sends at each check-in, never edits, picks, buys or deposits; the wake takes card 1.
- **PICKED**: IDLE + at each check-in the forecast's top package swap or level spend, a blacksmith step when
  affordable, a bank deposit.
- **TUNED** (replaces EDITED): PICKED + the pen (today's dayplayer rule-writer: `write_own_rows`, the wall patch).
- **RANDOM**: random package picks and random rows. PASSIVE and LEARNED retire (the stance cannot be emptied; facts
  now drive IDLE by design). FULL, COUNTERED stay as content-reach probes.

Time-to-milestone = simulated hours to reach D8/D13/D18/D23/D28; a ratio is the median over seeds of slower ÷
faster; a milestone the faster bot reaches and the slower never does within 14 days counts as passed.

**Retired rows** (metrics.rs / dayplayer / progression): DEFAULT dies by ≤ D8; EDITED − DEFAULT ≥ 15 pts;
LEARNED ≤ DEFAULT + 2; PETS ≤ D8; LEVELLED ≤ D9; TRIVIAL never passes D8; KITTED ≤ D8 and its five `Kitted:` twins;
`{set} never passes D{boss}` (FULL−D23/28/33) and its kitted twin; `DEFAULT passes the {den/lock/captive/hunger} ≤
20 %`; DEFAULT yields 0 xp/gold over 8 h; the three `Routes:` DEFAULT/EDITED/LEARNED rows and PASSIVE/RANDOM twins;
PASSIVE loses; whole forge < best row (the lever) and its oath twin; DEFAULT/EDITED never swear; oath best set ≠
bank-optimal; the dayplayer's unlock-days and stall ≤ 3 bars; the progression rows (unlock days, stall ≤ 3 on
13/18, marks unspent ≤ 8, purse ≤ 1.5 days' net). **Kept unchanged**: replay hash, dice ≤ 5 %, deaths with a
cause, stalls/dances/loops/thefts ≤ 1 %, lanes, divergence, verdict time, per-tick cost, events/min, situations,
story and reel rows, forecast rows, the Cut 29 meters and qa legs.

**The exact doc edits** (made in the merge commit of this cut, after its gates pass — not before):

AGENTS.md
- l. 3: `docs/CUT29.md` → `docs/CUT30.md`.
- "What the game is", first paragraph → *A real roguelike whose hero you never drive. The hero climbs on his own
  while you are away; you equip packages (stances, tactics, temperaments: pre-written rule bundles that level from
  runs), build a town, and late on open the pen to write your own rows. The engine does chores silently. Runs happen
  offline, uncapped. Every death names its cause. The dungeon has a bottom.* The loop diagram's first line → `town
  (packages, buildings, tracks, forecast; the pen late) → send → run (rows fire, facts learned, drills)`.
- "How work gets done" 4: `idle-roguelike` → `idle-hybrid`.
- "Hard invariants", the bots and policy bullets → *Idle alone progresses: IDLE (no picks, no edits) reaches D8 on
  day 1 and D23 by day 12, gold every day, stall ≤ 4 d. Engaging multiplies: PICKED (packages) reaches each wall ≥
  1.5× sooner than IDLE; TUNED (the pen) ≥ 1.5× sooner than PICKED from D13. Nothing is required: removing any one
  system never drops a bot below IDLE; each adds value; none is > 60 % of the TUNED − IDLE gap. RANDOM is slower
  than IDLE to D13.* · *Every death has a cause, a trace and a cheapest lever; `dice` deaths ≤ 5 %.* · *Facts are
  learned; policy is chosen, drilled or written — never silent: every drilled row is named, announced once, shown
  and revocable.*

PLAN.md
- Status line: add *2026-09-30: the idle-first pivot (docs/IDLE_FIRST.md, docs/CUT30.md) supersedes the rules-first
  spine; sections below marked (pre-pivot) are history.*
- Store contract: *you write the hero's rules; you never steer* → *you equip and tune the hero; you never steer.*
- l. 39 Riddle column: *Rule authoring **is** the spine* → *The idle climb is the spine; packages and the pen
  multiply it.*
- Pillar 1 → *Every death names its cause* — *and its cheapest lever (spend · package · row · wait).*
- Anti-pillars l. 65 → *No "the policy is required": idling progresses; choosing and writing multiply it.* l. 68 →
  *No silent policy: drilled rows are named, shown, revocable; facts are learned; policy is chosen or written.*
- l. 88 → *Wall bosses every five floors are speed bumps: the counter is a fact the hero learns and drills at the
  second meeting; scars wear the boss down; a package or a row gets there sooner.*
- l. 209 (PETS gate), l. 243 (level-10 gate) → *(retired in Cut 30; see the idle gates.)*
- l. 227 → *Active bonus through decisions only: packages ≥ 1.5× over idle, the pen ≥ 1.5× again.*
- Bots table l. 292–300 → the IDLE/PICKED/TUNED/RANDOM table of this section.
- Risks l. 348 → *"Plays itself" backlash → packages and the pen as visible multipliers; every death's lever.*

### 7. Scoring on `idle-hybrid`, a control cohort first (eval)

- `eval/SCORECARD.template.json` `"genre": "idle-hybrid"`; `eval/RATER_PROMPT.md` names the preset and adds the idle
  probe *did it progress while I ignored it?*; `killCriteria.defaultRulesBeatEveryEdit` stays (read as the default
  stance beating every pick); `returnWithNothingToDecide` reads *nothing grew and nothing to pick*.
- **Control first** (can run now, before any Cut 30 code): the pre-pivot build (Cut 29's merge, else `cut29-wip`
  head) under `idle-hybrid`, two blind raters, three absences (20 m / 4 h / 8 h), seeds `N01/N02`; a PLATEAU entry
  `control-hybrid`. Cohort 24 re-scored on paper is 72.85; the control is what Cut 30 is judged against.
- Cohort 25 on Cut 30, same protocol, after the owner's check (§8).

### 8. The owner validates each new system (process)

- For each new system — the idle climb, packages, the town, the tracks panel, the quest board — the owner gets the
  `tools/ship.sh --preview` URL, one headed screenshot per screen and nothing else (no explanation). The owner
  answers three questions per system: *what does it do?* (own words), *clear?* (y/n), *want more of it?* (y/n).
- Pass: the owner's words match the design without being told, and *clear* is yes. Agent readers (the blind copy
  protocol, `scratchpad/copy/`) run first as a filter only; an agent pass is not a pass.
- A system that fails is simplified and re-checked once; failing twice, it is deferred to Cut 31 (the quest board
  first in line). Answers are recorded below under **Owner check**.

## Reveal (day 0 → week 1; `systems.rs` re-ordered)

Day 0: the camp (mouth, crate, tent, gem), the watch, the report. First gold home: blacksmith, exits. First find:
storehouse. First stray/tame: party, kennel. Warlord met: a second stance, the stance panel. Warlord slain:
tactics, waystones, the quest board. A purse ≥ one night's net: the bank. Heir 3: temperaments (wake cards). Lich
met: tactic slot 2. Mother met or a 3-day stall: **the pen** (editor, trace, verdict detail, reorder, divergence,
the marks catalogue as it stands). Each arrival glints once; no tutorial text.

## Copy

Package chip `<name> L<n>` (`Guarded L3`); level beat `GUARDED L3`; `DRILLED`; `scarred ×3`; shadowed row `<winner>
wins`; quest goal ≤ 5 words (a new class in `eval/copy-budgets.json`), `QUEST DONE`; tracks `next · <stage> ·
<trigger>`. Glossary rows (COPY.md §2): **package**, **stance**, **tactic**, **temperament** (a package; the old
`trait` word maps to it), **drill**, **scar**, **quest** (not `oath`), **track**, **the pen**. Callouts ≤ 3 words,
labels 2, no sentences in chrome.

## Gates

| Gate | Bar | Where |
|---|---|---|
| IDLE reaches D8 by the end of day 1 | 8/8 seeds | dayplayer `--idle` |
| IDLE reaches D13 by day 4; D23 by day 12 | median ≤ day 4; ≥ 6/8 seeds and median ≤ day 12 | dayplayer `--idle` |
| IDLE longest best-depth stall before D23 | ≤ 4 d, every seed | dayplayer `--idle` |
| IDLE net gold > 0 every day | 14/14 days, every seed | dayplayer `--idle` |
| IDLE does not slay the Mirror King (D33) within 14 days | 0/8 seeds | dayplayer `--idle` |
| A 20-min absence returns ≥ 1 run (fresh and D13 lineages) | ≥ 95 % of 30 seeds | metrics |
| A drill-needed item is in the pack at its wall | ≥ 90 % of sends | metrics |
| PICKED ≥ 1.5× IDLE at D13, D18, D23 | each ratio ≥ 1.5 | dayplayer |
| TUNED ≥ 1.5× PICKED at D18, D23, D28 | each ratio ≥ 1.5 | dayplayer |
| IDLE never out-paces PICKED | PICKED ≤ IDLE hours on ≥ 90 % of seed × milestone pairs | dayplayer |
| RANDOM slower than IDLE to D13 | ≥ 80 % of seeds | dayplayer |
| Nothing required: TUNED − S for S ∈ {packages, pen, forge, pets, bank, quests} | never slower than IDLE at any milestone (± one check-in) | dayplayer leave-one-out |
| Each system adds value; none dominates | each S moves TUNED's D23 hours past its ±; none > 60 % of TUNED − IDLE | dayplayer leave-one-out |
| Every stance is best at ≥ 1 wall on ≥ 1 seed set; none best at all | 4/4 stances; 0 dominate | metrics |
| Packages level felt | the equipped stance L3 by day 2, L5 by day 7 (IDLE, median) | dayplayer |
| Every IDLE check-in grows ≥ 1 track | 100 % of check-ins | dayplayer |
| Days with a stage opened | IDLE ≥ 8/14, PICKED ≥ 10/14 | dayplayer |
| Bank interest | ≤ 10 % of daily income, never negative | metrics, tests |
| Quests completable with the pen closed | each ≥ 20 %/night by some package set | metrics |
| Compile order, provenance, shadowing, levels, drills, scars, save mapping, no temperament-chosen action | pass | tests |
| `forecastMove` parts sum (incl. `package`); verdict names `package · row` when one acted | pass | qa.rs |
| Replay hash, dice ≤ 5 %, stalls/dances/loops, lanes, divergence, per-tick ≤ 6 µs, the kept rows of §6 | pass | `node tools/gates.mjs --full` |
| Day 0 ≤ 4 surfaces; each trigger builds/opens its stage; next stage shown; ≤ 12 above the fold | pass | ui.mjs, `cut30.mjs` |
| Town 60 fps with 24 walkers (GPU); ≤ 30 fps idle after 10 s; 0 hidden; ctxloss draws the town | pass | GPU harness, ctxloss.mjs |
| A scripted IDLE check-in (harvest, one spend, send) | ≤ 3 taps | ui.mjs |
| Owner check: idle climb, packages, town, tracks, quest board | each *clear* = yes in the owner's words (quest board: else deferred) | §8 |
| Control cohort on the pre-pivot build under `idle-hybrid` | run and recorded before cohort 25 | two blind cards |
| Cohort 25 (three absences, `idle-hybrid`) | mean ≥ control + 4 and ≥ 74; progression, pacing, return 0.8 from both; α ≥ 0.80 | two blind cards |

## Split

| Area | Owns | Meets at |
|---|---|---|
| **Core** (crates/) | Steady preset; drills, scars, quartermaster reservation, short-absence run; `Package` data + compile + levels + provenance; temperaments from `traits.rs` (overrides out of `turn.rs`); save mapping; `Lineage.{packages, town, tracks, quest}`, bank; `ReturnReport.grew`; `systems.rs` stage table; the bots and rows in `dayplayer.rs`/`metrics.rs`/`qa.rs`; retiring the §6 rows | wire fields first in `web/src/engine/{types,fake}.ts` |
| **Client** (web/src/) | `render/town.ts` scene + DOM targets + building bar; packages panel (slots, levels, shadowing, one-line price); tracks panel; quest board; simplified death screen and forecast headline before the pen; the pen gated late; report leads with tracks; `cut30.mjs` | builds against `fake.ts` stand-ins |
| **Art** (Codex, ART.md registers, primitive fallback each) | town terrain ramp (grass ×4, path ×3 + edges, plaza, fence); mouth (cave), campfire (2 frames), tent, crate, staked plot, scaffold; blacksmith, storehouse, kennel, bank (look 1; blacksmith and bank looks 2–3); walk frame per class (4), sack small/large, glowing chest; notice board; package icons (4 stances, 4 temperaments; tactics reuse card icons) | `art/manifest.json`, `pack.py`, `art-qc.py` |
| **Eval** | scorecard/rater prompt to `idle-hybrid`; the control cohort (now); owner check; QA → cohort 25 | `eval/`, `docs/PLATEAU.md` |

Order: control cohort ∥ core baseline (`dayplayer --idle` on today's build, recorded) → core §1 + bots → §2 → wire →
client §3–5 ∥ art → gates → the AGENTS.md/PLAN.md edits → owner check → QA → cohort 25. **Hold before Cut 31**:
update HANDOFF with the cohort and owner answers, then stop for the owner.

## NOT in Cut 30

- Parallel heroes: houses, the tavern, the expedition board, the aggregated report, auto-send lanes (Cut 31).
- Advanced classes, specialisation forks, weapon/armour properties, loot with properties, relics (re-scope under the
  pivot).
- The library, watchtower and hall of heirs buildings; townsfolk; weather; the camera dive through the mouth; the
  desktop minimap and roster plaques (the desktop shows the same scene in the existing frame).
- The trait generator's random draws, blood slot, fade, twists, marked traits, bloodlines; the measured trait table.
- A marks-economy redesign (the catalogue stays as it is behind the pen); ascension changes.
- Editor changes of any kind; meters beyond the report; the gfx-eval bar (8.0) and audio scoring.

## Owner check

*(To fill: date, build, per system the owner's three answers, pass/simplify/defer.)*

## Status (core half, branch `cut30`)

**Checkpoint 1 — the idle floor and packages v1 in the core** (`crates/riddle-core/src/packages.rs`, `town.rs`,
`tests_cut30_pkg.rs`; wire in `web/src/engine/{types,fake,wasm,proxy}.ts`):
- Every new lineage (`Game::new`) climbs on the compiled `Steady` stance; harnesses that write their own sets use
  `Game::new_literal` (the pre-Cut 30 behaviour: `PkgState.literal`). Package rows carry their origin (`stance:steady`,
  `drill:lich`, `tactic:boss_focus`, `temper:skittish`) and sit outside the row cap (`Row::is_pkg`); `ROWS_TOTAL` grew by
  `MAX_PKG_ROWS` 24. The pen's rows (`set_rules` on a lineage on packages) sit above every package and compile only once
  the pen is open (the Mother met or a 3-day stall).
- Drills at a band boss's second meeting (a meeting = a day of the lineage's clock that saw him), the counter fact's row
  with `hp > heal` (a drill above the guard rows never stops the hero drinking); the Foundry's wall drill comes from its
  golems (`reflect_melee` known, D19+ reached) — cheap, deterministic, no wall search. Scars −5 %/meeting, cap −30 %, gone
  once slain (`Run.scars`, applied at the boss's spawn). The quartermaster packs a drill's item first and keeps the
  stance's heal in the free slots (the idle floor's pack of 3).
- Packages v1: 4 stances, the 6 cards as tactics, 4 temperaments (the old overrides are gone from `turn::choose_and_act`:
  a temperament acts only through its rows; the heir wears the mapped shape's gift and cost). Levels from runs (offline
  included) at 10 · 40 · 120 · 300; a mark spend buys the next level (`spend_level`: ◆ = the level's number). The camp's
  price (`packageOptions`): each move on the paired panel (past / reach / bank / death).
- Town core: buildings on their triggers, the bank (2 %/night, capped at 3 nights' net), the four tracks, `ReturnReport.grew`
  and `.packages` (the beats); the quest board (one goal ≤ 5 words, a reward picture, progress, one free swap a day, no stake).
- The forge's ladders grew (weapon +4 … +6 are damage, armour to plate +1) — the "not engaging fails" caps that held them
  are the rows this cut retires; the blacksmith is the multiplier the idle floor lacks.
- Save migration: an old save's set becomes the `custom` stance (as written; the pen open, the editor edits it in place);
  `saves_from_307dbed_send_identically` re-recorded (`3754cfc8…` → `9baed175…`: the temperaments no longer act, scars,
  drills).

**Checkpoint 2** (`cabf89d`): the Reveal's curriculum in `systems.rs`, drills at the second *run* that meets a boss
(scars by day), the tracks' stages, the IDLE/PICKED/TUNED/RANDOM dayplayer with leave-one-outs, the metrics' Cut 30 rows
(`examples/idle_lib`: 20-min absence, drill item packed, stances best at a wall, quests keepable) and the §6 retired rows
printed ungated (`metrics.rs` `RETIRED`, the contract's list plus the progression rows).

**Checkpoint 3** (`09e145d`): `forecastMove` gains a `package` part (the parts still sum to the whole); `Death.package`
names `package · row`; `Death.lever` (spend · package · wait) before the pen; the lineage key carries the scars; the wall
search edits the pen alone on a lineage on packages; qa legs (`check_packages`: all PASS on 6 seeds).

**Checkpoint 4 — PROGRESSION_V2 folded in** (owner-approved, `cut29-wip:docs/PROGRESSION_V2.md` §4): every system has a
minimum lineage age and a fallback age (`SystemDef.min_age_h`, `fallback_h`; `LineageState::age_h` = the absences' clock
or the ticks lived); one new system a report (`reveal_left`; a unit is the systems sharing a trigger — the pen's group is
one), the rest in `Lineage.reveal_queue` with `reveal_next` (id, trigger, hours still to wait); the pen = the Mother met
AND age ≥ 72 h, or 5 days whatever the climb (the 3-day stall is gone); tactics drip one per band boss slain or per day,
`Bold` a day after `Guarded`, `Hunter` a day after `Bold`; ≤ 5 beats a report (`+N more`; a system's reveal is one of
them); stance levels at 10 · 40 · 150 · 400 runs; reserved save fields `glory`, `expeditions`, `era_gate` (and the wire's
`age_h`, `reveal_queue`); the dayplayer reports (not gated) day-1 systems, systems and beats a report, days with
something new and the longest gap. The TUNED harness takes the wall's edit at most every other day and reads its worst
death once a day (the search was > 40 CPU-min a seed).

**Measured** (dayplayer, 8 seeds × 14 days × 3 check-ins, checkpoint 4 tree):

| Bar | Value | |
|---|---|---|
| IDLE D8 by day 1 · D13 by day 4 (median) · D23 by day 12 | 8/8 · 1.7 · 8/8, median 7.7 | PASS |
| IDLE stall before D23 ≤ 4 d · gold every day · no King in 14 d | 2 · 14/14 · 0/8 | PASS |
| Stance L3 by day 2 / L5 by day 7 (IDLE median) · every IDLE check-in grows a track | 1 · 7 · 336/336 | PASS |
| Days with a stage opened: IDLE ≥ 8 · PICKED ≥ 10 (median) | 8 · 7 | FAIL (IDLE at the bar) |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 1.50 · 1.09 · 1.32 | FAIL |
| IDLE never out-paces PICKED | 92 % | PASS |
| PROGRESSION_V2 (info): day-1 systems · most units a report · most beats · days with something new · longest gap | 7 · 3→1 (units fixed after the run) · 8→≤ 5 · 14/14 · 1 d | reported |
| TUNED (one seed, checkpoint 3): D28 day 7, D33 day 9 — far ahead of PICKED (D28 day 9–12) | | not yet gated over 8 seeds |

**Checkpoint 5** (`6d4defd`) and the first `node tools/gates.mjs --full` (2026-10-01, 48 min: metrics 30 seeds 2896 s,
qa 30 seeds 538 s — all PASS, dayplayer 8 seeds, TUNED and the leave-one-outs on 4):

| Bar | Value | |
|---|---|---|
| IDLE: D8 day 1 · D13 by day 4 · D23 by day 12 · stall · gold daily · no King · L3/L5 · a track every check-in | 8/8 · 1.7 · 8/8 med 8.2 · 2 · 14/14 · 0/8 · 1/5 · 336/336 | PASS |
| IDLE never out-paces PICKED · none > 60 % of the gap | 95 % · 27 % | PASS |
| Days with a stage opened: IDLE ≥ 8 · PICKED ≥ 10 | 7 · 9 | FAIL |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 1.50 · 1.42 · 1.23 | FAIL |
| TUNED ≥ 1.5× PICKED at D18 · D23 · D28 | 1.00 · 1.12 · 1.19 | FAIL |
| RANDOM slower than IDLE to D13 | 12 % | FAIL (any package pick helps: RANDOM's random picks are good picks) |
| Nothing required (TUNED − S ≥ IDLE ± a check-in) | forge s3 D18 | FAIL |
| Each system moves TUNED's D23 | packages +0 h · pen +16 · forge +0 · pets +4 · bank +0 · quests −8 | FAIL |
| metrics: COUNTERED ≥ D14 · lanes D5 · stalls on every cohort set · stances by wall · quests | 40 % · +12/+20 · worst 1.4 % · guarded 4 hunter 2 steady 4 · worst 0 % | FAIL |
| metrics: the forge lever on raterAU (a Cut 25 row the contract keeps? — its `whole forge < best row` twin is retired) | forge +26 vs row +20.8 | FAIL (the longer ladders) |

**Bisect of the older rows** (8 → 30 seeds, `metrics --bots`, the pre-Cut 30 tree with and without the temperament
overrides): removing the overrides alone reproduces the moves exactly (COUNTERED mean best 15.47 → 13.73, DEFAULT 7.75 →
6.62, the same run counts), and no single temperament's removal does (curious 15.13, cowardly 15.90, brave 15.93, greedy
16.47 — within the seeds' noise of the base). The rows were balanced with the heirs' random temperaments acting for
them; the contract removes that policy. Identifying what comes home at camp (a candidate fix) moved nothing: these bots
die every send.

**Checkpoint 6** (`a0a524c`, the owner's decision 2026-10-01: wall answers + slower idle, the 1.5× bars kept; the old
rows re-derived on the new bots) and the second `node tools/gates.mjs --full` (metrics 2891 s; qa all PASS on 30 seeds):

| Bar | Value | |
|---|---|---|
| IDLE floor (D8 day 1 · D13 by 4 · D23 by 12 · stall · gold · no King · L3/L5 · tracks · stage days ≥ 8) | 8/8 · 1.5 · 8/8 med 6.7 · 3 · 14/14 · 0/8 · 1/6 · 336/336 · 8 | PASS |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 1.50 · 1.75 · 1.43 (the run before: 1.50 · 1.60 · 1.63) | FAIL at D23 (noise around the bar) |
| IDLE never out-paces PICKED · RANDOM (re-expressed) · content reach (re-derived) | 100 % · 88 %/88 % · 7/8 | PASS |
| Stalls ≤ 1 % of sends (re-derived per bot): IDLE · PICKED · TUNED | 0.34 % · 2.16 % · 4.98 % | FAIL (PICKED, TUNED) |
| Days with a stage opened: PICKED ≥ 10 | 9 | FAIL |
| TUNED ≥ 1.5× PICKED at D18 · D23 · D28 | 1.00 · 1.00 · 0.94 | FAIL — the pen adds nothing over PICKED |
| Leave-one-outs: nothing required · each system moves D23 · none > 60 % | forge s1 D28 · packages +28 h, pen +4, forge +4, pets/bank/quests 0 · packages 88 % | FAIL |
| metrics: Lanes D5 on IDLE (re-derived) · expeditions on IDLE · 20-min absence · drill item packed | near 0.09 · far 0.05 · 16.6/11.0 · 100 % · 100 % | PASS |
| metrics: every stance best at a wall · quests keepable | bold 7 guarded 3 · worst `bank D22` 0 % | FAIL |

What moved them: the PICKED picker reads the wall (a second panel from the deepest lit stone, `PkgOption.d_wall`) and
takes no swap that costs the walk; later bosses are drilled at the third day met (the Foundry the fourth); Hunter plays
the Foundry card first and goes for archers; Bold strikes the boss through its heal and dives when hurt; the pack carries
what a tactic throws; RANDOM picks blind; a package row that loops rests for the floor (IDLE's stalls 1.66 → 0.34 %).

**Checkpoint 7** (`16ebe82`, the owner 2026-10-01: the pen re-scoped as optional late fine-tuning; each system measured
by its own output) and the third `node tools/gates.mjs --full` (metrics 3069 s; qa all PASS on 30 seeds; TUNED on the
bots' 8 seeds, the leave-one-outs on 4):

| Bar | Value | |
|---|---|---|
| IDLE floor (D8 day 1 · D13 by 4 · D23 by 12 · stall · gold · no King · L3/L5 · tracks · stage days) | 8/8 · 1.5 · 8/8 med 6.7 · 3 · 14/14 · 0/8 · 1/6 · 336/336 · 9 | PASS |
| PICKED: stage days ≥ 10 · never out-paced · content reach | 10 · 100 % · 7/8 | PASS |
| Stalls ≤ 1 % of sends: IDLE · PICKED · TUNED | 0.00 % · 0.00 % · 0.00 % | PASS |
| Each system by its own output (TUNED / TUNED − S) | packages 124/152 h→D23 · pen 27.6/27.2 mean best · forge 27.6/24.6 · pets 45.1/46.5 % deaths · bank $1559/$1311 a day · quests 12.8/0 kept | PASS |
| metrics: quests keepable · lanes on IDLE · 20-min · drill item · expeditions | worst 93 % · 0.09/0.05 · 100 % · 100 % · in band | PASS |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 1.50 · 1.75 · 1.28 (runs before: 1.43, 1.63) | FAIL at D23 — swings ±0.2 between runs at 8 seeds |
| TUNED beats PICKED by ≥ 15 % at D28 · D33 | 1.00 · 1.04 | FAIL — the pen adds no depth at the deepest walls |
| RANDOM never faster to D13 · slower to D23 | 88 % · 75 % | FAIL (6/8 at D23) |
| Nothing required (TUNED − S ≥ IDLE ± a check-in) | TUNED − forge, seed 1, D18 | FAIL (one seed × milestone) |
| Every stance best at a wall by ≥ 0.02 | bold 2 · guarded 1 · hunter 3 — Steady never | FAIL |

Steady's wall: weighed from each wall's stone and from D1, at one level, drills revoked, with a death free (time) or a
quarter (the walk's carry), Steady is never first: Guarded is the safer of the two careful stances, Bold passes more,
Hunter answers the Foundry. Rows tried on Steady (a fire throw at a boss in gas, a step out of gas) moved nothing; they
are out. Steady's identity — the school, the generalist — has no wall of its own in this row's terms.

**Checkpoint 8** (the owner's round 5, 2026-10-01). `metrics` (30 seeds, 4386 s) and `qa` (30 seeds, 905 s) all PASS
on `1ddf1ed`, including the new rows: *Steady is the safest default* (deaths a send from D1 across the walls: steady 7 %
· guarded 9 % · bold 87 % · hunter 21 %) and *Guarded, Bold, Hunter each best at a wall by ≥ 0.02* (bold 3 · guarded
1 · hunter 3 of 7). The 16-seed full dayplayer leg was stopped (the coordinator: the full gate only once everything
passes; at 24 threads a TUNED or leave-one-out fortnight takes 35–40 min, so 7 × 16 of them ≈ 3 h). Deep drills now
read the boss's depth on the lineage's own route (the forks move a boss a band: seed 2's Queen sits at D23), and TUNED
writes the Foundry's counter from its golems' fact. Fast tier, `dayplayer --seeds 6` on the committed core:

| Bar | Value | |
|---|---|---|
| IDLE floor (D8 day 1 · D13 by 4 · D23 by 12 · stall · gold · no King · L3/L5 · tracks · stage days) | 6/6 · 2.0 · 6/6 med 7.8 · 3 · 14 · 0/6 · 1/5 · 252/252 · 8.0 | PASS |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 1.75 · 2.12 · 1.74 (4 seeds: 1.70 · 2.12 · 1.50) | PASS |
| Never out-paced · content reach · stalls IDLE/PICKED/TUNED | 97 % · 6/6 · 0 / 0.43 / 0 % | PASS |
| Days with a stage opened: PICKED ≥ 10/14 | 9.5 (10 on 4 and 8 seeds) | FAIL at 6 seeds — PICKED reaches the bottom by day 10 |
| TUNED beats PICKED by ≥ 15 % at D28 · D33 | 0.95 · 1.13 (static-depth deep drills: 0.95 · 1.22) | FAIL |
| RANDOM never faster than IDLE to D13 · slower to D23 | 67 % · 67 % | FAIL |

Why D28 does not move: `h→D28` is reaching the D28 boss's floor, so it is gated by the walls at or above D23, which
drill at 3–4 days; PICKED crosses D23→D28 in a day (8–56 h). The pen opens at 72 h and TUNED's D23 times equal
PICKED's (104–144 h): its counters do not break the D18/D23 walls sooner than the drills and the picker's stances do.
Tried: deep drills from D23 (`DEEP_FROM = 23`, 4 seeds) — PICKED slows to D23 (152–176 h) and TUNED at D28/D33 reads
0.98 · 0.95; reverted. RANDOM: a random package is mostly a good package — on 2 of 6 seeds an early random stance
(Guarded) passes the Warlord before IDLE's drill; RANDOM is slower than PICKED on every seed at D23.

## Deviations
- **The pen re-scoped** (the owner, 2026-10-01: rules are an optional late fine-tuning layer): `TUNED ≥ 1.5× PICKED at
  D18, D23, D28` → *TUNED beats PICKED by ≥ 15 % at the deepest walls (D28, D33), and is never required* (TUNED on the
  bots' 8 seeds). **Systems by their own output** (the owner): `each system moves TUNED's D23 time` → per-system rows —
  bank: gold a day; pets: deaths a send; quests: rewards kept; forge and pen: the climb's mean best depth; packages: hours to
  D23 — each better with the system than without; `nothing required` kept. **`None > 60 % of TUNED − IDLE` retired**: the
  systems are no longer measured on one depth scale, so a share of a depth gap does not fit them.
- **A floor given up**: on the idle floor a floor the stall guard stops twice is given up home (a queued return keeping
  60 %), not paced to a stall (the old stall kept nothing); a harness's literal set stalls as before.
- **Quests drawn where a night keeps them**: `reach` the record's band (three under it), `reach · no return` five under,
  `home from` a cleared floor (the deepest lit stone at or under the record) — a bank or a return that reached it.
- **Old rows re-derived on the new bots** (the owner's decision): `COUNTERED reaches ≥ D14 ≥ 50 %` (a written counter
  set on a literal lineage, 70 → 40 % with the temperaments gone) → *content reach: PICKED ≥ D14 by day 2 on ≥ 50 %* (7/8);
  `Lanes D5: no set dominates` (EDITED's written lane sets; +12/+20 vs 15) → *the D5 lanes on IDLE: both viable, the
  weaker passes D8 at least half as often as the stronger* (near 0.09 · far 0.05); `Stalls ≤ 1 % on every cohort set`
  (worst 1.4 %) → *stalls ≤ 1 % of every bot's sends* (the bar kept, measured on IDLE/PICKED/TUNED). The originals print
  as retired. Bisect in checkpoint 5's notes: the temperaments' removal alone moved them.
- **RANDOM** is IDLE's twin until its first pick (the Warlord met, D8), which is a check-in or two before D13: the row
  reads *never faster than IDLE to D13, slower to D23* (each ≥ 80 % of seeds; 88 % · 88 %).
- **Guarded meets a telegraph with a drink**, not the contract's step back: `telegraph → retreat` looped retreat ↔
  explore on ~3 % of PICKED's sends. **Light hands** grabs the kill's drop (`on_kill → pick up`), not loose loot (the
  loose row looped pick up ↔ explore).
- **The Cut 25 forge lever** (`Whole forge's bank move < best row's`, raterAU +26 vs +20.8) is retired (the contract's
  list); it prints for the record.
- **The expeditions-per-8 h row** is gated on IDLE (fresh and at D13) in 6 – one send and its rest per 20 minutes
  (`REST_MIN_TICKS`: the band's own reason, no sortie farm), EDITED still 6–16: DEFAULT, the two-row fighter dead every
  send, is gone.
- **Every stance best at a wall** weighs each stance at the worn stance's level, from the deepest lit stone at or above
  the wall, the record at the wall (every bank row then asks for the floor past it), a death a quarter of a pass.
- **PROGRESSION_V2 over the contract's reveal**: the pen opens at the Mother met *and* 72 h (fallback 5 days), not "the
  Mother met or a 3-day stall"; Bold/Hunter/tactics drip by day (owner-approved refinement, 2026-10-01).
- **The full dayplayer table runs 16 seeds** (the owner, round 5: PICKED vs IDLE, RANDOM, the leave-one-outs and their
  TUNED base were noisy at 8 and 4): each job cached under `target/gates/dp/` by the binary's hash, an interrupted
  gate resumes; at 24 threads a fresh full gate grows past an hour (measured below). The picker re-reads its panels
  once a day and when a package arrives (a PICKED fortnight 1651 → 867 CPU-s).
- **Steady is exempt from the wall contest** (the owner, round 5: it is the idle default): its part is *Steady is the
  safest default* — the fewest deaths a send from D1 across the walls, against each other stance. Guarded, Bold and
  Hunter must each be best at a wall by the margin. Steady returns at 25 % from L3 (20 % before).
- **Deep drills arrive later** (the owner, round 5): a boss at D28 or deeper drills after 6 days met (3 for the
  others, the Foundry 4), so a player who writes the counter breaks the wall days earlier — the pen's ≥ 15 % at D28/D33.
  TUNED writes the counter row for a boss at its record or one past it (its card or throw bought).

- **Steady's bank row reads `depth ≥ record + 1`** (the contract's `depth ≥ best`): a bank at the record itself never
  passes it; the first floor past the record is banked (one new floor a successful send; L4 pushes one further when whole).
- **Stances by level** (ours to write): Steady L2 heals at 35 % and rests under 40 %, L3 rests under 50 %, L4 banks a
  floor further when whole, L5 steps off a telegraph when hurt — Steady never rests at full length (that is `Guarded`'s:
  rest under 80 %); `Hunter` arrives with the Warlord met (the contract left its stage open; it is the Warlord's counter).
- **A scar is a day**: runs of one day that see a boss scar him once (per run, a night's 20 sends scarred him to
  −30 % in one absence); the drill still comes at the second run that meets him.
- **The Foundry's drill** comes from its golems (the contract's "the wall search supplies a wall's drill when no
  fact-counter exists"): the search costs seconds natively and minutes in wasm per offer; the golems' `reflect_melee` fact
  names the same counter (`reflect read`).
- **Tests**: seven temperament-override tests replaced by `no_temperament_chooses_an_action`; four seed-specific repros
  (a dice death whose replays all survive, a chased death, a patch with its purchase, the fork tablet's reading floor)
  search for their case again, the temperaments having moved the nights.
