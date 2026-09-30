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

## Deviations

*(To record.)*
