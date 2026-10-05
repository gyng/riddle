# Record the scout's automatic sends — 2026-10-06

The scout enables uncapped automatic runs but never increments tree.acts;
report_acts cannot announce his first contribution. Add bookkeeping only at
actual automatic clock start sites (offline batches and open-app advance).
Count one per new run; manual send/resume/preview/forecast/sim/literal lineage,
paused or unhired scout do not count. Keep all timing/RNG/run rules unchanged.
Worker wire uses existing schema: scout, sent N, n N, first from persisted
zero→positive ledger. Later report is routine; load retains first-act history.

Numeric checks: automatic count equals new run ids in absence/open-app clock,
manual/resume zero, paused/sim/literal zero, first once across reload, helper
changes only tree.acts. Run fast workspace tests, build WASM, scoped UI/build/
copy, real-WASM mobile/desktop first and later report screenshots. No balance
tuning, new automatic behavior or deployment. No exhaustive audit claim.

Implementation: tree::scout_sent gates hired/on/non-sim/non-literal and updates
only existing acts map. Called after automatic start_run in offline main loop,
legacy sampling loop and live advance. Manual send/ensure_run/preview never
call it. Counts actual created runs, not statistically extrapolated records.
No schema/save migration; existing report_acts formats sent N.

Verification: fast workspace548 tests pass33.21s (1 pre-existing ignored);
quick host/typecheck/copy checks green72s with host access. Initial sandboxed
quick pass had native-host integration failure; rerun host test and complete
quick suite pass, no test weakened. Two new Rust cases cover automatic ids,
manual/resume/paused/unhired/sim/literal, reload first flag and full serialized
game equality after stripping acts. Clippy clean. Rebuilt actual WASM
5,218,820 bytes; web build/typecheck passes, existing chunk advisory. Existing
worker-first-act60 UI checks pass; diff/copy1503 clean.

Headed actual WASM400/1440 legal seed2 preparation: first8h report16 runs,
core scout sent16 first=true and porter hauled$1287 first=true both visible
with Details closed. Later8h scout sent16 first=false, no first announcement.
Zero warnings/overflow. Screens/evidence scratchpad/scout-acts-20261006/
*-first.png,*-real.json. No deployment or full statistical balance audit.
