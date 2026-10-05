# Measure multihero catch-up — 2026-10-06

Use legal one/two/three-slot saves from the same Scout camp. New extra heroes
start fresh, so these fixtures measure actual workloads, not identical hero
scaling. Five fresh8h repetitions in headed Chromium using current fast WASM
and existing profile-catchup tool; compilation/warm-up excluded. Inspect leaf
and inclusive stacks before proposing optimisation. Record exact artifact
hashes and per-fixture deterministic report/save results.

Native timing must load Session, not Game: existing sim_perf loads only selected
Game and cannot certify multihero performance. Add a Session diagnostic with
same quick-report mode, per-slot tick counts, repeat equality and optional exact
output record. Keep existing single-hero sim_perf/reference behavior unchanged.
Run native only after WASM profiling finishes to avoid measurement contention.

No speed claim before paired measurement. Any retained optimisation requires
complete report/save equality against original, shared gold/worker order tests,
new or existing arbitrary-slice tests, fast workspace/clippy/WASM/UI checks.
No frame-rate/full-gate/shipping-WASM/deployment claim from fast engine profiles.

Profile supports a bounded candidate: Tree done/acts/ranks maps participate in
repeated town_from value comparisons every100ms. Store these existing value
containers in Shared copy-on-write; synchronize using clone_from with an Arc
identity no-op. Mutations still detach, so every changed value is copied exactly
in the same original slot order. Keep serde map shape/equality unchanged and
all other town fields/clock quanta untouched. Target ≥10% improvement for these
three-slot fast-WASM fixtures over seven alternating pairs; never weaken gates.
Retain only with raw report/save equality per runtime, native/one-slot regression
measurements and full fast workspace checks. Original WASM/native binaries kept
in scratchpad/multihero-perf-20261006 before changes. No shipping speed claim.

First candidate (only done/acts/ranks): seven alternating pairs preserved raw
outputs but three-slot median638.6→597.3ms (6.47%), below10% target. Extend same
value-sharing to the remaining synced non-scalar town containers (Town, hired,
paused) rather than relaxing target. Numeric/shared-clock mutations unchanged.

## Retained result

Town and worker containers now share their immutable values between slots.
Every write detaches through the existing Shared wrapper; synchronization uses
allocation identity to skip redundant clone assignments, without changing value
equality, worker order, 100ms clock quanta, gold rules or serialized shape.

Seven alternating pairs per fixture in headed Chromium, fresh load each time,
8h quick catch-up. Compilation, load and save serialization excluded; report
bridge serialization included. All 70 timed calls preserve the exact original
raw report and save, as do the five alternating native pairs per fixture.
Native uses Session clones outside its timer and includes report serialization.
These fixtures add fresh extra heroes: they are actual camps, not equal-workload
scaling experiments. The progressed fixture has an older selected hero.

| Fixture | Original fast WASM | Candidate | Gain | Native gain |
|---|---:|---:|---:|---:|
| 1 hero, early | 168.7ms | 168.6ms | 0.06% | 4.55% |
| 2 heroes, early | 419.5ms | 359.9ms | 14.21% | 8.90% |
| 3 heroes, early | 654.1ms | 533.6ms | 18.42% | 13.39% |
| 2 heroes, progressed | 2123.9ms | 2036.8ms | 4.10% | 1.83% |
| 3 heroes, progressed | 2389.4ms | 2243.8ms | 6.09% | 2.84% |

Three-hero early fixture clears the original ≥10% target. One-hero timing is
unchanged at this resolution. Progressed camps improve less and remain about
2s in fast WASM; profile their remaining costs before another optimisation.
No shipping-WASM, frame-rate, full balance gate or exhaustive audit claim.

Reproducibility records in scratchpad/multihero-perf-20261006 (diagnostic files,
not committed): paired-wasm2/comparison.json, native-pairs.json and original
artifacts. SHA256 of measured artifacts:

- Original WASM: `e08958229a656092f941bd15c5f34b6bd851b547e620204cddacae85f88a48eb`.
- Candidate WASM: `8a96dde0942fe44d004b53025533e06323c44ffc33c81f99402aa6251bf35f9d` (5,240,610bytes).
- Original native diagnostic: `aacdde0a9afdfcb2f4169a3e44b55384f3c1ca8ca311c0f2a36724ca168244d9`.
- Measured candidate native diagnostic: `01f25e5e46254615cf9b49ab158583b53bb7352e7a1070261338852c26ce2f71`.

The diagnostic gained an optional wording flag after those timing series.
Ordinary native gate wording uses a 64bit event hash; WASM/native app uses 32bit.
Baseline native versus WASM had one parsed save difference in the one-hero
fixture: deaths/32/t10/event_recent/saved/0 (phrase index 0 versus 1). This
predates the optimisation. RIDDLE_SHIPPING_WORDS=1 selects the existing native
app wording adapter before load: candidate native then matches original WASM
raw report AND save exactly for that fixture. Cross-runtime wording equality
was checked only for this fixture; per-runtime original/candidate equality
covers all five. The option defaults off, preserving ordinary gate diagnostics.

Verification:556 fast workspace tests pass35.14s,1ignored; clippy all targets
-D warnings, rebuilt WASM, web build/typecheck, copy1511 and diff checks pass.
New isolation test covers Town, all worker containers, an earlier Session
snapshot, propagation in slot order and exact save/load. Existing clock/sliced
catch-up tests pass in the workspace suite. bloodline-training63 checks5.6s
and worker-first-act60 checks4.2s pass; an initial mistyped worker suite name
was corrected and rerun. Headed real-WASM legal second slot/two12h79runs/third
slot retains identities and selected appearance after reload, with no warnings
or overflow400/1440; roster screenshot shown. No deployment.

## Diagnostic commands

```sh
cargo run -q --profile fast -p riddle-core --example session_perf -- SAVE 8 5 OUTPUT.json
RIDDLE_SHIPPING_WORDS=1 target/fast/examples/session_perf SAVE 8 5 OUTPUT.json
node tools/compare-catchup.mjs ORIGINAL_PKG CANDIDATE_PKG OUT SAVE...
```

Browser defaults:8h,7 alternating pairs. CATCHUP_HOURS/CATCHUP_PAIRS override;
CATCHUP_RECORD_OUTPUT=1 also writes exact raw report/save records. These are
local diagnostics, not substitutes for gates. Preserve a baseline artifact
before rebuilding. Run timing series on a quiet machine sequentially.
