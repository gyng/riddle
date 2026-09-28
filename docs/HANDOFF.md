# Handoff — where Riddle stands and how to resume

*Written 2026-09-27 at the end of a long session. Read this, then `AGENTS.md`, then `docs/CUT29.md`.*


## 0. Owner decisions, 2026-09-28 — the idle-first pivot (read first)

- **Adopted: idle-first** (`docs/IDLE_FIRST.md`). Fun and a sense of progression are the north star.
  Idling alone progresses; rules and builds help and are fun but are never required. Rules become a
  late-game unlock for micro-tweaks; traits/arts/stances become **packages** (pre-written rule bundles that
  level from runs). Progression is four parallel tracks that open stage by stage: **character** (one
  default warrior → classes → pets → advanced classes), **items** (inventory → blacksmith → loot),
  **scale** (one hero → many → aggregate), **town** (tavern, bank, houses…). Start simple; add simple,
  coherent, fun systems.
- **Order**: finish **Cut 29 as written** (the owner's choice), then **Cut 30 = the pivot's first cut**
  (IDLE_FIRST.md's draft: the idle floor, packages, a day-0 camp → town strip with a bank, a tracks panel,
  the new gates). **Hold before Cut 31** and update this doc. The hard invariants in AGENTS.md/PLAN.md
  ("policy is never optional", DEFAULT must fail, ~20 "not engaging fails" rows) are **replaced in Cut 30**
  by the idle gates (idle alone reaches D8 on day 1 and D23 by day 12, gold every day; packages ≥ 1.5×
  faster; rules ≥ 1.5× again; nothing required) — not before, so Cut 29's gates stay as written.
- **Scoring preset: `idle-hybrid`** (`eval/presets.json`, the mean of idle-roguelike and idle-rpg). Cards
  from the pivot on use `"genre": "idle-hybrid"` (update `eval/SCORECARD.template.json` and the rater
  prompt when cohorts resume). Cohort 24 re-scored under it: 74.6 · 71.1 = **72.85**. Run a control
  cohort on the pre-pivot build under `idle-hybrid` before judging Cut 30.
- The earlier Cut 30 traits design (`docs/TRAITS.md`) is superseded: its data, wake cards and reveal are
  reused as packages. Cut 31 (specialisation forks, weapon properties) needs re-scoping under the pivot.
- **Oaths are still unclear to the owner** (2026-09-28), even after the Cut 28b clarity pass whose
  blind probe scored 9/9 — an agent-reader pass is not a human pass. In Cut 30, redesign oaths as a
  simple **quest board**: one plain goal line (`reach D10 · no return`), the reward as a picture, a
  progress bar; no stake mechanic; arriving late on the ladder, or cut if it doesn't fit the four tracks.
  Validate with the owner, not only blind agent readers.
- Death screen: the `gap` seal reads **YOU DIED** (score screen only; `no rule for it` stays under it).

## 1. The goal and the honest target

The owner's standing goal: keep cutting, improve the UI (WC3/SC2 skeuomorphic skin, UX/IA), speed up
iteration, and tune for fun "until 95". `docs/PATH_TO_95.md` shows 95 is above every calibrated title on
this scorer (Hades/Balatro 90.3, Chess 90.9, Stardew 88.5, Celeste 85.9, Into the Breach 83.6; Loop Hero,
the nearest comparable, 74.1). Every axis at 0.8 = 80.0; ~86.5 adds 1.0 on decisions/failure/return/
attribution. The owner has been told; the loop continues toward 95 regardless.

## 2. Scores (blind cohorts, two raters each; `docs/PLATEAU.md` has every read)

| Cohort | Build | Protocol | Mean | Note |
|---|---|---|---|---|
| 15 | fba365b | one absence | 74.35 | old best |
| 19–21 | 4b15a61 … 307dbed | one absence | 71.65 · 74.35 · 72.15 | plateau |
| control | 307dbed | **three absences** (20m/4h/8h) | 65.7 | the protocol costs ~6.5 |
| 22 | 420f27c (Cut 26 fork) | three | 68.65 | +2.95 vs control |
| 23 | 631fe23 (Cut 27 fold/scene + juice + perf) | three | 71.6 | +5.9 |
| 24 | 9720ff7 (Cut 28 oaths) | three | **73.25** (AW 75.1, session best) | +7.55; expression, tension, autonomy reach 0.8 |

Compare new cohorts against the control (65.7), not the old one-absence numbers.

## 3. The loop (unchanged; it works)

contract `docs/CUT<n>.md` → core agent (crates/, wire fields in `web/src/engine/{types,fake}.ts` first) ∥
client agent (web/src/ui) → full verify → commit → `tools/ship.sh --preview` → 2 QA players
(`eval/QA_TRANSCRIPT_PROMPT.md`, `eval/QA_PROMPT.md`, briefs via `sed` into `scratchpad/<qa>/brief.md`) →
one full-stack fix agent for both lists → commit → `tools/ship.sh` → 2 blind raters (`eval/RATER_PROMPT.md`,
three absences, seeds `N01/N02`, ports `57x1/57x2`) → `eval/score.sh` + `node ../eval-fun/tools/fun-reliability.mjs`
→ PLATEAU entry → next contract. Screenshots to the owner at each milestone (they asked).

Verify before any commit: `cargo test -q --workspace --profile fast`, `cargo clippy --workspace
--all-targets -- -D warnings`, `tools/wasm.sh`, `node tools/gates.mjs --full` (≈ 8 min; per-leg cache),
`cd web && npx tsc --noEmit -p . && node ../tools/copy-lint.mjs && pnpm -s test` (own Vite server per run,
≈ 5 min), `node tools/playtest.mjs --seed N` (headed walk).

## 4. State of the working tree at handoff

**Branches.** `main` stays at the last green commit (`56d680e` + this doc). Everything below is committed
on **`cut29-wip`** — check it out to resume. It is not shippable yet (§6 Cut 29 core item 3), and
`node tools/gates.mjs --full` fails only on the rows Cut 29 made hard. When those pass, merge it to main,
then QA → cohort 25. On the branch:
- **Cut 28b — oath clarity** (done, verified by its agent): the deal as a formula, `stake`, OATH
  KEPT/BROKEN beats (`Ev::Oath`), the board revealed at the first plateau/boss wall. Blind probe 1/9 → 9/9.
- **Copy pass** (`docs/COPY.md`): rules named by their words everywhere (no R1/R2 — `tokens.ruleName`,
  `tokens.nameRefs` rewrites core `R2` at render), `luck`/`rule`/`repelled` seals, `N/12 replays survive`,
  `vs last run`, `15th heir`, `best D8`. Blind comprehension 64.5 % → ~88 %; stands-alone 73 → 88 %.
- **Gfx/UI eval rounds** (`docs/JUICE.md` §10, `tools/gfx-eval.mjs`, `tools/gfx-round.sh`): blind visual
  mean 4.90 → 5.97 (bar 8.0); the desktop three-column frame (`web/src/wide.css`), WebGL context-loss
  fallback (`render/view2d.ts`, `render/fallback.ts`, `web/tests/ctxloss.mjs`), mobile layout check
  (`web/tests/layout.mjs`), boss plates, slash arcs, lit fog edges, icon plaques, audio ducking.
- **Cut 29 core** (built, gates NOT yet passing — §6): frontier mark removed, daily mark, tiers T0–T6,
  prices ≤ ◆8, condition words free, automations to gold, oath slots/draws/commissions, 23 systems
  opening one at a time (`systems.rs`), diagnostics meters (`meters.rs`, qa: totals == event sums), the
  wall search (`wall.rs`, breaks the D17 wall → D28–33), keep sheet only when close, standing orders,
  fallen companions named, tamed grudges close as `tamed`, `ForecastEnds.passage`.
- **Cut 29 client**: only the pack-4 fix (`camp.ts` reads `supply_cap`) — the rest is §6.

## 5. The owner's decisions this session (all binding)

- Dying is fine; progression must still work on an all-death night. Nothing punishes absence.
- Hero looks: male/female/cat per class, swap on the portrait (done, `ui/look.ts`).
- Oaths must be self-explanatory (done, Cut 28b).
- Copy must be human-friendly: **no rule ids anywhere**; name a rule by what it says. Blind tests show
  one screen at a time ("stands alone"). Labels/buttons/verdicts may use **2 words** (callouts ≤ 3, no
  sentences). World concepts get **an icon + a one-time ≤ 3-word caption**. Misread numbers are
  **restyled, not dropped** (± as a noise band, the move as a signed chip, `12/12` beside its fix).
- Keep iterating agents until a good score (copy ≥ 90 % per surface; visuals ≥ 8.0 mean, no moment
  < 7.0); stop only after three sub-threshold rounds, and then report blockers to be unblocked.
- Gfx agent cleared to: fix sprite scale vs tiles (zoom first, else re-author ~60 sprites via Codex),
  lit fog look, bigger ambient motion on still screens, fix the report's 0.6 s empty arrival, use real
  saves (web/tests/fixtures/deep.json) not the fake engine.
- Wanted next: diagnostics meters (dps/hps/"tps"), desktop IA, "a tap is a decision" (no busywork),
  progressive unlocks of the game's *systems* (rules not fully open at start), a trait system with
  proc-gen traits (Cut 30), specialisation forks + weapon/armour properties (Cut 31).

## 6. Open work, in order

### Cut 29 core (built; the progression bars still fail — not mergeable; resume here first)
State at `08a880b` (2026-09-28, core agent): 450 cargo tests (+1 ignored), clippy -D warnings (incl. examples), wasm rebuilt, qa all
PASS, every bot / lever / oath / dice / stall / dance / lane / divergence / return gate PASS (FULL−D23/28/33 held 100 %,
kitted too; COUNTERED ≥ D14 70 %; lever with every oath reward 4/4). `node tools/gates.mjs --full` exits 1 on exactly
these rows (final run, 30 seeds, 1776 s):

| Row | Handoff (27th) | Now | Bar |
|---|---|---|---|
| Dayplayer: days with an unlock / oath kept / system opened | 7.3 | 7.0 (9 · 7 · 5) | ≥ 10/14 |
| Dayplayer: longest stall, counter known | 8 d | 9 d (final best D29 · 28 · 28) | ≤ 3 |
| Progression: unlock days (18 rater lineages) | 5.5 (6.3 on the 28th's first run) | 8.3 | ≥ 10 |
| Progression: stall ≤ 3 d | 2/18 | 11/18 | ≥ 13/18 |
| Progression: marks unspent after day 2 | 11 (8 on the 28th's first run) | 13 | ≤ 8 |
| Progression: purse vs the day's net | 1.11× (1.35×) | 1.82× | ≤ 1 |

Rater lineages now end D20–D33 (were all D22–23): the stall and unlock rows moved; marks and purse got worse because the
lineages now stand at the D33 wall with the whole catalogue bought (marks pile, no oath reward left to draw) and the
commission price (10 units × 1.25ⁿ) outruns the purse bar's floor (10 units).

Done (in order of commits on `cut29-wip`):
1. `9ac89d0` **Lazy wall search** — `Game::wall_edit()` / wasm `wallEdit()` (once a day at a wall, then cached as
   `Lineage.wall`); `ReturnReport.wall` gone (`mergeReports` line dropped); `wallEdit` on a background lane (`lanes.ts`,
   mirror reload after). 8 h offline on `deep.json` at a wall: 4.9 s in wasm; `wallEdit` alone 67 s — ask it after the
   report paints, never block on it. The dayplayer and `progression_lib` call it after each report.
2. `775540c` **Queen silence slot** (`turn.rs` `queen_wants/queen_slot/queen_keeps`, from D24): held silence at D28 0/40 →
   40/42 on a D28 snapshot, past D28 0 → 28/48 with the counter row; FULL−D28 still held 100 %. The 307dbed save hash
   re-recorded (event diff: first difference is that pickup, send 6, D28).
   **No telegraph retreat against a countered boss** (`trace::telegraph_row` takes the facts).
3. `a52fc21` **Foundry wall** — probe (rater snapshots): the wall is iron golems at D19–20 (reflect_melee), not the
   Master; every lineage owned `reflect_read` and knew the tag, no goal set wrote it, no death offered it. With the card at
   R1: reach D23 0.3 % → 90 %. `buffer · D19+ → attack buffer` is harmful (0/320). Fix: the card in the wall stock
   (owned, not in the set) and pinned on a reflect-melee death (`trace::card_counter`). `PROG_SNAP=dir` snapshots.
4. `08a880b` The wall stock carries the wall boss's known counter (plain and floor-gated): stall 9 → 11/18.
Tried and reverted (patches in the session scratchpad `wall-fullmeasure-pin.patch`, `cadence-boss.patch`): measuring the
wall's candidates on every sim (the camp's tick budget keeps ~9 of 48 at D33), `WALL_BAR` 0.10 → 0.06, and always
measuring the floor-gated counter in full — 8–9/18, no better than 11 (noise ±2 between runs). `cadence` striking a boss
in view first (tests green, not gate-measured) — see next 2.

Next, in order (measure each with `target/fast/examples/progression --bars --dp-seeds 0 --jobs 18`, ~25 min; single runs
move ±2 lineages, so compare two runs before believing a move):
1. **Record spikes.** Most remaining stalls are a lucky record, not a wall: e.g. rater AP s1 reached D33 on day 4 with a
   set that reaches D33 on ~0–2 % of its sends (day 6), 14 % by day 10; the dayplayer's D28s come from sets that reach D28
   on 0–1 % of sends (its later patches degrade the set: `always → cadence` and `telegraph → retreat` rows). The wall
   search is blind there (share past/reaching the record ≈ 0 for every candidate). Options: a smoother search objective
   (mean depth toward the record), or measure a wall from the deepest lit waystone below it.
2. **Bare `cadence` card** (the client inserts a bought card bare: `always → cadence`): it pre-empts `boss focus` and hits
   the Warlord's shield-goblins until he drives the hero off (dayplayer seed 3, every send, nine days). `cadence-boss.patch`
   makes its plain blow go to a boss in view first; gate it (FULL's D33/D34 rows) before keeping.
3. **Marks at the deepest wall**: once the catalogue is bought and the oath pool is dry (bold/lean titles are per depth,
   fire per boss), draws fail and marks pile (AP s1: 25). Needs a repeatable late sink (e.g. `fire` naming any met boss
   without its title; escalating titles), or the D33 wall passing (ascension resets the catalogue).
4. **Purse**: the commission's 1.25ⁿ price outruns the bar's 10-unit floor at a stall (AS s1 $5959 on day 5). Price
   commissions against income (e.g. ≤ one day's net) or record a deviation — never loosen the bar.
5. Unlock days (item 5 of the old list) and the dayplayer bars were not worked on directly this session.
Also done earlier: `noise_discipline` is earnable (a blind foe in view teaches `foe:<kind>:blind`); the repeat offers a
`throw` row's kind as one tap (`Lineage.repeat_added`).

### Gfx/UI eval (paused for the owner's stop after round 21; motion-aware baseline 6.74, round 21 6.68)
The harness now shows raters full-resolution motion (`-motion.png`: changed pixels in red) and each moment's intended movement; round 20's
build re-rated under it is the new baseline **6.74** (death 7.90, boss entrance 7.70, forecast 7.40). Round 21 (c6008ad: 35 more painted
props, denser dressing, deeper wall-foot shadow, pixel-register boss shield, coin sprites) scored **6.68**. Bar 8.0 not met.
`docs/JUICE.md` §10.10–10.11 has the tables. Next, in order: the Fens (5.50: its own props, a warmer hero pool); the boss break (the pixel
shield muddles him: offset the split above him, bigger halves, a crack flash); early-floor camera framing; the IA asks (report OPENED chips,
the scene inset over the route row, desktop meters). The oath board is skipped: Cut 30 redesigns it as a quest board (the reward vignettes
painted for it wait, uncommitted, in `art/ui/oath/`).
Resume: `tools/gfx-round.sh scratchpad/gfx-eval/roundN`, two fresh raters (`roundN/rater/prompt.txt`), `python3 scratchpad/gfx-eval/score.py roundN`.

### Cut 29 client (2026-09-28 — every numbered item below is closed and committed on cut29-wip; `web/tests/cut29.mjs`, 34 checks)
**Closed:** 1 `mergeReports` (`ui/meters.ts mergeMeters`) · 2 keep sheet (`decide`/`note`; a full vault's decision is a `swap`
sheet) · 3 systems (`ui/systems.ts`; reveal steps follow `Lineage.systems`; reorder/vs/divergence/walls/route/automations/tags/
exit verbs gated; report `opened` plaques; `seenSystems` at the next send — a mutating call mid-edit cost clarity:paint ~1 s; the
proxy lists the Cut 29 methods; fake `?systems=all|none`, the older suites run with `none`) · 4 meters (watch toggle, death
fight, report night, forecast two-run compare, desktop slot) · 5 standing orders (one tablet + sheet, `setOrders`) · 6 oath
slots/draw/forswear-by-id/works · 7 `repeat_added` one tap · 8 passage (`~$125/run +$135 passage`) · 9 fallen lines · 10 wall
edit as a patch (`ui/wall.ts`, lazy `wallEdit()` after the paint, `App.applyRules`) · 11 automation/tier labels, long-press
reorder any distance · owner copy: YOU DIED seal (death screen only), budgets 2 words (`allowPhrases`), `runs banked`/`fights
only`, concept icons + one-time captions (`ui/concepts.ts`), the ± as a noise band + the move as a signed chip, the gem says
`apply` (count on its fix) · gfx raters' layout asks (opened under plaques, hex gems in the forecast panel, no `?` column,
the scene never mid-row, headline wraps by segment, sheets meet their anchor). Blind protocol pass 9: `scratchpad/copy/pass9`.
Known: core `keep()` on a full vault evicts the weakest item of any category (the swap sheet follows it; `auto_keep` evicts
same-category only) — a core call.
Done earlier: `web/src/ui/camp.ts` reads `Lineage.supply_cap` (the pack 4 → 3/3 bug). The rest below is open;
the fake engine (`web/src/engine/fake.ts` ~2082–2195) already has Cut 29 stand-ins, so build against it:
1. `mergeReports` in `app.ts` must carry the Cut 29 fields (sum `night_marks`, union `systems_opened`,
   concat `oaths_kept`/`fallen`, last slice's `wall`, `meters` field by field) — else offline reports drop them.
2. Keep sheet (`ui/watch.ts` `pendingExit` ~457, `finish()` ~2014): skip when `exit_pending.decide` is
   false → `autoKeep` + show `note`. Check the full-vault swap semantics against core `keep()` (engine.rs ~5139).
3. Systems: gate tiles by `systems[].open` alongside `reveal.ts`, glint `systems_opened`, call `seenSystems`.
4. Meters (one module for watch toggle, death `Death.fight`, report, camp two-run compare, desktop
   `metersSlot()` in `frame.ts`); units always; rules via `tokens.ruleName`.
5. Standing-orders panel (`setOrders`); oath slots/`drawOath`/`forswearOathId`/commissions in `oaths.ts`;
   `repeat_added` one-tap; `~$X/run +$P passage` from `ends.passage`; `fallen` lines; the `wall` edit as a
   patch; tier cards read `meet Warlord`, automations `$N more`; drop `◆+1 frontier`; long-press reorder
   any distance.
6. The owner's copy decisions (§5): budgets to 2 words in `eval/copy-budgets.json` (copy-lint reads it),
   reword `GAP`/`BANKED`/`highlights`…, concept icons + one-time caption, restyle the forecast move and
   move `12/12` beside its fix; rerun the blind protocol (`scratchpad/copy/capture.mjs`).
7. `web/tests/cut29.mjs`, full suite on a quiet box, headed screenshots.


## 7. Queued cuts

- **Cut 30 — traits** (`docs/TRAITS.md`, `docs/CUT30.md` draft). Prototype (`examples/traits_proto.rs`):
  stat-bonus traits are too strong and mostly just numbers (2/66 pass); real builds come from traits
  that change what a rule's condition reads (`hale`, `light sleeper`) and from costs. Ship only traits
  that pass the strict build test; a trait never acts on its own; arrives at heir 3 / first death past D5.
- **Cut 31 — specialisation forks at class levels 3/5/7 and weapon/armour properties** (reach, cleave,
  stagger, bleed, pin; quiet, fireproof) with conditions like `weapon: reach`; each fork must change the
  best set by ≥ 2 rows and stay under the lever.
- The owner's listening pass (`eval/AUDIO.md`) would let raters score audio for the first time.

## 8. Gotchas learned the hard way

- **Never `pgrep -f`/`pkill -f` in a Bash tool call** — the pattern matches the tool's own `bash -c`
  wrapper: waiters spin forever (several spun 1–6 h this session), kills kill your own shell. Wait on
  PIDs (`while kill -0 $PID`), and give every `until grep` waiter a PID exit.
- Never `cargo fmt` (the repo isn't rustfmt-formatted). Never stage from `/home/g`.
- Never run `tools/ship.sh` or heavy gates while raters play :5230 (feel/frame time is rated); cap
  threads (`RIDDLE_THREADS=8`, `nice -n 15`) when agents share the box.
- A full gate run is CPU-bound (~8 min, 32 threads); concurrent agents' runs push load to 60+ and make
  client timing suites flaky — rerun a failing timing suite alone before believing it.
- `tools/playtest.mjs` right after `tools/ship.sh` can abort (the pkg rebuild reloads Vite) — rerun it.
- Agents sharing the tree: give each an owned area, "targeted edits only, re-read before editing".
