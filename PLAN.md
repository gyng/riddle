# Riddle — PLAN.md

*Working title. Canonical plan, 2026-09-15. Research: `research/*.md`. Fun eval:
`docs/FUN_EVAL_IDLE.md` + `eval/`. Sibling conventions: `../tacticalswap/AGENTS.md`
(cut contracts, gates, WebMCP), `../tacticalswap/docs/ART.md` (watercolour register),
`../cyty/PLAN.md` (Rust sim → WASM, custom renderer).*

## One-liner

A real roguelike whose hero you never drive. You write its brain as an ordered rule list,
send it down, and it comes back, or doesn't, with loot, a story, and a death that names
the rule you got wrong. Runs happen while you are away. The dungeon has a bottom.

**Contract on the store page:** *you write the hero's rules; you never steer.* Present as an
automation/idle game first, never as an action roguelike (Pecorella's expectation-management
warning, `research/idle-attraction.md` §1.1).

## Why this should work where Shape Gacha did not

Shape Gacha had every retention hallmark (two currencies, offline catch-up, prestige,
multipliers) and no engine: tension 0, decisions were a maths puzzle with nothing at stake,
auto-arrange made the one interesting layer optional, nothing produced an anecdote. The v4
rule "progression never rescues a weak core" held.

Riddle inverts each of those:

| Shape Gacha | Riddle |
|---|---|
| Calm collector spine, optimisation optional | Rule authoring **is** the spine; there is no auto-arrange |
| Growth measured in Flux | Growth measured in depth, bosses, run length; numbers are the scoreboard |
| Nothing at stake | Permadeath as stakes: three-tier exit, loot at risk, the rule set's reputation |
| No story | A roguelike built for interaction density plus a chronicle and a highlight sifter |
| Prestige = flat ×1.6 | Every meta step adds a verb, condition, slot or zone |

The thesis is a decade old and validated: Loop Hero (93%, 500k in a week), Stone Story RPG
(91%), Auto Rogue (91%). Auto Rogue is the near-exact precedent and died in 1–5 hours on
"only one thing to unlock, no endless, no mid-run save". The design risk is execution and
content cadence, not concept (`research/comparables.md`).

## Pillars

1. **Every death names a rule.** The cause is a row the player wrote or failed to write,
   never "the AI". Sid Meier: players blame the game for losses 3:1; the trace defuses it.
2. **Automate no-brainers silently; surface trade-offs.** DCSS's rule is the split between
   engine and rule layer. Rest, pathing, pickup, auto-equip by preference: engine. Retreat
   thresholds, consumable timing, escape, dive-vs-explore, targeting, ally stance, ID
   policy, terrain tactics: the player's eight families.
3. **The return is the hero screen.** Every absence produces something to read and something
   to decide. Offline runs are uncapped; the wall caps yield, never the timer.
4. **Unlocks add vocabulary, not multipliers.** A new condition, verb, slot or zone roughly
   every play-day for two weeks, each changing what a good rule set looks like.
5. **The dungeon generates stories.** ~30 monsters and ~40 items chosen for interaction
   density, hazards that hit both sides, dual-use consumables, allies with state, a forward
   clock. No flavour-only lists, no hunger bookkeeping, no shops, no XP grind.
6. **Copy is near zero.** The rule rows are the sentences. Callouts are ≤ 3 words. The
   death screen is a cause line, a rule line, one verdict word, one button. No tutorial
   text; the first death teaches. Numbers and the trace do the telling.

## Anti-pillars

- No calm mode, no zero-skill spine, no auto-arrange, no "the maths is optional".
- No presence rewards: nothing for clicking, nothing for watching. Active play is rewarded
  only through decisions, in the 1.5–3× band over idle.
- No timers as walls, no offline cap, no decay for absence, no monetised waits. Free or
  buy-once.
