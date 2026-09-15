# Riddle

A real roguelike whose hero you never drive. You write its brain as an ordered rule list, send it
down, and it comes back with loot, a story, and a death that names the rule you got wrong. Runs
happen while you are away. The dungeon has a bottom.

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

Status: **Cut 1 built and playable** (2026-09-16). `tools/verify.sh` is the gate.

```sh
tools/verify.sh                    # tests → clippy → wasm → build → copy-lint → 17 bot gates
cd web && pnpm dev --port 5219     # http://localhost:5219/  (?engine=fake for UI without wasm)
node tools/browser.mjs --probe     # GPU harness under WSLg (see AGENTS.md)
```

Next: Cut 2 on the two gated axes, tension and expression (see PLAN.md → Milestones).
