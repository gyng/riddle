import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const [width,height] of [[320,568],[400,900],[1440,900]]){
 const p=await b.newPage({viewport:{width,height}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?fresh=1&engine=fake&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const before=await p.evaluate(async()=>{
  const a=window.__riddle;a.runnerOn=false;window.__descentCalls=[];window.__refuse=true;
  a.lineage={...a.lineage,ended:true,endgame:{tier:0,unlocked:1,cleared:0}};a.loadout=[1,2];a.runsSeen=23;
  a.engine.descentOffer=async tier=>{if(tier===1)await new Promise(r=>setTimeout(r,80));return{tier,hp_bonus_percent:tier*8,attack_bonus_percent:tier*4,stat_cap:1000000,affixes:tier?[{id:'armoured',mask:1,name:'Armoured',effect:'+1 armour',counter:'Poison or fire'}]:[],elites:[],elite_rate_denominator:tier?8:0,boss:tier?{id:'tight_mirror',mask:0,name:'Quick mirror',effect:'Second repeated attack reflects',counter:'Alternate attacks'}:null};};
  a.engine.beginDescent=async tier=>{window.__descentCalls.push(tier);if(window.__refuse)throw Error('refusal');return{...await a.engine.lineage(),ended:false,endgame:{tier,unlocked:1,cleared:0},selected_loadout:[1,2]};};
  a.go({kind:'ending'});return a.engine.save();
 });
 assert.equal(await p.locator('.ending-challenges').getAttribute('open'),null);
 await p.locator('.descent-choice').click();await p.locator('.descent-confirm:not([disabled])').waitFor();
 assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);assert.match(await p.locator('.descent-review').textContent(),/Enemy HP \+8%.*Armoured.*Poison or fire.*Second repeated attack reflects.*Keeps: hero.*Legacy.*class XP.*Restarts: depth/s);
 assert.equal(await p.locator('.descent-review .chips button').first().evaluate(el=>el===document.activeElement),true);
 const bounds=await p.locator('.descent-review').evaluate(el=>{const r=el.closest('.sheet').getBoundingClientRect();return{l:r.left,r:r.right,t:r.top,b:r.bottom}});assert.ok(bounds.l>=0&&bounds.r<=width&&bounds.t>=0&&bounds.b<=height);
 await p.getByRole('button',{name:'Easier dungeon',exact:true}).click();await p.locator('.descent-confirm:not([disabled])').waitFor();
 assert.equal(await p.locator('.descent-review h3').textContent(),'Original dungeon');assert.doesNotMatch(await p.locator('.descent-facts').textContent(),/Armoured|Mirror King/);
 await p.getByRole('button',{name:'Harder dungeon',exact:true}).click();await p.getByRole('button',{name:'Easier dungeon',exact:true}).click();await p.waitForTimeout(120);
 assert.equal(await p.locator('.descent-review h3').textContent(),'Original dungeon');assert.match(await p.locator('.descent-facts').textContent(),/HP \+0%/);
 await p.locator('.descent-tier').fill('99');await p.locator('.descent-tier').dispatchEvent('change');assert.equal(await p.locator('.descent-tier').inputValue(),'0');
 await p.keyboard.press('Escape');assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);assert.equal(await p.locator('.descent-choice').evaluate(el=>el===document.activeElement),true);
 await p.locator('.descent-choice').click();await p.locator('.descent-confirm:not([disabled])').waitFor();await p.locator('.descent-confirm').click();await p.waitForFunction(()=>document.querySelector('.descent-review .ascension-status')?.textContent==='Try again');
 assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);assert.deepEqual(await p.evaluate(()=>({loadout:window.__riddle.loadout,runs:window.__riddle.runsSeen,calls:window.__descentCalls})),{loadout:[1,2],runs:23,calls:[1]});
 await p.evaluate(()=>{window.__refuse=false;window.__riddle.engine.vocabulary=async()=>{throw Error('metadata unavailable')};});await p.locator('.descent-confirm').evaluate(el=>{el.click();el.click();});await p.waitForFunction(()=>window.__riddle.screen==='camp');
 assert.deepEqual(await p.evaluate(()=>({loadout:window.__riddle.loadout,runs:window.__riddle.runsSeen,calls:window.__descentCalls})),{loadout:[1,2],runs:23,calls:[1,1]});
 assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.deepEqual(errors,[]);console.log(`${width} preview/cancel/stale/locked/refusal/success/loadout/no-repeat/viewport PASS`);await p.close();
}}finally{await b.close();}
