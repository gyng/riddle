# Handoff — Riddle

**Verified quiet-tick fix, 2026-10-04:** exact batching in offline, advance,
run-to-end and forecast loops; `docs/PERF_CATCHUP.md`. Three differential tests
and full verification pass (3906s, resumed nine real completed cases after
SIGTERM). All272 fortnight records match the previous accepted records exactly.
Seven headed shipping-WASM pairs per camp give3.21% /6.09% /3.50% gains
(early/late/tuned); every report/save matches. Target15% missed; retained as a
consistent small gain with exact reference/gate agreement. No tuning speedup
claimed. The complete cached table passed in1.32s (all three genuine leg hits).
Core source `97a4dba1715763c3`, local shipping SHA256
`3451f53c3654c1d11063bee831d4815a3a6ca08d560c0b9411884dcca5a3a132`.
Evidence: `scratchpad/tick-batch-20261004/{full-resumed.log,full-exit.txt,
completed-case-comparison.json,cached-full.log,cached-full-time.txt}`.
Local shipping real-WASM/SW/offline/400/1440 and the eight-capture headed
send/scout/8h walk pass; screenshots shown inline. Walk104s under full-gate load:
functional evidence only. Publication is approved; finish the pushed release's
CI/public checks and inline public checkpoint before reporting it deployed.
Owner asks all gates/cases within minutes: plan at the top of
`docs/ITERATION_SPEED.md`. Split safe test-only dependencies first; fresh runtime
runs require major exact forecast-work reduction, not more worker threads.

**2026-10-03: Cut 30.5 and local iteration pass deployed; hold before Cut 31.** Read `PLAN.md`,
`docs/CUT30_5.md`, then `AGENTS.md`. The works tree, worker art, manual first
sends, run-clear/runs UI and Pages release are integrated. The first porter is
free; sends stay manual until the scout. Record checkpoints secure carried
gold without ending the run. Offline is uncapped after the scout.

The Cut 30.5 baseline `tools/verify.sh --full` passed in 437s with warm primitive
caches. The local performance follow-up passed genuinely fresh in 4679s, then
again in 111s with all three simulation legs cached (including test-artifact
rebuilds): 517 tests, one ignored,
TypeScript/copy lint, all-target Clippy, shipping WASM/web build, 30-seed engine
and trace/wire checks, all 272 fortnight cases. Focused client 88/88 and
real-WASM/SW/offline/headed 400/1440 release checks pass. Existing full client,
headed send/scout/8-hour walkthrough and 493-frame art QC passed earlier;
frontend/art inputs stayed unchanged.

Pre-optimization core source `2822afe224cb289b`, content commit `5386ab4`: Guarded L5 dry
exit 44%, heal45/hurt25 unchanged. Queen-scope fix `7fe1229` narrows generated
silence to learned brood; explicit rows/raw templates/counter facts stay intact.
Quartermaster requests stop only once the chosen start passes the boss.
Actual income/upfront guide pricing and checkpoint accounting repairs remain.

All 16 no-forge pairs meet48h, and companions add value:10.58349% deaths/send
with vs10.98394% without. Away workers27.26 vs27.14 mean best; daily27.52 vs27.41.
Pre-optimization shipping WASM3929188bytes SHA256
`2210c7a151a1887817ccc349cc04b006f9377d43317b0c623fcd8298c8a47a3b`.
Private evidence: `scratchpad/cut30_5-check-pack`; final log
`/tmp/riddle-final-verify-full.log`. Rejected probes and bounded causal audits
are retained there; no fabricated caches or weakened gates were used.

Next: owner fun review, then cohort25 when approved. **Do not begin Cut31.**
The public alpha is live at `https://gyng.github.io/riddle/`, repository
`https://github.com/gyng/riddle`, deployed core commit `ef316dd`. CI run `37128479275` passed;
headed public-site checks passed real WASM, scoped assets/service worker, dated
alpha banner, phone/desktop layout and offline reload. Public screenshots and
CI evidence are under `scratchpad/perf-20261003/`. Publication was explicitly
approved in the current chat on 2026-10-03.

