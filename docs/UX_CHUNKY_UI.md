# Chunky game UI — 2026-10-05

Owner: center parchment; add tactic icons; replace flat web controls with chunky
SC2/WC3-inspired construction while keeping the current painted art style.
Audit: command tiles already match; Tactics, Hero, Forge, Workers and Settings
use flat chips/rows and inconsistent condensed text. Report and death retain
unique scroll/banner silhouettes; combat log remains plain text over the map.

Extend skin.ts + packed nine-slice art with shared chrome.css material/type
roles and openWindow in sheet.ts. Main windows center between the resource bar
and command bar, both widths. Rule-token pickers remain anchored, live loot
modeless. Iron controls, inset item/icon sockets and gauges share one palette;
Forge uses bronze, hero/tactics gilt, workers blue, danger ruby. No new mechanics.
Grenze remains display; self-host standard-width Fira Sans for readable body,
Fira Condensed for compact numerical UI. All command labels same 14px, controls
same 14px, titles 24px; phone content can wrap. Icons reuse packed styles,
personalities and corresponding action families with primitive fallback.

Targets: art/ui/targets/*.png, generated via built-in imagegen from current
screens. These guide components; never ship baked screenshot text as controls.
Acceptance: center offset≤2px; no horizontal overflow 320/400/768/1440; primary
menus/fonts consistent, icons present, controls≥44px; expansion/resize, keyboard,
paid confirmations, close/back/Escape and popover placement preserved. Audit all
screens with native/shipping/public captures. Run tsc/copy/build and affected
client gates; Rust/simulation unchanged, cached numeric gate table remains valid.

Validation so far: tsc + shipping web build and copy-lint pass (1469 literals).
Affected package requests (46), Cut30 (41), Cut30town and Cut28 (27) pass.
Real shipping-engine controls: 36 checks across 400/1440px pass. Real shipping
walk passes house build, live watch, report and uncapped offline return.
Metrics, QA and selected dayplayer full tables revalidate from unchanged binary
caches; no simulation/performance change claimed. Private captures and logs:
scratchpad/chunky-ui-20261005 (never stage).

Layout: 176 checks pass over seven management windows at four widths.
Gold also uses the shared window. Tooltip interaction fix: a normal control
activation closes its preceding plate while retaining its action; long-press
continues to suppress activation. Tooltip density/copy limits are unchanged.

Tooltip interaction/density/copy suite: 70/70 pass on 360/400/1440px.

Anchored sheets and nested Back/Escape: 12 direct checks pass at400/1440.
Historical Cut23 harness deviation: its initial Forge phase fails because it
selects .kit-next and expects automatic forecast lines/two-tap buys. Those
controls were removed before this change (prior-source forge.ts has no
.kit-next); current explicit/direct Forge behavior is covered by Cut30town.
No historical assertion was weakened; private direct picker checks cover the
shared-sheet regression concern independently. Update this historical harness
under a separate migration contract rather than treating its old Forge phase
as the current implementation contract.

Source81873c2 pushed. CI build37322014020 passed; queued Pages deployment
cancelled at the owner's instruction. No public acceptance claimed. Publish
only when explicitly requested. Local/shipping screenshots remain in the
private milestone directory; target art is checked in.

Historical Cut23 deviation resolved by QA_CURRENT_UI.md: full33/33 PASS27.0s,
including anchored picks and the actual collapsed report link.
