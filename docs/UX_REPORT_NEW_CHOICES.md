# Earned tactics on return — 2026-10-06

Fresh seed3: house built by UI; three manual sends and porter/scout hiring
through the engine; four8h absences with unchanged Steady and no upgrades.
The first Warlord defeat is in the32h return, deepestD13. Core announces
`+Bold`, `+boss focus`, `+kite archers`; none appears outside folded details.
Expose those earned choices with icons, separated from class XP and Legacy.
Click opens the matching combat style/tactic section; never auto-equips.

Acceptance: only explicit recognised `+name` report beats, deduplicated per
reported bloodline. Preserve report ownership if another hero is selected.
Old single reports can act only when there is one active slot; ambiguous old
multihero reports display earned names without selecting a guessed owner.
Known empty per-slot arrays suppress duplicate top-level beats. No current
levels inferred. Details stay folded. Check400/1440 and inspect headed real
reward screenshots; relevant report/package tests and build/typecheck/copy pass.
No simulation, save, balance, or deployment changes.

Verified: compact `New choices` section before Legacy action and folded details;
existing icon module and chunky buttons. Report beats determine names, kinds
come from the catalogue; current ownership cannot invent a reward or level.
Recorded slot names/IDs stay fixed. An unowned old report with multiple active
slots is display-only; removed owners and superseded selection cannot open
another hero's choices. Slot topology is memoized so routine live updates do
not replace buttons under the pointer. New panel entry opens the relevant
alternatives and scrolls that section into view; no automatic comparison.

Final ownership46 checks pass2.8s; preceding combined44+classXP40+selection28+
report-actions32 pass4.5s. The final change was one additional test fixture;
production code already passed the combined run. Typecheck/build, copy1532/
zero violations and diff check pass; existing bundle advisory.

Headed400/1440 real report loaded the earned save: all three choices visible,
Boss Focus card opens its alternatives, equipped list unchanged on opening;
manual equip succeeds and survives save/load. WASM hash
ae07e9503204d65d236072dbe0183b8fd8608d0376dfd70f6411fc41528047c6 verified,
no page errors or overflow. Screenshots inspected and shown.
Diagnostic progression uses engine sends/hiring/absence acceleration, not a
human32h playtest or balance/performance audit. Empty initial chest and absent
empty tactics field handled in the QA harness, not production workarounds.
Evidence: scratchpad/first-boss-rewards-20261006/.
Next gameplay evidence: compare a newly earned tactic against the same saved
town without it for the next absence, inspect decisions and rewards before
claiming value. Avoid further report chrome changes without new evidence.
