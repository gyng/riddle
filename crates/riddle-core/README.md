# riddle-core

Deterministic roguelike sim, rule engine, facts, forecast, offline batch, chronicle, sifter,
meta (Cut 1 + Addenda A–E, Cut 2, Cut 3, Cut 4, Cut 5, Cut 6, Cut 7, Cut 8B, Cut 9). All game truth lives here; `riddle-wasm` is a JSON bridge.

## Wire deviations

The JSON shapes follow `docs/CUT1.md` exactly (snake_case, same enum strings, same optional
fields). Where the contract left a choice open, this is what the engine does:

- **Fractions, not percents.** `Forecast.depths[].reach`, `Forecast.causes[].share`,
  `Death.patches[].survive` and `forecast_delta` are `0..1` floats (`0.42`, not `42`).
- **Unknown consumables hide their kind.** `InvItem.kind` / `FloorItem.kind` is the true kind
  when `known`, otherwise `"potion"` or `"scroll"` (label `"blue potion?"`, `hint` when the
  hero got one). Gear and gold are always known.
- **`Ev.rule` is also emitted for chores** (`row: -2`, verbs `explore | descend | pick_up |
  wait | shuffle | paralysed | stumble | stuck | cornered`, Cut 5 `return` for a bail) and traits (`row: -1`, text starts with the trait).
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
  still missing (`"fact: ranged"`, `"tame once"`, `"slay a boss"`, a prerequisite id for the
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

## Cut 4 (legibility and feel) — deviations and additions

- **Verdict semantics.** `gap` iff the best candidate beats the unpatched baseline by
  `PATCH_MARGIN` 0.15 in survival, or — when none does — one of the top
  `VERDICT_DELTA_CANDIDATES` (3, by survival edge) moves the forecast at the death's depth by
  `DELTA_BAR` 0.02; else `dice`. The forecast deltas are simulated inside `verdict()` for that
  second test (so a verdict is final, not provisional) and reused by `death()`, which only adds
  the remaining candidates (`DeathRec.deltas_n`). A gap never shows an empty list (the
  fallback is unchanged). Candidates are scored at survival ≥ `min(0.6, baseline + 0.15)`
  (`trace::survive_bar`), so a row that only moves the forecast still reaches the delta test
  at a high baseline.
- **Replay window.** The checkpoint is unchanged (the last history entry at ≥ 50 % HP, at least
  8 turns before death, up to `HISTORY_TURNS` 30 back — measured at 40 too: 4.9 % dice vs 4.7 %
  at 30 on the full table, so the window stays). The window now
  ends at the killing blow plus one turn (`DeathRec.death_tick`; it ended one tick after the
  hero's last *action*, and a hero alive at 1 HP counted as survived — most deaths had a
  baseline of 0.9–1.0 and "dice" was the honest reading of a bug), and a replay that survives
  the blow runs on while an awake hostile is in view, at most `ENCOUNTER_TICKS` 300: surviving
  the tick of the blow is not surviving the fight. `tests_faithful` checks an un-reseeded replay
  dies at `death_tick`.
- **`DELTA_SIMS` 20 → 12** (the replay count): a delta of 0.02 is "one paired seed improved
  net" at either count, and the verdict now waits on up to three of these forecasts. `death()`
  with all deltas: 2.2 s → ~0.9 s native.
- **`forecast::reach_with` is memoised on the game** (`Game.forecast_cache`, per
  lineage-fingerprint × rules × depth × sims × tag × budget; the fingerprint skips marks,
  renown and the rest clock): a batch of verdicts, the stall verdict and the unlock catalogue
  ask for the same unpatched base at the same depth.
- **Remembered foes and the hunt.** `Snapshot.entities` also lists hostiles the hero
  remembers but cannot see — alive, out of view, seen within `REMEMBER_ACTIONS` 10 hero
  actions — at the tile they were last seen on, with `remembered: true` (serde skips it when
  false; hp shown is current). The row that last acted on a foe (`Run.hunt` = (id, row)) keeps
  walking toward that tile when the foe steps out of view, as that row's action (`Ev.rule`
  with the row's index, text `hunt <kind>`), until the foe is seen again, dead, forgotten,
  given up on, or the tile is reached (`ai::hunt_step`).
- **Callouts name the target.** A row whose verb acted on a foe reads `<verb> <kind>` (`attack
  goblin`, `shoot goblin archer`, `bash ogre`, `tame jackal`; ≤ 3 words, the kind's title
  lower-cased) instead of `cond → verb`; other rows are unchanged. A companion's row is
  announced `<kind>: <verb>` (`jackal: flank`) once per streak of the same verb, in view.
- **`foes>=` counts every hostile in view** (`View.foes`); melee targeting (`attack`,
  `shield_bash`) draws from `View.engage` — adjacent, or neither fleeing nor given up on —
  while ranged verbs, throws, `retreat`, `back_corridor` and the conditions see all of them.
  (Rater B: `hp<50 · foes>=2 → retreat` did not fire at 13/40 with five foes because the count
  excluded the foes the hero had given up chasing.)
- **Blocked rows are shown.** The first row per action whose conditions held but whose verb
  could not execute is recorded on the trace turn as `TraceTurn.blocked` (`R1 retreat ✗ no
  path`; serde-skipped when none) and called out once per streak (`retreat ✗ no path`, ≤ 3
  words). Reasons: `no path` (retreat, corridor, blinks), `no target` / `no line` / `no bow` /
  `cooldown` (foe verbs), `no item` / `no use` (drink, read, throw), `no leash` / `none weak`
  (tame), `not safe` (rest), `no stairs`, `no way` (return/bank/recall). Tactic cards fall
  through by design and are never "blocked".
- **Stalemate guards lift on blood.** A hit on the hero clears the oscillation guard, the
  same-row guard and every ignored foe (`Run::unstick`); a hit by the hero clears the guards.
  (Rater B: five `wait` chores at 9 → 1 HP with two foes while `foes>=1 → attack nearest` was
  live — the guards had suppressed the row for 30 actions.) With an awake hostile adjacent and
  nothing to do, the chore is `cornered` (text `cornered, no orders`), never `wait`; the
  chores neither step clear nor swing (PASSIVE must lose every seed).
- **`auto_supply` restocks when the hero comes home** (`Game::restock` in `finish_run` and
  after `keep`, and still at `start_run`), so `Lineage.supplies` shows the shelf before the
  next send; `clearSupplies` also clears the automation's memory (`last_supplies`), so a
  cleared shelf stays cleared.
- **One loot unit.** `Run.loot` is gold: every pickup adds its value to `Run.loot_raw` and
  `loot = loot_raw / GOLD_DIVISOR` (`Run::loot_add`). The HUD stake, `Ev.exit.loot_kept`, the
  exit notes (`Banked $58.` / `Returned with $34.`), the reel and `Lineage.gold` all carry the
  same number; salvage stays in gold as before. The `loot>=` token's offered value is 20 (was
  50, in the old unit).
- **Marks.** The first *bank* from each depth is a mark and a best `home:D<n>`
  (`LineageState.banked_depths`; reset by ascension; returns and timed-out runs do not count).
  Tier-2 costs: `row9` 8, `row10` 12, `vault5` 8, `party_slot_4` 8 (were 14/18/14/14);
  tier-2 cards 5, `lantern_rig` 6, `recall_sense` 8 unchanged. Rank thresholds 100·r² unchanged.
- **Reel and chronicle.** Every reel line is setup + turn + end in ≤ 8 words: `Down to 2 HP,
  then banked $313.` · `Brink, slew the Lich, banked $313.` · `Slew the Lich, fell on D9.` ·
  `Lost the hound on D4, returned with $80.` (`sifter::end_phrase`: `banked $N` · `returned
  with $N` · `fell on D<n>` · `lost the thread`); `first_kill` and `bones` keep their shape.
  `Run.low_hp` is the lowest HP of the run. When the hero fell to ≤ 20 % and lived to leave the
  floor (descend, bank, return), the chronicle names the first row that acted after the low
  point: `R3 rest caught him.` (`Run.saved_by`).
