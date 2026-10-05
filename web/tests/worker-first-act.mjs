import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const n=await p.evaluate(async()=>{
  const a=window.__riddle;let n=0;const check=(ok,message)=>{if(!ok)throw Error(message);n++;};
  const r={elapsed_s:28800,runs:17,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],deepest:7,gold:{home:986}};
  const first={id:'porter',n:974,what:'hauled $974',first:true},routine={id:'armourer',n:2,what:'wore 2',first:false};
  const go=workers=>a.go({kind:'report',report:{...r,workers,chest:40},absence:true});
  go([routine,first]);const visible=document.querySelector('.report-first-workers');
  check(visible?.textContent.includes('porter')&&visible.textContent.includes('hauled $974'),'actual first act visible with exact name/action');
  check(!visible.closest('[hidden]')&&document.querySelector('.report-details').hidden,'first act visible with collapsed Details');
  check(!visible.textContent.includes('armourer'),'routine worker absent from announcement');
  check(document.querySelectorAll('.report-sheet [data-worker=porter]').length===1,'first worker never duplicated in Details');
  check(document.querySelector('.report-details [data-worker=armourer]')?.textContent.includes('wore 2'),'routine worker retained in Details');
  check(document.querySelector('.report-details [data-worker=chest]')?.textContent.includes('$40'),'chest retained in Details');
  check(document.querySelectorAll('.report-basics .tile').length===3,'simple result trio retained');
  go([{...first,first:false}]);check(!document.querySelector('.report-first-workers'),'later act not reannounced');
  go(undefined);check(!document.querySelector('.report-first-workers'),'old wire has no fabricated announcement');
  go([]);check(!document.querySelector('.report-first-workers'),'empty worker record has no announcement');
  go([{id:'porter',first:true,n:0,what:''}]);check(!document.querySelector('.report-first-workers'),'empty first flag has no announcement');
  const {mergeWorkers}=await import('/src/ui/works.ts');go(mergeWorkers([first],[{...first,n:26,what:'hauled $26',first:false}]));
  check(document.querySelector('.report-first-workers').textContent.includes('hauled $1000'),'merged first record uses total action');
  const acts=[first,...['apprentice','clerk','drillmaster','kennel_hand'].map((id,i)=>({id,n:i+1,what:['+1 step','banked $2','+3 levels','fielded 4'][i],first:true}))];go(acts);
  check(document.querySelectorAll('.report-first-workers .work-act').length===5,'every first worker announced beyond ordinary four-line budget');
  check(!document.querySelector('.report-first-workers .dim')&&!document.querySelector('.report-details .work-act:not(.chest)'),'no first worker truncated or duplicated');
  await document.fonts.ready;await new Promise(r=>setTimeout(r,300));
  const section=document.querySelector('.report-first-workers').getBoundingClientRect();
  for(const el of document.querySelectorAll('.report-first-workers .work-act')){const rect=el.getBoundingClientRect();check(rect.left>=section.left-1&&rect.right<=section.right+1,'worker chip fits parchment');}
  check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');return n;
 });console.log(width,n,'worker first-act checks PASS');await p.close();
}}finally{await b.close();}
