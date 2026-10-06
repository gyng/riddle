# Named actors in the combat log — 2026-10-06

Fresh real seed2 first-session watch shows generic Hero despite roster Wren Ash.
Watch's actor lookup also excludes allies, so companion attacks become Foe.
Supply snapshot.hero.name from Rust's stable identity using run.heir (not the
current heir) and the game's bloodline. The existing optional Entity.name wire
field covers old clients. Watch captures names per batch for delayed display,
remembering named foes and companions across batches. Never infer a past run's
hero from the current selected slot. Unnamed old snapshots retain Hero fallback.

Acceptance: real snapshots/read/save/load preserve stable identity without
changing saves or RNG; replay snapshot names remain original after heir advance
and bloodline switching. UI event stream test covers named hero/companion/foe,
missing-name fallback, delayed events, retained damage/heal/item styling and
bounded80-row log at400/1440. Headed real first-run screenshots. Fast workspace,
clippy, rebuilt WASM, frontend build/copy and relevant watch checks pass.
No gameplay/balance changes or deployment.

Verified:558 fast workspace tests pass36.77s,1 ignored; clippy all-targets pass.
Rebuilt fastWASM5240576bytes; final frontend build/typecheck/copy1524/diff pass,
existing bundle advisory. Final isolated combat identity20 checks400/1440 and
existing watch-status pass24.4s combined. Read-only snapshot/save/load,
retained run heir, bloodline switch/replay and old optional-name decode tested.
Log captures only actors referenced by logged events; no full remembered-name
copy on movement-only batches.

Headed real-WASM seed2 manual house/send400/1440: Wren Ash attack/pickup names,
paused checkpoint, no page errors/overflow, screenshots shown. Named companions
verified in controlled event stream, not claimed as a real companion playtest.
Snapshot-absent actors retain Foe fallback (seen on phone when the monkey died
inside its first batch); no guessed names. Screenshot log scrolled to named
attack because overlay shows three rows. First harness used nonexistent engine
version property, then uppercase Send lookup; corrected to app.kind/version
and lowercase button text, verified terminal runs. Evidence:
scratchpad/combat-identity-20261006/ plus combat-identity-{tests,clippy,wasm,
build-final,ui-final}.log. No gameplay balance or deployment claim.

Next evidence: desktop first-run roster incorrectly says Ready while watching;
inspect live notifications and core snapshot ownership before fixing. The
first-batch unknown foe and duplicate attack/hurt rows also merit later review.
