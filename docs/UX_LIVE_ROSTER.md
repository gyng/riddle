# Current activity after a send — 2026-10-06

Real first-send screenshot shows Ready while watching. Normal human sessions
already poll Rust lineage once a second; webdriver/runs=0 disables that clock.
The concrete product gap is the stale pre-send state until the first poll.
Refresh Rust-owned slots immediately after send and before initial viewer load.
Reuse that guarded read for the existing periodic watch refresh. Do not invent
slot state from a UI action or snapshot. No new poll interval or background
advance. Share an in-flight read for the same view/selection; reject replies
from a disposed/replaced watch or changed bloodline.

Acceptance: Ready→Live D1 at first watch load, exact Rust floor/rest/wait updates
on existing live notifications, unchanged other slot/XP/Legacy values. Controlled
browser coverage400/1440 for immediate state, dedup, periodic refresh, delayed
reply after slot switch and replacement view, failed-read recovery and no clock
advance. Real-WASM first-send screenshots show Live instead of Ready. Existing
watch status/hero identity gates, build/typecheck/copy pass. No Rust, tuning or
deployment changes.

Verified: immediate send refresh, unchanged other slot/XP/Legacy, exact live
floor/rest/wait data, existing periodic refresh, shared read, rejected old
selection/view replies, old completion preserving newer read, failed read retry,
no clock advance:19 checks per width400/1440 (38 total),2.6s isolated. Existing
hero-names30 checks and watch-status pass on isolated server15.5s combined
with initial failing new fixture; product checks passed unchanged. First fixture
incorrectly expected an actual mobile watch roster and then appended to a camp
container absent during watch. Explicit roster fixture mounted into watch and
passed all cases. Real screenshot harness's same nonexistent mobile roster
lookup corrected: mobile asserts underlying Rust live slot, desktop visible row.
No assertions weakened on slot freshness or race guards.

Headed real-WASM seed2 house/send400/1440 with runs=1: first viewer frame slot
live (desktop LiveD1), normal polling retained current live state, paused picture
shows D1 while roster correctly follows core atD2. No errors/overflow. Final
desktop screenshot shown. No mobile-watch roster was added. Source Rust/WASM
unchanged. Build/typecheck/copy1524/diff pass; existing bundle advisory. Evidence:
scratchpad/live-roster-20261006/{400,1440}-watch.{png,json},real-watch-final.log;
scratchpad/live-roster-tests-verified.log,live-roster-tests.log,live-roster-build.log.
No deployment. Next: investigate first-watch framing: real screenshots show
floor scenery while hero is at/beyond the view edge; do a GPU camera/actor audit
before another generic layout adjustment.
