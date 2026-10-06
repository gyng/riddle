# Hero presence — 2026-10-06

Active bloodlines display D13 · In combat / Exploring / Heading home from
engine state, plus existing Resting/Ready states. Add compact activity colour
and HP/run detail on hover, keeping roster spacing and click/details controls.
Selected watch status must refresh from received engine snapshots without
extra lineage RPCs, even runs=0; all other slots stay engine-owned. Do not
confuse live-core presence with paused/earlier watched footage. Ignore stale
snapshot overrides on selection, newer turns, run change or heir inheritance.
Older wire data falls back to Delving rather than guessing combat. No mutation
of game rules, RNG, saves or balances; Rust adds presentation-only LiveRun data.

Core tests: home/combat/exploring/return priority, expiry and serialization
compatibility. UI: two bloodlines, phase labels/colours, live snapshot freshness,
paused picture, newer core reads, run/heir mismatch, exit, mobile/desktop bounds.
Actual earned WASM screenshots, exact raw reports and saves before/after,
build/copy/typecheck and required Rust/wasm checks.
Tactic activation icons remain a subsequent part of this owner-requested work.

Verification: fast workspace Rust tests563 pass/1ignored, clippy clean. Quick
verification's native-host test needed a socket-authorized rerun (2/2 pass);
original quick invocation did not exit green. Build includes real rebuilt WASM
5,252,092 bytes, SHA2563bccd626c19a9c01c76a528e97414e73865ec2b1c159260fd3555191f46fc2b1.
Full six UI sections132 checks plus presence/appearance/status/compact10jobs
pass41.5s; final Ready label presence16checks at320/400/1440 plus appearance
18checks each pass5.1s. Typecheck/build/copy1587/diff clean; existing bundle
size advisory. Four8h before/after catch-ups (three earned early camps and one
three-bloodline camp) have exact raw reports AND save equality. This is parity
verification, not a performance measurement; one timed pair with concurrent
work cannot establish speed. Full client/balance suites not certified.

Headed real-WASM screenshots: combat1440 D4 In combat23/42hp/run66; single
and multi400/1440 camp/watch with no pageerrors/overflow. Latest multi shows
D7 Exploring plus two Ready slots. Presence describes the engine frontier;
paused footage can show an earlier floor/HP. Raw captures/parity under
scratchpad/hero-presence-20261006/. No deployment.
