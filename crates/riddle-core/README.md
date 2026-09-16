# riddle-core

Deterministic roguelike sim, rule engine, facts, forecast, offline batch, chronicle, sifter,
meta (Cut 1 + Addenda A–E, Cut 2, Cut 3). All game truth lives here; `riddle-wasm` is a JSON bridge.

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
- **Unlock ids** (`buy`, Cut 2 §3, 35 entries; Cut 3 adds ten, see below): `row5 row6 row7 row8 · party_slot_2
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
- **Run cap 120 000 ticks** (Cut 3; was 40 000, and 20 000 before that: a D10 run on 32×32
  floors takes ~25 000, a D30 run ~75 000). A capped
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

## Cut 3 (biomes 4–6, items, tier 2, ascension) — deviations and additions

- **Ending at D31**; `biome_for`: Foundry 16–20, Deep 21–25, Sanctum 26–30; bosses
  `foundry_master` D20, `lurker_queen` D25, `mirror_king` D30. Sprite keys are the kind names
  in `docs/CUT3.md`. Tags (facts and tokens): Foundry `reflect_melee fire thief alarm heavy
  buffer`, Deep `blind water regen aura mirror`, Sanctum `reflect_melee reflect healer echo
  gaze`. Tag names not in the contract's prose: the forge imp's potion theft is the `thief`
  tag (potions only), the siren's confusion aura is `aura`, the sentinel's stun is `gaze`, the
  acolyte's healing is `healer`, the mirror shade's copy is `mirror` (the Mirror King's tag).
- **`Floor.vision`** (Deep 4, else 7) and **`Snapshot.vision`** (addition): the hero's radius
  on this floor, +2 with a lantern in the pack or `lantern_rig`. Monsters in the dark see the
  floor's radius too; `blind` kinds never see and go to the last noise they heard.
- **Noise.** `rest` (radius 8), every melee blow on or by the hero (6), a bell (40, at the
  landing tile), a bell sentinel's alarm (40). Blind hunters within the radius wake and path to
  the tile; the Lurker Queen within it telegraphs `listens` and calls two lurkers (cooldown
  30). `silence` (100 ticks) swallows every noise. `Run.noise` is the last one (diagnostic).
- **`reflect_melee`**: a melee blow (the hero's, or an ally's) lands on the attacker instead
  (`Src::Reflect`, never re-reflected); arrows, bolts, thrown potions and hazards land. The
  warden flips faces every 40 ticks (`shifts`): `Monster.warden_ranged` decides which of
  `reflect_melee` / `reflect` holds. `Monster::reflects_melee()` / `reflects_ranged()`.
- **Mirror King**: `Run.verb_ring` holds the hero's last three verbs while he is in view — the
  verb of the hit landed (`attack shoot bolt cleave shield_bash throw`), else the action's
  (`tactic`, `rest`, chores). A hit whose verb equals the last two is reflected; he telegraphs
  `mirrors` on his first sight and on every repeat (both learn `boss:mirror_king:counter`).
- **Iron golems** are slow (4), see four tiles and forget what they cannot see; the chores
  path around a golem whose `reflect_melee` tag is known as they path around water
  (`Run.mirrors`). The Foundry stocks three throwables per floor and a bow on D16 and D20.
- **Drain recovers a floor at a time** (`Hero.max_hp_base`, +5 per descent): the Crypt's
  wraiths and the Lich left a 54-HP fighter at 19 by D16, which no 30-floor run survives.
- **Depth growth freezes at D16** (`depth_hp_bonus`/`depth_atk_bonus`) and the group count at
  nine (D15): the new kinds carry the difficulty on their own numbers.
- **Items.** `recall` banks from anywhere (tier bank, 100 %); `recall_sense` reads it at
  `hp<15%` as a free row (`Ev.rule` row −2, text `recall sense`). `earthquake` turns walls
  within 2 into floor, never the rim (connectivity can only grow); the snapshot carries the new
  tiles (no tile event). `chalk` is spent on descending: `chalk:<depth>` is learned and every
  later visit of that depth starts with the stairs known and the chores walking straight to
  them (permanent, one depth per chalk). `bell` needs no target (lands 3–6 tiles off, away from
  the foes). `salt`, `clarity` are thrown at a target (`throw salt` / `throw clarity`).
  `mirror` (scroll) reflects the next blow taken (`Hero.mirror_charge`). `mirror_shard` in the
  vault is spent by `breed` to add the `mirror` tag; a `mirror` companion has the verb `mimic`
  (the hero's class verb: bash / backstab / shoot / bolt). Misc finds are facts
  (`item:lantern bell salt chalk mirror_shard`) and `item:recall` is learned on identifying
  recall; they gate `lantern_rig` / `recall_sense` and the tokens.
- **Chores added.** An identified enchant scroll (with a weapon) or strength potion is used
  when nothing is in view (a pack full of them blocked every throwable). A full pack keeps one
  spare weapon and one spare armour; a second spare makes way for a consumable, and any spare
  melee weapon for a bow. Fire and gas are not thrown at an adjacent target by the cards.
- **Cards.** `cadence`: two of a verb, then another (bash, cleave, a throw, volley, drain, a
  melee swing for casters, else a `feint`). `noise_discipline`: rests only while no blind foe
  seen on this floor still lives (`Run.blind_seen`), else keeps moving. `reflect_read`: shoots
  (a pack bow goes up for an action and comes back down by itself, `Run.bow_swap`), throws at a
  reflecting boss or when cornered, fights the others, routes around mirrors, steps clear,
  holds. `deep_march`: at vision ≤ 4 and ≥ 40 % seen, the stairs.
- **Unlocks** (45): the tier-2 rows as the contract. `bosses_slain()` counts kinds.
  `studied_all_<biome>` trophies are three marks; `ledger:<biome>` covers all six biomes.
- **Rows**: `MAX_ROWS` 10 (`RuleSet::validate`, `row_fired`); `short_list` caps `max_rows` at 6.
- **Ascension** (`ascend(variant)`, wasm `ascend`): only after the ending. Keeps classes,
  kennel (the party goes home to it), vault, ledger/facts, forge, trophies, rules. Resets
  marks, gold, heir, best depth, renown/rank, graveyard, grudges, bones, supplies, insurance,
  rest. **Unlocks start over** (the contract lists what carries and they are not on it; the
  second act re-buys them, which is what the dayplayer's purchase-day bar needs) except the
  class doors (`rogue ranger caster`), the mastery cards, and under `short_list` the tactic
  cards. `Lineage.ascension {level, variant}`; the ending under a variant records it in
  `LineageState.ascended`. All four variants are offered at every ascension (the contract's
  "more unlock by finishing with each" is not enforced; `ascended` is exposed for the client
  to shade chips). `no_rest`: `rest` never executes and leaves the vocabulary, camp rest and
  wake are halved (`Game::rest_after`). `bones_only`: `vault_slots()` 0, `loadout` ignored,
  `keep` salvages everything. `hunted`: `LineageState.hunter` = the deepest grudge of the old
  lineage, spawned awake on every floor from D3.
- **Run cap 120 000 ticks** (was 40 000; a D30 run needs ~75 000). Forecast sims stop at the
  depth asked (`known_to`, or a delta's target) and a forecast stops launching sims past a
  tick budget (400 000; deltas and stall patches 150 000; at least 5 sims): a D20 lineage's
  sims run ~40 000 ticks each, and the stall verdict's four forecasts took 6 s without the
  budget. Shallow lineages stay under the budgets (unchanged numbers).
- **Examples**: `metrics` has the FULL bots (`presets/full.json`, three 8 h batches, and the
  same set minus one boss counter row each) and measures the quiet per-tick cost before the
  parallel jobs; `dayplayer` ascends with `no_rest` after the ending and keeps counting;
  `probe --full` surveys a set's exits by depth; `watch [seed] [depth] [--survey]` traces one
  FULL run from a depth or summarises every run's last floor.

## Additions to the `Engine` interface (all JSON strings)

`unlocks()` → `UnlockInfo[] {id,cost,owned,available,needs?}` · `setClass(class)` → Lineage ·
`selectSet(i)` → Lineage (three saved sets; `setRules` writes the active one) ·
`fromSave(json)` (static constructor) · `setKeepPref(pref)` → Lineage ·
Addendum A: `setParty(idsJson)`, `setCompanionRules(id, setJson)`, `breed(a,b)`, `hatch(eggId)`,
`companionVocabulary(id)` · Addendum B: `buySupply(kind)`, `clearSupplies()`,
`supplyCatalogue()` → `{kind,price,label}[]` · Addendum D: `keep(idsJson)` · Cut 3: `ascend(variant)` → Lineage.
Companion condition tokens: `self_hp< self_hp>` plus the hero set; companion verbs
`attack shoot burst steal split flank drain follow recall`. Hero scope cond `{k:"party",t:kind}`,
`{k:"party_hp<",n}`, verb `tame` (`nearest | tag:T`).

## Layout

`src/` per `docs/CUT1.md` plus `wire.rs` (the wire structs) and `tests.rs` (integration
tests). `examples/cli.rs` playtest; `examples/metrics.rs` gates; `examples/bench.rs` timing.
`presets/{fighter,rogue,ranger,caster,good,full}.json` (`full` is the Cut 3 FULL bot: every
counter, every unlock assumed). `examples/dayplayer.rs` is the 14-day player simulation
(`--gate` checks the Cut 2/3 bars; `--verbose` logs purchases and bests; after the ending it
ascends with `no_rest`). `examples/probe.rs --full` and `examples/watch.rs` are the Cut 3
survey tools.

```
cargo run --release --example cli -- --seed 1 --rules presets/good.json --runs 3 [--verbose] [--all-deaths]
cargo run --release --example metrics [-- --seeds 30 --hours 8]
```

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`, Cut 3; `node tools/gates.mjs --full`)

