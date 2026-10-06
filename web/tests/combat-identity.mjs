import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3805`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{
  const a=window.__riddle,s=await a.engine.send();s.hero.name='Wren Ash';
  const rat={id:800,kind:'rat',name:'Nib',x:s.hero.x+1,y:s.hero.y,hp:10,max_hp:10,tags:[],ally:true};
  const foe={...rat,id:801,kind:'goblin',name:'Scarface',ally:false};s.entities=[rat,foe];
  a.engine.send=async()=>s;a.lineage.hero_legacy=[{name:'Wrong heir',heir:999}];a.watchMode='one';a.slowdowns=false;
  let turn=s.turn,done=0;a.engine.step=async ticks=>{
   const t=turn+1;turn+=ticks;const phase=window.__logPhase||0;let events=[];
   if(phase>done){done=phase;window.__emitted=phase;
    if(phase===1)events=[{k:'attack',t,src:s.hero.id,dst:801,hit:true,dmg:3},{k:'attack',t,src:800,dst:801,hit:true,dmg:2},{k:'hurt',t,id:s.hero.id,cause:'fire',dmg:2},{k:'heal',t,id:s.hero.id,src:'heal',amount:4},{k:'pickup',t,id:s.hero.id,item:'gold $1'}];
    if(phase===2)events=[{k:'attack',t,src:800,dst:s.hero.id,hit:true,dmg:1}];
    if(phase===3)events=Array.from({length:85},(_,i)=>({k:'pickup',t:t+i/100,id:s.hero.id,item:'gold $1'}));
   }
   const snap={...s,turn,hero:{...s.hero},entities:phase>=2?[]:s.entities};
   if(phase>=2)delete snap.hero.name;
   // Once the paused batch is queued, mutate subsequent snapshot names. Its
   // delayed rows must still use the names that came with their own batch.
   if(phase===1&&window.__emitted===1&&window.__renameActors){snap.hero.name='Later heir';snap.entities=[];}
   return{events,snapshot:snap,run_over:false};
  };a.go({kind:'watch'});
 });
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 await p.locator('.console .gem').click();await p.waitForFunction(()=>document.querySelector('.live-badge')?.dataset.status==='paused');
 await p.evaluate(()=>window.__logPhase=1);await p.waitForFunction(()=>window.__emitted===1);
 await p.evaluate(()=>window.__renameActors=true);await p.waitForTimeout(200);
 if(await p.locator('.combat-lines li').count())throw Error('paused rows released early');
 await p.locator('.console .gem').click();
 await p.waitForFunction(()=>document.querySelector('.combat-lines')?.children.length>=5);
 let checks=await p.evaluate(()=>{let n=0;const check=(ok,label)=>{if(!ok)throw Error(label);n++;},el=document.querySelector('.combat-lines'),text=el.textContent;
  check(text.includes('Wren Ash → Scarface'),'hero and foe snapshot names');check(text.includes('Nib → Scarface'),'companion named rather than Foe');check(!/Wrong heir|Later heir|Later actor/.test(text),'delayed events retain original identities');
  check(el.querySelector('.log-hurt')?.textContent==='−2 hp','hurt styling retained');check(el.querySelector('.log-heal')?.textContent==='+4 hp','heal styling retained');check(el.querySelector('.log-damage')?.textContent==='−3 hp','damage styling retained');check(el.querySelector('.log-gold')?.textContent==='gold $1','item styling retained');return n;
 });
 await p.evaluate(()=>window.__logPhase=2);await p.waitForFunction(()=>document.querySelector('.combat-lines')?.children.length>=6);
 const old=await p.locator('.combat-lines li').last().innerText();if(!old.includes('Nib → Hero'))throw Error(`old snapshot/remembered companion: ${old}`);checks++;
 await p.evaluate(()=>window.__logPhase=3);await p.waitForFunction(()=>document.querySelector('.combat-lines')?.children.length===80);
 if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('overflow');checks+=2;
 if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'combat identity/delayed actors/fallback/colours/bounds PASS');await p.close();
}}finally{await b.close();}
