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
