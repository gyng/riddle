# Vision follow-through — 2026-10-05

Continue the owner's local simulation performance work. Named fast-WASM package
sampling places vision at7.40% of leaf samples; this is a profiling lead, not a
shipping timing claim. Prototype clipped endpoint iteration using the existing
line-of-sight masks. Preserve seeds, tick counts, ordering, complete outputs,
visibility at edges, wall mutations and all radius/fallback behavior.

Compare against the preserved unchanged native executable on early, late and
tuned package reads. Require identical complete outputs on every pair and at
least2% consistent median improvement on expensive late/tuned reads over seven
alternating pairs before retaining. Reject inconsistent gains. Verify against
the straightforward line-of-sight oracle, then run full engine/routine acceptance
and shipping/native parity before publishing any retained runtime change.

Evidence stays private under scratchpad/vision-perf-20261005.

Clipped endpoint candidate rejected at the three-pair screen. First pair
overlapped test compilation and is excluded from timings. Remaining two-pair
medians: late12.4481→12.0745s (+3.00%); tuned9.7161→10.1157s (−4.11%).
All complete package outputs match; direct LOS oracle passes across narrow maps,
edges, moves, wall mutations, reloads and radii−1..33. Neither production change
nor candidate test is retained. These small/noisy samples justify rejection,
not a quantified general regression. Proceed to checkpoint profiling.
