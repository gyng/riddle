# Riddle — PLAN.md

*Working title. Canonical plan, v2 (reviewed 2026-09-15; review log at the end). Research:
`research/*.md`. Fun eval: `docs/FUN_EVAL_IDLE.md` + `eval/`. Sibling conventions:
`../tacticalswap/AGENTS.md` (cut contracts, gates, WebMCP), `../tacticalswap/docs/ART.md`
(watercolour register), `../cyty/PLAN.md` (Rust sim → WASM, custom renderer).*

## One-liner

A real roguelike whose hero you never drive. You write its policy as a short rule list; the
hero learns the dungeon's facts on its own. Send it down and it comes back, or doesn't, with
what it learned, what it found, and a death that names the rule you got wrong. Runs happen
while you are away. The dungeon has a bottom.

**Contract on the store page:** *you write the hero's rules; you never steer.* Present as an
automation/idle game first, never as an action roguelike (`research/idle-attraction.md` §1.1).

## The division of intelligence (the design in one table)

| | Who | What | Where it shows |
|---|---|---|---|
| **Facts** | the hero, autonomously | bestiary (what a bloat does), item identity (blue potion = heal), biome traits, boss counters, map of the descent | learned by encounter, mostly offline; each fact unlocks a condition or verb token |
| **Policy** | the player, explicitly | ordered rule rows: when to retreat, drink, flee, dive, whom to hit, what to risk | the editor; the trace; the death verdict |
| **Chores** | the engine, silently | pathing, rest, pickup, auto-equip by preference, corridor use when told | never shown unless a rule overrides it |

