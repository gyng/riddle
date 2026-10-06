# Persistent camp controls — 2026-10-06

Earned ascension capture exposed duplicate top-bar class offers and a misleading
worker shortcut. Audit: hired workers are not gated or lost; `nextPill` always
opens Works, while the core's fallback `tree.next` can name an unrelated system
(`automations · meet Lich`). Preserve Rust truth and core carry/reset behavior.

Remove top-bar class offers for all camps; preserve class XP/offers in the wire
and Change class through selected Hero Details. Keep actual worker/hire/send/
chest goals. If the Workers shortcut's fallback names an unrelated system or
has no goal, show Workers and the real hired count instead. No fake unlock,
new forecast, automatic build/purchase or core-system change.

Checks:320/400/1440 camp with class offers and closed automation curriculum;
no class chips in topbar, Details→Change class available, unknown system goal
becomes Workers/count with no progress bar, actual hire/send/chest unchanged.
Test repaint after changed hired count, no-worker/early fallback, spacing and
read-only complete save. Capture actual earned post-ascension phone/desktop
Workers and Hero Details; build/typecheck/copy/frame/QA and relevant class check.

Implemented: Workers fallback uses actual core `system`/`none` kinds, real hired
count and no invented progress. Actual worker node, send, chest and early
no-worker goals retained. Click uses displayed node focus; management has none.
All class offer chips removed from strip, orphan CSS removed, Hero Details
class choices and console portrait access retained. Wire/gameplay unchanged.

Verification:15 focused checks each320/400/1440 PASS; updated owner-amended
Cut16 class placement retains authoritative offers/signature gates and selected
class through the send,17checks PASS (2jobs6.8s). Cut28 27checks, frame63 and
QA41 PASS. Initial6job suite failed missing not-yet-written focused file and
obsolete Cut16 topbar-chip expectations; corrected. Cut305 remaining single
failure: day0 expects crate/works among ≤5 well/console surfaces, actual four
mouth/tent/Heroes/send. Other Cut305 checks passed in107.8s. Isolated previous
HEAD camp/works source reproduces the identical day0 failure; no gate weakened.
Track this separately against first-home/hero-roster amendments and visibility.

Actual earned Hunted three-bloodline WASM town400/1440: Workers11hired, no class
bar, hired worker panel and inherited class ladder accessible; full engine-save
bytes unchanged after both opened. No errors/overflow, headed captures viewed/
shown in scratchpad/persistent-camp-20261006/. No Rust/WASM/balance/deployment
changes or full-suite certification. Next impact queue: first-home day0 surface
audit from this baseline failure, then content variety and optional-reset design.
