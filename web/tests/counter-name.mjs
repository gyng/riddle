// Blind 5331f40 (both raters: the floor chart's `counter: cadence` and the boss bar's `COUNTER: CADENCE` never named the tactic
// `Mirror rhythm`; `reflect read` beside the tactic `mirror read`): a counter that is a tactic package reads as the package's name
// wherever a counter is shown — the floor chart's wall line, the oath chip, the enemy tip, the report's counter lines.
import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{const p=await b.newPage({viewport:{width:400,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
const n=await p.evaluate(async()=>{
  const {counterName,namedCounters}=await import('/src/ui/counter-name.ts'),{wallCounter}=await import('/src/ui/forecast.ts'),{enemyHost}=await import('/src/ui/enemy-tips.ts');
  let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;};
  const L=structuredClone(window.__riddle.lineage);
  L.packages={...(L.packages??{stance:'steady'}),all:[{id:'cadence',name:'Mirror rhythm',kind:'tactic',level:1,runs:0,owned:false},{id:'reflect_read',name:'mirror read',kind:'tactic',level:1,runs:0,owned:true},{id:'boss_focus',name:'boss focus',kind:'tactic',level:1,runs:0,owned:true}]};
  const king={boss:'mirror_king',text:'cadence',row:{conds:[{k:'foe:tag',t:'boss'}],verb:{v:'tactic',a:'cadence'}}};
  const master={boss:'foundry_master',text:'reflect read',row:{conds:[{k:'foe:tag',t:'reflect_melee'}],verb:{v:'tactic',a:'reflect_read'}}};
  const queen={boss:'lurker_queen',text:'read silence',row:{conds:[{k:'foe:tag',t:'boss'}],verb:{v:'read',a:'silence'}}};
  check(counterName(L,'cadence',king.row)==='Mirror rhythm','tactic row by package name');
  check(counterName(L,'reflect read')==='mirror read','tactic text by package id');
  check(counterName(L,'mirror rhythm')==='Mirror rhythm','core word by package name');
  check(counterName(L,'read silence',queen.row)==='read silence','non-tactic counter unchanged');
  check(counterName(undefined,'cadence')==='cadence','no packages: the core word');
  L.counters=[king,master,queen];
  check(JSON.stringify(namedCounters(L).map(c=>c.text))==='["Mirror rhythm","mirror read","read silence"]','named counters');
  check(wallCounter({lineage:L},'mirror_king')==='counter: Mirror rhythm','floor chart wall line names the tactic');
  L.counters=[];L.walls=[{boss:'mirror_king',title:'King',depth:33,slain:false,known:true,fact:'king: mirror rhythm'}];
  check(wallCounter({lineage:L},'mirror_king')==='counter: Mirror rhythm','wall fact names the tactic');
  L.walls=[{boss:'mirror_king',title:'King',depth:33,slain:false,known:false,fact:'king: ?'}];
  check(wallCounter({lineage:L},'mirror_king')==='king: ?','unknown counter stays unknown');
  L.counters=[king];L.ledger=[];L.walls=[];
  const el=enemyHost(document.createElement('span'),'mirror_king',L);document.body.append(el);
  el.dispatchEvent(new MouseEvent('mouseenter',{bubbles:true}));el.dispatchEvent(new PointerEvent('pointerenter',{bubbles:true}));el.focus?.();el.click();
  await new Promise(r=>setTimeout(r,400));
  const tip=[...document.querySelectorAll('.enemy-tip-line')].map(e=>e.textContent).join(' | ');
  check(tip.includes('Mirror rhythm')&&!/cadence/i.test(tip),'enemy tip names the tactic: '+tip);
  return n;});
console.log(n,'counter name checks PASS');}finally{await b.close();}
