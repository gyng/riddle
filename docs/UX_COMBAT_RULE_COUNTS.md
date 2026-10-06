# Repeated decisions in the map log — 2026-10-06

Coalesce consecutive identical displayed policy decisions into one row with ×N.
Require same floor, row index, text, verb and argument. Any displayed non-rule,
changed rule or hidden trait/chore rule ends the group. Movement alone need not
add log rows. Do not group damage, misses, healing, warnings, kills or pickups.
Keep the first event tick and latest event tick on grouped rows. Groups can span
engine batches, but release/count only when the replay reaches each event.
Raw Rust events, calls and meters remain unchanged. Plain unboxed overlay stays.

Acceptance: mounted watch400/1440 proves paused counts, grouping across batches,
changed row/text/verb/argument retention, combat/hazard/chore/floor boundaries,
and separate repeated pickups. Existing combat identity, meters and watch state
checks pass; build/typecheck/copy lint pass. Headed real-WASM screenshot shows a
repeated decision count alongside ordinary hit/kill rows. No deployment.

Verified: renderWatch groups only matching replay-released policy rows by
floor/index/text/verb/argument; resets for any other logged event including
hidden negative-row rules. Counts update the last existing li and retain
first/latest tick attributes. No raw event mutation, grouping of combat/items,
new box, wire or Rust change.

Mounted combat-rules15checks/width400/1440 (30total) pass16.3s. Covers initial
paused release and an already-visible ×2 count staying ×2 while its third event
queues paused, then ×3 after resume across the batch boundary. Distinct row,
argument, text and verb, attack/fire/chore/floor boundaries and duplicate gold
pickups preserved. Initial combined28group+48identity+16meter+watch-status
checks pass30.7s; final extra paused-count check changes only the fixture.
Build/typecheck/copy1524/diff pass; existing bundle-size advisory.

Headed real-WASM seed2 house/Send400/1440: both have
`D1 · attack monkey ×3`, first560/latest580; ordinary attack/kill rows remain,
no page errors/overflow. Instrumented event batches unchanged. Screenshots
inspected and shown, overlay scrolled to the grouped decision/first-hit rows.
Desktop picture advancedD2 while log shows its explicitD1 historical labels;
not a simultaneous D1 combat frame or a whole-run gameplay/performance claim.
Evidence scratchpad/combat-rules-20261006/, root client/build logs. No deployment.
