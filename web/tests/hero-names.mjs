import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const count=await p.evaluate(async()=>{
  const a=window.__riddle;let count=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);count++;};
  const {heroRoster}=await import('/src/ui/heroes.ts'),{openHero}=await import('/src/ui/town.ts'),{openChronicle,keptDeath}=await import('/src/ui/chronicle.ts'),{closeAllSheets}=await import('/src/ui/sheet.ts');
  a.lineage={...a.lineage,selected_bloodline:1,hero_slots:[{id:1,name:'Bloodline 1',hero_name:'Maren Thorn',heir:2,class:'fighter',level:3,xp:144,next:540,state:'waits',rest_s:0,legacy:{points:32,spent:0,upgrades:{}},notice:true}],hero_legacy:[{heir:1,name:'Lark Thorn',points:0,runs:3,best_depth:5,class:'fighter'},{heir:2,name:'Maren Thorn',points:32,runs:16,best_depth:7,class:'fighter'}],chronicle:['♟1 · fighter · D5'],graveyard:[{heir:1,death_id:77}]};
  const roster=heroRoster(a);document.querySelector('.well-wrap').append(roster.el);
  check(roster.el.querySelector('.hero-info b').textContent==='Maren Thorn','person named first');check(roster.el.querySelector('.hero-class-name').textContent==='Bloodline 1 · fighter','persistent slot and class preserved');check(roster.el.querySelector('.hero-jump').getAttribute('aria-label').includes('2nd heir'),'ordinal retained in accessible identity');
  openHero(a);check(document.querySelector('.sheet-wrap .hero-line').textContent.includes('Maren Thorn')&&document.querySelector('.sheet-wrap .hero-line').textContent.includes('fighter'),'same name/class in hero menu');await document.fonts.ready;await new Promise(r=>setTimeout(r,300));
  const line=document.querySelector('.sheet-wrap:not([inert]) .hero-line');const edge=line.getBoundingClientRect();check([...line.children].every(e=>{const r=e.getBoundingClientRect();return r.left>=edge.left-.5&&r.right<=edge.right+.5;}),'full name and class fit hero menu');closeAllSheets();
  openChronicle(a);check(document.querySelector('.sheet-wrap .cline').textContent.includes('Lark Thorn'),'past hero named only in Chronicle');check(keptDeath(a.lineage,a.lineage.chronicle[0])===77,'raw heir marker still links kept death');closeAllSheets();
  delete a.lineage.hero_slots[0].hero_name;roster.paint();check(roster.el.querySelector('.hero-info b').textContent==='Bloodline 1','old wire fallback preserved');
  a.lineage.hero_slots[0].hero_name='Maren Thorn';roster.paint();await document.fonts.ready;await new Promise(r=>setTimeout(r,250));
  check(roster.el.querySelector('.hero-info b').getBoundingClientRect().right<=roster.el.getBoundingClientRect().right,'name fits');check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');roster.dispose();return count;
 });console.log(width,count,'hero identity checks PASS');await p.close();
}}finally{await b.close();}
