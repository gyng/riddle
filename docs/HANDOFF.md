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

### Cut 29 core (built; the new hard gates still fail — resume here first)
Green: 441 cargo tests, clippy -D warnings (incl. examples), wasm rebuilt, qa all pass, every bot / lever /
dice / stall / dance / lane / divergence / oath / return gate; the client suite 36/36; tsc; copy-lint 0.
`node tools/gates.mjs --full` exits 1 on the rows Cut 29 made hard:

| Row | Now | Bar |
|---|---|---|
| Dayplayer: days with an unlock / oath kept / system opened | 7.3 (6 · 9 · 7) | ≥ 10/14 |
| Dayplayer: longest stall, counter known | 8 d | ≤ 3 |
| Progression: unlock days (18 rater lineages) | 5.5 | ≥ 10 |
| Progression: marks unspent after day 2 | 11 (one lineage) | ≤ 8 |
| Progression: stall ≤ 3 d | 2/18 | ≥ 13/18 |
| Progression: purse vs the day's net | 1.11× (one lineage) | ≤ 1 |

E1 (the wall search) broke the D17 wall: dayplayer D18·17·17 → D28·33·28. Next, in order:
1. **Queen silence lever** — 205 floors at D24+ had a silence scroll on the floor and 0 heroes held one:
   the pack is full (≈2 summon_ally, 2 spare mail, 2 spare swords, bow, fire, heal) so the full-pack swap in
   `turn.rs` (`would_take`/`can_take` ~3000) never fires for it. From D24, silence takes a slot from an unread
   summon_ally or a spare weapon/armour. Re-measure with the probes in
   `/tmp/claude-1000/-home-g-p-riddle/205f7863-43f1-46df-a1c8-ce180d81d61e/scratchpad/d28` (may be gone —
   rebuild: count held silence scrolls per floor; past-D28 share per seed), then FULL−D28, COUNTERED, lever.
2. Don't offer `telegraph → retreat` death patches against a telegraph boss whose counter is known (`trace.rs`).
3. ~~**Make the wall search lazy**~~ — **done (2026-09-28)**: `Game::wall_edit()` / wasm `wallEdit()` (searched
   once a day at a wall, then cached as `Lineage.wall`); `ReturnReport.wall` is gone (the client's `mergeReports`
   line dropped). `wallEdit` runs on a background lane (`lanes.ts`, a mirror reload after it). The dayplayer and
   `progression_lib` call it after each report and take a fresh offer (same semantics as before). Measured in
   wasm (node, fast build) on `deep.json` pushed to a wall: 8 h offline in 16 slices 4.9 s; `wallEdit` alone 67 s —
   the client must ask it after the report paints and never block on it.
4. Probe the Foundry wall (D21–23; holds rater lineages 4–10 days) like the Queen: pack census, counter
   facts (reflect_read / buffer), harmful patch rows.
5. Unlock days: rater lineages buy the whole catalogue by day 3–5 and open 20–21/23 systems by day 2 —
   gate later tiers/systems on deeper walls, count commissions and oath draws, add tier-gated oath rewards;
   re-measure each change with `target/fast/examples/progression --bars --dp-seeds 0 --jobs 18` (~22 min).
6. The marks (11) and purse (1.11×) outliers — one lineage each; find it in the per-lineage `progression` lines.
Also done this round: `noise_discipline` is now earnable (a blind foe in view teaches `foe:<kind>:blind`);
the repeat offers a `throw` row's kind as one tap (`Lineage.repeat_added`) instead of adding it.

### Gfx/UI eval (stopped cleanly; resume from `scratchpad/gfx-eval/round6`)
Round 6: mean **5.97** (bar 8.0), lowest 5.00 (desktop report). Best: death 7.20, boss entrance 7.00.
Next, in order (the agent's own list):
1. **Sprite scale** — no re-authoring needed: in `web/src/render/atlas.ts` `Atlas.override`, multiply the
   loaded-sprite `sc` by a `SPRITE_SCALE` (dev `?sprite=`); derive `HERO_TEXELS` (24) in `render/index.ts`
   from it; lower `PHONE_TEXELS` (100) / `DESK_TEXELS` (112) in `ui/viewer.ts` by the same factor. Try 0.5
   (phone ≈ 66) and 0.62 (≈ 80); shoot both with `tools/gfx-round.sh`; fresh raters pick. Check
   fights.mjs "a foe ≥ 24 CSS px" (the rat); `render/view2d.ts` needs the same factor. Codex re-author only
   if downscaled masters read mushy.
2. Real saves for the Fens/fight moments (`tools/gfx-eval.mjs` `phoneFake` → `web/tests/fixtures/deep.json`).
3. Report arrival: render the plaques from data on hand before the async content (`ui/report.ts`).
4. Still screens: bigger ambient motion (breathing portrait/tablets, parallax vista, mist; compositor-only).
5. Fog: a lit texture on the explored edge (`blit.ts`, the `FX > 0` rock block); tighter room framing.
6. Copy-bound asks: `docs/JUICE.md` §10.7 item 5.
7. Clean frame times: `tools/gfx-round.sh scratchpad/gfx-eval/round7` on a quiet box.
Each round: two fresh raters (`round*/rater/prompt.txt`), `python3 scratchpad/gfx-eval/score.py roundN`.

### Cut 29 client (barely started — only the pack-4 fix landed)
Done: `web/src/ui/camp.ts` reads `Lineage.supply_cap` (the pack 4 → 3/3 bug). Everything else is open;
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
