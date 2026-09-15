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

## Commands

```sh
tools/verify.sh [--quick]                       # tests → clippy → wasm → tsc → build → copy-lint → gates
cargo test --workspace
cargo run -p riddle-core --release --example metrics          # THE GATE TABLE
cargo run -p riddle-core --example cli -- --seed 1 --rules crates/riddle-core/presets/good.json --runs 3
wasm-pack build crates/riddle-wasm --target web --out-dir ../../web/src/engine/pkg
cd web && pnpm dev -- --port 5177 --strictPort  # ?engine=fake for UI without wasm
python3 art/pack.py && python3 tools/art-qc.py
eval/score.sh eval/cards/<card>.json
```

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

## Browser harness (GPU under WSLg)

Headless Chromium here runs on SwiftShader. Use `tools/browser.mjs` (headed Chromium under
WSLg with Mesa's D3D12 driver forced via `/usr/lib/wsl/lib`), which reaches the real GPU:
`node tools/browser.mjs --probe` should print `D3D12 (NVIDIA …)`. Every render playtest,
frame-time measurement and screenshot goes through `launchGpu()` from that file. The
Playwright MCP tools are headless/SwiftShader; do not use them for performance claims.
