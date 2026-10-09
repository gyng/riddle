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
