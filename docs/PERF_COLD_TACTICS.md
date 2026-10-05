# Cold Tactics comparison — 2026-10-05

Continue the owner's actual simulation/iteration optimization work. Publication
is manual; do not deploy. Profile the live24-simulation query on the current
source. Named fast WASM attributes functions; shipping WASM is the latency
acceptance target. Fresh Game/cache per query, loading outside timed samples,
no compilation alongside timing. Exact whole option replies and raw saves
must match. Preserve seeds, budgets, policy, ordered results and all gates.

First probe: permit cross-module inlining of definition lookup wrappers.
Definitions remain single-source static tables; lookup algorithm, collisions,
unknown fallback and order remain identical. Screen fast WASM on three saved
camps, alternating preserved binaries and checking every complete reply/save.
Retain only with a consistent benefit on representative native/shipping tests
(at least2% median improvement on late/tuned, no workload over5% slower).
Otherwise revert the source probe and record evidence. Do not turn profiler
shares or microbenchmarks into a whole-app/FPS/build-time speedup claim.

Current sourcee04ce46, late camp SHA256
153609d33fdf71436994212db44ca4d7a8323fb4389f00057aacef1a7597e61a.
Named fast-WASM24-simulation cold reads:8.121/8.202/7.846s (three fresh games,
module warm-up excluded). Profile includes loading/save serialization; reported
read timings exclude those. The workload is packageOptions itself, excluding
the small query-key/bridge-memo step and app/render scheduling. Inclusive simulation setup1.53%, Game clone0.41%,
set_rules0.06%; a setup rewrite cannot materially reduce this specific query.
Tick-batch inclusive94.27%; hero action59.09%, chores25.21%. Leaf vision6.92%,
nearest tile6.09%, flood4.10%, item/monster definitions2.83/2.77%.
Inclusive allocation and call shares overlap; do not add them as independent
costs. No shipping/whole-app speedup follows from these attribution samples.

Definition wrapper inlining rejected. Three alternating timed pairs per camp
after one discarded warm-up pair, named fast WASM, fresh24-simulation queries:
late8028.8→7976.7ms (+0.65%), tuned8582.9→8690.4ms (−1.25%), early was3.14%
slower. All12 complete price/save pairs (including warm-up) match exactly;
each query also preserves its own full raw save. This misses the2% screen,
so production defs.rs was restored byte-for-byte. No native/shipping acceptance
claimed; no need to rebuild/publish a rejected probe. Preserved binaries,
profiles, full replies and pair results: scratchpad/cold-profile-20261005.

Next: repeated work inside hero/chores. In particular rules::shadows is2.56%
inclusive and builds an implicit foe condition/scope collections repeatedly;
compare allocation-free borrowed predicates against the existing semantics.
Avoid clone/preparation architecture until another workload establishes a
larger setup share. Previously rejected BFS/layout probes remain rejected.
