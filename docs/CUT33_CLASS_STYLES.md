# Cut 33 — Sentinel and Hexbinder

Implementation contract,2026-10-06. Two specializations of existing classes,
not new XP ladders or currencies. Follow Cut32 Legacy; preserve the unselected
class/tier0 game and old complete saves. Entire contract includes core, bridges,
UI/art, earned gameplay, replay, parity/performance and required gates.

## Choice and persistence

Sentinel belongs to Fighter; Hexbinder belongs to Caster. Each needs its own
parent class at level10 and historical D23 in that bloodline. Free choice/change
at home only, no run in flight, no class/XP/gear/gold/Legacy reset. A `none`
choice removes the current class's specialization. Remember each parent choice
when switching classes. All refusals preserve the complete save. Old saves
have no selected style and no automatic assignment.

Choosing a style explicitly equips one named automatic tactic ahead of ordinary
tactics, below safety guards/drills. The review must show that behaviour before
confirmation; removing the style removes its row. No extra tactic slot/currency.
Custom/literal policies remain explicit; their authors can use the exclusive
verb. The pen vocabulary only offers the chosen, qualified ability. Snapshot
the style at send; subsequent camp choices cannot change live/replay abilities.
Class enum/XP stays Fighter/Rogue/Ranger/Caster; style is separate optional state.
Empty camp state and zero/absent run/monster state omitted from old saves.

## Actual actions

**Sentinel — Riposte.** With an adjacent hostile and cooldown0, arm20 ticks;
cooldown60 ticks. The next positive adjacent hostile melee strike, after normal
resistance/Legacy handling and the existing mirror response, consumes the arm:
halve incoming damage, rounding up, then counter for8 damage if the hero survives.
Normal enemy counter/reflection/shield-wall rules apply to the counter; it is a
hero melee hit, with an Attack event naming Riposte. Consume once; no recursive
counter on ranged attacks, hazards, reflected damage, allies or zero damage.
Expiration without a hit gives no counter. Default row: adjacent enemy→Riposte.

**Hexbinder — Hex.** Target a visible hostile within the existing bolt/Slow range
and line of sight. Cooldown60 ticks; hex lasts40 ticks. Actual attacks by that
enemy deal2 less damage, floor0, before other resistance/Legacy handling.
No reduction of unrelated hazards, poison ticks or another enemy's damage.
Cannot reapply an active hex; ordinary monsters/bosses retain their other traits.
Default row: visible enemy→Hex nearest, falling through normally when unavailable.
No extra enemies, summons, HP scaling, global scan per damage, or new RNG draws
for arming/hexing. Events, snapshot statuses, tips and replay must show the effect.

## Numeric acceptance

1. Both styles and exact parent/level/depth/home/run gates, `none`, class-switch
   memory, all saved other slots/shared resources unchanged, exact refusals.
   Six pre-cut old WASM load/advance/save baselines and numbered native fixtures
   remain byte-exact. Forecast keys differ for the selected active style only;
   removing it restores the base key. Old unselected game hash unchanged.
2. Controlled real actions: Riposte10→5 received and8 counter damage, consumed
   once; odd5→3, cooldown/timer boundaries, expired/ranged/hazard/reflection/ally
   exclusions, survival/death and mirror priority. Hex10→8 versus unaffected
   other enemy/hazard10, small1→0; exact40/60 countdown, target/range/LOS gates.
   Wrong class/style/level refuses without action mutation. Chosen verbs plus
   automatic named rows actually fire and appear in traces/events.
3. Normal simulation/forecast/send/live reload snapshots use the same chosen
   style. Native/WASM whole/sliced/reloaded8h entire saves/reports match one/three
   slots with positive styles and in-progress cooldowns/hex. Existing tier0
   current18-case suite passes; exhaustive audit remains separate.
4. Earn each parent level10 without XP/currency/gear grants. Same earned
   preparation seeds1/3/5, selected style versus base parent, fixed owned builds
   and Tier1/5 eligibility explicitly labelled when synthetic. All six style
   Tier5 cases clear within48h; report paired actual outcomes and any failed
   baseline. At least one genuine earned run/replay of each style shows its
   exclusive action. More base classes follow demonstrated gaps.
5. Chunky Hero choice/review, parent XP/progress, default tactic preview,
   distinct readable icons/portrait treatment, hero presence and enemy Hex tip.
   320/400/1440: locked/choose/remove/refused/stale/away/reload/keyboard/tap,
   no overflow, meaningful real-worker checks and actual earned screenshots.
6. Equal-work native tick/status/damage overhead<=10% versus pre-cut source,
   absolute timings and modifier coverage; no per-tick style allocation or
   unbounded history. Fast WASM, build/typecheck/copy/codegen/clippy/required
   tests and routine gate. Push approved work; no deployment.

No selectable but inactive styles. A core checkpoint does not complete this
contract; keep remaining app/art/gameplay/parity work explicit until verified.

## Core checkpoint — 2026-10-06

