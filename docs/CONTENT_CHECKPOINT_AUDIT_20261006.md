# Content and checkpoint audit — 2026-10-06

Audited main cb1c4f8, current packaged WASM SHA256
3bccd626c19a9c01c76a528e97414e73865ec2b1c159260fd3555191f46fc2b1.
Authority: PLAN's idle-first amendment, CUT30/CUT30_5, UX_SIMPLE,
UX_FIRST_HOME, UX_BLOODLINES and subsequent owner UI requests. This is an
implementation/readability audit and bounded diagnostic play, not a fresh full
balance certification. Historical projection tables are not measured gameplay.

The highest-impact finding is a catch-up boundary defect, followed by poor
visibility of meaningful milestones. Expanding the monster list comes later.
The existing dungeon has substantial breadth; the proposed months-long reset
layers have not been built.

## Inventory and completion evidence

| System | Current implementation | Classification / remaining evidence |
|---|---|---|
| Areas | 7: Warrens D1–4, Burrows D5–8, Fens D9–13, Crypt D14–18, Foundry D19–23, Deep D24–28, Sanctum D29–33; stairs at D34 finish | Implemented: descent.rs Biome/biome_for/ENDING_DEPTH. Actual current samples reach D28; no newly earned King/ending walkthrough in this audit. |
| Units/bosses | 37 MonsterDef entries, including captive/summoned units; six band bosses D8/13/18/23/28/33; Captain lieutenant D5 | Implemented: defs.rs/descent.rs; count is definitions, not37 ordinary enemies. All37 have a direct or boss-prefixed frame in packaged atlas. Presence of art does not certify visual quality. |
| Combat identities | 4 classes: fighter/rogue/ranger/caster, each with a level/verb ladder; 4 underlying traits | Implemented: hero.rs/class_ladder, class XP wire and hero menu. Earned source fighter reachesL10; three other classes need deliberate player picks and were not newly exercised here. Advanced classes from Progression v2 are missing. |
| Chosen behaviors | 4 combat styles,6 extra tactics,4 personalities,5 levels each; earned and revocable boss drills/scars, late optional pen | Implemented: packages.rs/systems.rs. Real current Guarded/Hunter and drill cues verified in prior checkpoint. Fourteen IDs are catalogue breadth, not14 independent slots. Custom stance is compatibility/player rules, not a fifth shipped style. |
| Items/forge | 43 definitions:6 weapons,4 armour,11 potions,13 scrolls,1 gold,8 miscellaneous; forge ladders, supply packing, properties and automatic worker upgrades | Implemented: defs.rs/kit.rs/items UI. Potions/scrolls share family icons deliberately. Gold/bones/trap currently lack direct it_* art and fall back to glyphs in itemIcon; remaining owner icon request. Relic sets from Progression v2 are missing. |
| Town/automation | Manual free house plus4 milestone buildings: forge/storehouse/kennel/Savings;11 worker definitions including given quartermaster, paid/free hires, ranksII–IV after4/8/12 days | Implemented: town.rs/tree.rs. Current rank bonuses are mechanical, not merely cosmetic (rank_bonus). Real no-choice traces leave every later plot unbuilt, as required. New sessions need stronger visibility of ready free construction. No automatic construction permitted. |
| Heroes/Legacy | 3 active persistent bloodlines, independent policy/XP/Legacy, shared gold/town, inherited health/damage/armour upgrades; retired heroes in Chronicle | Implemented: bloodlines.rs/legacy.rs and prior invariants/UI checks.3 upgrades capped at3 ranks; prices3/6/9 each, total54 points for all ranks. Earned week2 source has842 unspent points; no-pick trace intentionally buys none. This is a ceiling/opportunity, not evidence all players finish it on day1. |
| Checkpoints | New records secure carry without ending healthy runs; waystone passage secures eligible gold; remaining carry bank100%/return60%/death0% | Implemented: engine/offline/forecast passage paths and CUT30_5. Existing tests cover mixed checkpoint/theft/swap/exit accounting. Actual D8/D21/D28 camps captured. The heir purse floor is separate and is not death carry retention. |
| Choices/rewards | Waystones/route forks, companion taming/breeding/fielding, quest board, oath rewards, gear/drills/scars/class progression | Implemented code: descent/town/oath/probes/meta plus UI.5 route forks, initially only D5 open. Do not count all forks as immediately selectable. No new earned kennel/breeding/oath-completion walk here; retained tests and source are weaker than that evidence. |
| Ending/reset | Mirror King D33, D34 ending, opt-in4 ascension variants (no rest/short list/bones only/hunted) | Implemented: engine::ascend, ending.ts; tests reaching_d34_is_the_ending_and_d19_is_the_foundry, ascension_keeps_the_meta_and_restarts_the_descent, the_ending_after_an_ascension_records_the_variant. Current end screen is still old standalone art/chips, with little explanation of the variants. Needs current earned visual/choice audit. |
| Months-long spine | Repeatable early expedition reset, glory, eras/dynasty, advanced classes, relic sets, expedition buildings/new biome lanes | Missing/deferred design. PROGRESSION_V2 explicitly starts “Design…No src changes”; CUT30 status and engine.rs reserve glory/expeditions/era_gate as unread fields for a later cut. Existing ascension and multihero do not implement those reset layers. Do not advertise their projected dates as current content. |

