import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 for(const width of [320,400,1440]) {
  const page=await browser.newPage({viewport:{width,height:800}});
  await page.goto(`${url}?dev=1&engine=fake&seed=3004&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
  await page.evaluate(async()=>{
   const a=window.__riddle;
   const driven={boss:'goblin_warlord',title:'Warlord',depth:8,verdict:'counter',defence:'no counter',counter:'attack boss',row:{conds:[],verb:{v:'attack',a:'tag:boss'}},hp:31,max_hp:40};
   const exits=Array.from({length:8},(_,i)=>({run_id:i+1,reached:8,driven,kept:0,carried:0,keep_pct:60,spent:0,spent_on:[],text:'driven $0 · '+Array(14).fill('no counter').join(' · ')}));
   window.__readingReport={elapsed_s:28800,runs:8,sampled:false,learned:[],bests:[],deepest:8,found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],returned:8,driven:8,exits};
   window.__readingSave=await a.engine.save();a.go({kind:'report',report:window.__readingReport,absence:true});
  });
  const details=()=>page.locator('.report-details');
  if(!await details().isHidden())throw Error('First view starts folded');
  await page.locator('.report .details-fold').click();
  for(let i=0;i<2;i++) {
   const verdict=page.locator('.exit-row .verdict-chip').nth(i);await verdict.scrollIntoViewIfNeeded();
   const scroll=await page.locator('.report-well').evaluate(el=>el.scrollTop);
   await verdict.click();await page.locator('.death').waitFor();
   await page.locator('.console [data-tile="report"]').click();await page.locator('.report').waitFor();
   await page.evaluate(()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve))));
   if(!await details().isVisible()||await page.locator('.report .details-fold').getAttribute('aria-expanded')!=='true')throw Error('Back from death collapsed report details');
   if(!await page.evaluate(()=>window.__riddle.view.absence===true))throw Error('Back lost the absence context');
   const restored=await page.locator('.report-well').evaluate(el=>el.scrollTop);
   if(Math.abs(restored-scroll)>2)throw Error(`Back lost reading position: ${scroll} -> ${restored}`);
  }
  await page.locator('.report .details-fold').click();
  await page.evaluate(()=>window.__riddle.go({kind:'report',report:window.__readingReport,absence:true}));
  if(!await details().isHidden())throw Error('Explicit collapse not retained');
  await page.locator('.report .details-fold').click();
  await page.evaluate(()=>window.__riddle.go({kind:'report',report:{...window.__readingReport},absence:true}));
  if(!await details().isHidden())throw Error('New report inherited old expanded state');
  if(!await page.evaluate(async()=>await window.__riddle.engine.save()===window.__readingSave))throw Error('Reading reports changed save');
  console.log(`${width} two verdict/back cycles/scroll/absence/collapse/new report/exact save PASS`);await page.close();
 }
} finally {await browser.close();}
