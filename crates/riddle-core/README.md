# riddle-core

Deterministic roguelike sim, rule engine, facts, forecast, offline batch, chronicle, sifter,
meta (Cut 1 + Addenda A–E). All game truth lives here; `riddle-wasm` is a JSON bridge.

## Wire deviations

The JSON shapes follow `docs/CUT1.md` exactly (snake_case, same enum strings, same optional
fields). Where the contract left a choice open, this is what the engine does:

- **Fractions, not percents.** `Forecast.depths[].reach`, `Forecast.causes[].share`,
  `Death.patches[].survive` and `forecast_delta` are `0..1` floats (`0.42`, not `42`).
- **Unknown consumables hide their kind.** `InvItem.kind` / `FloorItem.kind` is the true kind
  when `known`, otherwise `"potion"` or `"scroll"` (label `"blue potion?"`, `hint` when the
  hero got one). Gear and gold are always known.
- **`Ev.rule` is also emitted for chores** (`row: -2`, verbs `explore | descend | pick_up |
  wait | shuffle | paralysed | stumble`) and traits (`row: -1`, text starts with the trait).
- **`throw` argument** is `"K"` or `"K,nearest"` or `"K,tag:T"` (one `a` string).
  **`tactic`** is a verb `{v:"tactic", a:"corridor_fighting"}`. `hold` is a valid verb
  (stay put) used by tests; the editor never offers it.
- **`recall` / `send` are free actions**: a row with them fires, then evaluation continues
  to the next row (the hero still acts this turn).
- **Sanity "no drink at full HP"** applies to an *identified heal*; `drink(unknown)` at full HP
  is allowed (that is the ID-policy family).
- **`turns>N`** counts turns (ticks ÷ 10), so the token keeps its meaning under Addendum E.
- **`Snapshot.entities`** lists visible monsters plus every living ally (companions,
  hounds, freed captives) even when out of sight; tags shown are the *known* tags (facts) plus
  status tags (`stunned paralysed confused fleeing captive grudge`). Hero `tags` are statuses.
- **Ticks.** `Ev.t` and `Snapshot.turn` are ticks (Addendum E); `ReturnReport.elapsed_s` is
  seconds (10 ticks/s). The verdict replays from the last history checkpoint (≈100 ticks).
- **Exit pending.** The last `StepResult` of a run carries `exit_pending` (Addendum D). If the
  client calls `send`/`step` without `keep`, the engine keeps by `keep_pref` automatically.
