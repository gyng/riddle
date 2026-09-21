#!/usr/bin/env bash
# Shipping build for a rating cohort: release wasm, production web build, preview server on :5230.
#   tools/ship.sh            build + (re)start the preview server, print the pinned commit
#   tools/ship.sh --preview  the same on the fast wasm (no LTO, no wasm-opt): ~25 s instead of ~2 min; the engine's
#                            results are the same source (integer-deterministic), runtime within ±5 % — for QA rounds;
#                            a blind cohort's Build field stays the --ship build (docs/ITERATION_SPEED.md §3.1)
#   tools/ship.sh --stop     stop the preview server
set -euo pipefail
cd "$(dirname "$0")/.."
port=${RIDDLE_SHIP_PORT:-5230}
if [ "${1:-}" = "--stop" ]; then pkill -f "vite preview --port $port" || true; exit 0; fi
[ -z "$(git status --porcelain --untracked-files=no)" ] || { echo "ship: working tree has uncommitted tracked changes; commit first" >&2; exit 2; }
if [ "${1:-}" = "--preview" ]; then tools/wasm.sh >/dev/null; else tools/wasm.sh --ship >/dev/null; fi
( cd web && pnpm -s build >/dev/null )
pkill -f "vite preview --port $port" || true
( cd web && setsid nohup pnpm exec vite preview --port "$port" --strictPort > /tmp/riddle-preview.log 2>&1 & )
for _ in $(seq 1 40); do curl -sf "http://localhost:$port/" >/dev/null && break; sleep 0.5; done
echo "ship${1:+ $1}: $(git rev-parse --short HEAD) at http://localhost:$port/"
