// Real earned first-ascension clear, paid216-point build; no granted state.
import {execFileSync} from 'node:child_process';
import {readFileSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const source=readFileSync(new URL('./fixtures/earned-first-ascension-clear.json',import.meta.url),'utf8'),savedLoadout=JSON.parse(source).loadout;
const browser=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await browser.newPage({viewport:{width,height:900},reducedMotion:'reduce'}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?dev=1&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 assert.equal(await p.evaluate(()=>window.__riddle.kind),'wasm');
 assert.equal(await p.evaluate(async({source,loadout})=>window.__riddle.importSave(JSON.stringify({v:2,engine:source,loadout,last_seen:Date.now(),runs:["runs_banked","runs_returned","runs_died"].reduce((sum,k)=>sum+(JSON.parse(source).lineage[k]??0),0)})),{source,loadout:savedLoadout}),true);
 await p.waitForSelector('.ending-town');
 const before=await p.evaluate(async()=>{const a=window.__riddle;a.runnerOn=false;window.transitionCalls={send:0,begin:0};const send=a.engine.send.bind(a.engine),begin=a.engine.beginDescent.bind(a.engine);a.engine.send=async()=>{window.transitionCalls.send++;return send();};a.engine.beginDescent=async tier=>{window.transitionCalls.begin++;return begin(tier);};return a.engine.save();});
 const unchanged=async()=>{assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);assert.deepEqual(await p.evaluate(()=>window.transitionCalls),{send:0,begin:0});};
 await p.locator('.ending-town').focus();await p.keyboard.press('Enter');await p.waitForSelector('.camp');await unchanged();
 assert.match(await p.locator('.gem.send').textContent(),/Next descent/);assert.equal(await p.locator('.gem.send').isDisabled(),false);
 await p.locator('button.stat.legacy').click();await p.waitForSelector('.legacy-panel');await unchanged();
 assert.equal(await p.locator('.legacy-respec').isDisabled(),false);await p.keyboard.press('Escape');
 await p.locator('.gem.send').click();await p.waitForSelector('.ending-town');await unchanged();
 await p.locator('.ending-town').click();await p.waitForSelector('.camp');
 await p.locator('.town-hit[data-building="mouth"]').click();await p.waitForSelector('.ending-town');await unchanged();
 await p.evaluate(()=>window.__riddle.go({kind:'watch'}));await p.waitForSelector('.ending-town');await unchanged();
 await p.locator('.ending-town').click();await p.waitForSelector('.camp');
 await p.evaluate(()=>window.__riddle.go({kind:'camp'}));await p.waitForSelector('.ending-town');await unchanged();
 await p.locator('.ending-town').click();await p.waitForSelector('.camp');
 assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 if(width===400){
  await p.locator('button.stat.legacy').click();const old=await p.evaluate(()=>({...window.__riddle.lineage.bloodline}));
  await p.locator('.legacy-respec').click();await p.locator('.legacy-respec-confirm').click();await p.waitForFunction(()=>window.__riddle.lineage.bloodline.spent===0);
  assert.equal(await p.evaluate(()=>window.__riddle.lineage.bloodline.points),old.points+old.spent);
  await p.locator('button.legacy-buy[data-upgrade="health"]').click();await p.waitForFunction(()=>window.__riddle.lineage.bloodline.spent===3);
  assert.equal(await p.evaluate(()=>window.__riddle.lineage.bloodline.points),old.points+old.spent-3);
  assert.equal(await p.evaluate(()=>window.__riddle.lineage.ended),true);assert.equal(await p.evaluate(()=>window.__riddle.lineage.live??null),null);
  assert.equal(await p.evaluate(async()=>JSON.parse(await window.__riddle.engine.save()).run),null);
  assert.deepEqual(await p.evaluate(()=>window.transitionCalls),{send:0,begin:0});
  await p.evaluate(()=>window.__riddle.flush());await p.reload();await p.waitForSelector('.ending-town');
  await p.locator('.ending-town').click();await p.waitForSelector('.camp');assert.equal(await p.evaluate(()=>window.__riddle.lineage.bloodline.spent),3);
 }
 if(width===1440){
  const multi=await p.evaluate(async()=>{const a=window.__riddle;
   a.lineage=await a.engine.addBloodline();a.lineage=await a.engine.selectBloodline(2);
   await a.engine.send();await a.engine.advance(250);
   a.lineage=await a.engine.selectBloodline(1);await a.refresh();a.go({kind:'ending'});
   window.transitionCalls={send:0,begin:0};return a.engine.save();});
  await p.locator('.ending-town').click();await p.waitForSelector('.camp');
  assert.equal(await p.locator('.hero-desktop .hero-row').count(),2);
  assert.equal(await p.locator('.hero-desktop .hero-row[data-slot="2"]').getAttribute('data-state'),'live');
  await p.locator('button.stat.legacy').click();await p.waitForSelector('.legacy-panel');
  assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),multi);
  assert.equal(await p.evaluate(()=>window.__riddle.lineage.ended),true);
  assert.deepEqual(await p.evaluate(()=>window.transitionCalls),{send:0,begin:0});
 }
 assert.deepEqual(errors,[]);console.log(width,'earned clear Town/Legacy/Next descent/mouth/watch guard, exact navigation save and paid home respec PASS');await p.close();
}}finally{await browser.close();}
