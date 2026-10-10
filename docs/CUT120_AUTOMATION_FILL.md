# Cut 120: automation fill and chore reduction

*2026-10-10. Owner: "yes fill out automation and chore reduction". A routine return is 2 taps (Cut 118); what is
still done by hand is listed below. The principle is unchanged (Cut 114): a worker carries out an order the
player gave and reports what it did, and nothing is silent.*

## Core (riddle-core)

1. **Legacy under an order.** A standing order `legacy`: `balanced` (health → damage → armour in turn, then the
   effects in the offer's order) · `health` · `damage` · `armour` · `off` (default `off` for saves; `balanced` for
   new lineages, announced once). It spends whenever an offer is affordable, at the bank step. A report line names
   each buy.
2. **Ranks by order.** The worker ranks II/III: an order `ranks: auto · off` buys each rank when it is due and
   affordable, keeping the reserve (RESERVE_UNITS). A rank-II perk choice (Cut 114 §5) is a per-worker chip,
   defaulting to the first perk.
3. **Same for all bloodlines.** Each order set on one bloodline offers `same for all`, which copies it to every
   bloodline. Orders gain a `shared` flag, so later changes follow. Hires stay per town.
4. **After the clear: the herald carries Ascension.** Once the King falls, an `ascend` order (`off` default ·
   `on`) lets the scout's sends carry on in the next ascension's dungeon while away. It never starts without the
   order. IDLE never orders it, and the King-never-in-a-fortnight bar holds.
5. **Kennel keeper as a worker.** A real node in tree.rs (`kennel_keeper`, chore `breed`, need 2, fallback 48 h)
   that carries the Cut 119 breeding order. It gets a town post, a ladder entry under workers, and an announced
   first act.
6. **The worker ledger.** `ReturnReport.workers[]`: one line per worker that acted, with its count and cost
   (`apprentice · sword +3 · mail +2 · −$X`, `clerk · banked $Y`, `guide · start D19`, `keeper · sorted 12`).
   Gate: every worker act in the absence appears (count = the `tree.acts` delta), and the gold reconciles with
   the ledger.
7. **Next worker visible.** Each worker node on the wire gets `progress` (`2/3`) and `fallback_h` (already there),
   plus `eta_h`, so the town's posts can show `forge 2/3 · or 30h`.

## Client

8. **An orders sheet.** One place lists every standing order: the scout, apprentice, kennel, heir, legacy, ranks
   and ascend orders, each a chip, with `same for all`. It is reachable from the ladder's `orders` rung and the
   run setup.
9. **The worker ledger** sits in the While away `Workers` fold, one line per worker. A worker's first act stays
   a highlight.
10. **Town posts** show progress and fallback (`forge 2/3 · or 30h`) on the next worker's post, with ≤ 3 markers
    (cut30town budgets).
11. **Chore audit.** Count the taps for each routine (a check-in with 3 bloodlines, buying a rank, spending Legacy)
    before and after, in a test.

## Gates

- IDLE unchanged (its orders default as before; `legacy: off` for IDLE's harness, or measured if `balanced`
  is the new-lineage default).
- PICKED, TUNED, workers-daily, automation-pays (report it either way), nothing-required.
- No added required tap.
- The full client suite.

## Core: built and measured

*2026-10-10, core items 1–7 (crates/riddle-core, crates/riddle-wasm; `web/src/engine/types.ts` fields added, none renamed).*

### What landed

1. **Legacy order** (`StandingSwitches.legacy`, `tree::LEGACY_ORDERS` `balanced · health · damage · armour · off`;
   `tree::legacy_pick`, `legacy_act`). `balanced` buys health → damage → armour in turn (the lowest rank, in that
   order) and waits for the next in turn; past the base three, the effects in the offer's order (a fork's first branch;
   one shut by depth, a parent or the other branch is passed). A focus takes its upgrade to the cap, then balanced. It
   spends at each send (the camp step where the clerk banks; the run takes it), through `legacy::buy_next`.
   Defaults: a save `off` (serde); a harness's bare `Game` `off`; a **player's new lineage `balanced`**
   (`Session::new`, a new bloodline; `tree::new_lineage_orders`). Announced once (`LEGACY BALANCED` beat, news `order`).
