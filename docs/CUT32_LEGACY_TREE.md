# Cut 32 — branching bloodline Legacy

Implementation contract,2026-10-06. Owner asks deeper Legacy trees and class
progression after numbered ascensions. This is the next runtime cut; none of
the new nodes or respec below are implemented yet. Class specializations follow
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
