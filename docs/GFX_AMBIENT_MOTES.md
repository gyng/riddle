# Ambient motes — 2026-10-06

The owner requested moving air and lightweight pointer interaction instead of
uniform upward scrolling. Replace the repeated gradient tile with individual,
prepainted CSS sprites. Each follows changing lateral currents with inertia,
a different upward speed, and a short, local mouse wake. No game state changes.

Acceptance: 16–36 decorative sprites, at most 30 transform updates/s; no layout
reads in the loop, full-screen canvas, animated backgrounds, blur or blending.
Pointer influence stays within 100 CSS px, decays within 1.2 seconds, and speeds
are bounded at 75 px/s. Layer cannot intercept input. No layer on watch,
juice-off, reduced motion or hidden documents; screen replacement removes the
old layer/animation. Resize measurements are event driven. Mobile gets drift
without touch interaction. Validate trajectories, locality, lifecycle, input,
actual WASM screenshots and GPU frame timing. No deployment.

Verified: client checks at400/1440 cover independently moving sprites, lateral
currents, local wake and speed bounds, real Settings clicks, replacement with
one layer and stopped detached sprites, hidden-page pause/resume, reduced
motion, watch exclusion and juice-off. Six targeted UI suites pass14.2s.
Actual WASM camp screenshots400/1440 and desktop WebM captured. GPU probe:
D3D12 NVIDIA RTX3080. Five-second phone samples both~16.8ms p95/no frames>33.5ms.
Desktop without recording: new299frames/old300, both16.8ms p95, no>33.5ms and
zero layouts. Script time new440ms/old249ms over5s (whole page, not isolated
particle cost); task time922ms/621ms. Recording depressed desktop cadence, so
that sample is retained separately and is not the frame-rate claim. This is
one machine and brief samples, not a low-end-device guarantee. Evidence:
scratchpad/ambient-motes-20261006/{results,timing-unrecorded}.json,
desktop-motion.webm and400/1440 screenshots. Build/typecheck/copy clean.
