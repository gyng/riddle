# Fast outcome estimates — 2026-10-05

Owner: forecasts do not need 100% accuracy. Supersedes exact preservation of
Tactics forecast numbers in PERF_COLD_TACTICS; real runs, saves, offline progress,
death verdicts and numeric gameplay gates retain their existing behavior.

First change: Tactics Compare outcomes requests eight expeditions per candidate
instead of24. Keep the explicit request, background lane and snapshot guards.
Show “rough estimate”; round displayed changes to five percentage points and
prefix them with ≈. Hide ambiguous differences. Since panels may stop at five
samples and this wire omits actual counts, use five as the conservative sample
count for a two-sided noise heuristic that includes candidate and baseline
variance. This is a rough filter, not a promise
of calibrated95% confidence. No new speed/sample controls or automatic refine.

Acceptance: three frozen early/late/tuned camps, three alternating fresh-query
pairs after warmup; full save unchanged for every query. Late and tuned median
at least25% faster; record differences versus24 samples and any opposite
signs among displayed estimates. Preserve query coalescing, failure retries,
hero/Legacy/rules/loadout invalidation and stale-reply guards; mobile/desktop
labels fit. Typecheck, copy lint and scoped Tactics client gates pass. Do not
claim these forecasts are exact or that this changes real game performance.

Follow-up: passage pricing has a fixed20-simulation cost, even for an eight
sample comparison. Profile that floor before introducing a separate approximate
passage cache; actual passage gold must keep its current calculation.

Measured on preserved named fast WASM from7663ef4; three alternating fresh-query
pairs/camp after one warmup pair, load excluded, no compiler alongside:

| Camp |24 samples |8 samples | Speedup |
| --- | ---: | ---: | ---: |
| Early |1.507s |0.927s |1.62× |
| Late |8.044s |2.854s |2.82× |
| Tuned |8.806s |5.649s |1.56× |

All24 queries (including warmups) preserve their complete save. Raw deltas
are approximate: mean absolute differences vs24 samples across the three
visible metrics are4.0/6.6/7.0 percentage points; maxima25/29/33 points.
The initial candidate-only filter showed two tuned-case losses of25 points
whose24-sample reference was a noisy8-point gain. Including baseline variance
hides those. Final filter displays0/1/2 estimates across the camps; none reverse
sign versus the24-sample reference. This limited comparison is not ground truth
or a confidence guarantee; it intentionally hides many small differences.
Neither native timing nor shipping-optimized latency is claimed by this named
fast-WASM benchmark. Private timings/full replies/save proof and quality summary:
scratchpad/forecast-samples-20261005. CPU attribution remains in PERF_COLD_TACTICS.

Parked the unmeasured rule-row allocation probe and restored rules.rs/turn.rs
byte-for-byte before this change. No runtime Rust change or WASM rebuild needed.
The shared camp/Forge/start forecasts still use their existing larger panels;
reduce those separately with actual-count-aware display and gameplay isolation.

Validation: final request/ranking/uncertainty lifecycle82 checks; Cut3041 checks;
lane memo16 and shared chrome176 checks pass. Typecheck, copy lint1471 literals
and client build pass. Real current local WASM controls at400/1440 request8,
show rough estimate and fit without horizontal overflow; screenshots archived.
This is local validation, not public deployment. No gameplay gates rerun: Rust,
shipping WASM bytes and actual simulation inputs are unchanged.
