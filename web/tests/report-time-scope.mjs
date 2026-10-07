import {execFileSync} from 'node:child_process';
import {mkdirSync} from 'node:fs';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  const a=window.__riddle,zero={hero:0,pets:0,foes:0},split={fight:600,travel:2800,chores:100,rest:100};
  window.__timeMeter={seconds:3600,dealt:zero,taken:zero,dps_dealt:zero,dps_taken:zero,healed:[],hps:0,time:split,time_s:split,rows:[],actions:0,supplies:{},gold:60,gold_per_min:1,hits_hero:0,hits_pets:0,fights:0};
  window.__timeReport={elapsed_s:1200,runs:1,sampled:false,learned:[],bests:[],found:[],deaths:[],pending:[],marks_earned:0,tamed:[],hatched:[],lost:[],reel:[],rested_s:1800,meters:window.__timeMeter};
  window.__timeSave=await a.engine.save();a.go({kind:'report',report:window.__timeReport,absence:true});
 });
 async function inspect(seconds,scope){
  await p.locator('.report').waitFor();await p.locator('.report .details-fold').click();
  if(await p.locator('.meters').getAttribute('data-seconds')!==String(seconds))throw Error('Changed metered seconds');
  if(!(await p.locator('.mhead').textContent()).includes('Completed runs'))throw Error('Meter owner unclear');
  if(await p.locator('.meter-scope').textContent()!==scope)throw Error('Wrong duration scope');
  const rest=await p.locator('.rest-line').textContent();if(!/Rest assigned.*30m.*Includes pending/.test(rest))throw Error(`Rest scope: ${rest}`);
  if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('Horizontal overflow');
 }
 await inspect(3600,'Includes before away · Camp rest separate');
 if(process.env.RIDDLE_TIME_SHOTS){mkdirSync(process.env.RIDDLE_TIME_SHOTS,{recursive:true});await p.locator('.meters').scrollIntoViewIfNeeded();await p.screenshot({path:`${process.env.RIDDLE_TIME_SHOTS}/time-${width}.png`});}
 await p.evaluate(()=>window.__riddle.go({kind:'report',report:{...window.__timeReport,runs:3,meters:{...window.__timeMeter,seconds:10800}},absence:true}));
 await inspect(10800,'Includes before away · Camp rest separate');
 await p.evaluate(()=>window.__riddle.go({kind:'report',report:{...window.__timeReport,meters:undefined,exits:[{run_id:1,reached:8,kept:0,carried:0,keep_pct:100,spent:0,spent_on:[],text:'banked',meters:window.__timeMeter}]},absence:false}));
 await inspect(3600,'Whole runs · Camp rest separate');
 if(!await p.evaluate(async()=>await window.__riddle.engine.save()===window.__timeSave&&window.__timeReport.elapsed_s===1200&&window.__timeReport.rested_s===1800))throw Error('Reading changed save/duration');
 if(errors.length)throw Error(errors.join('\n'));console.log(width,'completed whole runs/assigned rest/absence/aggregate/exit fallback/exact save PASS');await p.close();
}}finally{await b.close();}
