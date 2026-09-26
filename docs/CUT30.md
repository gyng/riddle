# Cut 30 — Heirs worth writing for

*Contract draft, 2026-09-27. Design: `docs/TRAITS.md`; prototype `examples/traits_proto.rs` (numbers in
TRAITS.md §6). Evidence: cohort 23 AU "traits can backfire invisibly (curious read a recall scroll and
wasted a $152 passage)"; cohort 9 "R4 retreat — brave held"; expression and autonomy 0.6 on every
plateau card ("the facts pick the row, I only order them"; "every run the same D13 bank").*

## Design

### 1. Traits are conditions on the hero, never actions (core; attribution, failure, clarity)

- The four temperaments' overrides leave `turn::choose_and_act` (the cowardly retreat, the brave hold,
  the curious use, the greedy grab, `trait_floor`, `trait_last`, `cowardly_streak`). A trait changes
  what a verb does or what a condition sees; only rows and chores choose.
- `hero::Trait` becomes data: `TraitShape { when, gift, cost, twist, tier }` (TRAITS.md §2) with
  `Trait::None` the neutral heir; `Lineage.traits: { blood, born }`; old saves map `cowardly` →
  `skittish`, `brave` → `unbowed`, `greedy` → `light hands`, `curious` → `iron gut`.
