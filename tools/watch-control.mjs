/** Use visible game controls only; current modes/Skip live inside Speed. */
export async function pressWatchNext(page) {
  return page.evaluate(() => {
    const button = document.querySelector('main.watch .next-gem');
    if (!button || !button.isConnected || button.disabled || !button.getClientRects().length || getComputedStyle(button).visibility === 'hidden') return false;
    button.click(); return true;
  });
}

export async function pressWatchControl(page, label) {
  const ids = { 'fights':'fights', 'fights only':'fights', fast:'fast', one:'one', normal:'one', '▶▶|':'skip' };
  const id = ids[label.toLowerCase()];
  if (!id) return false;
  // Opening Speed and tapping its option share one browser operation. Separate
  // protocol round trips can let the walk-out disable the option between them.
  return page.evaluate(id => {
    const visible = element => element && element.isConnected && element.getClientRects().length && getComputedStyle(element).visibility !== 'hidden';
    const press = button => {
      if (!visible(button) || button.disabled) return false;
      button.click(); return true;
    };
    const watch = document.querySelector('main.watch');
    if (!visible(watch)) return false;
    // A caller's modal is a meaningful stop. Modeless loot keeps the HUD usable.
    if ([...document.querySelectorAll('.sheet-wrap:not(.modeless)')].some(visible)) return false;
    const inline = [...watch.querySelectorAll(`[data-tile="${id}"]`)].find(visible);
    if (inline) return press(inline);
    if (!press(watch.querySelector('.console [data-tile="speed"]'))) return false;
    const menu = document.querySelector('.sheet-wrap .watch-options');
    if (!menu) return false;
    try { return press(menu.querySelector(`[data-tile="${id}"]`)); }
    finally {
      // Skip may open keep on top. Close our own parent, never the new choice.
      menu.closest('.sheet-wrap')?.querySelector('.close-stud')?.click();
    }
  }, id);
}
