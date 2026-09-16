# riddle-core

Deterministic roguelike sim, rule engine, facts, forecast, offline batch, chronicle, sifter,
meta (Cut 1 + Addenda A–E, Cut 2). All game truth lives here; `riddle-wasm` is a JSON bridge.

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
- **Eggs hatch after 3 camp rests *after* the one that laid them** (1 with `incubator`;
  Cut 2 §1 supersedes "5 expeditions"). `send` skips a rest but still counts it for the egg.
- **`hatch(eggId)` costs 50 gold** (Addendum B supersedes Addendum A's 2 marks).
- **The hero is never "lone"** for the `pack>lone` counter (it has no tags); monsters and
  companions are lone when no friend is within 2 tiles.
- **Companion ids**: tamed on the floor get `1_000_000 + run_id*100 + n`; hatched ones use the
  lineage counter (small ints). Ids are unique for the lineage.
- **Unlock ids** (`buy`, Cut 2 §3, 35 entries): `row5 row6 row7 row8 · party_slot_2
  party_slot_3 · vault2 vault3 vault4 · rogue ranger caster · tame throw · cond_alert cond_turns
  cond_loot cond_on_kill cond_on_see cond_party_hp · corridor_fighting kite_archers stair_dance
  gas_step pack_break thief_guard boss_focus last_stand · quartermaster auto_supply auto_insure
  incubator supply_cap_5 bone_sense third_tag`. `UnlockInfo.needs` is the human-readable gate
  still missing (`"fact: ranged"`, `"tame 1"`, `"slay a boss"`, a prerequisite id for the
  row/vault/party chains). Gates: `party_slot_2/3` tamed ≥ 1/3, `ranger`/`caster` 1/2 bosses
  slain, `tame` `item:leash`, `cond_alert` the `alert:rising` fact (learned when the floor
  alert first reaches 3), `cond_on_kill` a kill, `cond_on_see` any `foe:*`, `cond_party_hp`
  tamed ≥ 1, tactic cards a matching tag fact (`pack ranged boss-counter gas fast thief boss
  heavy`), `incubator` an egg ever laid, `bone_sense` any `bones:*`, `third_tag` bred ≥ 1.
  Mastery cards `phalanx hit_and_fade hawkeye archmage` are granted at class L10.
- **Condition tokens as unlocks.** `alert>= turns> loot>= on_kill on_see party_hp<` leave the
  vocabulary until bought; a row using one the lineage does not own never fires. `foe_hp<`
  opens once any kind is *studied* (five kills, fact `foe:<kind>:studied`) and holds only for
  studied kinds; studied kinds tame +20 points; `LedgerRow.studied`.
- **`set_rules` truncates to the unlocked row count** before validating (a patch inserted into
  a full set pushes the last row out; the editor never sends more).
- **Camp rest (Cut 2 §1).** `finish_run` schedules `Lineage.rest_left` ticks: a death is the
  fixed 20-minute wake; a bank or return rests as long as the expedition, capped at 30 min
  **and never under 20 min** (deviation: a two-minute `hp<50 → return` sortie followed by a
  two-minute rest farmed 300+ runs a day in the 14-day sim; the floor keeps every cycle at
  20–30 min plus the expedition, which is what the 6–16 per 8 h gate assumes). `Ev::rest
  {seconds}` follows `exit`. Offline, rest ticks come out of the same 10/s budget (`rested_s`
  in the report); online, `step` consumes rest first while the run has not begun; `send`
  zeroes it. The stall sampler counts rest in its per-run mean.
- **Run cap 40 000 ticks** (was 20 000: a D10 run on 32×32 floors takes ~25 000). A capped
  run comes home as `return` with **no yield** (`loot_kept 0`, no gold/XP/renown/salvage; the
  pack still comes home) — a stalemate is not a policy, and DEFAULT must yield nothing.
- **Yield follows the exit (Cut 2 §2).** `ExitTier::pct` bank 100 / return 60 / death 0 for
  gold, salvage gold, class XP and renown. On death the whole kit (pack + worn, minus the
  starting arm and insured brought items, which come home) becomes a `bones` pile
  (`Lineage.bones`, max 3, oldest expires) and the fact `bones:<depth>` is learned. The pile
  is a floor item of kind `bones` (id `1_000_000 + heir`, `amount` = heir) placed on that
  depth's floor of every later run; stepping on it recovers the items (`Ev::bones`, a note,
  overflow dropped on neighbouring tiles), the pile leaves the lineage at the exit, and a
  named heir (one with deeds) is a `bones` highlight (6). `bone_sense` paths the chores to it
  over the whole map.
