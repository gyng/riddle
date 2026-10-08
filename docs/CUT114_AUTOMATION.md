# Cut 114 — automation that serves the player (queued after Cut 113)

*2026-10-08, owner: "follow up progression improvements with automation improvements specifically".
Today (`tree.rs`, docs/AUTOMATION_TREE.md): eleven workers on one trunk; a chore done by hand 2–3 times
(or a fallback age, 24–44 h) lights its worker; one hire at a time for a share of a forge unit; ranks
II–III after 4 · 8 · 12 days; each worker's first act is announced and the worker can be paused.
Cohorts c4705f9 · 1fb7786: the apprentice "spent −$24 500 of my gold without asking" (both raters);
"most deaths came offline under drilled/default rules"; "every return asked forge + legacy + tactics +
unlock + worker taps ×3 bloodlines"; and `Workers pay the away player` fails by half a floor
(Cut 110 §4, a recorded deviation).*

## Principle

A worker carries out an order the player gave and reports what it did — never a silent policy
(AGENTS.md: policy is chosen, drilled or written, never silent; nothing punishes absence).

## Queue

1. **Every worker takes a standing order** (Cut 112 starts with the apprentice): one chip of two or three
   words each — apprentice `all spare · half · off`; clerk `keep N nights`; armourer `wear finds · keep
   forged`; guide `frontier · safe · off`; keeper `keep pref`. Default orders leave the returning player a
   purse and a decision. Gate: IDLE bars unchanged (its defaults); PICKED's orders measured on the
   paired panel; copy budgets.
2. **A worker's ledger on every return**: per worker what it did and at what cost (`apprentice · sword +3 ·
   mail +2 · −$X`, `clerk · banked $Y`, `guide · start D19`), one line each, folded under the report's
   head. Gate: every worker act in the absence appears (count = `tree.acts` delta); gold reconciles.
3. **Workers that pay at a wall** — the Cut 110 §4 deviation's fix: the drillmaster swaps a known wall's
   counter tactic into a slot while the sends meet that wall and swaps the player's tactic back after
   (announced `DRILLMASTER · mirror read`, revocable, never over a pen row). Gate: `automation-pays`
   PASS without a lowered bar; `workers-daily` holds.
4. **One order for every bloodline** (with Cut 113 §7): an order set on one bloodline offers `same for all`;
   hires stay per town. Gate: taps per three-bloodline check-in ↓ (ui QA script).
5. **Ranks you choose**: at rank II a worker offers one of two perks (apprentice `cheaper steps · keeps a
   reserve`; scout `shorter rest · safer start`) instead of a fixed bonus. Gate: neither perk dominates on
   the dayplayer's AWAY and DAILY bots.
6. **What comes next, visible**: the town's next worker shows its chore count (`forge 2/3`) and its
   fallback (`or 30 h`) on its post; a lit hire glints once (≤ 3 markers). Gate: cut30town's marker and
   fold budgets.
7. **After the clear**: a late worker (the herald's rank III or a new `chronicler`) carries the post-clear
   Ascension sends the owner opted into, so a cleared dungeon keeps an away loop. Gate: no auto-ascension
   without an explicit order; IDLE never slays the King in a fortnight (holds).

Each item lands with its gate, the routine full gate table without a lowered threshold, the full client
suite, then a fresh blind pair.
