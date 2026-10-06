# Securing gold is not a loss — 2026-10-06

Real seed2 phone checkpoint: Rust stake loot6/death_keep0→loot0/death_keep6,
with zero swapped omitted by serde. UI incorrectly labels −$6 swap to the recent
folded scroll pickup. Record checkpoints move remaining carried gold to secured.

Track previous secured gold alongside previous carry on the same watched run.
Subtract the secured increase from a net carry fall before displaying a loss.
Positive authoritative swap-counter deltas remain visible even if the checkpoint
or new pickups mask a carry fall. Preserve genuine residual theft/unknown losses,
older-wire pickup fallback, floor-load stale guards and carried/secured labels.
Do not change Rust accounting, event stream or gold flow.

Acceptance: mounted watch400/1440 checkpoint has no false loss with omitted or
explicit-zero swap counter, including new carry collected in the same batch.
Actual swaps/thefts and loss combined with securing remain correctly labelled.
Headed real WASM D2 checkpoint screenshots and stake wire evidence inspected.
Existing watch/combat/meter checks, build/typecheck/copy lint pass. No deployment.

Verified baseline headed real-WASM seed2 phone: last stake at703loot6/secured0
→747loot0/secured6, no swapped field; UI `−$6 swap → folded scroll?`.
Desktop baseline already collected9 after securing, so had no net fall.
Rust wire.rs omits swapped0; turn.rs checkpoint adds run.loot to run.secured
then zeros run.loot; snapshot death_keep is run.kept(Death). No money was lost.

Watch now offsets the same-run net carry fall by its secured increase; positive
swap deltas remain authoritative. Existing prior legitimate loss can remain for
its2.5s lifetime; it is not replaced with a new false checkpoint loss. TS wire
comment explicitly documents omitted zero counters. No new wire field or Rust.

Final checkpoint-gold9checks/width400/1440 (18total) pass7.7s: omitted/explicit
zero, actual theft, swap+checkpoint, theft+checkpoint, new pickup+checkpoint,
swap masked by unchanged carry, old-wire fallback and no overflow. Initial
combined14gold+16meter+30decision+watch-statuschecks pass18.0s; later two cases
fixture-only, production later changes comments only. Build/typecheck/copy1524/
diff pass; existing bundle-size advisory.

Real after normal batched watch400/1440: D2secured6 with new carry9/11, no false
loss, no errors/overflow. Exact phone branch: real engine stepped one tick per
QA call and pause clicked synchronously at zero-carry checkpoint; viewer745,
Rust740loot0/death_keep6, UI `Carried $0 · Secured $6`, no false loss. One-tick
stepping is a diagnostic harness choice, not a production simulation change or
performance claim. Desktop and exact phone screenshots inspected and shown.
Evidence scratchpad/checkpoint-gold-20261006/ before/after/exact screenshots,
JSON stake batches and logs, plus root client/build logs. Raw events unchanged.
No Rust/WASM/gameplay/deployment changes.
