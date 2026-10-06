// Renderer/HUD clock checks. Synthetic state events; real action coverage lives in Rust.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try { for (const width of [320,400,1440]) {
 const page=await browser.newPage({viewport:{width,height:900}}),errors=[];
 page.on('pageerror',e=>errors.push(e.message));
 await page.goto(`${url}?engine=fake&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 const count=await page.evaluate(async()=>{
  const {ReplayState}=await import('/src/render/state.ts'),{gunStatus}=await import('/src/ui/gun-status.ts');
  const snapshot=await window.__riddle.engine.send();
  const loaded={item:900,kind:'short_gun',loaded:2,capacity:2,range:3,damage:[4,7],armour_piercing:0,reload_ticks:15,reload_left:0};
  const t=snapshot.turn, r=new ReplayState(),g=gunStatus();document.body.append(g.el);
  let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;};
  r.load({...snapshot,hero:{...snapshot.hero,gun:loaded}});
  r.apply([{k:'gun',t:t+2,state:{...loaded,loaded:0}},{k:'gun',t:t+3,state:{...loaded,loaded:0,reload_ticks:8,reload_left:8,reload_until:t+11}},
   {k:'gun',t:t+11,state:loaded},{k:'gun',t:t+12,state:null},{k:'gun',t:t+13,state:{...loaded,aiming:true}}]);
  check(r.gun.loaded===2,'queued future does not spend rounds');
  r.seek(t+2);g.paint(r.gun,r.clock);check(g.el.textContent==='0/2 · Empty','burst spends both at picture clock');
  r.seek(t+7);g.paint(r.gun,r.clock);check(g.el.dataset.state==='reload','saved deadline remains active');
  check(g.el.querySelector('.gun-reload-fill').style.width==='50%','actual eight tick reload halfway');
  r.seek(t+11);g.paint(r.gun,r.clock);check(g.el.textContent==='2/2 · Ready','played completion restores chambers');
  r.seek(t+12);g.paint(r.gun,r.clock);check(g.el.hidden,'sidearm hides gun');
  r.seek(t+13);g.paint(r.gun,r.clock);check(g.el.textContent==='2/2 · Aim','played aim cue');
  r.seek(t+1);g.paint(r.gun,r.clock);check(g.el.textContent==='2/2 · Ready','backward seek restores loaded snapshot');
  check(g.el.getAttribute('aria-label').includes('4–7 damage')&&g.el.getAttribute('aria-label').includes('range 3 tiles'),'Rust metadata tooltip');
  const {openTip,closeTip}=await import('/src/ui/tips.ts');openTip(g.el,true);
  check(document.querySelector('#kw-tip').textContent.includes('1.5s reload'),'keyboard/touch tooltip uses readable seconds');closeTip();
  r.load(snapshot);g.paint(r.gun,r.clock);check(g.el.hidden,'ordinary hero has no gun chrome');
  r.apply([{k:'rule',t:t+1,row:0,verb:{v:'gunner_tactic'},text:'Gunner rat'}]);r.seek(t+1);
  check(!r.caption,'automatic handler never shows implementation caption');
  r.apply([{k:'rule',t:t+2,row:1,verb:{v:'fire'},text:'fire rat'}]);r.seek(t+2);
  check(!!r.caption,'explicit fire row retains action caption');
  r.apply([{k:'callout',t:t+3,text:'Gunner ✗ no target'}]);r.seek(t+3);check(!r.callout,'ordinary gun fallback never warns over the map');
  r.apply([{k:'callout',t:t+4,text:'fire ✗ no target'}]);r.seek(t+4);check(!!r.callout,'explicit fire refusal remains observable');
  const {verbLabel}=await import('/src/ui/tokens.ts');check(verbLabel({v:'gunner_tactic'})==='Gun handling','readable meter/rule label');
  const {combatLogEvents,gunShotText}=await import('/src/ui/combat-log.ts');
  const actual=[{k:'rule',t,row:0,verb:{v:'gunner_tactic'},text:'always → Gunner'},
   {k:'rule',t,row:1,verb:{v:'fire'},text:'chosen fire'},
   {k:'callout',t,text:'aim steady'},{k:'callout',t,text:'reloading'},{k:'callout',t,text:'loaded'},
   {k:'callout',t,text:'unrelated flavour'},
   {k:'attack',t,src:snapshot.hero.id,dst:99,hit:true,dmg:7,verb:'close_burst'},
   {k:'hurt',t,id:99,dmg:7,cause:'hero'}];
  const log=combatLogEvents(actual,snapshot.hero.id,new Map());
  check(!log.some(e=>e.k==='rule'&&e.verb.v==='gunner_tactic'),'automatic handler removed from plain log');
  check(log.some(e=>e.k==='rule'&&e.verb.v==='fire'),'explicit chosen fire retained in log');
  check(log.filter(e=>e.k==='callout').length===3,'actual aim/reload/completion retained without flavour flood');
  check(log.filter(e=>e.k==='attack').length===1&&!log.some(e=>e.k==='hurt'),'burst damage remains once');
  check(gunShotText(log.find(e=>e.k==='attack').verb)==='burst','actual burst is named');
  g.el.remove();return n;
 });
 if(errors.length)throw Error(errors.join('\n'));console.log(width,count,'gun picture-clock/seek/HUD checks PASS');await page.close();
} } finally {await browser.close();}
