# Cut 62 — current controls in watch and forecast QA

Isolated baseline clarity:watch and cut22 both reproduce failures. Watch's
retired glyph lookup never dispatches Skip and does not assert it found a
button; forecast expects removed automatic refinement despite explicit More
samples and approximate-first behavior. Preserve original timing/frontier,
ending, arithmetic, uncertainty and stale-state requirements. Use existing
visible-control helper, assert dispatch, explicitly request larger samples.
Update retired chrome wording only where supported by current UI. No hidden
clicks, time-limit relaxation, automatic refinement or gameplay gate changes.
Run repaired jobs and current forecast/watch regression tests. Diagnose any
remaining genuine defects before claiming a pass. No fun score or full client
suite inferred from scoped success; no deployment.

## Results

- Baseline isolated 0/2,49.1s: clarity watch reproduces both missing-Skip
  failures; cut22 times out waiting for automatic refinement.
- Use existing pressWatchControl for actual Speed → Skip; assert all six
  pause/resume/Skip dispatches through visible enabled controls. Original
  frontier and ending speed/timing limits unchanged. Watch now17checksPASS.
- Forecast fixtures intercept forecastEstimate/forecastVsEstimate, explicitly
  click More samples, observe rough-before-refined and retain the same baseline.
  Controlled paired uncertainty, marked/unmarked bars, inline replies and zero
  redundant calls remain asserted; intercept both comparison paths for the
  inline check. Update displayed bank→full haul, and simple Send label with
  remembered mode metadata (controls live in Watch Speed). All31checksPASS.
- First repair exposes nine stale method/copy expectations; later fixture edit
  briefly introduces an undefined variable, corrected before final verification.
  Logs retained, no failing result rewritten.
- Final single-job quartet 4/4PASS73.9s: clarity:watch17, cut22 31,
  forecast-requests16 per400/1440, watch-console labels/touch/geometry/speed/
  pause/Town. Syntax and diff checks pass. Only test sources changed.
- Real shipping WASM, earned pre-Gunner camp, fresh local5427: mobile400 and
  desktop1440 forecasts remain eight-sample estimates after1.2s; More samples
  produces requested33-sample refined result. Raw saves unchanged, no horizontal
  overflow or page errors. Screenshots shown. An immediate refined capture
  shows unfilled bars during existing0.9s animation; settled recapture confirms
  normal contrast. No visual/gameplay fix inferred from that transient frame.
  Owned preview stopped; shared5219 retained.

Raw /tmp/riddle-cut62-*.log and scratchpad/qa-cut62; scratch not staged.
No Rust/runtime/scoring/protocol changes, no deployment. Latest full-client
baseline remains98/117; scoped passes do not retroactively change it.
clarity:core's report assertions still need current-copy triage. Other known
failures (autodismiss, old report/editor/automation routes and resource cases)
remain open. Goal95 remains active and unverified; independent immutable QA
and rating cohort have not been completed.
