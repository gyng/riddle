import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';
const url = execFileSync('bash', ['tools/dev.sh'], { cwd: new URL('../../', import.meta.url), encoding: 'utf8' }).trim();
const browser = await launchBrowser();
try {
  const page = await browser.newPage({ viewport: { width: 320, height: 900 } });
  await page.route('**/ui/icons/it_trap.png', route => route.abort());
  await page.goto(`${url}?engine=fake&fresh=1&runs=0`);
  await page.waitForFunction(() => window.__riddle?.booted);
  await page.evaluate(async () => {
    const { itemChip } = await import('/src/ui/items.ts');
    const chip = itemChip({ kind: 'trap', label: 'ranger trap', rarity: 'rare' });
    chip.id = 'failed-icon'; document.body.append(chip);
  });
  await page.waitForSelector('#failed-icon .glyph');
  const fallback = await page.evaluate(() => {
    const el = document.querySelector('#failed-icon');
    return { text: el.textContent, rarity: el.dataset.rarity, kind: el.querySelector('.item-ico').dataset.kind,
      broken: !!el.querySelector('img'), glyph: el.querySelector('.glyph').dataset.glyph,
      border: getComputedStyle(el.querySelector('.item-ico')).borderWidth };
  });
  if (fallback.text !== 'ranger trap' || fallback.rarity !== 'rare' || fallback.kind !== 'trap' || fallback.broken || !fallback.glyph || fallback.border !== '0px')
    throw Error(JSON.stringify(fallback));
  console.log('failed packed image keeps identity/rarity/name and unframed primitive PASS');
} finally { await browser.close(); }