- **XP** = `(Σ_kills (2 + depth)) / 4 + 3 × max_depth`, × 1.0 / 0.6 / 0 by exit;
  `xp_to_next = 100 × level²`. A D8 bank with 50 kills ≈ 130 XP; the 14-day sim's returner
  reaches L10 on day 8.
- **Renown per absence = the best single run's score** (`Batch.best_score`, settled once per
  `run_offline` report; a run watched online settles on its own). Score = 10·depth + kill
  value + 25·boss + highlights, × the exit's yield. Ranks at `100 r²` grant a mark.
- **Marks**: new depth (1/floor), boss (3), trophy (2), rank (1). No first-kill marks (first
  kills stay in `bests`). Lifetime trophies added so marks trickle across the arc:
  `studied:5/10/14 home:10/50/100/200 bones:1 slain:100/500/1000` (plus Cut 1's
  `pacifist_floor no_heal_D5 ranged_only_D5 boss_untouched master:<class> ledger:<biome>`).
- **Automations**: `quartermaster` — `auto_keep` keeps the best weapon *and* armour regardless
  of `keep_pref`; `auto_supply` — `start_run` re-buys the last expedition's supply kinds when
  the shelf is empty and gold allows; `auto_insure` — `start_run` insures each brought item
  it can afford. `supply_cap_5` 3 → 5 supplies; `third_tag` breeding carries 3 tags (2 without).
- **Classes**: `ranger` (30 hp, starts with a bow) L1 `shoot` `kite` (keep range 3), L3
  `volley` (every foe on the line to bow range, cooldown 80 ticks), L5 `trap` (a `trap` floor
  item on the tile toward the nearest foe; the first hostile onto it is stunned 20 ticks), L7
  `mark` (target takes ×1.5 for 30 ticks), L9 `double_shot` (cooldown 40). `caster` (24 hp,
  dagger) L1 `bolt` (2–5 + str, range 6, no ammo, not reflected) `ward` (+2 def 30 ticks,
  cooldown 100), L3 `blink` (3 tiles away from foes, cooldown 50), L5 `slow` (speed −5 for 30
  ticks), L7 `nova` (fire on the 8 tiles around, 5 immediate, cooldown 150), L9 `drain`
  (steal 5 hp adjacent). `shoot bolt mark slow double_shot` take the attack selectors
  (`nearest lowest tag:T`). Sprite keys `hero_ranger` / `hero_caster`.
- **Tactic cards**: `gas_step` (out of hazards; gas foes at range, else a step back when
  hurt, else strike others first), `pack_break` (corridor, then the weakest adjacent; never
  chase the pack), `thief_guard` (a thief adjacent dies first; one fleeing with loot is shot or
  pelted; else a corridor), `boss_focus` (the Warlord aimed, the Mother at range / thrown,
  the Lich's summons first), `last_stand` (below 30 % with a foe adjacent: heal, second wind,
  unknown potions, throwables, then the weakest — no retreat), `hawkeye` / `archmage`
  (mastery bundles of the ranger / caster ladders).
- **`throw unknown[,sel]`** is in the vocabulary with `throw`: an unidentified potion,
  malevolent hint first, identified by the throw.
