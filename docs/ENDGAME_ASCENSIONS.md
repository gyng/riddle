# Endless ascensions and bloodline growth — direction 2026-10-06

Owner steering: clearing the dungeon should unlock endless ascensions with
 affixes, elites and stronger bosses; consider more classes and deeper Legacy
skill trees. Numbered tiers, affixes/elites/boss mechanics and selection are now
implemented under CUT31_ASCENSIONS.md; broader verification remains open.
The Legacy tree and specializations below remain proposed, not shipped.
Supersedes the queue's early-reset proposal as the next design focus;
expedition/era/dynasty layers in PROGRESSION_V2.md remain deferred.

## Current game

Mirror King on D33, ending at D34; optional numbered harder descents plus four
older repeatable challenge variants. Numbered tiers scale encounters and rotate
up to two affixes, with deterministic elites and a tighter King attack rhythm.
Five legally earned consecutive clears verify one build; broader tuning is open.
The old challenge counter remains distinct from numbered tier progress.
Classes: Fighter, Rogue,
Ranger, Caster, with inherited class XP and verb ladders. Bloodline Legacy
persists across heirs and ascensions, but only buys health/damage/armour,
three ranks each. There is no branching Legacy tree or advanced class yet.
Source: engine.rs Game::ascend; hero.rs Class; legacy.rs offers/apply.

The older challenge ascension clears shared spendable gold and selected kit/depth.
The new numbered path retains gold, equipment, Legacy and class progression,
resetting the selected descent while preserving other bloodlines/live runs.
Do not silently reinterpret existing saves or these verified consequences.
The new progression needs an explicit migration and revised pre-descent review.

## Recommended loop

Keep the dungeon's bottom. Clearing it unlocks the next numbered difficulty
for that bloodline: Ascension 1, 2, 3, and onward. A harder descent reuses the
34-floor structure with changed encounters; it is not an infinite corridor.
Track highest unlocked tier separately from the selected tier and current
run. Replaying a completed tier remains possible and never relocks progress.

Each tier combines bounded enemy scaling with a small, readable set of
mechanical modifiers. Start with three affixes, two elite abilities and one
Mirror King alteration. Candidate examples: regenerating enemies countered by
sustained pressure; armoured enemies countered by armour break; marked elite
encounters with telegraphed reinforcements. All need actual compatible verbs,
visible portraits/icons, death attribution and counters before inclusion.
Boss alterations should change a decision, not merely extend a health bar.

The four existing challenges become optional choices, not the entire endgame.
Do not indefinitely stack affixes: propose at most two active dungeon affixes
and one elite ability per enemy initially. Scale progression without scaling
map size, enemy population or simulation work with the tier number.

New ascension selection should preserve town, shared gold and other live
bloodlines. Only the selected bloodline starts a new descent. Exact treatment
of kit, supplies, checkpoints and kept items must be specified before runtime
changes. Existing challenges need compatibility tests and an explicit decision
about their old reset semantics; no silent shared-wallet wipe in the new loop.

## Legacy and classes

Replace the three isolated purchases with a compact persistent bloodline tree.
Propose three branches and twelve nodes initially: survival/recovery,
combat/control, exploration/resource use. Preserve purchased old ranks and
spent points through migration. Some nodes add tactical choices or alter an
existing verb; avoid twelve interchangeable damage multipliers.

First-clear depth/boss milestones award one-time branch access or Legacy;
ordinary runs still provide progress at a wall. Prevent repeated low-floor
milestone farming. Points belong to the bloodline, not the currently living
heir or shared town. Allow free town-only respec initially; changing builds
must not require hours of grinding. A preview must state actual effects.

Keep class XP responsible for class abilities, Legacy responsible for inherited
build choices, and gold responsible for equipment/town purchases. Avoid another
currency. Add two specializations after the tree and endgame work: candidates
are a shield/counter Fighter and a summon/debuff Caster. Names and final moves
are not committed. Each needs distinct equipment preferences, abilities,
automatic tactic support, counters, silhouettes and visible replay evidence.
Specializations should build on existing classes rather than force new heirs
through duplicate XP ladders. More base classes follow demonstrated gaps.

## Delivery order and acceptance boundaries

1. Specify tier/save/reset/reward wire and migration; implement one full
   Ascension-1 loop with three affixes, two elite abilities and one boss change.
   Add selection of unlocked tiers; verify five consecutive clears unlock
   tiers 1–5 without resetting other bloodlines or duplicating rewards.
2. Implement twelve-node Legacy tree, migration of all nine existing purchased
   ranks and respec. Show three viable builds with measurable different choices.
3. Implement two class specializations with at least one exclusive active verb
   each, supported by a default tactic and visible in real replay.
4. Expand boss/elite/loot variety in response to actual walls; defer additional
   reset layers and large class lists.

Before each runtime cut, write numeric balance gates against fixed seeds;
these scope counts are not promises of balance. Preserve the current tier-0
idle/engagement gates and manual construction. Same seed/rules/time/tier must
remain deterministic; whole/sliced/reloaded absences must match for one and
three bloodlines. Save/load during modified fights must preserve affixes.
Replays must snapshot modifiers, not read later camp selections. Legacy respec
cannot affect an already running hero. Death screens name elite/affix causes.
Bound stats and tier arithmetic against overflow; no hard-coded finite tier
list, but no promise of endlessly authored mechanics.

Profile equal-work tier-0/tier-5 scenarios before shipping. Initial target:
median native tick cost no more than 10% above the same baseline workload;
report absolute timings and modifier coverage. No per-tick heap allocation or
unbounded modifier history. Use targeted gates while tuning; broader required
checks follow the runtime cut, never weaken existing gates for new content.
Verify 320/400/1440 UI, no overflow, compact icons with hover/tap detail rather
than persistent explanations; show real earned app screenshots. No deployment
until requested.
