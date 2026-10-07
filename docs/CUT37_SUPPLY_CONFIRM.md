# Cut 37 — supply refund confirmation on the current control

Current qaAC packed-shelf failure is reproducible21.8s: the fixture targets a
retired hidden command tile. Probe confirms its badge has a0×0rect, no sheet
covering it; do not restore that tile or force-click invisible UI.

The actual Run setup → repeat pack → off control calls setOrders, which calls
Rust set_restock(false). Unlike the retired badge, this current control has no
refund confirmation. Preserve that safeguard on the current player path.

Acceptance:

- If disabling repeat unpacks bought supplies, first tap names the refund count
  and leaves the complete engine save unchanged. Second tap applies once.
- Count exactly the supplies Rust refunds; exclude free/found supplies. Switching
  on, already-off or an empty shelf needs no redundant confirmation.
- Repeat choice, refunded gold and retained free/found inventory persist through
  actual engine save/reload; app renders Rust truth, no TS economy calculations.
- Current qaAC uses the visible Run setup control and retains first-tap refusal,
  second-tap single mutation and refund-count checks. No forced hidden click.
- Verify mobile/desktop layout and show real-WASM screenshots. No deployment.

Evidence: /tmp/riddle-qaAC-current-repeat.log and /tmp/riddle-restock-probe.log;
scratchpad/restock-audit-20261007/qaAC-probe.mjs. Implementation pending; no
acceptance or95fun claim from writing this contract.

## Implemented current path

Run setup Auto restock off uses shared twoTap when the shelf contains paid
nonfound supplies. First tap says refundN?; selected off/empty shelf is a direct
no-op/single mutation respectively. Shared refundable-supply count also feeds
the old badge, but no hidden tile was restored or force-clicked. Label changes:
standing orders→Run setup, repeat pack→Auto restock.

Real QA exposed the anchored desktop window crossing the console (body bottom
780.40, console764). Run setup now uses the existing centered game-window layout.
Final body bottoms400:626.5 vs console762;1440:585.33 vs764, no overflow/pageerrors.
Screenshots shown, under scratchpad/supply-confirm-real-20261007.

Real earned Gunner save has two paid heals26gold each plus a free leash. Both
widths: first tap byte-exact engine.save; second tap gold48536→48588, only free
leash100631 remains, flush→actual page reload preserves exact gold/order/supply
IDs. No grants or mock engine for this acceptance. QA fixture additionally
includes one free/one found supply and still names only2paid refunds.

Final qaAC18/18PASS12.1s (/tmp/riddle-supply-copy-final-gate.log), retaining the
other15checks and adding already-off refusal/empty-shelf one-tap coverage.
Pre-copy current trio3/3PASS45.4s (qaAC18, hero-grounding2widths, watch-console5
widths). Real final wording/save evidence /tmp/riddle-supply-confirm-copy-real-final.log.
Core refund semantics unchanged. TS/copy/build final result recorded in handoff.

Additional actual earned found-supply proof: seed5-short D20 camp, paidheal/fire
26each plus foundheal5013/freeleash100136. First tap unchanged; confirm3561→3613,
found/free IDs unchanged after flush and page reload. No fixture grants.
/tmp/riddle-supply-found-real-final.log and scratchpad/supply-confirm-found-20261007.
Final TS/copy1733zero/webbuild/diff PASS. No native refund changes or deployment.