Rust choice/persistence/default rows and actual actions implemented. Twelve new
tests, final628 core PASS/one ignored; full quick and clippy/build pass. Final
routine18-case gate source596967ebcc35f5eb terminal0; optional exhaustive audit
not run. Old six WASM/two numbered native save baselines exact; both earned
styles native/WASM complete load/send/uneven advance/reload/8h saves/reports
match. Controlled one/three-slot whole/slice/reload tests cover both styles.
Riposte counter respects King reflection rhythm, uses melee hit semantics and
does not overwrite the hero's last chosen verb. No app choice is exposed yet.

Earned diagnostic starts from the same owned814-point Tier1 save. Root buys54,
Recovery/Warding108, Caster unlock8 owned marks. Generic Bolt alone fails level10
at14days (level9); preserve that artifact. Paid perks alone qualify296h, owned
boss counter80h, known reflection ranged counter40h. No XP/currency/gear grants.
Explicit post-training removal of ordinary Bolt override exposes default Hex;
production style choice preserves player rules. Final8h Tier1 campaigns both
clear: Sentinel656 exclusive actions/296 counters/8runs; Hex774/11runs. Complete
earned event streams and raw saves: scratchpad/class-prepare-core-final-20261006/.
Earlier diagnostic variants remain retained; no claim of generic-policy success.

Acceptance4's Tier5 pairs, acceptance5 app/art/responsive screenshot work and
acceptance6 equal-work overhead are pending. No deployment; do not mark full
contract complete based on this checkpoint.

## Complete scoped verification — 2026-10-07

Rust-owned offers now expose parent XP, level/depth gates, effect, duration,
cooldown and default row. Native, WASM and actual worker methods choose/remove
styles. Chunky Hero review, explicit override priority, remembered presence,
Hex enemy status and dedicated transparent icons are implemented. First-town
choices stay hidden. Live presence uses the sent hero snapshot. Failed Hex
range/sight validation now preserves the remembered target exactly.

Final quick:631 core PASS/one ignored,14 tool tests, TS and copy checks green
in219s under competing work;15 focused specialization tests PASS. All-target
fast clippy warnings denied, codegen check, fast WASM5378837B, web build and
copy-lint1705/zero violations pass. Existing bundle advisory remains. Final
routine18-case suite terminal0, source388e1612426bc870, no cached legs:
wire75.4s, metrics129.3s, dayplayer226.3s/all selected bars PASS. Optional
exhaustive/statistical/system-removal/historical audits remain uncertified.

Permanent style_check.rs pairs seeds1/3/5 on the same earned parent XP, owned
gear/policy and162 paid Legacy points; seed and Tier5 eligibility are synthetic.
No campaign purchases or policy edits. Fighter base clears8/8/8 game-hours;
Sentinel8/8/16, with716/627/1308 actions and460/426/758 counters. Caster base
8/24/16; Hexbinder8/8/8 with932/1156/912 Hex actions. All twelve reachD34 and
unlock6; all six selected cases meet48h. Sentinel is slower on seed5. This is a
fixed-policy comparison, not an optimized base-policy search or proof of every
higher-tier build. Final artifacts: scratchpad/class-balance-final-20261007/.

Actual earned native/WASM replay JSON matches completely: Sentinel run181,
72 exclusive actions/30 counters, hash687f35dac044b4d3; Hexbinder run244,
27 actions, hash1b05d4e7f2d30e84. Removing the camp style preserves the entire
existing replay. Whole8h,16 half-hour slices and uneven reloaded slices match
entire saves/reports for both styles and one/three legally funded bloodlines;
each native/WASM slice response/save also matches. Six older complete WASM
baselines and two numbered native saves remain exact. Capsules are not newly
persisted across page reload; this proof concerns existing replay capsules.

Meaningful real-worker UI tests cover320/400/1440, locks, preview/cancel,
stale class review, actual away refusal, one-call confirmation, adopted tactic,
remove/retained XP, exact durable reload, keyboard details and inner identity
bounds. Tactic tests cover old/new icon sizing and named style activation.
Actual earned headed screenshots at all three widths preserve entire read-only
saves, have no errors/overflow, and restore remove/rechoose state exactly.
The320 identity clipping and oversized live cue icons found during QA are fixed.
Artifacts: scratchpad/class-ui-20261006/; exact built-in imagegen prompts:
art/prompts/class-styles-20261006.md. Sources art/ui/icons/style_*.png, packed
web/public/ui/icons/style_*.png, existing packer validates alpha corners.

Permanent style_perf.rs uses equal32-foe arenas,500 tick/damage calls,101
samples, same9 hero resets and nine counterbalanced old/new rounds versus
a39e4d7. Median ns/call: tier0 none492.840→509.020 (+3.28%); tier5 none
514.762→538.822 (+4.67%); actual Riposte527.680→563.642 (+6.82%); actual Hex
520.360→545.380 (+4.81%). All meet10%; positive effects verified. Parsing and
map generation excluded; identical reset/clone work included. This isolates
tick/status/damage overhead, not catch-up, rendering or whole-combat pacing.
Source/binary hashes, full rounds and parity/replay artifacts retained alongside.

Cut33 is implemented and verified within these acceptance boundaries. No
deployment. Broader content work remains: audit base-class default/training
friction before adding more classes, expand elite/boss/loot mechanics for higher
tiers, then optional Rogue/Ranger specializations with demonstrated distinct
roles. Preserve the failed generic-Bolt14day preparation; do not grant XP or
silently replace authored rules. Keep-going goal remains active.
