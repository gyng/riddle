# Equivalent package panels — 2026-10-05

The current package query clones and simulates every offered move separately.
Rust forecast keys identify16/14,38/24 and17/16 requested/unique own-or-wall
panels in the frozen early/late/tuned camps. Count evidence is private under
scratchpad/flood-perf-20261005/panel-keys*. Exact-key duplication is an opportunity,
not a measured speedup.

Prototype within one package query: group moves by both own and wall panel keys,
plus the raw rules key to preserve passage policy inputs. Compute representative
panels once; retain every original move, its own price and stable score ordering.
Reuse base results when keys agree. No persistent cache, budgets, seeds or wire
changes. Keep existing native worker scheduling semantics; if grouping would
collapse a threaded query to one job, use the original path rather than switch
its simulations to a different parallel-budget execution path.

Screen with three alternating pairs, then seven if promising. Require at least
15% lower late-package median,2% lower tuned median, no workload regression over
5%; exact complete JSON outputs on all three camps and unchanged simulation
budgets. Shipping-WASM late query target10% lower. Broaden regression checks only
if the screen passes. Do not ship a merely lower allocation/work count.
