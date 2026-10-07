import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const n=await p.evaluate(async()=>{
  const a=window.__riddle,{openHero}=await import('/src/ui/town.ts'),{closeAllSheets,closeSheet}=await import('/src/ui/sheet.ts');let n=0,calls=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);n++;};
  a.lineage={...a.lineage,look:'male',class:'fighter',selected_bloodline:1,gold:500,bloodline:{points:7,spent:0,upgrades:{}},hero_slots:[{id:1,name:'Bloodline 1',hero_name:'Wren',look:'male',class:'fighter'}]};
  a.engine.lineage=async()=>a.lineage;
  a.engine.setLook=async look=>{calls++;a.lineage={...a.lineage,look,hero_slots:a.lineage.hero_slots.map(s=>({...s,look}))};return a.lineage;};
  openHero(a);const hero=document.querySelector('.sheet-wrap .hero-sheet'),button=hero.querySelector('.hero-appearance');check(button.textContent==='Appearance'&&!button.disabled,'discoverable available action');
  hero.querySelector('.hero-history').open=true;button.click();let picker=document.querySelector('.sheet-wrap:not([hidden]) .look-sheet');
  check(picker?.querySelectorAll('button.look').length===3&&picker.textContent.includes('Appearance'),'existing three-portrait picker');
  picker.querySelector('[data-look=cat]').click();await new Promise(r=>setTimeout(r,300));
  check(calls===1&&a.lineage.look==='cat',`one core choice: ${calls}/${a.lineage.look}`);check(hero.isConnected&&!hero.closest('.sheet-wrap').hidden&&!document.querySelector('.sheet-wrap .look-sheet'),'returns to same parent');
  check(hero.querySelector('.hero-window-face').dataset.art==='hero_fighter_cat','parent portrait immediately updated');check(hero.querySelector('.hero-history').open,'expanded Details preserved');
  check(a.lineage.gold===500&&a.lineage.bloodline.points===7&&a.lineage.class==='fighter','no currency/class change');
  hero.querySelector('.hero-appearance').click();document.querySelector('.sheet-wrap:not([hidden]) [data-look=cat]').click();await new Promise(r=>setTimeout(r,100));check(calls===1,'unchanged look makes no core write');
  hero.querySelector('.hero-appearance').click();closeSheet();check(!hero.closest('.sheet-wrap').hidden&&a.lineage.look==='cat','dismiss picker returns without changing appearance');
  a.lineage.hero_slots[0].hero_name='Niall';await a.afterLineage();check(hero.querySelector('.hero-line').textContent.includes('Niall'),'parent identity updates without class/look change');
  const old=hero.innerHTML;closeAllSheets();a.lineage.look='female';await a.afterLineage();check(hero.innerHTML===old&&!hero.isConnected,'closed parent no longer subscribes');
  a.engine.setLook=undefined;openHero(a);check(document.querySelector('.sheet-wrap .hero-appearance').disabled,'missing bridge disables appearance');closeAllSheets();
  a.engine.setLook=async()=>a.lineage;a.lineage.live={depth:5,turn:1};openHero(a);check(!document.querySelector('.sheet-wrap .hero-appearance').disabled&&document.querySelector('.sheet-wrap .hero-class').disabled,'cosmetic choice allowed while away');
  await document.fonts.ready;await new Promise(r=>setTimeout(r,300));const b=document.querySelector('.sheet-wrap .hero-appearance').getBoundingClientRect();check(b.width>=44&&b.height>=44,`touch target: ${JSON.stringify(b.toJSON())}`);check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');closeAllSheets();return n;
 });console.log(width,n,'hero appearance route/lifecycle checks PASS');await p.close();
}}finally{await b.close();}