Enumeration artifact: scratchpad/content-audit-20261006/inventory.json includes
IDs and source hashes. Its direct-icon check intentionally distinguishes family
icons; a missing direct potion image is not a missing UI icon. Archive audit
artifacts when a durable tester bundle is required; scratchpad is untracked.

## Earned progression and catch-up discrepancy

Three new real-WASM towns, seeds1/3/5: manually construct house, three real
manual runs, hire porter/scout when eligible, then no picks/edits/other builds.
No fabricated gold, unlocks, heroes or runs. Absence checkpoints8/24/72/168/336h
use increasingly long calls; that is a diagnostic cadence, not the application's
30-minute slices or the canonical3-check-in/day bot.

| Seed | 8h record | 24h | 72h | 168h | 336h |
|---|---:|---:|---:|---:|---:|
| 1 | 8 | 11 | 21 | 21 | 21 |
| 3 | 7 | 8 | 20 | 21 | 21 |
| 5 | 8 | 18 | 19 | 20 | 21 |

Each hires scout after3 sends and has positive home gold in every sampled
interval. That does not certify gold every individual day. Raw earned saves for
seed3 and summaries for all3 are in trace.json. Systems are wire objects;
count only entries with open=true, not the31-entry catalogue. No-choice towns
retain the free forge plot; opening a system is distinct from constructing it.
Seed3 day1: D8,58 unspent Legacy; week2: D21,L10,842 unspent Legacy, $58,384.
This absence-only player's idle class/Legacy still grows at the wall, but the
report/next-goal flow does not clearly connect that growth to a useful choice.

Fresh targeted native check `tools/tune.sh idle-d23 --seeds 3 --fresh` PASS:
seeds1/2/3 all reachD23 by day12, median6.33 days,10s. This is one targeted row,
not a gate pass; it doesn't certify stalled/King/gold/system-removal invariants.
Do not relabel the long-call trace as that canonical bot or weaken theD23 bar.

Controlled comparison: load identical earned seed3 eight-hour save; add exactly
160h without any picks/builds/edits. Wall clock ends168.59h in every case:

| Partition | API / calls | Record | Heir | total_turns /10, hours | Open systems |
|---|---|---:|---:|---:|---:|
| One160h call | runOfflineQuick,1 | 21 | 20 | 169.23 | 8 |
| Up to24h | runOfflineQuick,7 | 22 | 23 | 169.91 | 14 |
| 8h | runOfflineQuick,20 | 28 | 28 | 191.60 | 25 |
| Actual client-size0.5h | runOfflineSlice,320; only final last=true | 28 | 47 | 305.52 | 25 |

Cause in source: offline.rs::run_offline_with finishes a started run past the
budget **on every call**, clamps carried rest and resets reveal_left=1; app.ts
runOfflineChunked calls it every30min. runOfflineSlice's last flag only changes
stall verdict work, not the run boundary. age_h is max(clock_s,total_turns/10),
so repeated gifted completions also accelerate age-gated unlocks. Each internal
slice is effectively treated like a player return. This is not UI text error,
not forecast noise, and not a measured performance speedup. It causes extra
simulation work and different progression for the same absent duration.

Artifacts cadence.json/app-slices.json contain exact source/WASM hashes and
outcomes; cadence-*.json are raw resulting saves. Existing single-slice parity
checks preserve this behavior and cannot detect it. The application's 30-minute
path is the faster-progressing case, so the D21 coarse-call screenshots must
not be presented as current normal app fortnight progression. Actual client-
sized result D28 captured on400/1440. Data covers one controlled source, not a
statistical estimate or completed all-mode regression.

## Checkpoints compared with established idle references

This is a deliberately selected comparison set, not a current popularity
ranking: Melvor for combat-reward clarity, Idleon for persistent multi-character
progression, Stone Story for autonomous combat and late rule automation. Their
published mechanics establish comparison points, not Riddle tuning times.

