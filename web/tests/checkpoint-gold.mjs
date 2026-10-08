import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3908`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  const a=window.__riddle,s=await a.engine.send();window.__stakeBase=s;
  const stake=(loot,death_keep,swapped,swap_left)=>({loot,death_keep,brought:[],...(swapped===undefined?{}:{swapped}),...(swap_left?{swap_left}:{})});
  const states=[stake(30,10),stake(0,40),stake(20,40,0),stake(15,40,0),stake(0,50,5,'axe'),stake(20,50,5,'axe'),stake(0,65,5,'axe'),stake(20,65,5,'axe'),stake(10,75,5,'axe'),stake(10,75,8,'sword')];
  s.stake=states[0];a.engine.send=async()=>s;a.watchMode='fast';a.slowdowns=false;a.lastForecast=null;
  let turn=s.turn,done=0;a.engine.step=async ticks=>{turn+=ticks;const t=turn,phase=window.__stakePhase||0;let events=[];
   if(phase>done){done=phase;if([1,8].includes(phase))events=[{k:'pickup',t,id:s.hero.id,item:'folded scroll?'}];if([3,6].includes(phase))events=[{k:'steal',t,id:999,gold:5}];}
   return{events,snapshot:{...s,turn,stake:states[phase]},run_over:false};
  };a.go({kind:'watch'});
 });
 let checks=0;// `carry`: the stake's carry at risk; the strip's Carried is the core's carried (secured + at risk)
 const wait=(carry,secured)=>p.waitForFunction(({carry,secured})=>{const t=document.querySelector('.stake')?.textContent;return t?.includes(`Carried $${carry+secured}`)&&t.includes(`Secured $${secured}`);},{carry,secured});
 const check=async(regex,label)=>{const t=await p.locator('.stake').textContent();if(!regex.test(t))throw Error(`${label}: ${t}`);checks++;};
 await wait(30,10);await p.evaluate(()=>window.__stakePhase=1);await wait(0,40);await check(/^Carried \$40 · Secured \$40$/,'omitted zero swap checkpoint');
 await p.evaluate(()=>window.__stakePhase=2);await wait(20,40);
 await p.evaluate(()=>window.__stakePhase=3);await wait(15,40);await check(/−\$5 stolen/,'actual theft');
 await p.evaluate(()=>window.__stakePhase=4);await wait(0,50);await check(/−\$5 left axe/,'swap plus checkpoint excludes secured transfer');
 await p.evaluate(()=>window.__stakePhase=5);await wait(20,50);
 await p.evaluate(()=>window.__stakePhase=6);await wait(0,65);await check(/−\$5 stolen/,'theft plus checkpoint excludes secured transfer');
 await p.evaluate(()=>window.__stakePhase=7);await wait(20,65);
 await p.evaluate(()=>window.__stakePhase=8);await wait(10,75);await check(/Carried \$85 −\$5 stolen · Secured \$75/,'new carry and checkpoint preserve only prior real loss');
 await p.evaluate(()=>window.__stakePhase=9);await p.waitForFunction(()=>document.querySelector('.loot-drop')?.textContent==='−$3 left sword');checks++;
 // Fresh current wire with an explicit zero swap counter.
 await p.evaluate(()=>{const a=window.__riddle,s=window.__stakeBase;a.go({kind:'camp'});let turn=s.turn;
  a.engine.send=async()=>({...s,stake:{loot:6,death_keep:0,swapped:0,brought:[]}});
  a.engine.step=async ticks=>{turn+=ticks;const done=!!window.__explicitCheckpoint;return{events:done?[{k:'pickup',t:turn,id:s.hero.id,item:'folded scroll?'}]:[],snapshot:{...s,turn,stake:{loot:done?0:6,death_keep:done?6:0,swapped:0,brought:[]}},run_over:false};};a.go({kind:'watch'});
 });await wait(6,0);await p.evaluate(()=>window.__explicitCheckpoint=true);await wait(0,6);await check(/^Carried \$6 · Secured \$6$/,'explicit zero swap checkpoint');
 // Fresh watch on an older wire: no secured/swap counters, genuine carry fall
 // beside a pickup still uses its existing compatibility label.
 await p.evaluate(()=>{
  const a=window.__riddle,s=window.__stakeBase;a.go({kind:'camp'});let turn=s.turn;
  a.engine.send=async()=>({...s,stake:{loot:20,brought:[]}});
  a.engine.step=async ticks=>{turn+=ticks;const drop=!!window.__legacyDrop;return{events:drop?[{k:'pickup',t:turn,id:s.hero.id,item:'folded scroll?'}]:[],snapshot:{...s,turn,stake:{loot:drop?15:20,brought:[]}},run_over:false};};a.go({kind:'watch'});
 });
 await wait(20,0);await p.evaluate(()=>window.__legacyDrop=true);await wait(15,0);await check(/−\$5 swap → folded scroll\?/,'older wire genuine loss fallback');
 if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('overflow');checks++;
 if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'checkpoint gold/omitted zero/swap/theft/mixed pickup/old wire PASS');await p.close();
}}finally{await b.close();}