- **`Snapshot.stake`** `{loot, brought:[{label, insured}], return_row?}` — `return_row` is the
  index of the first `return`/`bank` row within the unlocked rows.
- **Floor alert clock**: every 800 ticks on a floor (`ALERT_EVERY`), 1 + alert/4 wanderers.
  (Cut 1's clock only ran on floors entered at a tick multiple of ten — a bug; D2+ had no
  wanderers.) The Warlord presses the hero himself while two goblins guard him (a wall that
  never bites was a stalemate). Sealed stairs are still walked *up to* by the chores once the
  floor is explored, so a sleeping boss far from the hero cannot deadlock a run.
- **Floors 32×32** (16 + depth/4 rooms of 3–6 × 3–5, three extra loops; caves keep ≥ 220
  open tiles); the chores descend at 60 % seen once the way down is known (only for a heir with at least
  one row: an empty list is no orders, and that heir sweeps floors whole as in Cut 1). D1 is
  the doorstep
  (rats, monkeys, lone goblins; packs from D2, archers from D4). 6 gold piles and 7 + depth/3
  items per floor.
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
  forecast by 0.02, an unconditioned row must beat it by 0.30; ranked by forecast delta when
  any patch moves it (> 0.02), else by (survive − baseline); a patch below −0.05 sinks under
  all others; only verbs the hero could have executed from the checkpoint's inventory are tried.
- **Stall verdict** (`ReturnReport.stall?: {row, fired, text, patches}`): present when the
  last ≥ 4 runs (since the last death, new depth or rule edit — a window on the game, so the
  client's 30-minute quick slices add up; state, not a delta: take the last slice's) all came
  home. `row` is the rule row that ended most of them (`Run.exit_row`, counted in
  `Batch.exit_rows`), `text` ≤ 12 words (`R1 return ended 16 runs at D5`), `patches` ≤ 3 with
  real forecast deltas at the stall depth + 1 (20 sims each; `survive` on these is the patched
  reach at that depth), kept when Δ > 0.02, ranked by Δ. Candidates: the ending row 10 points
  deeper (`hp<20 → hp<10`, `depth>=N → N+1`, `loot>=N → N×2`) as `replace`, that row as
  `remove`, `foe_tag:boss → attack tag:boss` / `throw <known>,tag:boss` on a boss floor once the
  boss or its counter is known, `hp<90 → rest` when the set never rests. The forecasts run
  once per (row, depth, rules, vocabulary) and are cached.
- **`Patch.replace?` / `Patch.remove?`** (stall patches only): `replace` swaps the row at
  `insert_at` for `row`; `remove` deletes the row at `insert_at` (`row` echoes it). Absent =
  insert before `insert_at`, as on the death screen.
- **Sifter**: one highlight per pattern per run (highest score); summons (`spectral_*`) are
  never a first kill or a best.
- **Thrown poison stacks** (each dose adds 40 ticks of 2/10). A full pack swaps its cheapest
  consumable for a dearer one (throwables are worth 14).
- **Offline gate scope**: `learned ≥ 1 / pending ≥ 1` is checked for the player-shaped bots
  (DEFAULT, EDITED, PETS, LEVELLED). LEARNED knows every fact by construction; RANDOM and
  PASSIVE are probes.

## Additions to the `Engine` interface (all JSON strings)

`unlocks()` → `UnlockInfo[] {id,cost,owned,available,needs?}` · `setClass(class)` → Lineage ·
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
`presets/{fighter,rogue,ranger,caster,good}.json`. `examples/dayplayer.rs` is the 14-day
player simulation (`--gate` checks the Cut 2 bars; `--verbose` logs purchases and bests).

