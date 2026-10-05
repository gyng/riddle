#!/usr/bin/env bash
# Build the wasm engine into web/src/engine/pkg.
#   tools/wasm.sh          fast: cargo `fast` profile (no debug info) + wasm-bindgen, no wasm-opt (~8 s after a core edit)
#   tools/wasm.sh --ship   checked reuse, or wasm-pack --release (fat LTO + wasm-opt)
#   tools/wasm.sh --ship --fresh   force packaging even when compiled input/output match
set -euo pipefail
cd "$(dirname "$0")/.."
if [ "${1:-}" = "--ship" ]; then
  # Cargo remains authoritative for source/toolchain/flag changes. Packaging is reusable only
  # for the same compiled wasm, tools and recipe, with every generated file checked.
  cargo build -q --lib --release --target wasm32-unknown-unknown -p riddle-wasm
  if [ "${2:-}" != "--fresh" ] && node tools/wasm-cache.mjs check; then exit 0; fi
  wasm-pack build crates/riddle-wasm --target web --out-dir ../../web/src/engine/pkg --release
  node tools/wasm-cache.mjs record
  exit 0
fi
# wasm-bindgen CLI must match the crate version; wasm-pack caches matching binaries.
want=$(grep -A1 'name = "wasm-bindgen"' Cargo.lock | grep version | head -1 | sed 's/.*"\(.*\)"/\1/')
bg=""
for d in ~/.cache/.wasm-pack/wasm-bindgen-*/; do
  if [ -x "$d/wasm-bindgen" ] && "$d/wasm-bindgen" --version 2>/dev/null | grep -q "$want"; then bg="$d/wasm-bindgen"; break; fi
done
if [ -z "$bg" ] && wasm-bindgen --version 2>/dev/null | grep -q "$want"; then bg=wasm-bindgen; fi
[ -n "$bg" ] || { echo "no wasm-bindgen $want; run: cargo install wasm-bindgen-cli --version $want" >&2; exit 2; }
# No debug info for the wasm, even when native profiling is enabled: wasm-bindgen strips
# DWARF from the pkg anyway, and emitting it was ~45 % of a rebuild after a core edit (13.7 → 7.6 s,
# docs/ITERATION_SPEED.md round 3). Native line tables are now opt-in (docs/NATIVE_DEV.md).
CARGO_PROFILE_FAST_DEBUG=0 cargo build -q --profile fast --target wasm32-unknown-unknown -p riddle-wasm
"$bg" --target web --out-dir web/src/engine/pkg target/wasm32-unknown-unknown/fast/riddle_wasm.wasm
echo "wasm: fast build → web/src/engine/pkg ($(stat -c %s web/src/engine/pkg/riddle_wasm_bg.wasm) bytes)"
