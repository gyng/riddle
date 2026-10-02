# Riddle — keyword tooltips (contract)

*2026-10-02. The owner: "add concise tooltips on keywords + highlight keywords etc. use judiciously so the ui doesn't become
cluttered. set a target clutter/density that helps explain the game + use appropriate styling that doesn't make it too
cluttered and intimidating."* Builds on the Cut 29 concept system (`web/src/ui/concepts.ts`: an icon and a one-time ≤ 3-word
caption per world concept). One registry, one tokenizer, one plate. UI layer only: no wire change, no game truth in TS.

## 1. What the best games do (one line each)

| Game | What gets marked | Nesting | Input | Fading |
|---|---|---|---|---|
| Slay the Spire | only mechanic keywords on cards (`Vulnerable`, `Exhaust`), in GOLD; never flavour words | a hovered card shows every keyword's box beside it, one level, never a keyword inside a keyword box | hover, no delay; on mobile, hold the card | never fades; the boxes are always one hover away |
| Hades | boon keywords (`Weak`, `Doom`) in a colour, with an icon | an inline glossary box under the boon, one level | the boon screen shows them at rest, no hover needed | never fades |
| Balatro | values and suits in colour, keywords on card hover | joker hover shows the referenced card's box | hover; on touch, tap the card (tap again to act) | never fades; the base game shows almost no text at rest |
| Marvel Snap | `On Reveal`, `Ongoing` bolded; the card text is short enough that keywords are the only marked words | tap a card → full card + keyword glossary under it | tap opens, tap elsewhere closes | never fades |
| Path of Exile | none at rest; holding Alt shows the "advanced mod" layer (tiers, ranges) | one level | an explicit modifier key: the detail is opt-in, never on screen by default | the opt-in *is* the fade |
| Dota 2 / WC3 | ability tooltips framed like the console (title, cost, body), live numbers per level | Dota's "alt for more" | hover with a short delay (Dota ~0.3–0.5 s) so moving across the bar does not flicker plates | not faded |
| Into the Breach | every unit/tile has a hover panel; the board itself has no marked words | pilot/weapon → effect panel, one level | hover; the tooltip shows the live numbers (damage, push) on the board | not faded |

Lessons taken: (1) mark only real mechanics, never ordinary words; (2) one level of nesting, shown together rather than as
a chain of popups; (3) a short hover delay stops flicker on desktop, and on touch the long-press belongs to the term while the
tap belongs to the control; (4) the strongest games put *live numbers* in the tip (WC3, Into the Breach); (5) none of them fade,
but none of them mark words in chrome at rest as often as a text-light idle UI would — PoE's opt-in layer is the closest to
"learned": the marker goes, the tip stays one gesture away.

## 2. Targets