```
cargo run --release --example cli -- --seed 1 --rules presets/good.json --runs 3 [--verbose] [--all-deaths]
cargo run --release --example metrics [-- --seeds 30 --hours 8]
```

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`, Cut 2)

```
DEFAULT dies by ≤ D6 ≥ 80% of seeds                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                 87%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts                87 pts  PASS
RANDOM loses 100%                                  100%  PASS
PASSIVE loses by ≤ D3 100%                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2           4.65 vs 4.57  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds               100%  PASS
TRIVIAL never passes D5 ≥ 90% of seeds             100%  PASS
COUNTERED reaches ≥ D11 ≥ 50% of seeds              87%  PASS
Unfair deaths (dice) ≤ 5% (n=2027)                 0.2%  PASS
Deaths tracing to a row (gap) ≥ 70%               99.8%  PASS
Top death cause share < 35% (goblin)              30.0%  PASS
Events per 600 ticks (renderable) ≥ 6              43.6  PASS
Replay hash identical (seed+rules+elapsed)               PASS
Forecast known_to == best_depth + 1                 all  PASS
Expeditions per 8 h (DEFAULT, EDITED) in 6–16  15.8 · 12.8  PASS
DEFAULT yields 0 xp/gold over 8 h                     0  PASS
EDITED banks ≥ 3 runs per 8 h                       7.2  PASS
Patches whose row fired in ≥ 50% of replays (n=227) 100%  PASS
Verdict time ≤ 0.4 s (mean of 2027)              0.06 s  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed      PASS
```

Run length (ticks) p10 / median / p90: DEFAULT 4117 / 5987 / 8840 (70% in 3600–7200, i.e.
6–12 min), EDITED 5790 / 8440 / 15640. Per 8 h: DEFAULT 15.8 expeditions (all deaths, 65% of
the absence is the 20-minute wake), EDITED 12.8 with 7.2 banked. DEFAULT reaches D5 in 56% of
runs and never leaves it; EDITED reaches D10 in 18% of runs, COUNTERED D11+ in 12%.

The 14-day player (`examples/dayplayer.rs --gate`, 3 check-ins a day): marks unspent ≤ 8
(max 8), empty check-ins 8%, class L10 on day 8 — PASS; days with a purchase 8.5 / 14 (bar
10) and the longest counter-known stall 4 days (bar 3) — FAIL, see the tuning notes.

## Tuning notes (how the gates were met, in order of importance)

Cut 2 (pacing), before the Cut 1 notes below:

0. **The wake is the metronome.** A death costs 20 minutes at camp; a DEFAULT run must
   therefore last ≥ 10 minutes for ≤ 16 expeditions per 8 h. That took 32×32 floors with 16
   small rooms (walking, not seeing, explores a floor), 6 gold piles and 7+ items per floor
   (detours), a gentler D1 (packs from D2, archers from D4, so DEFAULT reaches the Warlord
   more often), and a wanderer clock of 800 ticks — the Cut 1 clock (400) only ever ran on D1
   by accident. Rest after a bank/return has a 20-minute floor for the same reason.
1. **Stalemates had to end in a policy's failure, not a timeout.** Three were found by the
   timed-out runs: the Warlord never bit an armoured hero (he presses when guarded now), a
   sleeping boss far from the stairs left nothing to do (the chores walk up to sealed stairs),
   and a recovered kit that did not fit the pack made `pick_up` loop on its own tile (piles
   are scattered, and `pick_up` takes the first item that changes anything). The cap is now
   40 000 ticks and yields nothing.
2. **XP had to be cut three times** (÷4 kill value, 3/depth) once returns paid: a returner at
   60 runs a day would otherwise be L10 on day 2. The 14-day sim still finishes the dungeon
   on day 4–9 because `attack tag:boss` plus `rest` at L5–7 brute-forces the Bloat Mother and
   the Lich; the boss walls hold only against sets without the aimed row. That is why the
   sim's marks front-load (depth marks arrive by day 4) and the purchase-day bar sits at 8.5:
   the next lever is boss strength scaling with class level, which is a design call.
3. **`set_rules` truncation** fixed the verdict: a patch on a full 8-row set made a 9-row
   set that failed validation, so every EDITED death was "dice" (11% overall).

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
