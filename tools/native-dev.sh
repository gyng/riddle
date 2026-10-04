#!/usr/bin/env bash
# Real native engine on a separate dev port; a core rebuild is enough, no WASM packaging.
set -euo pipefail
cd "$(dirname "$0")/.."
task_native_bin=$(node tools/native-build.mjs)
task_native_port=${RIDDLE_NATIVE_PORT:-5367}
native_up() {
  curl --noproxy "*" --max-time 1 -sf "http://localhost:$task_native_port/__native/health" | RIDDLE_NATIVE_EXPECT_BIN="$task_native_bin" node -e 'let s="";process.stdin.on("data",b=>s+=b).on("end",()=>{try{const h=JSON.parse(s);if(h.native!==true)process.exit(1);const p=process.env.RIDDLE_BALANCE_FILE?require("node:path").resolve(process.env.RIDDLE_BALANCE_FILE):null;const watch=process.env.RIDDLE_NATIVE_WATCH!=="0";process.exit(h.binary===process.env.RIDDLE_NATIVE_EXPECT_BIN&&h.balanceFile===p&&h.threads===Number(process.env.RIDDLE_NATIVE_THREADS??8)&&Boolean(h.build)===watch?0:2)}catch{process.exit(1)}})'
}
if native_up; then
  echo "http://localhost:$task_native_port/?engine=native"
  exit 0
else
  task_native_status=$?
  if [ "$task_native_status" -eq 2 ]; then echo "native server has another configuration; choose RIDDLE_NATIVE_PORT" >&2; exit 2; fi
fi
RIDDLE_NATIVE_BIN="$task_native_bin" RIDDLE_NATIVE_DEV=1 setsid nohup pnpm --dir web exec vite --port "$task_native_port" --strictPort > /tmp/riddle-native-vite.log 2>&1 &
task_native_launcher_pid=$!
for _ in $(seq 1 80); do
  if native_up; then echo "http://localhost:$task_native_port/?engine=native"; exit 0; fi
  sleep .25
  if ! kill -0 "$task_native_launcher_pid" 2>/dev/null; then break; fi
done
echo 'native dev server failed; see /tmp/riddle-native-vite.log' >&2
exit 1
