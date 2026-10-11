# Cut 125: isolate the stacked-foe renderer fixture

2026-10-11. Cut124's complete client attempt failed `fights` after183.8s:
`their names sit on two rows (goblin y264)`. The corrected adjacent-name fixture
passed; this is the earlier map-stack fixture. The run was stopped after the
confirmed failure, exit143, and preserved as incomplete acceptance. Only one
of the two injected names was sampled; the exact failed-frame context is
unavailable. No application nameplate bug is established.

The map-stack injection still inherits a running watch's actor queue, visible
floor, frame transitions and quiet policy. Give this geometry test its own
actual renderer scene, just as the adjacent-name test has.

1. Open visible16×16 floor, hero and exactly two foes sharing tile6,6,
   paused map frame, quiet=false. Assert both foe sprites actually render,
   all actors are visible/in bounds, and no unrelated foes exist.
2. Preserve all three original gates: two foes fan by at least2 CSS texels with
   stack≥3; both names occupy distinct rows at least10px apart; each foe is
   at least24 CSS px tall. No missing-name exception or relaxed threshold.
3. Preserve other live-watch checks. No application or simulation changes.
4. Controlled fixture, then whole `fights`, then complete156-group acceptance.
   Preserve original failure evidence. No blind cohort until terminal acceptance.

The application and immutable preview remainb6df4f2. This is test setup repair;
it neither proves the earlier cause nor certifies the whole suite.

Verification: controlled400px renderer at400/1440 browser viewports passes;
not a desktop scaling test. Phone scale100 matches `ui/viewer.ts`. Whole
`fights` PASS51 checks,189.8s, terminal exit0. No numeric assertions changed.
Logs archived under `scratchpad/blind-b6df4f2-frozen/acceptance/`.
Full156-group acceptance still required.
