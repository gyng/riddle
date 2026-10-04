// Real-engine acceptance for the owner's simpler first session and live watch.
import { launchGpu } from './browser.mjs';
import { mkdirSync, writeFileSync } from 'node:fs';
import assert from 'node:assert/strict';
const [url='http://localhost:5367/?engine=native',out='scratchpad/simple-ui']=process.argv.slice(2);
mkdirSync(out,{recursive:true});
const browser=await launchGpu(), checks=[], errors=[];
const check=(name,value)=>{assert.ok(value,name);checks.push(name);};
try {
 for(const width of [400,1440]) {
  const page=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:2});
  page.on('pageerror',e=>errors.push(e.message));
  const u=new URL(url); for(const [k,v] of Object.entries({fresh:'1',seed:'1',lanes:'0'}))u.searchParams.set(k,v);
  await page.goto(u.href);
  await page.waitForFunction(()=>window.__riddle?.booted&&window.__riddle.screen==='camp');
  check(`${width}: fresh buildings empty`,await page.evaluate(()=>window.__riddle.lineage.town.buildings.length===0));
  check(`${width}: extra camp panel hidden`,!await page.locator('.camp-main').isVisible());
  check(`${width}: send has no speed switch`,await page.locator('.send-mode').count()===0);
  await page.screenshot({path:`${out}/${width}-camp.png`});
  await page.locator('button.send').click();
  await page.waitForSelector('main.watch');
  check(`${width}: explicit live label`,await page.locator('.live-badge').isVisible()&&(await page.locator('.live-badge').innerText()).includes('Live delve'));
  check(`${width}: two watch menu controls`,await page.locator('.console .cmd button.hud-btn:visible').count()===2);
  await page.waitForFunction(()=>document.querySelectorAll('.combat-lines li').length>0,null,{timeout:30000});
  check(`${width}: bounded ordered log`,await page.evaluate(()=>{const t=[...document.querySelectorAll('.combat-lines li')].map(x=>Number(x.dataset.tick));return t.length<=80&&t.every((x,i)=>!i||x>=t[i-1]);}));
  await page.screenshot({path:`${out}/${width}-watch.png`});
  await page.getByRole('button',{name:'Speed',exact:true}).click();
  await page.getByRole('button',{name:'normal',exact:true}).click();
  await page.waitForSelector(".watch-options", {state:"detached"});
  check(`${width}: speed selection applies and closes`,await page.locator('main.watch').getAttribute('data-mode')==='one'&&await page.locator('.watch-options').count()===0);
  const live=await page.evaluate(async()=> (await window.__riddle.engine.lineage()).live?.run_id);
  check(`${width}: clear town exit`,(await page.locator('button[data-tile=town]').innerText()).includes('Town menu'));
  await page.locator('button[data-tile=town]').click();
  await page.waitForSelector('main.camp');
  await page.waitForFunction(()=>!!window.__riddle.lineage.live);
  check(`${width}: leaving watch keeps same run`,await page.evaluate(id=>window.__riddle.lineage.live.run_id===id,live));
  await page.evaluate(()=>window.__riddle.absence(3600));
  await page.waitForSelector('main.report');
  check(`${width}: concise report`,await page.locator('.report-basics .tile').count()===3&&!await page.locator('.report-details').isVisible());
  check(`${width}: heading contrast`,await page.evaluate(()=>getComputedStyle(document.querySelector('.report-summary h2')).color===getComputedStyle(document.documentElement).getPropertyValue('--ink').trim()||getComputedStyle(document.querySelector('.report-summary h2')).color==='rgb(13, 12, 20)'));
  await page.screenshot({path:`${out}/${width}-report.png`});
  await page.getByRole('button',{name:'details',exact:true}).click();
  check(`${width}: details accessible`,await page.locator('.report-details').isVisible());
  await page.locator('main.report .gem').click();
  await page.waitForSelector('main.camp');
  check(`${width}: milestone did not auto-build`,await page.evaluate(()=>window.__riddle.lineage.town.buildings.length===0&&window.__riddle.lineage.town.next_ready));
  await page.locator('button.town-tag').click();
  await page.waitForFunction(()=>window.__riddle.lineage.town.buildings.some(b=>b.id==='blacksmith'));
  check(`${width}: explicit build works`,await page.locator('.town-hit[data-building=blacksmith]').isVisible());
  await page.evaluate(()=>window.__riddle.flush());
  u.searchParams.delete("fresh"); await page.goto(u.href);
  await page.waitForFunction(()=>window.__riddle?.booted);
  check(`${width}: construction persisted`,await page.evaluate(()=>window.__riddle.lineage.town.buildings.some(b=>b.id==='blacksmith')));
  await page.close();
 }
 check('no page errors',errors.length===0);
 writeFileSync(`${out}/checks.json`,JSON.stringify({pass:true,checks,errors},null,2)+'\n');
 console.log(`PASS: ${checks.length} real-engine UX checks at400/1440px`);
} finally {await browser.close();}
