# Cut 35 — Gunner, long guns and short guns

Owner request2026-10-07: add a Gunner class, with long and short guns, distinct
from archery. This becomes the next class priority, before proposed Rogue/
Ranger specializations. Finish Cut34's live verification without abandoning it.
Status: fifth class, paid unlock, per-class XP, weapon ownership/forge choice,
named default tactics and all five skill rungs implemented. Forge choice is
verified in the actual WASM worker at320/400/1440. Sprite/portrait packed and
rendered. Compact played chamber/reload/aim watch and death-trace replay
indicators are implemented with scoped real-worker QA. Public class offers remain hidden until action art and
the complete acceptance checks pass. Initial campaign source had an explicitly
selected bow; earlier gun-progression claims are superseded below. Corrected
actual-kit campaigns verify real guns firing/reloading. Normal earned paid-upkeep
campaigns initially reached D13 within48h on all six cases. After chosen
policy and reload-retreat fixes, all six pass48h; two owned seed1 builds master
at168/152h. The other four miss seven-day mastery. All six clear Tier0 within
seven days. Initial kits still expose early failures. Preserve failures
and continue tuning/full acceptance. No deployment.

## Identity and progression

A fifth real base class, using existing inherited per-class XP and the selected
bloodline Legacy tree. Firearms are actual equipment, not bow skins or renamed
Ranger verbs. Starting long gun, distinct short-gun alternative; house/manual
send, gold/forge, named workers and class-switch rules remain intact. Unlock
at historical D13 for8 existing marks, home only. No new currency, bullet
purchase flow or paid automatic reload. Validate affordability/depth before
mutation; invalid/away requests retain the complete save.

Old missing Gunner XP is absent until explicitly unlocked/selected. Preserve
older class saves and unselected gameplay/RNG; do not retrofit firearm loot
into all existing drop pools or silently equip guns on other classes. Gunner
weapons use normal ownership, forging, kept-item and auto-equip infrastructure,
with a class-specific gun preference that does not choose a sword because of
its nominal attack roll. Explicit loadout preference remains authoritative.

## Combat contract (initial tuning numbers)

Long gun: one loaded shot, range8 and line of sight, damage6–10; ignore2
armour, bounded at zero. Reload20 ticks. Range never grants sight through darkness: existing lantern/
lantern-rig sight supports the full8; ordinary visibility remains authoritative. Short gun: two loaded shots, range3,
damage4–7; primary target plus up to two hostile neighbours in a forward cone
with clear sight. One bounded target list, no allies or hits through walls;
reload both chambers in15 ticks. Each enemy's damage and reflection resolve
through ordinary counter/death handling; no aggregate hit bypasses shields,
resistance, King rhythm or source attribution. Do not add enemy-population
scans to the tick scheduler; target selection runs only when firing.

Fire spends one chamber only after target/range/sight validation succeeds,
including misses and reflected shots. An invalid shot preserves target memory,
ammo, RNG, cooldown and the full run. Start each run loaded. Reload is a timed
commitment: no gunfire before completion, no instantaneous reload on page
reload, offline slice or ordinary weapon change. Chamber/load state belongs
to the actual gun in that run, so swapping weapons cannot refill it. No global
ammo wallet or consumable ammunition inventory.

Level1 fire and reload; level3 aimed shot (long gun preparation) / close burst
(short gun spends both chambers); level5 smoke retreat; level7 fast reload;
level9 finishing shot. Final verbs must have actual distinct effects and numeric
costs before exposure. Do not expose placeholders merely to fill this ladder.
Aimed shot trades one stationary action for a stronger accurate next shot on
the same visible target; losing sight/moving cancels the aim. Close burst cannot
activate with one chamber or masquerade as Ranger double shot at long range.

A named automatic class row handles firing/reloading after safety/drills/player
rules and chosen tactics/temperaments, before generic attacks. Default Gunner works from its first send without
Pen edits. Authored/literal policies stay explicit; fire, reload, aim and burst
become optional vocabulary as owned class levels unlock them. Ranger retains
its marks, sustained shots and mobility; Gunner is defined by range choice,
shot opportunity and vulnerable reload windows.

## Observability and art

