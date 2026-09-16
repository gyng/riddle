# Cut 9 — The last four points to "fun"

*Contract, 2026-09-17. Cohort 4 (build 1cb869d): 71.9 · 67.6, α 0.901, mean 69.8, no gates.
"Fun" is ≥ 72. Remaining lost weight: story, pacing, expression, clarity at 1.2 each. Both
raters' evidence is small and specific; this cut is those items, verbatim, and nothing else.*

## Findings → changes

| # | Rater evidence | Change | Track |
|---|---|---|---|
| 1 | H: "the condition picker let me choose 'on see stray' while it was locked (trace: R1 locked cond)" | The vocabulary sheet shows only unlocked tokens; locked ones appear dim with their `needs` text and cannot be chosen. `Vocabulary` gains `locked: { cond, needs }[]` so the sheet can show *why*. | core + client |
| 2 | G: "unlock tiles buy on one tap with no description (◆7→◆5 for 'verb: throw'; I still don't know what throw does)"; H: "'+1 party' sits disabled with no reason" | Tapping an unlock card opens its sheet first (rows/effect, cost, `needs`) with a `buy` button; a second tap on the sheet buys. Disabled cards show their `needs` line (Cut 6 gave `needs`; ensure every gated card carries one, including party slots: `tame once`). | core (needs for every gate) + client |
| 3 | G: "forecast refreshes ~5 s late and fluctuates between reads with no edit (D4 86% then 71%)"; H: "57→70→58→64% for an unchanged rule set"; H: "unlock chips flip between reach −16% and +33%" | Cut 6 made the forecast deterministic per (rules, lineage seed, depth); the drift comes from `best_depth` moving (new facts) and from the refine pass replacing the 50-sim number with a different 100-sim number. Fix: the refine pass *widens* the same seeds (first 50 identical), so the number can only sharpen, and the panel shows `±N` (the binomial half-width) so a 3-point wobble reads as noise. Unlock deltas use the same seeds as the panel. | core |
| 4 | G: "cards are 'always' rows that eat every turn with foes present, and the editor never says so — both my deaths came from putting a card above my own row" | A card row in the editor reads `[card] pack break · foes ≥ 2` (its trigger, from the core's `unlock_rows` first row's conds) and the inline rows already show; the forecast `yours` line adds `· card R2 first` when a card sits above any player row. | client |
| 5 | H: "'hp < 30% → drink heal' at the top did not fire at 9/38 with three heal potions bought, and returned runs get no trace" | Returns and banks get the same last-5 trace as deaths, shown on the exit sheet's `trace` chip and on the report's exit line (`open`). The Cut 6 row accounting then names why R1 did not fire. (Likely cause: `trait first` or `card passed`; the trace will say.) | core (trace on every exit) + client |
| 6 | H: "five of the seven reel sentences are the same threat/resolution"; G: "the reel is an event list; the turn of my best run is not in it" | Reel dedupe by (threat, resolution) already exists per absence; extend to *across the last three absences* and prefer episodes whose turn beat is a row or combo over `no row fired`. The best-depth run's closing episode always leads. | core (sifter) |
| 7 | H: "only the latest death keeps its trace; older ones collapse to one reel sentence" | The graveyard keeps the last 5 deaths' traces; the chronicle sheet's heir line opens that heir's death (trace + patches) if kept. | core (cap deaths at 5 with traces) + client |
| 8 | G: "PENDING said 'R1 never fired' two lines above a reel saying 'R1 drank'" | `never fired` is computed over the same window as the report (the absence), and reads `R1 fired 0 of 15 runs` (numbers, no adverb). | core |
| 9 | H: "'slow' is dead (~100 s per floor)"; G: "at 1× dead stretches of 20–32 s" | `slow` is removed from the watch; the buttons are `auto · fast · ▶▶|`. Auto already runs fights at 1× and travel at 8×; nobody chose slow for anything but the rubric. | client |
| 10 | H: "'+1 party' sits disabled with no reason"; "the forge does nothing" | Party slot `needs: tame once`; the forge sheet shows, per kind, `salvaged 3/5 → craftable` so a player sees the ladder. | core + client |

## Gates

- Vocabulary sheet never offers a locked token (browser test).
- Every non-`available` unlock carries `needs` (core test over the whole catalogue at a fresh
  lineage and at ascension).
- Forecast: two reads of an unchanged set differ by ≤ the shown `±N` (test on 30 seeds).
- Every exit (bank/return/death) has a 5-turn trace with row accounting (test).
- Reel: ≤ 1 line per (threat, resolution) across the last 3 absences (test).
- Cohort 5: mean ≥ 72, no gates, α ≥ 0.80.

## Tracks

- **Core** (`crates/**`): 1 (`Vocabulary.locked`), 2 (`needs` everywhere), 3, 5, 6, 7, 8, 10.
- **Client** (`web/src/**`): 1, 2, 4, 5 (exit trace chip), 7 (chronicle → death), 9, 10.
