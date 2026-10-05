# Tactics — 2026-10-05

Owner: fix “package” wording and simplify tactics. Supersedes the automatic
price-query and always-expanded catalogue presentation in Cut30§2.

Player-facing title/town control: Tactics. Slot labels: Combat style, Extra
tactics, Personality. The underlying package/rule/save identifiers stay stable.
Lead with equipped choices and level progress. A Change control opens each
kind's available choices; hide unavailable categories and locked catalogue
cards from the main view. Keep direct free equipping and paid levels.
Boss counters/scars and compiled rules fold under Details. Keyword tooltips
describe hero behavior without the “package” jargon.

Outcome comparisons are optional: Compare outcomes starts the same24-sim
query, preserves uncertainty bands and prices, and shows pending status. Opening
or changing a selection never requires that query. Close/reopen without comparison
must issue zero price queries. Existing snapshot, failure and late-reply guards
remain. Rust-owned whole-query reuse applies when comparisons are requested.

Acceptance: real native/shipping/public controls at400/1440; no horizontal
overflow; zero initial query; all choices and level buys reachable; correct
equip/unequip/slot selection; Details exposes counters and rules; comparison
explicit; stale-reply/hero/Legacy guards pass. Numeric sim/bot gates unchanged.

Local acceptance:36 real native and36 real shipping control checks400/1440:
first house click/Enter, no opening queries, folded alternatives, visible
purposes, free style equip, two selected tactic slots/removal, mobile Details/
rules, no horizontal overflow, explicit24-sim comparison, reopening without
background comparison, Legacy invalidation and confirmed level purchase.
Scoped client: cut30 all41; cut30town all; cut28 all27; request46/lane16.
Copy-lint1468 literals/51 files:0 violations; final client shipping build PASS.
Private evidence:scratchpad/package-refresh-20261005/{native-ux,shipping-ux}.
Automatic review initially rejected publication; no command ran. Owner then
explicitly approved pushing. Public acceptance remains pending.