2. **Ranks by order** (`StandingSwitches.ranks`, `auto · off`; `tree::ranks_act`): each hired worker at work whose rank
   has come is bought from the purse while it keeps the reserve (`tree::reserve`) and the lit hire's price; `hire <id>
   rank` in the ledger's `hires`. Same defaults as Legacy (new lineage `auto`). **Rank-II perk chip** (`Tree.perks`,
   `tree::PERKS`): apprentice `cheaper steps` (the old −5 % a rank, default) · `keeps a reserve` (a forge unit more
   kept a rank, no discount); scout `shorter rest` (the old −5 % a rank, default) · `safer start` (+5 % max hp at the
   send a rank). Input `setPerk(worker, perk)`; hires and perks are the town's (copied across bloodlines).
3. **Same for all** (`StandingSwitches.shared`; wire `StandingOrders.shared`): `setOrders` with `shared: [keys]`
   (`tree::SHAREABLE`: insure · forge · wall · sink · heir · kennel · legacy · ranks · ascend) copies each named order
   to every bloodline (`Session::set_orders`), and every later change follows; a new bloodline takes them. An unknown
   key is refused.
4. **Ascend order** (`StandingSwitches.ascend`, `off` default · `on`): the King fallen, the scout and the herald at
   work, the herald begins the next unlocked descent at a send (`Game::carry_on`: `begin_descent`'s lineage reset,
   the absence's batch, deaths and reel kept). Never without the order; no bot orders it.
5. **Kennel keeper** (`kennel_keeper`, chore `breed`, need 2, price 3 units, fallback 48 h, gate `kennel`, post
   `kennel`, beat `AUTO BREED`; last in the tree's order). The Cut 119 breeding order is his: unhired, no egg is bred
   (the surplus release stays as before); a breed by hand (`Game::breed`) counts toward him. His acts: eggs and
   releases (`sorted N` · `bred 1` · `freed 2`). The dayplayer hires him under `pets` like the kennel-hand.
6. **Worker ledger** (`ReturnReport.workers[]`): every node's line `n` = its `tree.acts` delta; order lines `ranks`
   (`+2 ranks`, items `scout II`, `spent`) and `legacy` (`+3 upgrades · ◆12`, items `health 2/3`); the herald's
   `ascended` (`descent 1`); the apprentice's sinks (`sinks $X`). New `WorkerAct.gold`: the purse the line moved,
   signed. The ration's ledger term is now `sinks` (was `supplies`), so the apprentice reconciles.
7. **Next worker** (`WorkNode.progress` `2/3`, `WorkNode.eta_h`, `Works.next_worker`; the lit `WorkerPost` carries
   `progress`, `fallback_h`, `eta_h`).

### Tests (`tests_cut120.rs`, 9; plus `offline::slice_tests::the_cut120_orders_are_slice_stable`)

`the_legacy_order_spends_in_turn` · `the_legacy_order_takes_the_effects_after_the_base` ·
`the_ranks_order_buys_due_ranks_keeping_the_reserve` · `the_rank_two_perk_is_a_chip` · `same_for_all_copies_and_follows`
· `the_herald_ascends_only_under_the_order` · `the_kennel_keeper_is_a_worker` ·
**`the_worker_ledger_matches_the_acts_and_reconciles`** (the gate: 3 seeds × 8 h with nine workers, ranks and Legacy
on — every node's line count equals its acts delta, the order lines theirs, and Σ apprentice + clerk + ranks
`gold` = the ledger's apprentice + forge + sinks + hires + bank terms) · `the_next_worker_shows_progress_and_eta`.
Tests changed for the new defaults: the Legacy fixtures that buy by hand set `legacy: off` (legacy_tests `rich`,
tests_cut305 ×2, the three-bloodline slice test); `the_keeper_breeds_and_releases` hires the keeper; the 8 h slice
partition counts the order's spent points as earned.

`cargo test --workspace --profile fast`: 780 passed, 0 failed. Clippy `--all-targets -D warnings` clean.
The 307dbed send hash did not move (`sends_307dbed.txt`, the pinned-off restores unchanged).

### Gates (targeted rows, 16 seeds, `--full --rows`; the HEAD f9107e3 baseline run beside it with the same binary args)

| row | Cut 120 | baseline |
|---|---|---|
| IDLE D8 day 1 · D13 median day · D23 by day 12 | 16/16 · 1.7 · 16/16 (4.7) | same |
| IDLE stall · gold · King | 3 d · 14/14 · 0/16 | same |
| stance L3 · L5 | 1 · 6 | same |
| PICKED ≥ 1.5× IDLE (D13 · D18 · D23) | 2.50 · 2.75 · 3.45 | same |
| outpace · TUNED vs PICKED D33 · RANDOM | 97 % · 1.17 · 16/16 · 16/16 (+4 · +52 h) | same |
| nothing-required · nodes-bought | ok · 0 unbought (11 hired by day 14) | ok · 0 (10 hired) |
| workers-daily | 30.58 vs 30.87 PASS | 30.62 vs 30.87 |
| automation-pays | 48 vs 48 h · 28.16 vs 28.52 **FAIL** | 48 vs 48 h · 28.23 vs 28.52 FAIL |
| idle-delta | +8 h PASS | +8 h |

`node tools/gates.mjs --fast`: all PASS — metrics `--cut30` (8 rows), qa 10 seeds (gold header == ledger sum, report
terms sum to the purse's move, night gold, gold conserved: every invariant PASS), dayplayer IDLE.

### Deviations

- **Legacy and ranks defaults are a player's new lineage's, not the harness's.** Measured with `balanced` + `auto` as
  every bare `Game`'s default, IDLE slew the Mirror King on seed 1 on day 12 (`idle-king` FAIL). The contract allows
  `legacy: off` for IDLE's harness: the dayplayer's lineages (`Game::new_resident`) keep `off`, `Session::new` (the
  client's new lineage) and a new bloodline take `balanced` · `auto`. `ranks: auto` alone (acting hourly, an earlier
  build) kept every IDLE bar (D13 day ≤ 2.3, D23 16/16, King 0/16, idle-delta +8 h). Finding for the owner: a
  no-touch *player* under `balanced` is stronger than the IDLE the bars measure.
- **Legacy and ranks act at sends, not on the hour.** A sliced absence settles a run's end and the last hour's worker
  pass in a different order from a whole one (pre-existing; the clerk is idempotent enough to hide it); a Legacy buy
  on the hour moved between the two. At the send the run takes the purchase either way.
- **Ascend** begins the next unlocked numbered descent (`begin_descent`'s reset, not `ascend(variant)`'s), and needs the
  herald hired (he carries it) as well as the scout.
- **The kennel keeper's hire gates the breeding**: IDLE (no hires past the scout) no longer breeds eggs; every IDLE bar
  is unchanged. automation-pays fails as before (Cut 110 §4's recorded deviation; −0.07 floor on AWAY's mean best,
  the keeper's price), threshold unchanged.
- Perks for the apprentice and the scout only (Cut 114 §5's two); Cut 114 §5's "neither perk dominates" bar is not
  measured (no bot picks the second perk).
- The 307dbed hash did not move, so nothing was re-recorded.

## Defaults: option C measured, option B taken (owner 2026-10-10)

*The owner's option C: a real new player's defaults (Legacy `balanced`, ranks `auto`) become the harness IDLE's too,
re-tuned until every bar holds. It did not converge in the time given, so the coordinator's fallback, **option B**,
shipped. A new lineage (`Session::new`, a new bloodline) defaults both orders to `off`, the same as a save and the
harness (`Game::new_resident`). The client offers each order once when its rung lights. Harness and real defaults
are identical either way, so the bars measure what a new player gets.*

### Option C measured (`Game::new_resident` took `balanced` · `auto`; 16 seeds, `--full --rows`)

| row | HEAD (off) | C, HEAD content | C + King drill 14 | C + Legacy 15/100/300 · 400/800 + King drill 10 | C + Legacy 50/300/800 · 600/1200 |
|---|---|---|---|---|---|
| IDLE D8 · D13 day · D23 (day) | 16/16 · 1.7 · 16/16 (4.7) | 16/16 · 0.7 · 16/16 (2.0) | same as C | 16/16 · 1.0 · 16/16 (2.3) | 16/16 · 1.5 · 16/16 (2.7) |
| IDLE stall · gold | 3 d · 14 | 0 · 14 | 0 · 14 | 2 · 14 | 1 · 14 |
| IDLE King (slain seeds) | 0/16 | **16/16** (days 12–14) | 0/16 | 0/16 | **1/16** |
| stance L3 · L5 | 1 · 6 | 1 · **8** | 1 · **8** | 1 · 7 | 1 · 5 |
| PICKED ≥ 1.5× IDLE | 2.50 · 2.75 · 3.45 | 1.50 · 2.25 · 2.25 | same | 1.50 · 2.25 · 2.33 | 2.25 · 2.58 · 2.33 |
| outpace | 97 % | 99 % | 99 % | 100 % | 98 % |
| TUNED vs PICKED D33 | 1.17 | **1.00** | **1.00** | **1.00** | 1.18 |
| RANDOM (D13 · D23 seeds; medians) | 16/16 · 16/16 (+4 · +52 h) | 15/16 · **13/16** (+0 · +16) | same | 16/16 · 14/16 (+4 · +20) | 16/16 · 16/16 (**+0** · +32) |
| nothing-required | ok | ok | ok | ok | ok |
| automation-pays (mean best) | 28.16 vs 28.52 FAIL | 31.13 vs 31.15 FAIL | same | 30.81 vs 30.77 PASS | 30.52 vs 30.47 PASS |
| workers-daily | 30.58 vs 30.87 | 32.12 vs 31.88 | same | 31.77 vs 31.89 | 31.46 vs 31.58 |
| idle-delta | +8 h | +0 h | +0 h | −4 h | +8 h |

What drives it:
- **Legacy is cheap against its income.** IDLE earns about 110 ◆ on day 1 and about 2,300 by day 14. Under
  `balanced` the whole base (54 ◆) is bought by hour 16 and the whole tree (216 ◆) by hour 56. Rank 1 of each root
  alone (+3 HP, +1 damage, +1 armour) moves IDLE's D23 from day 4.7 to 2.2 and fails stance L5 (8). The bots never
  spent Legacy before, so the bars had never measured it.
- **The King falls to the idle path, not to raw stats.** IDLE meets him from about day 3 instead of day 11, and the
  deep drill (6 days met) plus scars (30 %) bring him down on days 12–14. With his drill withheld (diagnostic),
  IDLE never kills him in 18 days: about 80 tries, his least hp left is 18 %. Raising him to 360 hp and 16–23 atk with
  the drill kept still lost him on 13/16 seeds. Even Legacy priced at about ×40 (20/150/400) lost him on days 14–17.
- **Legacy compresses the walls.** With Legacy, PICKED crosses the Queen in one check-in (D28→D29 in about 8 h,
  against 16–24 h without it), even at 180 hp. TUNED's pen counter then gains nothing at D33 (1.00).
  Only rank-1 prices ≥ 50 ◆ restore 1.18.
- The nearest candidate (Legacy 50/300/800 · 600/1200) still failed the King on 1/16 seeds, and RANDOM's D13 median
  was +0. It would also need a King-drill change, with margin, and a rewrite of the Cut 32 Legacy price tests. That
  did not fit the hour, so option B shipped.

### Option B (shipped)

- `tree::NEW_LEGACY` = `NEW_RANKS` = `off`; `Session::new` and `add_bloodline` no longer turn the orders on (a new
  bloodline still takes those set `same for all`). `Game::new_resident` is unchanged, so the harness, a new player
  and a save are identical. The option C tuning was reverted; no content changed.
- Every bar is as in the "Core: built and measured" table above (the HEAD column): the harness is unchanged.
  The 307dbed hash did not move.
- `the_legacy_order_spends_in_turn` asserts that the new-lineage defaults are `off` · `off`.

### The Legacy line reconciles per bloodline (coordinator, the client's 3-bloodline audit: `◆102` vs a spend of 54)

`tree.acts` is the town's and sums every bloodline's buys, but Legacy is a bloodline's own. The order now also
writes per-bloodline keys (`tree::legacy_key`: `legacy@<id>`, `legacy◆@<id>`, `legacy:<upgrade>@<id>`). The report's
`legacy` line reads the shown bloodline's keys, so its `◆` equals that bloodline's `spent` delta. The town keys
remain for the ledger gate. Test: `the_legacy_line_is_the_shown_bloodlines_own` (3 bloodlines, 8 h, all `balanced`).
It asserts the line's `◆` equals the shown bloodline's spend, its count equals its ranks, and the bloodlines'
spends sum to the town's `legacy◆`. Without the fix it fails `+27 upgrades · ◆162 vs spent 54`.

### Dayplayer

The per-seed table now prints the King's least hp left and his tries (`king None left 84% ×3`). An info row names
the closest unslain seed (`IDLE King margin`), which is the bar's margin. `--verbose` day lines print Legacy `◆`
and `spent`.
