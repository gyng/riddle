# Earned Training icons — 2026-10-06

Close the remaining icon gap in Training on return reports and death screens.
Reuse the existing style/tactic silhouettes for level beats and the recorded
boss's portrait for drilled counters. Resolve only canonical printed core
names, not the current selected hero's loadout. Unknown old/future names keep
a generic progress icon and their original text. No new art or game changes.

Acceptance: all badges have decorative32px art with8px spacing and wrapping
labels, including long future names at320/400/1440; six canonical boss labels
resolve their own art; historical bloodline attribution and exact reward text
stay intact; no fake gains, no duplicate details, no missing-asset requests.
Verify existing report/death/ownership tests, real earned WASM screenshots,
read-only rendering, build and copy lint. No deployment.

Verified: training-icons covers all six core boss_short labels, Bold/Light
Hands/Boss Focus art, unknown boss/style fallback, exact labels, decorative
semantics,32px art/8px gap/center±1px/wrapping at320/400/1440. Existing
run-training/bloodline-training/report-bosses/death-hero checks pass:5/5 suites
in7.7s. Build/typecheck/copy1556 and diff clean. Actual previously earned
seed3 next8h Guarded/Bold case reports rendered in current real WASM at400/
1440: images decoded, badges bounded, exact original reward strings, opening
report leaves engine save byte-identical, no page errors. Screenshots shown;
scratchpad/training-icons-20261006/results.json. Reports were recorded by the
prior native/WASM exact-output comparison; this round does not rerun balance
or claim new gameplay outcomes. No core/new art/deployment.
