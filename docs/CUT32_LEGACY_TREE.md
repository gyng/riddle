# Cut 32 — branching bloodline Legacy

Implementation contract,2026-10-06. Owner asks deeper Legacy trees and class
progression after numbered ascensions. The new nodes, actual effects, home
respec and responsive Hero tree are implemented. Class specializations follow
this tree as a separate contract. Preserve Cut30.5 and Cut31 tier0 behavior.

## Tree and choices

Twelve nodes: three existing roots, three follow-ups and six fork leaves.
Existing Health/Damage/Armour retain IDs, three ranks, prices3/6/9 and exact
effects. New follow-ups have one rank and cost18; leaves one rank and cost36.
Each branch offers two mutually exclusive leaves. Roots and follow-ups in
different branches can coexist; no hidden automatic purchases or allocations.

| Branch | Root | Follow-up | Leaf A | Leaf B |
| --- | --- | --- | --- | --- |
| Recovery | Health | Restoration: +1 HP on an existing safe rest | Mending: heal potions restore25% more HP | Renewal: +1 HP on a hero kill of a natural hostile |
| Control | Damage | Control: hero-applied stun/slow last5 extra ticks | Venom: thrown poison deals1 extra damage per pulse | Debilitate: owned Slow lasts15 further ticks |
| Warding | Armour | Clear lungs: gas/poison damage reduced2 | Fireward: fire-tagged damage halved | Brace: incoming damage reduced2 while below25% HP |

Follow-up needs its root at rank1 and historically reached D8. Leaves need
their follow-up and historically reached D18. Use retained hero Legacy history
for deepest-ever access; numbered resets must not relock earned tree access.
Current hero/class/death does not own nodes: bloodline slot does. No added
currency. New nodes never grant class verbs, learned facts or automatic policy.
Renewal excludes allies, summons and repeated damage to an already dead enemy;
never heals past maxHP. Mending respects the existing potion/trait modifiers.
Damage reductions floor at0, after existing counter/resistance handling;
no heal from negative damage. Stun/slow bonuses only on hero-owned application,
not native monster attacks, ally actions or passive status countdowns.

Each live hero snapshots only selected new effects at send. Live/replay/save
effects never read a later camp allocation. Old roots alone omit new runtime
metadata, preserving their existing full-save behavior. Forecast keys already
include the complete owned-upgrade map; validate every new effect against the
same send snapshot in normal simulation and forecast/replay.

## Migration and respec

Keep the existing upgrades map and stable IDs; old nine bought ranks, unspent
points, historical spent points and effects unchanged on load. No automatic
point grants, currency wipe or inferred allocation. Unknown/locked/completed/
unaffordable/conflicting purchases refuse without any saved mutation.

Free respec at home refunds actual bloodline spent points, zeros spent and
removes all allocated upgrades. Checked arithmetic; overflow refuses exactly.
No class XP, gold, hero history, identity, town or other slot changes. Refuse
while selected hero has any run or no house. No implicit send. UI previews
refund/retained points before applying, and keeps failures visible/reviewable.

## Verification and delivery

1. Core catalogue, branch/ancestor/depth/fork validation, respec and snapshot
   mechanics. Twelve offers exactly; all nine old ranks/effects preserved across
   old-save migration; all six leaf conflict directions refuse; free respec and
   immediate rebuild produce identical allocation/points. Selected1/2/3 does
   not mutate full other games or shared wallet. Every failure preserves save.
2. Each of nine new effects gets a real action/damage/status regression with
   numeric result: safe rest5 vs4; ordinary heal potion125% baseline (rounding
   specified at its existing calculation); qualifying kill+1 versus summons0;
   bash stun15 vs10 and slow35 vs30; poison pulse3 vs2; specialized slow50 vs30;
   gas/poison damage−2; fire10→5; lowHP hit10→8 with threshold boundary unchanged.
   Saves/replays and native/WASM must retain chosen effects exactly.
3. Three fixed earned builds: same seed1/3/5 Tier5 common preparation, purchase
   old54-point roots plus one18+36 branch using owned814 points. Mender, Venom,
   Fireward choices; no granted gear/currency/class XP or floor bypass. All nine
   clear within48 game-hours; show different actual healing/control/hazard
   outcomes in paired arenas. Initial clear bars are new-content requirements,
   never substitutes for current tier0 gates. Keep any failed baseline.
4. Chunky Hero/Legacy UI, three branch groups, clear fork choice, icons and
   hover/tap effects/prerequisites. Keep initial root purchases easy to scan;
   desktop columns, phone stacked/collapsible branches. Numeric consequences
   from Rust. 320/400/1440 purchase, locked/fork/refused/away/respec/reload tests,
   keyboard/tap tooltips and no overflow. Real earned desktop/phone screenshots.
