// Real controller with delayed replies: previews are usable; refinement is explicit.
import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();const b=await launchBrowser();
try{for(const width of [400,1440]){const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);await p.waitForTimeout(200);
const n=await p.evaluate(async()=>{
const a=window.__riddle,old=a.engine;let count=0,previews=0,full=0;const pending=[];const tick=(ms=30)=>new Promise(r=>setTimeout(r,ms));const check=(v,s)=>{if(!v)throw new Error(s);count++;};
a.lineage={...a.lineage,town:{...a.lineage.town,home:true}};a.vsBase=null;
const sample={...await old.forecastEstimate(),sims:8,refined:false};
a.engine={...old,forecastEstimate:async()=>{previews++;return structuredClone(sample);},forecast:async()=>{throw new Error('unexpected full first pass');},forecastRefine:()=>{full++;return new Promise((resolve,reject)=>pending.push({resolve,reject}));}};
await a.emitForecast();await tick(1100);check(previews===1&&full===0,'preview cannot schedule refinement');check(a.lastForecast.sims===8&&!a.lastForecast.refined,'eight-sample result stands');check(!document.querySelector('.forecast .fc-settling'),'rough result has no endless settling indicator');
const x=a.refineForecast(),y=a.refineForecast();check(x===y&&full===0,'same-state explicit requests share their scheduled promise');await tick();check(full===1,'exactly one explicit refinement runs');
a.rulesChanged();await tick(100);check(previews===2,'edit requests a preview');check(full===1,'edit cannot launch a refinement');pending[0].resolve({...sample,sims:100,refined:true});await x;check(a.lastForecast.sims===8&&!a.lastForecast.refined,'stale refinement cannot paint after edit');
const failed=a.refineForecast();await tick();pending[1].reject(new Error('controlled failure'));await failed.catch(()=>{});check(!a.forecastRefining,'failed explicit request releases its busy state');
const retry=a.refineForecast();await tick();check(full===3,'failure permits explicit retry');pending[2].resolve({...sample,sims:100,refined:true});await retry;check(a.lastForecast.refined&&a.lastForecast.sims===100,'requested larger result paints');await a.emitForecast();check(previews===2,'same-state detailed result is retained without new automatic work');
a.lineage={...a.lineage};await a.emitForecast();check(previews===3&&!a.lastForecast.refined,'new lineage gets a new preview');const stale=a.refineForecast();await tick();a.lineage={...a.lineage,selected_bloodline:2};pending[3].resolve({...sample,sims:100,refined:true});await stale;check(!a.lastForecast.refined,'another hero rejects previous refinement');
a.lineage={...a.lineage};await a.emitForecast();const fresh=a.refineForecast();await tick();check(full===5,'another hero can request refinement after stale completion');pending[4].resolve({...sample,sims:100,refined:true});await fresh;
check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');a.engine=old;return count;
});console.log(width,n,'forecast lifecycle checks PASS');await p.close();}}finally{await b.close();}
