# Cut 23 — Gold buys the next floor; every row answers

*Contract, 2026-09-25. Cohort 18 (build afa0eed): 71.4 · 66.5, α 0.897, mean 68.95 (−2.15).
`docs/PLATEAU.md` "Cohort 18".*

## Design

### 1. Gold has a deep use (core + client; progression, decisions, return)

Both: ~$2100 after the absence and nothing to buy.

- **The forge**: the camp sells the heir's **starting kit tiers** — weapon, armour, and a pack slot
  — each a ladder of 4–5 steps priced from the lineage (a step costs ~2–3 good nights of a
  mostly-home set; the first step ≤ one night). A bought step is permanent for the lineage (every
  heir starts with it) and a death does not lose it. Its forecast move shows on the step before
  purchase (`mail +1 · D9 +7 · $340`), from the same paired forecast as edits.
- **The row slot** stays, priced to sit on the same ladder (not a $1650 wall).
- Nothing here outscales policy: the bot gates hold with every step bought (FULL walls, TRIVIAL
  never passes D8, LEARNED ≤ 2 floors, EDITED ≥ +15 over DEFAULT with DEFAULT given the same kit).
- Gate: a metrics row — on each cohort set, 8 h of play leaves ≥ 1 affordable forge step after
  the absence and the next step ≤ 3 nights away; a new bot `KITTED` (DEFAULT + all steps) still
  dies by D8; tests, ui.mjs.

### 2. Deaths vary; the forecast's low end is honest (core; failure, surprise, clarity)

AJ: every death `return too late` at 1–2 HP; `death 0%` then death.

- Measure the death-cause mix on the cohort sets (verdict row × killer); if one cause is > 50 %
  of a set's deaths, find the engine reason (a heal row never reached on the walk; a return that
  starts too late because hp is read before the blow; foes that always follow up the stairs) and
  fix it or vary it with content (ambushes on the way home, not every floor; different killers).
- The forecast never prints `0%` for a share it sampled at 0 of N: `<N%` with N from its sims
  (`death <2%`).
- Gate: measured mix before/after, reported; a test for the low-end label.

### 3. Every row answers (core + client; clarity, expression, attribution)

AJ: `foe: gas → throw unknown` fired 0/164, no word; AI: `read summon ally` never fired vs Drix.

- **A row's why-not**: the core keeps, per row, the most common reason it didn't fire when its
  conds matched (verb blocked: no item, no target, out of range, shadowed, guard) and how often its
  conds never matched at all (`gas seen 0×`). The tablet shows it on tap (`0/164 · no gas met` or
  `blocked · no scroll`), within the copy budget.
- **Every `✗` callout and shouted word has a reason on tap** (`read ✗ no use` → `nothing to
  learn`, `IT SHELLS` → the foe's trait name); the patch's `reach D5 −88` reads `reach −88%` or
  loses its number when survival is the point (one form).
- **Cards do what a row can't**: each paid card carries something outside the typed vocabulary
  (a verb, a cond, or a pre-bound target) or it is free; list which and why.
- Gate: tests; ui.mjs; copy-lint.

### 4. The editor gets out of the way (client; feel, autonomy)

- The option sheet never covers the chips it edits; taps outside close it without acting.
- A confirm never undoes itself; the supply × is ≥ 32 px; buying never moves the layout under the
  finger (reserve the space).
- Forecast settle (first pass) ≤ 1.2 s in real wasm on the cohort build; report the measured time.
- Gate: ui.mjs, clarity.mjs.

### 5. The leash is not the thief's favourite (core; tension, fairness)

AI: the leash stolen nearly every run. A leash (and any pet gear) steals only when the pack holds
nothing else, like a packed supply; measure leash thefts per run on the cohort sets (target ≤ 0.1).

## Gates

| Gate | Bar |
|---|---|
| Forge step affordable after an absence; next ≤ 3 nights; KITTED dies by D8; all bot gates with every step | metrics, tests |
| Death-cause mix reported; `<N%` low end | measured, test |
| Row why-not on tap; `✗` reasons; cards justified | tests, ui.mjs |
| Editor: sheet, confirm, ×, no jump; first paint ≤ 1.2 s real wasm | ui.mjs, clarity.mjs |
| Leash thefts ≤ 0.1 per run | measured |
| Bots, dice, stalls, dances, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Cohort 19: mean ≥ 76; progression 0.8 from both; α ≥ 0.80 | two blind cards |

## Deviations (core, recorded)

- The weapon ladder has 3 steps (class arm +1…+3), not 4–5: a +4 step broke the D33 wall (kitted
  FULL−D33 25–27/30). Armour tops out at mail +1: mail +2 made lurkers harmless.
- The forge gate counts mostly-home sets (like Cut 22's gold gate); T, AE, U print ungated.
- The walls were thin once kitted: the Lurker Queen takes half damage while a called lurker lives
  (called lurkers bite +2, up to 8); the Mirror King heals twice what he reflects. Unkitted walls
  stay ~29–30/30.
- §2: deaths still mostly come after the turn for home (50–100 % on return sets) — reported, not
  gated; no single walk × killer cause exceeds half on any set (0/21).
- Bought-supply thefts rose 0.001 → 0.019 per send (leash moved behind supplies); Cut 22's ≤ 0.1
  holds.