5. Whole/sliced/reloaded8h for one/three bloodlines with new effects, exact u64
   state/report comparison; tier0 old-root gameplay hash unchanged. Equal-work
   native overhead target<=10% versus pre-tree source, report absolute timings;
   no per-tick allocation or unbounded perk history. Routine required checks,
   current18-case gate distinct from optional exhaustive balance audit.

Do not expose selectable but inactive perks. Partial core/UI checkpoints are
not completion of this contract. Push approved changes; do not deploy.

## Implementation evidence — 2026-10-06

The nine new effects are snapshotted into an omitted-when-zero Hero bitmask.
Live history and reload retain it; later camp allocations cannot change it.
Clear lungs uses source tags, including gas bursts/Bloat attacks; Fireward also
uses actual fire tags. Potion healing rounds down after the existing modifiers.
All price, prerequisite, fork, effect, refund and eligibility metadata comes
from Rust. The worker proxy exposes respec as well as native/WASM adapters.
The review shows refund and resulting total; failure stays visible. Three
desktop columns and collapsible phone paths use the existing chunky UI/icons.

Thirteen focused core regressions pass: catalogue/prices/depth/history, six
fork directions, checked exact refusals, complete refund/rebuild, selected
slots1/2/3 isolation, old nine-rank migration, nine actual effects, send/history
snapshot, authoritative respec preview, and whole/30min/uneven/reloaded8h.
The latter compares entire raw saves and complete reports for one/three slots;
controlled fixtures are test scaffolding, not earned progression claims.
Final quick:616 core passed/one existing ignored, fourteen tool tests,
typecheck and copy-lint1678 tagged literals, no violations;143s with competing
gate/compile work. All-target fast clippy passes. Browser tree and real-worker
checks pass; existing hero/report/bloodline training checks pass at320/400/1440.

Final fixed earned preparation, synthetic seed/Tier5 eligibility, actual paid
108-point builds from owned814 points, no grants/new policy/floor bypass:

| Seed | Mending | Venom | Fireward |
| --- | --- | --- | --- |
| 1 | 8h /11 runs | 8h /12 runs | 8h /8 runs |
| 3 | 8h /11 runs | 8h /11 runs | 16h /15 runs |
| 5 | 8h /11 runs | 16h /18 runs | 8h /13 runs |

All nine clearD34/unlockTier6 within48h. These are controlled comparisons,
not independently earned campaigns or proof each leaf fired in that policy.
Actual distinct effects are established by paired action/damage regressions.
Earlier pre-gas-tag-correction measurements are retained separately.
Diagnostic: examples/legacy_check.rs; artifacts
scratchpad/legacy-balance-final-20261006/ and /tmp/riddle-cut32-final-balance.log.

Three chosen builds have exact native/WASM full lineage/send/advance/live-reload/
respec/root-purchase parity. Six old full load/advance/after baselines plus two
numbered native saves remain byte-exact. Headed real-WASM screenshots use an
actual earned Tier5 start:320/400/1440,12 nodes/icons, read-only opening retains
the entire raw save, no errors/overflow. Actual paid57-point Mending selection,
read-only review, UI refund, durable save and real page reload verified; the
UI-refunded full save equals the native respec result. Artifacts
scratchpad/legacy-ui-20261006/. Fast WASM5336294 bytes, no deployment.

Final routine18-case gate terminal0, source70342eb7d0cd8298/no cached legs:
metrics111.4s, wire10 seeds76.4s, dayplayer202.4s, all selected bars pass.
Logs /tmp/riddle-cut32-final-gates-full.log. The earlier pre-gas-tag runtime
also passed; its result is not substituted for this final-source check.
The exhaustive statistical/system-removal/historical audit remains separate.

Equal-work native comparison against44da245: nine alternating old/new rounds,
101 samples of500 ticks each,32 stunned enemies plus one actual10-damage gas
call/tick. Old/new median ns/tick: tier0 no perks645.318/626.362; tier5 no perks
683.922/655.938; tier5 selected perks677.262/667.918. All remain within the10%
overhead bar; small negative deltas are not a general speedup claim. Benchmark
scope is scheduler/status and gas protection, not full catch-up or FPS. Initial
5000-tick fixture failed the32-enemy assertion (41 after natural pressure
reinforcements); no timing accepted. Shortened to500 ticks before pressure,
then recompiled identical example against both sources. Every sample verifies
32 enemies/live run. No per-tick perk allocation. Example legacy_perf.rs;
scratchpad/legacy-ui-20261006/perf.json. Final build/typecheck/copy/codegen/diff
and all-target fast clippy pass; existing bundle-size advisory unchanged.
This scoped contract is implemented and verified; class specializations remain
next, with broader balance/complete catch-up profiling independent work.
