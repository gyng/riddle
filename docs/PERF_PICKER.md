# Package-picker follow-through — 2026-10-04

Continue the owner-approved local simulation/iteration work after the simplified
UI release. Preserve every seed, budget, simulation ordering and complete result;
no balance or UI behavior changes. Routine acceptance uses the owner-selected
18-case matrix; no claim that it certifies the exhaustive statistical audit.

Measure an uncached package read on early, late and tuned saved camps. Record
actual simulations/ticks, sampled CPU and allocation churn. Native gprofng on
this WSL machine reports its timer changed to zero: its partial samples are
unreliable and are excluded from conclusions. Use named fast-WASM browser CPU
samples instead; do not claim these are shipping timings. Keep instrumented
allocation timing separate from ordinary benchmarks. Requested bytes are
cumulative allocator traffic, not peak live memory.

Candidate: bounded stack storage for active decision rows, retaining a heap
fallback for arbitrary imported policies, and borrow the party condition scope.
Keep active/card/lent order, shadowing, guards and all event text. Require full
raw outputs to match on every alternating benchmark. Initial screen three pairs
on three camps; extend to seven pairs if useful. Retain only with lower allocation
counts and at least2% median improvement in representative expensive reads;
otherwise reject and proceed to the native rebuild watcher. Verify overflow and
mixed row origins, full engine tests, routine gates, shipping/native parity and
public shipping checks before publishing a retained engine change.

Evidence: scratchpad/picker-perf-20261004 (private, not checked in).

## Measured outcome

Named fast-WASM profile:43,147 CPU samples over three cold package reads on
seed15/day5; complete report/save hashes match across repetitions. Leaf samples:
hero_action8.13%, vision7.40%, nearest BFS6.02%, flood_resume4.24%, malloc/free
4.99% combined, condition evaluation1.43%. Inlining attributes some work to its
caller; these are sample shares, not independent phase stopwatch timings.
Uncached native workload:1,461 simulations,14,009,564 logical ticks and38 panels.

Separate counting allocator:50,152,725 allocations,8,627,739 reallocations,
20,474,284,348 cumulative requested bytes (whole process, including loading and
serialization). Stack candidate:48,874,615 allocations,6,071,519 reallocations,
19,901,691,074 requested bytes; complete output matches. Lower churn alone does
not establish lower latency.

Candidate REJECTED. Seven quiet alternating pairs with exact complete outputs:
late12.1221→12.0792s (+0.35%); tuned9.9265→9.6835s (+2.45%). Initial three-pair
screen had early regression/noisy offline outcomes. There is no consistent2%
gain across representative expensive reads. Production decisions are untouched;
private patch and evidence retained, no weaker simulation budgets introduced.

Retained follow-through: workload selection in the browser profiler, a separate
native allocation diagnostic, and automatic native dev rebuilding. The watcher
coalesces edits, serializes builds, queues edits arriving during compilation,
keeps the last executable on failure and exposes errors in the dev page. Client
bridge metadata publishes only after successful binary replacement, preventing
premature reloads into an unfinished bridge. Runtime balance edits need no build.

Real GPU browser watcher proof PASS: controlled invalid Rust compile keeps the
last binary and an isolated real lane's complete save, shows useful Cargo JSON
compiler diagnostics in the dev page, then valid binary replacement/recovery
and original-source restoration keep the same save. Example adapter edit takes
2,241ms including debounce; not a general core/cold-build claim. Queued edits
are explicitly marked pending, never ready before compilation. Existing native
servers with another watcher configuration are rejected by the launcher.

Owner overlay refinement: combat log is transparent text directly on the map,
no box/header. Three visible lines,80 ordered entries retained for scrolling,
small plain font with shadow. Hero damage red, outgoing damage/coins gold,
healing green (real Heal events), items blue, enemy warnings amber; signs/names
remain readable without color. Item
pickup events are included. No simulation changes. Real UI31 checks PASS at
400/1440px; shipping/public results are recorded in HANDOFF after verification.

## Diagnostic commands

```
cargo run -q --profile fast -p riddle-core --example sim_alloc -- SAVE.json packages
node tools/profile-catchup.mjs SAVE.json --out scratchpad/profile --mode packages --runs 3
```

Use --pkg DIR for a named profiling module; shipping may expose only function
numbers. Ordinary sim_perf/camp-check timings remain uninstrumented. Public
shipping validation passes on release b59a7e9: mobile/desktop/offline,8screenwalk,
exact CI WASM identity and transparent colored overlay. See HANDOFF for evidence.
