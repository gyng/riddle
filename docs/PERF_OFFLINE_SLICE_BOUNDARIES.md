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
earned-save diagnostic. Its nonzero mismatch exit is intentional while this
contract remains unimplemented. It is not a balance gate or performance test.

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
