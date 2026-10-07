# Cut80 — make short replay controls visible

Contract2026-10-07. Earned seed27 D8death cause opens a real forty-tick clip.
Screenshot qa-cut78-interactive/shots/death-replay.png: header t13375 is an
internal simulation index, and repeating the clip requires an undisclosed
canvas tap. Preserve exact replay window/event data and primitive fallback.

Keep floor/cause header; remove internal tick from player text, retaining its
existing data attributes. Add one chunky native Replay button below the view:
Loading until renderer ready, Playing while the same40tick clip runs, Replay
when paused. Pointer/keyboard reruns that exact window. Canvas tap remains.
No new panel, core mutation or extra simulation; static fallback says Still frame.

Gates: exact window/floor/seek/stop remain, button disabled during load/play;
button and keyboard restart at from, finish at to, repeated use doesn't create
extra timers; Escape disposes viewer. Technical tick absent from visible header;
no overflow at320/400/1440. Existing clips check and copy/type/build pass.
Real earned D8clip before/after screenshots verify discoverability. No score/deploy.

Verification: original QAJclips plus Replay control checks13PASS24.0s. Initial
control fixture timed out because its caption probe intentionally froze viewer
speed; restore original clock after probe, same10s cap preserved. Actual earned
run3 log (342302bytes, no generated/granted state) loaded as render-only replay
at320/400/1440: exact40tick span, end pause, keyboard replay,44px button,bounds,
Escape cleanup and unchanged lineage all PASS. Screenshot400-paused shown;
background is fresh empty town for isolated render verification, not QA progress.
TypeScript/copy1749zero/build/diffPASS. No Rust/wire changes.
