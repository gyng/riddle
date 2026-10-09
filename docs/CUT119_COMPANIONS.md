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
