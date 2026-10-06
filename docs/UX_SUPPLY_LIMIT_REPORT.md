# Away supply-limit feedback — 2026-10-06

Automatic sustain purchases skip the same away-income budget as manual repeat,
but fail to set the existing restock_capped report flag. Correct that flag only;
preserve purchase rules, spending protections, supplies, RNG and gameplay.
Tests must isolate automatic packing with manual repeat disabled, and distinguish
budget-blocked from funded and normal non-offline packing.

Render the recorded condition as Supplies limited with the reported income
budget and a short tooltip. Keep the ledger link. Show a compact main-report
notice only for a reported limit with known zero income; other occurrences stay
under Details. This describes the recorded absence, not today's selected hero
or a promise that restocking is still paused. Unknown old budget stays unknown.

Acceptance: native full tests and clippy; three earned town Steady/Bold8h cases
have identical reports and raw final saves after removing only restock_capped;
new actual seed3 zero-income warning; native/WASM full parity; phone/desktop
notice/tooltip/ledger/old-wire/positive-income checks; build/copy/screenshots.
No balance change or deployment. Broad gates not a claim of this reporting fix.

Verified: tools/verify.sh --quick passes (562 native tests, one ignored;
77 seconds including client sidechecks); workspace/all-target clippy clean.
Rebuilt real WASM: 5,250,773 bytes, SHA256
85366c7ba43d58661fe88403d6bdbe364261d6e0592c5f1a50bd809f1ada928c.
Production web build passes, with the existing large-bundle advisory.
Four targeted report suites pass: supply-limit, hero-upgrade, run-training,
bosses. New checks cover 320/400/1440, tooltip, ledger input, zero/positive/
unknown budget, false flag, touch bounds and overflow.

All six immutable earned seed1/3/5 Steady/Bold eight-hour native reports and
final saves equal the earlier audit after removing only restock_capped.
All six full corrected native reports/raw saves exactly match real WASM;
seed3 Bold repeated at desktop1440. Actual report opening/tooltip screenshots
at400/1440 are in scratchpad/supply-limit-20261006/, alongside native-results,
wasm-results and the comparison/capture scripts. No purchase, RNG, gold or
balance change. Screenshots shown to owner. No deployment.

Legacy suite limits: ui aborts at its initial hidden compact rule-editor click,
before any report (unchanged camp/editor path). cut21 passes its updated report
budget/link checks, then fails its shelf check: its caught click on the hidden
loadout tile yields no visible panel. These are not reported as green. Do not
force hidden controls or weaken the assertions; migrate the established-town
fixtures/current hero-menu paths as a separate iteration task. Exploratory
fixture changes were removed; only the required report wording assertions stay.
