# Wall search outcome projection — 2026-10-06

Owner accepts approximate forecasts and prioritizes local simulation latency.
Wall search consumes only depth shares, yet currently prices skipped-floor gold
for each edited candidate. Reuse the outcome-only panel projection, retaining
owned screening prefixes and complete-to-outcome cache reuse. Preserve candidate
generation, sample counts (12/48), budget, tie ordering and complete WallEdit.

Freeze current native/shipping binaries and three saved camps before editing.
Measure three alternating cold pairs after warmup. Require complete search
results equal, at least10% gain in a heavy deep-start search and no regression
above5% elsewhere. Preserve normal gold forecasts, explicit refinement, actual
sends and offline saves after the query; incomplete outcome cache entries must
never enter a priced forecast. Test screening-prefix continuation against a
priced reference and parallel/sequential outcomes. Full routine gates and
shipping comparisons required before retaining. Do not infer whole-suite or
rendering gains from query measurements. Push authorized; no deployment.

Native screening: all12 complete search pairs including warmup match. Three
alternating timed pairs per camp, cold load excluded, single-thread fast native:
early0.341→0.262s (23.2%); late6.364→3.509s (44.9%);
tuned21.355→11.985s (43.9%). Actual simulation counts fall116→96,
582→401 and1216→814 respectively. Timing is for these frozen camps, not the
whole progression suite or browser latency. Early/tuned inputs explicitly set
historical wall eligibility (day≥2, best_day=day−2, cleared cached offer/day);
late receives the same normalization. Frozen sources/binaries and raw pairs:
scratchpad/wall-outcomes-20261006/. Final shipping/gate acceptance pending.


The first full verifier attempt passed544 core tests (one ignored), tsc and
copy lint, then failed native-host tooling under the restricted subprocess
sandbox. The isolated tooling test passes with local subprocess access; the
complete verifier is rerun in that environment. No tooling/runtime changes
were made to bypass the test. New native test explicitly evicts the screening
cache, resumes the owned12-sample prefix to48, requires exactly the missing
simulations, matches an independent cold result and a priced reference's
depth/tier/cause/ticks/fires, and preserves later complete gold forecasts.
Sequential and parallel prefix continuations are identical.

Frozen baseline source: ddf201b. Shipping WASM SHA256 before:
160ad04c9a3de9709fc5495f4f6ad5432a79bd62ab24e96f3c3af1ef01d779b8;
candidate:
33cd26ef0e3d827efe8714ddf9a2da25a44a1efd79f8f6dcad22a3324da6cb00.

Full routine verification passes in the local subprocess environment:544 core
tests, one ignored,11 tooling tests, tsc/copy lint, all-target clippy, shipping
WASM and client build. All18 selected fortnight cases pass with fresh legs
(cache hits none): QA49.2s, metrics114.2s, dayplayer169.0s; total429s includes
compilation/packaging. The entire progression printout equals ddf201b's except
its elapsed header; normalized SHA256
667db71043158b47fa60d9ba478157f9ac37d73d8ceacbd0aa6767a03f566521.
Do not attribute the difference from the previous513s full run causally to this
query optimization: build state and shared-machine load also differ. This
selected routine suite is not the exhaustive272-case audit or a two-minute
verification claim. Final shipping query measurements and interaction checks
still determine retention.

Final shipping cold query, three alternating pairs per camp after discarded
warmup, module/load setup excluded: early0.440→0.342s (22.3% faster),
late7.865→4.396s (44.1% faster), tuned27.591→15.377s (44.3% faster).
All12 complete offer/save pairs including warmup match. Repeated cached reads
return the identical offer and leave the saved state unchanged. No competing
builds, native simulations or browser sessions ran during these timings.
Cold-query retention bars pass in both native and shipping builds; ordinary
forecast/refinement/offline parity and actual UI apply checks remain pending.


## Retained — 2026-10-06

Nine final shipping sequences (three camps × forecast-first, normal+refine
after search, and direct eight-hour offline after search) match complete
initial forecasts, offers, intermediate wall saves, normal forecasts, explicit
refinement replies and final saves. The parity producer wrote all nine PASS
results and its complete manifest, then terminated with143; an independent
terminal-zero recheck reloads all18 raw artifacts and verifies every complete
object exactly. No latency claims use the parity runs.

Real shipping report controls at400/1440 pass: open Details, wait for the real
wall offer, then apply its rules in order and its suggested D19 start. Existing
client origin tagging is preserved. Screenshots shown at the checkpoint. The
first harness attempt correctly found the offer hidden inside Details; the
updated harness follows that fold rather than bypassing it. Temporary UI
servers/browser contexts close in finally.

Retention gates met without a deviation: both heavy native/shipping searches
improve over10%, all cold cases improve, complete offers/saves/progression
remain identical and full routine verification passes. Approximate forecasts
remain owner-approved; this particular change removes unused work without
reducing sample quality. Push authorized; no deployment.

Next: first-pass camp/Forge forecasts and explicit refinement. App.emitForecast
and rulesChanged still schedule100-sample refinement automatically; separate
rough previews from this costly followup and preserve actual gold/progress.
The screenshot also exposes historical raw report copy `unlock supply_cap_5
(0)` and opaque wall-edit text; include these in the outstanding wording audit.
