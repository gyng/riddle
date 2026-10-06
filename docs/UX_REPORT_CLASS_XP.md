# Visible class growth on return — 2026-10-06

Fresh real seed3 house/first-send report shows runs, depth, gold and an upgrade,
but hides earned39FighterXP under details. Expose a compact Class XP section
below the three summary tiles: class, earned XP and earned level count. Keep
Legacy distinct and the detailed diagnostics folded. No invented current level
or selected-hero identity on historical rewards.

Multihero: add optional-compatible per-bloodline `xp` report arrays, containing
that slot's authoritative class XP report. Merge slices by slot and class; keep
top-level selected-slot XP behavior unchanged. Render each slot's earned XP by
reported bloodline name, never by the current selected class or heir. Older
reports fall back to their explicit top-level class XP without assigning a slot.

Acceptance: Rust report captures two different classes, round-trips old missing
XP, preserves selected top-level compatibility and does not repeat later gains.
Mounted report400/1440 shows gains before details, hides zero gains, distinguishes
XP/Legacy, merges same/different classes without losing gains, and retains owner
labels through selection changes. Build/typecheck/copy, fast Rust tests/clippy,
fast WASM rebuild and headed real first-report screenshots pass. No deployment
or balance/simulation changes.

Verified: BloodlineReturn now carries xp arrays cloned from each slot's Rust
report; selected top-level XP unchanged. Arrays support client slice merging
by class without reassigning earlier gains to the latest class. Missing old field
loads as empty; older raw client reports use the explicit top-level XP fallback
without attaching an inferred slot. Known slot arrays suppress that fallback.
The summary reports earned levels only, never today's selected class level.

Core test covers Fighter/Rogue returns with selected hero either running or
waiting, positive unselected gain, selected compatibility, rogue-class gain
conservation, wire round-trip, missing old field and no repeated later gains.
Workspace559pass33.34s/1ignored; clippy alltargets clean. Full suite preceded
the selected-waits fixture extension; expanded targeted test passes0.07s.
Fast WASM rebuilt5236591bytes, SHA256
 ae07e9503204d65d236072dbe0183b8fd8608d0376dfd70f6411fc41528047c6.
No simulation/balance inputs changed; no full statistical gate audit claimed.

Final report-class-xp20checks/width400/1440 (40total) pass3.0s: visibility,
folded details, zero/level-only gains, slot ownership, selected-waits case,
selection history, class-changing slice merges, input immutability, old fallback,
known-empty suppression and layout. Initial combined34XP+32report-actions+
40recent-progress pass3.6s; subsequent extensions fixture-only. Final rebuilt
WASM build/typecheck/copy1527/diff pass; existing bundle-size advisory.

Headed real seed3 fresh house/manual first Send after rebuild: phone400/desktop
1280 first report Fighter+39XP visible before folded details; eight-hour return
+372XP/+1level. Both ten-checkpoint walks complete30.7/31.9s, no console/page
errors; worker trunk uses two scripted engine sends/hiring before absence, not
an all-UI worker tutorial. First-run report screenshots and phone away screenshot
inspected/shown. Baseline first report hid39XP. Evidence:
scratchpad/first-growth-20261006/ and scratchpad/report-class-xp-20261006/;
root core/clippy/client/build/WASM logs. This is an automated reward-clarity
review, not a human fun score, production performance or deployment claim.
