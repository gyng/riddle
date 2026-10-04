# Faster local iteration

Start the real Rust engine behind the dev app:

```sh
tools/native-dev.sh
# http://localhost:5367/?engine=native
```

This uses a separate loopback-only Vite port, preserving the usual WASM server
on5219. Each foreground/background/refine lane owns an ordered native process;
by default each has8 forecast threads (`RIDDLE_NATIVE_THREADS` to change).
`?lanes=0` selects one lane. Errors stay visible; native never falls back to fake.
Production builds remove the native client and transport. The shipping engine
and explicit exhaustive balance audit remain the reference; routine --full now
checks18 selected current-player cases (docs/UX_SIMPLE.md).

After a Rust edit, rerun `node tools/native-build.mjs`. The next call restarts
native processes and restores their saves. There is no WASM packaging step.
The bridge is generated directly from shipping bindings; unknown signatures
fail closed. Run `node tools/native-codegen.mjs` after bridge edits. The client
checks the compiled bridge hash at boot and requests a rebuild when it is stale.

## Numerical tuning without compilation

Copy the complete Rust-owned template, then use a separate port:

```sh
cp tuning/balance.json scratchpad/candidate-balance.json
RIDDLE_BALANCE_FILE="$PWD/scratchpad/candidate-balance.json" RIDDLE_NATIVE_PORT=5368 tools/native-dev.sh
node tools/camp-check.mjs camp.json --balance scratchpad/candidate-balance.json
tools/tune.sh idle-d8 --balance scratchpad/candidate-balance.json --fresh
```

Edit that file to tune package levelling, scars/drill timing, or Steady/Guarded
heal/home/rest thresholds. Rust validates the complete profile: unknown keys,
missing values, zero scars, invalid percentages and unordered level thresholds
fail. Profiles are immutable inside a process. A changed valid file restarts
sessions between calls and restores the open game; invalid edits keep the last
valid process usable once the file is corrected. Existing live runs retain their
compiled rule indices until the ordinary camp recompile.

The template must equal `balance::DEFAULT` (unit checked). Shipping always uses
that Rust default; dev overrides require the `dev-balance` feature. To promote
numbers, update the Rust default and template, then run the affected rows and
full shipping verification. Profile tuning is explicitly targeted feedback;
`gates.mjs` rejects a profile on a full acceptance invocation. Exact profile
bytes are part of tuning/diagnostic identities and reference comparisons.

A server's profile path/thread count is fixed at launch. Use another port for
another configuration; editing its existing profile needs no restart or build.
Default builds reuse the ordinary gate/test core artifact; tuning builds have
a separate stable executable so switching Cargo features cannot replace a
running server with the wrong mode. The health endpoint `/__native/health` reports binary/profile hashes and width.

## Failures first and interrupted cases

Targeted fail-fast runs remember the seed that completed a failing verdict in
`target/gates/failure-hints/`. After a change, that seed is scheduled first.
Hints survive runtime edits but contain no results and cannot pass a gate.
All requested seeds and existing verdict bounds remain; final reductions retain
numeric seed ordering. `RIDDLE_SEED_ORDER_NUMERIC=1` disables the scheduling hint.

Long standard/full tables now checkpoint completed check-ins automatically.
For a direct or targeted run, add `--resume`:

```sh
tools/tune.sh nothing-required --resume
cargo run -q --profile fast -p riddle-core --example dayplayer -- --resume --gate
```

Checkpoints contain complete bot state, random state, accumulated output,
internal live-run bookkeeping and forecast caches. They bind the full executable,
CLI parameters and behavior-affecting environment. A changed runtime/harness,
parameters or profile starts fresh. Corrupt/truncated/wrong-key snapshots are
ignored; files are atomically replaced. `--fresh` ignores checkpoints too.
Resumed configurations run independently until completion; this can lose some
shared-prefix efficiency. A crash can still lose the current unfinished check-in.
Player save migrations are not applied to these internal snapshots. Replay
capsules are disabled by the harness; the disposable visibility cache rebuilds.

Validation commands:

```sh
node --test tools/native-codegen.test.mjs tools/native-host.test.mjs
RIDDLE_NATIVE_TEST_BIN="$(node tools/native-build.mjs --tuning)" node --test tools/native-balance.test.mjs
cargo test -q --profile fast -p riddle-core --example dayplayer checkpoint_tests
node tools/native-parity.mjs target/native-dev/default/native_dev scratchpad/parity early.json late.json tuned.json
```

Native/WASM parity compares complete replies and raw saves. The native path also
revealed an existing32/64-bit event wording difference; the native app adapter explicitly selects the
shipping WASM's32-bit choice. Standard native gates retain their established
64-bit event fingerprint. Shipping wording and gameplay remain unchanged.
Full fresh simulation still requires the complete seed table; these changes
improve feedback, avoid recompiles and recover work, rather than promising all
fresh simulation CPU within minutes.

The final manual-construction runtime also matches all27 complete shipping
replies/saves on three representative camps. Quiet round: packages6.6/7.4/7.5×,
late wall4.2×, forecasts5.1–11.1×. Evidence includes binary, shipping module,
bridge, harness and input hashes. These are engine-call timings, not an app FPS
or cold Rust compilation claim. Routine fresh gate timing is recorded separately.
