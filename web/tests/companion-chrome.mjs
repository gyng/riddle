// Hidden companion controls: diagnostic fake save, actual production components.
import assert from 'node:assert/strict';
import {execFileSync} from 'node:child_process';
import {mkdirSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{encoding:'utf8',cwd:new URL('../../',import.meta.url)}).trim();
const shots=process.env.RIDDLE_QA_SHOTS;if(shots)mkdirSync(shots,{recursive:true});
const b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900},reducedMotion:'reduce'}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&systems=none`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  const a=window.__riddle,b=JSON.parse(a.exportSave()),e=JSON.parse(b.engine);
  const pet=(id,name)=>({id,kind:'jackal',name,level:2,tags:['pack','fast'],gen:1,rules:{rows:[]},max_rows:2,hp:12,max_hp:12});
  e.lineage.kennel=[pet(7001,'Thix'),pet(7002,'Skog')];e.lineage.party=[];e.lineage.eggs=[{id:9001,kind:'rat',tags:[],gen:1,hatch_in:2,from_loss:true}];
  b.engine=JSON.stringify(e);await a.importSave(JSON.stringify(b));
  const {renderParty}=await import('/src/ui/party.ts'),{openWindow}=await import('/src/ui/sheet.ts'),{h}=await import('/src/ui/dom.ts');
  const party=renderParty(a),off=a.onChange(()=>party.refresh());
  openWindow(()=>h('div',{class:'sheet-body'},party.el),{onClose:off});
 });
 const card=p.locator('.sheet-wrap:not(.under) .party .comp').filter({hasText:'Thix'});
 await card.locator('.comp-main').click();await p.waitForFunction(()=>window.__riddle.lineage.party.some(c=>c.id===7001));
 await card.locator('.comp-main').click();assert.equal(await p.evaluate(()=>window.__riddle.lineage.party.filter(c=>c.id===7001).length),1);
 await p.evaluate(async()=>{await document.fonts.ready;await Promise.all(document.querySelector('.sheet').getAnimations().map(a=>a.finished.catch(()=>{})));});
 const controls=await p.locator('.sheet-wrap:not(.under) .party button:visible').evaluateAll(es=>es.map(e=>{const r=e.getBoundingClientRect(),s=getComputedStyle(e);return{cls:e.className,h:r.height,w:r.width,frame:s.borderImageSource,radius:parseFloat(s.borderRadius),inside:r.left>=0&&r.right<=innerWidth};}));
 assert.ok(controls.every(r=>r.h>=44&&r.w>=44&&r.frame!=='none'&&r.radius<=2&&r.inside),JSON.stringify(controls));
 assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 if(shots)await p.screenshot({path:`${shots}/companions-${width}.png`});
 await card.locator('.grip').click();await p.waitForFunction(()=>!!document.querySelector('.game-window .editor'));await p.keyboard.press('Escape');
 await card.locator('.drop-pet').click();await p.waitForFunction(()=>!window.__riddle.lineage.party.some(c=>c.id===7001));
 const breed=p.locator('.sheet-wrap:not(.under) .party .row-label button');await breed.click();assert.equal(await breed.getAttribute('class'),'mini game-control on');await breed.click();
 const hatch=p.locator('.sheet-wrap:not(.under) .hatch');const saved=await p.evaluate(()=>window.__riddle.engine.save());await hatch.click();assert.equal(await hatch.innerText(),'ok $50');assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),saved);
 await p.evaluate(async()=>{(await import('/src/ui/sheet.ts')).closeAllSheets();window.__riddle.go({kind:'death',death:{run_id:0,depth:3,cause:'goblin_archer',margin:'3 hp short',verdict:'gap',baseline:0.25,trace:{turns:[]},patches:[],morgue:'Cold stone'}});});
 const more=p.locator('.death-more');await more.waitFor();await more.scrollIntoViewIfNeeded();
 const material=await more.evaluate(e=>({h:e.getBoundingClientRect().height,r:parseFloat(getComputedStyle(e).borderRadius),frame:getComputedStyle(e).borderImageSource}));assert.ok(material.h>=44&&material.r<=2&&material.frame!=='none',JSON.stringify(material));
 await more.click();assert.equal(await more.getAttribute('aria-expanded'),'true');await p.locator('.death-morgue').click();await p.locator('.sheet-wrap:visible').waitFor();await p.keyboard.press('Escape');
 if(shots)await p.screenshot({path:`${shots}/death-details-${width}.png`});
 assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.deepEqual(errors,[]);console.log(width,'companion/death material/layout/select/dismiss/rules/breed/hatch/details/morgue PASS');await p.close();
}}finally{await b.close();}
