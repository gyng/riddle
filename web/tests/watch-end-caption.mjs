import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const b=await launchBrowser();
try{
 const p=await b.newPage(),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=5011`);await p.waitForFunction(()=>window.__riddle?.booted);
 const result=await p.evaluate(async()=>{
  const {ReplayState}=await import('/src/render/state.ts');
  const s=await window.__riddle.engine.send(),r=new ReplayState(),checks=[];
  const check=(ok,name)=>{if(!ok)throw Error(name);checks.push(name);};
  const rule=t=>({k:'rule',t,row:0,text:'attack weakest',verb:{v:'attack',a:'weakest'}});
  const play=events=>{r.apply(events);r.tick(100,performance.now());};
  r.load(s);play([rule(s.turn+1)]);check(!!r.caption,'live caption');
  play([{k:'die',t:s.turn+2,id:s.entities[0]?.id??s.hero.id+1,cause:'hero'}]);check(!!r.caption,'enemy death retains action');
  play([{k:'die',t:s.turn+3,id:s.hero.id,cause:'blade'}]);check(!r.caption,'hero death clears action at picture tick');
  play([rule(s.turn+4)]);check(!r.caption,'late rule cannot restore death caption');
  r.seek(s.turn+1);check(!!r.caption,'seek to living action restores caption');
  r.seek(s.turn+3);check(!r.caption,'seek to death stays clear');
  for(const tier of ['bank','return','death']){
   r.load(s);play([rule(s.turn+1)]);check(!!r.caption,`${tier} live caption`);
   // Queueing the exit must not clear an earlier watched action.
   r.apply([{k:'exit',t:s.turn+3,tier,loot_kept:0}]);check(!!r.caption,`${tier} engine lead retains action`);
   r.tick(200,performance.now());check(!r.caption,`${tier} exit clears action`);
   play([rule(s.turn+4)]);check(!r.caption,`${tier} late caption stays clear`);
  }
  r.load({...s,depth:s.depth+1});play([rule(s.turn+1)]);check(!!r.caption,'new floor resets end state');
  return checks;
 });
 if(errors.length)throw Error(errors.join('\n'));console.log(result.length,'picture-clock/end/seek/reset caption checks PASS');
}finally{await b.close();}
