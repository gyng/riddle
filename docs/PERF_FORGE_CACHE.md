# Forge preview reuse — 2026-10-06

Scope preview requests and completed answers to the App and engine that own
simulation state. A conservative key includes the complete lineage wire view,
full rules and selected loadout: hero/class, traits, party, Legacy upgrades,
route, owned kit, prices and affordability must invalidate answers. Keep at
most four completed answers per App; share identical pending requests. Reject
stale cache writes and UI paints after state/engine changes. An open Forge
clears its old estimate on rule/lineage changes and updates kit/price controls;
never automatically launch simulations. Closed sheets release subscriptions.

Acceptance: controlled lifecycle checks at400/1440 prove same-state one-call
reuse, pending deduplication, isolation across Apps and engines, invalidation
for each listed dependency, stale success/failure rejection, failure retry,
old-response ordering and bounded completed entries. Real shipping controls
prove repeated explicit Forecast does not call Rust twice and buying clears
old estimates without starting a simulation. No Rust/gameplay changes.
Run tsc/copy lint, scoped Forge/lifecycle/geometry gates and web production
build. Capture desktop/mobile checkpoints. Push authorized; no deployment.


Acceptance: real shipping400/1440 controls pass. Three explicit Forecast taps
including close/reopen issue one Rust query; purchasing a weapon step clears
old results with zero new query; the next explicit tap issues query two.
Screenshots and actual calls/query timings are in
scratchpad/forge-cache-20261006/. This improves reuse/ownership; no claim that
a cold simulation itself got faster. The key deliberately includes unrelated
wire changes too, preferring extra work over an incorrect forecast.

Tsc, copy lint and production web build pass. Existing Forge gates33 and20
checks, geometry176, forecast lifecycle32 and Forge cache lifecycle pass.
The broad historical Cut25 gate fails its watched-run tile assertion and then
aborts at retired Forge kit-next controls; that gate still assumes the previous
report/armed-purchase UI. These failures are disclosed rather than treated as
green. The changed nested-sheet cleanup is covered directly in the cache gate.
No Rust edits or rebuilt simulation required; no balance-suite speedup claim.
