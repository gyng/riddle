# Iteration work in impact order

Owner instruction: implement the options in order of impact. Preserve production
game behavior, complete gate samples, seed ordering in reductions and exact saves.

1. Native development engine behind the existing asynchronous JSON client API.
   Generate its bridge from the shipping Rust bridge; unsupported signatures fail
   the build. Dev selection is explicit, errors never silently select the fake.
   Each lane has an isolated game, calls remain ordered, native reads retain the
   existing worker-width controls. Compare outputs/save bytes with shipping WASM
   and measure representative calls; target2× faster expensive reads. Production
   bundles contain neither the dev transport nor native server.
2. Rust-owned balance data with compile-free native tuning. Defaults must reproduce
   shipping exactly. Profiles are immutable within a process; a profile change
   restarts/reloads native sessions at call boundaries. Hash the exact profile
   into diagnostic and simulation identities; reject unknown/invalid values.
   Browser/server and shipping use the same parsing/defaults. No TS game logic.
3. Previously failing seeds first in targeted fail-fast runs. Keep all requested
   seeds and unchanged verdict bounds. Persist hints separately from results;
   scheduling hints cannot become acceptance evidence. Demonstrate an unchanged
   passing reduction and a quicker rejection on a known failing case.
4. Same-runtime check-in checkpoints for long cases. Include complete bot state,
   game state, accumulated output and parameters; bind the exact runtime/harness.
   Atomic writes, corrupt/mismatched state ignored, full outputs identical to an
   uninterrupted case. Never reuse a checkpoint across changed runtime behavior.

Measure local build and runtime costs separately. Saved-camp checks are fast
feedback; final acceptance remains the full gate. No claim that fresh full
simulation CPU becomes five minutes without corresponding measured evidence.

Measured checkpoint: three quiet final adapter rounds, eight native forecast
threads; all27 replies/saves per round match shipping byte-for-byte. Package
reads0.154/0.682/0.743s vs0.998/5.101/5.777s (6.5/7.5/7.8×). Late wall
search1.723s vs7.040s (4.1×); basic forecasts5.1–11.0×. Direct8h catch-up
1.2–3.2×, workload dependent; not the app's sliced scheduler. Native timing
includes IPC; WASM measures direct calls. Warm native build0.19s, no claim
that a cold Rust compilation is cheaper than fast WASM. Default native builds
reuse the ordinary core artifact; profile builds/executables are isolated.

Live Rust profile-edit integration passes without recompiling; state/compiled
rows update, invalid edits reject and the valid process can recover. Known
failure rejection3.81→1.26s, seed4 first, unchanged16-seed requirements.
Passing numeric/priority default reductions both16/16. Four bot resume tests
compare exact final outputs and raw saves (76.74s); a real interrupted short
case has three complete records exactly equal to uninterrupted ones. Its
fortnight bars are deliberately inapplicable at2days, so this is integrity
proof, not acceptance. Full unchanged acceptance remains required.

An initial full test caught the legacy native event-stream fingerprint when
word width was normalized globally. The correction is adapter-specific: native
app processes explicitly select shipping32-bit words, ordinary native gates
retain their64-bit reference, and the existing fixture remains untouched.
The legacy fixture and final adapter parity both pass.

Evidence: scratchpad/native-dev-20261004. New public/runtime acceptance and
publication are recorded in HANDOFF only after completing verification.

Owner refinement: docs/UX_SIMPLE.md now authorizes bounded routine regression
in place of exhaustive-by-default checks. Manual construction changes game
behavior intentionally; the original exact-output audit was stopped at roughly205
completed matching records. It is not claimed complete. Current routine full
(18 current-player cases, current-game metrics, ten wire seeds) passes76s warm;
524 engine tests plus UI31 checks pass. Broad balance statistics are an opt-in
audit, not certified by routine success. Fresh routine table passes in247.5s
(4m8s); current metrics99.6s and wire QA43.2s run alongside. Final native/WASM
parity matches27 complete replies/saves; package calls6.6–7.5× faster, late
wall4.2× and forecasts5.1–11.1×. Final13 harness tests and all-feature clippy
pass. Publication validation is recorded in HANDOFF.