- Melvor's developer describes tiered Strongholds with equipment prerequisites,
  different mechanics and selectable enhancement rewards, plus area information
  and modifier-source visibility. Apply the principle: a milestone should name
  both the threat and the useful thing beating it opens. Riddle's bosses already
  change conditions/drills; those rewards are scattered across Tactics/Works/
  report Details. [Developer update](https://news.melvoridle.com/free-content-quality-of-life-updates-coming-in-v1-3/).
- Idleon's official site describes multiple characters progressing away, with
  classes/resources supporting other characters and world bosses opening more
  content. Riddle's bloodlines already provide persistent identities; they share
  gold, not a shipped crafting-specialization network. Strengthen distinct class/
  tactic plans before adding chores or asserting cross-hero roles exist.
  [Official site](https://www.legendsofidleon.com/).
- Stone Story's developer tutorial gates the Mind Stone behind a boss and prior
  practical equipment/potion knowledge; it changes automatic combat decisions
  and supports looping. Riddle's optional late pen fits that sequence, but early
  styles/drills must communicate their effects before the editor is asked to
  carry the experience. Its supplied20-minute tutorial duration is not a claim
  about time to unlock the stone. [Mind Stone introduction](https://stonestoryrpg.com/stonescript/intro_tutorial.html).

These sources were fetched2026-10-06; Melvor's referenced update is2024-06-04,
Stone Story's tutorial is older maintained documentation, not a recent patch.
No unsupported first-hour/week/month comparisons or offline-cap claims.

## Impact-ordered implementation queue

1. **Catch-up slice semantics and age correctness (highest).** Write a contract
   before edits. Distinguish internal transport slices from a genuine absence
   boundary; allow at most one past-budget finishing run per whole absence,
   preserve rest across intermediate slices and one reveal budget per report.
   Verify same earned source/total elapsed across0.5h/8h/whole call, save/reload
   in flight, one/three bloodlines, pre-scout behavior, fractional remainder,
   worker/quest/day transitions and real report merge. Require simulation state
   parity after excluding only explicitly documented presentation bookkeeping;
   don't silently normalize gold/XP/events/ages. Retain existing real-return
   convenience, manual construction, uncapped away progression and numeric bars.
   Profile before/after at equivalent outcomes, then relevant targeted rows.
2. **One readable milestone goal/reward at camp/report.** Compact shaft currently
   shows next encounter from D1 (WarlordD8) even at recordD21/28. Preserve that
   travel forecast but distinguish the next progression wall/new reward (e.g.
   FoundryD23/KingD33), with one existing icon and short reward phrase, details
   on demand. Use Rust-owned eligible/known content, no client invented unlocks.
   Verify new/fresh, early boss, post-boss, late wall, route-swapped and paused
   watch states at400/1440; hide spoilers and avoid adding a dashboard.
3. **Direct useful preparation from a wall.** Free ready forge plot and hundreds
   of available Legacy can coexist with repeated failures and a next-worker
   price. Link the existing construction/hero upgrades or class/tactic choice
   from the named obstacle; distinguish ready vs merely unlocked. No required
   forecast reruns. Verify meaningful real choice improves that player's target,
   keeps optionality and never builds/spends automatically.
4. **Finish remaining item icon coverage and late-screen readability.** Gold,
   bones and trap currently fall back to glyphs. Reuse matching existing art or
   generate style-consistent silhouettes if needed. Audit earned King/ending
   and all ascension variants, explain their actual changes on hover/tap in the
   chunky UI. Unit portraits/sprites should remain reusable with checked gaps;
   verify packaged paths and mobile bounds rather than only manifest entries.
5. **Variety with the existing content before a new progression layer.** Give
   first visits/boss victories readable biome identity and a useful existing
   reward/choice; test distinct class/tactic plans across bloodlines, Foundry
   and Deep situations rather than HP-only enemy inflation. No artificial time
   padding or mandatory pen edits. Require earned encounters and targeted rows
   for each changed content slice.
6. **A scoped early repeatable expedition proposal/prototype.** The missing
   months-long spine is architectural content work, not another UI polish task.
   After1–5, contract a single opt-in reset layer with explicit kept/reset state,
   measurable payoff, useful new decision and three-bloodline semantics. Retain
   finiteD34 ending; do not build expedition/era/dynasty simultaneously or reuse
   projected timing as proof. No auto reset, consumed absence or loss of shared
   town/Legacy without an explicit reviewed design.

Audit scope complete as an inventory/diagnostic/priority queue; implementation
items remain open. No new gameplay edits or deployment in this checkpoint.
