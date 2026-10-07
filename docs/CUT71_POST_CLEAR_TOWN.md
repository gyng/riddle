# Cut 71 — return to Town after a clear

Earned harder-descent app QA reproduces a blocked player route: `camp` always
redirects to victory for an ended bloodline, and victory offers no Town return.
Add a visible Town button that explicitly permits reviewing a cleared town.
The clear, wallet, Legacy, gear, classes, workers and other heroes stay intact.
Normal end-of-run navigation still lands on victory; reopening Town must not
implicitly begin any descent or send a completed hero.

The completed town's main gem says Next descent and returns to the existing
victory/descent review. The mouth and any watch route likewise cannot send an
ended bloodline. Upgrades/respec and class/tactic/forge choices remain ordinary
legal home actions; no allocations or purchases on navigation. Confirmed
Begin descent remains the only numbered transition and retains its refusal,
cancel, stale-response and single-flight behavior.

Verify real earned completed save at320/400/1440: visible Town, keyboard entry,
exact raw save unchanged on Town/hero/Next descent navigation, no send or
beginDescent calls, whole ordinal and no overflow/errors. Verify live guard,
normal automatic victory redirect, and an actual paid home Legacy change at
least once without starting a run. Existing numbered-descents and historical
ascension refusal coverage must pass. Show screenshots from the real app;
rebuild web only (no Rust runtime change). No deployment or95-score claim.

## Verification

Final five-job suite passes15.4s: real post-clear Town (320/400/1440), existing
numbered-descents, ascension-refusal (four viewports,39checks each), legacy-tree
(12nodes/three viewports), and real first-plot manual build/dedup/geometry.
Raw /tmp/riddle-cut71-final-client.log. Initial3/4 result has a new-test wire
expectation error: absent optional live is undefined rather than null. Accept
both absence encodings and additionally inspect the complete core saved run;
post-clear solo then final complete suite pass. No production behavior was
changed to satisfy that assertion. Original raw failure log retained.

Fresh built-WASM headed400/1440 imports an earned first-ascension clear and
uses the actual Town button, then actual Legacy button. All navigation leaves
the entire save byte-exact; no overflow/page errors. Desktop/phone captures
scratchpad/qa-cut70/{ending,town,legacy}-*.png, raw
/tmp/riddle-cut71-ui-final.log. First capture omitted app run-counter metadata
and displayed0; corrected wrapper uses the native completed totals359. Native
save itself is unchanged. Corrected phone victory screenshot shown. This is
scripted diagnostic evidence, not an independent rating or synthetic clear.

Checked-in fixture web/tests/fixtures/earned-first-ascension-clear.json is the
unaltered ranger5-mending-venom-brace-after result from Cut70's actual8h/14-run
Ascension1 campaign, based on the Cut64 publicly earned Ranger clear. SHA256
5e1787aaefd4d1172a188252b2b19d1cbf95918f85992ad31c6ad07c5cd35136.
Native file includes359 lifetime completed runs (108banked/237returned/14dead),
15th heir and the paid216-point tree; no game-value grants or trimmed history.
Real browser test refunds all216 spent points, pays3 for Health, verifies
ended=true/core run=null, no send/begin calls, durable reload and Town access.
Other widths retain the original allocation and exact navigation save.

Web build/TypeScript passes, copy-lint1739 zero, JS syntax/diff clean. Existing
chunk-size advisory remains. Root-directory build invocation failed before
correct web-directory build; raw logs retained. No Rust library change, assets,
full-suite/18-case/exhaustive certification, deployment or independent95 claim.
Owned preview5433 stopped after QA; shared5219 retained.

Multihero follow-up passes12.3s (/tmp/riddle-cut71-multi-final.log): on the real
1440px earned clear, buy a second bloodline using owned gold, select/send it
and advance250ms, then select the cleared first bloodline. Actual Town and
Legacy controls preserve the complete two-slot save byte-exact; roster renders
two heroes with second state=live, first remains ended, no extra send/begin
calls. This verifies snapshot preservation during review, not timed catch-up
parity. Core multihero scheduling is unchanged.