- The gift is live while its `when` holds (evaluated at the hero's action); the cost is always on.
  `Hero::atk`, `blunt`, `speed`, the regen tick and `Run::vision` read the live gift; nothing else does.
- Gate: tests — no `acting_row == -1` from a trait over 30 seeds × 8 h; a live gift turn always has a
  trace mark; old saves load with the mapped shape; the replay hash covers the traits.

### 2. The generator and its table (core; expression, surprise)

- Parts: 6 `when` × 4 `gift` × 4 `cost` + 4 twists (TRAITS.md §2). A shape ships only if it passes the
  build test and the lever on the cohort sets (§Gates) — the table is measured offline per build
  (`examples/traits.rs`, from the prototype) and checked in (`crates/riddle-core/src/traits.json`); the
  runtime draws from the table, never from raw parts.
- Tiers: common 70 %, uncommon 25 % (gift one step larger), rare 5 % (no cost, or a twist); marked
  traits come from lineage events (a grudge three deep, a band boss slain).
- Offers: `Rng::derive(seed, hash("trait_offer") ^ heir)` (as today's `trait_offer`); twist and fade
  from the same stream.
- Gate: metrics rows (§Gates).

### 3. The wake: three cards, blood and born (core + client; autonomy, expression, story)

- An heir wakes with three cards for the born slot — two fresh draws with different `when`s and the dead
  heir's born trait twisted — and the blood trait passing (fading one tier when its gift never went live
  in the parent's runs; `cut` empties it). A send without a pick takes card 1 and keeps blood (offline
  heirs do the same; nothing punishes absence).
- Each card: the chip (`wrathful · frail`), the formula (`[hurt] → fury +1 · frail`), the paired price
  with the current set (`bank +4 · death −2`, measure lane 1), `?` on an unlearned gift, the tier as the
  chip's rim.
- Bloodline at heir 5 / ascension: one `when` the family leans to (half of all fresh draws); a chronicle
  line (`the fen-born line`).
- Gate: ui.mjs — three cards, the price on each, the pick persists across a reload; tests — offers are
  deterministic, a blood trait unused by its heir fades, cut and bloodline persist.

### 4. Legible, learnable, priced (core + client; clarity, mastery, failure)

- A gift's size and exact trigger are a fact (`trait:<head>`) learned on its first live turn; the chip's
  `?` becomes the number; the report lists the facts learned (`wrathful: fury +1 under 50%`).
- Watch: the first live turn per floor stamps its word on the plate (`WRATH`, `GUARD`, `QUICK`, `MEND`);
  a live gift keeps a glyph; the trace's `trait` column.
- Death: the replays add a neutral-heir counterfactual; when it flips ≥ 6/12, the cause line leads with
  the trait (`frail · max 29 hp`); `forecastMove` attributes the heir's share (`heir wrathful · bank +4`).
- Conditions: `if: trait <head>` and `if: gift live` (unlocks, marks), usable by the editor, the patches
  and the divergence scene.
- Gate: tests (fact on first live turn; the counterfactual; the cause line); qa.rs (a trait's share of a
  forecast move sums with the other parts); copy-lint (stamps 1 word, cause ≤ 3 words).

### 5. Classes, items, pets, oaths (core; expression)

- A class verb takes the live gift (TRAITS.md §8); at class L5 one trait may be *set* into the class.
- Found weapons/armour may carry an edge (`when × gift`) from the table; gifts do not stack (max); forged
  kit never carries one.
- A pet's `when` shares the heir's gift while both are live.
- Oaths may name a trait; an oath reward may be a wake choice (a fourth card, a chosen twist), never a gift.
- Gate: the lever row with every edge and the strongest trait owned; PETS still dies by D8.

## Reveal

Traits are not day-0: heirs 1–2 are neutral; the born slot and the wake's cards arrive with the first
heir who dies past D5 or heir 3 (`reveal.ts` step `traits`); the blood slot, `if: trait`, `if: gift
live` at heir 5 (`heirs`); bloodline at the first ascension or heir 12; twists and marked traits with a
band boss slain or a grudge three deep. Each step glints once (Cut 17 §3).

## Copy

Chip: `<head> · <cost>` (≤ 3 words). Card formula: `[when] → gift +N · cost`. Stamps: `WRATH`, `GUARD`,
`QUICK`, `MEND` (1 word). Wake buttons: `pick`, `cut`, `keep` (1 word). Cause line leads with the part
(`frail · max 29 hp`). Glossary rows (COPY.md §2): **trait** (a heir's; not `temperament`, `perk`),
**gift**, **cost**, **blood**, **born**, **twist**, **fade**. No sentences in chrome, no tutorial text.

## Gates

| Gate | Bar |
|---|---|
| Every shipped shape: best set under it differs from B0 by ≥ 1 row on ≥ 1 cohort set, confirmed (gain ≥ 3 pts with the trait, ≥ 3 pts more than without) | metrics (`traits`) |
| Every shipped shape: \|Δbank\| < the set's best row edit (written set) and < the largest row drop (B0), every cohort set; \|Δbank\| ≤ 10, \|Δdeath\| ≤ 8 pts on B0 | metrics |
| The Cut 25 lever row with the strongest shape (and every edge) owned | metrics |
| ≥ 24 shapes ship, ≥ 4 distinct `when`s among them; the three cards of an offer differ in `when` | metrics, tests |
| No trait-chosen action; a trace mark on every live turn; the first-live callout | tests, qa.rs |
| Bots with the neutral heir unchanged; DEFAULT with the strongest shape dies by D8 ≥ 80 % | `node tools/gates.mjs --full` |
| dice ≤ 5 %, stalls, dances, lanes, divergence, DEFAULT yields 0 | `node tools/gates.mjs --full` |
| Wake cards, prices, persistence; stamps and glyph; reveal order | ui.mjs, cut30.mjs |
| Cohort 25 (three absences): expression or autonomy 0.8 from ≥ 1; no card names a trait as a loss it could not own; mean ≥ 74; α ≥ 0.80 | two blind cards |

## Deviations (to record)

- Prototype (TRAITS.md §6): at +2 damage / +2 armour / +5 speed / 1 hp per 4 turns, 2/66 shapes held the lever
  and the bound on all four sets; the loose build test (≥ 3 pts interaction) passed 55/66 but mostly on
  threshold notches driven by the cost; 3 strict builds, all a gift that moves what a row's condition reads.
  So v1 halves the stat gifts, adds row-coupled gifts and costs (`rested`, `sure`, `thin`), keeps `dim` for
  the Deep, and the strict gate (interaction ≥ 2 × ± on ≥ 256 sends, neutral gain ≤ 5 pts) decides the table.
  The count bar (≥ 24 shapes ship) is provisional until `examples/traits.rs` measures the v1 parts; a short
  table (≥ 12, ≥ 3 `when`s) ships rather than a stat.
