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
  a.lineage.best_depth=13;a.lineage.start=5;a.lineage.live=null;
  if(a.lineage.town)a.lineage.town.home=true;
  a.engine.forecastEstimate=async()=>f;a.engine.forecast=async()=>f;a.lastForecast=f;
  a.go({kind:'camp'});
 });
 const shaft=p.locator('.camp .shaft');await shaft.waitFor({state:'visible'});
 if(width>=1024){
  const geometry=await shaft.evaluate(el=>({height:el.getBoundingClientRect().height,rows:el.querySelectorAll('.depth-summary-row').length,overflow:document.documentElement.scrollWidth>innerWidth}));
  assert.ok(geometry.height<=320,JSON.stringify(geometry));assert.equal(geometry.rows,3);assert.equal(geometry.overflow,false);
  assert.equal(await shaft.locator('.notches').isVisible(),false);
  assert.equal(await shaft.locator('.depth-current b').innerText(),'D5');assert.equal(await shaft.locator('.depth-record b').innerText(),'D13');
  assert.equal(await shaft.locator('.depth-next b').innerText(),'D8');assert.match(await shaft.locator('.depth-next').innerText(),/warlord/);
  assert.match(await shaft.locator('.depth-chance').innerText(),/65%.*±3.*estimate/);
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.oaths=[{id:'depth-check',kind:'depth',chips:['D13','no rest'],text:'D13 no rest',price:0,sworn:true,reward:{kind:'card',id:'gas_step',label:'gas step'}}];a.emitChange();});
  assert.equal(await shaft.locator('.shaft-oath').isVisible(),true);
  const challengeSize=await shaft.evaluate(el=>({height:el.getBoundingClientRect().height,oath:el.querySelector('.shaft-oath').getBoundingClientRect().height,summary:el.querySelector('.depth-summary').getBoundingClientRect().height}));
  assert.ok(challengeSize.height<=320,'challenge must fit compact plate '+JSON.stringify(challengeSize));
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.oaths=[];a.emitChange();});
  await p.evaluate(()=>{const a=window.__riddle;a.lineage.live={run_id:1,heir:a.lineage.heir,depth:9,start:5,hp:30,max_hp:36,turn:100};a.emitLive();});
  assert.equal(await shaft.locator('.depth-current b').innerText(),'D9');assert.equal(await shaft.locator('.depth-next b').innerText(),'D13');assert.match(await shaft.locator('.depth-next').innerText(),/2× gold/);
 }else{
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
 assert.deepEqual(errors,[]);console.log(width,'compact geometry/start/milestones/uncertainty/reference/mobile/read-only PASS');await p.close();
}}finally{await b.close();}
