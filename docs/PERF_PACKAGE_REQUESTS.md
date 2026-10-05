# Package request reuse — 2026-10-05

A slow package price request is duplicated when the same panel is reopened
before it finishes. The old module-global memo projects a few package/gold/kit
fields; class, Legacy ranks, rules, loadout and hero identity can change without
changing that key. Reproduce on the real app with a controlled delayed reply:
three opens start3 requests; changing class/selected hero after completion
starts no new request (still3), reusing the prior hero's prices.

Cache the reply and in-flight promise per App, engine and immutable lineage
snapshot, plus editing rules/loadout. Do not mirror Rust simulation-input keys
in TypeScript. A refreshed lineage invalidates; a different app/engine cannot
share results. Compare current snapshot before repainting asynchronous results.
Failure must allow a later open to retry without creating a repaint/retry loop.
Package mutations while open must clear prices and remeasure the new snapshot.

Acceptance: same snapshot opened three times starts exactly1 request; changing
hero/class/Legacy, rules or loadout starts a new request. Old completion cannot
paint or overwrite a newer result. Separate App instances remain isolated.
Failed requests retry on reopening, without unhandled rejections or loops.
Check real-native UI400/1440, scoped package gates, tsc/copy and routine FULL;
shipping/public checkpoints include screenshots and exact artifact identity.
No game logic, sim count, budgets, seeds or numeric outputs change.

Evidence:scratchpad/flood-perf-20261005/cache-baseline.{mjs,json}; baseline
sameSnapshotCalls3,callsAfterHeroChange3. Flood probes are rejected separately
in PERF_PATHFIND.md; this request-level fix does not relax their acceptance.

Acceptance PASS:38 controlled browser race/retry/isolation checks at400/1440,
existing17 package-panel checks, real-native10 checks and shipping-WASM10 checks
at both widths. Real requests: three pending opens→1 call; completed reopen
reuses its prices; a real Health Legacy purchase triggers call2. Scoped client
gates6.3s, routine FULL38s (536 core/1ignored,11 tooling, tsc/copy/clippy/shipping
build; unchanged metrics/QA/18 player cases correctly reuse compiled keys).
Core source and shipping WASM are unchanged; no core simulation speedup claimed.
Screenshots:native-ui and shipping-ui under scratchpad/flood-perf-20261005.
The first native harness used mixed HMR module instances on the old shared dev
port; fresh native5594 passes, and assertions exclude inert animation copies.

This uses conservative snapshot invalidation: a new lineage object can cause a
fresh read even if the refreshed fields are irrelevant to simulation. Do not
reuse across refreshes using another client-side projection. A future Rust-owned
package query fingerprint must include candidate/price inputs as well as panel
inputs before broader reuse is safe.

Published source76a75ae, Pages37295295222 SUCCESS. Public HTML, entry JavaScript
and WASM match the CI artifact byte for byte; WASM remains3adfbebe739b0b468f94a1ab0c7a5f0ee3000cde97fd421a54af335e23e18bef.
Public real-controls checks10/10 pass at400/1440, including pending-query reuse,
completed-query reuse and actual Legacy purchase invalidation; no page errors.
Both public screenshots shown inline. Evidence:public-proof/proof.json and
public-ui/proof.json under scratchpad/flood-perf-20261005. The first artifact
poll reached its local deadline during the healthy release build; resumed
verification of the same CI run succeeded, without restarting publication.