Owner requested inline screenshots at milestones and queued performance/iteration
work after this checkpoint. The prioritized follow-up is at the top of
`docs/ITERATION_SPEED.md`: current measurements, QA checkpoint automation, gate
scheduling/reuse, remaining history allocations, then shipping-WASM catch-up.
Owner then requested continuing optimization, with local work ahead of CI.
The active measured runtime contract is `docs/PERF_LOCAL.md`.
Local tooling now provides `tools/tune.sh <rows>` (full seed counts and fail-fast;
`--list` for row IDs), independent completed-leg persistence/progress, checked
shipping-WASM packaging reuse, and direct-URL screenshot manifests. Fresh
`idle-d8` passed 16/16 in 6.12s; a checked no-edit shipping build took 1.88s.
The small healing-allocation core optimization is fully verified: fresh full
verification passed in 4679s (all 272 fortnight cases), then canonical verification
with all three genuine cached legs passed in 111s, including rebuilding affected
test artifacts. Current core source is `d84ba387fbdb62da`; shipping WASM SHA256
`5c1b05d077df71c0d166e1ed46df051b68e20afb260611d4907eb2e7222dde5d`.
Reports/saves and eight-seed fingerprint `35cb82317410f64e` are unchanged.
Native catch-up gain is 2–5%, with no measured WASM gain. A quiet native tuning
repeat did not confirm the earlier contended improvement (CPU15.52→15.88s);
no tuning speedup is claimed. Core commit `ef316dd` deployed successfully; public
real-WASM/SW/offline/400/1440 checks and an eight-capture headed walk passed.
The walk took29.6s with no console/page errors; two pre-existing WebGL warnings
remain. Evidence: `scratchpad/perf-20261003/core-public-{checkpoint,walk}`. Tooling commits
`cfed378` and `be1283f` passed public CI and deployed successfully.
This engineering queue does not start Cut 31 or a new cohort.

Catch-up profiling follow-up, 2026-10-04: `docs/PERF_CATCHUP.md` records actual
shipping-WASM and symbolized browser profiles, exact report/save comparisons and
rejected compiler/vision-cache probes. The vision candidate gave 3.95% in seven
headed pairs, below the 15% target, and was not promoted. Production core source
and live behavior remain unchanged. New reusable tool:
`node tools/profile-catchup.mjs SAVE --out scratchpad/profile`.

The dated sections below preserve earlier decisions and implementation history; this resume note supersedes
older "next" and branch-status statements.


> **Status, 2026-09-30.** **Cut 29 is merged to `main`** (merge `dc285a1` + the client fix `2f1bc77`):
> `node tools/gates.mjs --full` passes, 225 rows — progression unlock days 10.5, marks 8, stall ≤ 3 on
> 14/18, purse 0.76×; the dayplayer 10.0 unlock days, 3-day stall. The bars sit right at their thresholds —
> a small core change can flip them (runs move ±2 lineages). **Art direction phase 2** (the approved
> Bloodlust × Downwell × 80s-watercolour guide, `docs/ART_DIRECTION.md`) is in progress on `cut29-wip`
> (heroes, bestiary, UI done; tiles, pets/items, town next) and merges when whole. **Next: Cut 30**
> (`docs/CUT30.md`, the idle-first pivot) — then **hold before Cut 31**.


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

## Latest live gate finding (2026-10-03)

Integration `d1fef1d`, raw core `5f72af3420faf66d`, combines the Master sustain/canonical copied-counter order repair with automatic pack-plan deduplication, remembered automatic-slot refresh and protected manual-slot provenance. All **502 native tests pass (one ignored)**, the two self-contained dayplayer example tests pass, and all-target clippy passes. Shipping WASM remains the older 759c build and must be rebuilt after the final core freeze.