- **Eggs hatch after 5 completed expeditions *after* the one that laid them.**
- **`hatch(eggId)` costs 50 gold** (Addendum B supersedes Addendum A's 2 marks).
- **The hero is never "lone"** for the `pack>lone` counter (it has no tags); monsters and
  companions are lone when no friend is within 2 tiles.
- **Companion ids**: tamed on the floor get `1_000_000 + run_id*100 + n`; hatched ones use the
  lineage counter (small ints). Ids are unique for the lineage.
- **Unlock ids** (`buy`): `row5 row6 row7 row8 rogue vault2 vault3 corridor_fighting
  kite_archers stair_dance throw party_slot_2 tame`; mastery cards `phalanx` / `hit_and_fade`
  are granted at class L10. Tactic cards need a fact (`pack`, `ranged`, any boss counter).
- **Verdict baseline.** `Death.baseline` (addition, 0..1) is the survival of the *unpatched*
  rules over 20 reseeded replays from the same checkpoint. The verdict itself follows the
  contract literally (`gap` iff some one-row patch survives ≥ 60% of replays), but the
  client can show the baseline next to a patch's `survive`: a patch that barely beats it
  means the death was mostly the dice. (Measured: about half of all deaths have a baseline
  ≥ 60%, so a baseline-aware verdict would fail the ≤ 5% dice gate; the contract's
  definition is kept and the number is exposed instead.)
- **Patch forecast deltas use 20 sims per candidate** (`forecast::DELTA_SIMS`; the forecast
  itself uses 50). Four forecasts per death at 50 sims made the return report take ~6 s
  natively; 20 keeps it under ~2 s. Same seeds for base and patched runs, so the delta is
  paired and low-noise.
- **Boss walls (playtest pass).** A living boss seals the stairs down. Goblin Warlord: a
  shield wall — any goblin in his view takes an *unaimed* swing meant for him (reserves
  step in if none is left), he rallies two goblins whenever fewer than two guard him or he
  is hit, and every 30 ticks he shield-buffs the goblins (+1 def). Only `attack tag:boss`
  (aimed) or `shield_bash` lands on him and cancels his rally; `taunt` cancels telegraphs.
  Bloat Mother: every melee hit she takes vents a 5×5 cloud, she heals 3/10 ticks in gas
  and never chokes on it; the counter is range (`throw K,tag:boss`, bows). The Fens stock
  three throwables per floor. Lich: reflects arrows and thrown potions, chants two brittle
  skeletons (4 hp, tag `summoned`) every 60 ticks while any stand, 100 when none do; melee
  the summons first. Every boss telegraphs on its first turn and that observation is the
  `boss:<kind>:counter` fact. A run that cannot finish in 20 000 ticks comes home (`return`).
- **`rest` costs alert**: every 8 rests raise the floor alert by one; from alert 5 each step
  calls a pack (2–3) out of sight within 8 tiles ("they heard you").
- **Gold ÷ 4** on loot and salvage (`GOLD_DIVISOR`); `insure(id)` at 25% of salvage value ×10
  keeps a brought vault item on death (`Lineage.insured`, addition).
- **Patches**: candidates cover every family (ID policy `hp<N → drink/read unknown`, escape
  `hp<N → return`, retreat, targeting); a patch must beat the baseline by 0.15 or move the
  forecast by 0.02, an unconditioned row must beat it by 0.30; ranked by (survive − baseline)
  then delta, conditioned rows first on ties; only verbs the hero could have executed from the
  checkpoint's inventory are tried.
- **Sifter**: one highlight per pattern per run (highest score); summons (`spectral_*`) are
  never a first kill or a best.
- **Thrown poison stacks** (each dose adds 40 ticks of 2/10). A full pack swaps its cheapest
  consumable for a dearer one (throwables are worth 14).
- **Offline gate scope**: `learned ≥ 1 / pending ≥ 1` is checked for the player-shaped bots
  (DEFAULT, EDITED, PETS, LEVELLED). LEARNED knows every fact by construction; RANDOM and
  PASSIVE are probes.

## Additions to the `Engine` interface (all JSON strings)

`unlocks()` → `UnlockInfo[] {id,cost,owned,available}` · `setClass(class)` → Lineage ·
`selectSet(i)` → Lineage (three saved sets; `setRules` writes the active one) ·
`fromSave(json)` (static constructor) · `setKeepPref(pref)` → Lineage ·
Addendum A: `setParty(idsJson)`, `setCompanionRules(id, setJson)`, `breed(a,b)`, `hatch(eggId)`,
`companionVocabulary(id)` · Addendum B: `buySupply(kind)`, `clearSupplies()`,
`supplyCatalogue()` → `{kind,price,label}[]` · Addendum D: `keep(idsJson)`.
Companion condition tokens: `self_hp< self_hp>` plus the hero set; companion verbs
`attack shoot burst steal split flank drain follow recall`. Hero scope cond `{k:"party",t:kind}`,
`{k:"party_hp<",n}`, verb `tame` (`nearest | tag:T`).

## Layout

`src/` per `docs/CUT1.md` plus `wire.rs` (the wire structs) and `tests.rs` (integration
tests). `examples/cli.rs` playtest; `examples/metrics.rs` gates; `examples/bench.rs` timing.
`presets/{fighter,rogue,good}.json`.

```
cargo run --release --example cli -- --seed 1 --rules presets/good.json --runs 3 [--verbose] [--all-deaths]
cargo run --release --example metrics [-- --seeds 30 --hours 8]
```

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`)

```
DEFAULT dies by ≤ D6 ≥ 80% of seeds                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                100%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts               100 pts  PASS
RANDOM loses 100%                                  100%  PASS
PASSIVE loses by ≤ D3 100%                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2           3.47 vs 3.38  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds               100%  PASS
TRIVIAL never passes D5 ≥ 90% of seeds             100%  PASS
COUNTERED reaches ≥ D11 ≥ 50% of seeds              57%  PASS
Unfair deaths (dice) ≤ 5% (n=15343)                1.5%  PASS
Deaths tracing to a row (gap) ≥ 70%               98.5%  PASS
Top death cause share < 35% (goblin_archer)       24.1%  PASS
Events per 600 ticks (renderable) ≥ 6              68.4  PASS
Replay hash identical (seed+rules+elapsed)               PASS
Forecast known_to == best_depth + 1                 all  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed      PASS
```

Run length (ticks) p10 / median / p90: DEFAULT 2484 / 3760 / 7392 (70% in 1800–4800),
EDITED 2860 / 4470 / 9339 (57%); no run exceeds the 20 000-tick cap. Boss walls hold:
DEFAULT, LEARNED, PETS, LEVELLED and TRIVIAL never leave D5; EDITED and COUNTERED (which
carry the counters) reach D10+ in 12% / 8% of runs.

TRIVIAL (the playtest's 4-row set) never passes D5; COUNTERED (TRIVIAL + `throw fire,tag:boss`
/ `throw poison,tag:boss` at depth ≥ 6 / `attack tag:boss`) reaches D11+. See the gate table
in the report for the measured numbers.

## Tuning notes (how the gates were met, in order of importance)

1. **Rest is the policy lever, not potions.** No natural regeneration; `rest` heals 4 HP per
   action (only with no foe in view, not poisoned, not in a hazard). DEFAULT never rests and
   bleeds out by D4–6; the good set rests to 90% after every fight. Heal potions were made
   scarcer (weight 6 of ~80) because a lucky curious heir with three identified heals was
   the main source of DEFAULT runs past D6.
2. **Fens chip through mail.** Mail (+3) from D4 made every D1–5 monster harmless, so the
   Fens (D6–10) were retuned to keep hurting an armoured hero: jelly 1–3, eel 3–6, ogre 2–4
   (×2 on the wind-up), plus +1 max attack per 6 floors. A non-resting hero cannot absorb
   that; a resting one can. Mail is what lets EDITED reach D10; moving it to D7 collapsed
   EDITED's tail (3.9% → 1.0%) while barely touching DEFAULT's.
3. **Verbs compose, or the trace shows oscillation.** Three engine behaviours were needed
   before any rule set could be "good": `attack` prefers an adjacent target over walking to
   the weakest one; `attack` holds a corridor while an awake foe keeps closing (and gives up
   after three static actions, so lurking packs cannot deadlock it); `back_corridor` only
   goes for a corridor within 5 steps. Without these, `foes ≥ 2 → corridor` + `attack`
   ping-ponged the hero between the room and the doorway while packs bit it.
4. Early monsters are weak on paper (rat 4 hp 1–2, jackal 4 hp 1–2 fast packs, goblin 7 hp
   1–3) and the fighter starts with a sword; density comes from groups (2 + depth/2) and the
   alert clock (wanderers every 400 ticks, 1 + alert/3 of them). Bosses: Warlord 34 hp 2–5,
   rally every 150 ticks (2 goblins, +1 def buff); Bloat Mother 55 hp, swells at half HP,
   5×5 pop; Lich 50 hp, reflects throws, chants 2 skeletons every 90 ticks.
5. **Anti-deadlock guards** (from the coordinator's playtest). `attack` is executable only
   against an *engageable* foe: adjacent, or neither fleeing nor given up on; a target that
   is unreachable, or that three consecutive approaches failed to close on, is ignored for
   30 actions and the row falls through (`foes>=N` counts engageable foes only). Pathing
   remembers where hostiles were seen for 30 actions, so a corridor foe drifting in and out
   of view cannot flip "blocked"/"open" between explore and descend. Oscillation guard: 12
   actions on ≤ 2 tiles with no blood drawn → the visible non-adjacent foes are ignored and
   foe-targeting rows are suppressed for 30 actions (one `stuck` chore event); the same row
   firing 40 actions straight without damage is rested for 30 actions likewise. When every
   path is walled off by foes the chores push through and bump whoever blocks the first
   step. Traits pre-empt the rows at most once per 5 actions and never below 25% HP
   (cowardice excepted); `pick_up` is never chosen three times running without something
   entering the pack (items are then ignored for 20 actions), and greedy only grabs what fits.
6. Fleeing thieves are not chased (a fast monkey wasted whole floors of hero actions), the
   hero paths around water (eels), swaps past allies and chained captives, steps out of gas
   or fire when no foe is adjacent, and shuffles after 20 idle actions.

Performance (release, one core): ~2 µs per tick; an 8 h offline batch ≈ 0.6 s plus the
report's worst-death verdict (~0.6 s) and four 20-sim forecasts (~1 s); `forecast()` with 50
sims ≈ 0.5 s; a full run ≈ 3–8 ms.
