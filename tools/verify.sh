#!/usr/bin/env bash
# Green = cut done.
#   tools/verify.sh --quick   tests (fast profile) → tsc → copy-lint                 (~10 s warm)
#   tools/verify.sh           + clippy → wasm (fast) → web build → quick gates       (~1 min warm)
#   tools/verify.sh --full    + shipping wasm (wasm-pack --release) → full gate table (~4 min)
set -euo pipefail
cd "$(dirname "$0")/.."
mode=${1:-}
t0=$(date +%s)
cargo test -q --workspace --profile fast 2>&1 | grep -E "test result|error|panicked|FAILED" | grep -v "0 passed" || true
cargo test -q --workspace --profile fast >/dev/null
( cd web && pnpm -s exec tsc --noEmit )
node tools/copy-lint.mjs
if [ "$mode" != "--quick" ]; then
  cargo clippy -q --workspace --all-targets -- -D warnings
  if [ "$mode" = "--full" ]; then tools/wasm.sh --ship >/dev/null; else tools/wasm.sh >/dev/null; fi
  ( cd web && pnpm -s build >/dev/null )
  if [ "$mode" = "--full" ]; then node tools/gates.mjs --full; else node tools/gates.mjs; fi
fi
echo "verify${mode:+ $mode}: green in $(( $(date +%s) - t0 ))s"
