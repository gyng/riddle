# Offline transport boundaries — 2026-10-06

The real client divides an absence into 30-minute calls. Those calls currently
grant an extra run completion and reset rest and reveal allowances. The content
audit reproduced this using the same earned camp and exactly 160 hours: one
call ended at heir20, whereas320client calls ended at heir47. Both advanced the
wall clock by160hours. This is a gameplay defect, not forecast uncertainty.

## Contract

1. One absence has one initialization and one finalization. Internal calls
   consume only their supplied ticks; an unfinished run and remaining rest
   survive the boundary. Preserve the existing real-return convenience of
   finishing at most one begun run past the whole absence's budget.
2. Rest carry clamping, watched-rest clearing, passage pricing, renown
   settlement, reveal allowance and reel history operate at the real absence
   boundary. Transport calls cannot provide extra rewards or system reveals.
3. The client receives correctly additive reports or an explicitly versioned
   cumulative report protocol. Do not silently feed cumulative counts to
   mergeReports. Gold, XP, Legacy, worker acts, deaths, nightly marks, learned
   facts and elapsed seconds must describe the whole absence once.
4. A save/load between internal calls resumes the same absence. New fields
   have defaults for existing saves. Do not rely on in_absence alone: that flag
   intentionally also groups separate reports until a send/step/advance.
5. Shared-town wallet order and independent bloodline identity remain
   deterministic. Before the scout, only the manually sent run may finish.
   No automatic construction, simulation approximation or balance bar change.

## Acceptance

Use the same earned input and the same elapsed time for a whole slice,30-minute
slices, uneven slices and saved/reloaded slices. Compare the complete canonical
saved state, not selected depth/gold fields. Separately compare against the
complete-call API and account for any explicitly retained historical sampling
behavior. Record input and WASM hashes and terminal outcomes.

Cover one and three bloodlines, pre-scout home and in-flight states, in-flight
rest, exact run/rest boundaries, zero seconds, day crossings, workers, quests,
system reveals and report sums. Meaningful Rust regressions, fast workspace
checks, rebuilt real WASM and client report/catch-up checks are required before
claiming the fix. Recheck idle-d23 without weakening its bar; this targeted row
does not certify the full balance audit. Profile only equivalent outcomes.

`node tools/offline-slice-check.mjs SAVE --seconds 576000 --out DIR` is the
earned-save diagnostic. A nonzero mismatch exit is a regression failure.
It is not a balance gate or performance test.

## Baseline checkpoint

On the earned seed3 eight-hour save, another8hours reproduces unequal state
even with equal run counts: whole and30-minute calls both report16runs and
gold3837, but renown708/rank2 versus2149/rank4. Uneven calls report18runs,
gold4041 and rank4. All paths move clock_s to59720.30-minute save/reload
between calls matches uninterrupted30-minute calls on this input, which does
not prove future in-flight slice saves. Complete-call API and whole final slice
match exactly. The diagnostic compares all saved fields including batch,
ledger timestamps, history and RNG-bearing run state; differences are not
normalized away. Artifacts: scratchpad/offline-boundaries-20261006-8h/.

The160-hour baseline also fails terminally: whole203runs/heir20/rank4,
30-minute480runs/heir47/rank33, uneven572runs/heir73/rank33. Every path
advances576000seconds. Reloaded30-minute calls retain gameplay totals but
differ from uninterrupted calls in batch.bounty and one row_stats sends count;
the harness also compares these two paths directly. Zero seconds passes all
five paths. Artifacts: scratchpad/offline-boundaries-20261006-160h/ and -zero/.
No gameplay fix is claimed by this baseline checkpoint.

Keep complete/count APIs distinct from transport controls. Native dev bindings
are generated from crates/riddle-wasm/src/lib.rs by tools/native-codegen.mjs;
changing the bridge requires regenerating those bindings as well as WASM.

## Implementation checkpoint

One saved `Game.offline_absence` holds the cumulative tick budget, consumed
ticks, worker-hour cursor and report baselines. Single-hero clock_s commits at
the final boundary; intermediate run settlements use the same start clock and
consumed ticks as a whole call. Rest/run/quest processing continues immediately,
but return-only worker processing, nightly marks, renown and reel selection
wait for the final call. Multi-hero advance retains its shared wall clock and
stable wallet order. Its actual return now finishes at most one outstanding run
per slot, in slot order, bringing those heroes to camp too.

Intermediate reports carry `slice_pending: true` and zero additive counts;
the final report contains the whole absence once. The app ignores pending
acknowledgements. Older cores' ordinary additive reports still merge. Missing
method errors alone permit the older API fallback; runtime errors stop the
batch and cannot replay elapsed time. Zero seconds returns a final report.

Autosave/pagehide preserve the previous durable save and its absence timestamp
through catch-up, including failed batches. Open-app ticks stop after a failed
batch. Successful normal absence completion adopts the new rules and saves
before leaving the return workflow. This avoids treating a transport checkpoint
as a completed absence when the client cannot reconstruct remaining time.
Raw engine checkpoints remain resumable for tools: passage/bounty baselines
survive load, per-run row tallies serialize, and load does not redraw the oath
or recompile a home hero in the middle of the saved absence.

Current-game packages use exact simulation. Budget-dependent stalled-run
extrapolation remains only in direct historical literal harness calls; sliced
literal calls are exact. This explicitly retained legacy-tool difference is
not certified as partition parity. Current-game pen and migrated custom rules
are not literal harness mode. No progression bar or mine.bars change.

Verification: five new Rust regressions cover complete state and report parity
for seeds1/3/5, uneven/reloaded slices, pre-scout sends, rest, final zero seconds,
multiple days/worker/quest/reveal groups and three shared-wallet heroes. Reveal
budgets count a trigger group, not one feature: forge/loadout/exits are one.
Fast workspace verification passes568tests/1ignored and12tool checks; clippy,
typecheck/build/copy1591/diff checks pass. Initial authored reveal assertion
mistook a group for one feature and failed; corrected group assertion passes.

Final WASM5233480bytes, SHA256
`96fabde7c726366d23ee582a882b028a414171b37eb776270ba83578e1b1d8e1`.
Earned seed3+160h: allfive diagnostic paths203runs, heir20, D21, same complete
state SHA256601715d83abccaac4d75358d15b7a7d6baec35cf6025ddcdd25869060344473b
and identical final reports, including320/378calls and reload. This is also
the original whole-call state hash. Earlier candidate+8h matched allfive paths
exactly with16runs. Final earned three-bloodline+8h allfive paths42runs and
identical complete saves/reports; final empty-town zero paths also identical.
Artifacts scratchpad/offline-boundaries-final-20261006-{160h,multi,zero}/.

Fresh targeted idle-d23 seeds1/2/3 PASS11s, all byday12, median6.33d. This is
not a full balance gate. No timing improvement claimed. Actual headed WASM
client320slice return1440/400 shows203runs/heir20/D21 with no errors/overflow;
screens inspected/shown, artifacts scratchpad/offline-boundaries-20261006/.
Report UI itself unchanged. Existing132UI checks pass; picker pending-state
test now releases its mock explicitly rather than racing a400ms timer under
parallel CPU load. Existing Cut30.5 report check now respects the owner's
simpler layout: first-worker announcement in the lead, growth before recurring
workers inside Details. Lead word-budget/content checks retained. Final client
catch-up checks additionally cover zero, durable-save guards, failed runtime
calls and legacy API fallback. Final8client jobs PASS50.5s (all132UI plus
catch-up/merge9checks and scout/report). Task preview5435 process group2033261
cleaned, shared5219 HTTP200. Full client/balance suites remain uncertified.
No deployment.
