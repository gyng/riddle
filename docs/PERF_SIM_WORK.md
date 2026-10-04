# Simulation work — 2026-10-04

Owner approved all four follow-ups: reuse arbitrary forecast prefixes, retain
screening results through refinement, reduce pathfinding/input costs, and share
execution until candidate policies diverge. This is local engineering after
Cut30.5, not Cut31. Baseline core97a4dba1715763c3 (public9292f49).

Preserve seeds, ordered budget selection, RNG, simulation time, every gate/bar,
report/save/trace results and public wire/save types. Compare each candidate with
the preserved baseline binary, not a freshly overwritten target executable.

1. Measure actual executed simulations/ticks, prefix reuse, panel hits/misses and
   eviction. Retain screening results locally in the wall search and extend the
   exact12-run prefix to48, using the same inputs/tag/index and ordered budget.
2. Benchmark repeated BFS scratch buffers and distance reuse only where inputs
   are proven identical. Preserve FIFO direction order, layer/index ties,
   corners, seen/avoid masks, mutation and recursive-callback behavior.
3. Prototype common execution prefixes with explicit policy-dependency fences.
   Initial loadouts, projected lineage, supplies and RNG must agree; fork before
   a differing policy can affect state. Comparing final fired rows alone is not
   proof. Reject a prototype that adds work or cannot preserve exact behavior.

Targets:15% lower median native forecast/search CPU over seven alternating pairs
on three saved camps, and10% lower shipping-WASM workload time where applicable.
Record deviations; do not retain costly architecture solely to meet scope.
Keep successful low-risk exact-work reductions with measured benefit, and retain
rejected prototypes/evidence privately. Measure native and WASM separately.
Fresh all-case completion within five minutes remains an investigation target,
not an achieved gate. Do not infer it from cached output or focused cases.

First prove each change with reference comparisons and focused meaningful tests;
then freeze retained Rust changes and run required full verification once. Rebuild
shipping WASM, check fresh local/public real-WASM/offline/layout flows and show
inline screenshots at release checkpoints under existing publication approval.

## Candidate screen

Seven alternating native pairs per workload, raw serialized result bytes checked
on every run, with matched single-thread diagnostics and preserved executables.
Prefix-only tuning search medians: early0.342→0.338s (+1.13%),
late6.737→6.254s (+7.16%), tuned22.194→20.772s (+6.41%).
Late search654→582 executed sends and7654650→6960562 logical ticks;
tuned1312→1216 sends and21596303→20296491 ticks, including64 panel
evictions. Screening prefixes survive those evictions. No absence or package
forecast speedup is attributed to this change: their work counts are unchanged.

BFS scratch candidate rejected: incremental median differences range from
−0.92% to+1.55%; tuned search −0.36%. Buffer reuse passed the mutation/size/
recursive-callback path oracle but failed to show a consistent gain worth its
thread-local buffers and borrow fallback. Patch/test/evidence retained privately
at `scratchpad/sim-work-20261004`. Production BFS
remains unchanged. Text allocation is retained after a second seven-pair screen, with exact raw
result bytes on all seven workloads. Heavy searches late6.967→6.007s
(+13.79% total; text increment2.36%) and tuned22.070→20.162s
(+8.64% total; text increment2.50%). Early search0.334→0.336s is neutral
(−0.72%). Eight-hour native offline: early0.498→0.499s (neutral),
late1.877→1.818s (+3.14%), tuned1.555→1.543s (+0.79%).
Package workload10.177→9.673s (+4.95% overall), but unchanged executed
work and fluctuating prefix-only timings across rounds mean no package-work
reduction is claimed. Text increment there1.25%.

The15% native target is missed. Retain low-risk exact reuse plus the small
consistent text savings in heavy searches; remove the more complex BFS scratch.
Evidence `text-paired.json` and `candidate-provenance.json` under the private
sim-work directory.
Policy-prefix prototype rejected after seven alternating pairs on three real
camps and one literal threshold family; every raw SimResult array matches
independent reference execution. Twelve literal seeds ×three trial indices also
pass, with healing-threshold families and a structurally different control.
Eligibility requires complete initial Game equality after only active-policy/
pen normalization, plus identical origins/route/verbs/condition structure except
numeric hp thresholds. Whole-policy reads and a changed row that can hold force
a fork from the pre-tick snapshot; failed verbs/shadowing/homeward/lent rows are
covered by conservative fences. No production instrumentation from this probe.

