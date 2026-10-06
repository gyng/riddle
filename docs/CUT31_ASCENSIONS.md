# Cut 31 — numbered ascensions

Implementation contract, 2026-10-06. Owner direction and complete intended scope
are ENDGAME_ASCENSIONS.md: endless harder descents, affixes/elites/boss changes,
then branching Legacy and classes. This cut implements the endgame loop first.
Partial checkpoints below do not certify the whole cut.

Status: A/B/C implemented; D scoped checks completed with the explicit
unchanged-preparation deviation below. The app now
exposes numbered descents, authoritative previews, encounter modifiers and elite
map marking. Five consecutive actual clears demonstrate the loop on one earned
build. Fixed-preparation/build comparisons, exact absence parity, profiling and
the routine18-case suite are verified below. These do not certify the optional
exhaustive balance/migration audit or arbitrary higher-tier build viability.
The branching Legacy tree and class specializations remain separate work.

## Save, unlock and reset contract

Each bloodline owns selected difficulty `tier`, highest selectable `unlocked`,
and highest completed `cleared`. Difficulty zero is the existing dungeon.
Clearing tier N unlocks N+1 with checked arithmetic; repeating a completed tier
cannot increase unlocks again. Storage is constant size, not one record/tier.
A run snapshots its tier at send and saves it in history/replay copies.

Old saves with no numbered state remain difficulty zero, including old challenge
ascensions; never infer harder clears from their ascension counter. An old ended
save grants tier1 eligibility on the next explicit new-descent action. Loading
alone does not reset, alter live difficulty, consume RNG or spend currency.

`begin_descent(tier)` is a new path, separate from old `ascend(variant)`. Initially
available at an ending, it validates the entire request before mutation. Invalid,
premature or locked requests preserve the full save exactly. The selected
surviving hero begins the next descent; retain identity/heir/class, all currency,
Legacy, equipment upgrades, unspent supplies, vault/party, town/workers, facts,
trained tactics and chosen policy. Recompile generated tactics against the new
record immediately; authored policy is retained. Reset selected depth/checkpoints/freshness,
record-linked cache, recent departure counters and old run/replay presentation.
Other bloodlines/live runs remain unchanged. Retain global clocks and run IDs.
No automatic send or construction; scout continues existing automation.
Existing challenge API preserves its verified reset behavior while new UI is
being built. Do not expose a selectable higher tier until its mechanics exist.

## Delivery checkpoints and gates

A. Core progression, validated separate reset, old-save compatibility and run
snapshot. Tests: five ordered clears unlock1–5; replay0 does not unlock6;
three selected slots preserve full other games and shared wallet; premature,
locked and overflow refusals preserve exact saves; tier0 gameplay/RNG/hash
unchanged. Old live tier0 saves retain their replay and RNG. No client claim.
B. Three actual affixes, two elites and one boss alteration, bounded arithmetic,
modifier metadata in snapshots/tooltips/death attribution. At most two affixes
and one elite ability; deterministic choice independent of gameplay RNG.
Tests include spawn/summon/ally distinctions, counter effects, replay/save
faithfulness and tier0 parity. Specify each mechanic and its counter before
implementation; no hardcoded finite difficulty list.
C. WASM/native bridge and chunky ending/tier selection, read-only consequences,
320/400/1440 controls and actual earned screenshots; class/Legacy retained.
Five actual consecutive clears demonstrate loop and durable progression.
D. Fixed-seed tier1/3/5 tuning, meaningful build comparisons and normal required
checks. Median equal-work native tick overhead target ≤10%, report timings.
Whole/sliced/reloaded absence parity for one/three bloodlines. Preserve all
current tier0 balance gates, manual construction and no absence punishment.

A–D must all pass before calling numbered ascensions implemented. Affix balance,
higher tier completion, the Legacy tree and class specializations remain explicit
work, never implied by progression-only tests. No deployment until requested.

## Checkpoint A verification — 2026-10-06

`endgame.rs` owns constant-size Progress, validation/idempotent checked unlocks,
read-only eligibility for old endings, and a separate `begin_descent` reset.
Optional serialized state keeps old challenge counts distinct and leaves old
save fields absent. Run difficulty snapshots survive saves/history/capsules;
forecast keys distinguish positive selected tiers without changing tier0 keys.
Normal ending tick records the played run's tier. Old challenge API returns to
its unscaled tier0 and retains numbered unlock progress if present.

