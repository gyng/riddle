#!/usr/bin/env sh
# Score an idle-game card with eval-fun's scorer and this repo's idle presets.
# usage: eval/score.sh eval/cards/<card>.json [--profile=id] [--json]
set -e
here=$(cd "$(dirname "$0")" && pwd)
evalfun="$here/../../eval-fun"
[ -d "$evalfun" ] || { echo "expected ../eval-fun next to this repo" >&2; exit 2; }
card=$1; shift
exec node "$evalfun/tools/fun-score.mjs" "$card" "$@"
