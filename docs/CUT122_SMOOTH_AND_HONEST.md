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

## Core: built and measured

*Core agent, 2026-10-11. Gates: `node tools/gates.mjs --full --rows <every IDLE/PICKED/TUNED/RANDOM row,picked-l-king,picked-l-purse>`
(16 seeds, no threshold lowered) all PASS; `node tools/gates.mjs --fast` all PASS (metrics 8 rows, qa 10 seeds, IDLE 2 seeds); fast tests (796 passed, 6 ignored) and
clippy (workspace, all targets, `-D warnings`) clean; `tools/wasm.sh` rebuilt; `node tools/native-codegen.mjs` run (new export
`buyRation`). The 307dbed hash did not move as play: the two new reads (`Ev::Homeward`, a felled boss's `is back`) are stripped
in `sends_hash` as earlier new reads are; nothing re-recorded.*

### §1 A smooth Legacy curve (`legacy.rs`, `defs.rs`)

- `CAP` 5 → 8; `ROOT_PRICES` **15 · 40 · 110 · 300 · 800 · 2 000 · 5 000 · 12 000** (each ×2.4–2.75; was 15 · 600 · 1 500 ·
  3 000 · 6 000); `EFFECT_PRICES` **700 · 1 800** (D8 · D18; were 800 · 2 000), each between two ranks. A rank's effect is
  `legacy::root_effect` (still +3 HP / +1 damage / +1 armour a rank). Test: `tests_cut122::the_legacy_curve_is_smooth`.
- The cheap early ranks gave PICKED+L its old day-7 power by day ~4 (measured on this curve with the Cut 121 King: King median
  5.0, min 4 — FAIL). **The King 500 → 700 hp, 19–25 → 21–27** (650 · 21–27: 7.5 · min 5; 800 · 23–29: 11.0 · min 7, 4/16
  unslain). **Result: PICKED+L King median 7.0 · min 5 (13/16 slain); purse 99 % (15/16).** Every IDLE/PICKED/TUNED/RANDOM row
  unchanged: IDLE D8 16/16 · D13 1.7 · D23 16/16 (4.7) · stall 3 · gold 14 · King 0/16 · stance 1 · 6 · return pick 672/672 ·
  grew 672/672 · stages 11.0; PICKED ≥ 1.5× IDLE 2.50 · 2.75 · 3.45; outpace 97 %; PICKED stages 12.0; reach 16/16; stalls 0 %;
  TUNED vs PICKED D33 1.17; RANDOM 16/16 · 16/16 (+4 · +52 h).

### §2 Advice that never lies (`trace.rs`, `forecast.rs`, `offline.rs`)

- `trace::wall_price(base, with, wall)`: paired sims of the camp without/with a suggestion — sends **past** the wall (reach of
  wall + 1) and the **death** share, each before → after ± 95 %; `trade_off` when either is worse beyond its ± (an exit's lost
  floors count: `hp<70% → return` is a trade-off, no longer a fix).
- Death and stall screens (`camp_deltas`): every measured patch carries `PatchWhole.price` / `PatchWhole.trade_off` at the
  death's floor; a trade-off sinks below the fixes (`patch_harms`) and is never the gem (`gem_eligible`).
- The death's tactic pick (`Death.pick`, the `TRY` tablet): priced with `deathDeltas` wearing it as `take_fix` does
  (`DeathRec.pick_price`); `death(id)` after the deltas carries `Lever.price` / `trade_off`; before, `Lever.pending`.
- The forecast's `try` (`forecast::price_try`, in `forecast()` / `forecastRefine()` only — not `forecast_with`, so probes and
  bots pay nothing): the first try at a boss floor some send reaches, applied as its tap applies it (row at the top with the pen
  open, else its tactic worn) → `ForecastTry.price` / `trade_off`. Deeper tries stay unpriced.
- A return's `SUGGESTED CHANGES` (stall patches, full analysis only; the quick slice keeps its bound): `whole` (whole-run move)
  and `price` / `trade_off` at the stall's floor, trade-offs sunk.
