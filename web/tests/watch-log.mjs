import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const b=await launchBrowser();
try {
 for(const width of [400,1440]) for(const [depth,start] of [[5,5],[14,5],[1,9]]) {
  const p=await b.newPage({viewport:{width,height:900}}),errors=[];
  p.on('pageerror',e=>errors.push(e.message));
  await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=4203`);
  await p.waitForFunction(()=>window.__riddle?.booted);
  await p.evaluate(async({depth,start})=>{
   const a=window.__riddle,s=await a.engine.send();s.depth=depth;s.run.start=start;
   a.lineage.live=null;a.lineage.start=start;a.lastForecast=null;a.watchMode='one';
   let first=true,filled=false,turn=s.turn;
   a.engine.send=async()=>s;
   a.engine.step=async ticks=>{
    const t=turn+1;turn+=ticks;
    const events=window.__extraLog?(window.__extraLog=false,[{k:'pickup',t,id:s.hero.id,item:'sword +2'}]):first?[
     {k:'telegraph',t,id:s.hero.id,what:'rallies'},
     {k:'heal',t,id:s.hero.id,amount:3,src:'heal'},
     {k:'pickup',t,id:s.hero.id,item:'sword +1'},
     {k:'descend',t,depth:depth+1,biome:s.biome},
     {k:'hurt',t,id:s.hero.id,dmg:2,hp:s.hero.hp-2,cause:'poison'}
    ]:window.__fillLog&&!filled?(filled=true,Array.from({length:90},(_,i)=>({k:'pickup',t,id:s.hero.id,item:`gold ${i}`}))):[];
    first=false;
    return {events,snapshot:{...s,turn,depth:depth+1},run_over:false};
   };
   a.go({kind:'watch'});
  },{depth,start});
  await p.waitForFunction(()=>document.querySelectorAll('.combat-lines li').length>=5,null,{timeout:20000});
  const initial=await p.evaluate(()=>[...document.querySelectorAll('.combat-lines li')].map(x=>({text:x.textContent,warning:!!x.querySelector('.log-warning'),heal:!!x.querySelector('.log-heal'),item:x.querySelector('.item-ico')?.dataset.kind})));
  if(!initial[0].text.startsWith(`D${depth} · `)||!initial[0].warning||!initial[0].text.includes('rallies')||!initial[1].heal||!initial[1].text.includes('+3 hp')||!initial[1].text.startsWith(`D${depth} · `)||initial[2].item!=='sword +1'||!initial[2].text.startsWith(`D${depth} · `)||!initial[3].text.startsWith(`D${depth+1} · `)||!initial[4].text.startsWith(`D${depth+1} · `))throw Error(JSON.stringify(initial));
  await p.evaluate(()=>window.__fillLog=true);
  await p.waitForFunction(()=>document.querySelectorAll('.combat-lines li').length===80,null,{timeout:20000});
  // The batch deliberately exceeds the ring: retained later pickups must use the descended floor.
  const result=await p.evaluate(()=>({rows:[...document.querySelectorAll('.combat-lines li')].map(x=>x.textContent),overflow:document.documentElement.scrollWidth>innerWidth}));
  if(result.rows.some(x=>!x.startsWith(`D${depth+1} · `))||result.overflow||errors.length)throw Error(JSON.stringify({width,depth,result,errors}));
  await p.evaluate(()=>{document.querySelector('.combat-lines').scrollTop=180;});
  await p.waitForFunction(()=>!document.querySelector('.log-latest').hidden);
  const before=await p.evaluate(()=>{const rows=document.querySelector('.combat-lines');return {text:[...rows.children].find(x=>x.getBoundingClientRect().bottom>rows.getBoundingClientRect().top).textContent,top:rows.scrollTop,removed:rows.firstElementChild.offsetHeight};});
  await p.evaluate(()=>window.__extraLog=true);
  await p.waitForFunction(()=>document.querySelector('.combat-lines').lastElementChild?.textContent.includes('sword +2'));
  const after=await p.evaluate(()=>{const rows=document.querySelector('.combat-lines');return{top:rows.scrollTop,text:[...rows.children].find(x=>x.getBoundingClientRect().bottom>rows.getBoundingClientRect().top).textContent,latest:!document.querySelector('.log-latest').hidden};});
  if(after.top!==before.top-before.removed||after.text!==before.text||!after.latest)throw Error(JSON.stringify({before,after}));
  await p.locator('.log-latest').click();
  if(!await p.evaluate(()=>{const r=document.querySelector('.combat-lines');return r.scrollHeight-r.clientHeight-r.scrollTop<=2&&document.querySelector('.log-latest').hidden;}))throw Error('Latest must resume following');
  console.log(width,depth,'ordered floor attribution and80-entry bound PASS');
  await p.close();
 }
} finally { await b.close(); }
