# Current UI regression harness migration — 2026-10-05

The owner-approved first-home/Forge simplification supersedes historical Cut23
Forge forecast-on-open and two-tap kit purchase expectations. Bring that phase
forward without loosening accounting or picker gates: three slots; each next
upgrade names an item and exact Rust-owned price; opening/buying starts zero
kit queries; one tap spends exactly once and advances exactly one rung;
unaffordable controls disabled; future prices folded; explicit Forecast makes
one query and renders all slots. Preserve anchored picker no-overlap, outside
tap non-mutation, supply target dimensions/position, and paid supply confirmation.
No balance, timing, tooltip-density, or determinism bars change. Use scoped
client checks; no engine rebuild or publication for harness-only changes.

Accepted: full Cut23 client gate33/33 PASS in27.0s on its isolated no-HMR test
server. Existing uncertainty labels, why-not sheets, top/bottom anchored
pickers, outside tap non-mutation, 32px supply drop target, supply purchase
position, persistent paid drop confirmation, death reason and live refusal
gloss checks all execute and pass. Report pending Forge link opens through
its real Details control. No Rust/UI runtime change in this migration.
