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
