#!/usr/bin/env bash
# Idempotently ensure a Vite dev server on port 5219 (raters' pages hot-reload on it; a running one is
# never restarted), then print its URL. Log: /tmp/riddle-vite.log.  RIDDLE_PORT overrides the port.
set -euo pipefail
cd "$(dirname "$0")/.."
PORT=${RIDDLE_PORT:-5219}
URL="http://localhost:$PORT/"
up() { curl -sf -o /dev/null --max-time 2 "$URL"; }
if ! up; then
  setsid nohup pnpm --dir web exec vite --port "$PORT" --strictPort > /tmp/riddle-vite.log 2>&1 &
  for _ in $(seq 1 80); do sleep 0.25; up && break; done
  up || { echo "dev.sh: vite did not come up on $PORT; see /tmp/riddle-vite.log" >&2; exit 1; }
fi
echo "$URL"
