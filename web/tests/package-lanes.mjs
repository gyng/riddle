// Query reuse must preserve snapshot ordering, failures and independent replies.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 const page=await browser.newPage();await page.goto(`${url}?engine=fake&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 const checks=await page.evaluate(async()=>{
  const {packageMemo,twoLanes}=await import('/src/engine/lanes.ts');let n=0;
  const check=(v,label)=>{if(!v)throw new Error(label);n++;};
  const deferred=()=>{let resolve,reject;const promise=new Promise((r,j)=>{resolve=r;reject=j;});return {promise,resolve,reject};};
  let state='hero1',calls=0;const replies=[];
  const b={packageOptionsKey:async sims=>`${state}:${sims}`,packageOptions:()=>{calls++;const d=deferred();replies.push(d);return d.promise;}};
  const memo=packageMemo();
  const p=memo.run(b,[24]),q=memo.run(b,[24]);await new Promise(r=>setTimeout(r,0));check(calls===1,'concurrent identical keys share one query');
  replies[0].resolve([{id:'old',price:3}]);const result=await p;await q;result[0].price=999;
  check((await memo.run(b,[24]))[0].price===3,'caller cannot mutate cached reply');
  state='hero2';const r=memo.run(b,[24]);await new Promise(r=>setTimeout(r,0));check(calls===2,'changed key starts a new query');
  replies[1].reject(new Error('controlled'));await r.catch(()=>{});
  const retry=memo.run(b,[24]);await new Promise(r=>setTimeout(r,0));check(calls===3,'failed query retries');replies[2].resolve([{id:'new'}]);await retry;
  memo.clear();const rebuilt=memo.run(b,[24]);await new Promise(r=>setTimeout(r,0));check(calls===4,'native rebuild clears completed result');replies[3].resolve([]);await rebuilt;
  let oldCalls=0;check((await memo.run({packageOptions:async()=>{oldCalls++;return [1];}},[24]))[0]===1,'older bridge falls back');
  await memo.run({packageOptionsKey:async()=>{throw new Error('absent');},packageOptions:async()=>{oldCalls++;return [];}},[24]);check(oldCalls===2,'missing optional key falls back');
  // Real lane synchronization: a mutation during a pending query does not load
  // new state between that query's key and prices, including a second busy lane.
  let fgState='A',queryCount=0;const jobs=[],events=[];
  const fg={save:async()=>fgState,lineage:async()=>({}),setLook:async()=>{},setRules:async s=>{fgState=s;},packageOptions:async()=>[]};
  const bg=async()=>{let snapshot;return {load:async s=>{snapshot=s;events.push(`load:${s}`);},setRules:async s=>{snapshot=s;events.push(`rules:${s}`);},packageOptionsKey:async()=>snapshot,packageOptions:async()=>{queryCount++;const d=deferred();jobs.push({snapshot,...d});return d.promise;}};};
  const lanes=twoLanes(fg,bg,{mirror:true});
  const first=lanes.packageOptions(24);await new Promise(r=>setTimeout(r,0));
  await lanes.setRules('B');const second=lanes.packageOptions(24);await new Promise(r=>setTimeout(r,0));
  check(jobs[0].snapshot==='A','first query keeps original mirror');
  jobs[0].resolve([{id:'A'}]);check((await first)[0].id==='A','old reply belongs to old caller');
  await new Promise(r=>setTimeout(r,0));check(jobs[1].snapshot==='B','next query reads new foreground state');jobs[1].resolve([{id:'B'}]);check((await second)[0].id==='B','new caller receives new prices');
  await lanes.lineage();check((await lanes.packageOptions(24))[0].id==='B'&&queryCount===2,'read-only lineage refresh reuses complete prices');
  await lanes.setLook();check((await lanes.packageOptions(24))[0].id==='B'&&queryCount===2,'harmless mutation reloads then reuses complete prices');
  check(events.includes('load:B'),'full mutation synchronizes mirror before reuse');
  const other=twoLanes(fg,bg,{mirror:true});const isolated=other.packageOptions(24);await new Promise(r=>setTimeout(r,0));check(queryCount===3,'another engine isolates cache');jobs[2].resolve([]);await isolated;
  const bounded=packageMemo();let count=0;const immediate={packageOptionsKey:async k=>String(k),packageOptions:async()=>++count};
  for(let i=0;i<5;i++)await bounded.run(immediate,[i]);await bounded.run(immediate,[0]);check(count===6,'cache is bounded to four keys');
  return n;
 });console.log(`package-lanes: ${checks} checks PASS`);
}finally{await browser.close();}
