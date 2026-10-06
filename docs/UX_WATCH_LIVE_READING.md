# Live watch reading — 2026-10-06

Make playback mode visible without adding console tiles. Preserve exactly two
console actions, Speed and Town menu; selected Normal/Fights only/Fast fits
inside the Speed tile at320/360/400/840/1440. Keep Run ended visible through
the final picture rather than silently hiding the status. Pausing remains a
viewer action: the underlying run continues.

The plain combat overlay retains three visible lines and80 entries. Scrolling
back must hold the visible text when new events arrive, including when the
oldest entry is trimmed. Latest resumes following; keyboard focus supports
history scrolling. Damage/item colours and transparent styling stay intact.
Latest occupies its own line only while browsing history, preserving stacked
message spacing. No Rust/wire/game state changes.

Implemented persistent mode sublabel, retained end badge, scroll-follow state,
Latest action and keyboard-focusable history. Disable browser scroll anchoring
and compensate for the removed row's actual height when reading older entries.
The normal following path adds no height measurement; no animation-loop reads.

Verification: targeted log/status/console/message-spacing and QA+stall suites
5/5 pass48.4s. Log tests cover six floor/start/viewport combinations,80-entry
bound, uniquely numbered pickups, exact visible text/top retention after trim,
and resuming Latest. Console tests assert mode changes and label/touch bounds
at five widths. End badge explicitly asserted visible. Frame63checks also pass.
Typecheck/build and copy lint1570 literals pass; existing large-bundle advisory.
No full client or balance certification claimed.

Real headed WASM earnedseed3, legal Bold,400/1440 live and paused screenshots:
real engine, visible selected mode and status, no horizontal overflow/pageerrors.
Artifacts: scratchpad/watch-experience-20261006/. A combined live/pause/end
capture exceeded its60s end wait; separate checkpoint captures completed.
Final capture holds only verdict presentation after a real death, then releases
to the actual death screen. No fabricated outcomes or performance claims.
No deployment; shared5219 preserved.
