#!/usr/bin/env node
// A newly visible message must remove an already drawn overlapping nameplate
// before the next renderer frame, rather than showing both for one frame.
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { launchBrowser } from '../../tools/browser.mjs';
const url = execFileSync('bash', [resolve('../tools/dev.sh')], {encoding:'utf8'}).trim();
const browser = await launchBrowser();
try {
  const page = await browser.newPage({viewport:{width:400,height:800},deviceScaleFactor:2});
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&autosend=1&early=0`);
  await page.waitForFunction(() => window.__viewer?.debugLabels?.().some(l => l.w && l.h), null, {timeout:30000});
  const result = await page.evaluate(() => {
    const v=window.__viewer, label=v.debugLabels().find(l=>l.w&&l.h);
    const shown=()=>[...document.querySelectorAll('.rtag')].filter(e=>e.getClientRects().length&&getComputedStyle(e).display!=='none');
    const before=shown().length;
    // Simulate the layout reservation a message supplies, using its real CSS
    // coordinates. Both DOM tags and debug labels describe the displayed frame.
    v.setKeepOut([{x:label.x-label.w/2-2,y:label.y-label.h-2,w:label.w+4,h:label.h+4}]);
    return {before,after:shown().length,stillReserved:v.debugLabels().some(l=>l.id===label.id)};
  });
  if (!(result.before>0&&result.after<result.before&&!result.stillReserved)) throw new Error(`new message leaves old nameplate visible: ${JSON.stringify(result)}`);
  console.log('watch keep-out: immediate DOM nameplate removal PASS');
} finally {await browser.close();}
