import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3805`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  const {watchStatus}=await import('/src/ui/watch.ts');
  for(const [args,expected] of [[[5,5,false,false],['live','Live delve','continues away']],[[5,6,false,false],['earlier','Watching D5','Live D6']],[[5,6,true,false],['paused','Watch paused','continues away']],[[5,6,true,true],['ended','Run ended','Watching D5']]]){
   const x=watchStatus(...args);if(JSON.stringify([x.kind,x.label,x.detail])!==JSON.stringify(expected))throw Error(JSON.stringify(x));
  }
  const a=window.__riddle,s=await a.engine.send();s.depth=5;s.run.start=5;a.lastForecast=null;a.lineage.live=null;a.watchMode='one';
  let turn=s.turn,depth=5,ended=false;a.engine.send=async()=>s;
  a.engine.step=async ticks=>{const t=turn+1;turn+=ticks;window.__coreTick=turn;const events=[];
   if(window.__nextFloor&&depth===5){depth=6;events.push({k:'descend',t,depth,biome:s.biome});}
   if(window.__endWatch&&!ended){ended=true;events.push({k:'exit',t,tier:'return',loot_kept:0});}
   window.__coreDepth=depth;return{events,snapshot:{...s,turn,depth},run_over:ended};
  };a.go({kind:'watch'});
 });
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 await p.locator('.console .gem').click();
 await p.waitForFunction(()=>document.querySelector('.live-badge')?.dataset.status==='paused');
 const held=await p.evaluate(()=>Number(document.querySelector('.watch').dataset.tick));
 await p.evaluate(()=>window.__nextFloor=true);
 await p.waitForFunction(()=>window.__coreDepth===6&&Number(window.__coreTick)>Number(document.querySelector('.watch').dataset.tick)+30);
 const state=await p.evaluate(()=>({tick:Number(document.querySelector('.watch').dataset.tick),text:document.querySelector('.live-badge').textContent,depth:document.querySelector('.hud .depth').textContent}));
 if(width>=1024){if(!await p.locator('.depth-summary').isVisible())throw Error('Missing forecast hid current depth');if(await p.locator('.depth-current b').innerText()!=='D5')throw Error('Paused depth used engine frontier');}
 if(state.tick!==held||state.depth!=='D5'||!state.text.includes('Watch paused'))throw Error(JSON.stringify({held,state}));
 const earlier=await p.evaluate(()=>{document.querySelector('.console .gem').click();return document.querySelector('.live-badge').textContent;});
 if(!earlier.includes('Watching D5')||!earlier.includes('Live D6'))throw Error(earlier);
 await p.waitForFunction(()=>document.querySelector('.hud .depth')?.textContent==='D6',null,{timeout:15000});
 await p.waitForFunction(()=>document.querySelector('.live-badge')?.dataset.status==='live');
 await p.locator('.console .gem').click();await p.evaluate(()=>window.__endWatch=true);
 await p.waitForFunction(()=>document.querySelector('.live-badge')?.dataset.status==='ended');
 if(!await p.locator('.live-badge').isVisible())throw Error('End status hidden');
 if(width>=1024&&await p.locator('.depth-current b').innerText()!=='D6')throw Error('End depth mismatch');
 const end=await p.locator('.live-badge').innerText();if(!end.includes('Run ended')||!end.includes('Watching D6'))throw Error(end);
 if(errors.length)throw Error(errors.join('\n'));console.log(width,'status/paused core progress/resume/ending PASS');await p.close();
}}finally{await b.close();}
