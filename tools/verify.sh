#!/usr/bin/env bash
# Routine green certifies selected regressions; --exhaustive runs the broad balance audit.
#   tools/verify.sh --quick   tests (fast profile) → tsc → copy-lint                 (~45 s warm)
#   tools/verify.sh           + clippy → wasm (fast) → web build → quick gates       (~5 min warm)
#   tools/verify.sh --full    + shipping wasm (wasm-pack --release) → full gate table (bounded current-game regression;18 fortnight cases)
set -euo pipefail
cd "$(dirname "$0")/.."
mode=${1:-}
t0=$(date +%s)
# One test run: its output is kept for the summary lines, its status decides. tsc and the copy
# lint run alongside it (they need no core build).
log=$(mktemp); tsclog=$(mktemp)
( cd web && pnpm -s exec tsc --noEmit >"$tsclog" 2>&1 && node ../tools/copy-lint.mjs >>"$tsclog" 2>&1 && node --test ../tools/gold-range.test.mjs ../tools/runtime-key.test.mjs ../tools/camp-check.test.mjs ../tools/native-codegen.test.mjs ../tools/native-host.test.mjs ../tools/native-watch.test.mjs >>"$tsclog" 2>&1 ) & side=$!
cargo test -q --workspace --profile fast >"$log" 2>&1 && ok=1 || ok=0
grep -E "test result|error|panicked|FAILED" "$log" | grep -F -v "test result: ok. 0 passed;" || true
[ "$ok" = 1 ] || { grep -E "^(failures:|    [a-z_:]+$|thread .* panicked)" "$log" | head -20; rm -f "$log" "$tsclog"; exit 1; }
wait $side && sideok=1 || sideok=0
cat "$tsclog"; rm -f "$log" "$tsclog"
[ "$sideok" = 1 ] || exit 1
if [ "$mode" != "--quick" ]; then
  cargo clippy -q --workspace --all-targets -- -D warnings
  if [ "$mode" = "--full" ] || [ "$mode" = "--exhaustive" ]; then tools/wasm.sh --ship >/dev/null; else tools/wasm.sh >/dev/null; fi
  ( cd web && pnpm -s build >/dev/null )
  if [ "$mode" = "--exhaustive" ]; then node tools/gates.mjs --exhaustive; elif [ "$mode" = "--full" ]; then node tools/gates.mjs --full; else node tools/gates.mjs; fi
fi
echo "verify${mode:+ $mode}: green in $(( $(date +%s) - t0 ))s"
