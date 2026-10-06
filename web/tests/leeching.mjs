// Controlled combat fixture with real Rust/WASM worker; not an earned-clear claim.
import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),browser=await launchBrowser();
try{for(const width of [320,400,1440]){
 const page=await browser.newPage({viewport:{width,height:900}}),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(`${url}?fresh=1&seed=7&runs=0&speed=1`);await page.waitForFunction(()=>window.__riddle?.booted);
 const evidence=await page.evaluate(async()=>{
  const a=window.__riddle;if(a.kind!=='wasm')throw Error('real worker required');a.runnerOn=false;
  let raw=JSON.parse(await a.engine.save());raw.lineage.town.home=true;await a.engine.load(JSON.stringify(raw));await a.engine.send();raw=JSON.parse(await a.engine.save());
  const r=raw.run,m=r.monsters[0],p={x:r.hero.pos.x+1,y:r.hero.pos.y};
  raw.lineage.endgame={tier:6,unlocked:6,cleared:5};r.difficulty=6;r.monsters=[m];
  m.kind='goblin';m.pos=p;m.hp=10;m.max_hp=10000;m.atk=[10,10];m.def=0;m.awake=true;m.energy=100;m.speed=10;m.last_seen=r.hero.pos;
  m.modifiers={tier:6,affixes:0,elite:'leeching'};r.hero.hp=10000;r.hero.max_hp=10000;r.hero.energy=-100000;
  const i=p.y*r.floor.map.w+p.x;r.floor.map.tiles[i]='floor';r.floor.map.seen[i]=true;r.floor.map.visible[i]=true;
  a.lineage=await a.engine.load(JSON.stringify(raw));const before=await a.engine.save(),snapshot=(await a.engine.step(0)).snapshot;
  const actual=await a.engine.step(20),after=await a.engine.save(),recovers=actual.events.filter(e=>e.k==='recover');
  if(!recovers.length)throw Error('actual positive melee recovery required');
  await a.engine.load(before);const again=await a.engine.step(20);
  if(JSON.stringify(actual)!==JSON.stringify(again)||await a.engine.save()!==after)throw Error('whole step/save reload must match');
  const {ReplayState}=await import('/src/render/state.ts'),{combatLogEvents}=await import('/src/ui/combat-log.ts');
  const state=new ReplayState();state.load(snapshot);state.apply(actual.events);state.seek(actual.snapshot.turn);
  const foe=actual.snapshot.entities.find(e=>e.id===m.id);
  if(state.ents.get(m.id).hp!==foe.hp)throw Error('played recovery health must match Rust');
  state.seek(snapshot.turn);state.seek(actual.snapshot.turn);if(state.ents.get(m.id).hp!==foe.hp)throw Error('seek must retain authoritative HP');
  if(combatLogEvents(actual.events,snapshot.hero.id,new Map([[m.id,'goblin']])).filter(e=>e.k==='recover').length!==recovers.length)throw Error('recovery log lost or duplicated');
  await a.engine.load(before);raw=JSON.parse(before);raw.run.monsters[0].poison=[1,1000];await a.engine.load(JSON.stringify(raw));
  if((await a.engine.step(20)).events.some(e=>e.k==='recover'))throw Error('poison must stop actual healing');
  a.lineage=await a.engine.load(before);a.adoptSets();a.watchMode='one';a.go({kind:'watch'});
  return{recoveries:recovers.length,foeHP:foe.hp};
 });
 await page.waitForFunction(()=>[...document.querySelectorAll('.combat-lines li')].some(e=>e.textContent.includes('leeching')),null,{timeout:20000});
 await page.getByRole('button',{name:'Pause watch',exact:true}).click();
 const log=page.locator('.combat-lines li').filter({hasText:'leeching'});assert.ok(await log.count()>0);assert.ok(await log.first().locator('.log-heal').count());
 await page.evaluate(async()=>{const a=window.__riddle;a.go({kind:'camp'});const s=JSON.parse(await a.engine.save());s.run=null;s.lineage.ended=true;s.lineage.best_depth=34;s.lineage.endgame={tier:5,unlocked:6,cleared:5};a.lineage=await a.engine.load(JSON.stringify(s));a.adoptSets();a.go({kind:'ending'});});
 const endingSave=await page.evaluate(()=>window.__riddle.engine.save());
 await page.locator('.descent-choice').click();await page.waitForFunction(()=>!document.querySelector('.descent-confirm').disabled);
 await page.locator('.descent-elites summary').click();assert.match(await page.locator('.descent-elites').textContent(),/Leeching.*Melee hits restore up to2 HP unless poisoned.*Poison or fight at range/s);
 assert.equal(await page.evaluate(()=>window.__riddle.engine.save()),endingSave);await page.locator('.descent-review .chips button').first().click();
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);assert.deepEqual(errors,[]);
 console.log(width,JSON.stringify(evidence),'real-worker recovery/reload/poison/replay seek/log colour/viewport PASS');await page.close();
}}finally{await browser.close();}
