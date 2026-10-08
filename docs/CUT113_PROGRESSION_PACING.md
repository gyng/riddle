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
