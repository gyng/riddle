# Repeated package queries — 2026-10-05

The real shipping Packages control repeats a full query after a plain lineage
refresh or an appearance change. One late-camp diagnostic measured8.662s for
the first read,7.823s after refresh and7.902s after appearance; all complete
price vectors match. These are individual readings, not speedup acceptance.
Private evidence:scratchpad/package-refresh-20261005/baseline.json.

Reuse a bounded result/promise at the engine lane boundary. Rust owns the whole
query key: base and every applicable ordered candidate's own/wall/raw-rule
panel keys, move/slot/purchase price, best depth, simulation count and execution
settings. Include the immutable balance profile. Never project game inputs in
TypeScript. Compute key and prices on the same synchronized mirror, without
another reload between them. Preserve UI snapshot/race guards and old-engine
fallback. Drop failures; isolate engines; clear memo on native rebuild events.
Return independent reply objects so callers cannot modify cached values.

Accept only with exact full replies/saves on early/late/tuned camps, refresh
and appearance reuse, and remeasurement after hero/Legacy/rules/loadout/price
changes. Require three real shipping repeated-read pairs below100ms median
and at least90% quicker; no cold-query regression above5% over seven pairs.
Run controlled concurrent/mutation/failure/rebuild lifecycle checks, scoped
package client gates, core tests, copy/tsc/clippy and full release acceptance.
Verify published UI and artifact identity; show milestone screenshots.

Prototype native key cost1.239/5.312/3.297ms on the three frozen camps;
keys14.9/69.8/47.9KB. Complete prices after cosmetic and rest-clock changes
match fresh originals on every camp. No simulation budgets or seeds change.

Local acceptance: FULL623s PASS (538 core/1 ignored,11 tooling, copy/tsc/clippy,
shipping engine/site, fresh metrics/QA and all18 selected fortnight cases).
Sixteen lane lifecycle checks and46 UI snapshot/retry checks pass. Seven quiet
shipping cold-query pairs per camp show no regression: early1508.9→1450.3ms,
late7687.8→7629.4ms,tuned8328.3→8309.6ms. These small differences are not a
speedup claim. All24 full price/save pairs match exactly, including warmup.

Actual shipping late-camp UI: first comparison9046.8ms; three plain refreshes
14.9/7.3/10.1ms and three appearance changes86.4/93.0/72.8ms. Refresh median
10.1ms, appearance median86.4ms; both below100ms and over99% quicker than the
first comparison. Every complete price vector matches. No FPS, offline tick
or whole-app performance claim. The first check timed out because an unscoped
selector clicked the inert fading clone of a closed sheet; scoped live-panel
controls pass. Private evidence:scratchpad/package-refresh-20261005.

Published source43c9a09, Pages37314788570 SUCCESS. Public36 real UI checks
400/1440 PASS, no page errors; complete deployed HTML/entry JS/WASM match CI
artifact bytes exactly. Both public screenshots shown inline.
