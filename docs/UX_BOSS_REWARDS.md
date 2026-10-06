# First boss victories on return — 2026-10-06

Actual seed3 Bold next8h report: Bloat Mother first slain,12 deaths, Gold home0.
The first kill is present in report.bests and lead, but both render only under
Details. Show a compact Boss defeated section before the folded details, using
existing foe artwork and the report's explicit first-kill deltas. Preserve the
real gold/run/death figures. No extra reward, unlock, or balance changes.

Multihero summaries currently discard unselected heroes' bests. Add a defaulted
per-bloodline bests array; keep top-level selected-hero compatibility. Merge
arrays by bloodline across report slices. Show explicit reported owners in
stable slot order; never infer a victory from today's slain walls or selected
hero. Old summaries without the field may use top-level bests without invented
ownership; known empty arrays suppress that fallback. Deduplicate a boss within
one owner, retaining separate victories by different owners. Unknown explicit
boss ids must retain a readable name and primitive fallback.

Acceptance: all first boss victories visible outside Details; no boss rows for
ordinary records, counter facts, historical slain walls, or later empty reports.
Mounted checks at400/1440 cover ownership, deduplication, slice merges, old wire,
live selection changes and narrow layout. Core tests cover unselected bests,
empty selected reports, serialization/default migration and no repeated gains.
Workspace fast tests/clippy, real WASM rebuild, client checks/build/copy lint.
Inspect real saved Bloat Mother report and a real multihero absence on phone
and desktop. Show screenshots. No deployment or broad balance claim.

Verified: workspace fast560 tests passed,one ignored; clippy clean. Mounted
report-boss54 +classXP40 +newchoices46 +reportactions32 =172 checks passed.
Build/typecheck and copy1534 passed (existing bundle advisory). Real rebuilt
WASM5237180 bytes, SHA7554108a1f234dc3597d9473e5d5ca5b2fb552319541fb28786c065ae4fc1acd.
Actual firstboss saved-town Bold report shows Bloat Mother and Gold home0.
Legal second bloodline added and selected before8h absence:29 runs; slot1
first slays Mother while selectedslot2 has no boss gain. Phone400/desktop1440
show Bloodline1 ownership outside Details with no overflow/page errors.
Final captures use the actual absence report route (While away); early before
captures used the normal run report route. Accelerated saved-state engine QA,
not a fresh human playtest or broad balance/performance claim.
Evidence: scratchpad/boss-rewards-20261006/. No deployment.
