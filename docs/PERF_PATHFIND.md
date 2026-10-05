# Pathfinding follow-through — 2026-10-05

Current named fast-WASM package profile on the frozen late camp:42,848 samples,
hero_action8.11%, vision7.25%, nearest_tile5.86%, flood_resume4.18%. Three fresh
searches return the same complete-report/save hashes; report SHA256 also matches
the native compiler experiment. Sample shares are profiling leads, not shipping
latency measurements. Evidence:scratchpad/sim-profile-20261005/late-packages.

Probe specializing the nearest-tile predicate instead of passing it through a
trait object. Preserve search order, tie breaks, seen/avoid behavior, RNG, all
simulation budgets and complete outputs. Isolated source only until accepted.
Compare unchanged opt3/CGU16/debug0 baseline with candidate: three alternating
pairs on early/late/tuned package reads and offline8h. If promising, extend
late/tuned package reads to seven alternating pairs. Retain only if both
expensive reads improve at least2% median, no representative offline regression
over5%, and every complete output matches the frozen reference byte-for-byte.
No allocation or simulation-count reductions are presumed.

Any retained change requires existing pathfinding/engine/recovery tests, routine
FULL gates, shipping/native parity and public acceptance. Reject inconsistent
results. Do not revive the rejected clipped-vision or stack-row probes as wins.

Outcome: REJECTED after the three-pair screen. Late packages12.1825→11.9355s
(2.03% lower), tuned9.7288→9.6814s (0.49%), early2.4167→2.3946s (0.92%).
Offline early/late/tuned0.5220→0.5088s,1.8781→1.7941s,1.5598→1.5875s.
All36 full output files match the frozen reference bytes. Tuned reads do not
meet2%; do not extend an unpromising screen or retain the source change.
These small samples are not a quantified shipping improvement. Production
code is untouched; isolated source is restored and checked byte-for-byte.
Private patch, executable, raw outputs and timings are retained under
scratchpad/sim-profile-20261005/path-probe (log in its parent directory).

Next: quantify repeated flood/allocation work before considering scratch-buffer
reuse or cache changes. Prove invalidation for hero position, seen tiles, terrain,
obstacles, mirrors, sleepers, monsters and overlays; retain exact tie breaks.

Follow-through candidate: parent-only stopped floods. Nearest-target and
parent-to queries discard their distance vector today; use the already-required
parent vector for visited state and queue boundaries for BFS distance layers.
Preserve DIRS8 discovery order, full-layer stop calls, unreachable/root parent
values and every returned parent. Keep distance-producing APIs unchanged.
Before testing, count calls/cells/expansions and allocation traffic in an isolated
instrumented build; do not report its runtime as an ordinary benchmark.
Use the same three-pair screen/seven-pair expensive-read thresholds above,
plus fewer allocations/requested bytes. Oracle comparisons must cover narrow
maps, out-of-bounds starts/goals, unseen tiles, dynamic obstacles, disconnected
regions, corners and all parent entries, including the root marker.

Parent-only stopped-flood outcome: REJECTED at three-pair screen. Diagnostic
late read904,751 stopped parent queries (807,691 nearest,97,060 parent-to),
926,465,024 cells in their discarded distance arrays. Whole-process allocation
traffic50,152,729→49,247,978 calls;20,474,285,702→16,768,425,608 cumulative
requested bytes (18.10% lower; not peak live memory). Package medians early
2.3714→2.3730s, late11.9345→11.8079s (1.06%), tuned9.7253→9.5973s (1.32%).
Offline medians0.5080→0.5040s,1.8094→1.8019s,1.5556→1.5238s. All36 complete
outputs and both allocation-diagnostic outputs match the frozen reference.
The expensive reads miss2%; allocation reduction alone is insufficient.
Private evidence:scratchpad/flood-perf-20261005. Production remains unchanged;
isolated source restored. The draft exhaustive parent oracle is not an accepted
production test and was not needed to retain this rejected candidate.

Next isolated probe: nearest-target plus first-step result, since nearest_tile's
callers immediately reduce the returned parent map to a single step. Use bounded
stack visited bits/queue for up to1024 cells, with the existing heap/parent API
as fallback for larger maps. Keep target, layer and DIRS8 order and avoidance
fallbacks unchanged. Require the same timing/output/allocation thresholds.
Oracle must compare target/first step to existing bfs_nearest+first_step across
small/narrow/blocked/unseen/out-of-bounds maps and the large-map fallback.

First-step stack probe: REJECTED at three-pair screen. Package medians early
2.4247→2.3067s (4.87%), late11.9283→11.8042s (1.04%), tuned9.8998→9.5523s
(3.51%). Offline0.4882→0.4840s,1.8100→1.7769s,1.5451→1.5710s. All36 complete
outputs match. Allocation traffic47,729,656 calls/7,241,076,362 requested bytes,
64.63% fewer cumulative requested bytes than baseline; late runtime misses2%.
Production remains unchanged. Private evidence and rejected patch:
scratchpad/flood-perf-20261005/first-step. Draft oracle was not run/retained.
