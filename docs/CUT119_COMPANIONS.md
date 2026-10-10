# Cut 119: companions with depth

*2026-10-10. Owner: "do your improves to pet system. research/have an eval framework for this mechanic".
Today (map, 2026-10-10): taming odds rise with knowing a foe; strays and loss-eggs; per-pet rule rows; pets fall
back when hurt. Shallow: a level is +2 HP and one rule row (no attack, cap L5, levels only on a bank exit);
breeding passes tags only (kind is always parent A's; generation does nothing); 6 tag→verb mappings; a bare kennel list.
Starts after Cut 118's core lands (shared files). No added required taps (owner rule, Cut 118 amendment 2).*

## Items

1. **The pet carries the line through death.** When the heir falls and a party pet survives, it carries a share of
   the heir's carry home (the rest is the grave's, Cut 118). It is recovered carry, not new gold, and appears in the
   gold ledger as `fetched`. The pet takes a grudge against the killer: a small bonus against that boss, adding to the
   siege. A pet that served ≥ 3 heirs is the line's `old hound` (chronicle, graveyard).
2. **Roles.** Each kind has one role, first on its card: `fetcher` (grabs loot within reach: shortens the D1–D4
   pickups), `guard` (draws adjacent blows), `scout` (reveals the floor ahead), `mender` (heals the hero between
   fights). Role = kind's base behaviour; tags still add verbs.
3. **Levels that matter.** XP from every run (depth and kills), real stats per level (hp and attack), a fixed
   signature at L3 and L5 per role (no choice: fetcher `carry more` → `bring back`, guard `taunt` → `bulwark`, scout
   `map` → `warn`, mender `patch` → `revive once`). Cap L7.
4. **Breeding as an order.** The kennel keeper's standing order `breed for the wall · keep the best · off` breeds
   while away (a control-ladder rung); `pedigree`: each generation +2 % to the pet's stats (cap +10 %); the egg's kind
   from either parent (the order picks).
5. **Pet + build synergies.** Named pairs with one effect each, like Cut 115's: e.g. `Packmaster` (pack break +
   a guard), `Falconer` (kite archers + a scout), `Quartermaster` (light hands + a fetcher). On the build name when formed.

## The eval framework (gates)

