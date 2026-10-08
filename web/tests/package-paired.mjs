// Blind c4705f9 (A, B: `compare outcomes` said `all similar` nearly every time): a price read send by send on the same seeds calls a
// clear split (`better 7/8`), says `same` / `close` inside the noise, and the summary names what was compared.
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
   const {openPackages,priceOf,signP,compareSummary}=await import('/src/ui/packages.ts');const {closeAllSheets}=await import('/src/ui/sheet.ts');
   const real=window.__riddle;let checks=0;const tick=()=>new Promise(r=>setTimeout(r,30));
   const check=(v,label)=>{if(!v)throw new Error(label);checks++;};
   const base={action:'equip',slot:0,price:0,past:.3,bank:.4,death:.2,reach:.5,mean:6,d_past:.1,d_bank:0,d_death:-.12,d_reach:0,d_mean:.9,d_wall:0};
   // the old band could not call a 12-point paired move at eight sends; the pairs can
   check(priceOf({...base}).good===null,'unpaired band still hides a small move');
   check(priceOf({...base,n:8,better:7,worse:0}).text==='better 7/8'&&priceOf({...base,n:8,better:7,worse:0}).good===true,'clear paired gain called');
   check(priceOf({...base,n:8,better:0,worse:6}).text==='worse 6/8'&&priceOf({...base,n:8,better:0,worse:6}).good===false,'clear paired loss called');
   check(priceOf({...base,n:8,better:3,worse:2}).text==='close'&&priceOf({...base,n:8,better:3,worse:2}).good===null,'a mixed split is close');
   check(priceOf({...base,n:8,better:0,worse:0}).text==='same','identical sends read same');
   check(Math.abs(signP(5,0)-.0625)<1e-9&&Math.abs(signP(3,3)-1)<1e-9&&signP(0,0)===1,'two-sided sign test');
   check(compareSummary([{...base,id:'a',n:8,better:7,worse:0}])==='8 paired runs','clear summary names the sample');
   check(compareSummary([{...base,id:'a',n:8,better:3,worse:2}])==='noise · 8 runs','noise summary names the sample');
   check(compareSummary([{...base,id:'a',n:8,better:0,worse:0},{...base,id:'b',n:6,better:0,worse:0}])==='identical · 6 runs','identical summary names the sample');
   check(compareSummary([{...base,id:'a'}])==='all similar','older core keeps its words');
   // the sheet: three styles, one clearly better, one identical, one mixed
   const L=structuredClone(real.lineage);L.packages.stance='steady';
   L.packages.all=L.packages.all.map(p=>({...p,owned:p.kind==='stance'||['steady'].includes(p.id)}));
   const ids=L.packages.all.filter(p=>p.kind==='stance'&&p.id!=='steady').map(p=>p.id).slice(0,3);
   check(ids.length===3,'three styles to compare');
   const app={lineage:L,rules:structuredClone(real.rules),loadout:[],lastForecast:null,mutate:async()=>true};
   const splits=[[0,0],[2,1],[7,0]];
   app.engine={packageOptions:async()=>ids.map((id,i)=>({...base,id,n:8,better:splits[i][0],worse:splits[i][1]})),equipPackage:async()=>app.lineage};
   closeAllSheets();openPackages(app);document.querySelector('.pkg-compare').click();
   if(document.querySelector('[data-change-kind="stance"]')?.getAttribute('aria-expanded')==='false')document.querySelector('[data-change-kind="stance"]').click();
   await tick();await tick();
   const chips=[...document.querySelectorAll('.pkg-sec[data-kind="stance"] .pkg.alt')];
   const priceText=id=>document.querySelector(`.pkg-sec[data-kind="stance"] .pkg.alt[data-pkg="${id}"] .pkg-price`)?.textContent;
   check(chips[0]?.dataset.pkg===ids[2],'the clear gain leads');
   check(priceText(ids[2])==='better 7/8','clear gain painted');
   check(priceText(ids[0])==='same'&&priceText(ids[1])==='close','noise painted, never blank');
   check(document.querySelector('.pkg-estimate')?.textContent==='8 paired runs','summary names what was compared');
   check(/better 7 · worse 0 of 8/.test(document.querySelector(`.pkg.alt[data-pkg="${ids[2]}"] .pkg-price`).title),'price tooltip carries the split');
   check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');
   closeAllSheets();return checks;
  });
  total+=checks;console.log(`package-paired ${width}: ${checks} checks PASS`);await page.close();
 }
 if(errors.length)throw new Error(errors.join('\n'));
 console.log(`package-paired: ${total} checks PASS`);
}finally{await browser.close();}
