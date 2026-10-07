import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 const page=await browser.newPage({viewport:{width:400,height:800}});
 await page.goto(`${url}?engine=fake&fresh=1&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 await page.evaluate(async()=>{
  const a=window.__riddle;
  const report={elapsed_s:0,runs:1,sampled:false,learned:['foe:captive','foe:captive:ally','foe:goblin_archer:ranged'],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[]};
  a.go({kind:'report',report});
  const facts=document.querySelector('.report .facts')?.textContent??'';
  if(!facts.includes('captive · potential ally')||!facts.includes('goblin archer · ranged'))throw Error('Learned captive must be a potential ally; ordinary enemy traits stay unchanged');
  const {enemyHost}=await import('/src/ui/enemy-tips.ts');
  const host=enemyHost(Object.assign(document.createElement('button'),{id:'captive-fact-probe',textContent:'captive'}),'captive',{facts:report.learned,ledger:[],walls:[],counters:[]});
  document.querySelector('.report').append(host);
  window.__captiveSaved=await a.engine.save();
 });
 await page.locator('#captive-fact-probe').hover();await page.locator('#kw-tip:not([hidden])').waitFor();
 if(!(await page.locator('#kw-tip').innerText()).includes('Potential Ally'))throw Error('Actual enemy tooltip must distinguish potential allegiance');
 if(!await page.evaluate(async()=>await window.__riddle.engine.save()===window.__captiveSaved))throw Error('Reading facts changed saved engine');
 console.log('report traits, enemy tooltip and exact save PASS');
} finally {await browser.close();}
