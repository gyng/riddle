# First home and clearer run resources — 2026-10-05

Owner request: start with an empty town; build a house, a hero moves in, then
runs become available. Add per-hero Legacy points separate from ordinary and
class XP. Simplify run resources, gold flow and the forge. This supersedes the
old first-load Send contract; it is not a new cohort/Cut31.

New saves: zero buildings/resident/sends. One free Build house action; house
construction and run eligibility belong to Rust, never just client hiding.
The resident arrival is a short cosmetic scene, with reduced-motion support.
Old saves missing the home flag retain a resident; no reset or blocked old run.
Existing simulation/gameplay harnesses explicitly construct the free home before
playing, without granting currency, workers, XP, rules or changing simulation time.

Legacy: default to a separate per-heir lifetime record,
earned as 1 point per completed expedition plus newly reached personal depth floors, archived
after death. No spending or combat/stat effect until explicitly selected.
No policy learns implicitly; class XP/marks remain distinct. Award once per
completed run; zero points for blocked sends or constructing the free house.

Run UI: persistent clearly labelled town resources; carried gold distinct from
secured gold. Remove the bank/keep/death/rule formula line; keep outcome detail
available on demand. Coin motion only for actual Rust-owned transfers, never
invent currency or count the same secured gold twice; reduce motion accessibly.
New towns collect gold automatically when the run ends; the free porter becomes
available after three positive returned hauls. Old saves retain manual chest
collection and its existing porter gate. Savings remain distinct.

Forge: next weapon/armour/pack upgrade with item, price and one buy action;
future ladders/salvage/forecast detail fold away. No expensive forecast required
to open or buy. Preserve engine prices, affordability and purchase accounting.

Gates: fresh home/save/reload/refusal/old-save migration; per-hero awards/archive
independent of XP and stable across save/reload/offline/watch; gold conservation
and no duplicate credit. Real UI checks at400/1440: empty opening, free house,
arrival→send, clear resources, forge can buy immediately without a forecast.
Full engine/harness tests, tsc/copy/clippy, shipping/native parity and selected
routine regression gates. Record design-induced gate deviations honestly;
do not weaken balance bars. Fresh shipping/public screenshots at checkpoints.

Accepted locally: 528 engine tests (1 ignored), 14 recovery harness tests,
10 tooling checks, tsc/copy/clippy and shipping/web build. Routine full gates
pass in336s (18 current-player fortnight cases; no broad historical/system
removal audit). Final cached verification39s. Real-engine UX51 checks at400/1440;
25 complete native/shipping bridge replies match exactly, plus refused sends.
Scoped clients: fights46 checks155.2s, clarity:paint5 checks27.3s; numeric/timing
bars unchanged. First-time fixtures construct the free home explicitly.
Shipping GPU walk9 screens54.7s, no page errors. Private evidence is under
scratchpad/first-home-20261005. Published source c515281 (Pages37223080074): public UX51 checks at400/1440,
settled screenshots,9-screen GPU walk32.5s/no page errors, exact deployed/public
WASM hash match. Reduced-motion arrival check845.3ms and zero coin particles.
Publication details tracked in HANDOFF.md.
