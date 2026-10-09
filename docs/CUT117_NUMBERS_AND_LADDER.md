# Cut 117: numbers that agree, a ladder that moves

*2026-10-10. Ten blind cohorts. 8cf9050 (Cut 116) is the best so far: A 70.2, B 67.6, α .866, mean 68.9.
Varied descent raised surprise and return to 0.8 on A. The remaining gripes are about **trust**
(numbers that disagree, advice that flips) and **sameness** (the same boss order, a purse with nothing
to buy). Goal: 80.*

## 1. Numbers that agree (clarity, mastery)

- King death header `18 hp` vs trace `0/50`: the header reads the deciding moment's hp, as the trace does.
- Report gold head: `$0 GOLD EARNED · purse −$6789 · forge −$6750 · lost $2620` must reconcile in one line
  (earned − spent = Δpurse), each term named. Gate: a core test that the terms sum.
- Forecast vs outcome: `D33 88%` then banks at D30/D32. The forecast names the horizon it prices
  (`reach D33 · 88%` vs `bank by D33`) and the scout's wall order is part of it.
- Preview noise: the 8-sample previews swing. Show a range or hide a delta smaller than its noise.
  Gate: no shown delta flips sign between two consecutive reads of the same camp.

## 2. Advice that holds (autonomy)

- `TRY` never swaps a tactic silently. It names what it replaces and asks with one tap. It is stable
  across reads (same camp → same advice).
- The `Warlord tactic` button opens the sheet *on* that tactic.
- Quest and breed taps show a result (a toast, a changed chip).

## 3. A ladder that moves (surprise, pacing)

- Affixes must change which answer is best (Cut 116 gap: 1 of 20). Each affix gets a counter that is
  a *different* tactic or variant from the plain wall's best. Gate: ≥ 8 of 20 boss × affix pairs change
  the best answer; stance row and IDLE bars hold.
- Band bosses may swap order within a band pair per heir (D13 ↔ D18 bosses, D23 ↔ D28) where the
  biome allows. Gate: IDLE bars hold; dice deaths ≤ 5 %.

## 4. A purse with a use (return, progression)

- After `Kit complete`, gold buys something that matters: a deeper tree node, an heir's
  starting kit, or Ascension's entry. The apprentice never leaves `$0 budget` without saying why.
- 4 h vs 8 h returns differ in what they bring (one more wall tried, a named find).

## 5. Friction

- Check-in taps: target ≤ 4 sheet taps per return (ui QA script counts).
- Hunger's max-hp loss names its answer (a ration, a cook, a tactic).
- D1–D4 pickups at 1×: shorten the walk to a pickup on early floors.

Gates: focused tests per item; routine full gate without a lowered threshold; full client suite;
a fresh blind pair.
