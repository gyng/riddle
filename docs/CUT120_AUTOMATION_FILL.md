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
