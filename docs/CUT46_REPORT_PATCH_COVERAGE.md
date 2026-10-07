# Cut46 — report suggestions behind Details

Baseline patch-overflow terminal1/11.9s. Diagnostic browser trace proves the
failure is the final report replacement suggestion, folded under Details. All
preceding death insert/drop/cancel/count/order/card/held-row checks pass.

Keep the simplified report. Open its actual visible Details button before
selecting the replacement suggestion. Preserve every original assertion,
especially full-set capacity, exact chosen drop, cancelled unchanged rules,
measured insertion index, fired counts, protected card rows, held-row no insert
and final replacement preserving unrelated rules. No production change.

Actual report suggestion inspection must leave a real earned WASM save unchanged
at400/1440. Use an explicit diagnostic report suggestion fixture for this UI
path; it is not progression or a fun score. Verify visible Details → suggestion
and exact replacement persistence separately. Show screenshots at checkpoints.

Acceptance:

- Original patch-overflow25 checks pass unchanged except one actual Details
  click. Neighboring cut20 (16 each400/1440) and report-choices (23 eachwidth)
  pass3/3 in11.7s, terminal0, /tmp/riddle-cut46-client.log.
- Real earned Gunner WASM400/1440: add one legitimate player rule via existing
  insert API, then diagnostic report replacement targets that rule. Opening
  Details leaves full engine save byte-exact. Replacement retains all other
  rules, including generated drills, and exact17-rule semantic set persists
  after flush/reload. Zero page errors. JSON and shown screenshots under
  scratchpad/patch-cut46-20261007; /tmp/riddle-cut46-real-player-report.log0.
- First attempted diagnostic fixture targeted a generated drill after class
  selection; reload restored the drill, so its exact replacement assertion
  failed. Raw report-first.mjs and /tmp/riddle-cut46-real-report.log retained.
  This is not a passing replacement test or earned run report. Corrected
  fixture targets a real player-written rule and retains every generated row;
  no comparison allowlist or tolerance. Real test is not the four-row capacity
  gate; that remains the original25-check fake-engine test.
- JS syntax/diff pass. No production/Rust/WASM inputs changed. Owned5399 closed;
  shared5219 kept. No deployment, full-client/balance or independent95 claim.
