# Profile current core edit cost — 2026-10-06

Measure current HEAD in an isolated source copy. Prime native_dev fast-profile
artifacts, then two algebraically equivalent vision-body edits and exact source
restores. Capture Cargo HTML timing units and total wall time. Each restored
executable SHA256 must match the primed binary; main worktree stays untouched.
Cold prime is recorded separately and never used as warm-edit speed evidence.
No competing builds while measuring. Do not change optimization settings or
claim a speedup from this diagnostic.

Use the measured critical path to choose the next optimization. If compilation
is already short relative to the fast test execution, investigate test scheduling
with all existing test names/ignored status and failures preserved; no reduced
coverage, case budgets or gates. Persist artifacts/results and practical limits.

Measured source1d76e86, rustc1.96.0, fast opt3/CGU16/debug0/incremental.

| Isolated native build | First pair | Second pair |
| --- | ---: | ---: |
| Equivalent vision edit |10.302s|10.475s|
| Exact restore |10.466s|10.228s|

Edit median10.389s; restore10.347s. Core library dominates: warm frontend
median3.36s, code generation6.475s, example/bridge about0.5s. Cargo unit timing
is not a detailed compiler pass profile. Cold prime54.480s includes dependencies
and is separate. Both restored executable hashes match the primed binary.
Main worktree untouched; no optimization/architecture setting retained or
behavioral equivalence claim based only on restored binary identity.

Complete unchanged-suite runner experiment, sequential with no compiler job:
Cargo workspace fast35.211s; installed Nextest0.9.133/eight workers35.937s;
Nextest16 workers36.622s. Each passes548 tests, one pre-existing ignored/skipped.
This is one run per scheduling setting, not enough to establish small timing
differences. Neither candidate offers demonstrated improvement; keep Cargo.
No tests, examples, documentation tests, counts or check commands removed.

Per-test diagnostics with16 workers (concurrent wall durations, not isolated
CPU attribution): forecast prefix equality29.799s, quiet-tick/single-tick
boundary equality26.473s, frontier renown/night mark24.847s, attack-row death
verdict20.476s, large outcome panels19.629s. This identifies paths to profile;
do not add these overlapping numbers or attribute all suite cost to one case.
Next: profile actual ordered forecast/tick work against existing equality
coverage rather than swapping runners or sacrificing optimization for builds.
No full balance audit, new full-check speed claim, UI change or deployment.

Artifacts scratchpad/build-timing-20261006/: profile.py, results.json, five
Cargo timing HTMLs, original/restored hashes, test-runners.json, runner logs,
nextest16.json, slow-tests.json, native_dev-restored. Derived isolated target
~504MiB reclaimed after retaining its executable/timing evidence. Source copy
retained under /tmp/riddle-build-timing-20261006. Copy1503/diff pass.