- **`UnlockInfo.delta?`** for every tactic card or verb (`tame`, `throw`) not yet owned whose
  gate is open: the forecast reach at `best_depth + 1` with the unlock owned and its natural
  row inserted at the top of the list, minus the reach without (`DELTA_SIMS` paired sims,
  `CATALOGUE_TICK_BUDGET` 60 000 ticks per forecast, at least `MIN_SIMS`). **`unlockDeltas()`**
  (wasm; `Game::unlock_deltas`) simulates them and returns the catalogue; `unlocks()` never
  simulates and carries the deltas once they are memoised for the current lineage state (a
  purchase or a new fact changes the state: call `unlockDeltas()` again after `buy`). Rows
  (`meta::unlock_row`): `corridor_fighting`/`stair_dance` `foes>=2`; `kite_archers`
  `foe_tag:ranged`; `gas_step` `foe_tag:gas`; `pack_break` `foe_tag:pack`; `thief_guard`
  `foe_tag:thief`; `boss_focus` `foe_tag:boss`; `last_stand` `hp<30`; `cadence`
  `foe_tag:mirror`; `noise_discipline` `hp<90`; `reflect_read` `foe_tag:reflect_melee`;
  `deep_march` `depth>=21`; `throw` `foe_tag:boss → throw unknown,tag:boss` (else `foes>=2 →
  throw unknown,nearest`); `tame` `foes>=1 → tame nearest`. Absent for rows, vaults, slots,
  classes, conditions, automations, owned or gated entries. Memoised with the forecasts (a
  camp visit pays once). Cost, native and quiet: D3 with 4 open entries 0.3 s; D10 with 9
  open entries 1.5–2.2 s (each forecast is `MIN_SIMS` 5 sims of ~12 000 ticks under the
  budget) — the brief's ≤ 0.2 s for the whole catalogue is not met past D3, which is why the
  computing call is separate from `unlocks()`.
- **`Lineage.ascended: string[]`** — the variants the lineage has finished the dungeon with.


## Cut 5 (the story of a run) — deviations and additions

- **Episodes** (`sifter.rs`, rewritten). `Run.arc` is the live episode: the hp low-water mark
  since the last resolution with its tick, the foes in view when it was hit (the boss first,
  then the most numerous, then by name; a hazard or a situation when none was), the cause,
  whether the hero was cornered (two adjacent) or chased (a `fast` kind, or the hero fleeing),
  the first hero action after the low (`Act {row, verb, target, boss}`; row −1 trait, −2
  chores), the last action, items used, allies lost. `sifter::on_hurt` runs on every blow the
  hero survives, `on_action` after every action. It closes on: hp recovering past
  `RECOVER_PCT` 60 after a low ≤ `LOW_PCT` 25 (**sealed**: the episode keeps its low and its
  row and takes the run's *next* resolution — a floor, a boss, an exit — as its own, so a
  scare and what it bought are one line; at most `SEALED_MAX` 2 wait, the deepest kept), a
  boss dying (`first boss` the first time the lineage kills the kind, else `boss slain`), the
  floor changing after a low (`reached Dn`), a companion dying (`<kind> <Name> fell`), an exit
  (`banked $N` · `returned` · `lost the thread` for the run cap · `died to <cause>`). Boss
  kills and a companion's fall close the live episode only; floors and exits also resolve the
  sealed ones. A boss's first sight seals whatever was live, so the boss fight is its own
  episode. An exit with nothing to tell closes an `Untouched` episode, so every run has a
  closing line. A death from full health reads `took him down`.
