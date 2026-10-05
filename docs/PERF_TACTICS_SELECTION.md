# Tactics comparisons for visible choices — 2026-10-06

Measure the cost of the current all-moves comparison, then expose a read-only
Rust query for the exact equip choices shown by opened combat-style/tactic
choosers. Do not simulate hidden groups, purchased level changes or unrelated
tactic slots. Keep the same eight samples, seeds, probability calculations and
sort order. Existing all-moves API remains for gates and older clients.

Numeric acceptance: selected answers match filtering the complete query on the
same state; no engine/save changes or subsequent offline-result differences.
Empty selection performs zero panels. Cache keys include selected choices;
background mirror/cache ownership remains correct. Opening the panel performs
zero simulations. Changing selected slot/group clears mismatched answers; late
replies cannot paint another selection. Explicit retries, no automatic query
loop from clock changes. Profile actual WASM all/selected queries and record
whether savings justify promotion. Rust targeted tests + fast workspace tests,
WASM rebuild, tsc/copy/build, selected client lifecycle/lane/geometry gates,
real-engine desktop/mobile screenshots. No deployment.

Owner steering: after this high-impact pass, stop chasing small performance
gains and move to content/gameplay/graphics. Broader gate-speed goal remains
unfinished; do not describe selected-query gains as whole-simulation gains.


Acceptance evidence: release-WASM three alternating full/selected cold-game
pairs in each of three saved camps pass. Full query medians516/2448/2574ms,
selected combat-style medians169/944/455ms:67/61/82% less query time. Selected
moves1/3/3 vs all7/18/16. All nine selected replies exactly match filtering the
complete query; complete saves before/after remain equal and subsequent30min
offline reports/saves match. Artifacts/timings are in
scratchpad/tactics-selection-20261006/. The first early pair overlapped the
functional screenshot walk; medians are reported, not latency guarantees.
This measures combat-style subsets, not whole gameplay speed or every possible
opened group; selecting all groups can cost more.

Rust selection intersects legal equip candidates, deduplicates requests in
canonical order, and uses the original complete query's native execution policy
for matching answers. Empty/invalid selections run no panels. Full query API
and gameplay behavior remain unchanged; the optional WASM adapter falls back
to filtering the old query when bindings lack the new method. Background keys
and answers run on the same unchanged mirror; method identity and choices/
slots belong to the cache key. Native bridge regenerated and checked.

UI: Compare outcomes opens available styles (or tactics) if no chooser is open.
Only opened combat-style and tactic alternatives at the chosen slot are priced.
Hero/rule changes clear answers without auto-querying, late selection replies
cannot paint, explicit retry works and closing releases both subscriptions.
The first graphics follow-up moves estimates below descriptions, avoiding
Bold's narrow text column and keeping cards the same row height. Actual release
WASM400/1440 controls/screens pass and checkpoint screenshots were shown.

546 fast workspace tests pass, one ignored, including fresh selected/full
parity on two seeds and empty-panel/cache assertions. Clippy passes. Client
checks: selected lifecycle28, existing requests82, lane/cache19, geometry176.
Tsc/copy1492/production build and native bridge generator test/check pass;
native example compiles. Release WASM rebuilt. The full eighteen-case progression
suite and exhaustive balance audit were not rerun for this additive read-only
query; this is not a full-gate certification. No deployment.

Next work shifts to player experience: walk the first session, an eight-hour
check-in and a live boss encounter; use those artifacts to choose the highest
impact content/gameplay/graphics gap. Continue the requested chunky target
screens and shared visual module consistency. Further performance work needs
a measured bottleneck that outweighs this player-facing work.
