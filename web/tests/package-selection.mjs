import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();const b=await launchBrowser();
try{for(const width of [400,1440]){const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
const n=await p.evaluate(async()=>{
const {openPackages}=await import('/src/ui/packages.ts'),{closeAllSheets}=await import('/src/ui/sheet.ts');let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;},tick=()=>new Promise(r=>setTimeout(r,20)),jobs=[],change=new Set(),rules=new Set();
const real=window.__riddle,L=structuredClone(real.lineage);L.packages.all.forEach(p=>p.owned=true);L.packages.stance='steady';L.packages.tactic_slots=2;L.packages.tactics=[];
const app={lineage:L,rules:real.rules,loadout:[],lastForecast:null,onChange:fn=>{change.add(fn);return()=>change.delete(fn);},onRules:fn=>{rules.add(fn);return()=>rules.delete(fn);}};
app.engine={packageOptions:()=>{throw Error('must not query all moves');},packageOptionsFor:(sims,choices)=>new Promise((resolve,reject)=>jobs.push({sims,choices,resolve,reject}))};
const answer=(j)=>j.choices.map(([id,slot])=>({id,slot,action:'equip',past:.9,bank:.1,death:.1,d_past:.8,d_bank:0,d_death:0}));
closeAllSheets();openPackages(app);await tick();check(jobs.length===0,'opening starts no simulations');
document.querySelector('.pkg-compare').click();await tick();check(jobs.length===1&&jobs[0].sims===8,'explicit eight-sample query');
const styles=L.packages.all.filter(p=>p.kind==='stance'&&p.id!=='steady').map(p=>p.id);
check(jobs[0].choices.length===styles.length&&jobs[0].choices.every(([id,slot])=>styles.includes(id)&&slot===0),'only opened styles queried');
document.querySelector('[data-edit-slot="1"]').click();await tick();check(jobs.length===2&&jobs[1].choices.some(([id,slot])=>L.packages.all.find(p=>p.id===id).kind==='tactic'&&slot===1),'tactics use selected slot');
document.querySelector('[data-change-kind="stance"]').click();await tick();check(jobs.length===3&&jobs[2].choices.every(([id,slot])=>L.packages.all.find(p=>p.id===id).kind==='tactic'&&slot===1),'closed styles excluded');
jobs[2].resolve(answer(jobs[2]));await tick();jobs[0].resolve(answer(jobs[0]));jobs[1].resolve(answer(jobs[1]));await tick();
check(![...document.querySelectorAll('.pkg-sec[data-kind="stance"] .pkg-price')].some(p=>p.textContent),'stale styles cannot paint');
check([...document.querySelectorAll('.pkg-sec[data-kind="tactic"] .pkg-price')].some(p=>p.textContent),'current slot prices paint');
app.lineage={...app.lineage,selected_bloodline:2};for(const fn of change)fn();await tick();check(jobs.length===3,'hero change never auto-queries');check(![...document.querySelectorAll('.pkg-price')].some(p=>p.textContent),'hero change clears stale prices');
document.querySelector('.pkg-compare').click();await tick();jobs[3].reject(Error('failed'));await tick();check(jobs.length===4,'failed query does not loop');
document.querySelector('.pkg-compare').click();await tick();check(jobs.length===5,'explicit retry works in open panel');jobs[4].resolve(answer(jobs[4]));await tick();
for(const fn of rules)fn();await tick();check(jobs.length===5,'rule change never auto-queries');
closeAllSheets();check(change.size===0&&rules.size===0,'close releases subscriptions');check(document.documentElement.scrollWidth<=innerWidth,'no overflow');return n;
});console.log(width,n,'selected Tactics checks PASS');await p.close();}}finally{await b.close();}