- Gate `tests_cut122::advice_never_lies_at_its_wall`: 16 seeds of death screens (pen open), every patch and pick offered as a
  fix re-measured on the camp's panels at the death's wall: none worse past the noise (16 screens · 17 offered · 20 marked
  trade-off).

### §3 One truth about the King

- `LineageState.descent_kills` (a numbered descent's own kills; `None` in the first dungeon) and `slain_here(kind)`;
  `Lineage.walls[].slain`, `king_eta_h` (0 = slain) and `king_pct` read it — a new descent no longer starts `King · slain`.
- The reel's first sight of a boss the line has felled reads `The Mirror King is back.` (was `waits`).
- **Client:** the watch's death beat `slain · King` (`watch.ts`, the hero slain *by* the King) reads as the King slain — say
  `died to the King` / `fell · King`.

### §4 Credit before any rule

- `packages::credit`: a new credit **`written`** (a row the player wrote: origin `player` · `card` · none) apart from
  **`picked`** (a package he wore: tactic, style, a non-default stance, a chosen temperament); `CREDITS` = written · picked ·
  taught · default · chores. The default tame row (`drill:tame`) is `default`, no lesson. Rater A's first death was
  `stance:guarded` — his pick of Guarded, credited `picked`, which the client printed `own rule`. **Client:** map `written` →
  `own rule`, `picked` → `your pick` (the report's credit bar likewise).

### §5 Hunger with an answer (`situations.rs`, `feats.rs`)

- A ration (the apprentice's under his sink order, or bought by hand: `feats::buy_ration`, wasm `buyRation`, a forge unit, once
  a send) sets `Hero.fed`: the hunger bites one period in `RATION_SLOW` (4), still +10 % max hp. `Run.hunger_periods` /
  `hunger_lost` count it; `SimResult.hunger`.
- `Forecast.hunger` (`HungerForecast`: `lost` · `unfed` · `fed` · `kept` · `packed` · `price`), per send on the forecast's panel.
- Gate `tests_cut122::a_ration_keeps_two_thirds_of_the_hunger`: 16 seeds on a D12 hunger floor, the same send with and
  without a ration: **−65 vs −237 max hp (27 % ≤ 1/3)**.

### §6 No silent refusal

- `packages::row_owned`: `Counter`, `LedgerRow.counter` (`CounterChip`) and `ForecastTry` carry `owned` · `refusal` (the door's
  own words, stable: `card not owned: cadence`) · `wear` (the package that owns the card once worn). `setRules`' error text is
  unchanged. **Client:** offer an unowned counter as `wear Mirror rhythm` (equip) or not at all; show a `setRules` rejection.

### §7 Dead time

- `Ev::Homeward { t, ticks, bank }` (live runs only): the walk home committed, its length foreseen (a bank's path to the
  up-stairs; a return's ≤ `RETURN_TICKS`). **Client:** fold a walk past ~20 s of watch time into one beat and resume at the next
  fight or the exit; quiet stretches are already marked by `StepResult`'s calm spans (`fold::calm_tick`) — collapsing them to
  `quiet · N floors` is client pacing.

### Wire fields added (`web/src/engine/types.ts`, none renamed)

`WallPrice` {depth, past_from, past_to, past_pm, death_from, death_to, death_pm, sims, trade_off}; `PatchWhole.price`,
`PatchWhole.trade_off`; `Lever.price`, `Lever.trade_off`, `Lever.pending`; `ForecastTry.price`, `ForecastTry.trade_off`,
`ForecastTry.owned`, `refusal`, `wear`; `Counter.owned`, `refusal`, `wear`; `LedgerRow.counter.owned`, `refusal`, `wear`;
`Forecast.hunger` (`HungerForecast`); `CreditShare.credit` / `Death.credit` value `written`; `Ev` `homeward`; `Engine.buyRation`.

### Deviations

- The King is tougher again (700 hp, 21–27); the late ranks (5 000, 12 000) take more than a day's Legacy late in the fortnight —
  the cliff was rank 2; the top of the curve is a long goal by design.
- The hunger gate is measured on a D12 hunger floor (the same send with/without a ration, 16 seeds), not a PICKED+L fortnight.
- The forecast prices one `try` (the first a send reaches); deeper tries carry no price.