```
DEFAULT dies by ≤ D6 ≥ 80% of seeds                                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                                 93%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts                                93 pts  PASS
RANDOM loses 100%                                                  100%  PASS
PASSIVE loses by ≤ D3 100%                                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2                           4.75 vs 4.57  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds                               100%  PASS
TRIVIAL never passes D5 ≥ 90% of seeds                             100%  PASS
COUNTERED reaches ≥ D11 ≥ 50% of seeds                              80%  PASS
FULL reaches ≥ D26 ≥ 50% of seeds (3 × 8 h)                         87%  PASS
FULL−D20 never passes D20 ≥ 90% of seeds                           100%  PASS
FULL−D25 never passes D25 ≥ 90% of seeds                           100%  PASS
FULL−D30 never passes D30 ≥ 90% of seeds                            97%  PASS
Unfair deaths (dice) ≤ 5% (n=2227)                                 0.4%  PASS
Deaths tracing to a row (gap) ≥ 70%                               99.6%  PASS
Top death cause share < 35% (goblin)                              28.6%  PASS
Events per 600 ticks (renderable) ≥ 6                              44.3  PASS
Replay hash identical (seed+rules+elapsed)                               PASS
Forecast known_to == best_depth + 1                                 all  PASS
Expeditions per 8 h (DEFAULT, EDITED) in 6–16               15.8 · 12.2  PASS
DEFAULT yields 0 xp/gold over 8 h                                     0  PASS
EDITED banks ≥ 3 runs per 8 h                                       7.2  PASS
Patches whose row fired in ≥ 50% of replays (n=297)                100%  PASS
Verdict time ≤ 0.4 s (mean of 2227)                              0.15 s  PASS
Per-tick cost ≤ 6 µs (quiet, DEFAULT/EDITED/FULL)               3.01 µs  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed              min 19  PASS
```

