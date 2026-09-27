# AGENTS.md — operating manual for Riddle

**Resuming? Read `docs/HANDOFF.md` first.** Read `PLAN.md` (canonical design) and `docs/CUT29.md` (the current implementation contract),
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
tools/verify.sh --quick        # tests (fast profile) ∥ tsc + copy-lint            ~45 s
tools/verify.sh                # + clippy → wasm (fast) → web build → quick gates   ~5 min (the quick gate ~4 min fresh)
tools/verify.sh --full         # + shipping wasm → full gate table                  ~11 min
cargo test -q --workspace --profile fast                     # ~36 s warm (CPU-bound on the cores); never plain `cargo test` (7× slower)
node tools/gates.mjs [--full] [--fresh]                      # quick: 8 seeds × 8 h × 3 verdicts + dayplayer + the wire invariants (examples/qa.rs) alongside (~4 min); full: 30 × 8 × 8 (~8 min on a shared box); each leg cached by its own binary hash — a client-only change reprints in 0.2 s, a new qa invariant reruns qa alone
(cd web && pnpm -s test)                                     # the client gates, headless, on the suite's own no-HMR Vite server (~5 min); `node tests/run.mjs fights clarity:paint` for a few
tools/wasm.sh [--ship]                                       # fast wasm (~8 s after a core edit) / wasm-pack release (~2 min)
tools/ship.sh [--preview]                                    # cohort build on :5230 (fat LTO, ~2 min); --preview: fast wasm, ~25 s, for QA rounds
cargo run -q --profile fast -p riddle-core --example cli -- --seed 1 --rules crates/riddle-core/presets/good.json --runs 3
cargo run -q --profile fast -p riddle-core --example timing -- 1 8   # where a gate job spends its time
tools/dev.sh                                                 # ensure the Vite dev server on :5219 (never restart a running one)
node tools/playtest.mjs [--seed N] [--absent 8h] [--out dir] # scripted walk of every screen, text + screenshots, headed on the GPU (~30 s; --headless ~65 s)
node tools/driver.mjs --dir scratchpad/<who> --port 5347 [--headed] &   # one browser context across an agent session; tools/drive.sh 5347 '{"op":"text"}'
node tools/browser.mjs --probe                               # must print D3D12 (NVIDIA …); --headless prints SwiftShader
python3 art/pack.py && python3 art/art-qc.py
eval/score.sh eval/cards/<card>.json
```

Dev URL params (dev build only): `?seed=N&fresh=1`, `?absent=8h`, `?rules=<text>`, `?speed=4`,
`?autosend=1`, `?engine=fake`. Game at `http://localhost:5219/`.

Cost facts: a death verdict is ~0.1 s native single-threaded (candidates × 12 reseeded replays;
~0.02 s across the cores, `forecast::par_map`), ~1 s in wasm; the gate table samples verdicts
(3–8 per seed) for that reason. `runOfflineQuick` skips it per slice; the client calls `death(id)`
once at the end of an absence. Forecast panels and verdict replays run on all cores natively
(`forecast::parallel_sims`; `metrics.rs` turns it off because it fills the machine by seed);
results are bit-identical to the sequential order. 8 h offline ≈ 0.4 s of ticks (3 µs/tick, a
third of it the chores' floods).

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
- `tools/ship.sh` takes ~2 min (fat-LTO wasm + wasm-opt; `--preview` ~25 s). Never pipe it
  (`ship.sh | tail`): the detached preview holds the pipe and the call never returns — redirect
  to a file and poll `curl localhost:5230` (+ a marker string in the served JS). Never wait on
  it with `pgrep -f tools/ship.sh`: the waiter's own command line matches.
- `docs/ITERATION_SPEED.md` ranks what to speed up next (mechanical QA, compound driver ops,
  the history ring, a table cache).
- A stale Vite dev server from a previous day can serve stale transforms; `tools/dev.sh`
  reuses whatever is on the port, so kill it by pid when a walk shows old UI.

## Browser harness (hybrid: headless for text, headed GPU for render)

`tools/browser.mjs` has two paths. `launchBrowser()` is headless Chromium (SwiftShader WebGL):
the client gates (`pnpm test`, seven at once), QA sessions and anything that reads text and
clicks — fast to start, parallel-safe, no desktop window, audio muted. `launchGpu()` /
`launchBrowser({ gpu: true })` is headed Chromium under WSLg with Mesa's D3D12 driver forced via
`/usr/lib/wsl/lib`, which reaches the real GPU (`--probe` prints `D3D12 (NVIDIA …)` and `native
scale 1.5`): frame times, render QA, blind raters (feel is rated) and the playtest walk, whose
watch pump is per-frame — headless is pixel-bound (14 fps at 3×, 32 at 2×; the walk takes 65 s
headless vs 26 s headed), so headless sessions render at 2×. `RIDDLE_BROWSER=headed|headless`
overrides callers that pass nothing. Headless cannot reach the GPU whatever the flags (probed:
`--headless=new` + the Mesa env still reports SwiftShader/llvmpipe). WSLg's GDK_SCALE=2 ×
Xft.dpi=144 made headed Chromium think the 4K screen was 1280×720 at 3×; the launch pins
GDK_SCALE=1 and forces the Windows scale (`Xft.dpi/96`, override with `WSL_SCALE=`), hides the
15 px classic scrollbar that made full-page shots 385 CSS px wide, and mutes audio (`--mute-audio`;
WebAudio still schedules, `window.__audio` still sees cues). The Playwright MCP tools are
headless/SwiftShader; do not use them for performance claims.
