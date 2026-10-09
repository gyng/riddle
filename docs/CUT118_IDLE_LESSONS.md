# Cut 118: lessons from the idle genre

*2026-10-10. Owner: "implement all recommendations" from research/IDLE_STEAM_2026-10.md. Starts after Cut 117
lands (Cut 117 already holds item 5: one ledger, one horizon, fixed-seed advice). All gates hold without a
lowered threshold; IDLE's games change only where an item says so and the IDLE bars are re-measured.*

## Core (riddle-core truth)

1. **Choose when the boss comes.** A `challenge` order: the player picks which met band boss the next send
   seeks (one of the band's bosses ahead within reach), or `in order` (the default; IDLE). The forecast
   prices each choice. Gate: IDLE unchanged (default); no single order dominates on PICKED's panel.
2. **Restricted walls (trials).** One optional trial per real week, drawn from (week, lineage seed): a boss
   plus a restriction (`no boss focus`, `brood + swift`). Opt in from camp; a clear pays a legacy bonus and
   a trial mark. Never required; IDLE never opts in. Gate: every trial answerable by arrived tactics (probe).
3. **Away finds that scale with time.** Each completed run while away seals one find (cosmetic, a legacy
   shard, a small set piece); opened on return in the report/reel. A second table opens after 6 h away. A
   find log of sets gives small permanent bonuses (≤ 2 % each). Never decides a wall. Gate: IDLE bars hold;
   8 h brings ≥ 1.8× the finds of 4 h on the median seed.
4. **Gold sinks that renew.** After `Kit complete`: per-send supplies (a ration or a one-run hireling, priced
   by depth) under the apprentice's order, works that open a fork early, and gold → legacy at a falling rate
   (a tithe). No sink returns more than it costs. Gate: purse at day 14 on PICKED is no longer monotone-
   rising; workers-daily holds.
5. **Fast early floors.** Floors whose forecast survival is ≥ 99 % resolve as one beat (the watch plays them
   at 8× or skips with a summary line). First D8 boss ≤ 15 min of first launch on the dev walk. Gate: IDLE D8
   day 1 holds; replay hash recorded with a reason if it moves.

## Client

6. **One tap on return.** The report becomes one card: the highlights (finds, bosses, deaths, gold sum), then
   `collect & send`, with standing orders carried; detail folded. At most one decision prompt per return.
   Gate: ≤ 2 taps from opening to the next send on a routine return (cut118 test).
7. **A control ladder.** The automation you've earned shown as one rail: weights → orders → workers → pen,
   lit as unlocked, the next rung's condition shown. Within the clutter budget.
8. **A town crier.** One diegetic line (and a small animation) in town per notable act since the last visit:
   a boss slain, a set completed, a trial cleared. ≤ 3 words in chrome; longer in the crier's scroll.
9. **The ending in sight.** A forecast line of when the King may fall at the current pace (`King · ~day 9`),
   and `uncapped` stays a headline of the away screen.

Gates: focused tests per item; the routine full gate; the full client suite; a fresh blind pair.

## Round 2 amendments (research/IDLE_STEAM_2026-10_ROUND2.md)

- §1 challenge: **boss tokens**, earned from runs (one per band boss slain), banked across heirs. Spending one
  seeks that boss on the next send. No tokens means `in order`.
- §2 trials: the last 4 weeks stay open; every trial can be cleared offline; the legacy bonus is capped (≤ 1 day's legacy).
- §3 finds: one reveal, best first; the sealed count is shown on the away screen; never a click per find.
- §4 sinks: they add and never require upkeep; the tithe shows its rate; one sink per screen.
- §5 early floors: the swift descent is earned (3 clears of a band boss → the floors above it swift) and announced.
- §6 return: the one prompt shows its default; nothing routes through a shop.
- §7 ladder: each rung names the chore it retires and the feat or time that lit it. **Feats light rungs**
  (a trial or boss clear), with the existing time fallback so IDLE is never gated.
- §8 crier: in the hero's voice, from real trace events, short. Grows into **the hero's diary** (a scroll of
  dated lines from the trace, not chrome).
- §9 ending: a % bar to the King, and what comes after the King.
- New: **a roster preview of the next wall** (its boss, affix, guard and their counters) on the camp chart.
- New: **requests from the hero**: one wish at a time (`a lantern`, `a pet`), upside only, never decaying;
  granting one pays a small morale or legacy bonus.

## Owner amendment (2026-10-10): heroes yolo; death is progress

Owner: "yes [heroes yolo], but also think about how to make deaths core to progression/gameplay and less
punishing / less upsetting for the player". Measured basis: the scout's wall order bought no depth (mean best
28.58 vs 28.61, D18 at the same hour) and 26 % more gold in a game whose late gripe is gold with nothing to buy.

1. **No wall order.** The scout's `at a wall` order (`bank · carry · push`) is retired; every send pushes.
   "Bank" means only the exit share again. The forecast's `hold` line goes with it. Record checkpoints still
   secure carry permanently, even on death.
2. **The wall learns you back (siege).** Each heir who dies at a band boss adds to that boss's *siege*: a lineage
   counter per boss (tries, the best hp the boss was left on). Each try gives the next heir a small, capped edge
   against that boss alone (e.g. +4 % damage per try up to +20 %, kept until the boss falls, then cleared and
   remembered in the chronicle). Shown as `Queen · 4 tries · best 22 %` on the death screen, the report and the
   wall preview. Deterministic; IDLE gets it too. Gate: IDLE bars hold (King never in a fortnight); dice deaths ≤ 5 %.
3. **The grave.** A dead heir's carry lies where he fell, as a named grave (`Ada's pack · $1 240`). The next heir to
   reach that floor recovers it (one grave per floor, the newest kept; older ones settle into the chronicle).
   The death's exit share stays 0 % and the 30 % heir purse floor is unchanged; the gold comes home only when a
   later heir walks there. Gate: no gold created (a grave holds only lost carry); the ledger shows `recovered`.
4. **A memorial, not a loss.** The town keeps a graveyard: a stone per fallen heir with the cause as an epitaph
   (`fell to the Queen's mirror, D28`). On the death screen the lead line is the progress (`the Queen · try 4 ·
   +16 %`), then the cause, the trace and the lever, as now. A slain wall boss names the heirs who wore it down.
5. **An echo (optional, if the bars allow).** On the next try at the same wall the fallen heir's ghost fights one
   round beside the new heir (a callout `Ada's echo`). Cut if it moves a bar.

Every death still names its cause, trace and cheapest lever (AGENTS.md).
