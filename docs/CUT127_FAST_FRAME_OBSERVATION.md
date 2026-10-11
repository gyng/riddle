# Cut 127: observe fast travel and combat on browser frames

2026-10-11. Cut126's full attempt failed `fights` after280.9s:
`fast watches a fight at4× (speed NaN, frame undefined)`. It was stopped after
the confirmed failure, exit143, with the log/process tree preserved. This is
incomplete acceptance. The original failed frame sequence is unavailable.

Three passive frame-observation probes on the same real renderer/fake seed157
show actual fast-map travel≥32× followed by a fight frame at exactly4×. They
pass without changing playback. These probes do not establish the original
failure's precise cause. The original test polls via Node and separately waits
for travel and combat; that leaves sampling gaps between IPC calls.

1. Observe both consecutive phases within one passive browser-frame loop.
   Record the actual DOM state: travel speed≥32 and then fight frame speed=4.
   Preserve8s travel and30s subsequent fight budgets. Reject absent/late states;
   never manufacture a fight or accept a run that ended without it.
2. Keep the existing two assertions exactly, then the existing public skip,
   ending-clock, frame, bank, boss, collision and other fight checks unchanged.
   No game calls, pause, speed edits or playback changes in the observer.
3. Targeted observation and whole `fights`, then current complete156-group
   suite. Preserve failed/probe logs. No source, sim or numerical gate changes.
   Application/preview remainb6df4f2; no raters before terminal acceptance.

Verification: three passive renderer probes pass; focused whole `fights`
PASS51 checks181.7s, terminal0. Original assertions/budgets unchanged.
Logs archived under `scratchpad/blind-b6df4f2-frozen/acceptance/`.
Exact earlier cause remains unproven; whole156-group acceptance is required.