FULL (three 8 h batches, 30 seeds): best depth mean 30.2, 26 of 30 seeds reach D26; per run
D10 81 %, D15 60 %, D20 38 %, D25 28 %, D26 9 %, D31 8.6 %; 6.3 expeditions per 8 h, run
length p10 / median / p90 7 140 / 25 067 / 53 445 ticks. Quiet per-tick cost DEFAULT 1.9 µs,
EDITED 2.3 µs, FULL 2.9 µs (the contended batch number, reports included, is ~8 µs).

The 14-day player (`examples/dayplayer.rs --gate --seeds 3`): marks unspent ≤ 8, empty
check-ins 0 %, L10 on day 10 — PASS; **days with a purchase 8.0 / 14 (bar 10) and the longest
counter-known stall 5 days (bar 3) — FAIL.** The simulated player reaches D19–20 in 14 days
and stalls at the Foundry Master; it buys the cheap catalogue in the first three days and then
earns ~2 marks a day (ranks) while the tier-2 items cost 14–18, so purchase days stop at 8; it
never reaches the ending, so the ascension (the second act's re-buy) never starts. The Cut 2
notes said these two bars assume the 30-floor dungeon; they assume more than that — a player
who beats the Foundry Master by day 9, which needs a bow and the `reflect_read` card in the
first four rows. Content, not bars, was tuned; the remaining lever is the sim's player model
(it keeps `hp<50 → return` and never rearms its rows for the Foundry), which is a design call.

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

Cut 3 (content to the bottom), before the Cut 2 and Cut 1 notes below:

0. **Melee is reflected — so the fighter needs a bow.** The Foundry's golems and the Foundry
   Master are unbeatable by a set that only swings; the first FULL bot danced beside golems for
   a hundred actions and went home at 20 % HP. Three things made the Foundry passable: golems
   are slow (speed 4), see four tiles and forget what they cannot see, and the chores route
   around a golem whose tag is known; `reflect_read` puts a pack bow up (the Foundry stocks one
   on D16 and D20) and only spends throwables on a boss or when cornered; a full pack gives a
   spare melee weapon's slot to a bow. Golem damage per run fell from 40–94 to single digits.
1. **The pack is the run.** A D16 pack was `leash, sword, strength ×3, enchant ×5` — every
   throwable and every silence scroll walked past. Identified enchant and strength are used as
   chores; a second spare piece of gear, or a third copy of a consumable, makes way for a kind
   the pack lacks (never for something cheaper: a third poison and an aggravate scroll swapped
   for ever, 120 000 ticks of `pick_up`); silence is worth 18 so it displaces a 14. Drained max
   HP comes back five a floor (the Crypt left the L10 fighter at 19 HP by D16).
2. **The Queen is a storm, not a duel.** With her hearing the whole floor, FULL never saw her
   before the storm broke; with her calling only within twelve tiles and her called lurkers
   fading after 150 ticks, silence read *when called lurkers show* (`foe_tag:summoned`) lets
   the storm pass and the hero close. Silence is 200 ticks (the contract's 100 ran out before
   a fighter could finish her); she mends 2 per 10 ticks while a called lurker lives and her
   own row (`boss_focus`) kills the adjacent ones first. Without the scroll row FULL passes
   her in 0 of 23 D25 runs; with it about half. The Mirror King heals what he sends back and
   has 80 HP, which is what made the `cadence` row load-bearing (100 % held).
3. **Forecasts of a deep lineage need a budget.** The stall verdict's four 20-sim forecasts at
   D17 cost 6 s (24 µs per batch tick against 2.6 µs for the tick itself); sims now stop at
   the depth asked and a forecast stops launching sims past 400 000 ticks (deltas 150 000).
   The gate measures the quiet tick (one run to its end per bot, single-threaded).


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