Eight new focused tests within the full workspace run:578 PASS/1 existing
ignored in37.01s. Covers five sequential progress clears/repeat/locked/overflow,
old count37 not implying harder clears, all three selected bloodlines carrying
wallet/Legacy/class/kit/supplies with full other games untouched, live-run saved
tier, tier0 simulation/events/RNG/key parity, synthetic ending hook reading
played tier, and full-save busy/premature/locked/invalid refusals. Invalid saved
progress rejects load. Synthetic boundary fixture does not prove a King kill.

Earned King seed3/heir33 source from UX_ASCENSION_CONSEQUENCES.md: actual new
core reset to tier1 for single and legally added two other live bloodlines;
keeps heir33, gold2850(single)/450(multi, after their real send costs), classes,
Legacy and kit; full other runtime games unchanged. Complete result reloads
exactly. First diagnostic exposed stale generated record guard D35 becoming D2
only on reload; corrected by immediate recompile and covered in regression.
Diagnostic Rust source copied to scratchpad and temporary example removed.

Rebuilt real WASM load/save for both numbered native results byte-identical.
Six previous-WASM earned ending/old variant/multi saves: rebuilt WASM produces
byte-identical entire loaded saves,1s advance event/snapshot responses and
entire advanced saves. These exercise existing gameplay, not new difficulty
mechanics. Scripts, raw integer-preserving JSON and SHA256 records retained in
scratchpad/endgame-foundation-20261006/. Native/WASM numbered equality is a
save/reset proof; no harder encounter or tier completion claim.

Final clippy all workspace/all targets with warnings denied PASS; fast WASM
5,258,334B, package real directory. Web build/typecheck PASS (existing bundle
size advisory); copy-lint1630/0 violations and diff clean. Full statistical
balance gates, higher-tier profiling/completions, modifier/replay UI and UI
screenshots remain for B/C/D. No deployment or claim of full-cut completion.

## Checkpoint B mechanics contract — 2026-10-06

Three fixed dungeon affixes: Armoured (+1 defence, counter damage-over-time /
blasts that bypass hit defence); Swift (+2 speed, counter Slow); Regenerating
(heal1 each10ticks while awake, wounded and not poisoned, counter Poison or
sustained pressure). Tier1 Armoured, tier2 Swift, tier3 Regenerating+Armoured;
then rotate these pairs. At most two. Enemy HP +8%/tier, attack +4%/tier,
rounded up at spawn, bounded to1,000,000 per stat with wide integer arithmetic.
No extra floors, population, speed scaling by tier or per-tick allocations.
Numbers are initial tuning inputs; D still must establish balance/profiling.

Two elites, deterministic hash independent of gameplay RNG,1/8 of eligible
natural non-boss spawns: Shielded (+2 defence except while stunned/paralysed),
Frenzied (+2 top attack roll/+3 speed while HP≤50%, counter Slow/control/burst).
Summoned enemies carry dungeon scaling/affixes but cannot become elites.
Scripted spawn stat overrides (e.g. brittle4HP skeletons) retain their purpose.
Bosses/grudges do not receive random elite abilities. Captives, companion
spawns, nests and strays remain unmodified. Split offspring inherit parent
modifiers once; no rescaling already modified parent stats. On taming,
remove ascension stats/abilities before companion persistence, preserve health
fraction against ordinary kind/depth stats; no elite-to-pet power exploit.

Modified Mirror King reflects the second consecutive identical damaging verb,
including allies. Snapshot flag on that monster owns the rule, not later camp
tier. Existing cadence alternates early enough and existing telegraph warns.
Damage rolls remain on the original RNG stream; modifier selection consumes no
RNG. Store compact optional modifier metadata in each affected monster, entity
snapshot/spawn event, difficulty snapshot and exact fatal source. Avoid adding
per-kind learned traits for a temporary affix. Death records attach exact
fatal source modifiers for direct/reflected monster damage only; environmental
or overridden gamble causes never guess an elite from another same-kind foe.

Mechanical tests must prove counters and actual attack/regen/boss effects,
spawn/summon/split/tame distinctions, bounded extreme tiers, metadata and
save/replay fidelity. Compare original tier0 event/save bytes before/after on
six earned states. B does not certify C selection UI or D playable tier balance.

