import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const count=await p.evaluate(async()=>{
  const a=window.__riddle,{heroRoster}=await import('/src/ui/heroes.ts');let count=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);count++;};
  const slot={id:1,name:'Bloodline 1',hero_name:'Wren',heir:1,class:'fighter',look:'male',level:1,xp:0,next:60,state:'waits',rest_s:0,legacy:{points:7,spent:0,upgrades:{}},notice:true};
  a.lineage={...a.lineage,selected_bloodline:1,hero_slots:[slot]};const r=heroRoster(a);document.querySelector('.well-wrap').append(r.el);
  for(const cls of ['fighter','rogue','ranger','caster'])for(const look of ['male','female','cat']){slot.class=cls;slot.look=look;r.paint();check(r.el.querySelector('.hero-thumb').dataset.art===`hero_${cls}_${look}`,`roster exact ${cls}/${look}`);}
  slot.class='fighter';slot.look=undefined;r.paint();check(r.el.querySelector('.hero-thumb').dataset.art==='hero_fighter','old wire class art fallback');
  slot.look='missing';r.paint();check(r.el.querySelector('.hero-thumb').dataset.art==='hero_fighter','missing art class fallback');r.dispose();
  const report={elapsed_s:28800,runs:16,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],deepest:7,gold:{home:974}};
  a.lineage={...a.lineage,class:'fighter',look:'male',bloodline:{points:7,spent:0,upgrades:{}},legacy_upgrades:[{id:'health',rank:0,cap:3,price:3,effect:'+3 HP',affordable:true}]};a.engine.upgradeHero=async()=>a.lineage;a.go({kind:'report',report});
  const face=()=>document.querySelector('.report-upgrade .icon-socket');check(face().dataset.art==='hero_fighter_male','report honors initial appearance');
  for(const look of ['female','cat']){a.lineage.look=look;await a.afterLineage();check(face().dataset.art===`hero_fighter_${look}`,'report repaints appearance without class/balance change');}
  await document.fonts.ready;await new Promise(r=>setTimeout(r,250));check(document.documentElement.scrollWidth<=innerWidth,'portrait choices do not overflow');return count;
 });console.log(width,count,'slot/report appearance checks PASS');await p.close();
}}finally{await b.close();}
