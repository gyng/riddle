# Keep the watched hero visible — 2026-10-06

Real seed2 first-watch GPU probe (40 samples100ms apart, width400/1440,height900,
DPR1.5, speed4,runs1) shows entire or partial sprite clipping9/40phone,17/40desktop.
Camera targets are constrained relative to hero; actual spring position is not.
Fast travel moves the hero farther than the camera can follow before the frame.

Constrain the rendered camera after spring integration using actual hero sprite
bounds and view dimensions. Preserve room holds/targeting/spring in their normal
range; reset axis velocity only if constrained. Include sprite body, margin and
pixel snapping allowance. Apply map/fight, load/seek/resize alike. No simulation,
watch speed, art or HUD changes.

Acceptance: repeat real first-watch probe at both widths, zero clipped hero rects
among drawn unfaded samples; inspect before/after screenshots. Controlled GPU
renderer checks fast travel, fight framing, seeking, stationary and resize at
phone/desktop sizes. Existing watch-status and relevant renderer/client checks,
build/typecheck/copy pass. No broad FPS or balance claim; no deployment.

Verified: real headed WASM seed2 repeated40samples per width:0/40 clipped
phone and0/40 desktop versus9/40 and17/40 before. Same sampling protocol;
wall-clock samples/screenshots land at different replay ticks, not a paired
pixel comparison. After screenshots atD2 show full hero; before screenshots and
rectangles retained. GPU data includes timings but no FPS/performance claim.

New tools/watch-framing-check.mjs:15 cases/87 drawn-contained samples each
400/1440 (30 cases/174 samples total): load, moving map/fight, distant fixed
fight focus, three seeks per mode, short-height/narrow resize, stationary loaded
view, restored size. Fixture verifies at least3 distinct travelled positions.
It fails against original HEAD renderer (hero y842px in660px view). Temporary
baseline source removed; optional RIDDLE_RENDER_MODULE is diagnostic override.
First fixture compressed all movements into too few sampled frames; spread
moves over80tick intervals. Stationary assertion initially covered a spring
still settling after resize; now fresh loaded stationary view tested separately
from moving/resize containment. Production safety code unchanged by those fixes.

Existing watch-status, GPU diagnostics741 and live-roster38 pass15.3s isolated.
Build/typecheck/copy1524/diff pass, existing bundle advisory. No Rust/WASM/art
or sim speed changes. Real probes and logs scratchpad/watch-framing-20261006/.
Screenshots inspected and shown. No deployment or full balance certification.
Next: combat log repeats an attack as a hurt line and loses names for actors
slain in their first batch. Use actual events to make combat readable; retain
non-attack environmental damage and true identity fallback.
