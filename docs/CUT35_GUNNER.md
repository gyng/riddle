# Cut 35 — Gunner, long guns and short guns

Owner request2026-10-07: add a Gunner class, with long and short guns, distinct
from archery. This becomes the next class priority, before proposed Rogue/
Ranger specializations. Finish Cut34's live verification without abandoning it.
Status: contract and two transparent equipment icons prepared; Gunner gameplay
is not implemented or selectable yet. No deployment.

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
rules, before generic attacks. Default Gunner works from its first send without
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

Architecture audit starting points: hero.rs Class::ALL/ladder/starting_weapon,
meta.rs class unlocks, defs.rs/item.rs weapons and drop weights, ai.rs ranged
hits/reflection/target memory, packages.rs generated policy precedence,
turn.rs reload scheduling/minbatch, wire.rs HeroSnap, class offers/forge/presence,
render/look.ts manifests and ui/skin.ts icons. Current ranged ItemDef boolean
cannot alone encode chambers, gun range or reload commitment; introduce a
bounded firearm profile/state rather than inferring all ranged weapons as guns.

## Art checkpoint — 2026-10-07

Built-in imagegen, separate long/short prompts, actual alpha. Source
art/ui/icons/it_long_gun.png and it_short_gun.png; existing ui-skin.py packs
96px siblings into web/public/ui/icons and skin.json.115icons total, old assets
unchanged. Long slender rifle versus squat double-barrel silhouette inspected.
Exact prompts/provenance art/prompts/gunner-equipment-20261007.md. These icons
do not expose Gunner or firearm items before gameplay exists. Next A/B core
implementation; portrait/sprites and action icons remain part of later D.
