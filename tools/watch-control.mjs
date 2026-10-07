/** Use visible game controls only; current modes/Skip live inside Speed. */
export async function pressWatchControl(page, label) {
  const ids = { 'fights':'fights', 'fights only':'fights', fast:'fast', one:'one', normal:'one', '▶▶|':'skip' };
  const id = ids[label.toLowerCase()];
  if (!id || !await page.locator('main.watch:visible').count()) return false;
  const press = control => control.evaluate(button => {
    if (button.disabled || !button.getClientRects().length || getComputedStyle(button).visibility === 'hidden') return false;
    button.click(); return true;
  });
  // A caller's existing dialog is a meaningful stop; never click through it.
  if (await page.locator('.sheet-wrap:visible').count()) return false;
  const inline = page.locator(`main.watch [data-tile="${id}"]:visible`);
  if (await inline.count()) return press(inline.first());
  const speed = page.locator('main.watch .console [data-tile="speed"]:visible');
  if (!await speed.count() || !await press(speed.first())) return false;
  const options = page.locator('.sheet-wrap .watch-options');
  if (!await options.count()) return false; // run changed screens while the menu opened
  const menu = await options.first().elementHandle();
  try {
    const control = options.locator(`[data-tile="${id}"]`);
    return await control.count() ? await press(control.first()) : false;
  } finally {
    // Skip may open keep on top of this menu. Close our own parent, never Escape
    // the new choice or the caller's dialog. Detached menus are already closed.
    await menu.evaluate(body => body.closest('.sheet-wrap')?.querySelector('.close-stud')?.click());
    await menu.dispose();
  }
}
