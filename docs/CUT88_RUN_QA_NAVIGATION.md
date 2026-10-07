# Cut 88 — observe run completion controls without stale locator retries

RunsUI failed twice when next-gem existed at discovery but vanished before
Playwright's animated locator click. The test then waited30s for a button on a
screen already left. Use the same visible-node snapshot dispatch as the Speed
controls. Check connected, enabled and visible immediately before DOM click;
return false if the run changes screen. The existing loop then observes the
actual new screen. No App/engine shortcut, forced click or gameplay change.

Gate: full RunsUI retains the watched/replay exact event hash, public completion,
manual/scout, history and earned bloodline checks. Existing watch controls pass.