Chunky class choice with a distinct portrait/sprite and readable gun silhouette.
Both equipment items and actual verbs have icons, without icon boxes. Keep
existing gothic cartoon/Vampire Hunter D/Downwell direction; no mascot imagery.
Show a tiny1/1 or2/2 chamber indicator and reload progress beside hero presence/
watch portrait; do not introduce another speed panel or permanent instructions.
Combat log names shot, burst and reload completion; meaningful tactic cue pulses,
held shots and cancelled aim show short actual reasons. Replay and death traces
use played weapon/chambers/commitment, never later camp equipment or class.
Desktop/phone tooltips explain actual damage/range/reload through Rust metadata.

## Numeric acceptance and delivery

A. Save/class/unlock/gear architecture: old raw save/event baselines for all four
classes unchanged; all Gunner gates, paid unlock, switches and refusals exact.
Old zero/missing firearm state omitted. Snapshot class and gun state at send.
B. Real actions: long range8 passes/9 refuses, short3 passes/4 refuses, LOS,
cone bounds3 targets/allies excluded;6–10 and4–7 rolls/armour2 bypass; exact
20/15 reload boundaries and chamber consumption, miss/reflection, swap/aim
cancellation, actual safety and authored-rule precedence. Whole/sliced/reloaded
native/WASM8h saves/reports equal for one/three bloodlines and active reload.
C. Default normal earned Gunner (no XP/kit/currency grants) reaches D13 on
seeds1/3/5 within48h after legal unlock, both gun choices actually fire/reload;
earned mastery within7days with an explicitly owned build. Preserve failures.
Compare long/short guns on armour/ranged/open and close-pack arenas: each must
win at least one measured matchup; neither simply dominates Ranger on all.
Tier5 fixed earned builds each clear within48h after explicit preparation.
D. Real-worker320/400/1440 choice/forge/watch/reload/death/replay, keyboard/tap,
no overflow, exact durable save, actual earned screenshots shown to owner;
readable generated/packed gun and hero art with primitive fallbacks.
E. Profile equal-work positive firing/reloading against prior runtime, <=10%
overhead excluding intentional combat work; bound fanout, state and event
history. Full quick/clippy/codegen/build/WASM and routine18case gates pass;
optional exhaustive statistical audit stays separate. Push approved changes,
never deploy. Partial checkpoints do not certify a complete playable class.

## Class/progression/forge checkpoint — 2026-10-07 (partial A/B/D)

The original campaign source selected bow100253. Numerical results in this
section remain historical class-progression results and are not certification
of actual gun use. Corrected current checkpoint follows below.

Actual Gunner unlock costs8marks at historicalD13, at home with a built house.
Missing old Gunner XP stays absent until paid unlock/selection. Class changes
involving Gunner refuse for the entire live run. Starting long gun and purchased
short gun use existing kit forging; short ownership costs one forge unit once,
then owned switches are free. Automatic equipment retains the gun variant;
explicit worn finds remain authoritative. New sends start loaded, live saves
retain weapon-instance chambers and commitments.

Named class:gunner policy runs below authored/drilled/safety rules, above ordinary
tactics. L3 long preparation costs one action, then a stationary same-target
shot hits accurately at1.5× attack; movement/lost sight cancels. L3 short burst
uses both chambers. L5 smoke retreats one legal step and blinds nearby hostiles
30ticks, cooldown90. L7 fast reload takes10/8ticks, cooldown100. L9 finisher
requires foeHP<=25%, uses one chamber at2× attack, cooldown60. Mastery owns
gunslinger atL10. Refusals preserve ammo/RNG/cooldowns/save. Authored cadence
retains its row and uses actual aim/burst/reload after switching class.

Permanent examples/gunner_prepare.rs loads an earned camp, pays unlock/short
purchase and advances21 eight-hour check-ins without XP/kit/currency grants or
removing authored rules. Source class-prepare-core-final-20261006/sentinel-start
has inherited paid equipment/Legacy and earned marks/gold, seed3, tier1. Both
guns reachD13 in8h andD18 later; long endsL7/1262XP, shortL8/4178XP at168h.
Both mastery_hours are null: this fails C, not a success redefinition. Artifacts
scratchpad/gunner-earned-20261007/{results,*-start,*-after}.json. This source
does not prove seeds1/5, no-inherited-kit starts, Tier5 or matchup superiority.

Forge shows actual Rust damage/range/chambers/reload, existing chunky framing
around choices, unframed equipment silhouettes, readable gun names and correct
icons for completed upgrades. Actual earned save imported through app import,
real WASM worker; headed browser320/400/1440 has no horizontal overflow/page
errors, exact price225 paid once and free owned switch. Captures forge-*.png,
QA forge-qa.json and script in the same earned artifact directory. Gunner
portrait/sprite/actions, watch chamber progress, played replay/trace metadata
and full D still pending. UI_READY=false keeps class offers hidden.

