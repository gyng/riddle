# Parallel local UI checks — 2026-10-06

The full UI fixture contains132 checks across six independently runnable
sections: frame63, QA41 (both K/L), stall3, keep2, cut18 11, cut19 12. The
runner currently executes these in one browser serially. Measure original
full UI against the same six sections in the existing bounded worker pool,
with quiet serial/split/split/serial order and unchanged implementation.

If both paths pass all132 checks and split median wall time improves >=25%,
make bare ui and the default client suite expand to all six sections. Share
the section list with ui.mjs so the runner's full coverage cannot silently
omit a newly listed section. Explicit ui:part selections remain targeted.
Direct node tests/ui.mjs retains its serial full walk. Preserve timing-gate
headroom, pool width and failure handling. No tests/assertions removed, no
render/simulation/build changes. An isolated UI-suite speedup is not a claim
about full-suite wall time or game simulation speed.


Measured on unchanged a2e780f, seven-slot pool, isolated no-HMR Vite server
per run, no concurrent browser/profile/build jobs, serial/split/split/serial:

| Dispatch | Run1 | Run2 | Median |
| --- | ---: | ---: | ---: |
| Full serial UI |132.6s|132.5s|132.5s|
| Six bounded sections |41.3s|38.9s|40.1s|

All four runs passed all132 checks. Process wall time includes harness startup;
medians use unrounded raw values. Split improvement69.73%, about3.30x.
Artifacts: scratchpad/ui-parts-bench-20261006/results.json and four logs.
Baseline command was node tests/run.mjs ui before expansion; split command
named ui:frame ui:qa ui:stall ui:keep ui:cut18 ui:cut19 explicitly.

Implemented shared UI_PARTS in tests/lib/ui-parts.mjs; both the serial walk and
runner consume it. Bare ui and default discovery now schedule every listed
section. Longest-first hints reflect measured section durations. Explicit
ui:part and direct node tests/ui.mjs retain their existing semantics. Browser
pool, timing headroom, retries, isolated Vite, failure output and assertions
are unchanged. Full-suite contention/performance has not been measured.

After implementation, bare node tests/run.mjs ui passes six jobs/all132
checks in41.7s, confirming default dispatch and shared imports. Syntax and
diff checks pass. No full client-suite certification or deployment.
