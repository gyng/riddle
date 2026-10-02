// Cut 17: the camp's secondary objects (loadout · unlocks · vault · party; the forecast behind the shaft) are panels over the
// well, each opened by its console tile; a closed panel's content stays in the DOM unrendered (its text reads, its buttons wait).
// `openPanel(page, name)` opens one (idempotent); `{ all: true }` also opens the unlock panel's whole catalogue (`more`).
export async function openPanel(page, name, { all = false } = {}) {
  // a lineage that has not carved the unlock tile yet (no mark: docs/UI.md §5) still paints its shelf into the closed panel; `all`
  // then sets the viewer's `more` directly (what the stud would remember) and repaints
  if (name === "unlocks" && all && !(await page.locator(".cmd .tile[data-tile=unlocks]").count())) {
    await page.evaluate(() => { localStorage.setItem("riddle.unlocks.all", "1"); window.__riddle.go({ kind: "camp" }); });
    await page.waitForTimeout(300);
    return;
  }
  if (!(await page.locator(`.panel[data-panel=${name}]`).count())) {
    if (name === "forecast") await page.locator(".shaft").click({ timeout: 5000 });
    else await page.locator(`.cmd .tile[data-tile=${name}]`).click({ timeout: 5000 });
    await page.waitForTimeout(150);
  }
  if (all && (await page.locator(".panel .unlocks button.more").count())) { await page.locator(".panel .unlocks button.more").click({ timeout: 5000 }); await page.waitForTimeout(120); }
}
/** Cut 17: the camp's tablets are compact (one tap each) until editing is on — the `edit` tile, a viewer preference that persists
 *  (localStorage `riddle.editing`). `editRows(page)` turns it on (idempotent) so the rows carry their chips, ▲▼, × and `+`. */
export async function editRows(page) {
  if (await page.locator(".editor.compact").count()) {
    const t = page.locator(".cmd .tile[data-tile=edit]");
    if (await t.count()) await t.click({ timeout: 5000 }); else await page.locator(".editor .row.compact").first().click({ timeout: 5000 });
    await page.waitForTimeout(150);
  }
  await page.evaluate(() => localStorage.setItem("riddle.editing", "1"));
}

/** Death screen v2: the trace, the ledger, the margin and every tablet with its numbers fold under one `details` tap. Suites that
 *  read or tap those open it on every death screen this page mounts (an init script: call it before the first `goto`). The rest
 *  view itself (killer · floor · hp, the why line, the lit fix) is ui.mjs's. */
export async function deathDetails(page) {
  await page.addInitScript(() => {
    const open = () => { for (const b of document.querySelectorAll('.death .death-more[aria-expanded="false"]:not([data-auto])')) { b.dataset.auto = "1"; b.click(); } };
    new MutationObserver(open).observe(document, { childList: true, subtree: true });
  });
}
