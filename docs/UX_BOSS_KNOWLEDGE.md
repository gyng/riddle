# Victory tooltips retain the owning bloodline's knowledge — 2026-10-06

Multihero reports currently expose first victories but only selected lineage
knowledge. Include a bounded per-victory snapshot in each BloodlineReturn:
exact learned enemy facts and existing ledger/wall rows from that slot. Read
only; no selection, new simulation, or persistent save change. Old reports
retain the existing unavailable fallback. Merge snapshots by boss within each
owner across slices, preserving earlier victories when later slices are empty.

Acceptance: core own-data/immutability/old-wire checks; actual multihero WASM
unselected first victory shows own tooltip without selection/save changes;
client ownership/slice merge/fallback tests and phone/desktop screenshots.
Fast workspace tests/clippy, rebuilt WASM, client build/copy. No balance/deploy.

Verified: 561 workspace tests passed (one ignored), fast clippy clean; rebuilt
WASM 5,250,760 bytes. Enemy input/ownership checks pass at320/400/1440;
54 report victory checks pass. Actual earned two-bloodline8h WASM replay
matches the previous report after removing the new snapshots, and its final
raw save is byte-identical. Snapshot facts/ledger/wall match the owning slot;
hover leaves selection2 and the save unchanged. Phone400/desktop1440 captured
in scratchpad/boss-knowledge-20261006/. Custom detail hosts take priority over
nested glossary labels, so Bloodline text cannot replace the boss tooltip.
Shared tooltip regression70/70 passes. Client build/typecheck and copy1554 pass; existing bundle advisory remains.
