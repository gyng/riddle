# Ready construction plots — 2026-10-06

Actual seed3 first-session walk: first run returns143 gold atD4; town offers
Build blacksmith · first gold home above a tiny round plot. The ready building
uses the old tablet while the house has a painted foundation/carved button.
Unify ready plots with that shared module. Forge uses its menu name; other
physical buildings retain plain storehouse/kennel/bank names. Construction is
free once the Rust milestone is met (town::construct); no invented price.

Ready plots: painted empty foundation, full matching click target, compact
carved Build/name/Free plaque. Keep milestone reasons on unready plots. Never
render an unbuilt building, auto-build, or infer readiness from gold. Preserve
busy guard, keyboard controls, pre-paint save and existing construction reveal.
Reuse existing house_plot_v2 art/skin packing; no new bitmap needed.

Acceptance: real legal house→send→first positive return→forge build, phone and
desktop captures. Fake derived plot fixtures cover all four ready positions,
unready trigger/primitive art fallback and resize. No marker/plot overlap,
horizontal overflow or page errors at320/400/1440. Existing construction/save,
town and selection checks; build/typecheck/copy lint. No deployment/balance change.

Verified: headed real WASM seed3 at400/1440: first positive return, ready
forge, explicit free build with unchanged gold, then forge menu. All four later
plot positions pass24 phone/desktop geometry, resize and failed-art cases;
unready plots retain their trigger and cannot build. Fixtures explicitly settle
renderer resize before measurement. Foundation scales down near map edges,
marker stays inside the town and flips above the plot when needed; connector
tracks the physical plot. Existing first-house separation check now accepts
above or below while still requiring no overlap. Save39, selected tactics28,
first-house320/400/1440 and valid town day0/desk checks pass. Build/typecheck and
copy lint1540 pass (existing bundle-size advisory). No Rust/WASM/balance or
deployment changes. Evidence: scratchpad/ready-plots-20261006/ and the actual
first-session walk in scratchpad/first-session-followthrough-20261006/.
