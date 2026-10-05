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

## Rejected sized-passage probe contract

Tactics requests of eight samples or fewer may price skipped-floor gold using
no more passage samples than requested. Existing passage_for/from/run and all
real send/offline calculations retain20-sample behavior. Estimated passage and
camp panel caches must have a separate namespace; no reuse in either direction
between approximate and normal forecasts. Cache keys distinguish passage sample
counts so refinement cannot silently reuse differently priced outcomes.

Acceptance: approximate-first and normal-first query orders leave full saves
unchanged and produce the same normal passage/panel and actual send as controls.
Repeated estimates reuse their own cache. Sequential/native parallel outputs
match. Preserve exact24/50-package and offline complete replies against a frozen
baseline. Measure eight-sample queries on the same three camps in alternating
pairs; retain only with at least10% median late/tuned benefit and no>5% regression.
Run focused passage/refinement tests, workspace fast tests/clippy, shipping WASM
and appropriate client checks before retaining. No public deployment.

Sized-passage prototype rejected: named fast-WASM late13.3% faster, tuned only
5.0% faster. It fails the10% tuned retention bar. No production sized-passage
API retained. Better structural finding: PkgOption contains depth/exit shares
and mean depth, not gold income; its predictions never consume passage coins.

Revised implementation contract: small Tactics queries skip passage pricing
entirely and cache outcome-only panels under `outcomes:`. Regular panels and
real sends cannot consume those results. All complete eight-sample package
replies must remain byte-identical, alongside24/50-sample and offline outputs.
Keep the same timing retention bars. This removes unused forecasting work,
not simulated ticks or gold from actual game runs. Larger forecast/Forge gold
readers retain normal fully priced panels.


Outcome-only fast-WASM screen (three alternating pairs per camp after warmup):
early0.988→0.471s (52.3% faster), late2.801→2.283s (18.5%),
tuned5.588→2.491s (55.4%). All12 complete price/save pairs including warmups
are byte-identical; every query preserves its own save. Source baseline d70a884,
private modules, full replies and results under passage-estimate-20261005.
This is an additional reduction after switching Tactics from24 to8 samples;
measurements use named fast WASM, not a shipping/whole-app performance claim.
Two cache-isolation/actual-send/work-count/parallel-order core tests pass0.60s.
Accepted: all nine24/50-sample/offline-after-estimate complete reply/save pairs
match in named fast WASM, and all nine match in shipping WASM. Rebuilt shipping
controls request8 and show rough estimate at400/1440 without horizontal overflow.
Shipping baseline SHA232199b2…, candidate5e3d0aad…; full checksums and raw proof
are in the private artifacts. No shipping-latency or published-build claim.

Full verification passes540 tests (one ignored), all-target clippy, shipping WASM,
client build/typecheck/copy lint and11 tool tests. Fresh full metric/QA/dayplayer
legs all pass; full means18 selected current-player fortnight cases, not the
broad opt-in historical/system-removal audit. Initial verifier completed all
simulation gates but its final shell parse failed because the reporting filter
was edited while the script was running. Stable-file rerun is green46s using
those completed leg caches;46s is not a fresh simulation runtime. Canonical
acceptance log: verify-final.log, with fresh legs in verify-full.log.

Verifier reporting fix: exclude only the exact zero-test summary; the former
substring filter also suppressed totals ending in zero, including540 passed.
Manual fixture and stable full run verify the corrected summary.


Next by impact: extend outcome-only pricing to larger Tactics requests after
checking complete32/50-sample choices and warm-cache behavior. Dayplayer uses32
samples, so this can remove unused passage work from gate iteration as well as
UI comparisons. Then separate a smaller first camp/Forge forecast from the
fully priced panel used by watch folding, gold forecasts and real progress.
Do not change FORECAST_SIMS globally: gameplay currently also consumes it.
