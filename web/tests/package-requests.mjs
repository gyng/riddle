// Controlled delayed replies exercise the real package sheet's request lifecycle.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();let total=0;const errors=[];
try{
 for(const width of [400,1440]){
  const page=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:1});
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
  const checks=await page.evaluate(async()=>{
   const {openPackages}=await import('/src/ui/packages.ts');const {closeAllSheets}=await import('/src/ui/sheet.ts');
   const real=window.__riddle;const pending=[];let checks=0;
   const tick=()=>new Promise(r=>setTimeout(r,20));
   const check=(v,label)=>{if(!v)throw new Error(label);checks++;};
   const L=structuredClone(real.lineage);L.packages.stance='steady';
   L.packages.all=L.packages.all.map(p=>({...p,owned:['steady','guarded'].includes(p.id)}));
   const app={lineage:L,rules:structuredClone(real.rules),loadout:[],lastForecast:null};
   const query=()=>new Promise((resolve,reject)=>pending.push({resolve,reject}));
   app.engine={packageOptions:query,equipPackage:async(id)=>({...app.lineage,packages:{...app.lineage.packages,stance:id}})};
   app.mutate=async(fn)=>{app.lineage=await fn();return true;};
   const open=async(a=app)=>{closeAllSheets();openPackages(a);await tick();};
   const answer=(id='guarded',death=false)=>[{id,action:'equip',slot:0,price:0,past:death ? 0.1 : 0.9,bank:.1,death:.1,reach:.1,mean:1,d_past:death?0:.8,d_bank:0,d_death:death?-.8:0,d_reach:0,d_mean:0,d_wall:0}];
   const price=()=>document.querySelector('.sheet-wrap .pkg-panel .pkg.alt .pkg-price')?.textContent??'';
   await open();await open();await open();check(pending.length===1,'same snapshot must start one search');
   app.lineage={...app.lineage,class:'ranger',selected_bloodline:2};await open();check(pending.length===2,'another hero/class must remeasure');
   pending[1].resolve(answer('guarded',true));await tick();check(price()==='death −80','new hero prices paint');
   pending[0].resolve(answer());await tick();check(price()==='death −80','late old reply cannot paint');
   await open();check(pending.length===2&&price()==='death −80','late old reply cannot overwrite cached result');
   app.lineage={...app.lineage,hero_legacy:[{heir:1,points:0,spent:3,upgrades:{health:1}}]};await open();check(pending.length===3,'Legacy snapshot must remeasure');pending[2].resolve(answer());await tick();
   app.rules={...app.rules,name:'edited'};await open();check(pending.length===4,'rules edit must remeasure');pending[3].resolve(answer());await tick();
   app.loadout.push(123);await open();check(pending.length===5,'loadout edit must remeasure');pending[4].resolve(answer());await tick();
   const other={...app};await open(other);check(pending.length===6,'separate app must not reuse prices');pending[5].resolve(answer('guarded',true));await tick();
   await open(app);check(pending.length===6&&price()==='past +80','separate app reply cannot overwrite this app');
   app.lineage={...app.lineage};await open();check(pending.length===7,'refreshed lineage must remeasure');pending[6].reject(new Error('controlled failure'));await tick();await tick();check(pending.length===7,'failure must not create a repaint retry loop');
   await open();check(pending.length===8,'reopening after failure retries');pending[7].resolve(answer());await tick();check(price()==='past +80','retry result paints');
   app.engine={...app.engine};await open();check(pending.length===9,'another engine must not reuse prices');pending[8].resolve(answer());await tick();
   document.querySelector('.sheet-wrap .pkg.alt[data-pkg="guarded"]').click();await tick();check(app.lineage.packages.stance==='guarded','package mutation adopts new snapshot');check(pending.length===10,'mutation while open remeasures');check(price()==='','old prices clear on mutation');
   pending[9].resolve(answer('steady',true));await tick();check(price()==='death −80','new mutation prices paint');
   closeAllSheets();return checks;
  });
  total+=checks;console.log(`package-requests ${width}: ${checks} checks PASS`);await page.close();
 }
 if(errors.length)throw new Error(errors.join('\n'));
 console.log(`package-requests: ${total} checks PASS`);
}finally{await browser.close();}
