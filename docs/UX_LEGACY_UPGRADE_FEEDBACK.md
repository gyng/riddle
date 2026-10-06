# Legacy upgrade feedback — 2026-10-06

The real upgrade-sheet checkpoint compresses rank and incremental benefit into
one unlabeled line and repeats the same accessible action for all upgrades.
Give each buy action its own upgrade name, retain visible core price/currency,
and label Rank and Next separately. Completed ranks must not advertise a next
benefit. Preserve existing packed art and chunky rows, without new game math.

Source audit also finds the open sheet only refreshes when hero identity
changes. Refresh on core points/offers, away status, current class progression
and current heir history as well, while preserving expanded Details. A change
of run turn alone must not rebuild the sheet. No new estimates or TS truth.

Acceptance320/400/1440: unique health/damage/armour actions; exact core prices,
rank/effect; points/affordability/away updates without changing hero identity;
one core buy per click, no writes on open; capped rows no Next; stale closed
sheet no updates; touch targets≥44px, no overflow/overlap; existing hero/report/
appearance tests and actual WASM capture. Build/copy, no deployment.

Verified: new hero-upgrade-feedback at320/400/1440 checks exact Rank/Next,
unique names and core price descriptions, one successful core call, preserved
Details, away/points/prices/cap updates without identity change, no re-render
on turn-only changes, refusal retry, closed-sheet unsubscribe and44px touch
bounds. Existing report/appearance/slot-look/name checks:5/5 suites pass7.9s.
Build/typecheck/copy1560/diff clean. Actual WASM400/1440: opening sheet leaves
save byte-identical; health purchase88→85Legacy and rank0→1/price3→6 with gold
unchanged; displayed85 and exact next benefit; no page errors/overflow. Replay
clock disabled only for this UI checkpoint (`runs=0`). Screenshots shown;
scratchpad/legacy-choice-20261006/feedback-results.json and width images.
No core/new art/balance/deploy.
