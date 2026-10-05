# Honest watch status — 2026-10-06

The watch pauses its picture, not the ongoing simulation. Actual headed capture
showed roster Live D6 while picture/log remained D5; the badge always says Live
delve. Preserve both authoritative live roster and ordered picture/log.

Badge states: same-floor unpaused Live delve / continues away; paused Watch
paused / continues away; different-floor unpaused Watching D5 / Live D6; core
run ended but ending picture pending Run ended / Watching D5. No tick-delay
precision claims, new controls, or changes to simulation/clock/pause behavior.
Update only when displayed state changes. Existing badge disappears at final
run exit. Keep short labels, colours and compact mobile layout.

Scoped tests prove paused core progress while picture stays held, resume floor
catch-up, end-pending state, matching and different floors at400/1440. Actual
WASM headed pause/catch-up screenshots, controls/chrome regression, tsc/build/
copy/diff. Push, no deployment/full balance audit. Captain already has its own
atlas slot and name fallback (`captain`); inspect actual labelled frame before
changing art/identity based on a capture of the goblin summoned beside him.

## Results

400/1440 scoped UI walks pass14.6s. Explicit batches through actual watch UI
prove the core reaches D6 while held picture/tick stay D5; synchronous resume
shows Watching D5 / Live D6 before picture catches up to D6; ending while
paused shows Run ended / Watching D6. Pure label cases cover matching floors,
paused and ended precedence. This is client playback coverage, not core balance.
Existing watch-console five-width controls/geometry walks pass14.5s.

Actual release-WASM headed saved camp-1, legal D5 start, real Fast/Pause/Resume
controls at400/1440: paused and resumed badges match, no console warnings or
horizontal overflow; screenshots inspected/shown. Those captures stay on D5
and do not independently certify the cross-floor label. Artifacts
scratchpad/watch-status-20261006/{400,1440}-real.json and PNGs.
TypeScript/production build, copy1500 and diff pass; existing chunk advisory
remains. No clock/simulation/roster changes, full balance audit or deployment.

Captain audit: atlas has its own96px master slot, renderer kind fallback is
captain. No proven missing sprite/name bug from a crowded later rally capture;
no art replacement made. Next obtain the exact labelled rally frame (or its
recorded replay), inspect its sprite/name in isolation, and then choose a
concrete content/graphics change from player evidence.
