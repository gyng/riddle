// A manual construction must survive navigation from its first visible update.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const b=await launchBrowser();let checks=0;
async function open(width,motion='reduce'){
 const p=await b.newPage({viewport:{width,height:900},reducedMotion:motion});
 const errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?fresh=1&seed=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 assert.equal(await p.evaluate(()=>window.__riddle.kind),'wasm');checks++;
 return {p,errors};
}
try{
 for(const width of [400,1440])for(const motion of ['reduce','no-preference']){
  const {p,errors}=await open(width,motion);
  await p.evaluate(()=>{const a=window.__riddle;let navigating=false;a.onChange(()=>{if(a.lineage.town.home&&!navigating){navigating=true;sessionStorage.setItem('town-save-hero',JSON.stringify(a.lineage.hero_slots));location.replace('/?townSave=1');}});document.querySelector('.town-tag[data-next="house"]').click();});
  await p.waitForURL('**/?townSave=1');await p.waitForFunction(()=>window.__riddle?.booted);
  const restored=await p.evaluate(()=>({home:window.__riddle.lineage.town.home,heroes:window.__riddle.lineage.hero_slots,expected:JSON.parse(sessionStorage.getItem('town-save-hero')),awaiting:document.querySelector('main.camp').classList.contains('awaiting-home')}));
  assert.equal(restored.home,true);assert.equal(restored.heroes.length,1);assert.deepEqual(restored.heroes,restored.expected);assert.equal(restored.awaiting,false);assert.deepEqual(errors,[]);checks+=5;
  console.log(width,motion,'immediate navigation retains completed house and exact hero PASS');await p.close();
 }
 {
  const {p,errors}=await open(400);
  await p.evaluate(()=>{const a=window.__riddle,save=a.engine.save,build=a.engine.buildTown;window.saveCalls=0;window.buildCalls=0;let release;window.releaseSave=()=>release();a.engine.save=async()=>{window.saveCalls++;await new Promise(r=>release=r);return save();};a.engine.buildTown=async id=>{window.buildCalls++;return build(id);};});
  await p.locator('.town-tag').click();await p.waitForFunction(()=>window.saveCalls===1);
  const pending=await p.evaluate(()=>({awaiting:document.querySelector('main.camp').classList.contains('awaiting-home'),foundation:!document.querySelector('.town-foundation').hidden,busy:document.querySelector('.town-tag').getAttribute('aria-busy')}));
  assert.equal(pending.awaiting,true);assert.equal(pending.foundation,true);assert.equal(pending.busy,'true');checks+=3;
  await p.locator('.hit-staked').click();await p.locator('.town-tag').focus();await p.keyboard.press('Space');
  assert.equal(await p.evaluate(()=>window.buildCalls),1);assert.equal(await p.evaluate(()=>window.saveCalls),1);checks+=2;
  await p.evaluate(()=>window.releaseSave());await p.waitForFunction(()=>document.querySelector('.town-foundation').hidden);await p.waitForFunction(()=>document.querySelector('button.send')&&!document.querySelector('button.send').disabled);
  assert.equal(await p.evaluate(()=>window.saveCalls),1);checks++;
  // No test flush: the production build path must already have saved it.
  await p.evaluate(()=>history.replaceState(null,'',location.pathname));await p.reload();await p.waitForFunction(()=>window.__riddle?.booted);
  assert.equal(await p.evaluate(()=>window.__riddle.lineage.town.home),true);assert.deepEqual(errors,[]);checks+=2;
  console.log('Delayed save keeps build pending, deduplicated, then reloads completed home PASS');await p.close();
 }
 {
  const {p,errors}=await open(400);
  const saved=await p.evaluate(()=>localStorage.getItem('riddle.save'));
  await p.evaluate(()=>{window.saveCalls=0;const a=window.__riddle,save=a.engine.save;a.engine.save=async()=>{window.saveCalls++;return save();};a.engine.buildTown=async()=>{throw new Error('test construction refused');};});
  await p.locator('.town-tag').click();await p.waitForFunction(()=>document.querySelector('.town-tag').getAttribute('aria-busy')==='false');
  assert.equal(await p.evaluate(()=>window.__riddle.lineage.town.home),false);assert.equal(await p.evaluate(()=>window.__riddle.lineage.hero_slots.length),0);assert.equal(await p.evaluate(()=>window.saveCalls),0);assert.equal(await p.evaluate(()=>localStorage.getItem('riddle.save')),saved);assert.deepEqual(errors,[]);checks+=5;
  console.log('Refused construction retains empty town/save and releases guard PASS');await p.close();
 }
 console.log(checks,'manual construction persistence checks PASS');
}finally{await b.close();}
