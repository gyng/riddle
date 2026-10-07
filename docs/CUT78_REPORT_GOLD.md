# Cut78 — show earned gold including loot sales

Contract2026-10-07. Actual frozen40a19a8 seed12 manual opening: first report
Gold home87/header106. Public Gold sheet confirms +87collected/+19salvage;
Details confirms stored sword and a separate sold sword, no accounting defect.
Evidence scratchpad/qa-gold/walk/07-report-details.txt and -ledger.txt.

The simple report shows Gold earned: run coins plus loot sales, before spending,
excluding next-heir grants and existing balance. Same core wire values; no
balance or save mutation. A native button opens the existing gold ledger;
hover/focus/long-press explains Run gold and Loot sold without extra chrome.
Old reports use exit sums plus reported sales; missing fields contribute zero.

Gates: 87+19=106 despite a different current balance; wake and spend do not
alter gross earned gold; zero and old-wire cases stay exact. Pointer/keyboard
ledger access works at320/400/1440 without changing save, horizontal overflow
or extra summary tiles. Existing report and copy/type/build checks pass.
Real earned first-report screenshot confirms106 and readable chunky frame.
Not a score or full rating horizon. No deployment.

Verification: earned-gold/supply tests PASS at320/400/1440 (16.2s); original
clarity core26checks PASS16.4s. Report hero19/action16checks PASS, zero changes
to Rust/wire. QA92report8checks PASS6.4s after migrating its prior Collected
gold expectation to Cut77's Recent full hauls, preserving13runs/floor6 and
separate unrelated rows. First test fixture failed because invented old-wire
exits lacked required text; corrected to complete exit records before passing.
TypeScript/copy1746zero/build/diff PASS; existing build chunk warning. Public
earned house→run→scout→8h walk PASS35.3s/10dumps/0errors; catch-up1.9s. Report
106 confirmed against actual87+19ledger, no grants. Screenshot taken during
header coin tween showed103 temporarily; settled capture PASS37.0s/0errors with both header and tile106.
Artifact scratchpad/qa-cut78-gold/settled/07-report.png shown to owner.
No full-suite certification or independent95score.
