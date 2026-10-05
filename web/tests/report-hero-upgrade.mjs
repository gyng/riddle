import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const checks=await p.evaluate(async()=>{
  const a=window.__riddle,{closeAllSheets}=await import('/src/ui/sheet.ts');let checks=0,calls=0;const check=(ok,text)=>{if(!ok)throw Error(text);checks++;};
  const report={elapsed_s:28800,runs:17,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],deepest:7,gold:{home:986}};
  const offer={id:'health',rank:0,cap:10,price:10,effect:'+5 hp',affordable:true};
  a.lineage={...a.lineage,selected_bloodline:1,bloodline:{points:33,spent:0,upgrades:{}},legacy_upgrades:[offer]};
  const stub=async()=>{calls++;return a.lineage;};a.engine.upgradeHero=stub;a.go({kind:'report',report});
  const action=()=>document.querySelector('.report-upgrade');check(action()?.textContent.includes('Bloodline 1 · 33 Legacy'),'selected bloodline balance');check(document.querySelectorAll('.report-basics .tile').length===3,'simple result trio retained');check(document.querySelector('.report-details').hidden,'details collapsed');
  action().click();check(!!document.querySelector('.hero-sheet')&&document.querySelector('.hero-legacy').textContent.includes('33'),'opens existing hero upgrades');check(calls===0,'opening never spends');closeAllSheets();
  for(const test of ['locked','maxed','missing','bridge']){
   a.lineage.legacy_upgrades=test==='missing'?undefined:[{...offer,rank:test==='maxed'?10:0,affordable:false}];if(test==='bridge'){a.lineage.legacy_upgrades=[offer];a.engine.upgradeHero=undefined;}
   await a.afterLineage();check(!action(),test+' unavailable action absent');a.engine.upgradeHero=stub;
  }
  a.lineage={...a.lineage,selected_bloodline:2,bloodline:{points:77,spent:0,upgrades:{}},legacy_upgrades:[offer]};await a.afterLineage();check(action()?.textContent.includes('Bloodline 2 · 77 Legacy'),'selected slot updates without stale points');
  a.lineage={...a.lineage,legacy_upgrades:[{...offer,affordable:false}],live:{depth:6,turn:1}};await a.afterLineage();check(!action(),'core unavailable away offer respected');
  a.lineage={...a.lineage,live:null,legacy_upgrades:[offer]};await a.afterLineage();const old=document.querySelector('.report-upgrade-host'),before=old.textContent;a.go({kind:'camp'});a.lineage={...a.lineage,bloodline:{points:99,spent:0,upgrades:{}},legacy_upgrades:[offer]};await a.afterLineage();check(old.textContent===before&&!old.isConnected,'disposed report subscription does not repaint');
  a.lineage={...a.lineage,bloodline:undefined,hero_legacy:undefined,legacy_upgrades:[offer]};a.go({kind:'report',report});check(!action(),'missing balance never invents zero Legacy');
  const {renderConsole,tile,portrait,gem}=await import('/src/ui/frame.ts');document.querySelector('.report .console').replaceWith(renderConsole({portrait:portrait(a).el,gem:gem({label:'Town',onclick:()=>{}}),compact:true,tiles:['Clear','deepest','gold','enemy guide'].map((label,i)=>tile({id:'qa'+i,label,icon:'gold',onclick:()=>{}}))}).el);
  await document.fonts.ready;await new Promise(r=>setTimeout(r,300));
  const tiles=[...document.querySelectorAll('.console .cmd button')];check(!document.querySelector('.console .cmd .empty'),'report fillers removed');
  for(const button of tiles){const text=button.querySelector('.tl'),range=document.createRange();range.selectNodeContents(text);const b=button.getBoundingClientRect();check([...range.getClientRects()].every(r=>r.left>=b.left-1&&r.right<=b.right+1&&r.top>=b.top-1&&r.bottom<=b.bottom+1)&&b.width>=44&&b.height>=44,'footer label/touch bounds');}
  check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');return checks;
 });console.log(width,checks,'report upgrade/lifecycle/footer checks PASS');await p.close();
}}finally{await b.close();}
