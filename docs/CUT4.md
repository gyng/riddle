# Cut 4 — Legibility and feel, from the first blind cohort

*Contract, 2026-09-16. Cohort 1 (build 6e691ec, raters A seed 11 and B seed 23, Tier 1):
A 54.3 not yet, B 55.9 promising, before their absence redo (my harness omitted `dev=1`, so
the absence ran nothing; `return`/`pacing`/the empty-return kill flag are being re-rated).
Everything below is what both raters said independently, with their quotes.*

## Findings → changes

| # | Rater evidence | Change | Track |
|---|---|---|---|
| 1 | A: "tapping an offered patch when rows are full silently deletes your last rule (3 of 4 taps)"; B: "a tapped patch silently overwrote my rest rule" | A patch never evicts silently. When rows are full, inserting puts the editor **over budget** (`5/4`, red): `send` is disabled until the player removes a row; the row the core would have dropped is marked. Same for stall patches. | client |
| 2 | A: "'100% +0%' on every option, then '−15%/−30%' on every option, and 'gap' on every death even when my rule fired" | Patch rows read `survives 100% · base 75%` and show Δ only when ≠ 0 (`reach +5%`). Verdict: `gap` iff the best patch beats the baseline by ≥ 0.15 **or** moves the forecast ≥ 0.02; else `dice`. Re-measure the ≤ 5% dice gate; if it fails, the window (half-HP checkpoint) is widened to 40 turns, not the semantics. | core (trace.rs) + client |
| 3 | A: "a man swinging at nothing: in ~15 screenshots at 'attack nearest' I never saw a foe"; B: "the camera never shows a foe" | `Snapshot.entities` includes pursued-but-unseen hostiles with `remembered: true` at their last seen tile; renderer draws them dimmed, no shadow. The rule callout names the target and the act: `hunt goblin` when unseen, `attack goblin` when in view (≤ 3 words). | core + render |
| 4 | A: "HP fell 18 → 6 with no callout"; B: "60 s stretches of the same 'pick up'" | `hurt` events become callouts (`−7 archer`, red, 0.6 s); `pick up` shows once per streak; `explore` never. Events per minute at 1× must stay ≥ 6 with fewer words. | client (watch ticker) |
| 5 | A: "HUD $442 vs 'Returned with 199 loot' vs '+$64' in camp — I never learned what I bank"; B: "'Banked 232 loot' then $78" | One unit everywhere: loot is counted in gold at pickup (÷4 at the source), the exit note says `banked $58`, the HUD stake shows `$58`, the report tile shows the same number. | core |
| 6 | B: "marks dry up after the first D5 (0–2 per bank)"; A bought 6 unlocks in the hour, B one | **First bank at each depth** earns a mark (a distinct best: home from D5). Renown rank 1 arrives within the first hour for anyone who banks once. | core (meta) |
| 7 | A: story 0.3 ("the reel is thin"); B: "Uleth the rat fell" landed | Reel lines carry the turn: `Down to 2 HP, then banked $313.` (setup + turn + end in ≤ 8 words); the hero's chronicle names the row that saved it (`R3 rest caught him`). | core (sifter/chronicle) |
| 8 | A: "D6 sits at 0% with no hint of what is there" | Forecast row for `best+1` shows the top death cause even at 0% (`D6 0% · goblin warlord`) once the boss fact is known. | client |
| 9 | A: "'card: thief guard' ate 3 marks and changed nothing visible" | Tactic cards render as a row in the editor (`[card] thief guard`, not editable, movable) so the player sees where it sits; the forecast delta of buying a card is shown on the card (`reach +4%`) before purchase. | client + core (`unlocks()` carries `delta`) |

## Gates

- Cohort 1 redo cards: `return` ≥ 0.6 for both raters after the harness fix (else the return
  report itself is at fault).
- Patch overflow: a Playwright test that tapping a patch with full rows never changes any
  existing row and disables `send` until one is removed.
- Dice share ≤ 5% with the new verdict semantics (re-measured on 30 seeds).
- Unit test: `Snapshot.entities` contains a `remembered` hostile whenever a row targets one
  out of view; callout text for that action starts with `hunt`.
- Loot unit: the exit note, HUD stake and report tile agree on every seed (test compares).
- Cohort 2 (new build, new seeds, two raters) with α reported before any score is believed.

## Tracks

- **Client/renderer** (`web/src/**`): 1, 2 (display), 3 (render), 4, 8, 9 (display). Starts now.
- **Core** (`crates/**`): 2 (verdict), 3 (`remembered`, callout), 5, 6, 7, 9 (`delta`). Starts
  when the Cut 3 core lands (same files).
