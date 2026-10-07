#!/usr/bin/env node
// A diagonal corner move reflects the stack fan onto open ground. Full-size
// bosses cover <=10% and other sprites <=30% of the hero (+1% rounding).
import assert from 'node:assert/strict';
import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';

const url = execFileSync('bash', ['../tools/dev.sh'], { encoding: 'utf8' }).trim();
const browser = await launchBrowser();
try {
  for (const width of [400, 1440]) {
    const page = await browser.newPage({ viewport: { width, height: 800 }, deviceScaleFactor: 2 });
    const errors = [];
    page.on('pageerror', e => errors.push(e.message));
    await page.goto(url);
    await page.waitForFunction(() => window.__riddle?.booted);
    const rects = await page.evaluate(async () => {
      document.body.innerHTML = '<canvas style="width:100vw;height:100vh"></canvas>';
      const { createViewer } = await import('/src/render/index.ts');
      const viewer = createViewer(document.querySelector('canvas'));
      try {
        const w = 7, h = 7;
        const entity = (id, kind, x, y, tags = []) => ({ id, kind, x, y, tags, hp: 100, max_hp: 100 });
        const snap = {
          w, h, depth: 1, biome: 'warrens', turn: 10,
          tiles: Array.from({ length: w * h }, (_, i) =>
            i % w === 0 || i % w === w - 1 || i < w || i >= w * (h - 1) ? 'wall' : 'floor'),
          seen: Array(w * h).fill(true), visible: Array(w * h).fill(true),
          overlays: [], items: [], alert: 0, loot: 0,
          run: { id: 1, heir: 1, started_turn: 0 },
          hero: { ...entity(0, 'hero_fighter_male', 0, 2), inv: [], class: 'fighter', trait: 'steady' },
          entities: [entity(1, 'goblin_warlord', 2, 3, ['boss']),
            entity(2, 'goblin', 1, 3), entity(3, 'ogre', 1, 2)],
        };
        snap.tiles[2 * w] = 'floor';
        viewer.load(snap);
        viewer.setFrame('fight');
        viewer.setSpeed(0);
        viewer.apply([{ t: 10, k: 'move', id: 0, x: 1, y: 3 },
          { t: 20, k: 'move', id: 0, x: 2, y: 3 }]);
        viewer.seek(15);
        const deadline = performance.now() + 5000;
        while (viewer.debugRects().length !== 4 && performance.now() < deadline)
          await new Promise(requestAnimationFrame);
        return viewer.debugRects();
      } finally { viewer.dispose(); }
    });
    const hero = rects.find(r => r.hero), foes = rects.filter(r => !r.hero);
    assert(hero, 'hero remains drawn');
    assert.equal(foes.length, 3);
    for (const foe of foes) {
      assert(foe.z < hero.z, `${width}: ${foe.kind} behind hero`);
      const overlap = Math.max(0, Math.min(hero.x + hero.w, foe.x + foe.w) - Math.max(hero.x, foe.x)) *
        Math.max(0, Math.min(hero.y + hero.h, foe.y + foe.h) - Math.max(hero.y, foe.y));
      const share = overlap / (hero.w * hero.h);
      assert(share <= (foe.kind === 'goblin_warlord' ? 0.11 : 0.31), `${width}: ${foe.kind} covers ${share}`);
    }
    assert.deepEqual(errors, []);
    await page.close();
  }
  console.log('hero-grounding: ok (2 widths)');
} finally { await browser.close(); }
