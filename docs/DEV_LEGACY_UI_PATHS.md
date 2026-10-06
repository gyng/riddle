# Legacy UI test paths — 2026-10-06

Current fresh camps hide the advanced camp controls until a completed run;
Rules is an explicit hero-roster action, and active heroes own the default panel.
Legacy literal-rule fixtures must describe an established player when testing
supplies, forecast or editing. Do not force-click hidden nodes, expose controls
in product code just for tests, or swallow navigation errors.

Migrate cut21's shelf fixture to an established camp with actual engine-backed
supplies. Preserve no-row/found markers and all report/start-picker checks.
Migrate ui's first-screen assertion to hidden advanced controls, then use Rules
when checking editing after progress. Diagnose subsequent outdated assumptions
against current contracts before changing expectations. Record any outstanding
legacy failures; targeted success does not certify the full suite.

## Targeted commands

The default runner ui command runs every section as bounded independent jobs;
direct node tests/ui.mjs retains the serial full walk. Choose one or several for a
local edit; these are targeted checks, not a replacement for the full suite:

```
(cd web && node tests/run.mjs ui:frame)
(cd web && node tests/run.mjs ui:qa)
(cd web && node tests/run.mjs ui:stall,keep)
(cd web && node tests/run.mjs ui:cut18,cut19 cut21)
```

frame covers the main frame/reveal/panel/watch/report walk. qa includes both
historical QA K/L sections. stall, keep, cut18 and cut19 retain their complete
respective checks. Each selected section initializes its own fixture; the QA
fixture includes prior progress. Invalid part names fail. Direct ui runs stream
assertions as they finish. The suite runner keeps the blocking Playwright
locator/cause ahead of assertion failures in its bounded output; ANSI styling
is stripped. Formatter checks cover errors, assertion lists, no-error success
lines and a long failure list that previously crowded out the locator.

Verified independently: qa41 checks, stall3, keep2, cut18 11, cut19 12, cut21 19
all pass. The seven-job check took39.3 seconds; this is an observed targeted
parallel run, not a paired speed benchmark. first-plot, hero-appearance and
report-supply-limit pass together10.8s. Syntax, formatter checks and diff clean.
The full frame section retains one FAIL: watch header changes from5th to6th
heir on the final frame. Do not change this expectation. renderWatch currently
passes only watch:true to renderBar, whose repaint reads the newly inherited
Lineage.heir. Next: retain the viewed run's identity across that refresh and
verify real-WASM final-frame behavior. Full client suite remains uncertified.
No game truth, balance, player UI, WASM build or deployment changed in this pass.

Follow-up: watch identity fixed in UX_WATCH_HERO_IDENTITY.md. The original full
ui command now passes all132 checks in135.4s. Its default coverage remains all
six sections; targeted sections still do not certify the full client suite.


2026-10-06 default dispatch follow-up: shared UI_PARTS keeps runner expansion
and serial coverage aligned. Quiet counterbalanced serial/split/split/serial
measurements all pass132 checks; median132.5s→40.1s (69.73% reduction). Bare
runner ui and default suite now expand to six bounded jobs; targeted commands
remain targeted. DEV_UI_PARTS_DEFAULT.md records scope and exact measurements.
