# Preserve the finished town on refused ascension — 2026-10-06

App.ascend catches every engine error and calls again(), silently resetting to
a new town. A missing method, rejected variant or runtime failure must retain
the current town, ending, selected bloodline and durable save instead. Only a
successful core ascension can adopt state and navigate to camp. Return success
to the ending UI; prevent concurrent attempts, re-enable choices after refusal
and expose concise Retry feedback. Never invent completion or weaken the core's
reach-bottom requirement. Verify rejection, missing bridge, success and double
click; actual earned pre-ending town rejection must preserve full save bytes.
No simulation/balance changes. Full earned ending/variant presentation remains
next; this is the failure-path fix identified by that audit. Push approved;
do not deploy.

Implemented: app returns success, rejects concurrent attempts and catches core
refusal without again(). Ending choices re-enable with Try again feedback.
After a successful core transition, metadata failure cannot repeat ascension;
new state persists and opens camp. Existing successful cleanup retained.

12 UI checks each320/400/1440 cover rejected/missing bridge, exact save and
client counters/loadout, feedback, single in-flight request, successful variant
and failed post-commit metadata. Final affected5jobs23.2s pass; earlier frame/
ascension4jobs24.6s pass. Actual headed WASM earnedD21 town rejected premature
Hunted: gold53038/Legacy536, complete engine AND durable-save bytes unchanged,
still camp, no page errors. No fabricated King/ending screenshot. Evidence
scratchpad/item-icons-20261006/ascension-refusal.json. Successful King/all-four
variant carry/reset semantics remain unverified on earned towns; full ending
presentation is still the next task. No Rust/WASM/deployment changes.
