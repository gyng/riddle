# Cut 52 — make inherited progress visible at return

Fresh current-build QA: seed 5, manual house/three sends/scout, 20-minute
absence returns one run, +16 Fighter XP, 9 total Legacy in the header, but no
Legacy gain in the report. Played seed 7 first death at Warlord D8; Damage
upgrade, forged sword and Guarded choice lead to a return at D8 rather than
another death. Depth alone misses persistent progress and the chosen safety
trade-off. These are bounded QA observations, not an independent fun score.

Expose Rust-owned Legacy earned on each real exit and accumulated report,
including each bloodline separately. Show a compact Legacy gain next to class
training on the report; only positive known gains. Old reports remain valid
and must not infer gain from current totals. Inspection cannot spend points.
No new currency, earning rule, automatic purchase or progression rebalance.

Acceptance: actual earned gain equals persistent points increase across bank,
return and death; simulated forecasts award nothing. Single and three-slot
whole/sliced/reloaded reports retain exactly the same earned totals, with no
acknowledgement double count. Client merge preserves owner totals and unknown
old fields. Actual watched report uses its exit, not a current total. Phone and
desktop fit without overflow or state mutation; current report/preparation/
class-training tests plus core fast tests, wasm packaging, TS/build/copy checks.
Show actual earned screenshots. No deployment. Independent95 remains unverified.

The fresh interactive driver's old compound Skip/mode route timed out after
52s without skipping; actual visible Speed choice worked. Preserve that evidence
and restore the helper in a later iteration-speed pass; do not call it game lag.


Implemented: finish_run records the existing earned amount once on ExitLine and
Batch, simulations remain zero; the final report sums the batch and Session
preserves/sums each slot separately. Saved counters and wire fields default to
zero for old saves; zero is omitted from JSON. Watched report copies its actual
exit; client merge sums only known amounts. Compact report rows precede
preparation, separate from class XP. No award/pricing/rules/RNG changes.

Verification: 678 core tests PASS, one existing ignored, 39.74s warm test runtime;
all-target fast Clippy clean. Historic c70ffd182d4080f1 gameplay hash retained:
strip only new numeric observation, no expected hash update. Initial strip
missed the field at the start of the object; moving it beside XP restores the
old projection. Raw failed hashes retained. New test covers bank/return/death
and simulation/duplicate completion; strengthened existing single 8h and
three-slot 2h whole/sliced/reloaded full-state/report tests reconcile gains
against actual persistent balances.

Five related client jobs PASS 30.7s: legacy-earned, report-choices, report-bosses,
report-class-xp, wall-preparation. Final compact-row/current-WASM legacy-earned
and report-class-xp PASS 43.4s under concurrent compilation. Eleven ownership/
merge/old-wire assertions at 320/400/1440 plus actual WASM earned totals:
solo38; paid three-slot101 = 38+36+27. Full engine save unchanged by details
inspection, no overflow/page errors. Initial command used two nonexistent test
names; corrected names above, raw log retained. TS/build/copy1740 zero/diff pass.

Fresh final built headed seed5 walk: manual house, first watched D4 collection
shows +5 Legacy; two further sends/scout, 20m return shows +1 Legacy, total9.
10 screenshots/text dumps, 36.1s, no console/page errors. This is bounded QA,
not the 40-minute/three-absence independent rating horizon. First rat capture
measures23px against the existing24px visual bar: record for next renderer
inspection, do not claim universal render-size acceptance. Seed7 played choices
are archived separately: first Warlord death; manual forge, Damage purchase,
sword forging, Guarded choice; second hero returns D8 with gold. Not a paired
statistical improvement claim.

Artifacts: scratchpad/gameplay-cut52/{walk,interactive,walk-ready,
earned-reports-ready,solo-parity-ready,multi-parity-ready}; raw logs
/tmp/riddle-cut52-{core-verified,clippy,layout-ready,client-verified,
build-ready,walk-ready,solo-parity-ready,multi-parity-ready}.log. Final packaged
WASM exact 8h solo/paid-three-slot five-path state/report comparisons both
terminal0: zero differences across complete saves and final reports, no
acknowledgement double count; solo4 runs/38 Legacy, multi34 runs/101 Legacy. No deployment; independent95 and broad balance/client
certification remain outstanding. The stale compound driver and minimum rat
visual size are specific next QA/iteration follow-ups.