## Checkpoint B verification — 2026-10-06

Actual birth hooks cover natural encounters, bosses, escorts, grudges, summons,
wanderers, wraiths and scripted reinforcements. Split inheritance and taming
normalization prevent doubled scaling or persistent elite pets. Existing slow,
stun, paralysis and poison counter the abilities. Saved Mirror King rhythm also
applies to ally attacks; owned cadence alternates before the second reflection.

Thirteen additional mechanical tests; full workspace: 591 PASS, one existing
ignored, 35.80 s. Tests exercise real attack, fatal-source selection, regeneration,
split, Warlord summon, tame, companion persistence and ally reflection paths;
1,000 tiers plus u32::MAX validate bounded stats. Tier0 wire metadata is omitted.
Clippy all workspace/all targets with warnings denied PASS.

Read-only watch boss and death tooltips consume the played encounter's Rust
catalogue. Historical slain status cannot override a live boss. Focused tooltip
checks cover 320/400/1440, hover/tap/keyboard/Escape, exact unchanged saves,
missing metadata and changing encounter identity. PASS 6.0 s. Existing enemy-tip,
unit-icon, frame (63 checks) and QA (41 checks) jobs also pass. Composite tooltip
fixtures prove projection, not gameplay progression.

Earned source: original seed3 TUNED King clear from earned-ending-20261006.
Legal begin_descent(1), normal sends and one eight-hour check-in: 13 further runs,
King cleared at D34, heir34, selected tier1, cleared1/unlocked2. No depth bypass,
difficulty override or new tuning picks in that progression. Actual first-run
D3 elite and D8 Warlord saves retained. Separately named diagnostic-tier1-king
bypasses prior floors solely for a D33 metadata fixture; never a completion proof.
Headed GPU screenshots of the actual D8 Warlord tooltip captured/viewed at
400/1440: real WASM, no page errors or horizontal overflow, played tier1 and
Armoured counter visible; phone frame paused on its normal floor card.
Five new full saves/one-second event advances/after saves match native and WASM
exactly, including those four earned states and the explicit King diagnostic.
Six original tier0 old-WASM baselines still byte-identical for load, advance and
after saves; two earlier numbered-reset native saves also match exactly.

Equal-work native microbenchmark: 32 stunned enemies, frozen hero, 500 ticks,
21 samples, seven interleaved old/new binary rounds. Median: old 354.142 ns/tick,
new tier0 356.622 (+0.70%), new tier5 378.282 (+6.82%). Speed normalized to10 to
exclude intended Swift action-frequency changes. This isolates scheduler/status
cost, not active fights, pathfinding, catch-up or UI serialization; broader D
profiles and balance remain open. No extra optimization justified by this result.

Fast WASM 5,281,348 B, real pkg directory; web build and typecheck PASS, existing
bundle-size advisory; copy-lint 1,634 tagged literals, zero violations. Artifacts:
scratchpad/endgame-encounters-20261006/. Temporary Rust probe sources copied
there and removed from examples. No deployment or full statistical gate claim.
Next C: authoritative tier preview/bridge, ending selection, elite marking and
actual consecutive clears; D: higher-tier tuning and full absence/profile checks.
Legacy tree/classes remain subsequent content work.

## Checkpoint C interface contract — 2026-10-06

Rust supplies optional read-only progression on Lineage and a descentOffer(tier)
preview matching its spawn rules: scaling, bounded cap, affixes, elite abilities,
frequency and boss rhythm. No forecast sims or guessed difficulty from old
challenge counts. beginDescent validates atomically and preserves selected kit/
loadout; the client persists the actual result, never recreates a town on refusal.
Numbered descents remove the previous historical challenge's active rules;
historical challenge count/unlocks remain intact. Their separate restarts retain
old verified reset semantics. This explicitly amends A's variant retention.
Ending foregrounds the next unlocked ascension, allows replay0 through highest
unlocked without constructing a growing list, and folds challenge restarts away.
Preview/cancel must preserve exact saves; failure remains in the review, no retry
of a successful core mutation if metadata refresh fails. Stale preview responses
cannot enable another tier or bloodline. Elite map plates get a gold diamond,
reusing existing density/placement limits; never mark an ally as an elite.

