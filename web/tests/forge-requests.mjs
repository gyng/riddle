// Delayed Forge requests exercise actual cache and sheet ownership, without simulation noise.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try { for(const width of [400,1440]) {
 const page=await browser.newPage({viewport:{width,height:900}});
 await page.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);
 await page.waitForFunction(()=>window.__riddle?.booted);
 const count=await page.evaluate(async()=>{
  const {measureKit,openForge}=await import('/src/ui/forge.ts');
  const {closeAllSheets,openSheet}=await import('/src/ui/sheet.ts');
  const app=window.__riddle,original=app.engine;
  let count=0,calls=0; const pending=[];
  const tick=()=>new Promise(resolve=>setTimeout(resolve,20));
  const check=(value,name)=>{if(!value)throw Error(name);count++;};
  const rows=[{slot:'weapon',owned:0,steps:[],next:{price:100,affordable:true,depth:9,delta:0.2,pm:0}}];
  app.lineage={...app.lineage,kit:structuredClone(rows)};
  const makeEngine=()=>({...original,kitEstimates:()=>{calls++;return new Promise((resolve,reject)=>pending.push({resolve,reject}));}});
  app.engine=makeEngine();
  const first=measureKit(app);check(first===measureKit(app),'identical pending request deduplicated');
  await tick();check(calls===1,'one engine query');pending.at(-1).resolve(rows);await first;
  check(await measureKit(app)===rows&&calls===1,'completed answer reused');
  const clone={...app,lineage:structuredClone(app.lineage),rules:structuredClone(app.rules),loadout:[...app.loadout]};
  const other=measureKit(clone);await tick();check(calls===2,'Apps cannot share answers');pending.at(-1).resolve(rows);await other;
  app.engine=makeEngine();const replaced=measureKit(app);await tick();check(calls===3,'engine replacement invalidates answer');pending.at(-1).resolve(rows);await replaced;
  const saved={lineage:structuredClone(app.lineage),rules:structuredClone(app.rules),loadout:[...app.loadout]};
  const edits=[
   ['hero',()=>app.lineage.selected_bloodline=2],
   ['class',()=>app.lineage.class='mage'],
   ['traits',()=>app.lineage.trait='strong'],
   ['party',()=>app.lineage.party=[{id:71}]],
   ['Legacy',()=>app.lineage.hero_legacy=[{heir:1,points:3,runs:2,best_depth:4,class:'fighter',upgrades:{health:4}}]],
   ['loadout',()=>app.loadout=[71]],
   ['rules metadata',()=>app.rules.name='changed'],
   ['route',()=>app.rules.route=[2]],
   ['kit',()=>app.lineage.kit[0].owned=1],
   ['price',()=>app.lineage.kit[0].next.price=200],
   ['affordability',()=>app.lineage.kit[0].next.affordable=false],
  ];
  for(const [name,edit] of edits) {
   app.lineage=structuredClone(saved.lineage);app.sets[app.active]=structuredClone(saved.rules);app.loadout=[...saved.loadout];
   edit();const before=calls,ask=measureKit(app);await tick();check(calls===before+1,`${name} invalidates answer`);pending.at(-1).resolve(rows);await ask;
  }
  // Only four completed camps retained: the original has been evicted.
  app.lineage=structuredClone(saved.lineage);app.sets[app.active]=structuredClone(saved.rules);app.loadout=[...saved.loadout];
  let before=calls,ask=measureKit(app);await tick();check(calls===before+1,'completed cache bounded');pending.at(-1).resolve(rows);await ask;
  app.engine=makeEngine();ask=measureKit(app);await tick();pending.at(-1).reject(Error('controlled failure'));await ask.catch(()=>{});
  before=calls;ask=measureKit(app);await tick();check(calls===before+1,'failure permits retry');pending.at(-1).resolve(rows);await ask;
  app.engine={...original,kitEstimates:()=>{calls++;throw Error('synchronous failure');}};
  const sync=measureKit(app);check(sync instanceof Promise,'synchronous failures stay asynchronous');await sync.catch(()=>{});
  before=calls;await measureKit(app).catch(()=>{});check(calls===before+1,'synchronous failure releases pending entry');
  // Out-of-order old completion cannot replace the current state's answer.
  app.engine=makeEngine();const old=measureKit(app);await tick();const oldReply=pending.at(-1);
  app.lineage.selected_bloodline=3;const fresh=measureKit(app);await tick();pending.at(-1).resolve([{...rows[0],slot:'armour'}]);const current=await fresh;
  oldReply.resolve(rows);await old;before=calls;check(await measureKit(app)===current&&calls===before,'old completion cannot poison current cache');
  app.lineage.selected_bloodline=2;ask=measureKit(app);await tick();check(calls===before+1,'stale completion was not cached');pending.at(-1).resolve(rows);await ask;
  // Open sheet clears old state without starting work; stale success/failure cannot change controls.
  closeAllSheets();app.engine=makeEngine();const listeners=app.changeListeners.size,rulesListeners=app.rulesListeners.size;
  openForge(app);document.querySelector('.forge-details').open=true;
  const button=()=>document.querySelector('.forge-details button');
  button().click();await tick();const stale=pending.at(-1);app.lineage.selected_bloodline=4;app.emitChange();
  check(!button().disabled&&button().textContent==='Forecast','state change releases old busy control');
  check(!document.querySelector('.forge-forecasts').textContent,'state change clears estimate');
  before=calls;await tick();check(calls===before,'state change never auto-queries');
  button().click();await tick();pending.at(-1).resolve([{...rows[0],slot:'armour'}]);await tick();stale.resolve(rows);await tick();
  check(document.querySelector('.forge-forecasts').textContent.startsWith('armour'),'stale success cannot paint over new hero');
  app.lineage.selected_bloodline=5;app.emitChange();button().click();await tick();const failure=pending.at(-1);
  app.lineage.selected_bloodline=6;app.emitChange();button().click();await tick();failure.reject(Error('old failure'));await tick();
  check(button().disabled&&button().textContent==='Measuring…','stale failure cannot release a new request');
  pending.at(-1).resolve(rows);await tick();check(!button().disabled,'current request releases control');
  app.rules.name='new rules';for(const fn of app.rulesListeners)fn();check(!document.querySelector('.forge-forecasts').textContent,'rule changes clear old estimate');
  closeAllSheets();check(app.changeListeners.size===listeners&&app.rulesListeners.size===rulesListeners,'closing releases both subscriptions');
  let firstClosed=0,secondClosed=0;
  openSheet(()=>document.createElement('div'),{onClose:()=>firstClosed++});
  openSheet(()=>document.createElement('div'),{onClose:()=>secondClosed++});
  document.querySelector('.sheet-wrap:not([hidden]) .sheet-back').click();
  check(firstClosed===0&&secondClosed===1,'back releases only child subscription');
  closeAllSheets();check(firstClosed===1&&secondClosed===1,'parent close releases once');
  closeAllSheets();check(firstClosed===1&&secondClosed===1,'duplicate close is inert');
  check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');
  app.engine=original;return count;
 });
 console.log(width,count,'Forge request checks PASS');await page.close();
}}finally{await browser.close();}
