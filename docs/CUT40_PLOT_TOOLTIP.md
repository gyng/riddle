# Cut40 — show the next plot, not retired track progress

Actual earned-home screenshot shows the unbuilt plot with a tooltip saying
track · one way the family grows ·40stages. The plot acts on town construction,
not tracks. Replace its glossary host with existing rich game-object tooltip.

- Tooltip names the actual next building using the current town name mapping.
- Locked plot shows the Rust-owned trigger; ready plot shows Free/manual build.
- Hover/long-press must leave complete engine save unchanged. Input uses shared
  tooltip behavior; click construction remains the existing explicit action.
- Check real WASM empty and earned town at400/1440, no page errors/overflow.
  Capture the actual tooltip; no styling/layout/automation changes.
- Run TS/copy and existing shared tooltip checks.95 remains unverified.

Acceptance: real WASM400/1440 fresh/earned hover shows actualhouseFree/
kennelfirsttame. Phone trusted touchStart/End long-press on visible marker
opens the pinned tip and leaves the complete engine save byte-identical.
Four states zero overflow/pageerrors; actual Rust town next/next_trigger fields
match the displayed requirement. Evidence scratchpad/plot-tooltip-20261007,
/tmp/riddle-cut40-real-touch.log, session20980terminal0. Earlier diagnostic
mouse long-press timeout retained: tooltip intentionally ignores mouse holds;
corrected probe uses Chromium touch input. Final screenshots shown.

Sharedtips70/70PASS and enemy-tips400/1440PASS,2/2gates137.3s,
/tmp/riddle-cut40-tips.log session42624terminal0. TS/copy1734zero/build/diff
PASS; no Rust changes/WASM rebuild. Broad91/112 remains a failing historical
source verdict, not certification of these scoped fixes.95fun unverified.
No deployment.
