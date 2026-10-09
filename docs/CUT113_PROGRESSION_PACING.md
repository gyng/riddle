# Cut 113 — progression and pacing (queued)

*2026-10-08, owner: "queue up progression and pacing improvements". Evidence: three blind cohorts
(ad71e72 56.7 · c4705f9 64.7 · 1fb7786 67.4, α .835). Pacing (w 9.5) and return (w 9.5) sit at 0.6
on most cards; tension, mastery and decisions at 0.6 name the same causes. Runs after Cut 112
(apprentice standing order, the pen at the Mother, cadence slottable) and take control.*

## Queue (highest expected gain first)

1. **Early floors earn their place.** B: "most of each run is pre-resolved in a `D1–32 · 100%` chip,
   so stakes live in the last floors only". The guide starts sends at the stone nearest the frontier
   band the set clears ≥ 95 % (not four floors under the record); the fold line keeps its finds and
   thefts. Gate: watched seconds on ≥ 95 % floors per run ↓; gold/hr not below today's (metrics).
2. **Forge choices, not a ladder.** A, B: "forge/upgrades strictly ordered". Each forge tier offers
   two steps with different trades (weapon: damage · aim; armour: plate · pace), priced alike; the
   apprentice follows a standing order (Cut 112). Gate: neither branch dominates on every wall
   (paired panel), PICKED and IDLE bars unchanged.
3. **Each return carries a decision.** A: 20m return "1 RUNS · $27"; B: "20m and 4h returns felt
   similar"; A: the 8h return "felt like losses I didn't choose". An absence report offers one pick of
   three (a tactic level, a Legacy point, a forge step at cost) sized by the absence, never lost if
   unpicked (nothing punishes absence). Gate: every check-in in the dayplayer has ≥ 1 pick; copy
   budgets.
