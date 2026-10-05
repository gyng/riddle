# Appearance from hero details — 2026-10-06

Camp hides the old console portrait, so its cosmetic picker is not a discoverable
camp action. Reuse existing openLooks from an Appearance button alongside Change
class in hero details. Keep cosmetic choices optional, no new console controls,
art/defaults or game rules. Disable if bridge lacks setLook; allowed while away
per existing cosmetic API. Rename picker label to Appearance.

Refresh the parent hero window when selected identity/class/look changes,
including while hidden under the picker; keep expanded Details state. Dispose
subscription on close and avoid repainting on routine live pulses. Successful
choice returns to the same updated hero window; unchanged choice spends/writes
nothing. Failures leave current appearance authoritative.

Verify actual UI button→three existing portraits→choice→updated hero parent,
roster and report. No gold/Legacy/class/XP changes. Old/missing bridge, no-op,
away mode, subscription disposal,320/400/1440 touch/overflow and headed real-WASM
save-load/per-slot choice screenshots. Scoped client/typecheck/build/copy/diff;
no Rust changes/WASM rebuild/full balance audit/deploy.

Implemented and verified: 45 route/lifecycle checks across320/400/1440 pass
5.4s, including no-op/dismissal, selected name-only refresh, expanded Details,
closed subscription, missing bridge and away cosmetics. Existing appearance54,
names27 and report57 checks pass5.4s batch. Production build/typecheck passes;
large-chunk advisory remains. Copy1504/diff clean.
Headed real-WASM actual Appearance button flow at400/1440: Wren female/Vale cat
choices return to updated parent, roster and exact reload retain each look;
gold49, class, XP and Legacy unchanged, no warnings/overflow. Expanded Details
is fixture coverage. Diagnostic captures wait for closing sheet ghosts to
disappear. Fixture engine lineage getter corrected to prevent background reads
from unrelated fake lineage; geometry waits for opening animation to settle.
Artifacts scratchpad/hero-appearance-20261006. No Rust/WASM/full audit/deploy.
