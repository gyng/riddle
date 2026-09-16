# Render performance

Method: every number below comes from `tools/browser.mjs` `launchGpu()` (headed Chromium under
WSLg, ANGLE on Mesa D3D12 → RTX 3080), Vite dev build of `web/render-demo.html`, rAF rate over
5 s, per-frame timings from `viewer.stats()` (added 2026-09-16; scripts lived in the session
scratchpad, ~40 lines each: blank / `gl.clear` / demo pages, and a CDP CPU profile).

## Is 33 fps the game or the harness?

The harness. A page with nothing but a `gl.clear()` per frame is capped by the WSLg present path
as a function of the **window's CSS width**, independent of canvas pixel count and DPR:

| window (CSS) | DPR | canvas px | rAF fps, `gl.clear` only |
|---|---|---|---|
| 400×800 | 3 | 1200×2400 | 60 |
| 400×800 | 4 | 1600×3200 | 60 |
| 640×800 | 1 | 640×800 | 55 |
| 800×800 | 1 | 800×800 | 45 |
| 1024×800 | 1 | 1024×800 | 34 |
| 1280×800 | 1 | 1280×800 | 29 |
| 1280×800 | 1 | 400×800 (canvas inside the wide window) | 28 |
| 1280×800 | 2 | 2560×1600 | 28 |

A blank page (no WebGL) is 60 fps at every size. So: **1280×800 in this harness cannot show more
than ~29 fps whatever we do**, and the demo's 27–28 fps there is the ceiling, not the pipeline.

At 400×800@3 the demo measured **59–60 fps** in this session (three runs: 59.2, 58.6, 59.6 at 1×;
58.9 at 4×), i.e. the earlier 33 fps in INTEGRATION.md was not reproducible; the harness fps at
the phone viewport swings with host load (one 4× run gave 41 fps with the JS cost unchanged at
1.0 ms, so treat harness fps as a coarse gate, not a cost measurement).

## Per-frame cost of the pipeline (400×800@3, k=6, target 404×804)

| | mean | p50 | p95 |
|---|---|---|---|
| CPU `build()` + both `renderer.render` passes, 1× | 0.74 ms | 0.70 | 1.40 |
| same, 4× (more sprites moving) | 0.74 ms | 0.70 | 1.50 |
| of which `build()` (state → instance arrays) | 0.11–0.14 ms | | |
| draw calls | 5–7 | | |

CDP profile over 5 s at 4×: main thread 85 % idle; `bufferSubData` 0.07 ms/frame, `build` 0.07,
three.js render overhead ~0.3, everything else noise. GC 0.1 %.

**GPU time**: `EXT_disjoint_timer_query_webgl2` is advertised in this harness but no query ever
resolves (`QUERY_RESULT_AVAILABLE` stays false through 300 frames, with or without `gl.finish()`),
so the GPU number is unmeasurable here. The demo HUD prints it when the browser resolves it
(desktop Chrome on a native GPU, Chrome on Android): `GpuTimer` in `web/src/render/gputimer.ts`
issues one `TIME_ELAPSED` query per frame around the whole pipeline, polls without stalling, and
gives up after 120 unresolved frames so the polling cost (≈0.14 ms/frame of ANGLE round-trips)
does not stick around where it cannot work. Where it does not resolve the HUD shows `gpu ms -`
and the CPU wall time is the (lower-bound) proxy.

Reasoning about phones, since the GPU can't be measured here: the target pass is 0.32 Mpx across
≤7 instanced draws with alpha-test discard; the blit is one 2.88 Mpx fullscreen pass (1200×2400
at DPR 3) with one texture fetch, an 8-iteration nearest-palette loop and a Bayer function per
pixel. At 60 Hz that is ~190 Mpx/s of trivially-shaded fill, an order of magnitude below a
mid-range mobile GPU's fill rate, so the expected GPU cost is ~1 ms and the target of < 4 ms per
frame has headroom. If a phone measurement ever shows the blit dominating, the next lever is to
quantise at target resolution and upscale with a plain nearest blit (36× fewer palette searches
at k=6); it is not done now because it moves the fog-band edge from device to target resolution
and the brief was identical output.

## What changed

- `QuadLayer.push` is compare-and-write; `end()` skips the GPU upload when no instance changed
  (tiles/items/shadows most frames, everything when paused). Output verified pixel-identical:
  deterministic-clock screenshots before/after, 0 differing pixels of 2,880,000 at 400×800@3
  and 0 of 1,024,000 at 1280×800@1 (HUD hidden).
- `ViewerStats` gained `cpuMs`, `cpuP95`, `buildMs`, `gpuMs`, `gpuP95`, `gpuTimer`, `fps`; the
  demo HUD's second line prints `frame ms (cpu) / gpu ms / calls / fps` and the console logs the
  same every 2 s.
- No change to `setSize`/DPR handling (already only on a size-key change), no
  `preserveDrawingBuffer`, no readback: those were checked and were not costs.
