# Cut 122: a smooth curve and advice that never lies

*2026-10-11. Cohort 9621b19 (Cut 121): A 62.7, B 67.9 (incomplete), α .705, mean 65.3. The longer road held:
neither rater slew the King in the first session. The score fell on trust and friction: a Legacy cliff
(rank 1 costs 15, rank 2 600, with 388 ◆ unusable), suggested fixes that make the game's own forecast worse,
contradictions, hunger with no answer, and an edit refused silently. Goal: 80.*

## Core

1. **A smooth Legacy curve.** Each rank costs about 2–3× the last, not 40×: e.g. 15 · 40 · 110 · 300 · 800 ·
   2 000 …, with more ranks if needed so the late cost holds. Keep PICKED+L's King row (median ≥ day 7,
   min ≥ 4) and every IDLE row. The next rank is always within reach of a day's Legacy.
2. **Advice that never lies.** A suggested fix, a TRY chip, a death screen's pick and a return's suggested
   change are shown only when the paired forecast prices them no worse at the wall they answer (past/death
   within the noise or better). Otherwise they are dropped, or shown as `trade-off` with the numbers. Gate: a
   test over 16 seeds of death screens: no offered fix's forecast is worse than the current one past the noise.
3. **One truth about the King.** `King · slain` only after he has fallen in this descent. A death to him
   reads `died to the King`, never next to `slain`. The reel's `The Mirror King waits` agrees.
4. **Credit before any rule.** `own rule` only for a row the player wrote or picked. The default set is
   `default`, never `own rule`.
5. **Hunger with an answer.** `starving −N` (max hp lost to hunger on deep floors) names its counter and the
   counter exists: rations under the apprentice's order (Cut 118) and a buyable ration, with a forecast of the
   hp kept. Gate: a ration-carrying PICKED+L loses ≤ 1/3 of the max hp hunger takes without it.
6. **No silent refusal.** An edit that adds a card the player doesn't own (`card not owned: cadence`) is
   refused before it shows as added: the guide offers only owned cards, or offers to unlock it. Wire the refusal
   reason to the client.
7. **Dead time.** `Heading home` walks longer than ~20 s of watch time compress (a walk-home summary line),
   and quiet stretches at 48× collapse into a `quiet · N floors` line instead of a fast scroll.

## Client

8. Labels: `THIS EDIT` becomes `this change` (it also follows forge and other purchases), or is attributed to
   the actual change (`forge +1`).
9. Legacy shows progress toward the next rank (`◆388 / 600 → health III`), and the offer to auto-spend.
10. The dock never rearranges under a pending tap: new tablets append at the end, or a lit marker, never a
    reflow of existing positions.
11. Tactic compare: say what differs (`better at D23 · worse at D28`), or `no wall differs` with the number of
    sends, instead of a bare `close`.

Gates: focused tests per item; the routine full gate; the full client suite; a fresh blind pair.
