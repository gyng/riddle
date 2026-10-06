import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(), b=await launchBrowser();
try {for(const width of [320,400,1440]) {
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const checks=await p.evaluate(async()=>{
  const {tacticObserver}=await import('/src/ui/tactic-observer.ts'),{ReplayState}=await import('/src/render/state.ts'),{verbIcon}=await import('/src/ui/skin.ts');
  let n=0;const check=(ok,msg)=>{if(!ok)throw Error(msg);n++;};
  const rows=[{origin:'stance:steady',conds:[],verb:{v:'attack',a:'nearest'}},{origin:'tactic:boss_focus',conds:[{k:'foe_tag',t:'boss'}],verb:{v:'attack',a:'tag:boss'}},{origin:'stance:steady',conds:[{k:'hp<',n:25}],verb:{v:'return'}},{conds:[],verb:{v:'rest'}}];
  const catalog={all:[{id:'steady',name:'Steady',kind:'stance'},{id:'boss_focus',name:'Boss focus',kind:'tactic'}]};
  const o=tacticObserver(rows,catalog),host=document.createElement('div');host.className='watch';host.append(o.el);document.body.append(host);
  const fire=(row,t=1)=>({k:'rule',row,t,text:'attack boss',verb:rows[row].verb});
  check(o.el.children.length===2,'bounded owned icons');check(!o.owns(-1)&&!o.owns(3)&&!o.owns(99),'unknown/trait/chore never guessed');
  const steady=o.el.querySelector('[data-origin="stance:steady"]'),boss=o.el.querySelector('[data-origin="tactic:boss_focus"]');
  await new Promise(requestAnimationFrame);for(const cue of [steady,boss]){const box=cue.getBoundingClientRect(),img=cue.querySelector('img').getBoundingClientRect();check(img.width<=box.width&&img.height<=box.height,'existing icon fits compact cue');}
 o.fire(fire(0),1000);check(!steady.classList.contains('acting'),'default swings quiet');
  o.fire(fire(1),1000);check(boss.classList.contains('acting'),'conditional attack lights tactic');o.fire(fire(1),1100);check(boss.dataset.count==='2','repeats counted');
  check(boss.title.includes('Boss focus')&&boss.title.includes('2 activations'),'hover count/name');
  rows[1].origin='stance:steady';rows[1].conds=[];catalog.all[1].name='Changed';o.fire(fire(1),1200);check(boss.title.includes('Boss focus')&&boss.title.includes('boss'),'mounted metadata frozen');
  boss.click();check(document.querySelector('.tactic-detail')?.textContent.includes('3 activations'),'tap count');check(document.querySelector('.tactic-detail')?.textContent.includes('boss'),'tap condition');
  document.querySelector('.sheet-wrap .close-stud').click();
  o.fire(fire(2),2500);check(steady.title.includes('return'),'meaningful style condition shown');o.fire(fire(0),2600);check(steady.title.includes('return'),'routine swing retains meaningful choice');
  await new Promise(resolve=>setTimeout(resolve,950));check(!boss.classList.contains('acting'),'merged glow expires');
  o.end();check(!boss.classList.contains('acting'),'end clears glow');o.fire(fire(1),9000);check(boss.dataset.count==='3','late end event ignored');
  const fallback=tacticObserver([{origin:'stance:steady',conds:[{k:'foes>=',n:1}],verb:{v:'attack',a:'nearest'}}],catalog);fallback.fire({...fire(0),row:0},1000);check(!fallback.el.firstElementChild.classList.contains('acting')&&fallback.meaningfulRows.length===0,'actual core fallback quiet');fallback.dispose();
  o.disable();check(o.el.hidden&&!o.owns(1),'legacy changed indices disable');o.dispose();host.remove();
  for(const [id,verb,name] of [['sentinel','riposte','Sentinel'],['hexbinder','hex','Hexbinder']]) {
   const row={origin:`style:${id}`,conds:[{k:'foes>=',n:1}],verb:{v:verb}};
   const style=tacticObserver([row],catalog),cue=style.el.firstElementChild,styleHost=document.createElement('div');styleHost.className='watch';styleHost.append(style.el);document.body.append(styleHost);await new Promise(requestAnimationFrame);
   check(verbIcon(verb)===`style_${id}`,'class action uses the same icon');
   check(style.owns(0)&&style.meaningfulRows.includes(0),'class action owned and meaningful');
   check(cue.querySelector('img')&&cue.title.includes(name),'class icon and name');
   const image=cue.querySelector('img').getBoundingClientRect(),box=cue.getBoundingClientRect();check(image.width<=box.width&&image.height<=box.height,'class icon fits cue');
   style.fire({k:'rule',row:0,t:1,text:verb,verb:row.verb},1000);
   check(cue.classList.contains('acting')&&cue.dataset.count==='1','class event observable');style.dispose();styleHost.remove();
  }
  const s=await window.__riddle.engine.send();const foe=s.entities[0]??{...s.hero,id:99,kind:'goblin',tags:[],ally:false};s.entities=[foe];
  const r=new ReplayState();r.setTacticRows([0,1,2],[1,2]);r.load(s);
  const tick=t=>r.seek(t),rule=t=>({...fire(1),t}),attack=t=>({k:'attack',t,src:s.hero.id,dst:foe.id,dmg:1,hit:true});
  r.apply([rule(s.turn+1),attack(s.turn+1)]);check(!r.tacticTarget,'engine-ahead pair not shown');tick(s.turn+1);
  check(!r.caption,'known caption replaced');check(r.tacticMarked(r.ents.get(foe.id)),'real target marked');tick(s.turn+6);check(!r.tacticMarked(r.ents.get(foe.id)),'target expiry');
  r.load(s);r.apply([attack(s.turn+1),rule(s.turn+1)]);tick(s.turn+1);check(r.tacticMarked(r.ents.get(foe.id)),'reverse ordering');
  r.load(s);r.apply([{...fire(0),t:s.turn+1},attack(s.turn+1)]);tick(s.turn+1);check(!r.tacticTarget,'default no target');
  r.load(s);r.apply([{...fire(2),t:s.turn+1},attack(s.turn+1)]);tick(s.turn+1);check(!r.tacticTarget,'non-attack rule never claims target');
  r.load(s);r.apply([rule(s.turn+1),attack(s.turn+2)]);tick(s.turn+2);check(!r.tacticTarget,'tick mismatch no target');
  r.load(s);r.apply([rule(s.turn+1),{...attack(s.turn+1),src:foe.id,dst:s.hero.id}]);tick(s.turn+1);check(!r.tacticTarget,'enemy attack no target');
  r.load(s);r.apply([{...fire(3),t:s.turn+1}]);tick(s.turn+1);check(!!r.caption,'unknown rows retain caption');
  r.load(s);r.apply([rule(s.turn+1),attack(s.turn+1),{k:'die',id:s.hero.id,t:s.turn+2,cause:'blade'}]);tick(s.turn+2);check(!r.tacticMarked(r.ents.get(foe.id)),'death clears marking');
  r.load(s);check(!r.tacticTarget,'load clears target');return n;
 });
 if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'tactic ownership/clock/target/read-only checks PASS');await p.close();
}}finally{await b.close();}
