# Local saved-camp iteration

Contract: shorten the edit/measure loop without changing simulation inputs,
sample counts, outputs or acceptance gates. This is diagnostic feedback, never
a substitute for the targeted rows or final full gate.

- Build the native diagnostic once per invocation, using Cargo's fast profile
  and reported executable path. Each measured workload starts in a new process
  with its own loaded camp and empty transient caches.
- Accept existing game saves. Check their SHA256 and the workload against a
  recorded reference before comparing. Compare complete raw outputs, not only
  summaries or hashes. Repeated identical inputs must produce identical bytes.
- Record only after all workloads succeed; refuse to replace a reference.
  Missing, corrupt or mismatched references fail. Changed runtime binaries may
  be compared to the reference; record both binary hashes for provenance.
- Demonstrate early/late/tuned 8-hour checks within 15 seconds warm on this box;
  record build time separately. Package/wall costs are measured separately.
- Add diagnostic clone and initial-run setup measurements before considering
  immutable-input changes. These are indicative microbenchmarks, not a claim
  that every candidate's setup costs the same.

Creative next options: keep a small library of saved failing camps, replay the
previous failure before the whole target row, watch/debounce only relevant
inputs, and resume interrupted cases at check-ins. Checkpoints must bind the
exact runtime, harness, state and parameters; they cannot carry old progress
through a runtime change. Data-driven balance editing could avoid recompiles,
but needs explicit input hashes and shipping-equivalent behavior before use.

## Use

Use native `Game::save` JSON files (the engine JSON inside an app backup).
Start with a few camps that exercise the changed behavior, including its
previous failure. The defaults replay an 8-hour absence once per camp.

```sh
node tools/camp-check.mjs early.json late.json tuned.json --record scratchpad/camp-reference
node tools/camp-check.mjs early.json late.json tuned.json --compare scratchpad/camp-reference
node tools/camp-check.mjs tuned.json --mode packages,wall --repeat 3
node tools/camp-check.mjs tuned.json --mode clone,setup --repeat 3
tools/tune.sh <affected-row-ids>
```

Record before an optimization and compare afterward. Changes to game balance
may deliberately change the outputs; review them and use the affected target
rows to check the intended behavior. References are never updated automatically.
Each invocation snapshots the input files, builds once, then runs fresh processes.
Build time is separate from engine measurements. Package and wall checks can
still take seconds to tens of seconds on a deep camp.

## Measured checkpoint

Early/late/tuned saves from the existing seed15 progression investigation:
one 8-hour pass took4.25s total including0.27s warm Cargo build; three passes
took12.08s including0.04s build, with complete raw report/save agreement on all
nine executions. This is native direct catch-up, not the browser's sliced path
and not a full progression gate. A canary that passes can still miss other seeds.

4096 clone/setup operations per measurement, three rounds each: clones30/75/83µs;
initial-run setup107/186/207µs for early/late/tuned. These microbenchmarks include
dropping the copied games. Applied illustratively to the previous heavy-search
send counts, entire setup is around1–2% of the wall/package time. This is not a
strict bound: candidate camps, passages and later-floor work differ. It argues
against expecting a large gain from initial preparation alone; it does not rule
out sharing immutable geometry or reducing work inside the run.

CLI rejects overwritten, missing, mismatched-input and corrupted references
(four real negative invocations); unit checks cover complete-byte comparisons,
workload/input binding and invalid manifests. Reference checks are in the quick
verification tier. Evidence: scratchpad/dev-loop-20261004 and
scratchpad/sim-work-20261004/dev-loop-*.log.

Final snapshotted-input invocation:4.05s total, build0.04s, every output matches.
Quick verification PASS33s (522 native tests/one ignored, TypeScript, copy and
three tooling tests); diagnostic clippy PASS; complete gate table retains all
three genuine accepted leg hits. No production engine or UI change.