- **Story lines**: `<setup>; <turn>; <end>.` from tables, ≤ `STORY_WORDS` 12 words
  (`sifter::story_line`; `story_ok` is the gate's check). Setup: `Two jackals took|cornered|
  chased him to 3 HP` (`A jackal`, `An archer`, `The Warlord` — bosses by their short name —
  `Gas`, `The shrine`, `The nest`, `A stray jackal`), `The vault held three`, `Uleth the
  jackal came back`, `Untouched` / `Untouched by two rats`. Turn: `R2 drank` (`PAST`, every
  verb and chore; `CARD_PAST` for tactic cards, `PAST_SHORT` one-word forms), `R4 bashed him`
  for a blow on a boss, traits `greed took the gold` / `greed grabbed` · `cowardice ran` ·
  `bravery held` · `curiosity drank`, chores `the chores explored` or `no row fired` (waits,
  cornered, stuck), `paralysed, no row`, `confused, no row`; a vault's `he took the mail`.
  When the long forms overflow twelve words the beats shorten in turn: the threat (`Jackals`,
  `An archer`), then the resolution (`died to archer`, then `died`; `Uleth fell`), then the
  row (`greed grabbed`, `R3 held`, `no row fired`) — the turn beat is what the line is for.
- **Score** = low-point depth (1 + (100 − low%)/25, so 1–5; 1 for a vault, a stray or an
  untouched run) × resolution weight (`first boss` 5, a death 3 — 5 for an heir with deeds —
  `banked` 3, a companion's fall 3, `reached` 2, `boss slain` 2, `returned` 1) + 2 for a
  situation. Episode scores feed renown as the Cut 1 highlights did (a scare-and-bank ≈ 12,
  a quiet return 1).
- **Reel** (`sifter::reel(highlights, best_run)`): the top three episodes by score, never two
  with the same (threat key, resolution key) — `HighlightArc.threat` (`jackal`,
  `goblin_warlord`, `gas`, `nest`, `shrine`, `vault`, `stray`, `none`) and the resolution
  with its number and a death's cause dropped (`banked $`, `reached D`, `died`, `fell`) — nor
  two identical texts; then the closing episode of the run that reached the best depth
  (`Batch.best_run`; its best other episode when the closing pair is already shown); `bones`
  highlights fill to four. `Highlight.arc {low_hp, row, threat, resolution}` is on every
  episode line and absent on `bones`. `Game.reel` (the lifetime top 50) is unchanged.
- **Lineage chronicle** (`Lineage.chronicle`, cap `CHRONICLE_CAP` 40, oldest first): one line
  per heir on its end, `♟3 the greedy fighter · D7 · "corridor" set · took the Warlord · fell to
  gas · left bones on D7.` — the heir's best depth (`LineageState.heir_best`), the active
  set's name when set, up to two deeds (`took the <Boss>` first kills first, `freed a
  captive`, `tamed a jackal`, `found ♟3's bones`; `LineageState.heir_deeds`), the end
  (`fell to <cause>` · `retired at rank N` when the heir reached the bottom · `ascended`) and
  `left bones on Dn` when a pile was left. Written once per heir (`chronicled`): the heir who
  retires at the bottom gets no second line on a later death.
- **Hero voice** (`sifter::voice`, `Moment`): trait × {low, resolved, gold with foes in view,
  an unknown drunk or read, boss first seen}, ≤ 3 words each (`voice_line`), as `callout`
  events; at most one per `VOICE_EVERY` 100 ticks and never in a fight's first
  `VOICE_FIGHT_QUIET` 10 ticks (`Run.fight_t` is the tick a hostile last came into view after
  none). The Cut 1–4 callouts (`near death`, `boss down`, …) are unchanged.
- **Situations on D1–5** (`engine::place_situations`, rooms only; the Warrens). D1 always
  holds one, in one of the three rooms nearest the entrance; D2–5 hold one at 80 % and a
  second at 30 % (weights nest 3 / vault 3 / shrine 3; on D1 nest 2 / vault 4 / shrine 4).
  Placement draws from a side stream (`Rng::side`), so the floor's own stream is Cut 4's.
  Tiles `shrine | vault | vault_open | nest` are passable; facts `shrine vault nest stray`
  are learned on sight (a note each), and `Run.situations` counts what the run met (the
  gate). Tokens: `on_see: nest | shrine | vault` (`{k:"on_see", t}`) holds while such a tile
  is in view and live (a den asleep, an altar not prayed at, a cage unopened) — gated by the
  fact alone, not by `cond_on_see`.
  - **Nest**: a `nest` tile with a gold pile (`12 × depth + 4–12`) and three sleeping jackals
    (four from D4) on the tiles around it. Sleepers are D1 jackals (no depth bonus) and
    **scenery until they wake**: not foes for `foes>=`, `attack` or the pack, never woken by
    sight, and the chores keep two tiles clear of a seen den and leave its gold alone (like
    water and known mirrors; when the only way on runs through the den, they go through). The
    den wakes when the hero steps within two tiles, or on a blow — a melee row with nothing
    awake to fight may raid it (`on_see: nest → attack nearest` is the raid), a `pick_up` row
    fetches its gold, and a greedy heir with the gold in view is **tempted** (`Run.tempted`,
    the chores walk in; trait text `greedy → the den`). They stir one a turn apart (`stun` 0,
    10, 20…): a fight that builds, not a wall that lands. Callout `the nest wakes`. (A den
    that woke on sight, or that `foes>=1 → attack` walked into, cut DEFAULT's runs by a
    floor and put the expeditions gate at 16.6; as a choice it changes the runs that choose it.)
  - **Vault**: a `vault` tile with a three-item cage (`Run.vault_cage`: a weapon — sword,
    bow, an axe from D3 —, an armour — leather, mail from D3 —, both +1, and heal / strength /
    a teleport scroll; never an enchant scroll: the quartermaster keeps the best weapon for
    life and a scroll a run stacked FULL's sword to +12 by the Sanctum). The chores walk to a
    seen vault while the hero is above half health (`explore` chore). Standing on it opens
    it: the tile becomes `vault_open`, `Snapshot.vault_choice {items}` carries the three, and
    **`choose(itemId)`** takes one (the rest vanish; `Ev::pickup`, a note). Unanswered for
    `VAULT_GRACE` 50 ticks — watched or not, so a verdict replay stays faithful — the
    **`Lineage.vault_pref`** (`weapon | armour | potion | scroll`, `setVaultPref`, default
    `weapon`) picks. The choice is an episode (`The vault held three; he took the mail; …`).
  - **Shrine**: a `shrine` tile; verb **`pray row | trait`** enters the vocabulary with the
    `shrine` fact. The row walks to the altar seen on this floor and prays once per run:
    −`PRAY_COST_PCT` 20 % max HP for the run (`max_hp_base` too, so floors do not refund it),
    and `pray row` **lends a row** — the first row of the player's other saved sets that the
    active set lacks, appended last for the run (`Run.lent_row`; the editor cannot write past
    the cap, so the shrine lends from what the player wrote elsewhere) — or, with nothing to
    lend, and always for `pray trait`, **swaps the trait** for the run (`Trait::swap`: greed ↔
    curiosity, cowardice ↔ bravery). Blocked reasons `prayed` / `no shrine`. The prayer is a
    hurt with cause `shrine`, so it opens its own episode (`The shrine took him to 29 HP; R3
    prayed; reached D2`).
  - **Stray**: when a companion dies on an expedition it joins `LineageState.lost` (kind,
    name, gen, heir; six kept) and a later run places it, wild and asleep, on one D1–5 floor
    (50 % per floor, always by D5; `Monster.stray`, its old name). It fights like its kind;
    **`tame`** takes it at `STRAY_TAME` 60 % whatever its wounds, and it comes back as a
    companion with its name and generation (`Run.strays_tamed`; the lost list forgets it at
    home). The return is an episode (`Uleth the jackal came back; R1 tamed; …`).
- **Bail** (`bail()`, wasm `bail`): a `return` fires on the hero's next action as a chore
  (`Ev.rule` row −2, verb `return`, text `bail → return`; `Run.exit_row` stays unset, so it is
  not a stall row); the rules are untouched. The client's row-prepend still works.
- **Mirror King**: a paralysed turn no longer enters his verb ring (a sentinel's gaze was
  resetting the mirror for the hero — the wall read 77 % without the cadence row once FULL
  had the vault's gear; 100 % with the fix), and he mirrors the pack as he mirrors the hero
  (`Monster.verb_ring`: an ally's third blow of a kind comes back and heals him).
- **Death verdict**: a `rest` candidate family (`hp<N → rest`, first in the list, when no
  rest row fired in the trace) — a set that never rests wanders worn, and the honest patch
  for a death a reseeded replay mostly survives is the rest, not the escape. DEFAULT's
  population dice fell 6.1 % → 5.8 % with it. The gate table's dice and gap shares are now
  **death-weighted**: the verdicts are a sample of eight per seed, which weighted a FULL
  bot's dozen boss-wall deaths (dice by design) thirty times over against DEFAULT's 470; the
  per-bot rate × the bot's deaths is the unbiased estimate (4.4 % here, 3.9 % at the Cut 4
  head under the same estimator; `--verdicts 100000` verdicts every death: 3.8 %, and 3.6 %
  with the situations off). The bar is unchanged.
- **Examples**: `stories [seeds] [hours]` prints reels, episodes, chronicle lines and the
  situation counts of DEFAULT and EDITED absences (the Cut 5 probe).

## Cut 6 (clarity: every number reconciles, every silence speaks) — deviations and additions

- **The ledger line** (§1). Every exit settles into an `ExitLine {carried, keep_pct, kept,
  spent, spent_on, text}`: `carried` is the run's gold on the hero, `kept = carried ×
  keep_pct / 100` (bank 100 / return 60 / death 0 / a capped run 0), `spent` what the
  automations bought on coming home (`auto_supply`'s restock; `spent_on` the kinds), `text`
  ≤ 14 words: `$84 carried · return keeps 60% → $50`, a death `$144 carried · death keeps 0% →
  $0 · bones: 7 items on D5` (the bones clause only when a pile was left), a capped run `lost
  thread keeps 0%`. It rides on `Ev::exit.line` (attached by `step` once the exit settled, so
  the event's line already knows the restock; offline runs discard events), on `Death.line`,
  and `ReturnReport.exits` lists the absence's last `EXITS_CAP` 5, oldest first
  (`Batch.exits`). **Salvage is not in the line**: it is its own ledger movement (an exit's
  gold delta is `kept + salvage − spent`; the test reconciles all three on every exit over 30
  seeds).
- **`Lineage.gold_ledger`** (§1; the contract's `Lineage.ledger`, renamed because `ledger` is
  the bestiary): the last `GOLD_LEDGER_CAP` 20 movements `{t, delta, why}`, oldest first,
  `t` the lineage tick, `why` ≤ 3 words. Every gold change goes through
  `LineageState::gold_move`: `returned D5` · `banked D5` · `died D5` (kept even at `+0`, so a
  death's yield is on the sheet) · `lost thread D5` · `salvage` (one line per settlement) ·
  `heal` / `leash` (a purchase, by kind) · `refund heal` · `insure sword` · `hatch` ·
  `ascended` (the reset; the ledger starts over). Same tick + same reason merges into one line.
- **`Snapshot.stake.kept`** (§1): what the first `return`/`bank` row would bring home now
  (`loot × 60 or 100 %`), absent without such a row.
- **`Item.known`** (§2; serde default false, skipped when false): set on `buy_supply` (also
  the forge's crafted kinds) and on entering the vault (`keep`). `Item::is_known(facts,
  flavours)` = `known || is_identified`; `find_consumable("heal")`, the `item:` token, the
  full-HP heal sanity, the "Gambled" note and the ID-policy searches (`drink unknown`,
  `read identify`, the curious trait) all use it, and the wire label names a known item. The
  shop itself only lists identified potions and scrolls, so the rater's shape reaches the
  engine through a *vaulted* potion (kept unidentified, brought back later): before Cut 6
  `drink heal` could not find it (the trace read `no use`); now it drinks. A found, unnamed
  potion of the row's kind reads `unknown item` (a new block reason). The other way a
  `drink heal` row stays silent at 3 HP is a pre-emption — `cowardly → retreat` (three per
  fight below 50 %), paralysis, a hazard step — which the row accounting (§3) now names.
- **Row accounting** (§3): `TraceTurn.rows?: [{row, why}]` lists every row above the one that
  acted (all rows when a trait or a chore acted; absent when R1 acted) with a reason from the
  fixed table `turn::ROW_REASONS` (≤ 3 words; `turn::row_reason_ok` is the gate's check): the
  first failing condition (`hp not <30%`, `foes not ≥2`, `not in view`, `none held`, `no
  unknown`, `depth not ≥5`, `locked cond` …), the Cut 4 block reason when the conditions held
  (`no path`, `no target`, `cooldown`, `no item`, `unknown item` …), `card passed` (a tactic
  card that fell through), `brave held`, `fired, free` (a `recall`/`send` row that fired and
  let the list run on), the guards (`stuck`, `row guard`) and the pre-emptions (`trait first`,
  `hazard first`, `recall sense`, `paralysed`, `confused`, `bail`). Every turn of the trace
  carries it (the morgue prints it per turn); sims and replays skip the accounting (no trace
  is ever shown from them). A hunting row whose conditions lapsed still walks and counts as
  the fired row.
- **Counter facts carry the row** (§5): the telegraph fact is now
  `boss:goblin_warlord:counter=attack tag:boss` (`facts::boss_counter_fact`; the old key is
  its prefix, so `has_boss_counter` and the `stair_dance` gate still match). Rows per boss
  (`facts::counter_row`): Warlord `foe_tag:boss → attack tag:boss`, Bloat Mother `foe_tag:boss
  → throw fire,tag:boss`, Lich `foe_tag:summoned → attack tag:summoned`, Foundry Master
  `foe_tag:reflect_melee → tactic reflect_read`, Lurker Queen `foe_tag:boss → read silence`,
  Mirror King `foe_tag:boss → tactic cadence`. `Lineage.counters: [{boss, row, text}]` lists
  the known ones in floor order with ≤ 3-word texts (`attack boss`, `throw fire, boss`,
  `attack summoned`, `reflect read`, `read silence`, `cadence`). A save's old-form facts are
  upgraded on load (`facts::upgrade_counter_facts`). **Clients that matched
  `boss:<kind>:counter` exactly must prefix-match** (`facts::boss_counter_known`).
- **Boss deaths** (§5/§8): `DeathRec.boss` is the boss the hero died under (the cause, or a
  boss in view / awake within 8 tiles). When its counter is known, not already in the rules,
  in the death's vocabulary and fired in ≥ `FIRED_BAR` of the replays, the counter row is
  **pinned first** among the patches whatever its survival (`DeathRec.counter`; measured over
  the same 12 replays, exempt from the edge/delta cut, kept through the delta candidates).
  On any boss death the escape family (`return`, `bank`, `read teleport`) is ordered after the
  rest, and a lone escape patch is joined by the best other scored candidate (`trace::
  boss_order`). Not met when the counter row is not executable from the checkpoint (no fire
  potion for the Mother, no `throw`): the counter still shows on the banner and the forecast
  through `Lineage.counters`, and the patches are the ordinary ones.
- **`UnlockInfo.rows`** (§6; `meta::unlock_rows`): a tactic card's sub-rows in order, in the
  real vocabulary (`corridor_fighting`: `foes 2+ → corridor`, `corridor · adj 1+ → attack`,
  `corridor · foes 1+ → hold`, `foes 1+ → attack`; ≤ 5 rows, ≤ 3 conds), the mastery cards
  too; an automation is one row `{conds: [], verb: {v: "auto", a: "keeps best
  weapon+armour"}}` (`rebuys last supplies`, `insures brought items`, `eggs hatch 1 rest`, `5
  supplies`, `paths to bones`, `breeds 3 tags`, `+2 vision`); `recall_sense` is its real row
  `HP<15% → read recall`. Absent for rows, vaults, slots, classes, conditions and verbs.
- **Forecast determinism** (§9): the forecast's seeds are `forecast::forecast_tag(rules,
  lineage seed, depth)` (was one constant for every set), so re-reading the same set on the
  same lineage gives the same number and nothing transient (marks, renown, the rest clock, a
  run in progress) moves it — the lineage a sim starts from (facts, gold, class, trait, heir)
  still does, as it should. **`forecast_refine()`** (wasm `forecastRefine`) is the same
  forecast at `REFINE_SIMS` 100 sims — the same 50 seeds first, then 50 more — under twice
  the tick budget; the 50-sim default is unchanged. Patch deltas and the catalogue keep their
  paired constant tags.



`unlocks()` → `UnlockInfo[] {id,cost,owned,available,needs?}` · `setClass(class)` → Lineage ·
`selectSet(i)` → Lineage (three saved sets; `setRules` writes the active one) ·
`unlockDeltas()` → `UnlockInfo[]` with `delta` simulated (Cut 4) ·
`fromSave(json)` (static constructor) · `setKeepPref(pref)` → Lineage ·
Addendum A: `setParty(idsJson)`, `setCompanionRules(id, setJson)`, `breed(a,b)`, `hatch(eggId)`,
`companionVocabulary(id)` · Addendum B: `buySupply(kind)`, `clearSupplies()`,
`supplyCatalogue()` → `{kind,price,label}[]` · Addendum D: `keep(idsJson)` · Cut 3: `ascend(variant)` → Lineage ·
Cut 5: `bail()` · `choose(itemId)` → Snapshot · `setVaultPref(pref)` → Lineage ·
Cut 6: `forecastRefine()` → Forecast (100 sims).
Companion condition tokens: `self_hp< self_hp>` plus the hero set; companion verbs
`attack shoot burst steal split flank drain follow recall`. Hero scope cond `{k:"party",t:kind}`,
`{k:"party_hp<",n}`, verb `tame` (`nearest | tag:T`).

## Cut 7 (the first hour is not five floors) — deviations and additions

- **The descent** (§1). Warrens D1–8, Fens D9–13, Crypt D14–18, Foundry D19–23, Deep D24–28,
  Sanctum D29–33, the bottom at **D34** (`descent::ENDING_DEPTH`). Bosses one floor before
  each change: Warlord **D8**, Bloat Mother D13, Lich D18, Foundry Master D23, Lurker Queen
  D28, Mirror King D33 (`descent::BOSS_DEPTHS`, `boss_depth(kind)`, `biome_first(biome)`).
  The Cut 3–6 tuning that was keyed on depth (spawn groups, stat growth, item budget and item
  depths, the Foundry/Deep stock floors) runs on `descent::tier_depth`: D1–5 as before, D6–8
  at the old D5's numbers (one spawn group more), D9+ three floors shallower — so the Fens,
  Crypt and deeper biomes play exactly as they did three floors down. Every preset,
  `probes::full/countered`, the dayplayer's counter rows and the gate bars moved with the
  walls (`good.json` throws at bosses from D9; `full.json` at D19/D28/D33; `deep_march`
  reads `D24+`).
- **The Captain** (§1; `descent::lieutenant_for(5)`): `goblin_captain` (16 HP, 2–4, tags
  `summoner telegraph`, **not a boss**) is placed at D5 like a boss (by the stairs, goblins
  round him), rallies **once** (two goblins) on first sight and never shield-buffs; no counter
  fact, no marks, an ordinary ledger row. The stairs are not sealed by him (only bosses seal).
  DEFAULT leaves D5 alive in ~25–30% of the runs that reach it (the whole floor — archers
  in pairs, ogres — not the Captain alone; the walls above are the gates).
- **Preset and origins** (§2). The shipped fighter preset is two rows (`HP<30% → drink heal`,
  `foes 1+ → attack`), `max_rows` 4. **`Row.origin?: "preset" | "card" | "patch" |
  "player"`** (serde default none, skipped when none; **not part of a row's identity** —
  `Row: PartialEq` compares conds and verb only, so `rows.contains(&row)` and the patch
  machinery ignore it). The engine tags the preset rows on a new lineage
  (`probes::preset`), `meta::unlock_row` returns a card's row tagged `card`, and `set_rules`
  tags any bare `tactic` row `card`; the client tags `patch` / `player`.
- **Situations per band** (§3; `situations.rs`; `descent::SITUATION_DEPTHS`). Each is a
  bare fact, an `on_see: <fact>` token (gated by the fact alone, like `nest`), and an episode
  (`Episode.situation`, `Setup::Captive`; the sifter's subjects `The den` · `The lock` · `The
  captive` · `The hunger`). `Run.passed` records the ones answered; the offline batch keeps
  `Batch.band_runs: [{depth, met, passed}]` per real run for the gate.
  - **D3 the thief's den**: three monkeys (10 HP, def 2, `Monster.situation = "den"`) asleep
    within two tiles of the down stairs, never walling them off. A hero standing **on the
    stairs** is pounced on: each snatches one thing — a vault-brought pack item first, then
    any pack item, then the weapon in hand — and runs with it (a fleeing thief hides; a
    killed one drops it). A blow on a sleeper (the raid) wakes that one alone and it bolts
    for good, empty-handed; the raid is only for a row that names the den (`on_see: den` or
    `foe_tag thief` — `Run.raiding`), a plain `attack nearest` walks past sleepers as it does
    the nest's, and the chores route round them (`Run.sleepers`; a sleeper with no way round
    is shoved awake). Met on sight (the den is the stairs room); **pass** = leave D3 with
    every snatched item back on the hero (`Run.den_stolen`). Answer: `see den → attack`.
  - **D6 the gas lock**: every way into the down-stairs room — its doors and two corridor
    tiles behind each, up to `LOCK_MAX` 4 — holds a bloat (`situation = "lock"`). They stir
    when the hero is in sight or within 5 steps, drift at him (speed 7), **swell** the moment
    they stand beside him (`telegraph: "swells"`, `LOCK_FUSE` 10 ticks) and burst one action
    later: a burst hits a hero beside it for 3 at once (`Src::Burst`) and leaves a quick
    cloud (r1, 20 ticks). Met on arrival (the reek carries). **Pass** = leave D6 with ≤
    `LOCK_PASS_GAS` 6 of the lock's gas taken (`Run.gas_dmg_floor`: bursts, and cloud damage
    while its bloats stand or within 40 ticks of the last burst — a curious hero's caustic
    potion does not count). Answer: `gas · adj 1+ → retreat` (step back and let it pop at
    arm's length); the preset swings and eats each burst.
  - **D9 the captive gate**: a captive chained **on the down stairs** (`situation =
    "captive"`, neutral); no place-swapping past it. `free_captive` frees it (an ally for the
    Bloat Mother's floor) and passes; a chained captive **counts as a foe once the hero stands
    beside it**, so `attack nearest` cuts it down — the coward's way, trophy `no_friends` —
    and no set ever stalls at the gate. Met on arrival (its cry carries). Answer: `see captive
    → free`.
  - **D12 the crypt's hunger** (the Crypt reaches up into the Fens' last floors): a shrine
    4–7 steps down the path from the entrance and three wraiths already hunting
    (`situation = "hunger"`). Every `HUNGER_TURNS` 12 turns unlit the floor takes a point of
    max HP (never below 5; `Ev::hurt {dmg: 0, cause: "hunger"}`, callout `hunger`); light is
    a lantern in the pack or the shrine **lit**: `pray` on this floor lights it — free, no
    row lent or trait swapped, whatever was prayed above (`Run.lit`; the wraiths lose their
    nerve). Met by the first bite; **pass** = leave D12 lit. Answer: `see hunger → pray row`
    (`see shrine → pray row` works too, at the D1–5 shrines' price).
  - **The chores** learned two things for the lock that hold everywhere: lingering gas or
    fire is terrain when another way exists (chores, attack approaches; a target standing in
    it is waited out, not walked into), a hero deep in a cloud walks to the nearest clean
    tile instead of standing in it, and `retreat` weighs open ground (a step into a dead end
    scores a little lower).
  - **Gate probe** (`probes::situation_trial(seed, what, answered)`): a fighter shaped for
    the depth (L1/2/4/6, in mail below the Warrens' doorstep) starts on the band's floor with
    the shipped preset, plus `probes::situation_answer(what)` when answered, and plays the
    floor out. The metrics print appearance over every bot's real runs that reached the depth
    and both pass rates; the bars are appearance ≥ 90% and the preset ≤ 20%.
- **The watch as a scene** (§4). **`Snapshot.room?: {id, hostiles}`** — the room the hero
  stands in (1-based index into the floor's rooms; `0` for a corridor, a door or a cave) and
  the awake hostiles standing in it; **`Snapshot.rooms?: number`** the floor's room count.
  **`Ev::ending {t, ticks}`**: emitted before an exit the engine can foresee — `ticks: 30`
  when a `bank` row walks within three steps of the up stairs, when the bottom's stairs are
  within three steps, or when the hero is at ≤ 15% with a hostile adjacent (once per 100
  ticks); `ticks: 0` for an instant exit (`return`, recall, the bottom reached this action).
  Not renderable (the events-per-600-ticks gate ignores it).
- **Levels** (§5). `hero::xp_to_next` is `60·L²` for L1–3 (L4+ `100·L²` as before).
  **`Game.watched`** (serde default false): set by `send()` and `step()`, cleared by every
  offline batch; a watched **bank** earns +50% class XP (returns and sims never). The wasm
  bridge needs nothing new: the client's live loop is `send`/`step`.
- **The ledger's boss rows** (§1). `Lineage.ledger` now lists the six bosses after the kinds,
  in descent order; a boss whose counter is known carries **`LedgerRow.counter?: {row,
  text}`** (the same row and ≤ 3-word text as `Lineage.counters`). Clients counting "kinds
  known" over the ledger should skip boss rows (`defs::monster_def(kind).boss`).
- **Dayplayer** (§6). Two new bars: the **first hour** — three 20-minute check-ins of a
  *watched* player (the scene cadence averages ~4×, so a check-in is 48 000 ticks of play
  with no camp rest; between them the same hands as the fortnight: the worst death's patch,
  the bank row `HP<40% · D3+ → bank`, the situation answers as their facts land, the cheapest
  unlock) ends with best ≥ D6 and ≥ 2 player rows (any row not `preset`) on ≥ 80% of seeds;
  **L2 by the end of day 1** for the banking fortnight player ≥ 80%. The fortnight player now
  writes the bank row and the situation answers itself (`write_own_rows`); the content bars
  (purchase days, stall) stay informational.

## Cut 8B (rows become a roster) — deviations and additions

- **Combos are named** (§1; `rules::COMBOS`). A fixed table of adjacent-row verb pairs the
  engine already resolves as one move, each with a name the player did not write:
  `shield_bash → backstab | attack` **opener** · `throw → retreat | back_corridor` **hit and
  fade** · `taunt → cleave` **bait** · `shoot → kite | retreat` **kite** · `vanish → backstab`
  **ambush** · `pray → descend` **pilgrim** · `tame → send` **handler** · `drink unknown →
  attack` **gambler** · `back_corridor → attack` **chokepoint**. A pattern is a verb key
  (`throw` matches every throw) or `verb arg` (`drink unknown` only the gamble;
  `rules::verb_is`). `Vocabulary.combos: [{a, b, name}]` carries the table (empty for a
  companion's vocabulary); `rules::combos_in(set)` / `RuleSet::combos()` → `[{rows: [i, j],
  name}]` in row order (0-based, `j = i + 1`; a row may close one combo and open the next), on
  the wire as **`Lineage.combos`** (computed from the active set on every `lineage()`, so it
  follows `set_rules` and `select_set`). A card row (`tactic`) is never part of a combo.
- **The chronicle credits the combo** (§1). The heir line names the heir by its set's first
  combo (by row order) instead of the trait: `♟3 the chokepoint fighter · D7 · …`, a
  multi-word name hyphenated (`the hit-and-fade rogue`); a set with no combo reads as before
  (`the greedy fighter`). **Episodes credit combos** (`Episode.combo`): `Arc.acts` keeps the
  last four hero actions with their action numbers and `Arc.low_act` the number of the low
  point's act; when two adjacent rows fired *in row order* within the three hero actions
  around the low (the act before it, the act, the act after), the episode's turn beat reads
  `the bait landed` (short form one word: `the hit-and-fade landed`) in place of the single
  row; `story_ok` accepts both forms, `names_agent` counts them as a row. A pair out of order
  or further apart credits nothing.
- **Rogue free at the first bank** (§2). `rogue` costs **0** marks with the gate `bank once`
  (`banked_depths` non-empty: any `home:D*` best). Cut 8B moved the unlock catalogue's sum by
  −6 (rogue 4 → 0, tame 2 → 0).
- **A companion in the first hour** (§3). `tame` is **owned from the start** (cost 0; a new
  lineage and every ascension insert it) and the fact `item:leash` is held from the first camp.
  The **kennel's leash** (`LineageState::kennel_leash`, `Item.free`): while the lineage has
  never tamed (`tamed_kinds() == 0`) a free, known leash sits on the supply shelf — placed on a
  new lineage, back after every homecoming (`finish_run`, `keep`) and on ascension; never
  refunded by `clear_supplies`, never rebought by `auto_supply` (`last_supplies` skips free
  items; `auto_supply` restocks an empty-but-for-the-leash shelf). Clearing the shelf is still
  an order: the leash comes back only when the hero comes home. The **first stray**
  (`LineageState::first_stray`, `FIRST_STRAY_PCT` 80): on 80 % of lineages (a side stream off
  the lineage seed) a stray **jackal** with a fixed name waits on **D2 or D3**, the same jackal
  every run while nothing has been tamed and no companion has been lost (the Cut 5 stray takes
  over once one has); placed by `place_situations` through `Run.first_stray`, tames at
  `STRAY_TAME` 60 % like any stray, comes back named. Token **`on_see: stray`** enters the
  vocabulary once a stray has been seen (the `stray` fact, gated like the other situations;
  `Run::sees_situation("stray")` = a live stray in view); `probes::situation_answer("stray")`
  is `see stray → tame nearest`. *Deviation from the brief*: the brief placed the stray for
  `heir == 1` only; the first heir rarely lives to the first check-in that writes the tame
  row, so the stray waits (by depth and name) for every heir until a tame happens.
- **Dayplayer** (§3). `write_own_rows` writes the stray answer as its fact lands, before the
  band answers; the first hour's player, holding an answer the full set has no room for, buys
  the row unlock first (else the cheapest) and writes it. New bar: **first hour tamed ≥ 1 on ≥
  60 % of seeds** (80 % over 30 seeds; the misses are the 20 % of lineages with no first
  stray). The first-hour probe now returns `(best, player rows, level, tamed)`.
- **Tests** moved by the shelf: the arena starts with an empty pack (`arena_seed` clears the
  supplies), the supply tests mark the lineage as having tamed (`no_kennel_leash`), the
  stall-verdict seed runs 16 h (the D2 stray moved its bests).
- **Gate note**: the quick table (`node tools/gates.mjs`, 8 seeds × 3 verdicts = 237 sampled
  verdicts) reads the dice share at 5.4 % against the 5 % bar on this head; the full table
  (30 × 8 × 8 = 2116 verdicts) reads **4.1 %** (Cut 7 head: 4.4 %), and a 16-seed × 12-verdict
  A/B on the same code reads 4.3 % against the Cut 7 head's 4.7 %. The bar is unchanged; the
  quick sample is a different set of deaths, not a worse rate.

## Cut 9 (the last four points) — deviations and additions

- **`Vocabulary.locked`** (§1; `tokens::locked_conds`): every condition token the editor knows
  of but the lineage cannot use yet, `[{cond, needs}]`, `needs` ≤ 3 words — a cond unlock
  reads its price (`◆2`) or its own shut gate (`meet a foe`, `see alert rise`); `party_hp<`
  with the unlock but no companion `tame once`; `foe_hp<` `study a kind`; an unlearned tag of
  a kind the hero has met (`foe:jackal` held, `foe:jackal:pack` not) `fact: pack` — tags of
  kinds never met stay off the sheet; an unidentified potion or scroll `identify poison`; an unfound misc item
  `find: lantern`; an unseen situation `see: stray`. A token is never both open and locked;
  the number is the player's (`Cond::same_token` matches key and tag). Absent when empty; a
  companion's vocabulary locks nothing; `party: K` is not a gate. **`set_rules` refuses a
  row that uses a locked token**: `row 2: see stray is locked (see: stray)`. Sims skip the
  check (a verdict replay or a forecast takes the lineage's rows as they are), and
  **`Game::set_rules_raw`** writes rows past it for tests and tools (`metrics`' situation and
  replay-hash rigs, `stories`, `longrun`, `leak`, `probe`); a row that *becomes* locked in
  play (an ascension resets the cond unlocks, a companion is lost) still reads `locked cond`
  in the trace.
- **`needs` on every non-available unlock** (§2; `meta::needs`): the fact/trophy gate, else
  the missing prerequisite's id (`row5`), else the marks short (`◆2 more`). `available` is now
  exactly `!owned && needs.is_none()`. The forecast deltas still run for a card that is only
  short of marks (the marks are not a gate on the sim). Gate texts moved: `party_slot_2` and
  `cond_party_hp` read **`tame once`**, `party_slot_3` `tame 3 kinds`, `party_slot_4` `tame 6
  kinds`, `cond_alert` `see alert rise` (3 words).
- **Forecast `±`** (§3): `Forecast.depths[].pm` is the 95 % binomial half-width over the sims
  that ran, `1.96·√(p(1−p)/n)` (`forecast::half_width`; 0 at a share of 0 or 1). The refine
  pass was already the same seed sequence widened (`forecast_tag` + the sim index); its first
  50 results are the panel's, so the number moves by at most the noise of the second fifty
  and `pm` narrows. **Unlock deltas run the panel's seeds** (`meta::catalogue_with_deltas`):
  the base is `reach_counted` — the panel's tag and `FORECAST_SIMS` under
  `CATALOGUE_TICK_BUDGET` (raised 60 000 → 100 000, so a D1–5 lineage's 50 seeds fit and the
  base *is* the panel number) — and every candidate replays exactly the seeds the base ran
  (`reach_paired`, no budget), so a chip's delta is a paired difference over the same seeds,
  not a second 12-seed draw. Cost: ~1 s native at D3, ~3 s at D9+ (was 0.3 / 1.5 s); the
  client runs it off the main thread. `forecast_cache` now memoises `(reach, sims run)`.
- **Trace on every exit** (§5): `ExitLine.trace` — the run's last `EXIT_TRACE_LEN` 5 hero
  turns with the Cut 6 row accounting, read off the run's own 16-turn trace ring at
  `finish_run` (nothing more per tick) — on `ReturnReport.exits[]`, on the stall verdict
  (`Stall.trace`: the latest run the named row ended, `StallTally.traces`), and on the exit
  event (`Ev::Exit.trace`, beside `line`; the line's own copy is dropped there so the event
  carries it once). A death's line leaves `trace` out (`Death.trace` is the longer, 10-turn
  one).
- **Reel dedupe across absences** (§6; `sifter::reel(highlights, best_run, recent)`):
  `LineageState.reel_pairs` keeps the (threat, resolution) pairs of the last `REEL_ABSENCES` 3
  reels (each `report()` — a quick slice is a report — pushes its own), and the next reel
  skips them. Order: the best-depth run's closing episode leads (its latest *fresh* episode
  when the closing pair was shown lately), then three by rank where a turn beat that names a
  row or a combo (`sifter::names_row`) outranks `no row fired` and the score breaks ties,
  then `bones` fill to four. When nothing at all is fresh, the closing episode is the reel
  alone — the one repeat allowed (the alternative was an empty reel). The gate test allows
  exactly that lone line.
- **Graveyard keeps the last five deaths** (§7): `Grave.death_id` is the run id while the
  engine still holds the record — the last `KEPT_DEATHS` 5 graves only (`Game::prune_graves`
  on every death); `max_deaths` is floored at 5 on insert and on load, so a save that trimmed
  it still answers `death(id)` for those five. Older graves drop the id.
- **Pending lines in numbers** (§8): `R1 fired 0 of 15 runs: HP<30% → drink heal` over the
  absence's *real* runs (`Batch.run_ticks.len()`, the same window as the reel; a sampled
  absence's extrapolated runs are not counted). `never fired` is gone. A chunked absence
  reports per slice; the client sums the numbers if it merges slices.
- **Forge rungs** (§10): `ForgeRow.next: {need, label}` — the ladder `FORGE_LADDER` 5
  `craftable` · 15 `tier 1` · 40 `tier 2`, absent at the top; `ForgeRow::settle` /
  `ForgeRow::at(n)` keep `craftable`, `tier` and `next` in step, and `to_wire` settles an older
  save's rows.
- **Tests**: 228 (+9). The forecast test times its own refine bound (`worst overshoot of 2·pm`
  prints 0.000 over 30 seeds). `tools/gates.mjs` (quick) reads the same 237 verdicts as the
  Cut 8B head (dice 5.4 % on the quick sample; see the Cut 8B gate note for the full-table
  number).

## Layout

`src/` per `docs/CUT1.md` plus `wire.rs` (the wire structs), `situations.rs` (the Cut 7 band
situations) and `tests.rs` (integration tests). `examples/cli.rs` playtest; `examples/metrics.rs` gates; `examples/bench.rs` timing.
`presets/{fighter,rogue,ranger,caster,good,full}.json` (`full` is the Cut 3 FULL bot: every
counter, every unlock assumed). `examples/dayplayer.rs` is the 14-day player simulation
(`--gate` checks the Cut 2/3 bars; `--verbose` logs purchases and bests; after the ending it
ascends with `no_rest`). `examples/probe.rs --full` and `examples/watch.rs` are the Cut 3
survey tools; `examples/stories.rs` the Cut 5 story probe.

```
cargo run --release --example cli -- --seed 1 --rules presets/good.json --runs 3 [--verbose] [--all-deaths]
cargo run --release --example metrics [-- --seeds 30 --hours 8]
```

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`, Cut 7; `node tools/gates.mjs --full`)

```
DEFAULT dies by ≤ D8 ≥ 80% of seeds                                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                                100%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts                               100 pts  PASS
RANDOM loses 100%                                                  100%  PASS
PASSIVE loses by ≤ D3 100%                                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2                           5.18 vs 4.90  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds                               100%  PASS
TRIVIAL never passes D8 ≥ 90% of seeds                             100%  PASS
COUNTERED reaches ≥ D14 ≥ 50% of seeds                              53%  PASS
FULL reaches ≥ D29 ≥ 50% of seeds (3 × 8 h)                         87%  PASS
FULL−D23 never passes D23 ≥ 90% of seeds                           100%  PASS
FULL−D28 never passes D28 ≥ 90% of seeds                            97%  PASS
FULL−D33 never passes D33 ≥ 90% of seeds                            97%  PASS
Unfair deaths (dice) ≤ 5% (n=2073, death-weighted)                 4.7%  PASS
Deaths tracing to a row (gap) ≥ 70%                               95.3%  PASS
Top death cause share < 35% (goblin)                              19.1%  PASS
Events per 600 ticks (renderable) ≥ 6                              43.7  PASS
Replay hash identical (seed+rules+elapsed)             8808d0472b75d688  PASS
Forecast known_to == best_depth + 1                                 all  PASS
Expeditions per 8 h (DEFAULT, EDITED) in 6–16                15.4 · 9.5  PASS
DEFAULT yields 0 xp/gold over 8 h                                     0  PASS
EDITED banks ≥ 3 runs per 8 h                                       6.9  PASS
Patches whose row fired in ≥ 50% of replays (n=262)                100%  PASS
Verdict time ≤ 0.4 s (mean of 2073)                              0.37 s  PASS
Per-tick cost ≤ 6 µs (quiet, DEFAULT/EDITED/FULL)               2.92 µs  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed              min 18  PASS
Story lines ≤ 12 words, table verb (n=70639)                     100.0%  PASS
Reel ≥ 2 distinct (threat, resolution) pairs per 8 h (n=180)               100%  PASS
Top reel line names a row, trait or companion ≥ 80%                 91%  PASS
Situations: ≥ 1 per run on D1–5 ≥ 90% (n=5952)                    94.9%  PASS
Situation den (D3) appears when reached ≥ 90% (n=4725)                98%  PASS
DEFAULT passes the den ≤ 20% of seeds                                0%  PASS
Situation lock (D6) appears when reached ≥ 90% (n=3448)               100%  PASS
DEFAULT passes the lock ≤ 20% of seeds                               3%  PASS
Situation captive (D9) appears when reached ≥ 90% (n=2224)               100%  PASS
DEFAULT passes the captive ≤ 20% of seeds                            0%  PASS
Situation hunger (D12) appears when reached ≥ 90% (n=2077)               100%  PASS
DEFAULT passes the hunger ≤ 20% of seeds                             0%  PASS
```

```
captain: DEFAULT left D5 alive in 41% of 282 runs that reached it
  den      D3   appears   98%   preset pass    0% (left  73%)   answered pass   97% (left 100%)   row: see den → attack
  lock     D6   appears  100%   preset pass    3% (left 100%)   answered pass   60% (left 100%)   row: gas · adj 1+ → retreat
  captive  D9   appears  100%   preset pass    0% (left 100%)   answered pass  100% (left 100%)   row: see captive → free
  hunger   D12  appears  100%   preset pass    0% (left 100%)   answered pass  100% (left 100%)   row: see hunger → pray row
```

The walls moved, not the bars: DEFAULT 100% ≤ D8 (was ≤ D6), TRIVIAL 100% ≤ D8, EDITED 100%
≥ D10, COUNTERED 53% ≥ D14 (was 70% ≥ D11 — the three extra Fens floors and the situations
shave it; it answers the hunger and cuts the captive down), FULL 87% ≥ D29 (was 97% ≥ D26 —
three floors more of the Sanctum's approach in the same 3 × 8 h), the FULL−counter walls
hold at D23/D28/D33. DEFAULT's run histogram: D5 61% · D6 25% · D7 7% · D8 5% · D9 0 — the
Captain's floor takes a third, the lock a fifth, the Warlord the rest. The four situations
appear on 98–100% of the runs that reach them; the preset passes none but the lock's 3%; the
one-row answers pass 97 / 60 / 100 / 100% (the lock's `gas · adj 1+ → retreat` is a partial
answer: four drifting bloats can box a hero in; a second row — `foes 2+ → corridor` — closes
it). Dayplayer: first hour 100% (best D6–8, 2–3 player rows, L3–4 by its end on the watched
bonus), L2 by day 1 100%, L10 on day 9; the content bars stay informational (6.0 purchase
days; the stall bar is a seed that farms D9–13 with a `return` patch). Class XP is keyed on
the content depth too, so a Fens floor is worth what it was.

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`, Cut 6; `node tools/gates.mjs --full`)

```
DEFAULT dies by ≤ D6 ≥ 80% of seeds                                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                                 93%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts                                93 pts  PASS
RANDOM loses 100%                                                  100%  PASS
PASSIVE loses by ≤ D3 100%                                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2                           4.70 vs 4.58  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds                               100%  PASS
TRIVIAL never passes D5 ≥ 90% of seeds                              97%  PASS
COUNTERED reaches ≥ D11 ≥ 50% of seeds                              70%  PASS
FULL reaches ≥ D26 ≥ 50% of seeds (3 × 8 h)                         97%  PASS
FULL−D20 never passes D20 ≥ 90% of seeds                           100%  PASS
FULL−D25 never passes D25 ≥ 90% of seeds                           100%  PASS
FULL−D30 never passes D30 ≥ 90% of seeds                           100%  PASS
Unfair deaths (dice) ≤ 5% (n=2177, death-weighted)                 4.4%  PASS
Deaths tracing to a row (gap) ≥ 70%                               95.6%  PASS
Top death cause share < 35% (goblin)                              27.1%  PASS
Events per 600 ticks (renderable) ≥ 6                              44.0  PASS
Replay hash identical (seed+rules+elapsed)             0b69289ba09d19c4  PASS
Forecast known_to == best_depth + 1                                 all  PASS
Expeditions per 8 h (DEFAULT, EDITED) in 6–16               15.6 · 12.4  PASS
DEFAULT yields 0 xp/gold over 8 h                                     0  PASS
EDITED banks ≥ 3 runs per 8 h                                       8.0  PASS
Patches whose row fired in ≥ 50% of replays (n=280)                100%  PASS
Verdict time ≤ 0.4 s (mean of 2177)                              0.35 s  PASS
Per-tick cost ≤ 6 µs (quiet, DEFAULT/EDITED/FULL)               2.83 µs  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed              min 19  PASS
Story lines ≤ 12 words, table verb (n=73459)                     100.0%  PASS
Reel ≥ 2 distinct (threat, resolution) pairs per 8 h (n=180)               100%  PASS
Top reel line names a row, trait or companion ≥ 80%                 94%  PASS
Situations: ≥ 1 per run on D1–5 ≥ 90% (n=6611)                    95.1%  PASS
```

Cut 6 changes no run outcome (the sims are byte-identical: same depths, deaths, expeditions);
the replay hash moved because `Ev::exit` carries the ledger line and the counter fact carries
its row. Verdict time 0.35 s (the counter pin is 12 more replays on boss deaths), per-tick
2.83 µs (the row accounting is skipped in sims). Cut 6 unit gates: ledger reconciles on every
exit over 30 seeds (`ledger_line_reconciles_on_every_exit`), a bought heal fires
(`bought_heal_is_drunk_by_name`), 100 deaths' traces account for every row above the fired
one on every turn (`death_traces_account_for_every_row_above_the_fired_one`), 30 Warlord
deaths show the counter row first and never a lone `return`
(`boss_deaths_show_the_counter_row_first`), the forecast is deterministic
(`forecast_is_deterministic_per_rules_and_lineage`). The dayplayer bars are unchanged (7.0
purchase days, 4-day stall — the Cut 2–5 deviation).

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`, Cut 5; `node tools/gates.mjs --full`)

```
DEFAULT dies by ≤ D6 ≥ 80% of seeds                                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                                 93%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts                                93 pts  PASS
RANDOM loses 100%                                                  100%  PASS
PASSIVE loses by ≤ D3 100%                                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2                           4.70 vs 4.58  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds                               100%  PASS
TRIVIAL never passes D5 ≥ 90% of seeds                              97%  PASS
COUNTERED reaches ≥ D11 ≥ 50% of seeds                              70%  PASS
FULL reaches ≥ D26 ≥ 50% of seeds (3 × 8 h)                         97%  PASS
FULL−D20 never passes D20 ≥ 90% of seeds                           100%  PASS
FULL−D25 never passes D25 ≥ 90% of seeds                           100%  PASS
FULL−D30 never passes D30 ≥ 90% of seeds                           100%  PASS
Unfair deaths (dice) ≤ 5% (n=2177, death-weighted)                 4.4%  PASS
Deaths tracing to a row (gap) ≥ 70%                               95.6%  PASS
Top death cause share < 35% (goblin)                              27.1%  PASS
Events per 600 ticks (renderable) ≥ 6                              44.0  PASS
Replay hash identical (seed+rules+elapsed)             7e0865b3c8f23ecf  PASS
Forecast known_to == best_depth + 1                                 all  PASS
Expeditions per 8 h (DEFAULT, EDITED) in 6–16               15.6 · 12.4  PASS
DEFAULT yields 0 xp/gold over 8 h                                     0  PASS
EDITED banks ≥ 3 runs per 8 h                                       8.0  PASS
Patches whose row fired in ≥ 50% of replays (n=278)                100%  PASS
Verdict time ≤ 0.4 s (mean of 2177)                              0.34 s  PASS
Per-tick cost ≤ 6 µs (quiet, DEFAULT/EDITED/FULL)               2.87 µs  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed              min 19  PASS
Story lines ≤ 12 words, table verb (n=73459)                     100.0%  PASS
Reel ≥ 2 distinct (threat, resolution) pairs per 8 h (n=180)               100%  PASS
Top reel line names a row, trait or companion ≥ 80%                 94%  PASS
Situations: ≥ 1 per run on D1–5 ≥ 90% (n=6611)                    95.1%  PASS
```

Cut 5 measured: FULL 97 % of seeds to D26 (83 % at the Cut 4 head: the vault's +1 gear and
the dens' tames), the boss walls 100 / 100 / 100 %. Dice 4.4 % death-weighted (3.9 % at the
head under the same estimator; every death verdicted 3.8 %). DEFAULT's runs are the Cut 4
shape (D4 reach 94.9 % vs 95.3 % without the situations; expeditions 15.6 vs 15.7). The
14-day player (`dayplayer --gate --seeds 3`): marks unspent ≤ 8 (6), empty check-ins 0 %,
L10 on day 11 — PASS; days with a purchase 7.7 / 14 (bar 10) and the longest counter-known
stall 8 days (bar 3; 5 at the Cut 4 head) — FAIL, the Cut 2–4 deviation (the model's boss
play).

## Gate table (30 seeds × 8 h offline, `examples/metrics.rs`, Cut 4; `node tools/gates.mjs --full`)

```
DEFAULT dies by ≤ D6 ≥ 80% of seeds                                100%  PASS
EDITED reaches ≥ D10 ≥ 50% of seeds                                 83%  PASS
EDITED − DEFAULT (≥ D10) ≥ 15 pts                                83 pts  PASS
RANDOM loses 100%                                                  100%  PASS
PASSIVE loses by ≤ D3 100%                                         100%  PASS
LEARNED mean depth ≤ DEFAULT + 2                           4.76 vs 4.60  PASS
PETS dies by ≤ D8 ≥ 80% of seeds                                   100%  PASS
LEVELLED dies by ≤ D9 ≥ 80% of seeds                               100%  PASS
TRIVIAL never passes D5 ≥ 90% of seeds                             100%  PASS
COUNTERED reaches ≥ D11 ≥ 50% of seeds                              63%  PASS
FULL reaches ≥ D26 ≥ 50% of seeds (3 × 8 h)                         83%  PASS
FULL−D20 never passes D20 ≥ 90% of seeds                           100%  PASS
FULL−D25 never passes D25 ≥ 90% of seeds                           100%  PASS
FULL−D30 never passes D30 ≥ 90% of seeds                            90%  PASS
Unfair deaths (dice) ≤ 5% (n=2186)                                 4.7%  PASS
Deaths tracing to a row (gap) ≥ 70%                               95.3%  PASS
Top death cause share < 35% (goblin)                              29.1%  PASS
Events per 600 ticks (renderable) ≥ 6                              44.3  PASS
Replay hash identical (seed+rules+elapsed)             d174319c550a9654  PASS
Forecast known_to == best_depth + 1                                 all  PASS
Expeditions per 8 h (DEFAULT, EDITED) in 6–16               15.7 · 13.0  PASS
DEFAULT yields 0 xp/gold over 8 h                                     0  PASS
EDITED banks ≥ 3 runs per 8 h                                       8.4  PASS
Patches whose row fired in ≥ 50% of replays (n=265)                100%  PASS
Verdict time ≤ 0.4 s (mean of 2186)                              0.29 s  PASS
Per-tick cost ≤ 6 µs (quiet, DEFAULT/EDITED/FULL)               2.94 µs  PASS
Offline 8 h: learned ≥ 1 and pending ≥ 1 every seed              min 18  PASS
```

Verdicts under the Cut 4 semantics: gap 95.3 %, dice 4.7 % (5.3 % with `HISTORY_TURNS` 40
and two verdict delta candidates; 4.9 % with 40 and three — the window stays at 30). Before
the replay-window fix the same semantics read 13.9 % dice at 1.25 s per verdict. `death()`
with every delta 1.4 s (was 2.2 s). FULL 83 % of seeds to D26 (87 % in Cut 3; the boss walls
hold 100 / 100 / 90 %).

The 14-day player (`examples/dayplayer.rs --gate --seeds 3`, Cut 4 model: takes the report's
stall patch, declines a death patch that reads `reach −N%`, plays a bought card as a row, arms
a boss counter after one stalled day and buys what it needs): marks unspent ≤ 8 (7), empty
check-ins 0 %, L10 on day 10 — PASS; **days with a purchase 8.3 / 14 (bar 10) and the longest
counter-known stall 5 days (bar 3) — FAIL** (Cut 3: 8.0 and 5). Final depths 19 / 19 / 18 in
14 days; the same model without the `reach −N%` decline reached 25 / 31 (ascended day 8) / 20
with 8.7 purchase days and an 8-day stall in the second act. The stalls are now the Bloat
Mother (D10: the counter fact arrives only once the hero survives to meet her; 4–7 days at D10
before `boss:bloat_mother:counter` is known, which the bar does not count) and the Lich (D15:
the hand-written counter rows `attack tag:summoned` / `attack tag:boss` lose to the chant
loop for 4 days). Income after the cheap catalogue is 1–3 marks a day (ranks, first banks)
against 3–12 per item, so purchase days track depth progress. The tier-2 costs and first-bank
marks are in; the remaining lever is the player model's boss play, which is a design call.

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
