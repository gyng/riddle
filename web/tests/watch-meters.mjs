import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3906`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  localStorage.setItem('riddle.meters','1');const a=window.__riddle,s=await a.engine.send();
  const sides=hero=>({hero,pets:0,foes:0}),time={fight:1,travel:0,chores:0,rest:0};
  const meter=damage=>({run:{seconds:1,dealt:sides(damage),taken:sides(0),dps_dealt:sides(damage),dps_taken:sides(0),healed:[],hps:0,time,time_s:time,rows:[],actions:0,supplies:{},gold:0,gold_per_min:0,hits_hero:0,hits_pets:0,fights:1},fighting:false});
  const initial=s.turn;window.__meterInitial=initial;s.meters=meter(0);a.engine.send=async()=>s;a.watchMode='fast';a.slowdowns=false;a.lastForecast=null;
  a.engine.step=async()=>{const phase=window.__meterPhase??1;window.__meterCore=phase===1?6:phase===2?12:phase===3?18:phase===5?888:999;
   return{events:[],snapshot:{...s,turn:initial+phase,meters:meter(window.__meterCore)},run_over:false};
  };a.go({kind:'watch'});
 });
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 const read=()=>p.evaluate(width=>width>=1024?document.querySelector('.meters-live .mrow .mval')?.textContent:document.querySelector('.meter-line .dealt b')?.textContent,width);
 const wait=damage=>p.waitForFunction(({width,damage})=>width>=1024?document.querySelector('.meters-live .mrow .mval')?.textContent.includes(`· ${damage} hp`):document.querySelector('.meter-line .dealt b')?.textContent===String(damage),{width,damage},{timeout:5000});
 await wait(6);await p.evaluate(()=>window.__meterPhase=2);await p.waitForFunction(()=>window.__meterCore===12);
 await p.waitForFunction(()=>Number(document.querySelector('.watch').dataset.tick)>=window.__meterInitial+2);
 await p.evaluate(()=>window.__meterPhase=3);await p.waitForFunction(()=>window.__meterCore===18);
 await p.waitForFunction(()=>Number(document.querySelector('.watch').dataset.tick)>=window.__meterInitial+3);
 await p.locator('.console .gem').click();await p.evaluate(()=>window.__meterPhase=4);
 await p.waitForFunction(()=>window.__meterCore===999);await wait(18);
 await p.waitForTimeout(500);if((await read()).includes('999'))throw Error('future meter leaked into paused picture');let checks=4;
 await p.evaluate(()=>{window.__meterPhase=4;document.querySelector('.console .gem').click();});await wait(999);checks++;
 await p.evaluate(()=>window.__meterPhase=5);await p.waitForFunction(()=>window.__meterCore===888);
 await p.waitForFunction(()=>Number(document.querySelector('.watch').dataset.tick)>=window.__meterInitial+5);
 await p.evaluate(width=>{window.__detachedMeter=document.querySelector(width>=1024?'.meters-live':'.meter-box');window.__detachedBefore=window.__detachedMeter.textContent;if(window.__detachedBefore.includes('888'))throw Error('disposal did not cover a pending repaint');window.__riddle.go({kind:'camp'});},width);
 await p.waitForTimeout(450);
 if(!await p.evaluate(()=>window.__detachedMeter.textContent===window.__detachedBefore))throw Error('pending repaint ran after disposal');checks++;
 if(await p.locator('.watch').count())throw Error('watch survived disposal');checks++;
 if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('overflow');checks++;
 if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'watch meters/trailing paint/paused frontier/resume/disposal PASS');await p.close();
}}finally{await b.close();}
