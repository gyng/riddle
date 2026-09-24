# Cut 20 — No taxes; the watch moves; the night has stakes

*Contract, 2026-09-25. Cohort 15 (build fba365b): 74.6 · 74.1, α 0.969, mean 74.35 (+4.85), both
"fun". Tension, feel, expression and pacing hold 0.6 from both. `docs/PLATEAU.md` "Cohort 15".*

## Design

### 1. Thieves are a puzzle, not a tax (core; tension, decisions)

AC: "monkeys and dens steal the $40 potions I buy in nearly every run"; AD: "the den wakes …
on nearly every run and took the same gear each time".

- **A den wakes on ≤ 1 floor in 3** of the D3–5 band for every lineage (not only one that lost
  to it), deterministic from the run seed; the first den a lineage meets always wakes (it is the
  lesson).
- **A thief steals once per run**: after a theft, the run's other thieves flee instead of
  stealing (the carried item is the stake of that fight).
- **A killed thief drops what it stole** (it already should — verify) and the recovered item is
  a note (`got the heal back`).
- Gate: the den gates hold (met by D6 ≥ 90 %, DEFAULT passes ≤ 20 %, thief guard cuts snatches);
  a test that a run has ≤ 1 theft; measured thefts per run on the cohort sets, before/after.

### 2. A pet is a companion, not a consumable (core; expression, story)

AD: 8 pets lost in a session.

- **A pet flees at 30 % hp** (it runs to the hero's side, out of the fight) and **heals between
  floors** (to 60 % on each descent); it dies only to a blow that would kill it from above 30 %.
- The party sheet: a tap selects, a second tap does not dismiss (a `×` does).
- Gate: measured pet deaths per run before/after on a tamed lineage (target: halved or better);
  PETS bot bar holds.

### 3. The watch moves (client; feel, pacing)

AC, AD: "`fast` barely faster than 1×"; AD: "early runs at 1× too short to follow".

- **`fast` targets ≤ 40 % of `fights`'s wall time** over a whole run (not merely ≤): dead
  stretches at 32× from the first tick (no ramp), fights at 4×, beats as now.
- **`fights` at 1× on D1–D3** plays fights at 1.5× and travel at 8× (early runs were 22–67 s of
  fast travel with little to see; the fight is what to follow).
- The forecast's first pass paints within 1 s of an edit (50 sims), the refine after (the 4–8 s
  wait is the refine blocking the paint — verify).
- Gate: fights.mjs — on one world, `fast` ≤ 0.4 × `fights`; clarity.mjs — the forecast's first
  paint ≤ 1.2 s after an edit (fake engine timing + a real-wasm measurement reported).

### 4. Money lines tell the truth (client + core; clarity, tension)

AC: `carry $78 · keeps $78 · bank R4` then died $0; a silent repeat charge on death.

- The stake line shows **what a death keeps** beside what the exit row keeps: `carry $78 · bank
  keeps $78 · death $0`; `keeps` alone never shows while death would keep 0.
- A repeat re-pack after a death is a line on the death screen's gold (`repeat −$80`).
- Gate: ui.mjs; qa.rs (a stake's death keep equals the death tier's share).

### 5. The second half has stakes (core; tension, return)

AC: "after the absence 15 of 16 runs banked, so the second half had little at stake".

- **The deep calls**: each night the lineage's best depth + 2 becomes a *bounty floor* — its
  gold ×2 and one guaranteed item of the lineage's next tier; the forecast shows it (`D12 ×2`),
  so a safe set that banks at D10 sees what it leaves on the table. It moves every night.
- Gate: a test; the report names the bounty (`bounty D12 · missed` / `taken $412`); bots hold.

## Gates

| Gate | Bar |
|---|---|
| ≤ 1 theft per run; dens wake ≤ 1 in 3 floors; den gates hold | tests, gate table |
| Pet deaths per run halved on a tamed lineage; PETS bar holds | measured, gate table |
| `fast` ≤ 0.4 × `fights` wall time; forecast first paint ≤ 1.2 s | fights.mjs, clarity.mjs |
| Stake shows the death keep; repeat line on death | ui.mjs, qa.rs |
| Bounty floor each night, forecast and report | tests, ui.mjs |
| Bots, dice, stalls on every cohort set | `node tools/gates.mjs --full` |
| Cohort 16: mean ≥ 78; tension or pacing ≥ 0.8 from one rater; α ≥ 0.80 | two blind cards |
