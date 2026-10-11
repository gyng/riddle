# Cut 124: give the two-name collision check its intended scene

2026-10-11. Candidateb6df4f2's full client attempt failed `fights`' two-name
collision assertion209.0s into the run: only Captain Tain was sampled. The run
was intentionally stopped after the confirmed failure, exit143; it is **not a
completed full acceptance run**. Its original log and owned-process record are
preserved under `scratchpad/blind-b6df4f2-frozen/acceptance/`.

Six isolated probes and one full instrumented `fights` run pass unchanged.
The full diagnostic passes48 assertions and records both fixture foes visible,
quiet=false, plus two ordinary jackals despite the fixture's attempted clear.
The original failure did not record its complete context: **its exact cause is
unproven**. Do not call a game nameplate bug fixed. The existing injected scene
inherits a live floor, visibility, pending replay events, HUD reservations and
quiet beats (which intentionally show only one hostile's name).

## Contract

1. This renderer collision test receives a controlled open floor, exactly two
   visible hostiles on adjacent tiles, in-bounds positions, and quiet=false.
   Assert the setup: both sprites are actually rendered, no unrelated hostile
   is rendered, and the current renderer frame is fight.
2. Keep the original assertions: both12-letter names must be present; their
   horizontal spans overlap; their rows are separated by at least the plate's
   height; **no two drawn tags intersect**. Do not relax any number or permit a
   missing name.
3. Separately check quiet=true keeps the existing one-hostile-name behavior;
   releasing it restores both. This makes the distinct culling policy explicit.
4. Run the controlled 400px renderer scene inside400px and1440px browser
   viewports (this is not a desktop renderer scaling check), then the whole existing `fights`
   test, then the complete current client suite on settled test/source inputs.
   Retain original failed and diagnostic logs. The other live-watch assertions
   stay on the actual watch.

Only test setup and evidence change. The application remainsb6df4f2; its
shipping-WASM freeze and manifest are preserved. No source, sim, art, gameplay,
or numeric gate changes. No blind raters are released until complete current
mechanical acceptance. Native limitations remain explicit.

## Verification

Controlled fixture PASS in both browser viewports; same400px renderer scene.
Whole `fights` PASS50 checks, exit0,214.5s on its own fresh test server.
Original failure and unchanged48-check diagnostic logs retained. This verifies
the corrected test domain; it does not establish the earlier failure cause.
Full156-group acceptance remains required.
