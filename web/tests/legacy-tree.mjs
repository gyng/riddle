// UI transaction/lifecycle fixtures. Core purchase/effect truth has separate Rust gates.
import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),browser=await launchBrowser();
try {for(const width of [320,400,1440]) {
 const page=await browser.newPage({viewport:{width,height:900}}),errors=[];page.on('pageerror',error=>errors.push(error.message));
 await page.goto(`${url}?engine=fake&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 await page.evaluate(async()=>{
  const app=window.__riddle;app.runnerOn=false;window.legacyCalls={buy:0,respec:0};
  const nodes=[['health','Health','Recovery',null,0],['damage','Damage','Control',null,0],['armour','Armour','Warding',null,0],
   ['restoration','Restoration','Recovery','health',8],['control','Control','Control','damage',8],['clear_lungs','Clear lungs','Warding','armour',8],
   ['mending','Mending','Recovery','restoration',18],['renewal','Renewal','Recovery','restoration',18],['venom','Venom','Control','control',18],
   ['debilitate','Debilitate','Control','control',18],['fireward','Fireward','Warding','clear_lungs',18],['brace','Brace','Warding','clear_lungs',18]];
  const wire=()=>{const b=app.lineage.bloodline;app.lineage.legacy_respec={refund:b.spent,available:b.spent>0,points_after:b.points+b.spent,blocked:null};return nodes.map(([id,name,branch,parent,min_depth])=>{const rank=b.upgrades[id]??0;
   const fork=id==='renewal'&&app.lineage.bloodline.upgrades.mending;
   const blocked=rank>=(parent?1:3)?'Complete':fork?'Chosen Mending':parent&&!b.upgrades[parent]?`Buy ${parent}`:undefined;
   return{id,name,branch,parent,min_depth,rank,cap:parent?1:3,price:parent?(min_depth===8?18:36):3*(rank+1),effect:id==='health'?'+3 HP':'+1 effect',affordable:!blocked,blocked};});};
  app.lineage={...app.lineage,live:null,selected_bloodline:1,town:{...app.lineage.town,home:true},bloodline:{points:100,spent:0,upgrades:{}}};
  app.lineage.legacy_upgrades=wire();app.engine.lineage=async()=>app.lineage;
  app.engine.upgradeHero=async id=>{window.legacyCalls.buy++;const row=app.lineage.legacy_upgrades.find(u=>u.id===id);
   app.lineage={...app.lineage,bloodline:{points:app.lineage.bloodline.points-row.price,spent:app.lineage.bloodline.spent+row.price,upgrades:{...app.lineage.bloodline.upgrades,[id]:row.rank+1}}};
   app.lineage.legacy_upgrades=wire();return app.lineage;};
  window.installRespec=(deferred=false)=>{app.engine.respecLegacy=async()=>{window.legacyCalls.respec++;
   if(deferred)await new Promise(resolve=>window.releaseRespec=resolve);
   app.lineage={...app.lineage,bloodline:{points:app.lineage.bloodline.points+app.lineage.bloodline.spent,spent:0,upgrades:{}}};
   app.lineage.legacy_upgrades=wire();return app.lineage;};};window.installRespec();
  const{openHero}=await import('/src/ui/town.ts');openHero(app);
 });
 assert.equal(await page.locator('.legacy-branch').count(),3);assert.equal(await page.locator('.legacy-upgrade').count(),12);
 assert.equal(await page.locator('.legacy-icon img').count(),12);assert.equal(await page.evaluate(()=>window.legacyCalls.buy),0);
 const buy=id=>page.locator(`button.legacy-buy[data-upgrade="${id}"]`);
 assert.ok(await buy('restoration').isDisabled());assert.ok(await buy('mending').isDisabled());assert.ok(await page.locator('.legacy-respec').isDisabled());
 if(width<900){assert.equal(await page.locator('.legacy-path[open]').count(),0);await page.locator('[data-legacy-branch="Recovery"] summary').click();}
 await buy('health').click();await page.waitForFunction(()=>window.__riddle.lineage.bloodline.spent===3);
 await buy('restoration').click();await page.waitForFunction(()=>window.__riddle.lineage.bloodline.spent===21);
 await buy('mending').click();await page.waitForFunction(()=>window.__riddle.lineage.bloodline.spent===57);
 assert.equal(await page.evaluate(()=>window.legacyCalls.buy),3);assert.ok(await buy('renewal').isDisabled());
 assert.equal(await page.locator('section[data-upgrade="mending"] .legacy-buy').textContent(),'Chosen');
 await page.keyboard.press('Tab');await page.locator('section[data-upgrade="mending"] .legacy-node-name').focus();await page.locator('#kw-tip:not([hidden])').waitFor();
 assert.match(await page.locator('#kw-tip').textContent(),/Requires Restoration/);assert.match(await page.locator('#kw-tip').textContent(),/Reach D18/);await page.keyboard.press('Escape');
 await page.locator('.legacy-respec').click();assert.match(await page.locator('.legacy-respec-review').textContent(),/Refund 57 Legacy/);
 assert.equal(await page.evaluate(()=>window.legacyCalls.respec),0);await page.locator('.legacy-respec-cancel').click();assert.equal(await page.evaluate(()=>window.__riddle.lineage.bloodline.spent),57);
 await page.locator('.legacy-respec').click();await page.evaluate(()=>{window.__riddle.engine.respecLegacy=async()=>{throw Error('fixture refusal');};});
 await page.locator('.legacy-respec-confirm').click();await page.waitForFunction(()=>document.querySelector('.legacy-respec-review .legacy-feedback').textContent.includes('fixture refusal'));
 assert.equal(await page.evaluate(()=>window.__riddle.lineage.bloodline.spent),57);await page.locator('.legacy-respec-cancel').click();
 await page.evaluate(()=>window.installRespec(true));await page.locator('.legacy-respec').click();
 await page.locator('.legacy-respec-confirm').evaluate(el=>{el.click();el.click();});await page.waitForFunction(()=>typeof window.releaseRespec==='function');
 assert.equal(await page.evaluate(()=>window.legacyCalls.respec),1);assert.ok(await page.locator('.legacy-respec-confirm').isDisabled());
 await page.evaluate(()=>window.releaseRespec());await page.waitForFunction(()=>!document.querySelector('.legacy-respec-review'));
 assert.deepEqual(await page.evaluate(()=>window.__riddle.lineage.bloodline),{points:100,spent:0,upgrades:{}});assert.ok(await page.locator('.legacy-respec').isDisabled());
 await buy('health').click();await page.waitForFunction(()=>window.__riddle.lineage.bloodline.spent===3);
 await page.locator('.legacy-respec').click();await page.evaluate(async()=>{window.__riddle.lineage.selected_bloodline=2;await window.__riddle.afterLineage();});
 await page.locator('.legacy-respec-confirm').click();assert.equal(await page.locator('.legacy-respec-review .legacy-feedback').textContent(),'Review changed');assert.equal(await page.evaluate(()=>window.legacyCalls.respec),1);
 await page.locator('.legacy-respec-cancel').click();
 await page.evaluate(async()=>{const a=window.__riddle;a.lineage={...a.lineage,live:{depth:1,turn:1},legacy_respec:{...a.lineage.legacy_respec,available:false,blocked:'hero away'},legacy_upgrades:a.lineage.legacy_upgrades.map(u=>({...u,affordable:false}))};await a.afterLineage();});
 assert.ok(await page.locator('.legacy-respec').isDisabled());for(const button of await page.locator('.legacy-buy').all())assert.ok(await button.isDisabled());
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.deepEqual(errors,[]);
 console.log(width,'12 nodes/icons, mobile fold, purchase/fork/tooltips, cancel/refusal/one-call/stale/away/refund PASS');await page.close();
}} finally {await browser.close();}
