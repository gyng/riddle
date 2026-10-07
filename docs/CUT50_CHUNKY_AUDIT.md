# Cut50 — close gaps in the shared chunky game chrome

Audit actual earned management screens at400/1440 with computed control styles
and screenshots: Hero, Class, Tactics, Forge, Workers, Settings, Chronicle, Runs,
Appearance, Savings and Quest. Review home/watch/report/death through current
playthrough artifacts and source. Fix controls missing the material system via
explicit shared game-control/game-inset components, not blanket button styling.
Keep intentional gems/studs/portrait rims, map hits, resource animation,
item/trait silhouettes and plain combat-log overlay. Charts/status text stay quiet.

Findings: run-log tabs use dark parchment ink on dark header and32px targets;
run folds are flat rounded40px bars; interactive Chronicle links flat rules;
Appearance uses old bottom sheet and unframed choices; rename input retains8px
radius. Route Appearance/rename through management windows and shared material
controls, retaining labels, handlers, parent return and save semantics.

Acceptance:11 actual earned windows ×400/1440 no overflow/errors and read-only
full save unchanged, screenshots before/after; run tabs/folds ≥44px with framed
material and readable labels, actual tab/fold interactions unchanged. Appearance
choices fit320/400/1440, ≥44px, current marker and route lifecycle retained.
Rename keyboard/save unchanged; related existing chrome/runs/appearance/class
checks plus TS/copy/build/diff. No deployment or independent95 certification.

Additional actual inventory inspection: Supplies and Stored Gear panel buttons
bypassed the shared recipe, with muted ink on dark flat cards. Town labels also
painted over panels (map tag z3 versus panel-host z2). Shared recipe now covers
panel-body and ordinary card buttons; compact title plates, lit rarity labels,
secondary text/gold on dark surfaces; panel-host z6, above map labels and town
edge overlay. Keep item icons unboxed. Four actual400/1440 inventory captures,
full save equality, no overflow/errors and measured panel-above-town. Companions
is not built in earned fixture; attempted command times out, not claimed as
passed/earned. Source reviewed instead. No new state or purchasing behavior.

Acceptance: actual earned11 windows×400/1440, plus2 inventory panels×400/1440;
26 read-only views, zero browser errors/overflow. Final artifacts reviewed and
reviewed-panels under scratchpad/chunky-cut50; completed intermediate captures
and raw audit logs retained. Before/after Appearance, Runs and Supplies shown.
Actual320/400/1440 Runs→Heirs→Runs/fold close/reopen, framed44px controls,
Appearance choice/current marker/parent return, exact full-save inspection,
keyboard rename→actual Rust set name/unchanged rows→flush/reload pass. New check
waits for panel animation to settle before measuring touch geometry. Initial
unsettled appearance rectangle was36px; settled unchanged44px minimum passes.
Earlier old CSS min/radius overrides fixed in shared recipe, no threshold changes.

Final related chunky-controls/chrome184/appearance15eachwidth/class-switch/
item-icons114eachwidth5/5PASS31.2s, /tmp/riddle-cut50-panel-client.log. Tiny final
card recipe mapping and secondary label colour extension are presentation only;
final actual screenshots/build/TS/copy1739zero/diff pass. Prior HUD resize failure
retained, unchanged final chrome gate passes. HMR-interrupted intermediate audit
retained, fresh final ports used. Broad suite/hidden states/independent95 remain
uncertified. No Rust/WASM change or deployment. Shared5219 retained; owned audit
servers closed after final inspection. docs/UI_MATERIALS.md records component
usage, exceptions and coverage boundaries.