| Target | Bar | Why (where it moved from the proposal) |
|---|---|---|
| Keywords | glossary terms only (COPY.md §2 + the concepts): ~35 ids, never an ordinary word | as proposed |
| Per panel | a keyword is marked on its first occurrence in a panel only | as proposed |
| Per line | ≤ 1 marked keyword per visual line | as proposed |
| Per screen at rest | ≤ 4 marked keywords visible | as proposed (desktop too: its three columns make it denser, not sparser) |
| Share of words | marked ≤ 8 % of the visible words; **a screen under 25 words may carry 1** | day-0 town reads ~10 words; 8 % of that is 0, and the one mark there is the only sign that terms have tips |
| Captioned concepts | a concept whose one-time caption is on screen is not also marked | the caption already explains it; two treatments on one word is clutter |
| Fade | a keyword is learned (plain text) once its tip was opened **2×**, or after **8 sightings** (a sighting: marked and on screen ≥ 1.5 s, at most one per keyword per 5 min) | the proposal's N set to 8: about two sessions of play before an unasked term goes quiet |
| Learned / quiet | still a tip on hover (desktop), long-press (phone), focus (keyboard) | as proposed |
| Store | per viewer, localStorage `riddle.keywords` (like `riddle.concepts`); never the save, never game truth | as proposed |
| Tip copy | `tooltip` surface: ≤ 10 words (gloss + live value), no sentences, no `you`; the term is the plate's title | as proposed |
| Live values | read from the wire (`app.lineage`, the last forecast) at open: marks held, the worn stance's level and runs, the bank's balance and cap, the quest's progress, tracks' stages | WC3 / Into the Breach |
| Styling | a fine dotted underline in GILT, the word's own colour and weight; learned: no underline. No bold, no new colour | quiet; the text keeps its metrics (no layout shift) |
| Plate | BONE on UMBRA, the `tablet` 9-slice frame (the rule tablets' iron-edged plate), max 240 px, title in the display face | the existing frame language |
| Open at once | ≤ 1 plate | as proposed |
| Nesting | a keyword inside a tip is marked; hover/tap adds its gloss as a second line *inside the same plate*; nothing deeper | StS's stacked boxes, inside the one-plate rule |
| Desktop | hover 350 ms, close 150 ms after leaving (the plate can be entered); click opens at once; focus opens | Dota's delay |
| Phone | tap a marked keyword → tip; tap elsewhere closes. A keyword inside a control (button, tile, chip, tablet) never takes the tap: long-press 450 ms shows the tip and the release does not click | as proposed |
| A11y | `aria-description` on every keyword (its gloss), `aria-describedby` to the plate while open, the plate `role=tooltip`; Escape closes it; no motion under reduced motion | as proposed |
| Not here | the watch's live combat text | too busy |

## 3. Where

| Screen | Marked (at most, before the budget) | Quiet tip (hover / long-press) |
|---|---|---|
| top bar | `heir` | `$` gold (a button: long-press), `◆` marks, `★` renown, `best D` (stats: a tap opens) |
| town | — | the buildings (bank, forge, vault, kennel, quest board, the crate, the next plot), every console tile (packages, quest, loadout, forge, vault, kennel, bank, edit) |
| packages | `packages`, `stance`, `tactic`, `temperament`, `drills`, `scarred`, `reach`; the camp strip's slot word (in its button) | — |
| tracks | `tracks` | — |
| quest | `quest` | — |
| report | the grew lines (`package`, `bank` …), `banked`, `returned`, `marks`, `deaths`, `plateau` | — |
| death | — | the seal (its verdict), the lever tablet |
| forecast / shaft | `reach`, `ends` | the shaft (`reach`) |
| the pen | `priority`, `condition`, `action` | — |

## 4. Gates (`web/tests/tips.mjs`, phone 400 × 800 and 360 × 740, desktop 1440 × 900)

- On the camp (day 0, mid-game), the packages, tracks and quest panels, the pen open, the report and the death screen: each
  keyword marked once per panel, ≤ 1 per line, ≤ 4 per screen, ≤ 8 % of the visible words (1 allowed under 25 words).
- Every registry entry renders a tip of ≤ 10 words (live value included); copy-lint passes with the `tooltip` surface.
- A tap on a keyword inside a button runs the button and opens no tip; a long-press shows the tip and does not run it.
- Two opens fade a keyword to plain text; it still opens on hover.
- At most one plate in the DOM, ever; opening a second closes the first.
- Opening a tip moves nothing (no layout shift ≥ 1 px); the plate stays inside the viewport at 360 px.
- The client suite, tsc and copy-lint stay green.

## 5. Blind check

Two fresh readers (≤ 25 tool calls, ≤ 15 images each), before/after shots of the day-0 town, the packages panel and the report.
Bars: clutter and intimidation within ±0.5 of before; understanding +1 or better. If they find it busy, the density goes down.
Results: §6.

## 6. Results (2026-10-02)

- Built: the registry is `web/src/ui/concepts.ts` (`TIP`, `ALIASES`, `TITLE`, `LIVE` beside the concepts' icons and captions; 44
  terms); `web/src/ui/tips.ts` (`kw`, `kwText`, `kwHost`, the density pass, the fade, the plate, the input); `web/src/tips.css`;
  the `tooltip` surface in `eval/copy-budgets.json` (≤ 10 words, no sentences).
- Deviations found in the build: a hover's plate lets the pointer through (a plate over a tile's badge stopped a click; a tap's, a
  click's or a long-press's plate takes the pointer, for its nested keyword); a focus opens a plate only after Tab (a script's focus
  after a key opened plates over controls); no pass runs on the watch and ≤ 3 passes a second elsewhere (the watch HUD repaints;
  fights' timings); marks are chosen in reading order, not least-seen first (blind check: the set changed between two looks).
- Gates: `web/tests/tips.mjs` 70/70 (phone 400, phone 360, desktop; repeated ×5). tsc, copy-lint 0 violations. The client suite
  on a box at load 40–50 (other agents) flakes on both trees (the baseline cut30-client alone: 23/39); every test that failed in a
  full run on this branch passes in a targeted rerun.
- Blind check, two fresh readers (16 tool calls, 13 images each), before → after: clutter 3 → 3 and 3 → 3; intimidation 3 → 2 and
  3 → 2; understanding 2 → 3 and 2 → 3 (+1 each). Density "about right", "slightly sparse where it matters". Fixed from their notes:
  a tip on the package prices (`past +27`, read as the most opaque words), drill (`written in · revocable` → `added to his rules ·
  can be undone`), bank (`$0 of $3000` → `$0 in · cap $3000`), the package tip's live value (`worn: Steady L3`), a stable set of
  marks per screen.
- Shots: `scratchpad/tips/` — `before/`, `after/` (at rest and each marked term's tip open, phone and desktop, headed GPU),
  `sheet.png`.
