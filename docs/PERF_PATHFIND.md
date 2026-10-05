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