A probe, `examples/pets_eval.rs`, and a targeted gate row read the companion system on fixed seeds (16 seeds × 14
days of the dayplayer's PICKED and IDLE bots, plus paired sends with and without the party). Baseline recorded before
any change; the same table after.

| metric | what it tells | bar after Cut 119 |
|---|---|---|
| pet value: paired sends, party vs none (past/bank/death pts at each band wall) | a pet matters | ≥ +5 pts at ≥ 3 walls; never > +25 (no auto-win) |
| role spread: the best role per wall | no dominant role | each role best at ≥ 1 wall |
| pet share of damage dealt / blows drawn | pets are seen acting | 10–35 % |
| pet deaths per 10 runs; returns as stray/egg | loss is felt but recoverable | ≤ 2 per 10; ≥ 80 % recovered by day 14 |
| level curve: median pet level at days 3 / 7 / 14 | growth is visible | L2 / L4 / L6 |
| carry fetched on deaths (share of lost carry) | death softened | 15–35 % |
| D1–D4 pickup time (ticks) with a fetcher vs without | the pickup gripe | ≥ 25 % shorter |
| taps per return with pets (cut118 taps test) | no micromanagement | no increase |
| IDLE bars, PICKED bars, stance, dice ≤ 5 % | nothing else moves | hold, no lowered threshold |

The blind cohort brief adds one free-text prompt: "the companions: what did they do, and did you care?" Scored by
mentions (pets named among highlights or gripes) across cohorts; not a gate.

## Research amendments (research/PETS_2026-10.md)

- §1 carry home:
  - **invariant:** a death with a pet never pays better than a return (the fetched share plus the heir purse floor
    stays below the 60 % return share);
  - the grudge is capped and cleared when that boss falls;
  - one reel line names the pet (`Rook brought Ada's pack`).
  - **A pet goes down, not gone:** a pet killed is lamed for N runs with its level kept; permanent loss is rare,
    with a named cause.
  - The old hound's chronicle line names the heirs it served.
- §2 roles:
  - the fetcher's late job is `bring back` (death insurance);
  - strays arrive with a balanced mix of roles;
  - `revive once` is once per run and never in a boss fight.
- §3 levels:
  - L3 and L5 are announced once;
  - bench pets earn 25 % XP;
  - the signature by about day 9.
- §4 breeding:
  - eggs accumulate while away, up to a capacity (Idleon);
  - the generation is in the name (`Rook III`);
  - one report line;
  - surplus is released automatically;
  - `keep the best` scores pets by the forecast wall.
- §5 synergies:
  - formed from the role and the build alone (no level gates);
  - named once;
  - drop any pair forming on < 25 % of PICKED seeds.

**Eval table revisions:**
- pet value: points of the camp score (past/bank/death as in builds.rs), outside the seed spread;
- role spread:
  - "best" needs a margin of ≥ 1 seed SD;
  - pick-rate cap: no role > 50 % of PICKED party slots, no kind > 35 %;
- damage share applies to guards only; every role ≥ 3 traced acts per run;
- recovery: median ≤ 1 day; every pet death names a cause;
- level curve: the longest-serving pet;
- carry: death-with-pet < return;
- no manual breed needed for pedigree.

New rows:
- tenure (median heirs served; an old hound on ≥ 50 % of seeds by day 14);
- ≥ 1 report or reel line naming a pet per return;
- synergy formation rate;
- bred vs wild in use;
- IDLE's gain from pets;
- pets cause no dice death.

**Cohort brief:** ask for an unprompted highlight first, then the pet prompt, plus name recall ("what was your pet called?").

## What the baseline changed (probe `examples/pets_eval.rs`, worktree commit 549a833)

- **IDLE and PICKED never own a pet:** 0 tames in 17 533 live runs; no package carries a `tame` row; only the pen
  writes one. Hence a new item 0:

  **0. Pets come to everyone.** With a leash held and a party slot open, a default `tame` row (below the guard rows,
  above `explore`) tames a stray or a weakened tameable foe. The kennel's free leash makes the first pet arrive
  on its own, early. It is announced once, as drills are (`tames strays`), and it can be revoked. Gate: a pet in the
  party on ≥ 12/16 seeds by day 3, for IDLE and PICKED; the IDLE bars hold.
- **One kind dominates:** the goblin archer is best at every wall (margin 0.05–0.87 SD), and a single archer is
  worth +36 points at D13, above the +25 cap. Roles (§2) must rebalance; the ranged pet's damage is capped (the
  Hades cap).
- **TUNED's pets die on nearly every send (11.1 falls per 10 runs) and never level (L1 at day 14):** levels only come
  on a bank exit. §3's XP from every run and "down, not gone" (lamed, level kept) are required, not optional.
- **D1–D4 pickups take about 59 % of the early ticks** (≈ 997 of 1 700 per send from D1): the fetcher target
  (≥ 25 % shorter) is meaningful.
- **Probe cost:** 9 min fresh on a loaded box, 32 s with the fortnights cached.
## Baseline (pre-Cut 119)

