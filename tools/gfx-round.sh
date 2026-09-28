#!/usr/bin/env bash
# One eval round on a frozen server: a fresh no-HMR Vite (web/tests/vite.test.config.ts) on a free port — another agent's edit in web/src
# can no longer reload the walk's pages mid-capture — then tools/gfx-eval.mjs against it, then the server goes (`exec setsid`: the
# server keeps the subshell's pid as its session and group, so the trap's group kill reaches it — round 7 found six leaked servers).
#   tools/gfx-round.sh <outdir> [gfx-eval args…]
set -euo pipefail
cd "$(dirname "$0")/.."
out=$1; shift
port=$(node -e 'const s=require("net").createServer();s.listen(0,()=>{console.log(s.address().port);s.close()})')
(cd web && exec setsid node node_modules/vite/bin/vite.js --port "$port" --strictPort --config tests/vite.test.config.ts > /tmp/gfx-vite-$port.log 2>&1) &
pid=$!
trap 'kill -- -$pid 2>/dev/null || kill $pid 2>/dev/null || true' EXIT
for _ in $(seq 1 120); do curl -sf -o /dev/null "http://localhost:$port/" && break; sleep 0.25; done
curl -sf -o /dev/null "http://localhost:$port/src/main.ts" || true
node tools/gfx-eval.mjs --out "$out" --port "$port" "$@"
