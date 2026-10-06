import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const b=await launchBrowser();
try{for(const width of [400,1024,1440,1920]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=5307`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(()=>{
  const a=window.__riddle;
  const f={depths:Array.from({length:20},(_,i)=>({depth:i+1,reach:(20-i)/20,pm:.03,...(i===7?{boss:'goblin_warlord'}:{}),...(i===12?{bounty:2}:{}),...(i===17?{boss:'lich'}:{})})),known_to:20,start:5,refined:false,low:5,ends:{bank:.3,return:.2,death:.5,gold:12},causes:[]};
  a.lineage.best_depth=13;a.lineage.start=5;a.lineage.live=null;a.lineage.bounty={depth:15,pays:"$×2 · item",needs:"reach"};
  if(a.lineage.town)a.lineage.town.home=true;
  a.engine.forecastEstimate=async()=>f;a.engine.forecast=async()=>f;a.lastForecast=f;
  a.go({kind:'camp'});
 });
 const shaft=p.locator('.camp .shaft');await shaft.waitFor({state:'visible'});
 if(width>=1024){
  const geometry=await shaft.evaluate(el=>({height:el.getBoundingClientRect().height,rows:el.querySelectorAll('.depth-summary .depth-summary-row').length,overflow:document.documentElement.scrollWidth>innerWidth}));
  assert.ok(geometry.height<=320,JSON.stringify(geometry));assert.equal(geometry.rows,4);assert.equal(geometry.overflow,false);
  assert.equal(await shaft.locator('.notches').isVisible(),false);
  assert.equal(await shaft.locator('.depth-current b').innerText(),'D5');assert.equal(await shaft.locator('.depth-record b').innerText(),'D13');
  assert.match(await shaft.locator('.depth-next > small').first().innerText(),/Next encounter/);
  assert.equal(await shaft.locator('.depth-summary .progress-goal b').innerText(),'D15');assert.match(await shaft.locator('.depth-summary .progress-reward').innerText(),/2× gold.*item/);
  assert.equal(await shaft.locator('.depth-next b').innerText(),'D8');assert.match(await shaft.locator('.depth-next').innerText(),/warlord/);
  assert.match(await shaft.locator('.depth-chance').innerText(),/65%.*±3.*estimate/);
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.oaths=[{id:'depth-check',kind:'depth',chips:['D13','no rest'],text:'D13 no rest',price:0,sworn:true,reward:{kind:'card',id:'gas_step',label:'gas step'}}];a.emitChange();});
  assert.equal(await shaft.locator('.shaft-oath').isVisible(),true);
  const challengeSize=await shaft.evaluate(el=>({height:el.getBoundingClientRect().height,oath:el.querySelector('.shaft-oath').getBoundingClientRect().height,summary:el.querySelector('.depth-summary').getBoundingClientRect().height}));
  assert.ok(challengeSize.height<=320,'challenge must fit compact plate '+JSON.stringify(challengeSize));
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.oaths=[];a.emitChange();});
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.live={run_id:1,heir:a.lineage.heir,depth:9,start:5,hp:30,max_hp:36,turn:100};a.emitLive();});
  assert.equal(await shaft.locator('.depth-summary .progress-goal b').innerText(),'D15','progress goal does not follow the current picture floor');
  assert.equal(await shaft.locator('.depth-current b').innerText(),'D9');assert.equal(await shaft.locator('.depth-next b').innerText(),'D13');assert.match(await shaft.locator('.depth-next').innerText(),/2× gold/);
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.best_depth=21;a.lineage.bounty={depth:23,pays:'$×2 · item',boss:'unseen_enemy'};a.emitChange();});
  assert.equal(await shaft.locator('.depth-summary .progress-goal b').innerText(),'D23','goal changes with selected lineage metadata even with unchanged forecast');
  assert.ok(!(await shaft.locator('.depth-summary .progress-goal').innerText()).includes('unseen_enemy'));
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.bounty={depth:13,pays:'$×2 · item'};a.emitChange();});
  assert.equal(await shaft.locator('.depth-summary .progress-goal b').innerText(),'D22');
  assert.equal(await shaft.locator('.depth-summary .progress-reward').innerText(),'New record');
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.best_depth=13;a.lineage.bounty={depth:15,pays:'$×2 · item'};a.emitChange();});
 }else{
  assert.equal(await shaft.locator('.depth-mobile-goal').isVisible(),true);assert.equal(await shaft.locator('.depth-mobile-goal b').innerText(),'D15');
  assert.equal(await shaft.locator('.depth-summary').isVisible(),false);assert.equal(await shaft.locator('.notches').isVisible(),true);
 }
 await shaft.click();await p.locator('.forecast .fc-bars').waitFor({state:'visible'});
 assert.match(await p.locator('.forecast .fc-bars').innerText(),/D8/);
 await p.keyboard.press('Escape');
 assert.equal(await p.locator('.forecast .fc-bars').isVisible(),false);
 // The watch's detail action must work, but cannot apply a tactic to a live run.
 if(width>=1024){
  await p.evaluate(()=>{const a=window.__riddle;a.watchMode='one';a.lastForecast.depths[8].try={row:{conds:[],verb:{v:'attack',a:'boss'}},text:'attack boss'};a.go({kind:'watch'});});
  await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
  await p.locator('.console .gem').click();
  await p.locator('.watch .side-col .shaft').click();
  await p.locator('.depth-reference').waitFor({state:'visible'});
  assert.ok(await p.locator('.depth-reference .fc-bars .try').count()>0,'read-only reference retains suggested action');
  assert.equal(await p.locator('.depth-reference .fc-bars button').count(),0);
  await p.keyboard.press('Escape');assert.equal(await p.locator('.depth-reference').count(),0);
 }
 const model=await p.evaluate(async()=>{
  const {progressGoal}=await import('/src/ui/progress-goal.ts'),base=window.__riddle.lineage;
  return {empty:progressGoal({...base,town:{...base.town,home:false}}),fresh:progressGoal({...base,best_depth:0}),
   ended:progressGoal({...base,ended:true}),fallback:progressGoal({...base,best_depth:21,bounty:{depth:13,pays:'$×2 · item'}}),
   late:progressGoal({...base,best_depth:21,bounty:{depth:23,pays:'$×2 · item',boss:'unseen_enemy'}}),
   oldWire:progressGoal({...base,best_depth:21,bounty:{depth:23}})};
 });
 assert.equal(model.empty,null);assert.equal(model.fresh,null);assert.equal(model.ended,null);
 assert.deepEqual(model.fallback,{depth:22});assert.deepEqual(model.late,{depth:23,reward:'2× gold · item'});
 assert.equal(model.oldWire.depth,23);assert.equal(model.oldWire.reward,undefined);assert.ok(!JSON.stringify(model).includes('unseen_enemy'),'no unseen boss attribution');
 if(width<1024){
  await p.evaluate(async()=>{const a=window.__riddle,report=await a.engine.runOfflineQuick(0);a.go({kind:'report',report,absence:true});});
  await p.locator('.report-progress-goal').waitFor({state:'visible'});
  assert.equal(await p.locator('.report-progress-goal b').innerText(),'D15');
 }
 assert.deepEqual(errors,[]);console.log(width,'compact geometry/start/milestones/uncertainty/reference/mobile/read-only PASS');await p.close();
}}finally{await b.close();}
