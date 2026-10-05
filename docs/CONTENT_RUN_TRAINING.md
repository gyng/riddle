# Run training survives watched exits — 2026-10-06

Rust on_run_end already emits style/tactic level gains and automatic boss drills.
Away reports carry these in packages, hidden inside Details; watched reports omit
them entirely. Capture bounded per-run package beats on ExitLine after settlement,
including saved death line and batch exit. Preserve existing batch progression,
RNG/rules/ticks and five-beat cap. Old wire defaults empty. Watched ReturnReport
copies this exact core field, never calculates levels in TS.

Shared compact Training section shows only level/drill gains on report before
Details and death screen after cause/action; drill copy says <boss> tactic, with
original core source retained. No unrelated town/quest announcements under
Training; remaining package beats stay in Details. No duplicates in Details,
including matching grew rows. Empty/old reports stay compact. Same historical
report can be reopened; a later run announces only its own new gains.

Gates: per-run last_exit/batch/death copies identical, serialization/old wire,
second run no repeated drill/level message, sim no progression feedback. Fast
Rust suite, UI320/400/1440 old/empty/duplicates/stall/death/report, real rebuilt
WASM observed watched exit level→report and away/death captures, build/copy/diff.
No tuning, gameplay progression thresholds, controls or deployment changes.

Implemented: additive ExitLine.packages contains Rust beats(on_run_end lines),
not town/quest messages. last_exit/batch/death line get identical values.
Watched report copies field; shared trainingBlock displays level/drill messages
and readable Warlord tactic label, remaining beats retain original Details.
grewBlock highlighted facts suppress duplicate growth lines and plaques.

Validation: fast552 tests36.23s/1 ignored; new bank/death two-run test checks
level+drill payloads, identical copies, no repeat, serialization/old empty wire.
Initial compatibility hash failed due new serialized exit metadata; existing
normalizer now strips only packages, alongside prior reason/meters fields.
Original fixture c70ffd182d4080f1 unchanged and full suite passes: original
events/ticks/wallet/depth/death counts still hash identically. Sim excludes
feedback at existing !self.sim progression block; no new sim test.
UI54 checks320/400/1440 plus report upgrade57/worker60/death-actions44/
death-hero27 pass13.6s batch. Clippy/build/typecheck/copy1507/diff pass; existing
large-chunk advisory. Rebuilt real WASM5,245,585 bytes.

Headed400/1440 freshseed2 manualhouse/nine actual sends/real tenth watch with
Fast+Skip -> report STEADY L2; existingseed15 camp24h away -> STEADY L3 visible,
no warnings/overflow. Initial8h absence did not cross level40, correctly no
training field; actual sample extended24h. Screenshot harness corrected absent
jump id to Skip, then excluded inert closing sheet copies; no product change.
Screens shown; drill/death feedback has Rust/UI fixture coverage, not a new
real watched drill/death walk. Artifacts scratchpad/run-training-20261006 and
run-training-{rust-final,wasm,clippy-final}.log. No rules/tuning/full balance
audit/deployment. Automatic new drills remain revocable through existing Tactics.
