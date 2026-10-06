import {execFileSync} from 'node:child_process';import {readFileSync} from 'node:fs';import {launchBrowser} from '../../tools/browser.mjs';
const record=JSON.parse(readFileSync(new URL('./fixtures/captain-rally.json',import.meta.url),'utf8'));
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [400,1000]){
 const p=await b.newPage({viewport:{width,height:850}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const checks=await p.evaluate(async record=>{
  const {createViewer}=await import('/src/render/index.ts');let checks=0;const check=(ok,label)=>{if(!ok)throw Error(label);checks++;};
  document.body.replaceChildren();const host=document.createElement('div');host.style.cssText='position:relative;height:850px;width:100%';const c=document.createElement('canvas');c.style.cssText='display:block;width:100%;height:100%';host.append(c);document.body.append(host);
  const v=createViewer(c,{baseTexels:72});v.setSpeed(0);const paint=async()=>{await new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));};
  v.load(record.snapshot);v.setFrame('fight');v.setSpeed(0);await new Promise(r=>setTimeout(r,500));await paint();
  const tag=v.debugLabels().find(x=>x.id===record.captain.id);check(tag?.text==='captain · calls goblins','real rally snapshot text/name '+JSON.stringify(v.debugLabels()));check(tag.x-tag.w/2>=0&&tag.x+tag.w/2<=innerWidth,'warning nameplate within canvas');check(!!host.querySelector('.rtag.warning')&&getComputedStyle(host.querySelector('.rtag.warning b')).color==='rgb(242, 176, 103)','warning styling');
  const s={...record.snapshot,w:36,h:22,vision:100,tiles:Array(36*22).fill('floor'),seen:Array(36*22).fill(true),visible:Array(36*22).fill(true),items:[],overlays:[],hero:{...record.snapshot.hero,x:17,y:11},entities:[]};
  s.entities=Array.from({length:6},(_,i)=>({id:20+i,kind:'rat',x:15+(i%3),y:9+Math.floor(i/3),hp:4,max_hp:4,tags:[]}));s.entities.push({...record.captain,x:21,y:11});
  v.load(s);v.setFrame('fight');await paint();check(v.debugLabels().some(x=>x.id===10&&x.text==='captain · calls goblins'),'farther warning source wins crowded budget');check(v.debugLabels().filter(x=>x.id>=0).length<=5,'ordinary crowd budget unchanged');
  v.setQuiet(true);await paint();check(v.debugLabels().filter(x=>x.id>=0).length<=1&&v.debugLabels().some(x=>x.id===10),'quiet view keeps warning priority and one plate');v.setQuiet(false);
  v.apply([{t:s.turn+1,k:'attack',src:10,dst:s.hero.id,hit:true,dmg:1}]);v.seek(s.turn+1);await paint();check(!v.debugLabels().some(x=>x.text.includes('calls goblins'))&&!host.querySelector('.rtag.warning:not([style*="display: none"])'),'attack clears displayed warning');
  v.load({...s,entities:s.entities.map(e=>e.id===10?{...e,telegraph:undefined}:e)});v.apply([{t:s.turn+1,k:'telegraph',id:10,what:'rallies'}]);v.seek(s.turn+1);await paint();check(v.debugLabels().some(x=>x.text==='captain · calls goblins'),'live event carries warning text');v.seek(s.turn+100);await paint();check(!v.debugLabels().some(x=>x.text.includes('calls goblins')),'warning expires with glyph');
  v.load(s);v.apply([{t:s.turn+1,k:'move',id:10,x:20,y:11}]);v.seek(s.turn+1);await paint();check(!v.debugLabels().some(x=>x.text.includes('calls goblins')),'movement clears displayed warning');
  for(const kind of ['remembered','hidden','dead']){
   const f={...s,visible:[...s.visible],entities:s.entities.map(e=>e.id===10?{...e,remembered:kind==='remembered'}:e)};if(kind==='hidden'){f.visible[11*36+21]=false;f.vision=2;}v.load(f);
   if(kind==='dead'){v.apply([{k:'die',t:s.turn+1,id:10,by:'hero'}]);v.seek(s.turn+1);}await paint();check(!v.debugLabels().some(x=>x.id===10),kind+' warning source unlabelled');
  }
  const boss={id:50,kind:'goblin_warlord',x:18,y:12,hp:40,max_hp:40,tags:['boss']};v.load({...s,entities:[...s.entities,boss]});await paint();check(!v.debugLabels().some(x=>x.id===10)&&v.debugLabels().some(x=>x.id===50),'boss still owns hostile nameplate');
  v.load({...s,entities:[{...boss,telegraph:'rallies'}]});await paint();check(v.debugLabels().some(x=>x.id===50&&x.text==='warlord · calls goblins'),'Warlord warns of actual summons');
  v.load({...s,entities:[{...boss,kind:'ogre',tags:[],telegraph:'rallies'}]});await paint();check(v.debugLabels().some(x=>x.text==='ogre · rallies'),'unrelated actor keeps raw mechanic');
  v.load({...s,entities:[...s.entities.map(e=>e.id===10?{...e,ally:true}:e),boss]});await paint();check(v.debugLabels().some(x=>x.id===10&&!x.text.includes('calls goblins')),'ally retains its own name despite boss, without hostile warning');v.dispose();return checks;
 },record);if(errors.length)throw Error(errors.join('\n'));console.log(width,checks,'recorded rally/priority/lifetime/visibility checks PASS');await p.close();
}}finally{await b.close();}
