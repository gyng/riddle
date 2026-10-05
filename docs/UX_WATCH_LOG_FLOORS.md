# Accurate live-watch combat log — 2026-10-06

A real saved camp starting legally at D5 shows D5 in the watch HUD but D1 in
its combat log. The log initializes before `send()` from stale/missing camp
live state. Initialize it from the authoritative returned run snapshot instead;
resume uses current depth, and an unpaid waystone toll uses actual D1.
Descend events continue changing floor attribution in playback order.

The existing log renderer supports healing, telegraphs and pickups but its
caller filters them out. Include those meaningful events, preserving basic
colour styling, item silhouettes and the80-entry bound. No new log box, controls
or simulation edits. Do not queue movement events merely to filter them later.

Verify scoped UI batches at400/1440: non-D1 starts, resumed floor differing
from chosen start, toll fallback, ordered descent, warning/heal/item lines and
bounded log. Real WASM D5 with normal roster refresh must show consistent live
labels. Inspect headed desktop/mobile screenshots, tsc/build/copy/diff, push;
no deployment or full balance audit for this rendering-only change.

## Results

Six scoped UI cases (400/1440 × start5/resume14/fallback1) pass in17.2s.
Each checks the starting floor for warning/heal/item rows, ordered descent for
later damage, basic styling and the80-line bound. Uses explicit fake wire
batches through the actual watch renderer; does not certify core simulation.

Actual headed release-WASM saved camp-0 legally setStart(5), normal runs=1:
watch header/log D5, desktop hero roster Live D5, zero console warnings.
Desktop/mobile screenshots inspected and shown. Mobile has no horizontal
overflow; mobile watch has no desktop roster (no mobile-roster claim).
Artifacts scratchpad/death-actions-20261006/live-refresh* and live-mobile*.
TypeScript/production build, copy1492 and diff pass. Existing build chunk-size
advisory remains. No Rust changes/full balance audit/deployment.

The earlier stale Resting label was diagnostic-only: runs=0 disables the
existing watch roster refresh. Restored runs=1 verifies existing behavior;
no roster code changed. The earlier Captain90s wait remains uncertified:
this capture reaches ordinary D5 combat, not the Captain encounter.

Next evidence-based UI gap: mobile watch footer labels clip (`Speed`,
`Town menu`) inside narrow tile buttons. Audit their text/geometry without
adding controls, then finish live boss coverage with actual player speed controls.