Final checkpoint checks:661core PASS/one ignored,14tool PASS, TS/copy1716/zero;
fast all-target clippy warnings denied, codegen and web build PASS. Rebuilt
real fastWASM5466844bytes. Six old tier0/challenge event/save baselines and two
numbered migration saves exact. No fresh routine18case/full positive perf or
native/WASM8h one/three-bloodline certification in this partial checkpoint.

## Opportunity, counter and character checkpoint — 2026-10-07

Gun choice metadata now includes inherited Legacy damage through the same
Hero::apply_level/legacy::apply/atk calculation as send. Selected/Select labels
describe starting kit preference; explicit found loadout remains authoritative.
Automatic aim is reserved for targets a normal minimum roll cannot finish;
burst for a tough primary or useful legal forward spread. Authored fire/aim/
cadence remains explicit. Known ranged reflection selects an owned carried
melee sidearm in one announced action, preserving parked gun chambers and
absolute reload state via existing bow_swap. Actual ordinary melee hits counter
reflection, not a bypass of ranged-hit handling. Direct fire still reflects;
unknown traits aren't silently known. Gun restores when ranged reflection leaves
view, with gun ready callout. Positive save/park/reload/restore tests pass.

Permanent earned harness deselects the inherited weapon through public loadout,
retains all other finds/authored rules, verifies real loaded gun at send and
actual firing/reload completion over1000ticks. Optional --renewal-brace pays
existing Legacy respec/upgrades; --steady equips an explicit owned stance. All
earlier failures retained. Current seed3/tier1 inherited baseline: longD13/40h,
bestD23/L8 at168h; shortD13/8h, bestD15/L7. Renewal/Brace: longD13/48h,
bestD13/L7; shortD13/24h, bestD23/L8. Current mastery fails. This is one earned
post-King camp, not independently earned normal seeds1/3/5. Long14–22 damage
experiment failed D13/7day (D13 at64/160h); rejected, retained artifacts,6–10
restored. Original numeric gates remain intact. Need real normal unlock camps,
appropriate owned builds/counters and full C/E before public exposure.

Character sprite and portrait generated through built-in imagegen in existing
style, default/male aliases, actual alpha on sprite. Sources/prompts in
art/prompts/gunner-character-20261007.md. Existing pack pipeline495frames passes
QC, old493 frame pixels/dimensions exact. Real-worker headed1440/400/320 earned
loadout forge/watch captures have correct portrait and visible Gunner sprite,
no page errors/horizontal overflow. Full D remains open for action icons,
class choice, chamber/reload/aim cues and played death/replay state.

Final quick663core/14tool PASS, one ignored, TS/copy1716/zero;27focused tests
and fast all-target clippy warnings denied PASS; codegen/web build/WASM rebuilt
5467369bytes. Native/WASM full loaded camp, send and1000tick StepResult bytes
exact, long3shots/3reloads, short5shots/2reloads. Six old event/save fixtures/two
numbered saves exact. These scoped checks don't replace8h one/three slots,
routine18cases or equal-work perf. No deployment;95fun target remains unverified.

Architecture audit starting points: hero.rs Class::ALL/ladder/starting_weapon,
meta.rs class unlocks, defs.rs/item.rs weapons and drop weights, ai.rs ranged
hits/reflection/target memory, packages.rs generated policy precedence,
turn.rs reload scheduling/minbatch, wire.rs HeroSnap, class offers/forge/presence,
render/look.ts manifests and ui/skin.ts icons. Current ranged ItemDef boolean
cannot alone encode chambers, gun range or reload commitment; introduce a
bounded firearm profile/state rather than inferring all ranged weapons as guns.

## Saved weapon foundation — 2026-10-07 (partial A/B)

`firearm.rs` owns separate long/short profiles and chamber commitment, saved on
each `Item` as an optional field. Non-guns omit it exactly. Reload deadlines are
absolute run ticks (20/15); checked overflow, repeated reloads and invalid shots
refuse without mutation. Valid fire reserves one chamber, or both for short
burst, before future hit/miss/reflection rolls. Per-item state survives stowing,
swapping and serialization. The geometric spread filter restricts secondary
targets to neighbours of the primary in a90-degree forward cone within range3.
The future action caller must additionally validate visibility/hostility/LOS,
cap at three targets, commit target memory only after successful validation,
and resolve ordinary damage/counters. None of that combat work is certified yet.

