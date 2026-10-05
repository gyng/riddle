# Give each heir a name — 2026-10-06

Active roster currently identifies slots (Bloodline1) and numbers (2nd heir),
not people. Give current/past heirs deterministic short fantasy names owned
by Rust, derived from lineage seed/heir ordinal without consuming RNG.
Names stay constant through XP/class/action/slot selection/save-load changes;
the next heir has a different name. Bloodline identity/Legacy remain separate.

Add optional/default-compatible names to wire only: hero slot current name and
hero lifetime records. Decorate cloned lifetime records for UI; raw save and
simulation state remain byte-identical on name reads. Keep fixed name pool;
no gameplay/stat/seed/gold/upgrades or new active slots. Existing old/fake wire
renders its ordinal fallback. Roster shows name first, bloodline/class beneath;
hero sheet and selected bloodline Chronicle show same core names. Keep raw
chronicle lines for death lookup/navigation; other slot sections retain current
raw history until selected. No rename feature or portraits promised here.

Check deterministic/read-only/migration/new-heir/slot-selection behavior in Rust,
roster/menu/chronicle/fallback/longest-name geometry on320/400/1440, actual
real-WASM named heroes/screens after reload. Fast workspace tests, rebuilt WASM,
client/typecheck/build/copy/diff. No deployment or full balance audit.

Implemented fixed24-name pool with pure splitmix(seed) offset + heir ordinal;
consecutive heirs differ, names cycle after24 heirs and may coincide across
bloodlines (slot label always distinguishes them). HeroLegacy.name is omitted
when empty in underlying save; cloned wire records get names. HeroSlot adds
current hero_name. Old wire fallback preserved. Roster repaint key includes
names; initial UI check caught missing invalidation and the fix passes.

Verification:549 fast workspace tests pass37.62s (1 pre-existing ignored),
12 tool checks/typecheck/copy1503 green; entire quick125s including compile,
not a timing improvement claim. Clippy/build and rebuilt WASM pass; existing
chunk advisory.27 hero identity UI checks320/400/1440 pass9.7s, including
roster/menu/core-provided past name/raw kept-death lookup/old-wire fallback/
Maren(max5 letters) geometry. Existing report-upgrade57 checks pass6.2s.

Headed real-WASM400/1440 legal seed2 manual house/runs/shared-gold second
bloodline: Wren and Vale names match after exact raw-save reload and selecting
2→1. Both active names shown, correct Wren fighter L1 hero menu, no warnings/
overflow. Real screenshot evidence covers active heroes; historical name and
death-link display covered by UI fixture/Rust heir-history read test, not a new
full real-game death walk. No portrait variants/custom naming/topbar overhaul,
new slots, runtime progress changes/full balance audit/deployment.
Artifacts scratchpad/hero-names-20261006/{400,1440}-roster.png,*-hero.png,
*-real.json,quick.log,ui-final.log,build.log,wasm.log,clippy.log.

Forecast investigation: current camp_panel_priced already reuses exact ordered
prefixes across sample sizes/budgets; no repeated-prefix simulation removal
found in this read. Prior cold-query profile(PERF_COLD_TACTICS.md) attributed
setup1.53%/clone0.41%; no new profile or preparation performance claim. Follow
owner's diminishing-return direction toward visible hero/content personality.
