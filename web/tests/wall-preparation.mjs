import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';
const url = execFileSync('bash', ['tools/dev.sh'], { cwd: new URL('../../', import.meta.url), encoding: 'utf8' }).trim();
const browser = await launchBrowser();
try {
 for (const width of [320, 400, 1440]) {
  const p = await browser.newPage({ viewport: { width, height: 900 } });
  await p.goto(`${url}?engine=fake&fresh=1&runs=0`);
  await p.waitForFunction(() => window.__riddle?.booted);
  const n = await p.evaluate(async () => {
   const a = window.__riddle, { closeAllSheets } = await import('/src/ui/sheet.ts');
   a.runnerOn = false; a.rulesChanged = () => {};
   let checks = 0, builds = 0, spends = 0, forecasts = 0;
   const check = (ok, name) => { if (!ok) throw Error(name); checks++; };
   const pause = () => new Promise(r => setTimeout(r, 20));
   const report = { elapsed_s: 28800, runs: 10, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: 'goblin_warlord', n: 8 }], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 8, gold: { home: 500 } };
   const town = { home: true, buildings: [], next: 'blacksmith', next_ready: true, next_trigger: 'first gold home', bank: 0, bank_cap: 1000, interest: 0 };
   a.lineage = { ...a.lineage, ended: false, live: null, town, selected_bloodline: 2, bloodline: { points: 54, spent: 0, upgrades: {} }, legacy_upgrades: [{ id: 'health', rank: 0, cap: 3, affordable: true, price: 3, effect: '+5 hp' }] };
   a.lineage.packages.all.find(p => p.id === 'guarded').owned = true;
   a.engine.upgradeHero = async () => { spends++; return a.lineage; };
   a.engine.kitEstimates = a.engine.packageOptions = async () => { forecasts++; return []; };
   a.engine.buildTown = async () => { builds++; throw Error('refused diagnostic build'); };
   a.go({ kind: 'report', report });
   const action = cls => document.querySelector(`.preparation-actions .${cls}`);
   check(document.querySelector('.preparation-obstacle').textContent.includes('goblin warlord'), 'actual repeated obstacle visible');
   check(document.querySelector('.preparation-obstacle [data-unit="goblin_warlord"]'), 'named unit has matching icon');
   check(action('report-upgrade').textContent.includes('Bloodline 2 · 54 Legacy'), 'current selected bloodline named');
   check(action('preparation-forge').textContent.includes('Build forge') && action('preparation-forge').textContent.includes('Free'), 'ready plot differs from built forge');
   check(document.querySelectorAll('.preparation-action').length <= 3, 'bounded choices');
   action('report-upgrade').click(); check(document.querySelector('.hero-legacy').textContent.includes('54') && spends === 0, 'hero opening is read-only'); closeAllSheets();
   action('preparation-tactics').click(); check(!!document.querySelector('.pkg-panel') && forecasts === 0, 'owned choices open without forecasts'); closeAllSheets();
   check(action('preparation-forge'), 'forge action present before navigation: ' + JSON.stringify(a.lineage.town)); action('preparation-forge').click();
   check(!!document.querySelector('.preparation-build') && builds === 0 && !document.querySelector('.simple-kit'), 'unbuilt forge reviews construction without spending');
   document.querySelector('.preparation-build-button').click(); await pause();
   check(builds === 1 && !document.querySelector('.preparation-build-button').disabled, 'refused build re-enables review');
   const stale = document.querySelector('.preparation-build-button');
   a.lineage = { ...a.lineage, town: { ...town, next_ready: false } }; await a.afterLineage();
   check(!action('preparation-forge') && document.querySelector('.preparation-build-button').disabled, 'unready wire cannot offer free construction');
   stale.click(); check(builds === 1, 'stale ready button cannot construct'); closeAllSheets();
   a.lineage = { ...a.lineage, town }; await a.afterLineage(); check(action('preparation-forge'), 'forge action present before navigation: ' + JSON.stringify(a.lineage.town)); action('preparation-forge').click();
   check(document.querySelector('.preparation-build-button'), 'review remains available before engine swap: ' + JSON.stringify({town:a.lineage.town, html:document.querySelector('.sheet')?.textContent}));
   const oldEngine = a.engine, oldButton = document.querySelector('.preparation-build-button');
   a.engine = { ...oldEngine }; oldButton.click(); check(builds === 1, 'engine swap rejects captured build'); closeAllSheets(); a.engine = oldEngine;
   let release;
   a.engine.buildTown = () => { builds++; return new Promise(r => release = r); };
   check(action('preparation-forge'), 'forge action present before navigation: ' + JSON.stringify(a.lineage.town)); action('preparation-forge').click(); const double = document.querySelector('.preparation-build-button'); double.click(); double.click();
   check(builds === 2 && document.querySelector('.preparation-build-button').disabled, 'one build in flight');
   release({ ...a.lineage, town: { ...town, next: 'storehouse', buildings: [{ id: 'blacksmith', day: 1, level: 1 }] } });
   await pause(); check(!document.querySelector('.preparation-build') && !!document.querySelector('.simple-kit'), 'successful manual construction opens existing forge');
   check(spends === 0 && forecasts === 0, 'construction neither upgrades hero nor forecasts'); closeAllSheets();
   a.lineage = { ...a.lineage, kit: [{ slot: 'weapon', owned: 0, steps: [], next: { affordable: true, price: 100 } }] }; await a.afterLineage();
   check(action('preparation-forge').textContent.includes('Forge gear'), 'built forge offers actual affordable upgrade');
   check(action('preparation-forge'), 'forge action present before navigation: ' + JSON.stringify(a.lineage.town)); action('preparation-forge').click(); check(!!document.querySelector('.simple-kit') && builds === 2, 'built forge never rebuilds'); closeAllSheets();
   a.lineage = { ...a.lineage, selected_bloodline: 3, bloodline: { points: 8, spent: 0, upgrades: {} } }; await a.afterLineage();
   check(action('report-upgrade').textContent.includes('Bloodline 3 · 8 Legacy'), 'selected slot updates preparation');
   await document.fonts.ready; await new Promise(r => setTimeout(r, 100));
   // Cut 118 round 2 §6: on the report the shop rows (`Forge gear`) wait under the folded `details`, never the card's decision — the rendered ones keep their bounds
   check(!document.querySelector('.report') || !document.querySelector('.report .preparation-forge') || !!document.querySelector('.report .preparation-forge').closest('.report-details'), 'report shop row folded');
   for (const b of document.querySelectorAll('.preparation-action')) { if (!b.getClientRects().length) continue; const r = b.getBoundingClientRect(); check(r.width >= 44 && r.height >= 44, 'touch bounds'); }
   check(document.documentElement.scrollWidth <= innerWidth, 'no horizontal overflow');
   for (const patch of [{ live: { depth: 8 } }, { live: null, ended: true }, { ended: false, town: { ...town, home: false } }]) {
    a.lineage = { ...a.lineage, ...patch }; await a.afterLineage(); check(document.querySelector('.preparation-host').hidden, 'away/ended/empty preparation hidden');
   }
   a.lineage = { ...a.lineage, town, live: null, ended: false }; await a.afterLineage();
   const detached = document.querySelector('.preparation-host'), text = detached.textContent;
   const death = { run_id: 1, depth: 8, cause: 'goblin_warlord', margin: '2 hp short', verdict: 'gap', baseline: 0, replays: 12, patches: [], morgue: '', trace: { turns: [] }, lever: { kind: 'spend', text: 'sword +1' } };
   a.go({ kind: 'death', death, kept: true });
   check(!document.querySelector('.preparation-fold') && document.querySelector('.preparation-section h3').textContent === 'Next run', 'death next-run actions are visible and named');
   check(document.querySelector('.death-lever').dataset.kind === 'spend', 'engine primary recommendation retained');
   check(getComputedStyle(action('report-upgrade')).borderImageSource.includes('tablet.png'), 'death links use shared chunky frame');
   document.querySelector('.death-lever').click(); check(!!document.querySelector('.preparation-build') && !document.querySelector('.simple-kit'), 'death spend respects unbuilt forge'); closeAllSheets();
   check(!document.querySelector('.preparation-obstacle'), 'death does not repeat the killer beside next-run options');
   check(action('report-upgrade').textContent.includes('Bloodline 3 · 8 Legacy'), 'historical death prepares current bloodline explicitly');
   a.lineage = { ...a.lineage, bloodline: { points: 99, spent: 0, upgrades: {} } }; await a.afterLineage();
   check(!detached.isConnected && detached.textContent === text, 'disposed report does not repaint');
   check(builds === 2 && spends === 0 && forecasts === 0, 'only explicit build changed state');
   action('preparation-forge').click(); document.querySelector('.preparation-build-button').click();
   const owner = a.engine, beforeReplacement = a.lineage;
   a.engine = { ...owner };
   release({ ...a.lineage, town: { ...town, buildings: [{ id: 'blacksmith', day: 1, level: 1 }] } });
   await pause();
   check(a.lineage === beforeReplacement && builds === 3, 'late old-engine completion cannot overwrite replacement state');
   closeAllSheets(); a.engine = owner;

   return checks;
  });
  console.log(width, n, 'wall preparation/read-only/manual/stale/lifecycle PASS'); await p.close();
 }
} finally { await browser.close(); }
