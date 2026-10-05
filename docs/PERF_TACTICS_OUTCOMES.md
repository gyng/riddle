# Tactics outcome projection — 2026-10-05

Continue PERF_FAST_ESTIMATES.md after ad33447. Owner prioritizes local iteration
and accepts approximate forecasts. This step removes unused gold work from all
ordinary Tactics sample sizes (below explicit100-sample refinement), preserving complete option results exactly; samples and
simulation ticks are unchanged. Dayplayer uses32, diagnostic sim_perf uses50.
Regular gold forecasts and actual gold/offline calculations remain fully priced.

Before retaining: freeze baseline binaries. Three saved early/late/tuned camps,
32/50 samples, three alternating fresh-game pairs after warmup. Compare every
complete reply/save, not selected shares. Also test normal-forecast-first and
option-first order, repeat/refinement and offline output. Outcome-only caches
cannot poison priced panels. Forecast-first timing must be measured separately:
losing existing normal-panel prefix reuse can offset the omitted gold work.

Retain only if late/tuned cold median improves at least10%, no workload exceeds
5% regression, and warm total forecast+query has no material regression. Otherwise
reject this extension rather than expanding a successful eight-sample shortcut
without evidence. Keep exact sample counts/budgets, candidate grouping/order,
prices and hard gameplay gates. Native parallel/sequential outcomes must match.
Meaningful native tests, clippy, shipping WASM, real client controls and full
routine gates required if retained. No deployment authorized.

Native cold screen: all24 baseline/candidate pairs including warmups match
complete32/50-sample option bytes. Three timed alternating pairs each:

| Samples | Early | Late | Tuned |
| --- | --- | --- | --- |
|32 |1.543→1.529s |6.962→7.089s |7.796→5.479s |
|50 |2.185→2.111s |8.213→7.843s |9.734→7.230s |

The original10% late retention target is unmet. Late work counts are identical
(688 simulations at32,911 at50): its D1 panel already covers the wall passage,
so there is no extra passage simulation to remove. Tuned work falls636→512 at32
and807→683 at50 because it starts deeper and originally separately simulated
those passages. Early counts also unchanged. No broad speedup claim: cold
late32 is1.8% slower, within the5% regression bar.

Record a retention deviation if warm checks and complete regressions pass:
retain the material deep-start gain with neutral D1 cases, rather than requiring
10% from work those cases already avoid. This is based on explicit work counts,
not reduced gameplay gates; exact sample counts and option bytes remain required.
Warm regressions or semantic differences still reject the extension.


Compatibility finding: explicit100-sample panels serialize refined_panels so
later forecasts preserve the requested quality. Ordinary32/50 queries do not.
Keep the original fully priced explicit-refinement path, including this state;
do not silently remove it while broadening the ordinary outcome-only path.
A test initially asserted that explicit refinement preserved a raw save, found
this legitimate metadata change, and now distinguishes pure outcome reads from
matching explicit-refinement state. Actual-run equality remains required.


Warm screen caught a real regression: isolated outcome caches unnecessarily
reran the active panel after the camp forecast. Retention deferred. Fix information
flow one way: outcome readers may reuse complete normal prefixes; normal gold
readers may never reuse incomplete outcomes. Both caches remain separately keyed.
This supersedes the earlier prohibition on reusing priced results in outcomes;
no estimated gold is promoted to a real gold/quality forecast. New native test
requires zero extra simulations for a32-sample outcome query after50 priced
samples and still requires complete gold work in the reverse order.


D1 exception: skipped-floor gold is exactly zero, so outcome panels below100
are complete gold results and use the normal cache. This preserves option-first
reuse by later camp forecasts as well as forecast-first reuse by options.
Deep-start incomplete results remain outcomes: only. Explicit refinement still
records its existing metadata via the fully priced path. A native test compares
complete32/50 D1 panels against an independent cold control and requires zero
rerun for the32-sample gold read after outcomes.


Final named fast-WASM warm screen, three alternating pairs/camp after warmup,
normal forecast followed by32-sample options, load/module setup excluded:
early2.123→2.157s (1.6% slower), late9.541→9.548s (0.1% slower),
tuned10.388→7.834s (24.6% faster). All12 complete forecast/option/save pairs
including warmup match; three explicit refinements (one per camp) also match.
Ordinary queries preserve their complete raw save; explicit refinement records
the same metadata in both binaries. Warm regression bars now pass. Retain the
recorded late cold-target deviation if final shipping/gates pass. Native cold
figures above are screening data from the initial all-size projection prototype;
final code adds explicit-refinement compatibility and safe cache reuse. Do not
claim those prototype numbers as final native or whole-suite speedups.

sim_perf now accepts an optional fourth argument after OUTPUT for package sample
count, e.g. `SAVE packages /tmp/read.json 32`. Its metadata
records requested_sims; work.simulations reports actual expeditions including
passages. Other workloads keep their existing behavior and report no sample
request. This allows native profiling to match dayplayer32 or UI8 instead of
implicitly measuring50. Final acceptance follows below.


## Acceptance — 2026-10-06

Retained the documented late/D1 retention deviation: material deep-start gain,
neutral D1 cases and all semantic/no-regression gates pass. Final shipping WASM,
tuned camp, three alternating cold pairs after a discarded warmup, load/setup
excluded:32 samples10.126→7.242s (28.5% faster);50 samples12.328→9.241s
(25.0% faster). All eight complete option/save pairs including warmups match.
These are query timings for one deep-start fixture, not a general gate-runtime
or frame-rate claim. Private evidence: scratchpad/tactics-outcomes-20261005/.

Shipping compatibility comparisons cover three saved camps × five workloads:
32/50/100 options, normal forecast and eight-hour offline progress. All15
complete reply/final-save pairs match. The initial ordinary32 query preserves
its raw save; explicit100 refinement retains identical quality metadata.

Full verification passes:543 core tests, one ignored;11 tooling tests; tsc,
copy lint, all-target clippy, optimized shipping WASM and client build. Fresh
full routine gates (no cache hits) pass all18 selected fortnight cases. Total
verifier513s includes compilation/build; QA49.7s, metrics112.2s,
dayplayer212.8s. This is the selected routine suite, not the exhaustive272-case
audit, and it does not meet a two-minute total. No causal whole-suite speedup
is claimed from separate historical runs.

The full progression output matches the previous version byte-for-byte except
its single elapsed-time line; normalized SHA256
667db71043158b47fa60d9ba478157f9ac37d73d8ceacbd0aa6767a03f566521.
Real shipping controls at400/1440 confirm an explicit eight-sample comparison,
rough-estimate label and no horizontal overflow; screenshot checkpoint saved.
Push authorized; deployment remains manual and unrequested.

Next inspect wall-search panels: they consume depth shares and may benefit from
the same projection. Require their own exact candidate/prefix/refinement-state
comparison before retention. Then separate small camp/Forge preview panels
from real progress and make expensive refinement explicitly requested. Do not
lower a globally shared forecast constant used by actual gold/progress.
