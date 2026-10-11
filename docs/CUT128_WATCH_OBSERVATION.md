# Cut 128: observe the exit setup and card coverage in the browser

2026-10-11. Cut127's complete client attempt was stopped after confirmed
failures, exit143; it is incomplete acceptance. Initial five timing groups
passed, including51 fight checks. Failures: qaj's initial watch pause click
could not find its gem within2s; clarity:hold seed7 observed4 matching cards
among61 samples, below its5-card-sample minimum. No mismatching card/HUD
content was reported. Original logs/process tree retained.

Isolated unchanged qaj exit diagnostic passes; same-run Node/browser timer
card probe sees12/11 cards with no mismatches. Neither reproduces the failed
frames. Their exact causes remain unproven; do not claim a game bug fixed.

1. qaj's exit setup installs its existing exit recorder and presses the actual
   visible, enabled, unoccluded public pause control in one browser operation.
   Retain20s watch readiness and2s action budgets. No forced/hidden click,
   engine clock edit, fabricated exit, or ended-run exception. Resume and all
   COLLECTED/tick/depth/bank assertions stay unchanged.
2. clarity's card/HUD observations stay20s and no faster than one sample per
   100ms, read atomically on browser frames. Keep cards≥5 and zero mismatches,
   both seeds, and every existing depth/carry comparison. Do not count five
   rapid frames as five100ms samples or lower the coverage minimum.
3. Whole qaj and clarity:hold pass, then complete current156-group suite.
   Preserve original and diagnostic logs. No application, sim, art or numeric
   gate changes; preview/app stayb6df4f2. No blind raters before full acceptance.

Verification: targeted whole qaj PASS38 checks94.6s; clarity:hold PASS11
checks81.1s; runner PASS2/2 in176.3s, terminal0. Source and all numerical
assertions unchanged. Original failure causes remain unproven. Full156-group
acceptance remains required before blind release. Logs archived in acceptance.
