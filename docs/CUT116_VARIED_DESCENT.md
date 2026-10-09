# Cut 116 — every heir's descent is its own

*2026-10-09. Nine blind cohorts; the last eight plateau at 64–69 (latest 67.3, α .86). The one structural
gripe across cohorts: "every run climbs the same boss ladder in the same order (D8, D13, D18, D23, D28,
D33)", "D1–D23 replays with the same beats", "the Warlord, Mother and Lich repeat on every run" — surprise,
tension and pacing sit at 0.6. Owner (2026-10-09): vary the descent, plus the long-tail fixes, then one
more cohort.*

## Core — the descent varies by heir

1. **Boss affixes per heir.** Each heir draws, deterministically from the lineage seed and the heir, one
   affix per band boss from a small pool (the existing encounter modifiers where they fit: swift, armoured,
   brood, vampiric, enraged…), shown on the floor chart and the boss bar before the fight (`Warlord ·
   armoured`) so a counter can be chosen. An affix changes which tactic or variant answers the wall.
   Gate: no affix makes a wall unanswerable (each has a counter among arrived tactics/variants/items);
   IDLE bars hold (IDLE meets affixes too — tune so D8 day 1, D23 by day 12, King never in a fortnight).
2. **More roads.** Open the D9 and D14 forks (`descent::OPEN_FORKS`) beside D5, so heirs can take
   different biome orders; the guide/route chip chooses, the forecast prices both lanes. Gate: Cut 26's
   no-set-dominates-both-lanes bar on each new fork; IDLE/PICKED bars hold.
3. **A rotating guest.** Once per band, a heir may meet a wandering champion (a named elite from another
   band's roster) on a non-boss floor — a story beat with a grudge if it kills, never a wall. Gate: guest
   deaths ≤ a small share; it appears in the chronicle and reel.

## Fixes (client + core)

- `▶ watch kill` for a boss slain in an earlier run of an absence (B: "not held") — keep the capsule of
  every absence run that killed a band boss (bounded), not only the run in flight.
- Fix lines still read "survives 9/12" beside "death 29→90%": make the per-run horizon unmistakable and
  explain `risky` / `measuring` / `WRITE` states.
- Legacy piles up unspendable (146): a sink or the deeper tree reachable.
- Normal dead stretches (~90–170 s): find the remaining case.
- Hand control inert in gas / 40 identical taps vs the Warlord: actions must visibly resolve; repeated
  attack taps should be one 'attack until' order.
- First death stamped LUCK over a swarm the rules could have handled: the luck verdict needs a margin.

Gates: focused tests per item; the routine full gate (no lowered threshold); the full client suite; a
fresh blind pair.

## Core 1–3: built and measured (2026-10-09, uncommitted)

**What shipped (core).** `descent::Affix` (armoured, swift, regenerating, brood, enraged, vampiric), each boss its
own pool of four (`affix_pool`; the Mirror King draws none: the bottom stays the bottom), drawn purely from
(lineage seed, heir, boss) — `boss_affix`, `LineageState::affixes`, carried on `Run.affixes`; the boss spawns
with it (`endgame::apply_affix`: the tier mechanics at any tier — +1 armour, +2 speed, regeneration, the
frenzied and leeching elites — a brood guard at 75 % per tile, and a 5 % hp price, `AFFIX_HP`). Wire:
`Entity.modifiers.affix`, `ForecastDepth.affix`, `Lineage.affixes` (boss, depth, affix, effect, counter).
`OPEN_FORKS = [5, 9, 14]`. `descent::guest_for`: half the heirs × bands meet a wandering champion (a named foe from
another band's roster, +20 % hp) strictly inside the band; slain → `Champion` episode (reel), the run's news
(`slew Grak`), the heir's deed (`slew Grak, the wandering skeleton` — the chronicle); kills → a grudge under his
own name. Probes' pins: `LineageState.affix_pin` / `guest_pin` (none in play).
Client (minimal): `types.ts` fields, `forecast.ts` floor chart `· armoured` (tooltip: effect · answer), `watch.ts`
boss bar `Warlord · armoured`.

**Gates.** Routine full gate on the combined tree (with the fixes agent's edits): all PASS — IDLE D8 day 1 2/2,
D13 day 2.0, D23 day 6.0 (was 4.2), King 0/2; PICKED ≥ 1.5× IDLE 3.00 · 3.50 · 7.58; TUNED vs PICKED 2.42;
stance walls bold 2 guarded 3 hunter 1. Deviation of tuning: a first pass with 10–15 % hp prices failed the
stance row (guarded won no wall); the price was cut to 5 %. Workspace tests 740 + 7 new (`tests_cut116.rs`) pass.

**Probe** (`examples/varied_descent.rs`, 4 IDLE lineages, 48 paired sends):
- Affix answerability: PASS for all 20 boss × affix pairs (the best arrived answer passes the affixed wall at least
  as often as the worn set passes the plain one; e.g. Warlord worn 14 % plain, best answer 85–89 % affixed).
  D28 Queen: no signal (0 % for every answer, plain or affixed, on these lineages). **Gap, recorded:** the affix
  rarely changes *which* answer is best (1 of 20: brood Mother → bold over guarded); best-affixed is within
  −9…+1 pts of best-plain. Making affixes bite harder broke the stance row; left for the next cut.
- Forks: both lanes viable at D9 and D14 and the per-lane best sets differ (no set is best on both), but **Cut 26's
  numeric bar fails**: cross-lane loss +8 · +11 pts at D9, +5 · +5 at D14 (bar ≥ 15 each). Opened anyway per the
  owner's choice; recorded as a deviation.
- Guests: 0 of 225 IDLE deaths by a champion over 4 fortnights (≤ 5 % PASS); they appear in chronicle lines
  (136 lines name one) and the reel.

**307dbed hash** ddd4c46c7dc03d58 → 9cfbd23b2027a263 (affixes and guests; pinned off, the combined tree hashes to
ddd4c46c7dc03d58 exactly — the forks and the fixes agent's edits move nothing there).