4. **The late bands breathe.** B cleared D34 in ~57 active min + ~12 h away. Late walls (Lich,
   Foundry Master, Lurker Queen, King) take a learned counter *and* a kit threshold, and the clear
   rolls straight into Ascension 1 with one tap (B: "Ascension asked for more setup and was never
   sent"). Gate: PICKED median D33 in ≥ 2 days; IDLE never slays the King in a fortnight (holds);
   the clear's next step is one tap.
5. **Walls with an answer the player can find.** Hunger (B: "starving −56, a wall I never learned to
   answer"): a ration supply and a lantern line on the hunger callout; the cheapest-lever for a drain
   death names it. Gate: Fens drain deaths ≤ today's with the answer bought; death lever names the
   drain's counter.
6. **More than one road.** Autonomy 0.3–0.6: "route fixed after the D5 fork". Open the D9 and D14 forks
   (`descent::OPEN_FORKS`) with both lanes viable. Gate: Cut 26's no-set-dominates-both-lanes bar.
7. **Less admin per bloodline.** B: "every return asked forge + legacy + tactics + unlock + worker
   taps" ×3 bloodlines. A return's choices apply to the bloodline shown, with one "same for all"
   toggle per order. Gate: taps per check-in ↓ (ui QA script).

Each item lands with its numeric gate, the routine full gate table without a lowered threshold, the
full client suite, then a fresh blind pair.

## §4 measured — the late bands breathe (core, 2026-10-08)

Evidence: blind 5331f40 (68.3 / 59.2): both raters cleared D34 inside ~70 active min + ~12.5 h away
(A: 27 runs, sword +5 · plate at the King); B cleared Ascension 1 in one run. The counter alone broke
each late wall; the forge's upper steps were bought on day 1.

**Before** (HEAD 5331f40+, `dayplayer --routine --seeds 8 --tuned-seeds 8`, median hours / King-kill day):
PICKED D23 24 · D28 24 · D29 120 · D33 136 · King day 6. TUNED D23 24 · D28 24 · D29 36 · D33 40 · King
day 2. IDLE D23 128 · D28 168, King 0/8. Probes: Queen 60→150 hp alone moved TUNED's D29 34→60 h but not
its King (killed the day D33 was reached even at 200 hp: with the counter he never touched the hero's
forged armour). Damage through the armour is what makes the kit a threshold.

**Changed** (content numbers only, `defs.rs`, `endgame.rs`):
Foundry Master hp 42→52; Lurker Queen hp 60→120, atk 4–7→7–11; Mirror King hp 80→220, atk 3–6→13–19;
Sanctum wardens 30→34, echoes 16→18, sentinels 26→30. Numbered descents add +5 % HP / +2 % damage a
tier (was 8 / 4) since tier 0's own late walls rose. Deep-band boosts (lurker 12, troll 40) were tried
and dropped: they walled every numbered tier on the Queen's swarm.

**After** (same harness): PICKED D23 24 · D28 28 · D29 136 · D33 136 · King day 9 (6–14). TUNED D23 24 ·
D28 28 · D29 48 · D33 60 · King day 4 (3–7). IDLE unchanged to D23 (median day 5.3, stall 4 d), King 0/8.
All dayplayer bars PASS (PICKED/IDLE 1.50 · 2.39 · 5.33, TUNED/PICKED at D33 2.43, RANDOM 8/8 · 8/8).

**Kit threshold** (`examples/descent_check SAVE 0 --kit …`: the earned seed-3 King save, every counter
known, Legacy full, replayed from D1 at tier 0, 4 seeds): full kit 16–48 h; sword +5 · plate (the rater's)
24–64 h; +3 · mail +1 40–64 h (the kit now buys days; the King kills 1–28 heirs per clear).

**Ascension 1** (same save, untuned set, 8 h check-ins to the next clear): before 8–16 h (11–22 runs);
after 16–32 h (25–65 runs). Tier 3 3/4 within 14 days; **tier 2 1/4** — its Swift affix on the
13–19 King is a cliff for an untuned set (before: tiers 1–5 all cleared). Recorded, not gated
(Cut 31: higher-tier build viability is not certified); a tuned set or the King's damage is the lever.

Not measured: a human's day. The raters ran ~3× faster than TUNED; on the same ratio the first King
kill lands day 2 (TUNED day 4). The next blind pair is the check.

**Gate** (`node tools/gates.mjs --full --fresh`, source ffd6335885f2ffbb): metrics 8/8 PASS, qa 10 seeds PASS,
dayplayer all PASS except the recorded deviation *Workers pay the away player* (docs/CUT110 §4):
mean best 27.71 vs 28.04 (was 28.25 vs 28.36). Rows that moved: TUNED/PICKED at D33 1.55 → 2.62, at D29
1.66 → 2.62; Steady deaths a send 5 → 6 % (bold 88 → 89 %); stance walls won 7 → 6 (Guarded 4 → 3, still
≥ 1 each); workers-daily 30.18 vs 29.96 → 29.25 vs 29.00 (PASS); the tree also held the concurrent oath.rs/engine.rs edits; IDLE stall 3 d, King 0/2.
`cargo test --profile fast`: 703, the 307dbed send hash re-recorded (f22db4c57141f4d0 → ea30535a6c429390;
the tree with only defs.rs at HEAD hashes to f22db4c57141f4d0).

### §4b — the Queen had no answer below a full kit (blind b8dd77c, 2026-10-09)

Evidence: rater A lost five heirs to the Lurker Queen at D28; every offered fix replayed 0/12. Her brood hunts by
sound and she calls it before she comes in view, so `foe: boss → read silence` fired with up to eight called lurkers
already on the hero, her blows halved and her mending running behind them.

Measured (temp harness: A's exported set, fighter L10, every fact, a fresh hero on D28 with heal ×2 + silence,
30 seeds × 12 sends; *won* = Queen slain / Queen engaged): at 120 hp (7–11) kit weapon 1 · armour 3 (A's) 1/50 (2 %),
weapon 3 · armour 4 1/56 (2 %), full kit 30/127 (24 %) — a wall with no answer below the full forge. Hp/damage alone
barely moved it (median Queen untouched at the end: the brood kills first).

**Changed.** Reading silence loses her called brood (it fades; the floor's wild lurkers stay — `ai.rs`); Lurker Queen
hp 120 → 100, atk 7–11 → 5–8 (`defs.rs`). Same harness: A's kit 10 %, weapon 3 · armour 4 17 %, full kit 57 % of
engaged fights won — the counter answers, the kit still buys it (×5).

**Pacing** (`dayplayer --routine --bots idle,picked,tuned --seeds 8 --tuned-seeds 8`, final tree incl. the b8dd77c
corridor-hold fix and Cut 115; King-kill day per seed): PICKED 7 · 7 · 6 · 7 · 8 · 5 · 8 · 4 → median 7 (before Cut 113: 6;
Cut 113: 9); TUNED 3 · 2 · 3 · 3 · 3 · 3 · 3 · 3 → median 3 (before: 2; Cut 113: 4); IDLE King 0/8. Rejected: 90 hp (5–8) —
PICKED median 5 < 6; 100 hp (6–9) A's kit 4 %; 110 hp (6–10) mid kit 9 %. All dayplayer bars PASS.

**Gate** (`node tools/gates.mjs --full --fresh`, combined tree with Cut 115): metrics, qa (10 seeds) and dayplayer all PASS.
`cargo test --profile fast`: 727 pass; the 307dbed hash re-recorded ea30535a6c429390 → e9c001509e3a6fc0 (reason in the test).

## §2 measured — forge choices, not a ladder (core + client, 2026-10-09)

**Changed.** Each weapon and armour tier offers two steps priced alike (`kit::BRANCHES`): weapon `aim` (to-hit:
+4 % a step for the first three aim steps, +3 % past them, at most 100 %) · `edge` (the blow: +1 in the first three
tiers, `EARLY_EDGE`, +2 from the fourth); armour `plate` (the next piece of the old ladder) · `pace` (speed in tenths,
`PACE_TENTHS` 8 · 7 · 6 · 5 · 5 · 4 · 4 %, a fractional point paid a tick at a time). Each tier's default is the old
ladder (aim to +3, then the edge; plate throughout), so a lineage that never chooses forges exactly what it did
(`LineageState.kit_alt` empty, no `Item.forged`, no `Hero.pace`; the send hash and every test unchanged). Wire:
`KitLadder.branches` (`KitBranch {id, label, default, lean}`), `KitStep.branch`; `buyKit("weapon:edge")`. The apprentice
follows `kit_lean` — the player's last off-default pick on that ladder (a default picked by hand clears it), else the
default. The forge sheet shows a tier's two steps as two buttons (`aim +4% $300` · `edge +1 $300`); the pack keeps one.

**Gate** (`examples/forge_branch 16 96`: IDLE's wall snapshots, the record at the wall, sends from the deepest lit stone;
value = share past the wall − ¼ deaths; paired, the same seeds both ways; median of 16 seeds). *Last tier* is the
per-tier choice (k − 1 default tiers, then each branch); *whole lean* is k tiers all one way.

| ladder · view | first branch ahead at | second branch ahead at |
|---|---|---|
| weapon · last tier (edge − aim) | k1–3: D13, D23 · k5–6: D28 | k1–3: D8, D18, D28 · k4–6: D8–D23 |
| weapon · whole lean | k1–2: D13/D23 · k5–6: D28 (2/16 seeds edge) | k2–6: D13, D18, D23 |
| armour · last tier (pace − plate) | k1: D8, D13, D23 · k2: D18, D23, D28 · k3: D18 · k5: D23 · k6–7: D23/D28 | every tier ahead somewhere (k4: D8–D28 ≥ 0, D23 0.000 with 5+/6−) |
| armour · whole lean | k2–4: D18 (−0.03…−0.07) · D8/D13/D23 at k1–2 | k3+: D28 (11–16/16 seeds) |

On every tier each branch is ahead at some wall, but for two ties: weapon k4 (D28 0.000, 7+/6−) and armour k4 (D23 0.000,
5+/6−); and on the whole lean pace's 5th–7th steps lead every wall's median except D18's 3+/3− split. Two tunings got
here: a full +2 edge in the aim's tiers out-reached aim at every wall (v1: k1 +0.003…+0.016, edge 7/8 seeds at D8); a
full speed point a pace step out-reached plate at every wall from the fifth (v2: k5–7 +0.03…+0.42). Recorded, not gated
(a diagnostic over IDLE's snapshots; D18 has 6 seeds, D13 10).

**Bars**: no bot picks a branch or a return pick, so the dayplayer's games are the old ones (cargo tests incl. the send
hash unchanged). `node tools/gates.mjs --full --rows return-pick,idle-d8,idle-d13,idle-d23,idle-stall,idle-king,picked-idle,outpace`
(16 seeds, fresh cache): IDLE D8 16/16 · D13 day 1.3 · D23 16/16, day 4.8 · stall 4 d · King 0/16 · PICKED/IDLE 2.00 · 2.39 ·
3.42 · never out-paced 100 % · return pick 672/672 check-ins — all PASS (targeted, not the gate).

## §3 measured — each return carries a decision (core + client, 2026-10-09)

**Changed.** `returns.rs`: every return of ≥ 10 min opens a pick (`LineageState.return_pick`: minutes, the absence that
opened it); an untaken pick waits and the next return adds its minutes (never lost, nothing punishes absence). Size by
minutes: 20 m 1 · 1 h 2 · 4 h 3 · 8 h 4 · 16 h 5 (4 h and 8 h now differ). Three offers, seeded from the lineage seed and
the absence (never the game's dice): **drill** +3/6/10/16/24 runs on a worn package below L5 · **Legacy** +2/4/6/9/12 ·
**forge** the next step of a seeded ladder at −10/20/30/40/50 % on the apprentice's branch (only once the blacksmith
stands; `marks` +◆1–3 stands in for an empty drill or forge). Wire: `ReturnReport.pick`, `Lineage.return_pick`
(`ReturnPick {minutes, size, offers}`), `takeReturnPick(id)` (wasm, native bridge regenerated). Client
(`web/src/ui/return-pick.ts`): three chunky `tile`s on the report (under the summary); on the camp one `pick` tile under the town
(below the fold: runsui's ≤ 12 controls above it hold) opens the three in a sheet; gone once taken; forge sheet branch buttons (`forge.ts`).

**Gate**: dayplayer row `return-pick` — *Every IDLE check-in offers a return pick* — PASS 16/16 seeds (IDLE takes none).
Rust: `tests_cut113.rs` (branches priced alike, default = old ladder, lean; pick sized, seeded, waits and grows, taking
closes it; an untaken pick never moves the game). Client: `web/tests/return-pick.mjs` (real wasm: 30 m report shows three
tiles, untaken waits on the camp as one tile opening the three, Legacy taken pays +2, forge tiers offer two steps). Copy-lint clean.