- No personalities that drift silently (Dragon's Dogma 1). Traits deviate loudly or not at all.
- No content the hero grinds silently: if the watcher sees nothing, cut it.
- No infinite treadmill. The dungeon has a bottom; reaching it is an ending that says "you
  can stop"; ascension variants are opt-in.

## The loop at three scales

**Watching (optional, seconds to minutes).** Camera follows the hero through the floor.
Callouts name the firing rule (`HP 31% → Potion (R2)`), monsters telegraph intent one turn
ahead, near-misses are visible. Speed: 1×, 4×, skip-to-event, key-highlights. Watching is
never required; it is how the player learns which rule to write next.

**Check-in (2–10 minutes, 1–5 times a day).** Read the return report: runs, exits by tier,
deaths by cause, best depth, drops, new bestiary entries, unlocks affordable. Open the worst
death: last five rule firings, telegraphs that preceded them, verdict (`gap` = a rule would
have saved it, `dice` = nothing in the vocabulary could), one button: patch. Adjust loadout
from the vault. Spend unlocks. Send. Leave on a chosen stopping point: nothing left to decide.

**Day to day (two weeks).** Unlock cadence turns the rule set over. Wall bosses every five
floors cannot be passed until the brain is rewritten; the counter is discoverable in-game
(the boss telegraphs its trait; the bestiary records it) and a parallel lane (side biome,
trophies) progresses while stuck.

**Weeks to months.** Reach the bottom (target depth 25–30). Ending. Ascension variants
(per-run constraints à la Idle Champions/Astronarch) each add a system and are chosen, rare,
and reward new bests, never repeated farming.

## The rule layer (the game)

**Row shape.** `[scope] [≤ 2 conditions] → [verb]`, ordered, first-true fires each turn,
fall-through when the verb cannot execute, hidden sanity rules (no recasting an active buff,
no drinking at full HP). Two conditions per row is the demonstrated mass-appeal ceiling
(Unicorn Overlord, DAO); unlimited trees (PoE2, Gladiabots) are a minority taste.

**Slot budget.** Start at 4 rows. Grow to ~12 via unlocks. The budget is the difficulty knob
(Auto Rogue's "strict limitations on the size of your program" is the praised part). Slots
never compete with combat power.

**The eight families** (from `research/roguelike-and-spectating.md` §A3), each with a
default row shipped and a wider vocabulary unlocked:

| Family | Example row | Why it is a trade-off |
|---|---|---|
| Retreat | `HP < 40% · foes ≥ 2 → back to corridor` | Safety vs tempo vs loot |
| Consumables | `HP < 25% · potion known → drink` | The potion might be the last one |
| Escape | `HP < 20% · no path to stairs → read unknown scroll` | Gamble vs certain death |
| Dive / explore | `floor cleared ≥ 60% → descend` | Under-levelled dives are tension |
| Targeting | `foe: caster → attack first` | Threat vs proximity |
| Ally stance | `captive found → free` | Allies can die, betray, level |
| ID policy | `no foes in view · HP full → try unknown` | Cautious vs reckless |
| Terrain | `foe: bloat · adjacent → step back` | The clever play the watcher wants |

Plus **event triggers** (`on hurt`, `on kill`, `on see`) alongside state thresholds; event
verbs are the more discoverable half (Super Auto Pets, Path of Achra). **Verbs include
movement**: retreat, kite, pick up, descend, step back, throw. DAO's "cannot make a
character flee from Inferno" is the failure to avoid.

**Conditions come from the bestiary.** `foe: undead` exists only after the hero has met
undead. Knowledge is the metagame currency (Idle Loops, Dragon's Dogma bestiary stars).

**Forecast before send.** N deterministic sims of the current rule set on unseen seeds:
`reaches D6 in 71% · dies to: ghoul pack 18%, gas 9%`. Converts "the AI failed" into "my
plan was wrong" (Melvor's safety inequality, Auto Rogue's forecast, Idle Loops' top
request).

**Replay and trace.** Every run is a seed plus the rule set; scrubbable; the firing row is
highlighted each turn. This is the legibility kill-test: if watching the replay does not
make a tester say "my rule was wrong", nothing else matters.

**Presets and sharing.** Three class presets to copy from. Three saved sets per heir.
Copy-as-text export from day one (communities trade gambit sets: FF12, DAO, Stonescript).

**Traits.** An heir carries 1–2 traits (greedy, cowardly, curious). A trait is an announced
deviation (`Greedy: ignores R3, takes the gold`), never a hidden weight. Partial obedience
reads as alive only when it is attributable (Creatures, Monster Rancher).

## The roguelike (the story generator)

- **Turn-based grid**, every state inspectable, every consequence attributable. The display
  runs turns at the chosen speed; there is no real-time input.
- **Floors** of rooms and caves, 24 tiles wide max (portrait readability), biomes every
  five floors with **traits that break last biome's program**: reflects magic, steals gear,
  only hittable at range, spawns on death.
- **~30 monsters** for interaction density: bloat (pops into gas), jelly (splits), thief
  (steals and flees), summoner, shield-buffer, archer (telegraphs), captive (freeable ally),
  named grudge monsters carried across runs ("the goblin that killed heir 3").
- **~40 items**, all dual-use: throwable potions, read-vs-keep scrolls, partial ID
  (benevolent/malevolent), a few escape items that are scarce enough that thresholds matter.
- **Hazards that hit both sides**: gas, fire, deep water (sweeps items), chasms.
- **Forward clock**: alert level rises with turns on a floor; drives dive-vs-explore.
- **Allies with state**: level, die, can be abandoned by rule.
- **Vault "choose one of N"** and altar commitments as the only direct interrupts, and only
  when the player is watching. This is the presence reward: a decision, not a click.
- **Three-tier exit**: reach the camp stairs = keep 100% of run loot; retreat rule fires
  mid-floor = 60%; death = 30%. Meta (rule sets, bestiary, unlocks, vault) is never lost.
  Retreat is itself a rule (`loot ≥ 3 rares → return`), so idle runs bank.
- **Death screen**: cause, margin, last five firings, telegraphs, `gap`/`dice` verdict,
  `patch`. Also a morgue-style text export with seed and rule set for retellings.
- **Chronicle and sifter**: DCSS-style auto-notes during the run; a Felt-style sifter picks
  comeback, first-kill, near-death, ally-lost, item-gamble into a highlight reel per run and
  a lineage history across runs. The hero has a voice (one-line diary), not a log.

## Equipment and skills

- **Class** chosen per heir; class = the verb set (fighter has `shield bash`, rogue has
  `vanish`). Three at launch, more unlocked.
- **Gear** is found per run and auto-equipped in-run by a preference rule (`prefer resist
  over damage · depth > 10`). At camp the player chooses a **loadout from the vault**: one
  kept item at first (Dead Cells blueprints), more slots unlocked. Gear and rule slots do
  not share a budget.
- **Skills** are verbs; new skills are new rows the player can write, so every skill unlock
  changes the rule set.

## Idle model

- **Unit = expedition.** A run takes a real-time duration at 1× (target 3–8 minutes to a
  natural exit; camp rest is instant). Offline, the sim runs `elapsed / run_duration`
  expeditions deterministically, uncapped. Yield is bounded by the rule set's wall, not by a
  timer; Pecorella's math (exponential difficulty, linear offline) says the cap is unneeded.
- **Return report** is the most important screen in the game. Delta, not totals. At least
  one pending decision after any absence ≥ 20 minutes (a death to read, an unlock
  affordable, a new bestiary entry that unlocks a condition).
- **Active bonus** through decisions only: a patched rule set is worth 1.5–3× an unpatched
  one over the next absence. Presence is worth zero.
- **Cadence targets**: a rule set stays productive 8–24 h before it stalls; something small
  to decide every 1–4 h; a new unlock roughly daily for two weeks (`research/idle-attraction.md`
  §6).
- **Bumpy curve**: place power spikes (new verb, new slot, a breakpoint) so progress
  alternates fast and slow.

## Metagame

- **Lineage**: heirs with traits and a graveyard listing cause and deeds. Bestiary knowledge,
  rule sets, unlocked vocabulary and the vault persist.
- **Unlocks** spend a single meta currency earned by new bests (depth, boss, trophy), never by
  repeated farming. Every unlock is a condition, verb, slot, class, zone, vault slot, or an
  automation (something the player no longer has to set). Toggleable so the pool never
  bloats (Loop Hero praised, Brotato blamed).
- **Trophy ledger** rewarding different rule sets (clear D10 with no healing rows; with only
  ranged verbs). Doubles as a build-exploration engine and a to-do list.
- **Ending** at the bottom. Then ascension variants, each adding a system.

## Presentation

From `research/art-tech.md` "Recommended pipeline":

- **Environment**: lo-fi pixel art, env texel = 4 device px (integer scale
  `k = floor(min(w,h)/270)`), 8×8 tiles for floor/wall/dither, 16×16 for props, ≤ 8-colour
  per-biome palette applied in the blit shader with world-texel-indexed Bayer dither; biome
  = palette uniform swap. Internal target 270×585 portrait, 480×270 desktop, no letterbox.
- **Camera**: three.js `OrthographicCamera`, 30° dimetric if walls have tops, else top-down;
  snapped to the env-texel grid with the sub-texel remainder as a blit UV offset (t3ssel8r);
  critically-damped spring follow on the hero.
- **Hero and monsters**: codex-generated watercolour-anime sprites in the tacticalswap ART.md
  keyed register (bold ink contour, upper-left key light, `#0000FF` chroma key, pack + QC
  scripts reused). Composited at **two densities in one target** (sprite texel = 1 target
  px, env texel = 2): hero 48 sprite-texels = 3 tiles tall, outline dilated to ≥ 1 env
  texel, 40–60% palette tint, 1-texel contact shadow, positions snapped, 8–12 fps animation.
  Framed as paper cut-outs on a pixel diorama, deliberately.
- **Lighting**: unlit + baked vertex darkness, ≤ 2 point lights, additive torch quads,
  banded fog from the hero.
- **Audio**: sparse; a callout tick, telegraph sting, death chord. Later.
- **UI**: rule editor is the main screen. Rows are chips: `[scope] [cond] [cond] → [verb]`,
  drag to reorder, tap to swap a token from the unlocked vocabulary. Phone-first, one thumb.
- **Copy budget**: callout ≤ 3 words; death screen ≤ 12 words plus the trace; return report
  is numbers and nouns; no sentence anywhere in chrome. Enforce with a copy-lint like
  tacticalswap's (`tools/copy-lint.mjs`, budgets in `eval/copy-budgets.json`).

## Tech

- `crates/riddle-core`: deterministic sim (dungeon gen, monsters, items, rule engine, trace,
  chronicle, sifter, forecast, offline batch, probes). All game truth. No wall clock.
- `crates/riddle-wasm`: thin wasm-bindgen bridge (`new`, `set_rules`, `forecast`, `step`,
  `run_offline(elapsed)`, `snapshot`, `trace`, `export`).
- `web/`: Vite + TS + three.js. Custom pixel pipeline, no post-processing stack, one blit.
  Renderer keeps primitive fallbacks per sprite id; missing art never blocks.
- `art/`: manifest + codex generation + `pack.py` + `art-qc.py`, lifted from tacticalswap.
- `tools/`: `verify.sh` (tests → clippy → wasm → tsc → build → copy-lint), `gates.mjs`,
  `probes` (the eval bot proxies), WebMCP inspector for agent playtests.
- PWA, offline-first save with export/import (Melvor's server-outage reviews). Versioned
  precache; verify on a fresh port (paynyaa lesson).
- Same-seed replay hash tests from cut 1.

## Bots and gates (Rust `examples/metrics.rs`, 30 seeds each)

Inviolable, never weakened to pass:

| Bot | Rule set | Must |
|---|---|---|
| DEFAULT | shipped preset, untouched | die by depth ≤ 6 on ≥ 80% of seeds |
| EDITED | preset + 1 h of an agent's edits | reach depth ≥ 10 on ≥ 50%; win-rate gap vs DEFAULT ≥ 15 pts |
| RANDOM | random rows | lose every seed |
| PASSIVE | no rows (engine only) | lose every seed by depth ≤ 3 |

Proxies exported for the eval (`docs/FUN_EVAL_IDLE.md` §5): unfair-death share ≤ 5%;
death-cause entropy top cause < 35%; share of deaths whose trace ends in a player row ≥ 70%;
events per minute at 1× ≥ 6, no dead stretch > 20 s; offline ÷ active yield 0.25–0.5;
decisions pending after any absence ≥ 20 min ≥ 1; rule-set diversity across clears ≥ 0.5.

## Kill-tests (P0, build first, throwaway by policy)

**K1 — Legibility.** Text-only dungeon (Rust, CLI + minimal web table), 4-row editor with
the retreat/consumable/targeting/dive families, forecast, replay trace with the firing row
highlighted, death screen with `gap`/`dice`. No art, no meta, no offline. Two agents and
the user play 30 minutes each. Pass: after a `gap` death, a tester patches a rule within 60
seconds without prompting on ≥ 70% of deaths; EDITED beats DEFAULT by ≥ 15 pts; a tester's
retelling of one run uses first-person pronouns. Fail → the row grammar or the trace is
wrong; fix before anything else.

**K2 — The look.** One dungeon room in three.js with the pixel pipeline, one codex
watercolour hero, options A (everything low-res) and C (two densities) as a toggle. Phone
screenshots. Pass: five people read C as "one game"; the 8×8 tiles read at phone size; the
hero's ink line survives. Fail → invest in the downscale pipeline and a 48-texel pixel hero.

**K3 — The return.** Offline batch over a simulated 8 h with the K1 sim; generate the
return report. Pass: the report contains ≥ 1 pending decision and a death worth opening on
every seed, and the yield is visibly bounded by the wall not the clock.

All three inside the first cut. No metagame until K1 passes.

## Milestones

| M | Deliverable | Gate |
|---|---|---|
| 0 | Scaffold: workspace, wasm bridge, Vite, verify.sh, replay-hash test | `tools/verify.sh` green |
| P0 | K1, K2, K3 | pass conditions above |
| 1 | Sim core: floors, 12 monsters, 20 items, hazards, clock, rule engine with 8 families, trace, chronicle, three-tier exit, forecast | bot gates green |
| 2 | Renderer: pixel pipeline, snapped camera, tiles, sprites at two densities, callouts, telegraphs, speed controls | K2 look at 60 fps on a mid phone; events/min ≥ 6 |
| 3 | Check-in loop: return report, death screen, rule editor UI (phone), vault loadout, offline batch, save/export | return proxies green; check-in median 2–10 min in dogfood |
| 4 | Metagame: lineage, traits, bestiary-gated vocabulary, unlock tree, trophies, presets, share-as-text | unlock cadence ≈ daily over a 14-day simulated play log |
| 5 | Content to the bottom: 30 monsters, 40 items, 5 biomes with program-breaking traits, 5 wall bosses, ending | death-cause entropy green; every boss has an in-game discoverable counter |
| 6 | Story layer: sifter, highlight reel, lineage history, hero voice, named grudge monsters, audio | tell-a-friend test passes on 3 of 5 runs |
| 7 | Release hardening: PWA, copy-lint clean, WebMCP harness, Tier-2 eval on the idle-roguelike preset | ≥ 72 with no gate on `return`/`attribution`; second blind card α ≥ 0.67 |
| 8+ | Ascension variants, more classes, live cadence (small drops every few days) | new bests only, never farm |

Work is done as cut contracts (`docs/CUT<n>.md`) with numeric gates, core/client/art tracks
in parallel meeting at the wire types, then a fun-verdict playtest whose gaps become the
next contract (tacticalswap method).

## Risks

- **"The game plays itself" backlash.** Mitigation: the contract on the store page, the
  DEFAULT-must-die gate, decision density at every check-in.
- **Rule editor on a phone.** Token chips and drag; K1 on desktop first, then a phone pass
  before M3 ships.
- **Style clash** between watercolour sprites and 8×8 tiles: no shipped game does it. K2
  decides; option A is the fallback.
- **Content faucet slower than number faucet** (Auto Rogue, Nomad, Loop Hero's grind).
  Data-driven monsters/items/traits from M1; unlock cadence gate in M4; live cadence after.
- **Why watch if runs happen offline?** Watching is how you learn the next rule; the
  highlight reel replaces watching for those who will not. If events/min stays < 6 at 1×,
  cut floors, not the watch.
- **Forecast makes runs predictable.** Forecast is a distribution over unseen seeds, and
  biome traits invalidate it every five floors.
- **Long-loop editing brittleness** (Stuck In Time). Rows are independent; forecast is
  instant; no loop to wait out.

## Open questions

- Ending depth and total arc length (proposal: depth 30, ~3 weeks of daily check-ins).
- Whether wall bosses are `dice`-proof by construction (no unavoidable deaths in boss rooms).
- Whether to show the dungeon top-down (Downwell-flat, cheaper) or 30° with wall tops.
- Name.

## Immediate next task

Cut 1 = M0 + K1: the text-only sim with a 4-row editor, forecast and trace, plus the bot
gate harness. Write `docs/CUT1.md` as the contract, then build core and CLI in parallel.
