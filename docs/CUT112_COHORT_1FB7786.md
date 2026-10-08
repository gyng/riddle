# Cut 112 — cohort 1fb7786's lapses, and take control

*2026-10-08. Blind cohort 1fb7786: 65.1 · 69.7, α .835 (reliable), mean 67.4 (from 64.7, 56.7).*

## Fixes
- **The apprentice's standing order** (`StandingOrders.forge`: `half` default · `all` · `off`): under `half`
  he forges with half of each haul home since his hire, never the purse the player left — the absence
  no longer "spends my gold without asking". Run setup shows the order once he is hired.
- **The pen at the Mother**: its group opens outside the one-system-a-report budget (`systems::pen_due`);
  the lock reads what it waits for (`after Bloat Mother` · `next report`); stale tips retire.
- **Counters you can take**: Mirror rhythm arrives at the King met; `try:` names only what can be taken
  now (`packages::counter_offer`: the pen row · the package · `restore drill` · `wait for drill`).
- **REPORT never hangs**: every close of the keep sheet keeps the picks and moves on.
- **Carried includes Secured**; the speed eases (≤ ×2 every 150 ms above 16×); the pen's last row shows
  `▼ packages below`; the combat style's toggle reads `close` while open.
- Recorded deviation (Cut 110 §4) narrowed: `automation-pays` 28.25 vs 28.36.

## Take control (owner 2026-10-08: "a take control option as a secondary game mode")
In the watched run the panel's toggle gives the player the hero: the core waits at each of his turns
(`Run.manual`, `Snapshot.awaiting`) for one action — a step (a foe on the tile is attacked), a verb as
rows write it (`attack nearest` · `drink heal` · `throw fire` · `descend` · `return`), or a wait — and
`Release` hands him back to the rules. Arrows / WASD / numpad and `.` drive it. Inputs are recorded on
the run's capsule (`Input::Control`, `Input::Act`) and replayed at their tick; a sim (forecast, verdict
replay) or an absence always plays the rules (`Game::tick_inner`). Player actions are row −3: no row's
tally, streak, oath or stall guard reads them. The watch steps the engine only on an order (no fold,
jump, travel or world clock) and holds the picture at the frontier. Gates:
`tests_cut30_pkg::take_control_waits_for_the_player_and_releases`, `web/tests/take-control.mjs`.
