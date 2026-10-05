# Bound quick plateau estimates — 2026-10-06

The progressed three-hero8h catch-up spends roughly62% of sampled engine time
in the selected plateau forecast. Native work diagnostic:69 simulations,
1,375,344ticks; base23seeds, no useful patch. Preserve the actual simulation;
reduce only the quick report's optional paired estimate. Owner allows approximate
forecasts. Full reports and direct stall_verdict retain the existing50-seed cap.

Candidate: quick reports cap the base at12seeds with the existing450,000tick
budget/minimum5, and every candidate uses exactly the base's achieved count and
same ordered seeds. Include sample cap in the transient stall cache key so
quick and full calls cannot contaminate each other. Do not change candidate
generation, patch applicability, positive delta threshold or full gate paths.

Retention gates, recorded before implementation:

- ≥20% median speed improvement for progressed three-hero fast WASM over7
  alternating fresh-load pairs; no >5% regression for the early one-hero case.
- Exact raw save equality and every report field except stall.patches unchanged
  against the preserved original for all comparison fixtures. Exact repeat
  equality within each mode. Retained recommendations remain deterministically
  sorted, applicable and above the existing positive threshold.
- Quality sample includes16distinct literal wall seeds and at least6 progressed
  real-camp states. At least6 strong reference cases (top full delta≥0.15) must
  exist. In≥80% of strong cases quick has a recommendation, and in≥80% its first
  patch belongs to full's displayed positive set. For matching patches median
  absolute delta error≤0.10. Across all cases with a quick recommendation,
  ≥80% of first choices must belong to the full displayed positive set.
  Report misses and differences, never claim exact
  forecast equality or population-level statistical confidence.
- Tests prove bounded quick work, untouched save/report facts, deterministic
  repeats and quick/full cache separation; existing full plateau tests pass.
  Full fast workspace/clippy/WASM/build/typecheck and relevant client report
  checks. Headed screenshot of real report at400/1440. No deployment.

If quality or timing fails, improve or discard the candidate; don't weaken
these gates. No shipping-WASM/FPS/exhaustive balance-audit claim.

## Quality evidence

First sample22cases (16literal seeds +6progressed saved-camp states) had only
5strong reference cases, below the required6. Expanded to32literal seeds,
keeping the same hp<35→return rule and elapsed workload, instead of weakening
the minimum. The final38cases have13strong reference cases:13/13 retain a
quick suggestion and13/13 first choices are in the full positive set. Across
all19quick first choices,18/19 match the full positive set (94.7%). The sole
additional first suggestion is camp-3, +8.3points, absent from the full estimate.
36matching patch deltas have median absolute error6.7points and maximum24.3
points. These are a bounded local diagnostic sample, not a calibrated accuracy
estimate or representative field population; literal fixtures intentionally
exercise player-written early return rules. No accuracy guarantee.

Every raw save and report field except stall.patches matches the original in
all38cases. Native uses the same existing gate wording for both versions.
The full-reference artifact predates the optimisation. Diagnostic inputs,
quality script and results: scratchpad/plateau-perf-20261006/fixtures and
quality.json. Full paths retain50-seed cap; no full balance gates weakened.

New test: quick work is bounded for base+three candidates, repeated quick
queries memoize, full after quick equals fresh full, and quick after full
returns the same quick result without changing save. Only one suggestion
result is stored; switching modes may replay candidates while retaining base
memo. Initial test incorrectly assumed two stored results and undercounted
rest candidate; corrected. Moving the test initially left a misplaced cfg
attribute; corrected and the complete workspace/clippy rerun, without suppressing
lint. Full fast workspace557pass39.57s/1ignored (558total).

## Timing and final checks

Seven alternating fresh-load pairs per five fixture, headed Chromium on quiet
machine after builds, tests and quality comparison completed.8h fast-WASM
quick calls; compilation/load/save serialization outside timer, report bridge
serialization inside. Each version repeats exactly, cross-version raw saves
and all report fields except stall.patches match on all70timed calls.

| Fixture | Original | Bounded quick | Gain |
|---|---:|---:|---:|
| 1-slots | 172.2ms | 169.1ms | 1.80% |
| 2-slots | 361.9ms | 360.8ms | 0.30% |
| 3-slots | 545.8ms | 548.8ms | -0.55% |
| 2-late | 2039.3ms | 1405.2ms | 31.09% |
| 3-late | 2258.2ms | 1617.5ms | 28.37% |

The three-hero progressed result clears20% target; early one-hero no-regression
check passes. Candidate native diagnostic on progressed3hero: 36
forecast simulations and749,619ticks, versus69/1,375,344 before.
This counter covers all forecast work in the call; actual catch-up ticks are
separate. No native speed claim from counts alone.

Measured original WASM SHA256: `8a96dde0942fe44d004b53025533e06323c44ffc33c81f99402aa6251bf35f9d`.
Candidate SHA256: `d0eeb0fffc0e6a9d5f6829d618919a8ff597f201498f42c7bf7cef6be2487ab5`;5,240,670bytes.
Paired records: scratchpad/plateau-perf-20261006/paired/comparison.json.

Clippy all targets-Dwarnings, final rebuiltWASM, web build/typecheck,
copy1511/diff checks pass; existing web chunk advisory. UI report-actions48
and report-hero-upgrade57 checks pass in10.6s (three widths each). Actual
real-WASM progressed camp-3 quick8h report400/1440: generated +8.3point boss
attack suggestion, source payload visible, Details opens, no warnings/overflow;
headed screenshots shown. This is also the quality sample's extra quick first
suggestion absent from full, intentionally demonstrating the approximate case.
No patch applied in that visual walk. Full/quick save isolation and applicable
candidate construction remain covered by Rust/quality checks.

Reproduction:

```sh
cargo run -q --profile fast -p riddle-core --example plateau_perf -- SAVE 8
cargo run -q --profile fast -p riddle-core --example plateau_perf -- --fixtures OUT 32
CATCHUP_ALLOW_PLATEAU_ESTIMATE=1 node tools/compare-catchup.mjs ORIGINAL_PKG CANDIDATE_PKG OUT SAVE...
```

The paired harness defaults to exact raw report/save equality. Its explicit
estimate flag permits differences solely in stall.patches and records both
suggestions; exact report repeat equality within each version remains required.
A separate identical-baseline test exercises the unchanged strict default.
No speed claim from that one-pair harness test. No deployment/full balance audit.
