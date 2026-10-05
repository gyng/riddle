// Real-engine acceptance: earn the second slot, then observe independent live heroes.
import { launchGpu } from './browser.mjs';
import { mkdirSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
const [url='http://localhost:5367/?engine=native',out='scratchpad/bloodlines-ui']=process.argv.slice(2);
mkdirSync(out,{recursive:true});
const browser=await launchGpu(),checks=[],errors=[];
const check=(name,value)=>{assert.ok(value,name);checks.push(name);};
try {
 for(const width of [400,1440]) {
  const page=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:2});
  page.on('pageerror',e=>errors.push(e.message));
  const u=new URL(url);for(const [k,v] of Object.entries({fresh:'1',seed:'1',runs:'0',lanes:'0'}))u.searchParams.set(k,v);
  await page.goto(u.href);await page.waitForFunction(()=>window.__riddle?.booted);
  check(`${width}: fresh roster empty`,await page.evaluate(()=>window.__riddle.lineage.hero_slots.length===0));
  await page.locator('.town-tag[data-next=house]').click();await page.waitForFunction(()=>window.__riddle?.lineage.hero_slots.length===1);
  const earn=await page.evaluate(async()=>{const a=window.__riddle;let runs=0;while((await a.engine.lineage()).gold<250&&runs<5){await a.engine.send();await a.engine.runOfflineQuick(3600);runs++;}await a.refresh();return {runs,gold:a.lineage.gold};});
  check(`${width}: earns slot price through real runs`,earn.runs>0&&earn.gold>=250);
  await page.locator('.topbar button.legacy').click();await page.locator('.legacy-buy[data-upgrade=health]').click();
  await page.waitForFunction(()=>window.__riddle.lineage.bloodline.upgrades.health===1);
  await page.locator('.sheet-wrap button.stud').click();
  if(width===400)await page.locator('.hero-expand').click();
  const before=await page.evaluate(()=>window.__riddle.lineage.gold);
  await page.locator('.hero-add:visible').click();await page.waitForFunction(()=>window.__riddle.lineage.hero_slots.length===2);
  await page.waitForFunction(()=>[...document.querySelectorAll('.hero-list')].some(e=>e.getBoundingClientRect().width>0&&e.querySelectorAll('.hero-row').length===2));
  check(`${width}: explicit slot costs exactly 250`,await page.evaluate(n=>window.__riddle.lineage.gold===n-250,before));
  await page.waitForFunction(()=>document.querySelector('.topbar .stat.gold')?.textContent.replace(/\s/g,'')===`Gold$${window.__riddle.lineage.gold}`);
  check(`${width}: roster includes active slots only`,await page.locator('.hero-list:visible .hero-row').count()===2);
  await page.mouse.move(width-1,0);await page.waitForTimeout(1200);await page.screenshot({path:`${out}/${width}-roster.png`});
  await page.locator('.hero-list:visible .hero-row[data-slot="2"] .hero-details').click();
  await page.waitForFunction(()=>window.__riddle.lineage.selected_bloodline===2);
  await page.waitForSelector('.hero-sheet button.hero-class:visible');
  check(`${width}: second bloodline has independent Legacy`,await page.evaluate(()=>window.__riddle.lineage.bloodline.points===0&&!window.__riddle.lineage.bloodline.upgrades.health));
  check(`${width}: class controls in Details`,await page.locator('button.hero-class').isVisible());
  await page.locator('button.hero-class').click();check(`${width}: class picker accessible`,await page.locator('.sheet-wrap:visible').innerText().then(t=>t.includes('fighter')));
  await page.locator('.sheet-wrap:visible button.close-stud').click();
  await page.evaluate(async()=>{const a=window.__riddle;await a.engine.send();await a.engine.advance(100);await a.refresh();});
  await page.evaluate(async()=>{const a=window.__riddle;await a.selectBloodline(1);await a.engine.send();await a.engine.advance(100);await a.refresh();});
  check(`${width}: both heroes live`,await page.evaluate(()=>window.__riddle.lineage.hero_slots.every(s=>s.state==='live'&&s.live)));
  const live=await page.evaluate(()=>window.__riddle.lineage.hero_slots.map(s=>({id:s.id,run:s.live.run_id,turn:s.live.turn})));
  if(width===400)await page.locator('.hero-expand').click();
  await page.locator('.hero-list:visible button[data-hero="2"]').click();await page.waitForSelector('main.watch');
  check(`${width}: row focuses selected live hero`,await page.evaluate(()=>window.__riddle.lineage.selected_bloodline===2));
  await page.mouse.move(width-1,0);await page.waitForTimeout(1200);await page.screenshot({path:`${out}/${width}-multi-watch.png`});
  await page.locator('button[data-tile=town]').click();await page.waitForSelector('main.camp');
  check(`${width}: no duplicate bottom class bar`,await page.locator('.well-slot:visible').count()===0);
  await page.evaluate(async()=>{await window.__riddle.refresh();});
  check(`${width}: both runs retained after focusing`,await page.evaluate(ids=>ids.every(x=>window.__riddle.lineage.hero_slots.find(s=>s.id===x.id)?.live?.run_id===x.run),live));
  await page.evaluate(async()=>{const a=window.__riddle;await a.selectBloodline(1);await a.flush();});
  check(`${width}: first bloodline upgrades retained`,await page.evaluate(()=>window.__riddle.lineage.bloodline.upgrades.health===1));
  check(`${width}: no horizontal overflow`,await page.evaluate(()=>document.documentElement.scrollWidth)<=width);
  u.searchParams.delete('fresh');await page.goto(u.href);await page.waitForFunction(()=>window.__riddle?.booted);
  check(`${width}: both bloodlines and upgrades reload`,await page.evaluate(()=>window.__riddle.lineage.hero_slots.length===2&&window.__riddle.lineage.bloodline.upgrades.health===1));
  await page.close();
 }
 check('no page errors',errors.length===0);writeFileSync(`${out}/checks.json`,JSON.stringify({pass:true,checks,errors},null,2)+'\n');console.log(`PASS ${checks.length} real-engine bloodline UI checks`);
}finally{await browser.close();}
