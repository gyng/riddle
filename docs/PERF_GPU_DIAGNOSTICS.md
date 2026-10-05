# Reliable GPU diagnostics — 2026-10-06

Opening-session walks report a WebGL query enum warning and a texture-copy bounds
warning. Trace the actual calls before fixing either. GPU timer uses WebGL1 EXT
query-result constants with the WebGL2 query API; its stall threshold counts
issued queries even though the ring stops issuing after16. Restore actual
asynchronous timing, accurate unavailable state, bounded pending-query lifetime
and cleanup. No blocking result reads or per-frame GL error drains. Timing is
measurement only; preserve replay/simulation/rendered output.

Acceptance: fake protocol test uses an extension without query-result constants;
ready samples return milliseconds/percentiles; pending reads do not block or
reuse in-flight queries; never-resolving driver disables after bounded frame
attempts; disjoint samples are discarded; disposal ends/deletes live queries;
missing extension/allocation failure gives explicit unavailable readings. Actual
headed GPU run has resolved finite timings and no timer enum warning; context
loss still restores replay/rendering. Trace texture-copy warning independently,
fix only its proven cause. Scoped tests/tsc/copy/build and screenshots; no deploy.

Implemented: WebGL2 core QUERY_RESULT_AVAILABLE/QUERY_RESULT constants; pending
queries age by frame attempts, including a full ring and stalls after prior
results. No GL error drain in polling. Disjoint clocks discard every pending
query and old percentile samples; allocation/context failure and disposal clear
availability/readings and release queries. Lost-context event explicitly clears
viewer timing stats; restore creates a new timer.

Texture trace: texSubImage2D uploaded a512×512 canvas into the sprite-normal
texture's earlier default300×150 allocation. SpriteNormals now initializes its
canvas to the sprite atlas size before first texture use; grow/shrink changes
replace the texture/source and material uniform after disposing the old one.
Same-size derivations keep their existing allocation/UVs. No simulation changes.

Evidence: protocol + actual GL texture upload/grow/shrink tests741 checks pass;
context-loss recovery19 checks pass (26.9s). The historical recovery harness
was stalled before manual house construction, then silently clicked the removed
footer Jump shortcut. It now waits for boot/hero and opens Jump in Speed, with
all original recovery/exit assertions and60s exit bound retained; also asserts
unavailable GPU readings during context loss. Earlier171s failed check is not a
performance baseline for the game.

Actual real-engine headed GPU run: both instrumented and clean checks have zero
warnings/errors; clean GPU latest0.183296ms/p951.262592ms are finite (previously
NaN). This proves timing availability, not an FPS or simulation speedup. The
trace instrumentation drains GL errors to identify the failing call, affecting
the old timer's warning frequency; do not compare warning counts as a benchmark.
Tsc/build, copy1492 and diff check pass. Local evidence and checkpoint screenshot:
scratchpad/render-diagnostics-20261006/{baseline,after,clean}.json,
watch-clean.png, protocol-texture.log, final-tests.log, context-controls.log.
No deployment or full balance audit.

Saved actual-WASM Warlord death(run3 from a migrated camp) captured at400/1440:
no warnings/overflow. This is historical death-screen coverage, not a live boss
encounter. It exposes raw `Steady · foes 1+ → attack` copy and a sword recommendation
without an icon. Next player-facing change: readable last-action context and
shared item icons on death recommendations, then a live boss encounter audit.
