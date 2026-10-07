# Cut 63 — preserve manual sends before the Scout

Reproduced autodismiss:death baseline: 2/11 failures are stale Camp→Town copy.
Inspection also finds a real conflict with the manual opening: default-on death
continuation presses Send after12s even when Rust tree.auto_send is false.
The existing fake test expects that behavior, so it does not enforce the newer
manual-send contract.

Change: before the Scout, death timer returns to Town; Send remains manual.
After the Scout, preserve existing safe continuation; never apply a patch,
spend, equip or build automatically. Historical report/chronicle deaths return
to their source. Preserve12s/20s clocks, input/hidden/tab/settings semantics.
Use Rust-owned tree.auto_send; retain compatibility for older wires without tree.

Verify manual/hired routing and exact no-send/no-spend before Scout, existing
full auto-continue gates, death-actions/wall-preparation and manual-town checks.
Real WASM earned pre/postScout saves plus explicit diagnostic death,400/1440,
no automatic engine.send beforeScout, unchanged rawsave/manualcount and visible
ring destination. Show screenshots. Build/TS/copy/diff; no deployment.

## Validation so far

Only runtime change is the death timer target; no core/simulation mutation or
clock duration change. Rust tree.auto_send false (including paused Scout) now
selects Town; older wires without tree retain compatibility.

Initial fake manual fixture failed: its demo/migrated heir already had a Scout.
Explicitly remove Scout for manual case, then add him for automatic case;
fixtures assert both flags before interpreting routing. Full autodismiss now
29/29PASS. Panel fixture uses earned-depth diagnostic state and must click a
visible forecast: no hidden first-load click or skipped panel check. Original
clock/input/hidden/settings/no-spend assertions retained.

Scoped quartet4/4PASS100.8s: autodismiss29,death-actions22×400/1440,
wall-preparation39×320/400/1440,first-plot×320/400/1440. Build/TypeScript,
copy1736zero,syntax/diff pass; existing web chunk warning remains.

Real shipping WASM on fresh5428: preScout save earned through public manual
house construction and explicit send +1h catch-up; comparison uses existing
earned Scout camp. Render explicit diagnostic deaths, not claimed freshly earned
deaths. At400/1440 manual timer targetsTown,0sendcalls,rawsaveunchanged;
hired timer targetsSend,1sendcall. Final reduced-motion captures also assert
exact killer+depth, no horizontaloverflow/pageerrors. First screenshots caught
intro motion; final stable screenshots shown. One added text assertion initially
omitted displayed D4 suffix, corrected diagnostic (no UI copy change).
Owned5428 stopped,shared5219 retained. Raw /tmp/riddle-cut63-* and
scratchpad/death-cut63; artifacts not staged.

Fresh full client audit started with TEST_JOBS=3,TEST_HEAVY_WIDTH=1, exact exec
session53404, /tmp/riddle-cut63-full-client.log. Still pending; must poll this
handle, not restart on buffered output. No full-client or fun-score claim yet.
