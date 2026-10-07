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
