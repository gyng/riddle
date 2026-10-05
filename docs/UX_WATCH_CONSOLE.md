# Readable watch console — 2026-10-06

Actual400px watch screenshots show Speed and Town menu clipped in an eight-slot
command card that has only two actions. Extend shared renderConsole with optional
compact layout: retain its eight-action overflow handling, but omit decorative
empty slots. Watch opts in. Its two chunky tiles share the available middle
column; narrow320px phones stack them with horizontal icon/text. Preserve font
sizes, icons, portrait, round pause gem and their actions. Other screens retain
existing console layouts. No new controls or simulation changes.

Verify320/360/400/840/1440: exactly two footer action tiles, no fillers, complete
labels inside buttons,44px touch minimum, no overlap/overflow; speed sheet,
mode selection, pause and Town menu continue working. Real WASM desktop/mobile
screens and actual player-controlled boss walk; scoped geometry/client checks,
production build/copy/diff. Push without deploying. A boss timeout is diagnostic,
never permission to change rules or claim the encounter passed.

## Results

Five320/360/400/840/1440 client walks pass17.6s: complete label ranges inside
buttons,44px minimum, no fillers/overlap/overflow, actual speed selection,
pause/resume and Town menu. Existing chrome184 checks pass28.8s.
TypeScript/production build, copy1492 and diff pass. Existing chunk advisory
remains. Actual headed WASM400/1440 watch captures inspected/shown; warning-free.
No simulation changes, full balance audit or deployment.

Boss diagnostic now uses actual Fast button, not an unchanged90s wait.
Camp-0 ends before a Captain capture on both sizes. Stronger existing camp-1
legally starts D5 and reaches a visible Captain in17.2s desktop walk. Final
predicate requires living visible renderer entity and on-screen rectangle;
immediate pause/screenshot shows RALLIED combat. Initial predicate counting
fogged entities was rejected after screenshot inspection; slower screenshot
also caught a later fight. This is encounter coverage, not a full boss victory,
mobile boss encounter or King/progression audit. Artifacts
scratchpad/watch-console-20261006/*strong-live* and strong-visible-final.log.

Next graphics/readability audit: Captain identification in the rally scene,
and clarity when the engine's live floor moves ahead of the watched picture.
A prior delayed capture showed roster Live D6 while picture/log still D5;
verify semantics before changing them. Do not change game rules to ease QA.
