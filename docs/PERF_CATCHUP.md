# Catch-up throughput — 2026-10-03

Owner requested faster offline tick processing after the local iteration pass.
Keep the 10 ticks/second progression rate, policies, elapsed-time accounting and
current sampling behavior unchanged. This is engineering work, not Cut 31.

First screen: compare shipping WASM optimization settings using the exact same
saved camp and generated bridge. Preserve the original shipping artifact. Run
seven alternating pairs, exclude module loading/compilation from simulation time,
and require identical report and saved-state bytes. Target at least 15% faster
median catch-up. Record misses and reject unproven settings. Validate any retained
setting in a real browser as well as Node; distinguish throughput from total cold
startup and artifact size. Do not assume native results transfer to WASM.

If an engine implementation change is necessary, profile the remaining tick hot
paths first. Require deterministic continuation and the existing complete gates
before landing; never reduce seeds, assertions or simulated game time for speed.

Private benchmark inputs, binaries and raw measurements:
`scratchpad/catchup-20261003/`.

## Profiling and rejected screens

Captured the actual shipping module with Chromium's CPU profiler on the headed
browser: 5534 samples, three eight-hour catch-ups taking 1.804–1.878 s. Its symbols
are stripped, so this profile records WASM function numbers. A separate symbolized
build captured 5620 samples and preserved the exact report/save hashes. Its code
sections differ from shipping, so names were not falsely mapped onto the shipping
profile. The symbolized profile attributes 22.38% of sampled leaf time to
`Game::tick_inner` (including inlined work), 6.86% to `Map::update_vision`, 6.53% to
`ai::nearest_tile`, 3.67% to `Map::flood_resume` and 4.40% to WASM allocation.
Samples include save loading and final serialization; module compilation and a
warm-up batch were excluded. This is one retained late-game camp, not all seeds,
and the engine call alone, not the application's complete chunked absence path.

Compiler screens, seven alternating Node/WASM pairs, exact reports/saves:
re-optimizing the shipping module at O3 gave 2.010732 → 2.009557 s (no meaningful
gain); optimizing the original bindgen module at O3 gave 1.810099 → 1.826169 s.
Both were rejected. Their first-load times are single observations and are not
startup-performance evidence.

The vision prototype encapsulates map tile writes, invalidates cached visibility
when sight-blocking walls change, and skips the wall scan when the cached position,
radius, dimensions and walls remain valid. It passes a direct-LOS oracle over wall
changes, movement, radius changes and save/load. Seven native pairs against the
rebuilt canonical baseline gave 1.479908 → 1.423476 s (3.81%); seven Node/WASM pairs
gave 1.881774 → 1.868519 s (0.70%). All output hashes match. Initial native pairs
used a retained shared-target executable whose source provenance was insufficient;
those were superseded by the rebuilt canonical baseline. No engine prototype has
entered production, and the 15% throughput target has not been met.

Final headed Chromium comparison, seven alternating pairs with compilation and
one warm-up batch excluded: shipping 1.900100 → candidate 1.825000 s (3.95%).
Reports and saves match exactly. The candidate is rejected for this pass: a small
single-camp gain does not meet the 15% target or justify promoting the map API
change without broader performance evidence. Preserve the private patch and
profile; do not describe the public game as faster from this experiment.

`tools/profile-catchup.mjs SAVE --out DIR` now records the actual selected WASM
module's CPU profile and grouped leaf samples, verifies repeated report/save
hashes, and records input/module/bridge/harness hashes and browser version.
`--pkg DIR`, `--hours N` and `--runs N` select other builds/workloads. Open
`catchup.cpuprofile` in Chrome DevTools. It profiles the engine call, not the UI
or chunk scheduler; a stripped shipping module shows function numbers. A real
shipping run verified the tool. The production benchmark executable was restored
after the private probes; source and public game behavior stayed unchanged.
