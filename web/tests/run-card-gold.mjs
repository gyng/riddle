import {execFileSync} from 'node:child_process';import {readFileSync} from 'node:fs';import assert from 'node:assert/strict';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const source=readFileSync(new URL('./fixtures/earned-first-ascension-clear.json',import.meta.url),'utf8');
const b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900},reducedMotion:'reduce'}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?dev=1&fresh=1`);await p.waitForFunction(()=>window.__riddle?.booted);
 assert.equal(await p.evaluate(()=>window.__riddle.kind),'wasm');
 const checks=await p.evaluate(async()=>{
  const {clearCard}=await import('/src/ui/runclear.ts'),a=window.__riddle;let n=0;
  const check=(v,message)=>{if(!v)throw new Error(message);n++;};
  for(const [auto,porter,expected] of [[true,false,'Gold'],[true,true,'Gold'],[false,false,'chest'],[false,true,'purse'],[undefined,false,'chest']])for(const end of ['bank','return','death']){
   const lineage=structuredClone(a.lineage);lineage.town.auto_collect=auto;lineage.tree.nodes=lineage.tree.nodes.map(w=>({...w,state:w.id==='porter'?(porter?'done':'lit'):w.state}));
   const x={end,reached:8,carried:200,kept:end==='death'?80:140,secured:80,keep_pct:end==='death'?0:60,spent:0,spent_on:[],text:`${end} D8`};
   const card=clearCard({...a,lineage},x);document.body.append(card);
   check(card.querySelector('.rc-secured')?.dataset.secured==='80','secured stays exact');
   if(end==='death'){check(!card.querySelector('.rc-to'),'death has no invented destination');check(!card.querySelector('.rc-coins'),'death only shows secured gold');}
   else {check(card.querySelector('.rc-to')?.textContent===expected,'actual gold destination');check(card.querySelector('.rc-coins')?.dataset.gold==='140','home total stays exact');}
   card.remove();
  }return n;
 });
 assert.equal(await p.evaluate(async source=>{const n=JSON.parse(source);return window.__riddle.importSave(JSON.stringify({v:2,engine:source,loadout:n.loadout,last_seen:Date.now(),runs:['runs_banked','runs_returned','runs_died'].reduce((t,k)=>t+(n.lineage[k]??0),0)}));},source),true);
 await p.locator('.ending-town').click();await p.waitForSelector('.camp');
 const before=await p.evaluate(()=>{window.__riddle.runnerOn=false;return window.__riddle.engine.save();});
 await p.locator('.hero-row.selected:visible .hero-details').click();await p.locator('.hero-runs').click();
 const entry=p.locator('.run-entry:not(.sampled):not([data-tier="death"]) .re-body').first();await entry.click();await p.waitForSelector('.run-card-sheet');
 assert.equal(await p.locator('.run-card-sheet .rc-to').textContent(),'Gold');
 const id=Number(await p.locator('.run-card-sheet').getAttribute('data-run'));const rec=await p.evaluate(id=>window.__riddle.lineage.runs.find(r=>r.id===id),id);
 assert.equal(Number(await p.locator('.run-card-sheet .rc-coins').getAttribute('data-gold')),rec.gold);
 assert.equal(await p.evaluate(()=>window.__riddle.engine.save()),before);assert.equal(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.deepEqual(errors,[]);
 console.log(width,checks,'gold destination/amount/secured/old wire cases and earned historical card exact save PASS');await p.close();
}}finally{await b.close();}