Actual firearm `ItemDef`s are separate from legacy `ITEMS`, preserving existing
global loot and vocabulary enumeration. They have zero global drop weight.
They are not obtainable through the UI or current class. No fifth-class enum,
unlock, forge choice, firing verb or automatic reload is exposed in this step.
Scheduler must stop batches at reload deadlines and emit completion exactly
once; these primitives do not yet run timers themselves. Seven focused tests
cover range/LOS/sight refusals, exact timer boundaries, weapon-instance saves/
swaps, slice serialization, burst/overflow, spread geometry and old-item JSON.
These are primitive checks, not full native/WASM8h gameplay parity.

## Art checkpoint — 2026-10-07

Built-in imagegen, separate long/short prompts, actual alpha. Source
art/ui/icons/it_long_gun.png and it_short_gun.png; existing ui-skin.py packs
96px siblings into web/public/ui/icons and skin.json.115icons total, old assets
unchanged. Long slender rifle versus squat double-barrel silhouette inspected.
Exact prompts/provenance art/prompts/gunner-equipment-20261007.md. These icons
do not expose Gunner or firearm items before gameplay exists. Next A/B core
implementation; portrait/sprites and action icons remain part of later D.

Foundation verification: final quick643core PASS/one ignored,14tool PASS,
TS/copy1705/zero; final fast all-target clippy warnings denied PASS. Rebuilt
real fastWASM5383173B. Six old tier0/challenge complete loaded/advanced events/
saves and twelve old Tier5 style/base campaigns remain byte-exact. This does
not certify all four class migrations, Gunner gameplay or CUT35 completion.

## Combat/scheduler checkpoint — 2026-10-07 (partial B)

Real `fire`/`reload`/L3 `close_burst` actions reserve actual equipped-gun chambers
before any RNG/target-memory mutation. Shared ordinary hit resolution applies
long-gun2armour bypass, misses, ranged reflections, King rhythm, marked damage,
shield-wall interception, death/kill attribution and monster awakening. Short
spread uses a fixed three-entry target list from actual visible hostiles, with
range/LOS/forward-neighbour geometry; burst reserves both chambers and doubles
the one resolved damage roll per target. Empty guns cannot smuggle shots through
existing generic attack or direct ranged attack paths. Ordinary attacks keep
original defence/RNG calculation (no new zero clamp for non-guns).

One saved `Run.gun_reload` commitment identifies the real item/deadline. Quiet
batches stop one tick before that deadline; actual tick completion occurs before
hero actions, emits `loaded` once and preserves per-weapon state while stowed,
dropped or stolen. No inventory/monster scan on ordinary ticks; ownership scan
only at a positive completion deadline. `reload` starts on a partially empty
short gun too. While any reload is active no gun fires, including a swapped gun.
GunSnap contains actual item id, kind, chambers, range, adjusted damage, armour
bypass, reload duration/remaining ticks. Old heroes omit it. TS matches Rust;
no chamber UI or replay event-driven updates are certified in this checkpoint.

Seventeen focused tests (seven primitives plus ten combat/scheduler/snapshot)
pass: real range8/9 and3/4, LOS/darkness/allies and full-save/event refusals,
forty matched-RNG damage rolls with positive misses, three-target cap, forward/
wall exclusions, Lich reflection, generic ammo bypass, exact reload deadlines,
swapping/reload continuation, actual quiet-batch versus single-tick equality,
burst requirements, camp-independent played metadata and positive secondary
Warlord shield interception. Only the primary target can be deliberately aimed;
incidental spread cannot borrow the primary boss's aimed bypass.

Permanent `examples/gun_check.rs` drives explicit literal reload/fire rows in
controlled arenas. Both weapons have positive real shots/reload completions,
with whole versus uneven sliced/reloaded final native saves exact. Synthetic
weapon/arena preparation is explicit, not earned Gunner/class progression. This
is not the48h/7day/Tier5 or native/WASM8h one/three-bloodline acceptance proof.
Fifth class, home-only paid unlock, actual forge ownership/preferences, default
named class row, aimed shot/smoke/fast reload/finisher, sprite/action art, UI and
positive full perf measurement remain required next. Do not expose a partly
implemented class or weaken the original A–E bars.

Combat checkpoint final verification:653core PASS/one ignored,14tool PASS,
TS/copy1705/zero; final all-workspace/all-target fast clippy warnings denied,
codegen and web build PASS; real fastWASM5393860B. Both controlled61tick
positive firing/reload cases have complete native/WASM whole and each uneven
sliced/reloaded StepResult/save equality. Six old tier0/challenge full event/
save fixtures and12older Tier5 campaigns exact. No fresh routine18case or full
positive-performance gate at this partial checkpoint. Full A–E remains open.


