# One row per unchanged hit — 2026-10-06

The three-line map overlay currently spends two lines on the same unchanged hit:
attack names the participants and amount, then hurt repeats that amount. Retain
the attack row and omit only a later hurt with identical tick, destination,
positive damage and source cause. Match known source kind (hero uses `hero`)
and consume matches one-to-one within a batch. Preserve misses, zero attacks,
unknown sources, different amounts, hazards, extra hurts and different ticks.
Rust counters/shields can alter damage: an attack alone cannot prove its amount
was the actual loss. Both entries remain when their amounts differ.

Read spawn identities before capturing delayed log names, so an actor spawned
and slain inside the same batch can be named using its actual spawn wire data.
Keep raw events intact for renderer, audio, meters and gameplay. No wire or Rust
changes. Existing plain overlay, item/damage colours and80-row bound retained.

Acceptance: mounted watch at400/1440 proves deduplication, one-to-one matching,
hazard/modified/unknown/miss retention, same-batch spawn names and paused name
capture. Existing identity/roster/watch checks pass. Headed real-WASM first-send
screenshots inspected. Build/typecheck/copy lint pass. No deployment.

Verified: mounted combat-identity fixture expanded from10 to24 checks per width
(48 total), retaining existing identity/paused-capture/colour/fallback/bounds
checks. Adds same-tick hazards, changed damage, unknown sources, misses, zero,
different ticks/destinations/causes, preceding hurts, repeated identical hits,
extra hurt preservation and spawned-and-slain companion names. Final isolated
48checks pass29.8s. Initial combined combat44/watch-status/live-roster38 run
passes30.2s before two additional ordering/multiplicity checks were added;
only the fixture changed thereafter. Build/typecheck/copy1524/diff pass;
existing bundle-size advisory. No Rust or WASM rebuild needed.

Headed real-WASM seed2 manual house/Send at400/1440: each checkpoint captured
one raw hurt and zero displayed hurt rows, with its matching attack retained.
Instrumented event batches remain byte-for-byte unchanged through filtering.
No page errors or horizontal overflow; paused screenshots inspected and shown.
These are first-hit checks, not a whole-run statistical or performance claim.
Spawned-and-slain naming is proved in the controlled stream. Mobile first-hit
actor still falls back to Foe when absent from both snapshot and spawn events;
no name is inferred from a rule's target. Desktop captured the monkey identity.
Evidence scratchpad/combat-log-rows-20261006/, plus root client/build logs.
No gameplay or deployment changes.
