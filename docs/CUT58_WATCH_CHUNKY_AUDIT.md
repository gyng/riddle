# Cut58 — audit remaining chunky UI material gaps

Reinspect current earned management windows and uncommon live-watch controls.
Use existing panel/tablet art and explicit shared material components. Preserve
map hits, gems/studs, item silhouettes, chart bars and text combat overlay.

Acceptance: inspect eleven earned windows at400/1440, exact full-save equality,
no horizontal overflow or page errors. Watch mode/Skip/Meters/Bail controls at
320/400/1440 share framed44px commands, readable consistent labels and keyboard
focus. Selecting a mode closes its own window, preserves the live run and saved
mode; Town still exits watch without cancelling it. Capture screenshots here.
No deployment, Rust/balance changes or independent fun-score claim.

Findings: eleven management windows×400/1440 retain existing shared materials,
no page overflow/errors and full engine-save equality. Live-watch mode and run
controls sit outside `.cmd`: all six had no border-image,6px rounded faces and
17/18px labels. Explicit game-control fixes material ownership; openWindow
centers the parchment, with responsive three-column control groups,24px icons,
14px common label scale and selected gilt edge.

Real-WASM earned-save320/400/1440 captures and interactions pass: six controls,
80px heights, minimum width78.65px at320, framed square material, within viewport,
Fast closes its window and sets saved mode; Town returns to a still-live run,
no page errors. scratchpad/chunky-cut58/watch-real-settled; screenshots shown.
Earlier diagnostic assertion read lineage immediately after Town (before async
refresh), then early send/paused initialization; retained raw failures. Final
capture waits for actual initialized watch frame and refreshed live lineage.
Build/TS and copy1740zero/diff pass. Related qa9/screens/chunky-controls attempt stopped after machine file-descriptor
exhaustion prevented a shell spawn; no completed test result. Exact owned runner
1118639 and temporary server1118646 stopped; shared5219 retained. Short
chunky-controls follow-up1/1PASS12.5s; actual320/400/1440 history/fold/
appearance/rename reload/read-only checks. Broader attempt remains incomplete.
Cut57 resource/forecast triage paused for explicit owner UI request; isolated
fights replay passed48, earlier target crash remains unexplained. No deployment.