## Played-state observability checkpoint — 2026-10-07

Actual tick-boundary Gun events carry worn chambers, aim and reload deadline;
null clears a parked/stolen gun. Event payload boxed: Ev remains80bytes. Absolute
reload duration records actual fast reload10/8ticks, with default omission for old
prototype saves. Snapshot and played events use one Rust calculation. Negative
checks allocate nothing or scan inventories/enemies; countdown ticks add no events.
Shared watch/death-trace replay component shows chambers, Ready/Aim/Reload and
progress. Actual rendered gun state follows replay clock including backward seek;
GPU and2D fallback expose the same state. Existing hover/focus/tap stat tooltip
shows damage, tile range, reload seconds and armour piercing. No new speed panel.
Real headed earned seed3Tier1 app1440/400/320 has no overflow/page errors; actual
400px firing/reload and replay show0/1 Reload. Evidence gunner-observer-20261007.
29focused tests and665core/14tool quick checks pass; clippy/codegen/TS/copy/build/
fastWASM pass. Six old full-save/event fixtures and two numbered saves remain
exact. Both controlled61tick positive native/WASM firearm fixtures, including
uneven sliced/reloaded complete steps/saves, exact. These are explicitly arenas,
not normal earned progression or eight-hour/multi-bloodline proof. Original A–E
remain open: normal three-seed balance/mastery/Tier5, action icons, full eight-hour
parity/positive performance and routine18case gates. UI_READY=false; no deployment.


## Normal earned campaign checkpoint — 2026-10-07

examples/gunner_campaign.rs earns real Tier0 unlock camps from empty towns on
seeds1/3/5; only legal manual construction/sends/hiring and paid kit/Legacy.
No fabricated seed, depth, currency, XP, kit, facts or authored policy. Camp record
persists across class change: D13 measurement uses actual report.deepest, with
first10000tick positive fire/reload and full native/WASM save/step equality.
Reusable camps and --hours/--seed/--gun bound tuning; short/targeted jobs are
explicit diagnostics, not full acceptance. Full mastery observations legally
repeat owned descents after a clear. Default AI with declared paid upkeep reaches
D13 long/short at32/16h(seed1),48/8h(seed3),16/8h(seed5). Short1 masters168h;
remaining five miss7days. Fixed initial kit48h has three failures. Evidence:
scratchpad/gunner-normal-{fixed,upkeep}-smoke35-20261007; all failures retained.
This verifies the stated earned paths, not C's missing arenas/Tier5 or full A–E.

Default Smoke can act below35% health during a committed reload; original deadline
and actual action/level/cooldown costs remain. A50% trial regressed several builds
and was rejected. Neither35% behaviour nor paid progression implies every build
is faster. Automatic Gunner internal labels stay out of ticker/map captions;
actual shot/log/ammo/skill cues remain, explicit fire rows retain captions,
meters say Gun handling and unloaded chambers say Empty. Scoped real-worker
screens/tap tips pass1440/400/320;666core/14tool quick and focused client checks
pass. Six earned first10000tick native/WASM snapshots/events/saves exact, eight
legacy fixtures exact. Original8h multi-slot, positive perf, action icons,
arena/Tier5 and routine18case acceptance remain pending. Public class hidden.


## Chosen-policy acceptance — 2026-10-07

Selected tactics and temperaments must precede automatic class:gunner handling,
which remains above generic stance fallback. Pen, counter drills and stance safety
retain precedence. Actual gas_step actions with long/short guns must consume
one/two chambers, then yield to the automatic reload without reporting phantom
attacks. No fire or shortened deadline during the commitment. Actual Skittish
retreat must act both with loaded chambers and during reload, without changing
ammo/deadline. Non-firearm attack results and compiled policy remain unchanged.
Recheck earned three-seed paired48h paid campaigns after the policy change;
previous campaign numbers belong to their recorded runtime, never this change
without a rerun. Positive actual Rule events prove choices were executed.


