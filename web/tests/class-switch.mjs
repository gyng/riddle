// Actual earned save: class replies must be adopted before the UI writes rules again.
import {execFileSync} from 'node:child_process';
import {readFileSync,mkdirSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const engine=readFileSync(new URL('./fixtures/earned-gunner-home.json',import.meta.url),'utf8');
const shots=process.env.RIDDLE_SHOTS;if(shots)mkdirSync(shots,{recursive:true});
const browser=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await browser.newPage({viewport:{width,height:900},reducedMotion:'reduce'}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?dev=1&fresh=1&seed=47`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async engine=>{const a=window.__riddle;if(a.kind!=='wasm'||!await a.importSave(JSON.stringify({v:2,engine,loadout:[],last_seen:Date.now(),runs:0})))throw Error('earned real import');a.insertRow({conds:[{k:'hp>',n:90}],verb:{v:'rest'}},0);},engine);
 await p.waitForFunction(async()=>{const a=window.__riddle,L=await a.engine.lineage();return a.rules.rows[0]?.conds[0]?.k==='hp>'&&JSON.stringify(a.rules.rows)===JSON.stringify(L.sets[L.active_set].rows);});
 const preserved=await p.evaluate(()=>window.__riddle.rules.rows.filter(r=>r.origin?.startsWith('drill:')||r.origin==='player'));
 await p.evaluate(()=>{const a=window.__riddle,original=a.engine.setClass.bind(a.engine);window.classReplies=[];window.classPaints=[];a.engine.setClass=async cls=>{const L=await original(cls);window.classReplies.push({cls,rows:structuredClone(L.sets[L.active_set].rows)});return L;};a.onChange(()=>{const reply=window.classReplies.at(-1);if(reply&&a.lineage.class===reply.cls)window.classPaints.push({cls:reply.cls,equal:JSON.stringify(a.rules.rows)===JSON.stringify(reply.rows)});});});
 for(const cls of ['fighter','gunner']){
  await p.locator('.hero-details:visible').first().click();await p.locator('.hero-class').click();await p.locator('.sheet-wrap .class-row>button').filter({hasText:new RegExp(`^${cls} L`)}).click();
  await p.waitForFunction(cls=>window.__riddle.lineage.class===cls,cls);
  const result=await p.evaluate(async()=>{const a=window.__riddle,L=await a.engine.lineage();return {ownRows:a.ownRows(),editorGun:[...document.querySelectorAll('.editor .row[data-i]')].filter(e=>e.textContent.includes('Gun handling')).length,rows:a.rules.rows,core:L.sets[L.active_set].rows,reply:window.classReplies.at(-1).rows,count:window.classReplies.length,paints:window.classPaints.filter(x=>x.cls===L.class)};});
  assert.deepEqual(result.rows,result.reply);assert.deepEqual(result.rows,result.core);assert.equal(result.count,cls==='fighter'?1:2);assert.ok(result.paints.length>0&&result.paints.every(x=>x.equal));
  assert.equal(result.rows.filter(r=>r.verb.v==='gunner_tactic').length,cls==='gunner'?1:0);
  assert.equal(result.editorGun,cls==='gunner'?1:0,'rendered editor reflects the class reply');
  assert.equal(result.ownRows,1,'generated class handling never consumes a player rule slot');
  if(cls==='gunner')assert.equal(result.rows.find(r=>r.verb.v==='gunner_tactic').origin,'class:gunner');
  assert.deepEqual(result.rows.filter(r=>r.origin?.startsWith('drill:')||r.origin==='player'),preserved);
  await p.keyboard.press('Escape');await p.waitForFunction(()=>!document.querySelector('.sheet-wrap'));
  const inspected=await p.evaluate(()=>window.__riddle.engine.save());
  await p.locator('.cmd [data-tile=edit]:visible').click();
  await p.locator('.hero-return:visible').waitFor();
  if(shots){await p.waitForFunction(()=>!document.querySelector('.sheet-ghost'));await p.screenshot({path:`${shots}/${cls}-${width}.png`});}
  await p.locator('.hero-return:visible').click();
  assert.equal(await p.evaluate(()=>window.__riddle.editing),false);
  assert.equal(await p.locator('.hero-details:visible').first().isVisible(),true);
  assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),inspected);
 }
 const before=await p.evaluate(async()=>({save:await window.__riddle.engine.save(),rows:window.__riddle.rules.rows}));
 assert.equal(await p.evaluate(()=>window.__riddle.setClass('unknown-class')),false);
 assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before.save);assert.deepEqual(await p.evaluate(()=>window.__riddle.rules.rows),before.rows);
 await p.evaluate(()=>window.__riddle.flush());await p.reload();await p.waitForFunction(()=>window.__riddle?.booted);
 assert.equal(await p.evaluate(()=>window.__riddle.lineage.class),'gunner');assert.deepEqual(await p.evaluate(()=>window.__riddle.rules.rows),before.rows);assert.deepEqual(errors,[]);
 // Controlled presentation case: the return route precedes extra set-tab access.
 const earlySave=await p.evaluate(async()=>{const a=window.__riddle,s=await a.engine.save();a.lineage={...a.lineage,seed:47000+innerWidth,heir:1};a.go({kind:'camp'});return s;});
 await p.locator('.cmd [data-tile=edit]:visible').click();assert.equal(await p.locator('.hero-return:visible').isVisible(),true);assert.equal(await p.locator('.tabs .tab').count(),0);
 await p.locator('.hero-return').click();assert.equal(await p.evaluate(()=>window.__riddle.editing),false);assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),earlySave);
 console.log(width,'owned class reply/first paint/no stale gun row/player+drills/one-call/refusal/exact reload PASS');await p.close();
}}finally{await browser.close();}
