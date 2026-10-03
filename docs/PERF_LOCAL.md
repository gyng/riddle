# Local performance pass — 2026-10-03

Owner requested optimization after Cut 30.5 publication, with local iteration and
runtime ahead of CI work. This pass changes implementation costs, not Cut 31 content.

Baseline: deployed `df30c53`, core `2822afe224cb289b`. Local quick verification
took 105.87 s including checkout warm-up, then 34.77 s without an edit; 517 tests
pass with one ignored. Native seed-1 single-threaded 8 h simulation took 0.45 s,
17 verdicts 0.93 s, forecast 0.24 s, refined forecast 0.24 s, final death 0.20 s.
These shallow native values are not late-game or browser/WASM measurements.

1. Add a reusable saved-camp benchmark for offline catch-up and history snapshot
   fields, with source/input provenance, repeated samples and output fingerprints.
2. Measure a retained late-game camp before choosing a history-storage or tick-allocation change.
   Keep the baseline binary and input unchanged for paired comparisons.
3. Target at least 10% lower median catch-up time over seven paired samples;
   reject changes without a repeatable gain or with changed output. A target miss
   must be recorded explicitly with the reason for any retained smaller change.
4. Preserve exact reports, save/load continuation, death traces, deterministic
   fingerprints and every existing gate/sample/bar. Run focused behavioral checks
   during iteration, required full verification before landing a core change,
   rebuild shipping WASM and capture a public checkpoint after any deployment.

Native and WASM results are reported separately. Benchmark samples exclude input
loading/builds; total iteration measurements include them and state cache conditions.
Private inputs, baseline binaries and raw results stay under `scratchpad/perf-20261003/`.
Owner review, cohort and Cut 31 remain on hold.

First retained camp: input FNV `ca07c0cb12307442`, IDLE seed 15 day 5,
loaded through current save migration; 188541 ticks per 8 h and exact output
`cdac0a2e82099f2f`. Seven samples: median 1.440185 s. D19 run clone 2.306 us,
monsters 0.526 us, map 0.075 us, meters 0.186 us. Map/meter sharing is not the
first candidate. Test removing the unconditional per-live-tick hero-healing Vec
and collecting post-tick ally ids directly through an iterator. Keep pre-tick
pet HP snapshots and event order exactly as before. `samply` is unavailable under
the current kernel profiling restriction; no system settings were changed.

The healing-array candidate preserves the output and removes 188541 allocation
requests (one per live tick); seven alternating native pairs improved median
1.490069 → 1.413008 s (5.17%), below the pass target alone. An isolated userspace
`pprof` probe succeeds without kernel changes. Of 4322 samples, 287 leaf samples
are `Map::in_bounds` under `update_vision`. The next candidate iterates the clipped
vision rectangle using the original square-mask indices, omitting bounds checks
and mask work for out-of-bounds endpoints. No vision memo or tile-write contract
changes. Check it against direct Bresenham LOS across small maps, boundaries,
radius zero and a changing wall, then compare combined performance with baseline.

The clipped-vision prototype passed its independent LOS checks but seven paired
samples showed no added benefit (combined 4.46% versus healing-only 5.17%); it was
removed along with its prototype-only boundary check. The profile attributes 241 leaf
string-comparison samples to repeated verb comparisons in the row-shadow search.
Next candidate: an unsaved, shared run memo of active rows and structural shadow
relations, validated against row content, origin, authored-row limit and lent row.
Check condition availability at each row attempt, including facts learned by
earlier attempts in the same action. Cache state must not affect save JSON or
game equality, and run-history clones must share it.

The structural rule-plan prototype preserved benchmark outcomes but improved
combined median only 4.23%; input-validation costs erased its expected gain.
It and its associated tests were removed. Both rejected prototype patches and
raw paired samples are retained privately. No vision or rule memo change remains.

**Recorded target deviation:** the first local pass retains only the healing-array
and ally-iterator change, pending full verification and shipping-WASM comparison.
Native improvement is modest, not the planned 10%: the first seven pairs gave
5.17%; a later quiet repeat gave 1.82% by ratio of medians (six of seven pairs
faster). The exactly eliminated
per-live-tick allocation, small scope and repeated identical report/save results
justify measuring and validating this smaller improvement instead of retaining
unproven caches. Existing game gates, bars and sample sizes are unchanged.

Shipping-WASM seven alternating pairs: median 1.776748 → 1.786235 s, within
variation; no browser runtime speedup is claimed. All reports and saves are
byte-identical. Native eight-seed DEFAULT/FULL × 8 h fingerprint is unchanged:
`35cb82317410f64e`. Final shipping WASM SHA256
`5c1b05d077df71c0d166e1ed46df051b68e20afb260611d4907eb2e7222dde5d`.

Local QA improvement: `tools/playtest.mjs --url <preview-or-public-url>` now walks
the selected built app directly and writes `checkpoints.json` with labeled PNG/text
pairs, target URL, tool-checkout commit and fetched WASM hashes. It does not start
or rely on a Vite server for an explicit URL. The headed public walk passed eight
captures in 32.6 s, covering manual send, watch, report, scout hire and 8 h return;
two existing WebGL warnings were observed, with no console/page errors. The reusable
`tools/perf-local.mjs` compares unchanged native binaries in alternating order,
records binary/input hashes, and fails immediately if outcome fingerprints differ.

