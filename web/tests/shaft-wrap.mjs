// Blind c4705f9 (B: `be hin d wa rlo rd` wrapped one or two letters a line in the floor-chance column; status labels overlapping): at
// phone and desk widths every word of a notch's labels stays whole, and no two of the column's labels overlap.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const shots=process.env.SHAFT_SHOTS;
const browser=await launchBrowser();let total=0;const errors=[];
try{
 for(const width of [400,1440]){
  const page=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:1});
  page.on('pageerror',e=>errors.push(e.message));
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=24`);await page.waitForFunction(()=>window.__riddle?.booted&&window.__riddle.screen==='camp');
  await page.evaluate(async()=>{const r=window.__riddle;const b=JSON.parse(r.exportSave());const e=JSON.parse(b.engine);e.lineage.best_depth=8;b.engine=JSON.stringify(e);await r.importSave(JSON.stringify(b));});
  await page.waitForFunction(()=>window.__riddle?.booted&&window.__riddle.screen==='camp');
  await page.waitForTimeout(500);await page.waitForTimeout(1500);
  await page.evaluate(()=>{
   const r=window.__riddle,b=r.lastForecast;
   const depths=Array.from({length:9},(_,i)=>({depth:i+1,reach:i<7?.9-i*.05:i===7?.5:.12,pm:.03,biome:i>=8?'fens':'shaft'}));
   depths[7].boss='goblin_warlord';depths[8].wall='goblin_warlord';
   const x={...b,depths,known_to:9,refined:true,start:1};r.lastForecast=x;for(const fn of r.fcListeners)fn(x);
  });
  await page.waitForTimeout(300);
  if(shots)await page.locator('.shaft').first().screenshot({path:`${shots}/shaft-${width}.png`});
  const out=await page.evaluate(()=>{
   const bad=[],overlap=[];const shaft=[...document.querySelectorAll('.shaft')].find(s=>s.getBoundingClientRect().width>0);
   if(!shaft)return {err:'no visible shaft'};
   const labels=[...shaft.querySelectorAll('.notch .dl i, .notch small, .shaft-head *, .progress-goal *')].filter(e=>e.getBoundingClientRect().width>0&&!e.children.length||e.matches('.unit-label-text'));
   // a word broken across lines has a Range with more than one client rect
   for(const el of shaft.querySelectorAll('.notch .dl, .notch small')){
    const walk=document.createTreeWalker(el,NodeFilter.SHOW_TEXT);let t;
    while((t=walk.nextNode())){const re=/\S+/g;let m;while((m=re.exec(t.data))){const rg=document.createRange();rg.setStart(t,m.index);rg.setEnd(t,m.index+m[0].length);const rects=[...rg.getClientRects()].filter(r=>r.width>0);if(rects.length>1)bad.push(m[0]);}}
   }
   const leaves=[...shaft.querySelectorAll('*')].filter(e=>!e.children.length&&e.textContent.trim()&&e.getBoundingClientRect().width>0);
   for(let i=0;i<leaves.length;i++)for(let j=i+1;j<leaves.length;j++){const a=leaves[i].getBoundingClientRect(),b=leaves[j].getBoundingClientRect();const ix=Math.min(a.right,b.right)-Math.max(a.left,b.left),iy=Math.min(a.bottom,b.bottom)-Math.max(a.top,b.top);if(ix>2&&iy>2&&!leaves[i].contains(leaves[j])&&!leaves[j].contains(leaves[i]))overlap.push(`${leaves[i].textContent.trim()} × ${leaves[j].textContent.trim()}`);}
   const box=shaft.getBoundingClientRect();
   for(const el of leaves){const r=el.getBoundingClientRect();if(r.right>box.right+1||r.left<box.left-1)overlap.push(`${el.textContent.trim()} runs past the column`);}
   const wall=shaft.querySelector('.notch[data-d="9"] .wall')?.textContent??'';
   return {bad,overlap,wall,scroll:document.documentElement.scrollWidth<=innerWidth};
  });
  if(out.err)throw new Error(`${width}: ${out.err}`);
  const check=(v,l)=>{if(!v)throw new Error(`${width}: ${l}`);total++;};
  check(/behind warlord/.test(out.wall),`the wall names its boss (${out.wall})`);
  check(!out.bad.length,`no word broken across lines (${out.bad.join(', ')})`);
  check(!out.overlap.length,`no labels overlap (${out.overlap.join(' | ')})`);
  check(out.scroll,'no horizontal overflow');
  console.log(`shaft-wrap ${width}: PASS`);await page.close();
 }
 if(errors.length)throw new Error(errors.join('\n'));
 console.log(`shaft-wrap: ${total} checks PASS`);
}finally{await browser.close();}