Real medians reference→prototype: early0.00990→0.01034s (4.36% slower),
late0.470→0.483s (2.93% slower), tuned11.366→11.396s (0.26% slower).
Late candidates have no eligible group; tuned saves54 of1250232 logical ticks
across75 trial expeditions (0.0043%). The controlled family saves114 ticks with
~1.66% median gain at44ms, insufficient to justify hot-loop policy tracking and
Game snapshots. This narrow prototype does not rule out other sharing designs;
it demonstrates that these policy candidates diverge too early for this design.
Evidence `prototype-paired.json`, private sources and `check-final.log`.
Shipping-WASM seven-pair comparisons pass exact reports/saves throughout.
Late cold wall8.3965→7.5424s (+10.17%); tuned cold wall27.9994→25.6158s
(+8.51%); early0.4311→0.4474s (−3.78%, shallow workload). Early/tuned
cold-wall fixtures explicitly set historical wall eligibility (day at least2,
best_day=day−2,clear wall_day/offer); late uses its original eligible saved camp.
Only the late workload meets10%; no general10% promise. Retain exact-work
reuse and small measured heavy-search savings, record the missed targets.

Two offline rounds disagree in direction: first early/late/tuned −5.42%/−2.11%/
−1.77%, then three-build rotated/reversed round +1.76%/+1.36%/+5.54%. The
second includes a forecast-only shipping control; text relative to that is
+2.68%/+0.82%/−1.96%. No consistent catch-up gain or regression is established.
Treat offline as neutral for this change, preserving the prior quiet-tick fix.
Shipping hash9fd7f12a53f5b645cec1c21d34afab8506b0042111b6d21c6ff83965eb30b1ca.

## Follow-through: test-only iteration cache

Under the owner's continuing local-iteration authorization, primitive job keys
now use the actual compiled native production core library, its Cargo profile/
features, manifests, lockfile and compiler identity (`native-core-rlib-v1`).
Cargo JSON artifacts select the production library explicitly; absent/wrong/test
artifacts fail closed. Existing harness/body/argument/runtime-data keys remain
intact. No old records are relabeled under the new scheme; this full run warms it.
Standalone cfg(test) edits no longer invalidate the targeted simulation identity.

`tools/runtime-key.test.mjs` compiles a real isolated Rust fixture: test-only and
harness edits preserve the core key; a deliberately failing test still executes;
runtime code, transitive include files, compiler flags, manifests/lock and compiler
identity change it. Wrong/missing artifacts are rejected. Pass2.91s; now runs in
`tools/verify.sh` alongside TypeScript/copy lint. Full integration passes (below).


## Acceptance checkpoint

Frozen `tools/verify.sh --full` passes in3870s:522 engine tests/one ignored,
TypeScript/copy lint, real-compiler key proof, clippy, shipping WASM/web build,
metrics1455.7s, wire QA225.8s and all dayplayer bars. Runtime key
8eb5723c2442502e; new primitive prefixsrc4fd4d0e184912ad3. All272 completed
fortnight records match accepted src7b74699adc230110 exactly, including every
decoded field. No records copied or renamed. Complete cached table passes0.43s
with genuine metrics/QA/dayplayer leg hits. Fresh work still64.5minutes; this is
not a five-minute fresh suite or a paired full-suite speedup claim.

Fresh local shipping real-WASM/service-worker/offline reload and400/1440 layouts
pass. Headed seed1 send/scout/8h walk:8 captures,no errors,two existing WebGL
warnings; fetched shipping9fd7f12a… matches the built artifact.82.4s under full
gate CPU load is functional evidence only. Camp/report screenshots shown inline.
Public deployment checkpoint follows the approved release.