Chosen-policy result: native668tests/oneignored and scoped actual browser checks
pass. All eight direct tactic attack branches now propagate actual firearm
refusal; no empty-gun fake success. Six earned10000tick samples and old fixtures
remain native/WASM exact. Full current paid168h: seed1 longD13=56h (FAIL48h),
mastery136h; other five D13=16/48/8/16/8h, mastery misses L9/7/9/9/8.
Chosen Boss focus or no temperament does not fix seed1long48h. Preserve those
failures; no gate weakened. Current sources gunner-chosen-policy-paid{48,168}
-20261007. Earlier six-of-six claims describe their earlier recorded runtime.


## Close-threat shot experiment — 2026-10-07

Hypothesis: automatic long-gun preparation spends an exposed action when a
hostile is already adjacent. Trial suppresses automatic preparation at adj>0,
retaining explicit aimed_shot, already committed aim, shot damage/range,
chambers,20tick reload, safety and selected-policy precedence. No stat/XP grants.
Compare paid48h seeds1/3/5, both guns against50016d8. Positive controlled checks
must show adjacent automatic fire consuming one chamber, distant preparation
spending one action, authored adjacent preparation still legal. Preserve failed
trials; accept only with evidence of useful balance, not just passing code tests.

First trial rejected: skipping all adjacent preparation passes seeds1/3 long
at40/8h but regresses seed5 long to FAIL48h. Data close-aim-paid48-20261007.
Narrower trial: only skip preparation under adjacent threat when target HP is
within an ordinary maximum shot after armour; sturdy foes still prepare.
Damage/range/reload and authored aim remain unchanged. Test covers both close
weak and strong cases plus an already prepared foe closing, not just a fire call.

Second trial also rejected: narrow normal-max finishing rule makes both seed1
and seed5 long fail48h (seed3 passes48h), versus baseline5/6. Code/tests patches
preserved under gunner-close-aim-20261007; production restores50016d8 policy.
No accepted combat-policy improvement from these trials. Stop guessing against
three campaign seeds; measure controlled gun/Ranger matchups next to identify
where preparation/range/reload tradeoffs actually pay. Native baseline retained
in native-before (hash recorded) for scoped equal-work future comparisons.


Controlled comparison contract: permanent gunner_matchups records per-seed
clears/time/damage/shots/reloads/aims for long gun, short gun and Ranger bow.
32 paired seeds, L3 first, open24x14, equal light and class-level HP/attack;
no forged kit, armour, heals or Legacy. Fixtures explicitly declare modified
foe HP/armour and are not earned campaign proof. Living enemies use actual AI;
no stuns. Death, exit and1200tick timeout fail. Ranger uses real level-gated
kite/volley/double-shot/shoot, Gunner its actual class handler. Report clear
count and time/damage conditional on clears separately; do not hide failures
inside a median or claim fixture success certifies default progression/Tier5.

Ranger policy audit: the first kite-first fixture has0/32 close-pack clears even
atL10, so comparing guns only against it would exaggerate their strength.
Retain the original artifacts; permanent driver now reports both sustained bow
(volley/double-shot/shoot) and kite-first bow explicitly. Neither is a claim of
an optimal Ranger build. A bow win against both guns is required for the scoped
non-dominance observation, not a weak kite-only comparator.


Reload-step trial: close-pack fixtures reveal the long gun's vulnerable wait.
During an active reload only, default class handling may spend its scheduled
action on ordinary retreat when a hostile is adjacent. No free movement on the
reload-start action; no fire/refill/timer change, no added range/stat or new
save field. Safety/authored/chosen rows retain precedence. Existing earned L5
Smoke remains separate and higher priority; normal movement has no blindness
or cooldown grant. Blocked steps wait. Positive full-save/action/deadline tests,
paired32-seed arenas and all six paid48h campaigns decide whether to retain it.


Reload-retreat accepted checkpoint: earned paid D13 long/short seed1=16/16h,
seed3=48/8h, seed5=48/8h, all6PASS48h. One longer first gate than baseline is
preserved: seed5long16→48h. Mastery1long168h/1short152h, other4L9miss; all6
actual Tier0 clears128/144/128/80/136/88h. No grants/relaxed numeric gate.
Controlled L3 close-pack long27/32vs21,short32vs30; long clears more safely but
slower130vs100ticks. L10 median armour90/170/330, close90/30/70, cross50/80/40
(long/short/sustained bow); each gun has a win and bow does not lose everywhere.
Full670tests and old fixtures pass; four8h one/three-slot positive-reload parity
cases and two actual one-tick retreat nativeWASM cases exact. Scoped equal-work
native firing/reload overhead+1.41/+1.98%, intentional new retreat work excluded.
Artifacts and limitations in HANDOFF. Tier5/action art/full UI/routine gate open.