The isolated combined no-forge seed 1 reaches D23/D28 at 104 h and D29/D33 at 256 h. Its exact same-source IDLE control reaches D23 at 160 h and D28 at 176 h, but never reaches D29 in 14 days. Earlier 759c IDLE numbers are not a valid control for this new source. The exact 16-seed IDLE-only subset passes bounded D13 delta (−4 h) and other idle bars but fails the pre-D23 stall cap: seed 7 holds D20 for five days (allowed four). Master-only d0 reproduces that trajectory exactly; pack-only 243f has a three-day stall. A scope-restoration prototype keeps the current heal guard plus the original D19+ restriction, with `attack tag:buffer` supplying target filtering within two conditions. It remains isolated pending behavioral tests and a bounded seed-7 pair.

A controlled proof exposes prospective edit forecasts simulating raw row placement while public `set_rules` compiles authored pen rows before packages. A suggested generated-row move can also compile back to its original position. The scoped prospective repair (`af9dc52e419cbbbd`) passes five behavioral tests, all-target clippy and an actual saved-camp proof: every measured candidate equals public application, with archived fights unchanged. The example consumer repair passes six tests and clippy: landed non-harming gems, positive report-stall measures and faithful move/remove/replace/drop operations. Both are applied to integration (uncommitted); **507 native tests pass (one ignored)**. Example/integration lint checks are pending.

The retained complete gate session 87510 remains the **759c comparison**, not the current integration source. It completed all 272 jobs with exit 1. Metrics and QA (30 seeds) pass; TUNED/PICKED D33 ratio 1.67, workers and IDLE bounded delta −4 h pass. Its two remaining balance failures are no-forge seed 1 D23 (192 h versus 136 h, +56 h against a 48 h cap) and the complete 16 pet pairs (20.9534% deaths with pets versus 17.8967% without). Evidence is `pet-waystone-final-*` in the check pack. Do not merge or publish before current-source verification is green.

The retained older `f3b0f27564074581` comparison unexpectedly exited with SIGTERM 143; no owned comparison process remains. Its 233 completed job caches and partial log are retained as `supply-interrupted-*`, without a full verdict. Avoid non-TTY ETX stops during parallel checks. Rejected trace sustain and owned-counter maintenance prototypes remain unapplied.

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

