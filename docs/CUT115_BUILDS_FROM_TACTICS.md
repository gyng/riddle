# Cut 115 — builds from tactics

*2026-10-09. Seven blind cohorts, reliable (α .81–.91), plateau 64–69. Expression gates 6 of the last
12 cards at 0.3: "my own rows fired in 0 of 7 runs", "chores 67 % · tactic 25 %", "Steady + boss focus —
little felt mine", "the fixes did the learning". Owner (2026-10-09): **builds from tactics** — the
player's tuning stays tactics (not the pen), and the picks must add up to a build that is theirs.*

## 1. A build is the player's picks, named and credited

- **The build's name** from what the player wore (stance + the tactics and their variants + temperament):
  `Guarded · corridor (at two) · boss focus (boss first) · skittish` → a two-word title from its two
  strongest picks (`Guarded skirmisher`), on the hero card, the report head, the chronicle line.
- **Credit**: every compiled row carries who chose it — `picked` (a package the player equipped or a
  variant they set), `taught` (a drill, a death fix applied), `chore`. The report's decision share reads
  `picked 61 % · taught 9 % · chores 30 %`, and a death's deciding row names whether it was the player's
  pick. Gate: the share is computed in core from row origins (no new rows), on the wire, and a test pins
  that an equipped tactic's rows count as `picked`.

## 2. Variants are real choices from the first level

- A tactic's variant pick is available when it is equipped (L1), not only at L3, and the two variants
  differ in the tactic's **main** row (not only an extra row): e.g. corridor fighting `at three · at two`,
  boss focus `summons first · boss first`, kite archers `hunt · cover`, pack break `to corridor · pick
  the weak`, gas step `step away · burn`, thief guard `chase · swat close`. Gate: on the paired panel,
  each tactic's two variants differ by ≥ 10 points of past/bank/death at ≥ 1 wall; neither variant
  dominates at every wall (16 seeds).

## 3. Compare says which wall each pick is for

- `compare outcomes` rates tactics and their variants with paired sends and names, per pick, the wall it
  wins (`pack break · pick the weak — better at D8 Warlord · worse at D28 Queen`), not a global `close`.

## 4. Fixes as picks

