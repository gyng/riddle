# Local core compilation — 2026-10-05

Reduce the wait after a real core edit while preserving simulation speed and
complete outputs. Shipping release settings are separate. This follows the
published bloodline release; it introduces no gameplay changes.

Acceptance: at least10% lower median warm edit and restore times, consistent
in both repetitions; identical complete outputs and raw saves on early/late/
tuned camps (8h offline and package reads), and one/two/three bloodline catch-up.
No representative simulation workload may regress over5% without an explicit
owner-approved tradeoff. Retention requires unchanged core/dayplayer/tooling
checks and routine FULL gates. Do not shrink seeds, replays or timeouts.

Isolated source87f07c5 (game07ae69c), rustc1.96.0, LLVM22.1.2. Prime each target
separately; cold preparation is excluded. Change the vision helper to an
algebraically equivalent body, build native_dev, restore exact original source,
and rebuild. Two pairs per candidate. Every restored executable matches its
original SHA256. Three runtime repetitions per saved camp/workload and bloodline
count. Inputs match their existing saved-camp reference hashes. Compare full
output files byte-for-byte, including complete raw saves, not just summaries.

| Fast profile candidate | Warm edit median | Restore median | Result |
| --- | ---: | ---: | --- |
| Existing: opt3, CGU16, debug1 | 25.696s | 26.006s | Reference |
| CGU32 only | 27.596s | 27.589s | Reject: slower builds |
| CGU64 only | 27.702s | 27.599s | Reject: slower builds |
| debug0 only | 9.396s | 9.259s | Retain:63.4% /64.4% lower |
| opt2 only | 23.695s | 23.089s | Reject: late simulations slower |

All candidates match all nine complete workload outputs exactly. Debug0 saved-
camp runtime ratios to reference are0.990/1.005 (early offline/packages),
0.998/0.998 (late),1.031/1.015 (tuned). One/two/three bloodline ratios are
0.999/0.955/0.976. These small differences are within the5% guard; no simulation
speedup is claimed. Opt2 late offline/packages regress8.8%/6.0%, and its edit
improvement is only7.8%. Preserve opt3, CGU16, incremental and no LTO.

Normal native fast builds now omit DWARF line information. For line-level CPU
profiling, explicitly set CARGO_PROFILE_FAST_DEBUG=1 with a separate target
(see NATIVE_DEV.md). Fast WASM already used debug0, so this change claims no
fast-WASM gain. Shipping release settings are unchanged.

Evidence (private): scratchpad/compile-20261005/{bench.py,results.json,bench.log},
scratchpad/compile-options-20261005/results.json and
scratchpad/compile-20261005/{more.py,more.log}. Retained executables, complete
JSON outputs and hashes accompany the results; experimental targets reclaimed
only after outputs were recorded. Production source was untouched during the
isolated experiments. Actual running watcher: two core edit/restore pairs, ready durations9.744/
9.483s and9.475/9.516s (includes250ms debounce). Poll-observed wall times
9.847/9.671s and9.630/9.606s. Original source SHA256 and restored production
native executable match the isolated debug0 original exactly. The first profile
switch rebuilt artifacts in47.615s; this is not a cold-build improvement claim.
Evidence: scratchpad/compile-20261005/{watch-check.py,watch-results.json,watch.log}.
Final acceptance PASS:536 core tests (one ignored),14 dayplayer recovery tests
(74.06s),11 tooling checks, tsc/copy/clippy, shipping build and routine FULL
571s. Fresh native source key dafafd349471570b; metrics100.4s, wire QA44.9s,
all18 selected player-fortnight cases PASS. Broad historical/system-removal
statistical audit was not run. No seed, budget, timeout or bar was changed.
58 complete native/shipping bridge replies match, including multihero live
state, shared wallet, upgrades, refusal and save migrations. Local shipping
WASM remains byte-identical to the prior local release:
a1974e7382cfaae5cc4acef14312d37fc16b27fe9170ad13a28e630eca899caf.
Evidence: scratchpad/compile-20261005/{verify-full.log,dayplayer.log,parity.json}.
The full acceptance time is not a suite speedup measurement.

Publication:source08f764a, Pages37289223600 SUCCESS. Public index and WASM
match the actual CI artifact bytes. Entire shipping site matches the previously
accepted bloodline-release CI artifact byte-for-byte, so its94 public UI checks
remain applicable. Public WASM (CI toolchain, separate from local builds):
3adfbebe739b0b468f94a1ab0c7a5f0ee3000cde97fd421a54af335e23e18bef.
Private evidence:scratchpad/compile-20261005/public-proof/{proof.json,artifact}.