## Checkpoint C verification — 2026-10-06

WASM/native adapters and generated bridge expose descentOffer/beginDescent.
Optional Lineage.endgame grants eligible old endings only tier1 regardless of
historical challenge count; normal old non-ending wire remains unchanged.
Preview reads never mutate the save. Numbered path clears active old challenge
rules and retains kit/loadout; original challenge API and its reset semantics
still pass their refusal/preservation checks. Extreme grudge bonuses now also
respect the advertised difficulty stat cap; original tier0 grudges unchanged.

Ending foregrounds the next tier, folds old challenges, and provides a constant
size 0..unlocked stepper. Rust owns all scaling/affix/elite/boss descriptions.
Cancel, stale responses, locked input, refusals, duplicate clicks and failed
metadata refresh verified at320/400/1440; focused client job PASS6.0s. Original
ascension refusal/review39checks at four viewports PASS; frame63/QA41 PASS.
These injected fake-engine UI cases prove presentation/control behavior, not
actual harder victories. Elite map plates reuse existing placement/density and
add a gold diamond only for hostile elite metadata. A fresh natural Tier1 send
stepped to turn5667 captures the actual visible D3 Frenzied jackal. Isolated GPU
renderer diagnostic at normal scale confirms the gold diamond within viewport
(x438/y183,w59/h19), exact unchanged engine save, no errors; an explicit ally
projection fixture removes the mark. This is a rendering proof, not a new tame
or completion claim. Diagnostic screenshot viewed; actual tame is covered in B.

Earned normal TUNED seed3 King source, no depth bypass or difficulty overrides:
five successive legally started tiers cleared with normal eight-hour check-ins,
no new manual tuning purchases. Per-tier check-ins/runs:1:1/12,2:2/25,3:4/51,
4:16/319,5:10/183. Each ends atD34 and unlocks exactly the next tier. Tier4 takes
128 game-hours, tier5 80; one build/seed is proof of the loop, not balance.
The clear probe sources/artifacts are endgame-loop-20261006; temporary example
removed. The first B tier1 probe had13runs because it sampled a watched first
run before catch-up; this C probe goes straight into offline play.

Actual headed real-WASM UI:320/400 original earned ending to tier1;1440 tier1
with two other legally added/live bloodlines;1440 earned tier5 ending to tier6.
Preview/cancel leave full saves byte-identical. All four confirmed transitions
match complete native expected saves and durable local storage exactly. Shared
gold stays2850(single),450(multi after their send costs),263506(tier6); selected
heirs33/273 survive, depth resets0, both other complete live games unchanged.
No page errors or horizontal overflow. Ending/review/camp screenshots viewed,
app run count restored from the source's real counters alongside raw save; no
gameplay/display depth overrides. Artifacts endgame-selection-20261006.

594 core tests PASS/one existing ignored39.04s; clippy all workspace/all targets
with warnings denied PASS. Native code-generation/adapter coverage PASS.
Six original pre-foundation WASM complete loaded/advanced saves and advance
responses remain exactly equal; two earlier numbered resets also exact. Five B
positive encounter load/advance/after saves still match rebuilt native/WASM.
The explicit lineage/preview APIs add intended eligibility data without changing
those saved states or sampled advance responses. Fast WASM5,320,487B; build/
typecheck/copy1659/diff clean, existing bundle-size advisory. No deployment.
Next D fixed-seed balance/build comparisons, whole/sliced/reloaded one/three
bloodline absence checks and full catch-up profiling; then Legacy tree/classes.

## Checkpoint D verification contract — 2026-10-06

Before the Legacy tree changes runtime: compare all saved state and final reports
for earned tier1/3/5 starts, one and three bloodlines, eight-hour whole,30-minute,
uneven and checkpoint/reloaded calls plus the complete API. No approximate
integer conversion for RNG or seeds; retain raw save JSON/hash as evidence.
Add a bounded Rust regression for positive-tier solo and mixed-tier shared-wallet
slots, including interrupted fights. Same elapsed time must produce zero state/
report differences, and at most one finalization/unlock per selected slot.

