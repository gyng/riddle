# Readable report actions — 2026-10-06

Reports must not show internal unlock IDs or ambiguous zero-price parentheses.
Use existing catalogue labels; once the catalogue arrives use its current price
formatter. The initial/failure fallback shows the readable name without making
a price promise. Keep available cards/actions and pre-pen visibility unchanged.

A wall offer names its target floor and sampled chance, with before/with-fix
labels and sample count. Replace opaque sequential edit strings with a Review
rules fold showing current and suggested ordered rule chips, including item
icons. Show a changed starting floor explicitly. Apply keeps exact Rust rules
and start; no simulation/balance behavior changes.

Acceptance: mobile/desktop fixtures contain no raw unlock IDs, `(0)` prices or
raw wall edit strings; review shows exact ordered rules and item icons; apply
uses exact offered rules/start. Existing report/geometry/forecast checks pass.
Update historical Cut25 checks for the simpler report and one-tap Forge, while
retaining real trace, sheet-stack, lane, purchase-price and rendering checks.
No loosened runtime bars. Tsc/copy/build; checkpoint screenshots; push, no deploy.

Validation: 32 focused checks across 400/1440 pass, including readable initial
news/reel/pending text, catalogue gold/mark prices, exact ordered current and
suggested rules, item icons, fold opening and exact apply/start. Existing Cut24
report checks (20), Cut29 wall checks (3), geometry (176) and forecast paint (5)
pass. Tsc, production build and copy lint (1485 literals) pass. No Rust edits.

Actual shipping-engine walks at 400/1440 pass: the real offer targets D21 with
48 samples, changes start D1 to D19 and shows Before 4% / With fix 81%. Applied
rules and start exactly equal the Rust offer. Visible report text contains no
raw unlock IDs, zero-price parentheses or sequential edit strings. The walk
caught an additional raw-ID source in the lead news; the shared formatter now
covers news and reel as well as pending. Screens/controls are under
scratchpad/report-actions-20261006/.

Historical Cut25 checks now follow the owner's simpler report, one-tap Forge
and single Speed menu. Exact purchase price, one purchased step, live 1× rate,
mode persistence and original drain/black-frame timing limits remain checked.
Management fixtures explicitly boot a mature town instead of inheriting the
preceding watch session. Section selection allows a local report/sheets/caps/
unlocks/forge run in 9 seconds; the Speed section takes 8 seconds. These are
scoped checks; the complete default walk remains available.

The full default Cut25 walk passes all 22 checks in 124 seconds, including
the real-wasm black-frame check. This supersedes the historical stale-selector
failures recorded in the earlier Forge/icon handoffs. The entire frontend
suite and native balance table were not rerun for this frontend-only change.