- A death's fix offers, where one exists, a **tactic or variant** that answers the death (`try: gas step ·
  burn`) before a raw row; an applied fix row is credited `taught`, not `picked`.

Gates: each item's focused test; IDLE/PICKED/TUNED bars without a lowered threshold (IDLE picks
nothing, so its games are unchanged); the full client suite; then a fresh blind pair.

---

## Implemented (2026-10-09) — what shipped, and the measured results

Owner addendum (2026-10-09, via the coordinator): "be creative, feel free to simplify or new mechanic". Adopted: **synergies** —
a named pair of worn picks with one small effect only that pair has; the build's name comes from it. Nothing was cut: the
admin the cohorts named ("the fixes did the learning", "little felt mine") is answered by credit and by the variants, not by
removing a system.

### §1 Build name, synergies, credit (core + wire + client)

- `packages::build_name`: the synergy's name when a pair forms one, else `<Stance> <tactic noun>` (`Guarded slayer`), else
  `<Stance> <temperament noun>` for a picked temperament; **none while nothing is the player's** (Steady, no tactic, the wake's
  draw — IDLE). On the wire as `Packages.build {name, synergy, effect, picks}` (picks carry the variant: `corridor fighting · at
  three`) and `HeroSlot.build`; shown in the tactics panel's head, the report's head (`Bulwark held D23` / `Guarded slayer · D14`),
  the hero row, and the chronicle line (core, only when a build exists — IDLE's lines are unchanged).
- `packages::SYNERGIES` (7, each holds a tactic, so IDLE plays none): Bulwark (Guarded + corridor fighting: −1 melee taken in a
  corridor) · Duelist (Bold + boss focus: +2 one on one) · Marksman (Hunter + kite archers: +3 on a fresh foe) · Ghost (skittish +
  kite archers: −2 at range, −1 when under 35 %) · Scavenger (light hands + pack break: +2 to packs) · Iron lungs (iron gut + gas
  step: −2 gas and poison) · Warden (Steady + thief guard: −2 from telegraphed blows). Deterministic, in `turn::damage_hero` /
  `damage_monster`, read off the compiled rows' origins (`packages::build_mask`, one pass; a literal set plays none); the first time
  a run plays one it is called out by name (`Bulwark`).
- Credit: `Meter.origins` counts rule fires by the firing row's origin (core, no new rows); `packages::credit` maps an origin to
  `picked` (an equipped tactic or a non-school stance, a picked temperament, a set variant, a pen row) · `taught` (a drill, a
  `patch`, a tactic a death's fix put on) · `default` (Steady, the wake's draw, a trait's step) · `chores`. `MeterWire.credit` on
  every metered surface; the report reads `picked 61% · taught 9% · chores 30%`; `Death.credit` names the deciding row's credit
  (`corridor fighting · to corridor vs 2+ · picked`).

### §2 Variants from L1, distinct main rows

`set_variant` is open from L1 (owned). The first variant leads with the card (its L3 row unchanged, so PICKED/TUNED — which never
set a variant — play as before); the second leads with its own row(s) ahead of the card, a **leading** row compiled ahead of the
stance's guard rows (`tactic_lead_rows`):

| tactic | first | second (its main row) |
|---|---|---|
| boss focus | summons first | boss first: `boss, hp>20 → attack boss` (leads) · `hp<70 → rest` |
| corridor fighting | at three | at two: `foes≥3, hp<50 → return` · `foes≥2 → to corridor` |
| kite archers | hunt | fall back: `archer, hp<50 → return` |
| thief guard | chase | head home: `thief, hp<70 → return` |
| gas step | step away | wade in: `gas → throw fire` · `gas, hp>25 → attack gas` (both lead) |
| pack break | to corridor | stand: `foes≥2, adj≥1, hp>20 → attack weakest` (leads) |

Measured — `examples/builds.rs 16 48 variants` (IDLE's wall snapshots, 16 seeds — D8 16, D13 9, D18 4, D23 13, D28 16 —, sends
from the deepest lit stone under the wall, record at the wall, drills revoked, 48 paired sends; Δ = second − first in points of
past · bank · death):

```
boss focus   D8 +9·−11·+8  D13 +0·−9·+3   D18 +0·−9·+5    D23 +2·−3·−0   D28 +0·−4·+5    PASS
corridor     D8 +0·−10·−5  D13 −1·−20·−2  D18 −9·−32·−10  D23 −1·−39·−4  D28 −0·−27·−12  PASS
kite         D8 +0·−12·−5  D13 −1·−12·−4  D18 −0·−4·−1    D23 −0·−1·−1   D28 +0·−1·−1    PASS
thief guard  D8 −0·−15·−3  D13 +0·−7·−0   D18 −4·−11·−2   D23 −2·−39·−4  D28 −0·−19·−3   PASS
gas step     D8 +0·−1·−0   D13 +2·+0·+2   D18 −2·+12·−5   D23 −1·−0·+0   D28 +0·−0·−1    PASS
pack break   D8 +0·−6·−2   D13 +0·−11·+0  D18 −2·−8·+5    D23 −0·−22·+2  D28 +0·−12·+1   PASS
```

Every tactic's variants differ by ≥ 10 points at ≥ 1 wall. **Dominance** is read Pareto (a variant dominates when it is no worse by
more than 2 points on past, bank and death at every wall): each pair trades — boss first passes the Warlord 9 points more and
dies 8 more; the "home" variants bank less and die less. (Deviation, recorded: the camp score's single number, printed beside each
line, favours the first variant at most walls for four tactics — it weighs a bank at 0.2 and a death at 0.2, so a variant that
trades haul for safety always reads worse on it; the Pareto read is the "neither dominates" this section asks.) Three probe designs
were tried and rejected on the way: rows behind the card (≤ 5 points anywhere: the card already does them), a 25 %-seen dive
(dominated: +28 deaths), corridor at two ahead of the heal (dominated: +25 deaths).

Synergies — `examples/builds.rs 16 48 synergies` (each pair worn, its effect on − off, points of the camp score; the pair's total
in brackets):

```
Bulwark     D8 +4.2  D13 +6.7  D18 +10.0  D23 +7.1  D28 +1.1   PASS (pays ≥ 3)
Duelist     D8 +6.0  D13 +5.7  D18 +2.7   D23 +9.5  D28 +2.9   PASS
Marksman    D8 +3.7  D13 +5.2  D18 +6.7   D23 +7.1  D28 +3.3   PASS
Ghost       D8 +5.1  D13 +2.4  D18 +5.7   D23 +1.4  D28 +1.6   PASS
Scavenger   D8 +1.1  D13 +1.0  D18 +3.5   D23 +0.4  D28 +0.4   PASS
Iron lungs  D8 +1.2  D13 +4.4  D18 +6.1   D23 +2.5  D28 +0.3   PASS
Warden      D8 +7.0  D13 +1.9  D18 +0.6   D23 +1.9  D28 +0.8   PASS
pays most:  D8 Warden · D13 Bulwark · D18 Bulwark · D23 Duelist · D28 Marksman   PASS (none everywhere)
```

Recorded: the best build **total** at every wall is Bulwark — its stance, Guarded, is the strongest stance from the stones (it was
before this cut; the effect is +1–10 of it). "None dominates" is read on what the synergy adds, not on the stance under it.

### §3 Compare per wall

`options_for` (the client's chooser) prices a worn tactic's other variant (`id#v` → the `variant` move) and reads every move at the
walls the lineage has met (`compare_walls`: each lit stone's next band boss at or under the record's next floor, deepest three),
paired from the stone (`PkgOption.walls`). The client names the walls whose split clears the sign test (≤ 10 %): `better at D8
Warlord · worse at D28 Queen`, under the alternative's chip and beside the variant chip. PICKED's `options()` reads no walls (its
cost and choices unchanged).

### §4 Fixes as picks

`packages::fix_pick` maps the killer's tags (a boss's own counter first: reflection, mirror, blind, summoner; then ranged, gas,
thief, pack, boss) to an arrived tactic and variant, when a slot is open and it is not worn so. `Death.pick` carries it; before the
pen it is the death's lever (`try · kite archers · hunt`, ahead of a purchase), after it a `try` tablet above the patches.
`take_fix` wears it (first open slot, else the last), sets the variant and marks it `taught`; the player's own equip or variant
pick makes it `picked` again. Patch rows (`patch`) and drills were already `taught`.

### Gates

- Rust: `tests_cut115.rs` (7: main rows differ from L1 and a committing variant leads the guard; credit classes, shares and a
  real absence's tactic fires credited `picked`; IDLE wears no build and plays no synergy; a pair names Bulwark and sets its bit,
  literal sets none, names ≤ 2 words; Iron lungs −2 and its callout; fix picks, lever, `take_fix` → taught → re-picked;
  compare's variant move and wall order) and `a_tactic_variant_is_the_players_pick_from_l1` (was `…_from_l3`).
- Client: `web/tests/cut115.mjs` (panel: real wasm on 307dbed — the Bulwark head, the variant chips at L1, the compare's price
  beside the other variant; words: credit line, build head, a death's credit, walls line).
- The 307dbed send hash: unchanged by this cut (verified on a tree with only Cut 115's changes; the concurrent corridor/Queen
  work moves it separately).
