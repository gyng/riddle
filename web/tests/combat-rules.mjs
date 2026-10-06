import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3907`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  const a=window.__riddle,s=await a.engine.send();s.hero.name='Wren Ash';a.engine.send=async()=>s;a.watchMode='one';a.slowdowns=false;
  let turn=s.turn,done=0;a.engine.step=async ticks=>{const t=turn+1;turn+=ticks;const phase=window.__rulePhase||0;let events=[];
   const rule=(row=0,text='attack nearest',v='attack',arg='nearest')=>({k:'rule',t,row,text,verb:{v,a:arg}});
   if(phase>done){done=phase;window.__ruleEmitted=phase;
    if(phase===1)events=[rule(),{...rule(),t:t+.1}];
    if(phase===2)events=[rule()];
    if(phase===3)events=[rule(1),rule(1,'attack nearest','attack','tag:caster'),rule(1,'attack archer','attack','tag:caster'),rule(1,'attack archer','shoot','tag:caster'),
     {k:'attack',t,src:s.hero.id,dst:999,hit:true,dmg:2},rule(1,'attack archer','shoot','tag:caster'),
     {k:'hurt',t,id:s.hero.id,cause:'fire',dmg:1},rule(1,'attack archer','shoot','tag:caster'),
     rule(-2,'pick up','pick_up'),rule(1,'attack archer','shoot','tag:caster'),
     {k:'descend',t,depth:2,biome:s.biome},rule(1,'attack archer','shoot','tag:caster'),
     {k:'pickup',t,id:s.hero.id,item:'gold $1'},{k:'pickup',t,id:s.hero.id,item:'gold $1'},
     {k:'pickup',t,id:s.hero.id,item:'rules checkpoint'}];
   }
   return{events,snapshot:{...s,turn},run_over:false};
  };a.go({kind:'watch'});
 });
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);await p.locator('.console .gem').click();
 await p.evaluate(()=>window.__rulePhase=1);await p.waitForFunction(()=>window.__ruleEmitted===1);
 await p.waitForTimeout(150);if(await p.locator('.combat-lines li').count())throw Error('paused decisions appeared early');let checks=1;
 await p.locator('.console .gem').click();await p.waitForFunction(()=>document.querySelector('.log-repeat')?.textContent===' ×2');
 if(await p.locator('.combat-lines li').count()!==1)throw Error('identical decisions occupy multiple rows');checks++;
 await p.locator('.console .gem').click();await p.evaluate(()=>window.__rulePhase=2);await p.waitForFunction(()=>window.__ruleEmitted===2);
 await p.waitForTimeout(150);if(await p.locator('.log-repeat').first().textContent()!==' ×2')throw Error('paused count includes future decisions');checks++;
 await p.locator('.console .gem').click();await p.waitForFunction(()=>document.querySelector('.log-repeat')?.textContent===' ×3');
 if(await p.locator('.combat-lines li').count()!==1)throw Error('batch boundary split identical decisions');checks++;
 checks+=await p.evaluate(()=>{const row=document.querySelector('.combat-lines li');if(!(Number(row.dataset.firstTick)<Number(row.dataset.tick)))throw Error('group tick range lost');return 1;});
 await p.evaluate(()=>window.__rulePhase=3);await p.waitForFunction(()=>document.querySelector('.combat-lines')?.textContent.includes('rules checkpoint'));
 checks+=await p.evaluate(()=>{let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;},rows=[...document.querySelector('.combat-lines').children],text=rows.map(r=>r.textContent);
  check(rows.length===15,'distinct events preserved');
  check(text.filter(t=>t.includes('attack nearest')).length===3,'changed row and argument remain separate');
  check(text.filter(t=>t.includes('attack archer')).length===6,'changed text/verb and hit/hazard/chore/floor boundaries');
  check(text.some(t=>t.includes('Wren Ash → Foe · −2 hp')),'hit retained');
  check(text.some(t=>t.includes('Wren Ash · fire −1 hp')),'hazard retained');
  check(text.some(t=>t.includes('D2 · attack archer')),'floor attribution retained');
  check(text.filter(t=>t.includes('gold $1')).length===2,'pickups never coalesced');
  check(rows.filter(r=>r.querySelector('.log-repeat')?.textContent).length===1,'only initial identical group counted');
  check(getComputedStyle(document.querySelector('.combat-log')).backgroundImage==='none','plain unboxed overlay');return n;
 });
 if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('overflow');checks++;
 if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'combat decisions/counts/paused release/batch and action boundaries PASS');await p.close();
}}finally{await b.close();}
