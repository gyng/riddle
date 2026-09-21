# AGENTS.md — operating manual for Riddle

Read `PLAN.md` (canonical design) and `docs/CUT1.md` (the current implementation contract),
then this file. `CLAUDE.md` is a bare import of this file.

## What the game is

A real roguelike whose hero you never drive. The player writes an ordered rule list (≤ 8 rows
of `conds → verb`); the hero learns the dungeon's facts on its own; the engine does chores
silently. Runs happen offline, uncapped. Every death names a rule. The dungeon has a bottom.

```
camp (edit rules, loadout, unlocks, forecast) → send → run (rules fire, facts learned)
   → exit: bank 100% · return 60% · death 30% → death screen (trace, gap/dice, patches)
   → report (learned · bests · found · deaths · pending · reel) → camp
```

## Repo map

```
crates/riddle-core/    all game truth; examples/metrics.rs = the gate table; examples/cli.rs = text playtest
crates/riddle-wasm/    thin JSON bridge (wasm-bindgen camelCases: run_offline → runOffline)
web/src/engine/        types.ts (wire), fake.ts (UI dev), wasm.ts (real), pkg/ (built)
web/src/ui/            camp, watch, death, report, settings
web/src/render/        three.js replay viewer (pixel pipeline); render-demo.html
art/                   ART.md, manifest.json, prompts/, generated/, tiles/, pack.py
tools/                 verify.sh, gates.mjs, copy-lint.mjs, art-qc.py
eval/                  idle fun-eval presets, template, calibration, copy budgets
docs/                  FUN_EVAL_IDLE.md, CUT*.md
research/              the four research reports behind the plan
```

## Commands (iteration tiers — use the cheapest that answers the question)

```sh
tools/verify.sh --quick        # tests (fast profile) → tsc → copy-lint            ~10 s
tools/verify.sh                # + clippy → wasm (fast) → web build → quick gates   ~1.5 min
tools/verify.sh --full         # + shipping wasm → full gate table                  ~4 min
cargo test -q --workspace --profile fast                     # 5 s incremental; never plain `cargo test` (7× slower)
node tools/gates.mjs [--full]                                # quick: 8 seeds × 8 h × 3 verdicts (~50 s); full: 30 × 8 × 8
tools/wasm.sh [--ship]                                       # fast wasm (~15 s incremental) / wasm-pack release (~40 s)
cargo run -q --profile fast -p riddle-core --example cli -- --seed 1 --rules crates/riddle-core/presets/good.json --runs 3
cargo run -q --profile fast -p riddle-core --example timing -- 1 8   # where a gate job spends its time
tools/dev.sh                                                 # ensure the Vite dev server on :5219 (never restart a running one)
node tools/playtest.mjs [--seed N] [--absent 8h] [--out dir] # scripted walk of every screen, text + screenshots, on the GPU harness
node tools/browser.mjs --probe                               # must print D3D12 (NVIDIA …)
python3 art/pack.py && python3 art/art-qc.py
eval/score.sh eval/cards/<card>.json
```

Dev URL params (dev build only): `?seed=N&fresh=1`, `?absent=8h`, `?rules=<text>`, `?speed=4`,
`?autosend=1`, `?engine=fake`. Game at `http://localhost:5219/`.

Cost facts: a death verdict is ~1.2 s native (candidates × 20 reseeded replays); the gate table
samples verdicts (3–8 per seed) for that reason. `runOfflineQuick` skips it per slice; the client
calls `death(id)` once at the end of an absence.

## How work gets done

1. Write a contract (`docs/CUT<n>.md`) with numeric gates.
2. Build core / client / renderer / art in parallel; they meet at the wire types.
3. Gate it. Never weaken a gate to pass it; tune content, or record a deviation with a reason.
4. Fun-verdict playtest on the `idle-roguelike` preset (`docs/FUN_EVAL_IDLE.md`); the gaps
   become the next contract.

## Hard invariants

- All game truth in Rust. TS renders and edits only.
- Determinism: same seed + rules + elapsed ⇒ identical events. Replay-hash test exists.
- Inviolable bots: DEFAULT dies by D6; EDITED beats it by ≥ 15 pts; RANDOM and PASSIVE lose
  every seed; LEARNED (facts only) gains ≤ 2 floors. These keep policy load-bearing.
- Every death has a verdict and a trace; `dice` deaths ≤ 5%.
- Facts are learned; policy is written. Nothing learns policy implicitly.
- Art never blocks the game: every sprite id has a primitive fallback.
- Copy: callout ≤ 3 words, verdict 1 word, no sentences in chrome (`eval/copy-budgets.json`,
  `tools/copy-lint.mjs`). No tutorial text.
- Offline is uncapped; nothing punishes absence.

## Gotchas

- `/home/g` is itself a git repo. Never stage from the home toplevel.
- Codex writes asynchronously; poll `art/generated/` mtimes to quiescence; run batches
  detached (`setsid nohup`).
- After editing Rust, rebuild `web/src/engine/pkg`; the PWA precache is versioned, so verify
  frontend changes on a fresh port.
- `tools/ship.sh` takes ~6 min (fat-LTO wasm); run it in the foreground with a long timeout
  or in the background and poll `curl localhost:5230` + `web/dist` mtimes. Never wait on it
  with `pgrep -f tools/ship.sh`: the waiter's own command line matches.
- A stale Vite dev server from a previous day can serve stale transforms; `tools/dev.sh`
  reuses whatever is on the port, so kill it by pid when a walk shows old UI.

## Browser harness (GPU under WSLg)

Headless Chromium here runs on SwiftShader. Use `tools/browser.mjs` (headed Chromium under
WSLg with Mesa's D3D12 driver forced via `/usr/lib/wsl/lib`), which reaches the real GPU:
`node tools/browser.mjs --probe` should print `D3D12 (NVIDIA …)` and `native scale 1.5`. WSLg's
GDK_SCALE=2 × Xft.dpi=144 made Chromium think the 4K screen was 1280×720 at 3×; `launchGpu()` pins
GDK_SCALE=1 and forces the Windows scale (`Xft.dpi/96`, override with `WSL_SCALE=`), and hides the
15 px classic scrollbar that made full-page shots 385 CSS px wide. Every render playtest,
frame-time measurement and screenshot goes through `launchGpu()` from that file. The
Playwright MCP tools are headless/SwiftShader; do not use them for performance claims.