Fixed-seed tuning diagnostic uses the same earned preparation at tiers1/3/5
with controlled alternate RNG seeds1/3/5; this is explicitly a seeded diagnostic
fixture, not three independently earned campaigns. All inherited unchanged
prepared builds must clear within14 game-days on these nine cases; never grant
further gear/Legacy or bypass floors to pass. Supplement with real combat policy
comparisons over fixed seeds: Shielded counter via control, Regenerating counter
via poison, Swift/Frenzied counter via slow, and King cadence versus repeats.
Measure actual actions/health/damage, not only formula assertions; define each
comparison's inputs and numeric bar before running it. Existing actual five
consecutive clears remain the earned loop proof. Initial balance bars are new
content's contract, not replacements for tier0 gates.

Profile actual headed-WASM eight-hour catch-up from tier1/3/5 earned starts,
report repeated deterministic outcomes, absolute times and CPU attribution.
Retain the equal-work native <=10% modifier overhead check; full catch-up times
are workload-dependent and cannot substitute for equal-work overhead. Run the
normal required checks after fixes. Broad exhaustive migration/statistical
audit remains separate from the routine18-case current-player suite.

## D findings and explicit preparation-gate deviation — 2026-10-06

The unchanged-preparation14-day bar above **failed**, not green. One earned
Tier1 preparation, controlled seeds1/3/5 and tiers1/3/5: Tier1 clears in8h for
all three; Tier3 in104/24/16h; Tier5 reaches only D31/D30/D33 after336h.
This is a common-preparation diagnostic, not three earned campaigns. The build
has814 unspent Legacy, no Legacy upgrades, forged armour7/pack4/weapon6,
Guarded and no equipped tactics; its authored boss counters remain unchanged.
Artifacts: scratchpad/endgame-balance-d-20261006/, including failed saves.

Explicit deviation: unchanged builds are not required to clear every higher
tier. Higher difficulties may demand a build choice. Keep the failed result;
do not certify the original bar or weaken existing tier0 gates. This is justified
by the same nine cases purchasing all nine **existing** Legacy ranks at home
for54 of the source's814 earned points: all nine clear in8–16h. No granted
currency/equipment, depth bypass, new authored policy or difficulty tuning.
Artifact: scratchpad/endgame-balance-d-legacy-20261006/; reproducible with
`ascension_check -- SAVE NEW_OUT_DIR --spend-legacy`. This isolates a meaningful
owned build choice; it does not establish broad balance or new-tree viability.

The stronger exact-integer transport check exposed two solo continuation bugs:
guide quotes refreshed during an absence were not saved, losing passage income
after reload; a split inside camp rest inserted an hourly worker action, changing
its ledger timestamp. Current quote and rest-continuation state now persist.
Earned Tier1/3/5 eight-hour whole/30-minute/uneven/reloaded/complete API paths
match every saved field and final report, with exact u64 RNG comparison and
original raw saves retained. Tier1/Tier5 previously failed reload; Tier1 also
failed uninterrupted30-minute partitions. Whole outcomes remain unchanged.
Artifacts: scratchpad/endgame-parity-d-fixed2-tier{1,3,5}/; old failures retained.

Mixed1/3/5 diagnostic Rust transport exposed discarded fractional clock state
on reload; `advance_rem` now saves nonzero millisecond/subsecond remainders,
defaulting to zero for old saves. Six previous tier0 WASM save/advance baselines
and two numbered reset saves remain byte-exact. Real earned reset with selected
Tier1 and two legally added/live tier0 bloodlines: all five eight-hour WASM
transport paths match full saves/reports exactly,40 completed runs each.
Artifact: scratchpad/endgame-parity-d-fixed-multi/. Full Rust run598 PASS/one
existing ignored37.60s; the subsequently added fractional-clock regression
also passes. Headed Chromium, rebuilt fast WASM5321443B, actual earned Tier1/3/5
eight-hour catch-up, three fresh-load/cache repetitions each: median0.8082 /
0.6912 /0.9980 seconds; complete raw save and report hashes match each repeat
and transport references. Profile includes load/save while timings cover only
runOfflineQuick, excluding compilation/warm-up; engine timing, not app FPS or
shipping-WASM performance. Tier1/Tier5 passage forecast inclusive samples42.4% /
27.3% of total profile respectively; no extra modifier allocation hotspot.
Artifacts: scratchpad/endgame-profile-d-tier{1,3,5}/summary.json and CPU profiles.
The prior equal-work native modifier overhead remains6.82%, within10% target;
these different encounter workloads do not establish an equal-work speedup.
Final routine quick green39s:599 core PASS/one existing ignored35.47s, typecheck,
copy1659 and14 side tests; initial restricted-sandbox native-host failure passes
both isolated and whole-suite host execution. Clippy workspace/alltargets fast
warnings-denied and webbuild PASS, existing bundle-size advisory. Routine
18-case current-game full gate is running, not yet certified.
D findings above were a checkpoint; final comparisons/gates follow below.