Next local iteration change: the unchanged shipping build in full verification
still spent 88 s in wasm-pack/bindgen/wasm-opt after Cargo finished in 0.03 s.
Keep Cargo's release build check on every call, then permit packaging reuse only
when the actual compiled WASM, packaging tools, recipe, crate metadata and build
environment digest match, and every generated package file matches its hash.
An absent/invalid stamp or changed output must rebuild; `--ship --fresh` forces
packaging. Create stamps only after a successful real build. Measure first build
and repeated hits separately, and check output corruption and changed environment
invalidate reuse. Do not change game gate sampling or treat fast WASM as shipping.

The genuine cache-prime shipping build took 153.94 s while the full dayplayer was
running. The following checked no-edit build took **1.88 s** (Cargo plus input,
tool and output hashing). The earlier unchanged packaging run took 88 s under
different load; these are phase measurements, not a paired whole-pipeline result.
Cache negative checks passed for a changed generated declaration, changed build
environment and corrupt stamp; exact restoration restored a hit. No retrospective
stamp was created for an unverified package.

Owner clarified that the painful tuning wait is the developer simulation gates.
`tools/tune.sh <rows>` wraps the existing targeted dayplayer with full seed counts
and fail-fast by default; `--list` lists the rows. Fresh `idle-d8` passed 16/16 seeds
in **6.12 s** under concurrent full-suite load. This is one milestone row, not a
replacement for the 272-case full gate. Complex fortnight/ratio rows take longer.
The wrapper preserves each row's exact bot/milestone requirements and labels the
result as targeted. Its repeated same-input invocation reuses genuine evidence.

Three alternating fresh `idle-d8` runs at 24 versus 12 workers screened contention:
median wall 8.883 versus 9.023 s; median CPU 39.460 versus 34.901 s. Twelve workers
reduced CPU but did not improve wall time. The full dayplayer ran concurrently;
this narrow screen does not justify changing any suite-wide worker default.

`tools/gates.mjs` now saves each completed leg immediately, before awaiting the
longest leg. It prints source key, worker widths, cache hits, completion timings
and a 30-second pending-leg update. An isolated process-interruption fixture
verified metrics/QA remain saved while dayplayer is pending and are reused after
restart. The fixture is orchestration testing, not game gate evidence. Binary,
source and runtime-input keys, samples, assertions and pass criteria are unchanged.
Completed-leg and targeted printout writes use atomic rename; corrupt older
printouts are misses. `node tools/gates-cache-test.mjs` repeats the interruption,
restart/reuse and corrupt-cache checks in an isolated temporary fixture.

Additional screens (both under concurrent full-dayplayer load):

- All workspace fast tests, three alternating pairs at 24 versus 8 test threads:
  517 passed/one ignored each run; median wall 82.961 versus 83.351 s, CPU 952.988
  versus 954.401 s. No default was changed.
- Actual TUNED seed 1 first day, fresh, one simulation thread, unchanged retained
  baseline versus healing-allocation candidate: three alternating pairs, median
  CPU 38.730 versus 35.967 s (7.13% lower); all three candidate CPU samples lower.
  Printed outcomes are identical after removing the duration. Median wall 44.337
  versus 37.707 s is affected by the concurrent test screen finishing, so it is
  not a quiet full-gate speedup claim. This native forecast-heavy sample complements
  the saved-camp benchmark; it does not establish a browser/WASM gain.

The shipped tooling commit `cfed378` passed public CI run `37125173952`; real
public-app WASM/layout/service-worker/offline checks passed. A second headed walk
verified eight screenshot/text checkpoints and the harness SHA256 field. It took
81.5 s under active full-gate load versus the earlier 32.6 s walk; these QA walks
are functional checks, not paired performance measurements. The core candidate
still awaits its complete fresh 272-case verification before landing.

The isolated TUNED profile produced 7643 filtered samples. `each_step` appeared
under BFS/flood paths in 1157 samples; vision and rule evaluation were also hot.
Library allocations are blocklisted by this profiler, so these are not complete
process CPU percentages. The first-day picker used 25–30 CPU s versus 2–3 s offline.
A private core copy tested retaining package candidates' panel caches, borrowing
the cage picker's pattern. Three paired runs preserved printed outcomes but had
no repeatable gain: median CPU 43.446 versus 44.528 s, with one candidate pair
about 19% slower. Rejected; no package-cache code entered production. The patch,
input/binary hashes and samples remain private. Only that probe package's build
artifacts were intended for cleanup after measurements. Cargo's package cleanup
matched the shared `riddle_core` artifact family more broadly than intended,
removing 30.6 GiB of fast-profile artifacts. Source, gate-result caches, existing
example executables and shipping WASM remained intact; the running gate continued.
Rebuild the affected workspace fast library before final verification. Do not use
package cleanup for renamed probes sharing a production library target name.

Final acceptance: fresh `tools/verify.sh --full` passed in **4679 s**, all 272
fortnight cases and every existing bar. `cargo test --profile fast --example
dayplayer` passed 11 tests. After rebuilding affected local artifacts, canonical
full verification passed again in **111 s**, with all three genuine leg printouts
cached, 517 tests/one ignored, TypeScript/copy lint and all-target Clippy. Current
core source key is `d84ba387fbdb62da`; shipping WASM hash remains the candidate
hash above. The 111 s includes rebuilding test artifacts and is not a fully warm
unit-test build measurement. Full-gate cold cost remains about 78 minutes in this
run; the iteration gains are targeted checks and correct unchanged-work reuse,
not a claim that all fresh simulations now finish in seconds.
