# Cut 16 — What a night is

*Contract, 2026-09-24. Cohort 12 (build 238bd67): 69.2 · 66.5, α 0.859, mean 67.9. Three
cohorts down (73.8 → 72.4 → 68.7 → 67.9). `docs/PLATEAU.md` "Cohort 12": the four axes of the
core loop (decisions, failure, return, attribution) hold 0.8; everything around it holds 0.6.
Lapse-driven cuts have stopped paying. This cut changes what a night contains.*

## Design

### 1. The shallows are picked clean (core; tension, progression, return)

X: "no deaths in 20 runs, even with my return threshold at 10 %, so the tension went away";
W: 0 deaths in 16 offline runs; both nights banked the same floors every run.

- **Freshness.** Each depth has a freshness per lineage (`LineageState.picked: BTreeMap<u32,
  u32>`, banks and returns from that depth). A floor's gold and item drops scale by
  `0.8^picked` (floor 0.25), restored by one step per night the depth is not visited. The
  deepest floors the lineage has seen are always fresh; the shallows the safe set farms thin
  out, so a safe set's income falls night over night until the player pushes the bank row
  deeper.
- The report says it in a line (`D3 · picked clean`) and the forecast's `~$N` reads it.
- Gate: `DEFAULT yields 0 xp/gold` holds; EDITED banks ≥ 3 runs per 8 h holds; a test that a
  depth banked 10 times pays ≤ 30 % of a fresh one and recovers.

### 2. The heir chooses a class at the wake (core + client; expression, autonomy)

X: "I bought class: rogue and class: ranger and never found where to use them." Expression is
at 0.6 for eight cohorts; four classes exist and nobody plays the second.

- The wake offers the owned classes beside the traits (Cut 13 §2's chip): a chip per class
  with its one signature (`rogue · vanish`, `ranger · mark`, `caster · slow`). The pick is the
  heir's; the set's own rows are kept, and the class's verb chips join the editor.
- Each class's signature verb is on its chip and in the forecast's try line when it would
  move the reach (`try: vanish · +12%`).
- Gate: a test that the wake's class pick sticks to the run; screens.mjs: the chip row.

### 3. The Warrens split in two (core + render; surprise, aesthetic)

U: "D1–D6 look the same every run ('the Warrens')". W, X: the same floor card for eight floors.

- D1–4 stay the **Warrens**; D5–8 become the **Burrows**: their own palette (red clay, torch
  amber), their own floor card and biome fact, and their own monster weight (monkeys and
  goblin archers up, rats out). The Captain's floor (D5) opens the Burrows; the Warlord's (D8)
  closes it. No new sprites: a palette and a spawn table.
- Gate: `biome_for(5) == Burrows`; the twists gate (D3–10 ≥ 4 kinds) and the situation gates
  hold; a screenshot of D6 on the GPU harness in the Burrows palette.

### 4. A boss is a fight in phases (core + client; tension, feel, story)

X: "the Warlord fight looped for about 7 minutes … the same four callouts"; J: 4+ min at `fast`.
The Warlord now has finite reserves (b4c5363); the fight still has no shape.

- **Two phases.** Above half hp the Warlord fights behind his wall (rallies, shields). At half
  hp he **breaks**: `WARLORD BREAKS` (a beat), the wall's buffs drop, and he charges — faster,
  hitting harder, no more rallies. The counter row (`attack boss`) matters in phase one; a heal
  row matters in phase two.
- The watch shows the phase: the boss's hp bar in the HUD while he is in view (a thin bar under
  the hero's, his name), and the break as a beat.
- Gate: a test that the break happens once at half hp and ends the rallies; the bots hold
  (TRIVIAL never passes D8, COUNTERED ≥ D14, the counter lifts D9 reach ≥ 0.3); fights.mjs: the
  boss bar in view and the break beat.

## Gates

| Gate | Bar |
|---|---|
| A depth banked 10× pays ≤ 30 % of fresh; recovers night over night | test |
| The wake's class pick sticks; the chip row | test, screens.mjs |
| `biome_for(5..=8) == Burrows`; its palette on the GPU harness | test, screenshot |
| The Warlord breaks once at half hp; the boss bar; the break beat | test, fights.mjs |
| Bots hold: DEFAULT dies by D6, EDITED ≥ +15, RANDOM/PASSIVE lose, LEARNED ≤ 2, TRIVIAL never passes D8, COUNTERED ≥ D14, dice ≤ 5 %, stalls ≤ 1 % on DEFAULT and every cohort set, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 13: mean ≥ 75; tension and expression ≥ 0.8 from one rater; α ≥ 0.80 | two blind cards |

## Tracks

- **Core**: §1, §2 (wire: `Lineage.class_offer`, the pick), §3 (biome, spawn table, facts),
  §4 (phases), their tests; the heal-by-name fix (1951fe7) and the loop fixes (b4c5363) ship
  with it.
- **Client**: §2 chips, §3 palette + floor card, §4 boss bar and beat, their tests.
