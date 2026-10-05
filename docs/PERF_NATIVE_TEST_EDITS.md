# Skip proven test-only native watcher edits — 2026-10-06

Native watcher currently queues Cargo for every core src edit, including ten
cfg(test) modules absent from native_dev.d. It can join a build lock and cause
a second rebuild during a test-writing loop despite no runtime change.

Skip a core root test module only when its current lib.rs has exactly one
simple #[cfg(test)] mod declaration and Cargo native_dev.d excludes it.
Require valid dependency shape/root and lib.rs dependency. Missing/unreadable
proof, changed declaration, compiled runtime include, nested/unknown module,
custom target directory/escaped paths all retain conservative rebuilds.
Runtime/bridge/preset/manifest invalidation unchanged; unit tests still run.
No compile/test/gate is removed. No claim that this explains previous72s quick
check: no live native server currently confirms that causal attribution.

Verify classifier boundaries with temporary source/compiler-dependency
fixtures and actual existing Cargo deps. Verify no watcher request/build for
all ten proven test modules, runtime change still queues. Existing debounce/
serialization/recovery/close checks retained. Node tool checks, tsc/copy/diff;
no Rust runtime change, WASM rebuild, screenshots, full balance audit or deploy.

Verified:9 native watcher/bridge/host/runtime-key checks pass3.31s, including
real-compiler proof that test edits preserve runtime identity while runtime/
include/flag/manifest edits invalidate it. Fixtures verify missing/malformed/
incomplete/escaped dep-info, current declaration changes/duplicates, runtime
include of test-named source, nested/unknown modules and custom target fallback.
Actual current repo lib.rs + Cargo native_dev.d classifies all10 test modules
as excluded. Passing those actual paths through the queue produces0 builds;
before classifier would coalesce them to1 build. Runtime tree.rs queues1.
This measures avoided queue entries, not a wall-time improvement or a live
server editing benchmark. Native5367/5368 not listening at inspection; no
attribution of the prior72s delay. tsc/copy1503/diff pass. Evidence:
scratchpad/native-test-edits-20261006/{checks.log,classification.json,tsc.log}.
No checks/gates removed, Rust/WASM/UI change or deployment. Existing servers
need normal restart/fresh port to load the changed watcher module.
