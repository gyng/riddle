#!/usr/bin/env bash
# tests → clippy → wasm → tsc → build → copy-lint → gates. Green = cut done.
set -euo pipefail
cd "$(dirname "$0")/.."
quick=${1:-}
cargo test --workspace -q
cargo clippy --workspace --all-targets -- -D warnings
wasm-pack build crates/riddle-wasm --target web --out-dir ../../web/src/engine/pkg --release >/dev/null
( cd web && pnpm -s build )
[ -f tools/copy-lint.mjs ] && node tools/copy-lint.mjs
if [ "$quick" != "--quick" ] && [ -f tools/gates.mjs ]; then node tools/gates.mjs; fi
echo "verify: green"
