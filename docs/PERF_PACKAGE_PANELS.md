# Equivalent package panels — 2026-10-05

The current package query clones and simulates every offered move separately.
Rust forecast keys identify16/14,38/24 and17/16 requested/unique own-or-wall
panels in the frozen early/late/tuned camps. Count evidence is private under
scratchpad/flood-perf-20261005/panel-keys*. Exact-key duplication is an opportunity,
not a measured speedup.

Prototype within one package query: group moves by both own and wall panel keys,
plus the raw rules key to preserve passage policy inputs. Compute representative
panels once; retain every original move, its own price and stable score ordering.
Reuse base results when keys agree. No persistent cache, budgets, seeds or wire
changes. Keep existing native worker scheduling semantics; if grouping would
collapse a threaded query to one job, use the original path rather than switch
its simulations to a different parallel-budget execution path.

Screen with three alternating pairs, then seven if promising. Require at least
15% lower late-package median,2% lower tuned median, no workload regression over
5%; exact complete JSON outputs on all three camps and unchanged simulation
budgets. Shipping-WASM late query target10% lower. Broaden regression checks only
if the screen passes. Do not ship a merely lower allocation/work count.

Native acceptance so far: seven alternating package pairs on each frozen camp,
plus three offline pairs. Package medians early2.543→2.202s (13.39%),
late12.869→8.201s (36.27%), tuned10.286→9.443s (8.20%). All60 complete JSON
files agree pairwise. Offline medians remain within1.1%; this does not claim a
catch-up speedup. Late work1461→911 simulations,14,009,564→9,000,482 ticks,
38→24 panel misses; each representative retains the original ordered budget.
These are single-thread native query timings, with fresh games and no concurrent
browser/build activity. Native24-sim comparisons with2/8 workers and cold/warm
base caches also match every returned field on all three camps (12 paired cases).
Regression test retains both tactic choices, neutral level prices, unchanged
save and warm-cache ordering. Two additional cold/warm comparisons of exactly
two equivalent moves with2 workers pass the single-job fallback against the
preserved executables (the old rlib has since been rebuilt; do not relink it).

Shipping-WASM acceptance: seven alternating browser pairs per camp after one
discarded warmup pair; fresh Game per query,50 sims, load/save outside timing.
Early3.581→3.149s (12.07%),late17.884→11.242s (37.14%),
tuned14.540→13.371s (8.04%). All24 paired cases have exact price/save strings matching the
old shipping engine, and each complete price vector matches native output.
No compiler or other harness ran alongside timings. This measures package-query
CPU/wall time, not rendering or whole-app latency.

FULL460s PASS:537 core/1ignored,11 tooling, tsc/copy/clippy/shipping site;
fresh metrics108.6s, QA44.0s and all18 current-player fortnight cases pass.
Scoped client38 request checks+17 package checks pass9.8s. Real native10 and
shipping-WASM10 checks400/1440 pass through visible controls, including actual
Legacy purchase remeasurement; both complete real-UI price vectors match.
The58 complete bridge replies for Legacy/multihero/offline/migration match
native/shipping exactly. All14 dayplayer recovery tests pass58.24s. Publication
checkpoint complete: source2b48e00, Pages37301872292 SUCCESS. Public HTML,
entry JavaScript and WASM match CI byte for byte. Public10 real-controls checks
400/1440 pass with no page errors; complete price vectors and upgrade results
match native. Both public screenshots shown inline. WASM SHA256:
5c8ba5dffc017976016f7114344e53c4a56024669147ac0bcaeae043398f6d2a.
Evidence:public-proof/proof.json,public-ui/proof.json,public-ui-parity.json.

First local FULL attempts failed before tests with a compiler SIGSEGV and then
undefined hidden linker symbols. Quarantining one incremental directory was
insufficient. A scoped `cargo clean -p riddle-core --profile fast` repaired the
generated core object cache; unchanged source/flags then passed FULL. Dependency
caches, native-dev target and shipping artifacts were preserved. Do not infer
the failure's root cause from hardlinked incremental objects alone. Keep frozen
baseline executables outside Cargo target directories before any such cleanup.
Evidence:scratchpad/flood-perf-20261005/panel-reuse; never stage private files.

Requested server cleanup:49 stale Riddle dev/preview/QA listeners stopped by
verified PID/cwd, including5365; all stopped ports confirmed closed. Cleanup
targeted91 server/wrapper/child processes with SIGTERM. Keep5219(main),5594(current
native) and5264(shipping preview). Other projects and Codex processes were
excluded. Evidence:server-cleanup.json under scratchpad/flood-perf-20261005.
