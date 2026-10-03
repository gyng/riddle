# Riddle

An idle roguelike whose hero you never drive. Send a warrior into the dungeon, collect the haul,
and hire workers to keep the town and expeditions running while you are away. Choose stances and
tactics as they unlock; write rules later to fine-tune a build. Watch live runs or replay the log.
The dungeon has a bottom.

[Play the alpha](https://gyng.github.io/riddle/). The banner shows the build date.

| Doc | What |
|---|---|
| `PLAN.md` | Canonical plan: pillars, rule layer, roguelike, idle model, metagame, presentation, tech, gates, kill-tests, milestones |
| `docs/FUN_EVAL_IDLE.md` | Fun evaluation for idle games, specialising `../eval-fun` (two idle axes, idle boundaries, kill criteria, absence-aware protocol, proxies) |
| `eval/` | Presets, scorecard template, `score.sh`, copy budgets, calibration cards |
| `research/idle-attraction.md` | What makes idle games attractive and why they fail; 12 design rules |
| `research/comparables.md` | Auto-hero games and behaviour-programming systems; steal/avoid per game |
| `research/roguelike-and-spectating.md` | Roguelike story generation without a human; watching and raising an AI |
| `research/art-tech.md` | Pixel-art environment in three.js with watercolour sprites; recommended pipeline |

Score a card: `eval/score.sh eval/cards/<card>.json [--profile=achievement]` (needs `../eval-fun`).

Status: **Alpha · Cut 30.5**. `tools/verify.sh` checks the engine and web build; `node tools/gates.mjs --full` checks progression.

```sh
tools/verify.sh --quick            # engine tests, TypeScript, copy checks
tools/wasm.sh                     # build the engine for local play
tools/dev.sh                     # http://localhost:5219/  (?engine=fake for UI without wasm)
node tools/browser.mjs --probe     # GPU harness under WSLg (see AGENTS.md)
```

Pushes to `main` build the release engine and deploy through GitHub Actions to Pages.
For a local Pages build: `RIDDLE_BASE=/riddle/ pnpm --dir web build`.

Current contract: [Cut 30.5](docs/CUT30_5.md); resume notes: [HANDOFF](docs/HANDOFF.md).
