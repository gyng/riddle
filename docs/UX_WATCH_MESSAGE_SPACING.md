# Watch message spacing — 2026-10-06

The death-cause caption sits at12px while the combat log occupies the bottom
54px of the map, so phone text overlaps. Group the bottom ticker, detail tip,
end-cause and plain combat log in a layout that accounts for wrapped text.
Keep the map overlay plain and transparent, existing damage/item colours,
three visible history lines and80 retained entries. Keep ticker/cage input and
history scrolling; empty/inactive messages must not reserve blank rows.
No simulation/state or animation-loop layout measurements.

Verify320/400/1440, narrow wrapped names, simultaneous messages, hidden/empty
rows, pointer behaviour and overflow; current watch log/controls/status/identity
checks; actual WASM phone/desktop final-frame screenshots. Build/copy/typecheck.
No deployment.

Implemented watch-messages as a transparent flex stack: contextual tip, ticker,
end-cause, combat log. Existing message elements and input handlers stay intact.
Inactive, suppressed fight and cut ticker rows collapse; empty log collapses.
Renderer keep-out now includes the visible end-cause too. No per-frame layout
reads added; no game state or wire changes.

Verified: new geometry/input test320/400/1440 covers short/long causes, all four
rows, >=8px gaps, bounds, transparent stack/log, hidden/empty/suppressed rows,
ticker clicks, history wheel scroll and Speed menu. Five targeted watch suites
pass27.8s (new layout, log, status, console, frame63checks); QA41/stall3/meters
also pass24.5s. Final new layout +console pass14.1s after suppressed-row change.
Build/typecheck/copy1562/diff clean; existing large-bundle advisory. WASM unchanged.

Actual headed WASM earned seed3, legal Bold run66, genuine D6 death at400/1440:
end-cause above log with8px gap, no page errors/overflow, viewed2nd heir stays
while engine inherits3rd; refreshed gold still8443. Verdict presentation held
only for screenshots then released to the actual death screen. Artifacts:
scratchpad/watch-message-spacing-20261006/{capture.mjs,results.json,
400-final-frame.png,1440-final-frame.png}. Screenshots shown. No deployment or
full-client-suite certification. No new bitmap art.
