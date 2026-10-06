import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?fresh=1&engine=fake&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const before=await p.evaluate(()=>{window.__riddle.runnerOn=false;return window.__riddle.engine.save();});
 await p.evaluate(async()=>{
  const {enemyHost}=await import('/src/ui/enemy-tips.ts');
  const catalogue=[{id:'armoured',mask:1,name:'Armoured',effect:'+1 armour',counter:'Poison or fire'},
   {id:'regenerating',mask:4,name:'Regenerating',effect:'+1 HP each second while awake and unpoisoned',counter:'Poison or sustained damage'},
   {id:'shielded',mask:0,name:'Shielded',effect:'+2 armour unless stunned or paralysed',counter:'Stun or paralyse'},
   {id:'frenzied',mask:0,name:'Frenzied',effect:'+2 damage and +3 speed at half HP or lower',counter:'Slow or burst damage'},
   {id:'leeching',mask:0,name:'Leeching',effect:'Melee hits restore up to2 HP unless poisoned',counter:'Poison or fight at range'},
   {id:'tight_mirror',mask:0,name:'Quick mirror',effect:'Second repeated attack reflects',counter:'Alternate attacks'}];
  const L={facts:[],ledger:[{kind:'mirror_king',seen:true}],walls:[{boss:'mirror_king',depth:33,slain:true}]};
  window.__encounter={modifiers:{tier:3,affixes:5,elite:'shielded',tight_mirror:true},modifier_catalogue:catalogue,alive:true};
  const host=enemyHost(document.createElement('span'),'mirror_king',L,false,()=>window.__encounter);host.id='encounter-host';host.textContent='Mirror King';host.style.cssText='position:fixed;top:250px;left:12px;z-index:9999;background:#111;padding:8px';document.body.append(host);
 });
 const host=p.locator('#encounter-host'),tip=p.locator('#kw-tip');await host.hover();await tip.waitFor({state:'visible'});
 const text=await tip.textContent();assert.match(text,/Ascension 3.*Armoured.*Poison or fire.*Regenerating.*Poison or sustained damage.*Shielded.*Stun or paralyse.*Quick mirror.*Alternate attacks.*Encounter Alive/s);
 assert.doesNotMatch(text,/Frenzied|Defeated|Traits Regenerating/);
 const box=await tip.boundingBox();assert.ok(box.x>=0&&box.x+box.width<=width&&box.y>=0&&box.y+box.height<=900);
 await p.keyboard.press('Escape');assert.equal(await tip.isVisible(),false);
 await p.evaluate(()=>{window.__encounter={modifiers:{tier:1,affixes:1,elite:'frenzied'},modifier_catalogue:window.__encounter.modifier_catalogue,alive:true};});
 await host.click();assert.match(await tip.textContent(),/Ascension 1.*Frenzied.*at half HP or lower/s);assert.doesNotMatch(await tip.textContent(),/Regenerating|Shielded|Quick mirror/);
 await p.keyboard.press('Escape');await p.evaluate(()=>{window.__encounter={modifiers:{tier:6,affixes:6,elite:'leeching'},modifier_catalogue:window.__encounter.modifier_catalogue,alive:true};});
 await host.click();assert.match(await tip.textContent(),/Ascension 6.*Leeching.*Melee hits restore up to2 HP unless poisoned.*Poison or fight at range/s);assert.doesNotMatch(await tip.textContent(),/Shielded|Frenzied|Quick mirror/);
 await p.keyboard.press('Escape');await p.evaluate(()=>{window.__encounter={};});await host.click();assert.doesNotMatch(await tip.textContent(),/Ascension|Frenzied|Armoured/);assert.match(await tip.textContent(),/Defeated/);
 await p.keyboard.press('Escape');await p.evaluate(()=>{window.__encounter={modifiers:{tier:1,affixes:1}};});await host.click();assert.match(await tip.textContent(),/Ascension 1.*Details unavailable/s);assert.doesNotMatch(await tip.textContent(),/\+1 armour/);
 await p.keyboard.press('Escape');await p.evaluate(()=>{const b=document.createElement('button');b.id='encounter-before';b.textContent='Focus';document.querySelector('#encounter-host').before(b);});await p.locator('#encounter-before').focus();await p.keyboard.press('Tab');await tip.waitFor({state:'visible'});await p.keyboard.press('Escape');
 assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);assert.deepEqual(errors,[]);
 console.log(width,'encounter metadata/played state/counters/fallback/keyboard/viewport/read-only PASS');await p.close();
}}finally{await b.close();}
