import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const checks=await p.evaluate(async()=>{
  const {heroRoster,heroPresence}=await import('/src/ui/heroes.ts');const a=window.__riddle;
  let checks=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);checks++;};
  const live={run_id:7,heir:2,depth:13,start:1,hp:24,max_hp:42,turn:100,activity:'combat'};
  const s={id:1,name:'Bloodline 1',hero_name:'Wren',heir:2,class:'fighter',look:'male',level:4,xp:100,next:200,state:'live',live,rest_s:0,legacy:{points:0,spent:0,upgrades:{}},notice:false};
  const other={...s,id:2,hero_name:'Yara',live:{...live,depth:18,activity:'returning'}};
  a.lineage={...a.lineage,selected_bloodline:1,hero_slots:[s,other]};
  const r=heroRoster(a);document.querySelector('.well-wrap').append(r.el);
  const text=id=>r.el.querySelector(`.hero-row[data-slot="${id}"] .hero-action`).textContent;
  check(text(1)==='D13 · In combat','combat');check(text(2)==='D18 · Heading home','other bloodline owns its phase');
  check(r.el.querySelector('.hero-action').title.includes('24/42 hp'),'HP detail');
  r.setPresence({...live,depth:14,turn:110,activity:'exploring'});check(text(1)==='D14 · Exploring','watch snapshot freshness');check(text(2)==='D18 · Heading home','does not alter other slot');
  r.setPresence({...live,depth:12,turn:105,activity:'combat'});check(text(1)==='D14 · Exploring','late same-run snapshot ignored');
  s.live={...live,turn:120,depth:15,activity:'returning'};r.paint();check(text(1)==='D15 · Heading home','newer core summary wins');
  s.live={...live,run_id:8,depth:1};r.paint();check(text(1)==='D1 · In combat','new run ignores old observation');
  s.heir=3;s.live={...live,heir:3,depth:1};r.paint();check(text(1)==='D1 · In combat','new heir ignores old observation');
  s.heir=2;s.live=live;r.setPresence({...live,depth:16,turn:130},true);check(text(1)==='D16 · Run ended','end presence');
  s.state='rests';s.rest_s=120;r.paint();check(text(1)==='Resting 2m','rest wins ended observation');
  s.rest_s=0;r.paint();check(text(1)==='Ready','zero rest is ready');
  s.state='waits';r.paint();check(text(1)==='Ready','ready');
  check(heroPresence({...s,state:'live',live:{...live,activity:undefined}}).text==='D13 · Delving','old wire never invents activity');
  check(heroPresence({...s,state:'live',live:null}).text==='Starting run','no invented starting depth');
  // Both slots can have the same run id/heir; late updates retain the mounted slot identity.
  s.state='live';s.live=live;a.lineage.selected_bloodline=2;
  r.setPresence({...live,depth:17,turn:140,activity:'exploring'});check(text(2)==='D18 · Heading home','selection cannot reattribute old snapshot');
  r.dispose();return checks;
 });
 await p.waitForTimeout(100);if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('overflow');
 if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'presence/epoch/freshness/compatibility/identity checks PASS');await p.close();
}}finally{await b.close();}
