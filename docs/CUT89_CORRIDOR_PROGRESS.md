# Cut89 — corridor fighting must finish fights

Earned seed27 D19 setup, Guarded3/Bossfocus2/Corridor1: a4h absence returned
nine runs, deepestD6, eight stuck returns. Read-only native comparison on the
identical earned save reproduces9runs/D6/$360 home; replacing only slot1 with
kite archers produces3runs/D22/$5655 home. Diagnostic, not a balance gate.

Corridor fighting holds indefinitely against nonadjacent visible enemies and
can retreat again immediately after stepping out. Apply the existing pack-break
progress safeguards: actionable threats only, nearby retreat, no repeat retreat
from one tile, approach commitment, and bounded holds against closing melee.
Retain corridor preference and adjacent attacks. Ranged foes must be approached;
ignored or unreachable foes must release the tactic to other rules/chores.

Numeric gates: frozen ranged corridor foe must see approach/attack within120ticks;
far pack must not cause retreat. Existing pack-break/corridor guard tests pass.
The same earned4h snapshot must exceedD6 and produce fewer than8 stuck returns;
compare exact deterministic reruns, saved baseline untouched. Run full core fast
checks, rebuild real WASM, and client watch/tactics checks before freezing the
new evaluation candidate. Keep raw old-build evidence. No95/deployment claim.

Verification checkpoint: before regression fails at stationary archer; after9
seam tests pass. Exact earned4h native before9runs/D6/$360/8stuck; after3runs/
D19/$2002/0stuck (two full hauls, one death). Native timing0.269→0.208s is one
diagnostic sample, not a performance benchmark. The old placement regression
expected bottom7 because every position stalled; actual new stall0 chooses
earliest safe5. Retain mock selector, cached-placement equality and≤0.15 gate,
add exact0stall and5 placement assertions. Full fast suite must rerun.

Fast verification green101s:684 core tests PASS/one existing ignored,14tool
tests plus TS/copy; all-target fast Clippy PASS. Shipping WASM build PASS.
Bounded current-game gate suite now running; no full balance certification yet.