"Smart AI" therefore means: the hero gets *knowledgeable* by itself and gets *wise* only
through the player. Nothing learns implicitly; every learned fact is a named token, every
policy is a row. This is what keeps attribution intact (Dragon's Dogma 1 and Black & White
lost it to silent drift; FF12's rows kept it).

## Why this should work where Shape Gacha did not

Shape Gacha had every retention hallmark and no engine: tension 0, decisions optional via
auto-arrange, nothing produced an anecdote. The v4 rule "progression never rescues a weak
core" held. Riddle inverts each:

| Shape Gacha | Riddle |
|---|---|
| Calm collector spine, optimisation optional | Rule authoring **is** the spine; there is no auto-arrange |
| Growth measured in Flux | Growth measured in facts learned, depth, bosses; numbers are the scoreboard |
| Nothing at stake | Three-tier exit, brought items at risk, the rule set's reputation |
| No story | A roguelike built for interaction density plus a chronicle and highlight sifter |
| Prestige = flat ×1.6 | Every meta step adds a verb, condition, slot or zone |

Precedents: Loop Hero (93%, 500k in a week), Stone Story RPG (91%), Auto Rogue (91%, the
near-exact precedent, died in 1–5 h on "one thing to unlock, no endless, no mid-run save").
The risk is execution and content cadence, not concept (`research/comparables.md`).

## Pillars

1. **Every death names a rule.** The cause is a row the player wrote or failed to write,
   never "the AI".
2. **Automate no-brainers silently; surface trade-offs.** DCSS's rule is the engine/policy
   split.
3. **The return is the hero screen.** Every absence hands over facts learned, the tail of
   the run distribution, and at least one decision. Offline is uncapped; the wall caps yield.
4. **Unlocks add vocabulary, not multipliers.** A new token, slot, class or zone roughly
   every play-day for two weeks, each changing what a good rule set looks like.
5. **The dungeon generates stories.** Interaction density over list size.
6. **Copy is near zero.** Rows are the sentences. Callouts ≤ 3 words. No tutorial text; the
   first death teaches. Budgets in `eval/copy-budgets.json`.

## Anti-pillars

- No calm mode, no auto-arrange, no "the policy is optional".
- No presence rewards. Active play is rewarded only through decisions, 1.5–3× over idle.
- No timers as walls, no offline cap, no absence decay, no monetised waits. Free or buy-once.
- No implicit learning of *policy*. Facts are learned; policy is written.
- No HP-inflation difficulty. Depth scales by traits that break the current program.
- No infinite treadmill. The bottom is an ending that says "you can stop"; ascension is opt-in.

## The loop at three scales

**Check-in (2–10 min, 1–5×/day) — the game.** Return report, delta not totals, in this
order: *learned* (new tokens), *bests* (depth, first kills), *found* (vault candidates),
*deaths by cause*, *pending* (unlocks affordable, rules flagged). Open the worst death: last
five firings, the telegraphs before them, verdict `gap` or `dice`, and up to three candidate
patches each with a forecast delta, plus free editing. Choose the vault loadout (brought
items are lost on death). Spend unlocks. Send. Leave when nothing is left to decide.

**Watching (optional, seconds to minutes).** The renderer is a **replay viewer**: it plays
event streams from Rust. Live view is the replay at the head. Death replays and the sifted
highlight reel are its main consumers; the idle player mostly watches those. Speed 1×, 4×,
skip-to-event, key-highlights. Callouts name the firing rule (`HP 31% → Potion R2`);
monsters telegraph one turn ahead; near-misses are visible.

**Day to day (two weeks).** Unlock cadence turns the rule set over. Wall bosses every five
floors cannot be passed until the policy is rewritten; the counter is a *fact* the hero can
learn (the boss telegraphs it, the bestiary records it), so the wall diagnoses itself. A
parallel lane (side branch, trophies) progresses while stuck.

**Weeks to months.** Reach the bottom (depth 30 proposed). Ending. Ascension variants, each
adding a system, chosen and rare, rewarding new bests only.

## The rule layer

**Row shape.** `[scope] [≤ 2 conditions] → [verb]`, ordered, first-true fires each turn,
fall-through when the verb cannot execute, hidden sanity rules. Two conditions per row is
the mass-appeal ceiling (Unicorn Overlord, DAO).

**Slots and tactic cards.** Rows 4 → 8 by unlock, no further. Late-game density comes from
**tactic cards**: a named bundle occupying one row (`corridor fighting`, `kite archers`,
`stair dance`), unlocked as facts and trophies are earned. Auto Rogue's praised strictness
stays; Stuck In Time's brittle long programs are avoided.

**The eight families** (from `research/roguelike-and-spectating.md` §A3), each with a
shipped default row:

| Family | Example row | Trade-off |
|---|---|---|
| Retreat | `HP < 40% · foes ≥ 2 → back to corridor` | Safety vs tempo vs loot |
| Consumables | `HP < 25% · potion known → drink` | The potion might be the last one |
| Escape | `HP < 20% · no path to stairs → read unknown` | Gamble vs certain death |
| Dive / explore | `floor seen ≥ 60% → descend` | Under-levelled dives are tension |
| Targeting | `foe: caster → attack first` | Threat vs proximity |
| Ally stance | `captive found → free` | Allies can die, betray, level |
| ID policy | `no foes in view · HP full → try unknown` | Cautious vs reckless |
| Terrain | `foe: bloat · adjacent → step back` | The clever play |

Plus **event triggers** (`on hurt`, `on kill`, `on see`) and **movement verbs** (retreat,
kite, step back, pick up, descend, throw).

**Tokens come from facts.** `foe: undead` exists only after the hero has met undead;
`potion: heal` only after it was identified. Knowledge is the idle yield and the unlock
gate at once.

**Forecast, bounded by knowledge.** N deterministic sims on unseen seeds, reported only as
far as the hero has facts: `D1–6: reaches D6 71% · dies to ghoul pack 18%, gas 9% · D7+: ?`.
The frontier is always uncertain, so the return still has a tail worth reading.

**Replay and trace.** Every run is a seed plus a rule set; scrubbable; the firing row is
highlighted per turn.

**Presets, saved sets, sharing.** Three class presets; three saved sets per heir;
copy-as-text export from day one.

**Traits.** An heir carries 1–2 traits (greedy, cowardly, curious): announced deviations
(`Greedy: skips R3, takes the gold`), never hidden weights.

**Allies with rows** (mid-game unlock): a freed captive gets two rows of its own. Party play
is deferred; this is the taste of it.

## The roguelike

- **Turn-based grid**, every consequence attributable. No real-time input.
- **Persistent macro, procedural micro.** Per lineage seed, the *descent* is fixed: biome
  order, wall bosses, named grudge monsters and where they live. Floors regenerate every run.
  Facts about the descent accumulate ("D5–9: the Bloat Warrens"); situations never repeat.
- **Floors** of rooms and caves, ≤ 24 tiles wide (portrait). **Biomes every five floors with
  program-breaking traits**: reflects magic, steals gear, only hittable at range, spawns on
  death, gas-rich.
- **~30 monsters** for interaction density (v1: 15): bloat, jelly, thief, summoner,
  shield-buffer, archer, captive, grudge monsters.
- **~40 items** (v1: 25), dual-use: throwable potions, read-vs-keep scrolls, partial ID
  (benevolent/malevolent), scarce escape items.
- **Hazards that hit both sides**: gas, fire, deep water, chasms. **Forward clock**: alert
  level per floor drives dive-vs-explore. **Allies with state.**
- **Vault "choose one of N"** as the only direct interrupt, only while watching: presence
  earns a decision, never a click.
- **Three-tier exit**: camp stairs 100% of run loot; retreat rule mid-floor 60%; death 30%.
  Meta never lost. Brought vault items are lost on death (the loadout is a bet).
- **Death screen**: cause, margin, last five firings, telegraphs, `gap`/`dice`, candidate
  patches with forecast deltas, free edit. Morgue text export with seed and rules.
- **Chronicle and sifter**: auto-notes during the run; a sifter picks comeback, first-kill,
  near-death, ally-lost, item-gamble into a reel per run and a lineage history.

## Equipment and skills

- **Class** per heir = verb set (v1: fighter, rogue; later ranger, caster). Skills are verbs,
  so every skill unlock is a new row the player can write.
- **Gear** found per run, auto-equipped by a preference row. Vault keeps one item at first,
  more by unlock. Bringing it risks it.

## Idle model

- **Unit = expedition.** The hero runs at 1 turn/s at 1×, so a run's duration is its turn
  count (target 3–8 min to a natural exit); dive rules shorten runs, explore rules lengthen
  them. Offline, the sim runs `elapsed / mean_run` expeditions, uncapped. When runs become
  statistically identical (same wall, no new facts for K runs) the batch is sampled and the
  report says so honestly.
- **Yield of absence**, in order: facts learned → new bests → vault candidates → chronicle.
  Facts are the continuous lane (Pecorella's "present"); bests are the bumpy lane.
- **Active bonus** through decisions only: a patched policy is worth 1.5–3× over the next
  absence. Presence is worth zero.
- **Cadence targets**: a policy stays productive 8–24 h before it stalls; something to decide
  every 1–4 h; a new token or slot roughly daily for two weeks.

## Metagame

- **Lineage**: heirs with traits; graveyard with cause and deeds; facts, rule sets,
  vocabulary and vault persist.
- **Unlocks**: one meta currency earned by new bests only (depth, first kills, trophies).
  Every unlock is a token, slot, tactic card, class, zone, vault slot or automation.
  Toggleable.
- **Trophy ledger** rewarding different policies (D10 with no healing rows; ranged only).
- **Ending** at the bottom, then ascension variants.

## The opening (first ten minutes, no text)

Heir 1, fighter, two rows preset (`HP < 30% → drink`, `foe in view → attack`). Send. The
camera follows; callouts fire. Death on D2 to a pair of jackals: the trace shows `attack`
firing at 35% HP with two adjacent, no retreat row. Verdict `gap`. Candidate patch: `foes ≥ 2
· HP < 50% → back to corridor` with forecast `D3 reach 40% → 78%`. Patch. Send. D4. First
fact learned: `jackal: pack`. Time-to-first-edit < 3 minutes is the gate.

## Presentation

From `research/art-tech.md`:

- **Environment**: lo-fi pixel art, env texel = 4 device px (`k = floor(min(w,h)/270)`), 8×8
  tiles, 16×16 props, ≤ 8-colour per-biome palette + Bayer dither in the blit shader; biome =
  palette uniform swap. Target 270×585 portrait, 480×270 desktop, no letterbox.
- **Camera**: three.js `OrthographicCamera`, **top-down flat for v1** (30° wall tops are a
  later cut; the rendering risk is the sprite mix, not the walls), snapped to the env-texel
  grid with the sub-texel remainder as a blit UV offset; damped-spring follow.
- **Hero and monsters**: codex watercolour-anime sprites in the tacticalswap ART.md keyed
  register, composited at **two densities in one target** (sprite texel = 1 target px, env
  texel = 2): hero 48 sprite-texels = 3 tiles tall, outline dilated ≥ 1 env texel, 40–60%
  palette tint, 1-texel contact shadow, snapped positions, 8–12 fps. Framed deliberately as
  paper cut-outs on a pixel diorama. Option A (all low-res) stays as a toggle.
- **UI**: the editor is the main screen; rows as token chips, drag to reorder, tap to swap.
  Phone-first. **Copy**: callout ≤ 3 words, verdict 1 word, no sentences in chrome; lint.

## Tech

- `crates/riddle-core`: deterministic sim (descent + floor gen, monsters, items, rule
  engine, facts, trace, chronicle, sifter, forecast, offline batch, probes). All truth.
- `crates/riddle-wasm`: thin bridge (`new`, `set_rules`, `forecast`, `step`,
  `run_offline(elapsed)`, `snapshot`, `events`, `export`).
- `web/`: Vite + TS + three.js; replay-viewer renderer with primitive fallbacks per sprite.
- `art/`: manifest + codex + `pack.py` + `art-qc.py` from tacticalswap.
- `tools/`: `verify.sh`, `gates`, `probes`, copy-lint, WebMCP inspector.
- PWA, offline-first save with export/import; versioned precache, verify on a fresh port.

## Bots and gates (30 seeds, `examples/metrics.rs`, never weakened)

| Bot | Rule set | Must |
|---|---|---|
| DEFAULT | shipped preset | die by depth ≤ 6 on ≥ 80% of seeds |
| EDITED | preset + 1 h of agent edits | depth ≥ 10 on ≥ 50%; gap vs DEFAULT ≥ 15 pts |
| RANDOM | random rows | lose every seed |
| PASSIVE | no rows | lose every seed by depth ≤ 3 |
| LEARNED | DEFAULT after 200 offline runs (facts only, no edits) | ≤ 2 floors deeper than DEFAULT: facts alone must not solve the game |

Eval proxies (`docs/FUN_EVAL_IDLE.md` §5): unfair deaths ≤ 5%; top death cause < 35%; deaths
tracing to a player row ≥ 70%; events/min at 1× ≥ 6; offline ÷ active 0.25–0.5; pending
decisions after any absence ≥ 20 min ≥ 1; rule-set diversity across clears ≥ 0.5.

## Kill-tests (P0, throwaway by policy)

**K1 — Legibility.** Text sim, 4-row editor with four families, forecast, trace, death
screen. Variant A: diagnosis only. Variant B: diagnosis + candidate patches. Pass: after a
`gap` death a tester edits within 60 s unprompted on ≥ 70% of deaths; EDITED beats DEFAULT
by ≥ 15 pts; retellings use first-person pronouns *in both variants* (if B loses attribution,
ship A).

**K2 — The look.** One room, pixel pipeline, one codex hero, A/C toggle, phone screenshots.
Pass: five readers call C "one game"; tiles and ink line read at phone size.

**K3 — The return.** 8 h simulated absence over the K1 sim. Pass: every seed's report leads
with ≥ 1 learned fact and ≥ 1 pending decision; yield bounded by the wall, not the clock.

## Milestones and v1 scope

Tracks run in parallel (core / client / art) and meet at wire types; each milestone is a cut
contract (`docs/CUT<n>.md`) with numeric gates and a fun-verdict playtest whose gaps become
the next contract.

| M | Deliverable | Gate |
|---|---|---|
| 0 | Scaffold, wasm bridge, Vite, `verify.sh`, replay-hash test | green |
| P0 | K1 (A and B), K3; K2 on the art track | pass conditions |
| 1 | Sim core: descent + floors, 15 monsters, 25 items, hazards, clock, 8 families, facts, trace, chronicle, exits, forecast, offline batch | all five bots green |
| 2 | Client: replay viewer with pixel pipeline, snapped camera, two-density sprites, callouts, telegraphs, speed controls; editor UI; return report; death screen; vault; save/export | events/min ≥ 6; check-in median 2–10 min in dogfood; 60 fps mid phone |
| 3 | Metagame: lineage, traits, fact-gated tokens, unlocks, tactic cards, trophies, presets, share-as-text | unlock cadence ≈ daily over a simulated 14-day log |
| 4 | Content to the bottom of v1: 3 biomes, 3 wall bosses, depth 15, ending | death-cause entropy green; every boss counter learnable in-game |
| 5 | Story layer: sifter, reel, lineage history, hero voice, grudge monsters, sparse audio | tell-a-friend passes on 3 of 5 runs |
| 6 | Release: PWA, copy-lint clean, WebMCP harness, Tier-2 eval on `idle-roguelike` | ≥ 72, no gate on `return`/`attribution`; blind second card α ≥ 0.67 |
| 7+ | 30 monsters / 40 items / 5 biomes / depth 30, more classes, ascension, live cadence | new bests only |

**v1 playable** = M0–M6 at the reduced content counts above.

## Risks

- **"Plays itself" backlash** → store contract, DEFAULT-must-die gate, decision density.
- **Editor on a phone** → chips + candidate patches; desktop K1 first, phone pass before M2.
- **Style clash** → K2 decides; option A fallback.
- **Content faucet** → data-driven monsters/items/traits from M1; cadence gate in M3.
- **Forecast kills the return** → forecast bounded by facts; the frontier stays unknown.
- **Candidate patches turn the player into a clicker** → K1 variant test; `attribution`
  axis; patches never exceed three and never auto-apply.
- **Facts alone solve the game** → LEARNED bot gate.
- **Solo-dev burnout after the first content wave** (the genre's modal failure) → an ending
  ships in v1; live cadence is a bonus, not a promise.

## Open questions

- Ending depth for v1 (15) vs full (30); whether ascension ships in v1.
- Whether allies-with-rows is M3 or M7.
- Name.

## Review log (v1 → v2, 2026-09-15)

1. **Idle yield was numbers-shaped.** v1's absence returned loot and deaths; a walled policy
   returned "40 runs, 40 deaths". Fix: the hero learns *facts* autonomously (bestiary, item
   ID, biome traits) and facts are the tokens the editor uses. Knowledge is both the
   continuous return lane and the unlock gate. Source: Idle Loops/Increlution "instinct",
   Dragon's Dogma bestiary, Pecorella's "present when you return".
2. **Forecast cancelled the return.** A deterministic forecast over N sims *is* the offline
   result. Fix: forecast only as deep as the hero has facts; the frontier is always `?`.
3. **Full procgen wasted knowledge.** Fix: persistent descent (biome order, bosses, grudge
   monsters) per lineage, regenerated floors. Facts about *this* descent accumulate;
   situations still vary. Source: the "loops" archetype the research names closest.
4. **Slots 4→12 recreated Stuck In Time's brittle long programs.** Fix: rows cap at 8;
   density comes from tactic cards.
5. **The renderer was a live view for a player who is not there.** Fix: it is a replay
   viewer; live is the head of the replay; top-down flat first.
6. **"Smart AI" was unanswered.** Fix: the division of intelligence table. Facts learned
   autonomously, policy written explicitly, chores silent. Plus a LEARNED bot gate so
   learning alone cannot win.
7. Added: candidate patches with forecast deltas as a K1 variant (with the attribution
   risk named), the opening, trait-based difficulty, vault-as-bet, v1 scope.
