# Active bloodlines and hero roster — owner follow-up 2026-10-05

A hero slot is a persistent bloodline. Its Legacy wallet and health/damage/armour
ranks survive the current heir's death; Chronicle keeps the individual's history.
Old per-heir Legacy migrates once into the bloodline without losing earned points.
This supersedes the current-hero-only default in UX_LEGACY_UPGRADES.md.

Introduce multiple independently active bloodlines, initially one; new bloodlines
are explicit purchases (250 gold, cap3), require the first home and never auto
appear. Each has its own hero, rules, class XP, Legacy, inventory and live run.
Town construction, worker hiring and gold/Savings accounting are shared. Selection
never restarts a run. Rust owns selection, wallet transfers, clocks, roster state,
purchase refusals and save migration. Simulation forecasts remain single-bloodline
and do not clone/simulate unrelated live heroes.

Desktop defaults to an active-hero roster on the left: portrait, bloodline/name,
class/XP, current action, notification, Details. Row focuses the hero; Details
opens the hero menu. Past heroes stay in Chronicle. Class controls move into the
hero menu; the duplicate class bar goes. Rules remain explicitly accessible.
Mobile uses a compact active row and a Heroes sheet for the roster, maintaining
44px targets and the visible map. Same actual state, no fabricated heroes/actions.

Gates: Legacy costs/refusals/stat effects/migration/successor retention; independent
rules/XP/Legacy/live runs across selections and reloads; simultaneous progress and
uncapped absence; shared wallet conservation, failed purchases leave state unchanged,
no repeated shared bank interest; deterministic native/shipping replies. Baseline
single-bloodline regression bars unchanged. Real UI400/1440: roster focus/details,
class access, selection without restarting either run, correct balances and no
horizontal overflow. Shipping/public screenshots and exact deployed artifact proof.

Local acceptance:536 core tests/1 ignored,14 dayplayer recovery tests,11 tooling
tests; tsc/copy/clippy and shipping WASM. Final routine FULL561s: metrics111.4s,
QA46.5s, all18 fortnight cases PASS. No numeric/timeout reductions; broad
historical/system-removal audit not run. Native/shipping58 complete replies
match exactly, including refusals, two live heroes, selection/reload and save
migrations. Real native and shipping UI:29 bloodline checks plus65 first-home,
Legacy/watch/report/forge checks at400/1440. Evidence is private under
scratchpad/legacy-upgrades-20261005. Publishing evidence follows below.
Scoped client gates:fights46 checks153.8s; clarity:paint5 checks58.3s. The first
headless fight attempts hit Chromium resource/crash errors with the workstation
nearly out of disk space. Removing1.73GiB of inactive example/probe incremental
caches older than7days restored the unchanged test; current core/WASM caches
were retained. No timing/seed/assertion reductions.
