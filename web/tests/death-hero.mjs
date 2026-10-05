import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);const n=await p.evaluate(async()=>{
let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;};const a=window.__riddle;a.lineage.packages.pen_open=false;
const d={run_id:1,depth:8,cause:'goblin_warlord',margin:'2 hp short',verdict:'gap',baseline:0,replays:12,patches:[],morgue:'heir 1',trace:{turns:[]},hero:{name:'Wren',bloodline_id:1,heir:1,class:'fighter'}};
a.lineage.heir=7;a.lineage.class='ranger';a.lineage.selected_bloodline=2;a.lineage.hero_slots=[{id:2,hero_name:'Vale',heir:7,class:'ranger'}];a.go({kind:'death',death:d,kept:true});
const label=()=>document.querySelector('.death-hero');check(label().textContent==='Wren · fighter · Bloodline 1','historical identity wins over current hero');check(!label().textContent.includes('Vale'),'no name inferred from selected hero');check(document.querySelector('.verdict').textContent==='you died','owner seal retained');
a.lineage.heir=8;a.lineage.selected_bloodline=3;a.afterLineage();check(label().textContent==='Wren · fighter · Bloodline 1','lineage pulse cannot rewrite history');
await new Promise(r=>setTimeout(r,350));const rect=label().getBoundingClientRect();check(rect.left>=0&&rect.right<=innerWidth&&rect.width>0,'identity visible and within viewport');
a.go({kind:'death',death:{...d,hero:undefined},kept:true});check(!label(),'old death wire invents no identity');
a.go({kind:'death',death:{...d,verdict:'stall'},kept:true});check(label().textContent==='Wren · fighter · Bloodline 1','stall records keep identity');check(document.querySelector('.death').classList.contains('stalled'),'stall presentation preserved');
check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');return n;
});console.log(width,n,'historical death hero checks PASS');await p.close();}}finally{await b.close();}