### Fixed-combat comparison inputs and bars

Diagnostic arenas, not earned progression: RNG seeds1/3/5, one awake adjacent
goblin, identical paired starts, normalized large HP to measure actions without
an early death. Shielded Tier1: Fighter's two attacks versus bash then attack;
aggregate extra damage >=2, with actual successful bash/stun evidence. No mace
random stun. Regenerating Tier3: forty real status ticks, frozen actors, wounded
enemy; throw one known poison versus leave it unused. Counter enemy HP at least
10 lower per seed, poison actually consumed/applied. Swift/Frenzied Tier5:
wounded elite, base speed10, atk2, hero waits thirty ticks; owned Caster L5 slow
versus no slow. At least one fewer hostile attack each seed and no greater total
damage to the hero. King Tier1: two attack actions versus two owned cadence
actions, manually recording each actual damaging verb just as hero_action does;
repeat causes one reflection, cadence zero, and higher retained HP per seed.
King arena amendment before rerun: the initial two-action health bar failed
seed1 because the reflected attack missed (0/0 HP loss), while both reflection
event counts were correct. Keep that failure, not a mechanics regression.
Extend this arena to20 actions, requiring19 repeat reflections versus zero
cadence reflections and strictly higher retained HP on every seed. Ordinary
hit RNG remains enabled; feints record their actual tactic verb. This observes
enough real attempts to measure health without assuming every attack hits;
it does not weaken the original per-seed health bar or change encounter stats.

All comparisons exercise actual verbs/events/status ticks; no assertion of
general build balance, whole-run viability or an earned encounter from arenas.

### D completion evidence and remaining broader scope

Routine `node tools/gates.mjs --full` terminal exit0: metrics exit0 in84.4s,
wire QA exit0 in51.7s,18 current-player fortnight cases all selected barsPASS.
Source keya21a8fdf0ffd38a6; no cached legs. Broader system-removal/statistical/
historical migration audits explicitly not run. Current tier0 requirements unchanged.

Four new paired combat comparisons PASS on seeds1/3/5. Shielded extra damage
0/0/2 across two verb pairs (at least one actual successful bash, total+2);
poison lowers regenerating enemy HP by12 each; Swift/Frenzied hostile attacks
3→2 each, hero damage6/6,6/3,7/5 (plain/Slow). Twenty King actions: nineteen
reflections versus zero cadence reflections on every seed, HP loss288/306/324
versus0. These are normalized arenas: only Slow and poison comparisons advance
the tick scheduler; bash/King compare actual verb pairs/sequences, not whole
combat pacing. Initial two-action King's missed-hit result is retained above.
Logs /tmp/riddle-endgame-counter-comparisons{,2}.log.

Final quick suite603 corePASS/one existing ignored35.74s,14 side tests,
typecheck/copy1659 green79s including compilation. Final clippy alltargets/fast
warnings deniedPASS after fixing suspicious negative-assignment spacing.
Runtime and WASM unchanged since the earlier transport/profile proof; only
diagnostic tests/docs added. D's owned-build comparison, full transport parity,
equal-work overhead target, actual catch-up profiles and routine checks complete.
Fresh headed actual earned Tier5ending→Tier6review screenshots at400/1440 on
shared5219: realWASM, exact complete read-only source save, noerrors/overflow;
viewed and shown. Artifacts scratchpad/endgame-d-checkpoint-20261006/.
Original unchanged-preparation14-day bar remainsFAILED, with the explicit
design deviation justified above. Do not describe every original bar as green.

Next CUT32_LEGACY_TREE.md: twelve branching nodes, old purchases preserved,
free home respec, snapshots, meaningful builds and real UI; then two class
specializations. This endgame checkpoint does not certify that later scope.