*2026-10-10, tree 0b1c62e + the probe's counters. `cargo run -q --profile fast -p riddle-core --example pets_eval -- --seeds 16`
(`--seeds N`, `--sims 24`, `--kind-sims 12`, `--days 14`, `--bots idle,picked[,tuned]`): 16 seeds × 14 days of the
dayplayer's IDLE and PICKED (`dayplayer::walk`, 3 check-ins), paired panels from each bot's wall snapshots (the first
check-in under D8/13/18/23/28; sends from the wall's stone, drills revoked, as `builds.rs`), score = builds.rs's camp
score in points (`past + 0.3·reach + 0.2·bank − 0.2·death + 0.1·mean depth`). Counters: `riddle_core::petstats`
(per-thread, write-only; the 307dbed hash and the suite hold). Deterministic (sequential and parallel-sims runs print
the same table). 9 min fresh on a shared 32-core box (189 CPU-min; the fortnights are the cost), 32 s with the
fortnights kept (`RIDDLE_SRC_KEY=<any>`, `jobcache_lib`).*

**The headline: IDLE and PICKED never own a pet.** 0 tames in 17 533 live runs (IDLE 6 220, PICKED 11 313): no package
holds a `tame` row, the stray's row is TUNED's pen alone, and the kennel's free leash is never used. Every fortnight
row reads zero for them; the paired "party worn vs removed" panels are identical (0 snapshots with a party of 160).

| metric | IDLE | PICKED |
|---|---|---|
| walls reached (seeds) D8/13/18/23/28 | 15/13/1/11/16 | 9/8/4/6/15 |
| pet value: worn − removed, Δscore ± seed SD, every wall | +0.0 ± 0.0 (no party) | +0.0 ± 0.0 (no party) |
| pet share of damage dealt / blows drawn (worn) | — (no pet) | — (no pet) |
| acts per run per pet | — | — |
| pet deaths per 10 runs · named cause · recovered by day 14 · median days to recovery | 0 · — · — · — | 0 · — · — · — |
| level curve, all pets / longest-serving, days 3 · 7 · 14 | none owned (0/16 seeds) | none owned (0/16 seeds) |
| tenure (median heirs served) · old hound by day 14 | — · 0/16 | — · 0/16 |
| pets owned / in party, days 1 · 3 · 7 · 14 | 0 / 0 throughout | 0 / 0 throughout |
| pick rate per kind (party slots) | — | — |
| D1–D4 pickup walk, per send from D1 (D8 snapshot) | 997 of 1 701 ticks (58.6 %), 88.9 steps | 996 of 1 678 ticks (59.4 %), 87.8 steps |
| carry fetched on deaths | 0 % | 0 % |

**A pet given** (the best-kind panels: each kind alone in the party at the lineage's best pet level — L1 here — on both
bots' wall snapshots, 12 sims; Δscore vs no pet; margin = (best − 2nd) / SD of their paired difference):

| wall (snaps) | best kind (tags) | Δ vs none | margin | the rest |
|---|---|---|---|---|
| D8 (24) | goblin_archer (ranged) | +9.6 | 0.42 SD | pink_jelly +3.9 · skeleton +3.8 · monkey +2.3 · ogre +2.3 · jackal +2.2 · rat +1.7 · bloat +1.2 |
| D13 (21) | goblin_archer | +36.1 | 0.87 SD | pink_jelly +9.5 · jackal +6.9 · skeleton +6.1 · bloat +4.8 · rat +3.4 · monkey +2.4 · ogre +2.3 |
| D18 (5) | goblin_archer | +8.0 | 0.05 SD | pink_jelly +6.8 · skeleton +5.5 · ogre −2.5 · jackal −6.0 · bloat −10.8 · monkey −11.2 · rat −17.2 |
| D23 (17) | goblin_archer | +8.9 | 0.33 SD | pink_jelly +2.3 · monkey +0.5 · ogre −1.5 · skeleton −2.9 · rat −2.9 · jackal −3.1 · bloat −5.4 |
| D28 (31) | goblin_archer | +7.3 | 0.36 SD | skeleton +4.0 · pink_jelly +3.7 · bloat +1.9 · ogre +1.6 · jackal +1.5 · monkey +1.2 · rat +0.6 |

One kind is best at every wall (the ranged archer's `shoot` row), never by ≥ 1 seed SD; a pet's worth already reaches
+36 at D13 (the "never > +25" bar). Pet share, all walls pooled (damage dealt · blows drawn · acts per run): archer
22.4 % · 4.9 % · 220; jackal 3.3 % · 7.1 % · 160 (`flank`); skeleton 5.8 % · 11.2 % · 36; ogre 3.5 % · 12.8 % · 21;
monkey 1.8 % · 8.4 % · 20; pink_jelly 1.0 % · 5.2 % · 18; rat 0.9 % · 6.7 % · 10; bloat 0.0 % · 7.8 % · 8.5. No kind
draws more than 13 % of blows (no guard exists); only the archer reaches the 10–35 % damage band.

**Reference, TUNED (4 seeds, `--bots tuned`; the only bot that tames: the pen's stray row):** 36 tames in 2 831 runs;
pets fell 3 146 times = 11.1 per 10 runs (a fielded pet dies on nearly every send), 100 % named, 100 % recovered as
loss-eggs (3 123) or strays (33), median 0 days (inside the check-in); every pet stays L1 (median level 1.0 on days
3/7/14; the longest-serving L1.0–1.5); median tenure 1 heir, no old hound; owned 1.0/1.8/3.2 on days 3/7/14; party
slots jackal 80 % · mirror_shade 7 % · echo 7 % · sentinel 5 %. Its wall snapshots still held no party (0/11).

## Cut 119 core: built (2026-10-10, uncommitted)

All truth in `crates/riddle-core/src/pets.rs` (`LineageState.pets: PetsState`, `Run.pets: RunPets`, `Companion.life:
PetLife`, all serde-default: an older save loads with Cut 119 on and its pets at zero), with hooks in `engine.rs` (the send,
the run's end, the hatch, the stray, the ledger term), `ai.rs` (the tame row's `open`, the roles' acts, the pet's blow, the
limp home), `turn.rs` (the guard's draw, the scout's `warn`, the mender's `revive once`, the scout's look ahead, down not
gone), `packages.rs` (the tame row compiled, the build's name), `feats.rs` (the grave less the fetched share), `tree.rs` (the
kennel-hand fields a mix). `pets::pin_off` pins every Cut 119 system off (and recompiles: the tame row out).

- **0, pets for everyone.** The default tame row `foes ≥ 1 → tame open` (origin `drill:tame`; `pets::tame_row`) compiles
  below the stance's guard rows and the tactics/temperament, above the fallback (`attack nearest`, `rest`); never in a
  custom (pen-written) stance. `open` tames a stray, or a weakened (< 25 % hp) foe whose kind is known well enough
  (`tame_chance ≥ 40`) and whose role the party lacks, only with a leash, a party slot open, and no band boss standing on
  the floor. The first stray now waits on every lineage (was 80 %), its kind drawn across the roles (jackal · skeleton ·
  goblin archer · pink jelly). Announced once (`TAMES STRAYS`, a report beat), revocable (`revokeTame(bool)`, or
  `revokeDrill("tame", bool)`). Every bot owns a pet in its party by day 3 (16/16 IDLE and PICKED).
- **1, the pet carries the line.** A heir's death with a party pet standing brings `pets::fetch_pct` of the lost carry home
  (20 %; a fetcher 24 %; its `bring back` 28 %), out of the grave (`feats::fallen` takes the rest: no gold made, test), as
  the ledger's `fetched` term (`fetched by Rook`), a news line (`Rook brought Ada's pack · $N`) and a reel note. The
  research invariant holds by construction: 28 % + the 30 % heir purse floor < the 60 % return. The killer, a band boss,
  becomes the pet's `grudge` (+1 a blow on him, cleared when he falls). A pet that served 3 heirs is the line's **old
  hound** (`Lineage.pets.old_hounds`: `Rook · old hound · served Ada, Bram, Cole`, a heir deed, a news line).
  **Down, not gone:** a blow from above its fall-back line leaves a pet at 1 hp; a wounded pet (≤ 30 %) **limps home**
  (out of that run, safe; callout `Rook limps home`); a wounded pet cornered before it acts falls — **lamed** for
  `LAME_RUNS` 3 runs, its level kept (`Fallen.lamed`, the cause named), gone only at its 6th fall (the old egg/stray path).
- **2, roles** (`pets::role_of`, first on the card: `PetLife.role`): **fetcher** (rat, jackal, monkey, forge imp — walks to
  the seen gold and the items the hero would take within 14 tiles of him, L3 18, and brings them to his hand; keeps on while
  no foe is within 3), **guard** (the rest — takes a melee blow aimed at the hero from a foe beside it, 65 %, L3 `taunt`
  80 %, L5 `bulwark` at half damage), **scout** (archer, bell sentinel, lurker, wraith, siren, conjurer, eels — sees the
  stairs down and the tiles round them, radius 4, L3 `map` 8; L5 `warn` halves a floor's first blow), **mender** (pink
  jelly, bloat, acolyte, mirror shade, echo, smith — heals the hero 1 hp, L3 `patch` 2, every 90 ticks between fights while
  he is under 60 %; L5 `revive once`: a killing blow leaves him at a quarter, once a run, never with a band boss standing).
  The archer's cap: a ranged pet's shot is capped at `1 + level/3`.
- **3, levels.** xp from every run (the depth reached; bench pets 25 %), `LEVEL_XP` 500 · 1 200 · 2 400 · 4 000 · 6 000 ·
  9 000 to L2…L7 (cap 7); +4 hp a level, attack +1 every two levels (+1 floor at L4/L7); the L3 and L5 signatures announced
  once (`Rook L3 · taunt`, a report best and a note).
- **4, the kennel keeper** (`StandingSwitches.kennel` / `StandingOrders.kennel`: `breed` default · `best` · `off`): an egg
  every 20 runs while two eggs at most wait (no tap): the best pet (level, pedigree, the wall's role) and the best of
  another kind are the parents, `breed` takes the kind the party holds least (then the wall's role), `best` the better;
  generation +1 (`Egg.sire`, the pup `Rook II`), pedigree +2 % a generation on hp and attack (cap +10 %). Surplus released
  past the party's slots + 3 (never a lamed pet; a second of a kind first; a news line). A lamed party pet gives way to a
  fit kennel pet; the kennel-hand fields a kind the party lacks first.
- **5, pet + build synergies** (`pets::PET_SYNERGIES`, role + a worn package, no level gate, named once as a report beat
  and on the build: `BuildWire.pet_synergy`): **Packmaster** (pack break + a guard), **Shieldmate** (corridor fighting + a
  guard) — the guard draws +10 %; **Falconer** (kite archers + a scout) — the scout sees 3 further. Dropped by the research
  §5 rule (< 25 % of PICKED seeds): Quartermaster (light hands + a fetcher, 0/16), Field medic (guarded + a mender, 1/8).
- **A, heir shapes.** `traits.json` ships 10 commons (each gift with a cost, three gifts at two `when`s so the wake's three
  cards differ: guard@boss·frail, guard@crowded·slow, mend@hurt·thin, mend@deep·frail, quick@first·frail, quick@hurt·thin,
  fury@boss·frail, fury@crowded·thin, rested@quiet·slow, sure@deep·dim) and the two rare twists, hand-shipped (the
  `examples/traits.rs` measurement not re-run). `answer the killer` now has its card: a boss killer → guard@boss, gas or
  poison → mend@hurt, a fast foe → quick. Bots that run the neutral heir (`metrics`) are unchanged; the dayplayer's heirs
  wear them from heir 3.
- **Wire** (added, nothing renamed): `Companion.life` (`PetLife`: role, xp, lame, falls, heirs, grudge, runs, bred,
  announced, fetched), `Egg.sire`, `Fallen.lamed`, `Lineage.pets` (`PetsWire`: tame, tame_revoked, kennel, synergies,
  fetched, old_hounds, bred, released), `BuildWire.pet_synergy · pet_effect`, `StandingOrders.kennel`, the gold term
  `fetched`, `ReturnReport.feats` kinds `pet` (one line a return: `Rook · guard L4`), `fetched`, `old_hound`, `bred`,
  `released`, `pet_synergy`. Input: `revokeTame(revoked)`; the keeper's order through `setOrders`. `types.ts` carries each.

## After Cut 119

*2026-10-10, the tree above (uncommitted). `cargo run -q --profile fast -p riddle-core --example pets_eval -- --seeds 16`
(17 min on a loaded box; the probe gained the rows below: role slots, the best role per wall, a role given, a fetcher sent
from D1, fetched vs lost carry, synergies, bred vs wild, the pet line, old hounds; recovery now counts a lamed pet back on
its feet, `petstats::LameHealed`). Bars as amended by the research.*

| metric | IDLE | PICKED | bar | result |
|---|---|---|---|---|
| a pet in the party by day 3 (item 0) | 16/16 | 16/16 | ≥ 12/16 | **PASS** |
| pet value, worn − removed, Δscore ± SD at D8 · D13 · D18 · D23 · D28 | +3.2±3.4 · +2.5±2.9 · +19.4±20.5 · +3.2±10.1 · +9.6±9.9 | +3.4±7.6 · +8.1±9.8 · −2.8±6.8 · −2.5±15.5 · +1.5±8.9 | ≥ +5 at ≥ 3 walls | **FAIL** (2 walls · 1 wall) |
| … never above | 19.4 | 8.1 | ≤ +25 (the archer alone was +36 at D13; now +17.2) | **PASS** |
| best role per wall (role = mean of its kinds) | mender D8 (0.35 SD) · scout D13 (0.03) · fetcher D18 (0.72) · D23 (0.23) · D28 (0.53) | (pooled) | each role best at ≥ 1 wall, by ≥ 1 SD | **FAIL** (guard never best; no margin ≥ 1 SD — no role dominates) |
| pick rate, party slots: roles · top kind | fetcher 27 · guard 28 · mender 18 · scout 27 % · jackal 19 % | fetcher 22 · guard 41 · mender 13 · scout 24 % · goblin 20 % | no role > 50 %, no kind > 35 % | **PASS** (was archer 28 % · scout 54 % mid-tuning) |
| guards' share of blows drawn · damage (kind panels, guard role) | 7.9 % · 4.5 % (2.1 draws a pet-run) | (pooled) | 10–35 % (guards) | **FAIL** |
| traced acts per run per pet, by role | fetcher 343 · guard 52 · scout 35 · mender 19 | (pooled) | ≥ 3 each | **PASS** |
| pet falls per 10 runs · named · recovered by day 14 · median days | 0.04 · 100 % · 96 % · 0.0 | 0.03 · 100 % · 97 % · 0.0 | ≤ 2 · all · ≥ 80 % · ≤ 1 | **PASS** (was TUNED 11.1; the limp home) |
| level curve, longest-serving pet, days 3 · 7 · 14 | L2 · L4 · L5 | L4 · L7 · L7 | L2 · L4 · L6 | **PASS** at 3 and 7; IDLE day 14 one short (**FAIL**) |
| carry fetched on deaths: of the carry lost with a pet standing · of all carry lost | 20.0 % · 2.4 % | 20.0 % · 3.5 % | 15–35 % | **PASS** with a pet standing (most deaths come after the pet limped home) |
| a death with a pet pays under a return | 28 % max + 30 % heir floor < 60 % | same | always | **PASS** (by construction; test) |
| D1–D4 pickup walk, a fetcher vs none (ticks) | 634 vs 838: −24.3 % | 694 vs 933: −25.6 % | ≥ 25 % shorter | **FAIL** IDLE (0.7 pt short) · **PASS** PICKED |
| tenure: median heirs served · old hound by day 14 | 39 · 16/16 | 90 · 16/16 | old hound ≥ 50 % of seeds | **PASS** |
| a report line naming a pet per return (check-ins) | 100 % | 100 % | ≥ 1 a return | **PASS** |
| synergy formation, PICKED seeds | — (IDLE wears no tactic) | Shieldmate 14 · Falconer 9 · Packmaster 4 /16; Quartermaster 0, Field medic 1/8 dropped | drop < 25 % | **PASS** (two dropped) |
| bred vs wild in use (party slot·check-ins) | 2 % bred (266 eggs) | 8 % bred (530 eggs) | reported | — |
| IDLE's gain from pets | +3.2 … +19.4 by wall (mean +7.6) | — | reported | — |
| pets cause no dice death · dice ≤ 5 % (`metrics --quick`) | 3.5 % (n = 301, death-weighted) | | ≤ 5 % | **PASS** |
| taps per return | the core adds no prompt (the keeper's order and the tame row default on; nothing waits on a tap) | | no increase | not re-run (the taps test is the client's `web/tests/cut118.mjs`) |
| IDLE, PICKED, stance bars | see Gates | | hold | **PASS** |

**What the tuning learned (8-seed probes between).** The first cut (fetcher on gold only, mender every 30 ticks, pets
killable from full) made the mender best at every wall (+16 at D8), the archer +39.7 at D18, and pets fell 3–9 times a
10 runs (one-shot from full hp deep). Then: a pet's blow from above its line leaves it at 1 hp and a wounded pet limps
home (falls 0.03–0.04 a 10 runs); the mender heals only under 60 % every 90 ticks; the shot cap `1 + level/3`; the fetcher
brings the items the hero would take, not only gold (pickup −18 % → −25 %); the open tame takes a role the party lacks,
the keeper breeds the kind the party holds least and the kennel-hand fields a missing kind (archer slots 41 % → 12 %).

**Gates (final tree; `node tools/gates.mjs --full --rows … --fail-fast`, 16 seeds; ranges are the fail-fast's settled
bounds).**

| row | value | result |
|---|---|---|
| IDLE D8 day 1 · D13 by day 4 · D23 by day 12 | 16/16 · median day 1.0…2.0 · 12–16/16 (median day 4.2…5.8) | PASS |
| IDLE stall · gold every day · King in a fortnight | 16/16 · 16/16 · 16/16 | PASS |
| stance L3 / L5 | median day 1.0 · 4.0…7.0 | PASS |
| PICKED ≥ 1.5× IDLE at D13 · D18 · D23 | 1.75…3.00 · 2.29…4.75 · 2.77…5.58 | PASS |
| IDLE never out-paces PICKED | 91–97 % (102/105 pairs) | PASS |
| TUNED beats PICKED at D33 (≥ 1.15) | 1.15…1.21 | PASS (tight, as Cut 118's 1.17–1.18) |
| RANDOM slower than PICKED at D13 · D23 | 16/16 · 16/16 (median +4 · +52 h) | PASS |
| workers-daily (mean best ≥ by hand − 1) | 30.62 vs 30.87 | PASS |
| nothing-required (no fail-fast, every seed) | no seed > 48 h behind from day 5 | **PASS** (Cut 118's seed-14 failure no longer reproduces) |
| `metrics --cut30` (8 rows) | stance walls bold 2 · guarded 4 · hunter 2; Steady safest (6 % deaths) | all PASS |
| dice (`metrics --quick`) | 3.5 % (n = 301, death-weighted ±2.5) · routes 1.0 % | PASS |

**Cut 118's five cohort-set rows (`metrics --quick`), measured in temporary worktrees at c495638 (before Cut 118) and
4617799 (Cut 118), and on this tree:**

| row | c495638 | 4617799 | Cut 119 tree | verdict |
|---|---|---|---|---|
| lanes D5, EDITED both viable | 92 % · 38 % FAIL | 92 % · 38 % FAIL | 93 % · 37 % FAIL | before Cut 118 |
| thief guard ≤ 20 % | 25 % FAIL | 25 % FAIL | 25 % FAIL | before Cut 118 |
| card ↔ chore loops ≤ 1 % | worst 1.8 % FAIL | worst 1.8 % FAIL | worst 0.0 % **PASS** | before Cut 118; passes now |
| return row (40 sets) | 36/40, worst +6.6 FAIL | 36/40, worst +6.6 FAIL | 36/40, worst +6.2 FAIL | before Cut 118 |
| waystone $/h ≥ D1 $/h | 42/48 FAIL | 37/48 FAIL | 36/49 FAIL | failing before; **Cut 118's graves added 5 sets** |

The waystone row: with only `feats::recover_graves` switched off, 4617799 gives 42/48 exactly as c495638 — the graves
bring a dead heir's lost carry home on the D1 sends that walk past the shallow graves, so D1 $/h rises over the waystone's
(the tally's `other` on D1 went from +0 to +5…+270 a send). It is recovered carry, not income; the row compares income.
**Deviation, recorded, not fixed:** no threshold was lowered and the row's measurement was not changed (excluding
`recovered` from its $/h would be the honest read, but it is a gate's arithmetic and the owner's call).

**307dbed hash.** 2f3eb706b3d24c7c → **4c5ee355b67975f1** (`fixtures/sends_307dbed.txt`, the reason in
`tests::saves_from_307dbed_send_identically`): the tame row tames the first stray (now on every lineage) and a pet walks
every send. With the companions pinned off (`pets::pin_off`) it is 2f3eb706b3d24c7c exactly
(`tests_cut119::the_307dbed_hash_restores_with_pets_pinned_off`); with Cut 118 pinned off too it is aa808dca72425139
(`tests_cut118::the_307dbed_hash_holds_with_the_systems_pinned_off`). The shipped heir shapes move nothing in these sends.

**Tests.** `cargo test --workspace --profile fast`: 769 pass, 0 fail, 6 ignored (9 new in `tests_cut119.rs`, 1 in
`tests_cut118.rs`). Clippy `--all-targets -D warnings` clean; wasm rebuilt (`tools/wasm.sh`), the native bridge regenerated
(`node tools/native-codegen.mjs`); `tsc` clean; copy-lint 0. Older tests of the pre-Cut 119 companion path (a fall is an
egg, the bank levels, the 80 % stray, the wounded fall-back) run with Cut 119 pinned off (`tests::pets_off`, the arena);
three fixtures read the patch arithmetic / the fork tablet on the neutral heir or with pets off, as Cut 116 pinned its
affixes. Two fixes on the way: `trace::record` restores this run's avenged / tamed grudges in the replay's lineage
(the death's replay generated a floor without the grudge the run had met — exposed by the shapes moving seed 3's deaths);
the drive-off test reads the record checkpoint's secured carry.

**Deviations.**
- *Pet value* (≥ +5 at ≥ 3 walls): IDLE 2 walls, PICKED 1; the cap (≤ 25) holds. PICKED's party of four with the panels'
  removal reads ±7–15 SD; the paired gain is real early (D13 +8.1) and noise later. Next lever: a guard that matters at
  the boss walls.
- *Role spread*: no role is best by ≥ 1 SD anywhere (none dominates, the research's intent), but the guard is best at no
  wall and draws 7.9 % of blows (bar 10–35 %) even at 65–90 %: a guard stands beside the striker too rarely. Next:
  a guard that steps between (positioning), not a larger chance.
- *Levels*: IDLE's longest-serving pet is L5 on day 14 (bar L6); PICKED reaches L7 by day 7. One curve for both bots
  sits between; the L3 signature arrives by day 3–7.
- *Pickup*: IDLE −24.3 % (0.7 pt under the 25 % bar), PICKED −25.6 %.
- *Fetched*: 20 % of the carry lost with a pet standing (in the band), but 2–4 % of all lost carry: the limp home that
  holds pet deaths at 0.03–0.04 a 10 runs takes most pets out of the run before the heir dies.
- *Keeper as a ladder rung*: the order is a standing switch defaulting to `breed`; it is not wired as a new system unit
  in `systems.rs` (no reveal), so IDLE breeds without a tap.
- *Heir shapes* hand-shipped without `examples/traits.rs`'s strict build test (the measurement was not re-run); IDLE's
  bars hold with them.
- *Waystone row*: above.
