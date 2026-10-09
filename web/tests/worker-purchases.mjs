// blind c4705f9 (A, B: `purse −$12562` while away, the apprentice's `+4 steps` the only word): the report names what the
// apprentice bought and what it paid — on its chip, under the purse, and in the gold tip — and slices merge.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const n=await p.evaluate(async()=>{
  const a=window.__riddle;let n=0;const check=(ok,message)=>{if(!ok)throw Error(message);n++;};
  const gold={home:290,salvage:0,wake:0,spent:0,wake_cap:40,wake_n:0,net:290-12562};
  const r={elapsed_s:1200,runs:1,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],deepest:13,gold};
  const app={id:'apprentice',n:4,what:'+4 steps',first:true,items:['sword +3','mail +2'],spent:12562};
  a.go({kind:'report',report:{...r,workers:[app]},absence:true});
  const chip=document.querySelector('.report-first-workers [data-worker=apprentice]');
  check(chip?.textContent.includes('sword +3')&&chip.textContent.includes('mail +2')&&chip.textContent.includes('−$12562'),'chip itemises purchases: '+chip?.textContent);
  check(document.querySelector('.report-bought')?.textContent.trim()==='(forge $12562)','purse names the forge spend: '+document.querySelector('.report-bought')?.textContent);
  // Cut 117 §1: the head reconciles — earned − spent = purse
  const led=document.querySelector('.report-ledger');check(led&&+led.dataset.earned - +led.dataset.spent === +led.dataset.net&&/earned \$290 − spent \$12562 \(forge \$12562\) = purse −\$12272/.test(led.textContent),'ledger reconciles: '+led?.textContent);
  const {boughtText,mergeWorkers}=await import('/src/ui/works.ts');
  check(boughtText({})==='','a worker that bought nothing adds no words');
  const m=mergeWorkers([app],[{...app,n:1,what:'+1 step',first:false,items:['sword +4'],spent:3000}])[0];
  check(m.spent===15562&&m.items.join()==='mail +2,sword +4'&&m.what==='+5 steps','slices merge: '+JSON.stringify(m));
  await document.fonts.ready;await new Promise(r=>setTimeout(r,300));
  const sec=document.querySelector('.report-first-workers').getBoundingClientRect(),cr=chip.getBoundingClientRect();
  check(cr.left>=sec.left-1&&cr.right<=sec.right+1,'chip fits parchment');
  check(document.documentElement.scrollWidth<=innerWidth,'no horizontal overflow');return n;
 });console.log(width,n,'worker purchase checks PASS');await p.close();
}}finally{await b.close();}
