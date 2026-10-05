# Rough camp and Forge previews — 2026-10-06

Owner accepts approximate forecasts. Eight-sample camp and Forge previews
replace their first50-sample reads; never launch100-sample camp refinement
automatically after a paint or edit. More samples is an explicit action inside
forecast details. Rough estimates stand as usable answers, not endless pending
ellipses. Keep the actual sample count and uncertainty from Rust. Preserve
requested detailed results for the same state; reject stale refinement replies
after rules/hero/lineage changes. Deduplicate explicit requests and permit retry.

Keep existing full core APIs and all actual-run/gold/offline behavior. Camp
estimates still price gold; Forge estimates need only depth/exit outcomes and
skip unused passage-gold work. Default full Forge calculations remain compatible.
Opening/buying Forge still starts zero forecasts, with one explicit preview ask.

Numeric acceptance: zero automatic refine requests on camp forecasts/edits;
exactly one for an explicit request, even on duplicate taps; no stale paint.
Three alternating shipping pairs/camp after warmup must improve heavy cold
preview workloads by at least25%, with no case over5% slower. Report8-vs50
differences as sampling error, not truth. Native tests require pure preview
saves, full forecasts/refinement and actual sends/offline equal to controls.
Real400/1440 controls/screens, client lifecycle/geometry/copy checks and full
routine verification required. Push authorized; no deployment.

The paired edit comparison also has an eight-sample API, so preview bars and
`vs last run` deltas use the same indexed samples. Default full comparisons
remain unchanged. The shipping bridge, generated native bridge, proxy and lane
registries expose all three preview APIs; Forge keeps its dedicated measure
lane and camp previews retain foreground/mirror scheduling.

Core test proves preview purity, exact normal forecasts/default Forge outputs,
explicit-refinement metadata and actual send/one-hour offline saves against an
independent control. Full routine verification passes:545 tests, one ignored,
11 tooling tests, tsc/copy lint, all-target clippy, shipping/client build and
fresh18 selected fortnight cases (470s total including compilation/packaging;
QA48.3s, metrics111.6s, dayplayer168.7s). Complete progression output equals
cb10e0c except its elapsed header. This is the selected routine suite, not the
exhaustive272-case audit or a two-minute verification claim.

Client lifecycle32 checks, Forge33, geometry176 and current shipping paint5
checks pass. Updated historical Cut24 gate20/20 and Cut28 gate27/27 pass:
explicit Forge Forecast, folded report details, storage terminology and rough
answers standing without automatic refinement. Timing checks request the
larger pass explicitly while retaining their original latency bars. Prior
Cut24 failures concerned retired UI selectors/report placement; production
report code is unchanged. Shipping timing and parity acceptance pass; visual controls recorded below.


Shipping acceptance (three alternating cold pairs per camp, after warmup;
query-only medians, load/render excluded):

| Camp | Forecast before → after | Forge before → after |
| --- | --- | --- |
| Early | 286 → 45 ms (84% faster) | 1266 → 204 ms (84%) |
| Late | 460 → 127 ms (72%) | 1896 → 597 ms (69%) |
| Tuned | 835 → 379 ms (55%) | 3382 → 783 ms (77%) |

These are shipping-WASM query measurements, not whole-app load or FPS claims.
All heavy cases exceed the25% retention bar; none regress. Three camps × four
subsequent workloads (normal forecast, explicit refinement, default full Forge,
eight-hour offline) give12 exact complete reply/save pairs after all three
preview APIs run first. The producer terminates successfully.

Eight samples trade precision for latency. Before panels actually used50/25/34
samples (the full API has a time budget). Mean absolute reach differences are
2.6/5.8/7.0 percentage points; maximum individual differences12/14/34.6 points;
maximum exit differences2.5/15/6.6 points. Forecast gold estimates also vary
(80.86→61.25,310.04→109.125,540.62→548.63). Neither estimate is ground truth.
Forge target floors can differ, so deltas at different target floors should
not be interpreted as direct accuracy comparisons. Sample count and existing
Rust uncertainty are displayed; actual gold and progression are unchanged.

Artifacts: scratchpad/rough-previews-20261006/, named shipping-baseline and
shipping-candidate modules, cold pair JSON, shipping-cold-summary.json,
sampling-comparison.json, shipping-parity.json and full verification log.

Next queue: scope Forge forecast memoization to each engine/hero and complete
camp state, reject stale Forge replies; simplify raw report unlock IDs and wall
edit descriptions; profile querying only requested Tactics groups. The selected
routine suite is still470s fresh, not all gate cases within minutes.

Real shipping400/1440 control checkpoints pass: exactly one eight-sample camp
query, zero automatic larger queries, exactly one explicit refinement and one
explicit Forge query, no horizontal overflow. The larger pass produced52
samples under the existing budget; More samples promises a larger estimate,
not a guaranteed100 samples. Screenshot artifacts are400/1440-forecast.png
and400/1440-forge.png. No deployment performed.
