# Save manual construction before showing completion — 2026-10-06

Evidence: reduced-motion first home reload can restore the empty town before the
one-second debounced save. The pagehide handler writes the last cached engine
snapshot, which still has no home or hero. Preserve the existing fast paint for
ordinary changes, but save a manual building before announcing/painting its
completion. Keep the build guard active while that snapshot is fetched. No
simulation or build costs change; no arbitrary extra delay or timeout.

Acceptance: real WASM immediate navigation from the first home change notification
restores home and the same single hero at400/1440, normal and reduced motion.
While the save response is delayed, house completion is not painted, both build
surfaces remain deduplicated; rejected construction does not save or pretend to
succeed. Existing manual-build/selection/death checks, typecheck/build/copy lint.
Headed before/after screenshots. No deployment.

Verified baseline real headed WASM: first construction notification navigates
to a clean URL with no fresh flag; restored homefalse/hero count0. Same sequence
after fix restores hometrue/hero count1, no explicit test flush. Read persistence
before waiting for sprites; final screenshot waits for art loading only.

Final town-save39 checks (four400/1440 normal/reduced immediate navigations,
exact hero slots, delayed snapshot/build dedup, refusal), first-plot320/400/1440,
selection28 and death44 all pass11.7s. Build/typecheck/copy1535 pass; existing
bundle advisory. No Rust/WASM rebuild required: application ordering only.
The normal rules refresh may schedule a later debounced save; the first cached
snapshot already contains construction. Existing storage/serialization failure
handling remains play-on; this is not a private-mode/quota persistence guarantee.
Evidence: scratchpad/town-save-20261006/. No full balance/performance claim.
