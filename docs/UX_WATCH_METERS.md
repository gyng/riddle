# Complete the watch meter repaint — 2026-10-06

Meters use Rust snapshots at their final tick; logs use individual event ticks.
A paused picture between those ticks can legitimately show older meter totals.
Do not derive game truth in TS or show frontier totals in the paused picture.

The400ms DOM throttle drops its final paint if no later snapshot is released.
Schedule one trailing paint of the latest already-reached snapshot; coalesce
updates, preserve400ms cadence, clear pending work on forced repaint/disposal.
No polling, engine stepping or future snapshot reads in the timer.

Acceptance: mounted watch at400/1440 refreshes the latest reached meter after
rapid updates and pause, keeps queued future totals hidden, updates after resume,
and stops after disposal. Existing watch/roster checks, build/typecheck/copy lint
pass. Headed real-WASM screenshots shown. No Rust/WASM/gameplay/deployment.

Verified: original implementation fails the mounted fixture at the paused12hp
update (5s timeout), confirming a lost trailing repaint rather than relying on
the earlier screenshot. New single trailing timeout uses lastMeters only,
coalesces updates and clears on forced repaint/disposal. No extra engine calls.

Final mounted watch-meters test8checks/width400/1440 (16total) passes7.2s:
reached6→rapid12/18→pause retains18 while core999 queued; resume shows999;
queue888 and dispose before its pending paint, detached panel stays unchanged.
Final fixture uses monotonic snapshot ticks. Earlier fixture parked future state
at1000 then resumed at3; corrected to consecutive ticks without changing code.
Combined initial meter12checks plus watch-status and roster38pass15.5s;
subsequent edits only extend the fixture. Build/typecheck/copy1524/diff pass;
existing bundle-size advisory. Initial file creation used the web cwd with root
paths and failed harmlessly; files then written from repo root.

Headed real-WASM seed2 manual house/Send400/1440 checkpoint has no page errors
or overflow. Desktop inspected6hp panel and snapshot evidence: first attackt590,
paused viewer742/latest reached snapshot724; panel remains from reached data.
Different screenshot/metadata rates0.09/0.08 reflect a pending reached repaint
between observations, not changed core truth. Phone screenshot keeps compact
meters off by default; mobile refresh is verified in the controlled mounted
fixture with the existing meter preference enabled. Evidence:
scratchpad/watch-meters-20261006/ and root baseline/client/build logs.
No real-run claim of exact per-event meter updates: snapshots remain batched.
No Rust/WASM/gameplay/deployment or simulation-speed claims.