### Cut 29 core (2026-09-30: every gate passes — ready to merge; the history below)
**Final (c51f613 core + the two client fixes below):** `node tools/gates.mjs --full` exit 0 — 225 PASS, 0 FAIL (30 seeds,
1957 s): progression unlock days 10.5 · marks 8 · stall 14/18 · purse 0.76× · frontier 0; dayplayer unlock days 10.0 · stall 3
· marks 8; FULL ≥ D29 93 %, FULL−D33 100 %, dice 4.7 %, lever with every oath reward 4/4, oath gates 19/19; qa all PASS.
`cargo test` 453 (+1 ignored), clippy -D warnings, wasm rebuilt. Client suite (`pnpm -s test`) 36/37 with `clarity:card` a
timing flake (passes alone ×2). Client fixes: an anchored sheet re-places itself when its anchor moves (`ui/sheet.ts`;
cut23's `R1's verb sheet leaves the row in view` failed 3 of 4 runs — the camp repainted under the open sheet, R1 rose
63 px); `clarity:core,watch` expected `· killer:` on the try row, which gfx round 18 (83a1e6d) removed on purpose
(failing since bdb9d25; the counter is the gate, the killer now optional).
**2026-09-30 (core agent, resuming the list below).** Baseline re-measured at `bdb9d25`: the progression run is
deterministic (8.3 · 13 · 11/18 · 1.82×, exactly the 28th's). Checkpoint 1 (items 1, 2, 4):
- **Record spikes** (`wall.rs`): the wall's search measures from the deepest lit waystone at or above the record
  (the offer carries `WallEdit.start`, its first edit `start D24`; the client's apply and the harnesses set it), past
  `wall_floor` — the record, or the floor the set reaches on ≥ 25 % of its sends when the record was one lucky send's
  (AP s1 d10: `start D24 · R6 → boss → cadence` past D33 0 → 43 %). The counters/stock rows are gated to that floor.
  **The last exit row is never dropped or written over**: AO s1 / AR s1 lost their only bank to a wall edit and
  earned nothing for 8–9 days (every send died).
- **Cadence** strikes a boss in view first (the old `cadence-boss.patch`); FULL rows unchanged (D34 9.7 % vs 9.9 %,
  FULL−D33 still 0 past D33).
- **Commissions** priced against income: min(10 units × 1.25ⁿ, max(one day's net, 5 units)) (`kit::COMMISSION_FLOOR`).
- Measured: progression unlock days 8.3 → **9.2**, marks 13 → **9**, stall 11 → **15/18 PASS**, purse 1.82 → **0.85× PASS**;
  dayplayer unlock days 7.0 → **10.0 PASS**, stall 9 → **4** (seed 3 at the Queen D28, days 7–10).
- Remaining: marks (AP s1 / AS s2 at 9: draws fail once the oath pool is dry — item 3), unlock days (AS s2 5,
  AS-stall s1 6, AV s1 6: late days with nothing left to buy and oaths sworn that the set never keeps), dayplayer stall.
  Diagnostics: `PROG_MARKS=1` / `PROG_OATHS=1` on `examples/progression` print the check-ins over ◆8 and each day's oaths.

Checkpoint 2 (item 3 + the oath a player keeps):
- **Titles come again, numbered** (`oath::next_title`: `Bold at D34`, `Bold at D34 II`, … — the first of its line
  neither owned nor on the board); a numbered title stands while its line is the kind's line. A draw may re-draw the
  replaced oath's own kind when it gives something else (AP s1 at D33: every other kind's reward already on the
  board → `no oath to draw`, ◆9). The ◆2 draw is now a sink that never dries.
- **The replay's player reads the oath line before swearing** (`progression_lib::pick_oath`, the day's first
  check-in): the unsworn oath the set keeps most on the camp's panel, when a night keeps it ≥ 50 %. It swore the
  board's first oath whatever the set could keep (`Warlord · fire` nine days by a set with no fire; `tame · a new
  kind` 53 times on empty days by sets with no tame row) — PROGRESSION.md §7's projection assumed kept oaths.
- Measured (p3): unlock days **10.2 PASS**, marks **8 PASS**, purse **0.93× PASS**, stall **11/18 FAIL** (was 14–15:
  chaotic — kept oaths grant waystones/routes; the stalls are the Queen D28 ×3, the King D33, and walls after an
  ascension: AU s1 D28 5 d, AQ s2 D23 5 d). Dayplayer unchanged (10.0 PASS, stall 4). `WALL_DAYS` 1 was tried on
  the dayplayer: stall 4 → 6 (reverted). The Queen: from D24, 48 % reach her floor and 0 past on every candidate
  (21 of 32 deaths to her lurkers); FULL passes her on ~23 % of the sends that reach her.

Checkpoint 3 (the walls and the purse — every progression and dayplayer bar passes):
- **The wall's start is chosen, not assumed** (`wall::search`): the wall floor is the deeper of the set's floor from
  its own start and from the deepest lit stone, and the stone is offered only when it passes that floor more often (a
  deep start skips the shallow finds and levels: the dayplayer from D19 met the Foundry at D19–20 while its D1 sends
  met the Queen at D28, and the offer optimised past D19). Every band boss's known counter between the floor and the
  record is weighed (AS-stall s1 at D33 read a D31 floor and never weighed `cadence` for four days). Stock: `hp < 90%
  → noise discipline` and `summoned → attack summoned` (the Queen's brood shields and mends her).
- **Commissions at half a day's net** (floor 5 units): at a whole day's net a purse keeping tomorrow's oath beside it
  sat at 1.02× (AS s2: $3078, a $2020 work, a $930 oath). `PROG_PURSE=1` prints check-ins over 0.9 of the bar.
- Measured (p6): progression unlock days **10.5**, marks **8**, stall **14/18**, purse **0.76×**, frontier 0 — all PASS;
  dayplayer (dp7) **all PASS**: unlock days 10.0, stall 3, marks 8 (every seed passes D34 and ascends by day 11).
- Harness gaps found (not changed): the replays never buy supplies (a `read silence` / `throw fire` row packs only
  what is found; the repeat's `+ fire · for throw fire` offer is never tapped); an absence fields the vault only on
  its first send (`start_run` takes `Game.loadout`), so the camp's panel (fielded) overstates the night.

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

### Art direction phase 2 (2026-09-30, the owner approved docs/ART_DIRECTION.md: regenerate the library to the guide)
Owner decisions: the primary/SEND gem is BLOOD; "the hero must be readable" — the proportion is picked by a blind read at game size.
Pipeline (all under `art/`): `make_prompts.py` `header3()` puts the guide's STYLE PREAMBLE (read verbatim from `prompts/style_targets.txt`)
first in every sprite prompt and points Codex at the style targets; briefs are `brief3` in `manifest.json` (heroes; `phase2_briefs.py`
the bestiary; `phase2_ui.py` UI/portraits/items/town; `painted.py p2` the tiles, `BRIEF3` for the places that are not masonry).
Sprites come back on alpha or a MAGENTA key (`pack.key_source` reads the key off the corners; blue still works). `art/p2sheet.py`
makes the before/after sheets (`scratchpad/style/phase2/`) and `qc` composites every sprite with the hero into its biome's dressed room
and runs `art-qc.py --style`. Codex batches: `art/prompts/p2_*.txt`, logs `art/logs/p2_*.log`, ~8 concurrent, ~10 min a batch.

1. **Heroes** (4 classes × man/woman/cat + the 4 default-look aliases, 16 portraits). Blind read test (`scratchpad/style/phase2/readtest/`:
   fighter-man, rogue-woman, ranger-cat at 1/4, 1/5 and 1/6.5 head, 18 panels at true device size on two floors, random facing and
   hurt; two fresh screenshot-only readers): class 12/12 at every proportion, look **5/12 · 7/12 · 11/12**, facing 10 · 12 · 12, hurt
   12 · 12 · 12 — overall **81 % · 90 % · 98 %**, so the long 1/6.5 figure ships (its BONE face patch and MIST blade rim carry the
   read; the chunky 1/4 figure's face and ears drowned in the dark kit). The three tested sprites are the shipped ones; the other nine
   were drawn against them. Every hero wears the BLOOD cloak (the rogue's retried once: 0.1 % blood in a frame). Style QC: every hero
   frame passes in the Warrens.
2. **Foes and bosses** (29 monsters, 2 summons, 6 bosses + their 6 death poses, the captive and the goblin bosses' portraits; the
   killer portraits re-cut by `tools/foe-portraits.py`). One pass, no retries: goblins INK-and-DUSK skin with BONE eyes (the old
   bright green gone), ghosts (wraith, spectral blade/hound, echo) in MIST and BONE, the Warlord a gaunt king in a BLOOD-and-INK
   mantle. Known weak: the bell sentinel's bell reads dark (not GILT), the acolyte bows less than briefed, the Foundry Master carries
   a lot of bronze. Style QC: every foe frame (with the hero, in the Warrens room) passes.
3. **The painted tile register** (7 places × 20 pieces + the Fens' own 7): `painted.py p2` with the place's tinted DUSK/MOON hexes
   in every prompt and `BRIEF3` for the places that are not masonry — the **Fens** a boardwalk of long planks over black water with a
   log palisade (its first pass came back as teal flagstones: the shared "flagstone" briefs had won), the **Burrows** packed ochre
   earth with timber shoring, the **Deep** unworked cave rock; the Warrens, Crypt, Foundry and Sanctum keep masonry. The D5 fork's
   two lanes read apart (earth vs planks over water). The 8-colour ramps (`make_tiles.style_ramp`, the fallback register, the decals,
   `atlas.json meta.palettes`, `palette.ts`) are now the palette with each place's tint; the foundry's orange remap is gone. Style QC:
   the Fens, the Burrows, the Deep and the Foundry never reached a moonlit highlight in a room frame (p99 L* 48–59), so
   `painted.convert` lifts their lit edges toward MIST (`MOON_LIFT`; the Burrows' earth floors kept calm, its ledges carry the moon)
   after one Codex retry each for the Fens and the Burrows. Every room passes, and all **413 sprite frames** (every hero and foe with
   the hero, in all 7 rooms) pass `art-qc.py --style`.
4. **UI**: 16 frames (bar, console, panel, tablets, tiles, well, the **BLOOD SEND gem** and a darker cracked CLOT danger gem,
   the death banner, gauge, stud, seal, the carved button, the scroll), 35 BONE ink icons, the death/report backdrops, pillar/brazier/
   candle, the boss shield + shards, the title key art (the camp's vista: the hero on the stair down, a moon shaft). CSS on the palette
   (a fork's pass: tokens `--ink … --gilt` in `:root`; `--acc` GILT for trim/text, the primary action BLOOD, `--hp` BLOOD, ok/info MIST,
   warn EMBER; ~900 literals moved by role; EMBER kept only where a flame is; the old `--ink` text token is `--ink-text`). Style QC
   on the walked screens: death, report and the second camp pass; the first camp sits at p99 L* 60 (bar 62).
5. **Pets, items, effects**: the pet portraits (rat, jackal, monkey, goblin; the pet sprites are the bestiary's), the item glyphs
   (potion, scroll, sword, breastplate, coins), the torch (EMBER, retried once: its flame read as a red stick), the BLOOD banner and
   blood decals, shrine, vault / open vault, nest, bones, moss, crack, rubble — the shared hue assets and ramp decals (`p2_items*`).
6. **The town** (docs/TOWN.md §7, enough for Cut 30's town v1): the dungeon mouth ×3 (cave · timber · gatehouse), campfire ×2
   frames, tent, supply crate, staked plot, scaffold; blacksmith, storehouse, kennel and bank × 3 looks (built · improved · grand);
   townsfolk (smith, merchant, child, carter), the mule cart, a dog, loot sacks ×2, a glowing chest, a BLOOD flag ×2 frames; a walk
   frame per class (`walk_<class>`, the default look); the town terrain as painted tiles (`envp_town_*` → `town_env_*`: grass ×4,
   dirt ×3 + edge, plaza ×2, water, cliff, fence, low wall, gate, bridge, two trees); the `!` rune marker (`icons/alert`) and the
   minimap / roster plaques (`frames/plaque_*`, 9-slice 64). All keyed sprites are `kind: "town"` in the manifest (art-qc treats an
   undrawn one as optional: the town scene draws a block + its icon). Nothing renders them yet — Cut 30 wires the scene.
7. **The look is the default** (`web/src/render/wash.ts`, `blit.ts`, `index.ts`, `tags.ts`): the wash pass on for FX > 0 (`?look=off`),
   reworked for on-palette art; moonlit grades, EMBER torches everywhere, a MIST hero light, moon pools; in-world plates on the palette.
   **Blind round 27** (JUICE §10.15, raters BG and BI against the style targets): **6.60** vs round 26's 6.84 (−0.24). Camp +0.35,
   desktop camp +0.30, bosses held; the dungeon watches fell (Warrens 5.80 → 5.30, Fens 6.00 → 5.10, scene 6.35 → 5.30): darker frame,
   the long hero reads small, the Fens' planks shimmer under the grain. Next, in order: calm the Fens floor (fewer stripes; no grain on
   world texels under motion), lift the watch's ambient a step and the hero's MIST light, lighten the scene insets, bevel the navy
   plaques, forecast bars back to a hot fill; then the camera-on-the-exchange blocker (needs the rule's target on the wire).
   Before/after sheets: `scratchpad/style/phase2/` (`batch1_heroes_sprites`, `batch2_foes`, `batch3_tiles`, `rooms7`,
   `phase2_before_after_phone`, `phase2_before_after_desktop`, `readtest/`).

### Gfx/UI eval (stopped on the rule again after round 26: 6.84, the best on the motion-aware harness — JUICE §10.14 has the blockers)
Rounds 24–26 since the resume: 6.74 · 6.75 · 6.84. Kept: carved EDIT + candle + desktop sheet (edit 7.45), two-pane scene, plaque slams,
slash arcs, desktop meters in the side column (d-watch 6.2), darker memory and warmer torches. The closer zoom was reverted. The floor
is the map watch (Warrens 5.8): the camera needs the rule's target on the wire to frame the encounter ahead of time.
Round 24: the carved EDIT + candle + desktop sheet kept (edit 7.2, d-edit 7.0); the closer zoom (80/58) reverted (every watch moment fell).
Earlier (stopped on the rule 2026-09-30 after round 23: 6.77 · 6.77; boss break 7.70, Fens 6.00; JUICE §10.12–10.13 has the blockers)
Round 23 (raters AY, AZ): 6.77 again (−0.06, +0.09, +0.00: three rounds under +0.2). The floor is the map watch (Warrens 5.8, desktop 6.0):
raters want a closer zoom (a PHONE_TEXELS call against the fights gates), a carved EDIT button and bigger desktop sheet, a side-by-side scene.
Round 22 (raters AW, AX): Fens props (7 Codex pieces) + a warmer pool, the boss break re-staged (shield above him, white-hot crack, arcs),
the map camera frames the seen floor, the vista grows under the phone scene. Round 23's changes are committed unscored (camera hold,
OPENED medallions, desktop meters 16 px, Fens fog + ghost wisps). The stop rule's count: 21 (−0.06), 22 (+0.09) — two rounds under +0.2.
Earlier:
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

- **Cut 30 — idle first: the hero climbs on his own** (`docs/CUT30.md`, contract 2026-09-30; after Cut 29
  merges). Eight parts: (1) the idle floor — the `Steady` school stance, drills at a boss's second meeting,
  scars, the quartermaster packing known counters (IDLE: D8 day 1, D23 by day 12, gold every day); (2)
  packages v1 — 4 stances, 6 tactics (today's cards), 4 temperaments (the trait core's mapped shapes: lexicon,
  fact learning, wake cards, ladder arrival, neutral bots, old-save mapping), levelling from runs; the editor
  becomes the pen, a late stage; (3) the town hub v1 (`docs/TOWN.md`: camp → blacksmith, storehouse, kennel,
  bank; walkers; the mouth = send; a three.js `town` scene with DOM targets); (4) the tracks panel; (5) oaths
  as a one-quest board with no stake, or deferred; (6) the replacement invariants (IDLE/PICKED/TUNED/RANDOM,
  1.5× · 1.5×, nothing required) with the exact AGENTS.md/PLAN.md edits, made at merge; (7) a control cohort on
  the pre-pivot build under `idle-hybrid` first (can run now), then cohort 25; (8) the owner checks each new
  system (agent readers only filter). Hold before Cut 31. The superseded traits draft is
  `docs/CUT30-traits-superseded.md`.
- **Cut 31 — to re-scope under the pivot**: parallel heroes (houses, tavern, expedition board, aggregated
  report) first; specialisation forks and weapon/armour properties (reach, cleave, stagger, bleed, pin; quiet,
  fireproof) return as packages/loot later, not as rows to write.
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
