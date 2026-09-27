# Riddle — copy (research, glossary, audit, passes)

*2026-09-27. The owner's brief: "Labels don't make sense, e.g. R1, R2. Fix writing, but AI slop — and
more — isn't better." Budgets stay as `eval/copy-budgets.json` sets them (callout ≤ 3 words, label and
button 1 word, verdict 1 word, no sentences in chrome, no tutorial text). This pass changes words, not
rules.*

## 1. Research

### 1.1 Microcopy rules (one line each)

| Rule | Source |
|---|---|
| Front-load: a scanner reads the first ~2 words (~11 characters) of a line; filler openers were picked right 15 % of the time, invented jargon 5 % | [NN/g, first 2 words](https://www.nngroup.com/articles/first-2-words-a-signal-for-scanning/) |
| People scan in an F: line starts get read, line ends skipped — the information-carrying word goes first | [NN/g, F-pattern](https://www.nngroup.com/videos/f-pattern-reading-digital-content/), [GOV.UK content principles](https://www.gov.uk/government/publications/govuk-content-principles-conventions-and-research-background/govuk-content-principles-conventions-and-research-background) |
| Numbers as numerals; digits catch fixations | [NN/g, numerals](https://www.nngroup.com/articles/web-writing-show-numbers-as-numerals/) |
| Short common words (`buy`, not `purchase`); no coined terms | [GOV.UK clear language](https://guidance.publishing.service.gov.uk/writing-to-gov-uk-standards/writing-guidelines/clear-language/) |
| Sentence case; all-caps runs read slower | [Microsoft style, capitalization](https://learn.microsoft.com/en-us/style-guide/capitalization) |
| Buttons are verbs that name the result (`apply`, `buy`), never `ok/submit` where a verb fits | [Mailchimp style guide](https://styleguide.mailchimp.com/), [Microsoft UI text](https://learn.microsoft.com/en-us/windows-server/manage/windows-admin-center/extend/guides/ui-text-style-guide) |
| One term per concept, everywhere; a synonym reads as a second thing | [Google developer style, UI elements](https://developers.google.com/style/ui-elements); Podmajersky, *Strategic Writing for UX* ([review](https://www.uxmatters.com/mt/archives/2020/02/book-review-strategic-writing-for-ux.php)) — edit for purpose, then concision, then voice, then clarity |
| An error/verdict says what happened, in the player's words, blames the right thing, and offers the fix | [NN/g, error messages](https://www.nngroup.com/articles/error-message-guidelines/) |
| A number has a unit or a referent; a bare delta reads as a chance (cohort 17: `bank +3%` read as odds) | NN/g numerals (above); PLATEAU.md cohort 17 |

### 1.2 How comparable games label dense systems

- **Into the Breach** shows intent instead of describing it; Justin Ma: "sacrifice cool ideas for the
  sake of clarity every time"; three sentences of tooltip lost to one looping animation
  ([Game Developer](https://www.gamedeveloper.com/design/-i-into-the-breach-i-dev-on-ui-design-sacrifice-cool-ideas-for-the-sake-of-clarity-every-time-)).
  For Riddle: the shaft, the replays and the reel carry meaning; words only name.
- **Slay the Spire**: an enemy's intent is an icon plus a number — no sentence
  ([wiki](https://slaythespire.wiki.gg/wiki/Intent)). Keywords (`Vulnerable`, `Weak`) are one fixed word each, never paraphrased.
- **Balatro**: `chips × mult` — two terms, two fixed colours, one formula on every hand
  ([analysis](https://blakecrosley.com/guides/design/balatro)). A number's colour is its unit.
- **Loop Hero**: players asked for tooltips on unlabelled glyphs and stats; the fix was more legible
  type, not more text ([Steam thread](https://steamcommunity.com/app/1282730/discussions/0/3112522283873518441/)).
  A glyph nobody decodes is a defect.
- **Dwarf Fortress** (classic): every screen dense text, keyboard-only; the Steam edition added icons
  and hover text on each symbol ([PC Gamer](https://www.pcgamer.com/the-new-dwarf-fortress-ui-looks-so-much-better/)).
- **Final Fantasy XII gambits** — the nearest cousin: `target condition → action`, checked top to
  bottom, the first that holds acts; priority *is* list position. Players name them by slot:
  "gambit #2" ([guide](https://www.thegamer.com/final-fantasy-12-complete-guide-gambits/),
  [FAQ](https://billpringle.com/games/ffxii_gambits.html)). **Dragon Age: Origins** tactics show a
  numbered list; the top slot has priority ([wiki](https://dragonage.fandom.com/wiki/Tactics_(Origins))).
  **Gladiabots** has no numbers — the layout is the order
  ([wiki](https://wiki.gladiabots.com/index.php?title=BotProgramming_Basics)).

### 1.3 AI-slop patterns banned from Riddle's copy

From Wikipedia's [Signs of AI writing](https://en.wikipedia.org/wiki/Wikipedia:Signs_of_AI_writing) and
word-frequency studies ([arXiv 2406.07016](https://arxiv.org/abs/2406.07016v1)):

- Brochure verbs and filler adjectives: *unleash, embark, delve, elevate, harness, seamless, robust,
  vibrant, intricate, pivotal, epic, legendary, mighty*.
- Faux-epic tone: *"Your hero's saga continues…"*, *"The depths await"*. Riddle's diegetic voice is flat
  and factual (`Freed a captive. She followed.`).
- Triads added for rhythm; `not X but Y`; em-dash chains; Title Case On Everything; emoji bullets.
- Hedging (`may potentially`), restating what the screen shows, summary lines (`Overall…`).
- Over-labelling: a caption on a thing that already reads (a label `gold` over `$40`), or a new coined
  handle for a mechanic that already has a plain word. More words are not more clarity.

### 1.4 What this means for the rule identifier

`R1`, `R2` is programmer shorthand nothing on the screen defines (NN/g's "invented jargon", 5 %). The research
first pointed at the genre's slot number (FF XII "gambit #2", Dragon Age's numbered tactics), and pass 1 shipped
`#2`. The owner's ruling overrode it: a number is still something a player has to remember after weeks away.
So a rule is named by **what it says** — its action plus the one condition that places it (`return at 20%`,
`drink heal at 30%`, `attack nearest`) — everywhere it is quoted; its place in the list shows only as a small
ordinal on its own tablet, under a `PRIORITY` head. This is also Slay the Spire's lesson: the thing on screen
is named by what it does, not by an index.

## 2. Glossary — one term per concept (final)

| Concept | On screen | Never | Notes |
|---|---|---|---|
| a `conditions → action` line | **rule**, named by what it says: `return at 20%`, `drink heal at 30%`, `attack nearest`, `attack boss` | `R2`, `#2`, `row 2`, `row` | `tokens.ruleName`: the action alone when no other rule shares it; else + its first placing condition (`at 20%`, `at D5`, `vs boss`, `vs 3+`); else + `if <cond>`. The core still writes `R2`; `dom.append` renders every string through `tokens.nameRefs` (`R2 return saved him` → `return at 20% saved him`; `R1 drank` → `drank`; `same as R2` → `same as return at 20%`); canvas callouts too (`render/state.ts`) |
| its priority | the small **ordinal** on the tablet, under a **PRIORITY** head | the ordinal used as a name | |
| the chips | **condition** / **action** (sheet titles, unlock cards `condition: alert`, `action: throw`) | `cond`, `verb`, `if`, `do` | |
| a rule slot | **+1 rule slot** | `+1 row` | gate `fill rules`; counter `4/4 rules + 3 tactics` |
| a ready-made rule | **rule: kite archers** (unlock card, oath reward); `tactic` on its tablet | `card:` | read as "an unlockable item" until it said `rule:` |
| one trip | **run** (`17 runs`, `vs last run`) | `send`, `sends`, `sent` | the button stays `send` |
| a hero of the family | **15th heir** (bar, report), `by heirs 2–14`, `heir 4 bones` | `♟15` | the pawn read as "party size" 4/4 |
| deepest floor ever | **best D8** | bare `D8` | |
| chance to get to a floor | **reach** (the shaft's head; `reach D6 78→26%`) | a bare % | |
| a move inside the noise | **same** | `≈`, `≈ ±14` | |
| the rules as they were | **was** (`survives 12/12 · was 7/12`, `reached D7 · was D5`, the vs line's `was dies · D8`) | `unpatched`, `base`, `vs` | |
| a death's 12 replays | **replays** (`7/12 replays survive`, `fixes · 12 replays of D4`) | `live unpatched`, `patches tie` | |
| a suggested rule on the death screen | a **fix** (block head `fixes`) | `patch` (internal) | |
| verdict seals (1 word) | **gap** (+ `no rule for it` on the headline when the margin does not say it) · **luck** · **order** · **rule** · **stall** · **route** · **repelled** | `dice`, `row`, `driven`; `unanswered`/`unmet` (tried, failed) | |
| a boss that holds the stairs | `behind warlord` | `sealed`, `sealed by` | |
| a boss's counter | `counter: attack boss` (the rule, when known) | `warlord: aim` (core word) | |
| run ends | **bank** · **return** · **death** · `~$113/run`; report tiles as shares of runs (`0/16 BANKED`) | bare `~$113`, `BANKED 0` | |
| a boss driving him out | **repelled** (`repelled by Warlord`, `REPELLED $0`) | `driven off` | |
| a rule's record | `fired 19/13754 turns · 17 runs` | `acts`, `sends` | |
| the vault's pick at an exit | **keep for heirs** | `home`, `keep`, `bring home` | |
| the pick at a cage | **from cages → weapon** | `cage pick`, `cage loot`, `cage · takes` | a world object the screen cannot define: fails every pass |
| watch modes | **highlights** · **fast** · **normal** | `fights`, `skim`, `1×` | |
| the purse after a death | `next heir +$40` · `stolen $8` · `paid $7` | `heir purse`, `carry −$8` | |
| forge estimate | `reach D6 +23` · `measuring…` | `D6 +23`, `…` | |
| bounty | `bounty 2× gold` | `$×2` | read as "$2" |
| route | `route D5 burrows`, `or fens · untried`, `fork D5` | `⑂` | |
| heir's rest | `heir rests 20m` | `send skips rest` | nothing punishes a send |

## 3. Audit (pass 0 → the fix)

Ratings: **clear** · **jargon** (a coined word or glyph the screen does not define) · **ambiguous** (two
readings) · **wrong** (says something else than it means) · **redundant**. Evidence: the pass-0 blind
readers (§4) and the QA / cohort quotes in `docs/PLATEAU.md`, `eval/qa/*`. Oaths are out of scope (another
track owns `oaths.ts` / `oath.rs`).

### Rule identifiers (everywhere)

| Where | Pass 0 | Rating | Now |
|---|---|---|---|
| tablet | `R1`, big | jargon | a small ordinal `1` — position only |
| trace column / rows | `R` · `R2 attack nearest` | jargon | `rule` · `attack nearest` |
| order verdict | `R5 under R2` | jargon | `drink heal under attack nearest` |
| row verdict | `goblin · D8 · R2 return` | jargon | `goblin · D8 · return at 20%` |
| patches | `move R4 above R2`, `cut R3`, `R3 ↻`, `drops R3`, `restore R2`, `at R2` | jargon | `move above attack nearest`, `cut`, `replaces return at 20%`, `drops drink heal`, `restore`, `already written` |
| why sheet | `WHY R1`, `↑ R2 first` | jargon | `WHY` + the rule's own chips, `under attack nearest` |
| shadow mark | `↑ R3` | jargon | `under attack nearest` |
| unlock card | `reach ≈ at R3`, `joins at R2` | jargon | `reach same above attack boss`, `joins above attack boss` / `joins at end` |
| reel / notes (core) | `R2 return saved him.`, `R1 drank`, `same as R2` | jargon | `return saved him.`, `drank`, `same as return at 20%` (rendered by `nameRefs`) |
| pending (core) | `R1 fired 4 of 16 runs: hp < 30% → drink heal` | jargon | `hp < 30% → drink heal · fired in 4 of 16 runs` |
| watch callout | `R2 · attack nearest` | jargon | `attack nearest` |
| divergence | `sent · R2`, `R2 now → lives` | jargon | `last run · return`, `return at 20% now → lives` |

### Camp · editor · shaft

| String | Rating | Fix |
|---|---|---|
| `♟15` (bar), `♟2–14` (report) | jargon — "party size" 4/4 | `15th heir`, `by heirs 2–14` |
| `D8` (bar) | ambiguous — "current depth" | `best D8` |
| shaft `D6 70%` | ambiguous — "success rate" | a `REACH` head over the column |
| `▼4` on a notch | jargon | `−4` (signed points; the vs line says against what) |
| `D9 · sealed` / `· warlord` under a boss | ambiguous | `behind warlord` |
| `warlord: aim` (core fact) | wrong — read as the boss's move | `counter: attack boss` when known (the fact word is oath.rs's) |
| `~$113` | ambiguous | `~$113/run` |
| `VS SENT` | jargon | `VS LAST RUN` |
| `bank ≈` | jargon | `bank same` |
| `heir rests 20m · send skips rest` | wrong — read as a cost | `heir rests 20m` |
| `set 2 · 0` | ambiguous | `set 2 · empty` |
| `cond` / `verb` sheet titles | jargon | `condition` / `action` |
| `no rows`, `written: 2 of 4 rows` | redundant term | `no rules`, `… rules` |
| `cage pick → weapon` | jargon (a world object the screen cannot define) | `from cages → weapon`, the same words on the vault panel — still fails |
| `◐` (look stud) | jargon — glyph, "no idea" ~10× a pass | kept (an icon, not copy): for the art track |
| `⑂` (fork) | jargon — glyph, 8/8 "no idea" | the words `route`, `fork D5` |
| tablets' `1`, `2` | ambiguous — order, never which acts first | a `PRIORITY` head |
| `set 2 · 0`, `fighter · 4`, `✎` | ambiguous | `set 2`, `fighter · 4 rules`, `rename` |

### Watch

| String | Rating | Fix |
|---|---|---|
| `carry $3 · death: lose all` | ambiguous — "a carry limit" | `carrying $3 · death: lose all` |
| `fights` (mode), `1×` | ambiguous — "combat log" 6/6; `1` "no idea" | `highlights` (read as "key moments"), `normal` |
| `28/36` | ambiguous — "XP or rooms" | `28/36 hp` |
| `DRIVEN $0` | wrong — "I drove off enemies" | `REPELLED $0` |

### Death

| String | Rating | Fix |
|---|---|---|
| seal `GAP` | jargon — "no idea" 4/4 alone | tried `UNANSWERED` (did not fit the seal), `UNMET` ("a goal not met" 6/6); kept `GAP`, the headline says `no rule for it` |
| seal `DICE` / `ROW` | jargon | `LUCK` / `RULE` |
| `7/12 live unpatched` | wrong — read inverted | `7/12 replays survive` |
| `a max hit at 4 hp · 1 in 2` | ambiguous — "hit for 4" | `max-damage hit at 4 hp · 1 in 2` |
| `lineage` (bar) | jargon — 6/8 | removed (`family` read "no idea" 5/6 too) |
| the item wall (`bones on D8: leash, summon ally, …, left behind: identify ×2, …`) | redundant | `bones: 9 items on D8 · stolen $8`; swaps and finds left behind are the gold sheet's (the line is its button) |
| `stolen … · carry −$8`, `swapped out … · carry −$7` | ambiguous — "upkeep cost" | `stolen $8`, `paid $7` |
| `heir purse +$40` | ambiguous — "another heir's pool" | `next heir +$40` |
| patch tablets (no head) | ambiguous — read as his own rules 7/8 | head `fixes · 12 replays of D4` |
| `survives 12/12 · unpatched 7/12` | jargon | `survives 12/12 · was 7/12` |
| `reach D5 ≈ ±14` | wrong — "about D5, give or take 14" | `reach D5 same` |
| `return early · D4 88→74%` | ambiguous — "% of what" | `return early · reach D4 88→74%` |
| `D4 death · replayed · patches tie` | jargon | folded into the head; `tie` dropped |
| gem `harms` | jargon | `risky` (still read as a general risk) |
| `learned 3` | ambiguous — "3 things" | `3 facts learned` |

### Report · sheets · unlocks · forge

| String | Rating | Fix |
|---|---|---|
| `warlord: aim learned` (core news) | wrong | `warlord counter: attack boss` |
| `driven off: Warlord`, tile `DRIVEN` | wrong | `repelled by Warlord`, `REPELLED`; the counter tablet `Warlord repelled him ×3` |
| tiles `BANKED 0` | wrong — read as gold banked 4/4 | `0/16 BANKED` (shares of the runs) |
| `deeper: D7, last D5` | ambiguous | `reached D7 · was D5` |
| `bones on D5 · D8` | jargon | `bones to recover: D5 · D8` |
| `R1 fired 4 of 16 runs: …` | jargon | the rule's chips first, `· fired in 4 of 16 runs` |
| why `19/13754 acts · … · 17 sends` | jargon | `fired 19/13754 turns · … · 17 runs` |
| vault `home` | ambiguous | `keep for heirs` |
| vault `cage · takes weapon` | ambiguous | `from cages · weapon` (the tablet's words) |
| unlock `+1 row`, `⊘ fill rows` | term | `+1 rule slot`, `⊘ fill rules` |
| unlock `card: kite archers · reach ≈ at R2` | jargon — read as "an unlockable item"; the reach part as a requirement | `rule: kite archers · vs archers` |
| unlock `verb: tame`, `cond: alert` | jargon | `action: tame`, `condition: alert` |
| forge `· …`, `D9 +7` | jargon — "no idea" 6/6 | `· measuring…`, `reach D9 +7` (still read as a stat bonus) |
| oath `[Warlord] [fire]`, `card: gas step`, `warlord: aim` | jargon | `[slay Warlord] [with fire]`, `rule: gas step`, `counter: attack boss` (hidden on the fire oath, where it read as a contradiction); a broken oath names its rule (`OATH BROKEN · return at 20%`) |

## 4. Passes — blind comprehension

### Protocol

Everything is in `scratchpad/copy/`: `capture.mjs`, `mkrater.sh`, `reader-prompt.txt`, `runner-prompt.txt`, `key.md`,
`screen-probes.md`, `words.py` / `words2.py`, each pass's shots, answers and `scores.md` (`pass0` … `pass8`,
`final`), and the before/after sheet `sheet.png` (top row pass 0, bottom row final).

- **Capture** (`scratchpad/copy/capture.mjs <passN>`): the same walk every pass on the dev server, headless
  400 × 800 @2×, seed 4242 — first camp, two watch frames, the first death, the 8 h report, the worst
  death (and a second fix lit), the camp with four rules and the forecast panel, the why / loadout /
  unlocks / vault / forge / oath sheets, a second run's report. A screen that scrolls is shot whole on a
  400 × 2000 viewport (`-tall.png`); `mkrater.sh` stages one image per screen under a neutral name.
- **Readers**: two fresh readers per screen per pass, never reused (Sonnet subagents), each shown ONE
  screenshot with no other screen, code or doc, asked to write what every visible label, number and
  symbol means and tag each `[clear]` / `[guess]` / `[elsewhere]` (needs something from another screen or
  a remembered code) / `[no idea]`.
- **Scoring** (a fresh scorer per pass, `runner-prompt.txt`, against the fixed concept key `key.md` —
  55 probes; `screen-probes.md` maps them to screens; a probe counts only where its label is visible in
  the screenshot): comprehension 1 / 0.5 / 0 per probe; **stands alone** = understood (≥ 0.5) and not
  tagged `[elsewhere]`/`[no idea]`.
- Pass 0 was also read the old way first (two Opus readers seeing all 17 screens in order): 86.7 % / 85.7 %
  — they decoded `R1` as "rules in priority order" without trouble. The owner's bar is a human seeing one
  screen after weeks away, so from pass 1 on every number is the one-screen protocol, and pass 0 was
  re-read under it (64.5 % / 72.8 %).
- Pass 1 was scored against the .txt dumps (a probe below the fold counted 0) and on 16 viewport shots; from
  pass 2 the scorer checks the image and the screens are whole (tall shots).

### Scores per pass (comprehension % / stands-alone %, mean of the two readers)

| Surface (probes/reader) | p0 | p1 | p2 | p3 | p4 | p5 | p6 | p7 | p8 |
|---|---|---|---|---|---|---|---|---|---|
| camp · editor · shaft (24–25) | 57 / 62 | 57 / 66 | 82 / 88 | 75 / 85 | 74 / 76 | 85 / 88 | 85 / 82 | **90** / 82 | 79 / 82 |
| watch (4–6) | 75 / 62 | 62 / 92 | 70 / **100** | 71 / 67 | 75 / 75 | **100 / 100** | 71 / **92** | 88 / **100** | 81 / **100** |
| death (31–36) | 67 / 77 | 85 / **93** | 89 / **90** | 85 / 88 | **96 / 94** | **90 / 91** | **95 / 96** | **94 / 94** | **92 / 96** |
| report (11–13) | 63 / 81 | 58 / 62 | 50 / 64 | 73 / **91** | 75 / 86 | 89 / 77 | 84 / 82 | 86 / 77 | 84 / 82 |
| sheets: why · loadout · vault (6) | 67 / 83 | 75 / 83 | 75 / 83 | 83 / **92** | 83 / **92** | 75 / 83 | 83 / 83 | 83 / 83 | **92** / 83 |
| unlocks (2) | 75 / **100** | 75 / 75 | 50 / 50 | 75 / 75 | 75 / **100** | **100 / 100** | **100 / 100** | **100 / 100** | **100 / 100** |
| forge (1) | **100** / 0 | **100 / 100** | **100 / 100** | **100 / 100** | **100 / 100** | 75 / **100** | **100 / 100** | 75 / **100** | **100 / 100** |
| oaths (2–3) | — | — | — | — | 58 / 50 | 83 / **100** | **92 / 100** | 75 / 83 | **100** / 50 |
| **overall** | **64.5 / 72.8** | **70.7 / 79.3** | **79.1 / 85.3** | **79.2 / 86.3** | **83.1 / 85.5** | **87.9 / 88.8** | **88.4 / 89.2** | **89.9 / 87.9** | **87.1 / 88.2** |
| readers A · B, comprehension | 64.2 · 64.8 | 73.0 · 68.4 | 80.0 · 78.2 | 80.4 · 78.0 | 82.6 · 83.7 | 88.2 · 87.6 | 89.2 · 87.5 | 90.2 · 89.7 | 87.1 · 87.1 |

Bold: at or over the 90 % bar. Every pass's two readers agree within 10 points overall (largest gap: stands-alone,
pass 7, 90.8 vs 85.1). Small surfaces swing on one probe (forge 1, unlocks 2, oaths 3, watch 4–6 probes per reader).

### What each pass changed (and why)

- **Pass 1** — rule identifiers and the first glossary. `R1` → the rule's name everywhere (first `#2`, then the
  owner's ruling: no number a player has to remember — names from `tokens.ruleName`, core text through
  `dom.append` → `tokens.nameRefs`); the tablet keeps a small ordinal. `unpatched` → `was`; `live unpatched`
  → `replays live`; `patches tie` → `patches equal`; `acts`/`sends` → `turns`/`runs`; `cond`/`verb` → `if`/`do`;
  `+1 row` → `+1 rule`; vault `home` → `keep`, `cage · takes` → the tablet's `cage pick`; verdict seals
  `dice` → `luck`, `row` → `rule`.
- **Pass 2** (death 67 → 85 → 89) — `♟15` → `heir 15`; bar `D8` → `best D8`; `lineage` → `family`; a
  `REACH` head on the shaft; `sealed` → `behind warlord`; a known counter as its rule (`counter: attack boss`,
  also `warlord counter: attack boss` over the core's `warlord: aim learned`); `≈ ±N` → `same`; patch head
  `fixes · 12 replays of D4`; `return early · reach D4 88→74%`; `~$113/run`; `max-damage hit`; brief death
  line (counts, not item walls); `stolen $8`, `paid $7`, `next heir +$40`; `vs sent` → `vs last run`;
  notch `▼4` → `−4`; `driven` → `repelled`; forge `…` → `measuring…`; dropped `send skips rest`.
- **Pass 3** — `gap` tried as `unanswered` (did not fit the seal) and `unmet` (read "a goal not met" 6/6):
  back to `gap`, with the headline carrying `no rule for it` when the core's margin does not; `family` label
  removed (11/14 "no idea"); watch `fights` → `skim` (then `highlights`, pass 4); `card` → `tactic` (then
  `rule:`, pass 5); the counter tablet says who (`Warlord repelled him ×3`); vault `keep` → `bring home`;
  route `⑂` → the word `route`; the shaft's words wrap instead of clipping.
- **Pass 4** — `replays live` → `replays survive` (read inverted); patch tablets drop their `±`; gem
  `harms` → `worse` (→ `risky`, pass 6); report exit tiles as shares of the runs (`0/16 BANKED` — read as gold
  4/4 before); `deeper: D7, last D5` → `now D7 · last run D5`; unlock cards drop `reach same above …` (read as
  a requirement); forge moves say `reach D6 +23`; `15th heir`; oath board included (the oath track handed it
  over): the counter as its rule, `tactic:`/`rule:` rewards, the oath's broken rule by name.
- **Pass 5** (87.9 / 88.8) — a `PRIORITY` head over the tablets (6/6 read the ordinals as order, none as
  which acts first); watch `28/36 hp`, `1×` → `normal`; set tabs `fighter · 4 rules`, `rename` for `✎`;
  `+1 rule slot`, `rule: kite archers` (a ready-made rule, never read from `card:`/`tactic:`), `action:`/
  `condition:`; `bounty 2× gold` (`$×2` read as "$2"); oath chips `slay Warlord · with fire`;
  `bones to recover: D5 · D8`.
- **Pass 6** (88.4 / 89.2) — `cage pick` → `cage loot`; vault `bring home` → `keep for heirs`; gem `risky`; `fork D5`
  for the last `⑂`s; word cuts: the death line drops swaps and finds left behind, the counter tablet drops
  `counter unwritten` (its `try:` says it), empty set tabs drop `· empty`.
- **Pass 7** (89.9 / 87.9) — `learned 3` → `3 facts learned`; `now D7 · last run D5` → `reached D7 · was D5`;
  `cage loot` → `from cages`; the fork's other lane `fens · D5 · untried` → `or fens · untried`.
- **Pass 8** (87.1 / 88.2) — `survives 8/12 replays` (reverted: no gain, one more word); the fire oath drops its
  `counter:` line. Third pass under +3: stop.

### Where it stopped, and why

Passes 6, 7 and 8 moved overall comprehension +0.5, +1.5 and −2.8 (pass-to-pass noise between fresh reader
pairs is about ±3), so by the owner's rule the passes stop here: **87–90 % comprehension and ~88 % stands-alone
overall**, from 64.5 / 72.8. Death, unlocks and forge are over the bar; camp, report, sheets and watch sit at
79–92 with stands-alone at 77–83 on camp, report and sheets. What the remaining misses have in common, and
what would unblock each:

1. **A world concept one screen cannot define.** `from cages → weapon` (0 every pass, three wordings),
   `keep for heirs` on the vault (never read as "kept between heirs"), `bones to recover`, `◆ 21` marks ("don't
   know what it buys"), `★ 1`, `oaths ³`, `bounty`, `AVENGED …`, `grudge`, the fork's other lane (`or fens ·
   untried`). The copy has 1–3 words and no room to say what a cage or a mark is; the readers were told they
   remember nothing. Unblock: an icon that shows the thing (a cage glyph on the tablet, the ◆ drawn as a key that
   opens unlocks), or a first-time reveal beat that names it once (UI.md §5's ladder) — a design call, not copy.
2. **A budget that forces jargon.** `label` and `button` are 1 word and `verdict` is 1 word: report tiles cannot
   say `banked runs` (`0/16 BANKED` helps, still "some exit type" 3 of 8), the seal cannot say `no rule`
   (`GAP` holds only because the headline says `no rule for it`), the watch mode cannot say `fights only`
   (`highlights` reads as "key moments"). Unblock: a 2-word allowance for tile labels, the seal and the
   command card, or icons that carry the second word.
3. **Numbers the tests pin in their current form.** The shaft's per-floor move (`88% ±7 −4`, read as a range or
   a penalty every pass; `▼4` before it read "no idea") and the death gem's count (`12/12 apply`, read as "all
   fixes selected" and misread as `17/12` in the display face) are asserted by cut22/qa778 and ui/qaAC. Unblock:
   let the move ride only the `vs last run` line (drop the per-notch mark), and the gem say `apply` alone — both
   are gate changes, not wording, so they are left for the owner.
4. **Data that reads as a contradiction.** A fix with `survives 8/12 · was 0/12` over `death 75→100%` (this
   death's replays vs every run's death share) was flagged in every pass; `survives 8/12 replays` (pass 8) did
   not help and was reverted. Unblock: show the fix's whole-run line only when it agrees with its survive count,
   or split the tablet into `this death` / `every run` rows — a layout change.
5. **Core-emitted words the client can only reword around.** `a max hit at 4 hp`, `warlord: aim` (oath.rs
   `counter_word`), `learned 3`, `deeper: D7, last D5`, `driven off: X`, the `R2 …` rule ids: all are rewritten at
   render (`nameRefs`, `said2`, `newsTexts`), which works on screen but leaves the core's text (morgue export,
   future surfaces) in the old words. Unblock: move those strings to the glossary's words in the core
   (chronicle.rs, sifter.rs, offline.rs, turn.rs, oath.rs, trace.rs) with their tests — a core pass.

### Word count

Rendered copy on the screens the capture makes deterministic (watch ×2, the two deaths, the two reports and the
why / loadout / vault / unlocks / forge sheets; a word = a token with two or more letters, so `R2`, `D8`, `$40`
are data): **pass 0 470 → final 477**. Pass 0's two death screens were captured before their patches were measured
(`reach …`, a `measuring` gem): measured, they carry ~6 words each that the final capture shows (`reach D5 same`,
`return early · reach D4 88→74%`, `death 100→14%`, `apply`), so like for like the final copy is ~5 words under
pass 0. All rendered words on every captured screen (the camp's async `vs` lines included): 651 → 647, with the
oath board (25 words, not in pass 0) inside the 647. The per-surface moves: death −25 (the item walls), unlocks
−14 (`reach ≈ ±3 at R3`), camp −22 (`send skips rest`, the ids), sheets +2, forge +7 (`reach D6 +23` where `…` was),
report +11 and watch +12 (`15th heir`, `hp`, `repelled by`, `bones to recover`, `highlights`/`normal`).
