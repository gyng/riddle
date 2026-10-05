# Active bloodline portraits honor chosen appearance — 2026-10-06

Existing class×male/female/cat portraits and appearance picker already exist.
Topbar/hero menu use selected Lineage.look; HeroSlot lacks it, so roster uses
class default for every slot. Report Upgrade hero also ignores selected look.
Expose each slot's Rust-resolved cosmetic look (chosen else Class.default_look),
render it with existing paintFace and include look in repaint keys. No new art,
default appearance, RNG, stat, upgrade or simulation changes. Old wire falls
back to existing class art; missing assets keep existing fallback chain.

Numeric checks: three looks resolve to corresponding existing portraits;
two bloodline appearances remain independent across selection/save-load;
current roster/report repaint on look changes even if class/name/points don't.
Core rejects invalid appearance; frontend unknown asset falls back. Verify
320/400/1440 layout and actual two-look real-WASM screenshots. Quick core/tool/
type checks, scoped client/build/clippy, rebuilt WASM/copy/diff. No deployment
or full balance audit.

Implemented HeroSlot.look (default-compatible wire) from that game's chosen
look/default class appearance; roster paintFace and repaint key use it.
Report upgrade paintFace/repaint key use selected Lineage.look. Existing
selected menu/topbar/render look handling stays as before.

Verification:550 fast Rust tests35.95s,1 pre-existing ignored;12 tooling/type/
copy checks, quick107s includes compilation. Clippy/web build and rebuilt real
WASM5,221,684 bytes pass; existing chunk advisory.54 appearance UI checks
320/400/1440 cover all12 asset ids, old/missing-art fallbacks, appearance-only
report updates/no overflow; hero names27/report-upgrade57 regression checks
pass in5.2s batch. No new simulation settings/time gain claim.

Headed real-WASM seed2 legal house/manual runs/second slot: chosen female
Wren/cat Vale via existing engine.setLook API, both slots survive exact save
reload and2→1 selection.400 and1440 active roster screenshots show distinct
correct portraits, Wren menu hero_fighter_female, no warnings/overflow.
This validates chosen appearances, not automatic variety or an appearance
picker UI walk. Existing portrait picker uses openLooks; camp hides console
portrait, so next inspect convenient entry from hero details before adding art.
Screens/evidence scratchpad/hero-slot-looks-20261006/{400,1440}-roster.png,
*-hero.png,*-real.json,ui.log,quick.log,wasm.log,build.log,clippy.log.
No new art/default appearance/save migration/portrait generation or deployment;
no full balance audit. Cosmetic getter does not alter gameplay inputs.
