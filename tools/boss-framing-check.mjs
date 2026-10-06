#!/usr/bin/env node
// Actual renderer/GPU geometry QA, not a frame-rate benchmark.
import {execFileSync} from 'node:child_process';import {readFileSync} from 'node:fs';import {launchGpu} from './browser.mjs';
const record=JSON.parse(readFileSync(new URL('../web/tests/fixtures/warlord-rally.json',import.meta.url)));
const url=execFileSync('bash',['tools/dev.sh'],{encoding:'utf8'}).trim(),b=await launchGpu();
try{for(const width of (process.env.RIDDLE_BOSS_WIDTHS??'400,1440').split(',').map(Number)){
 const p=await b.newPage({viewport:{width,height:900},deviceScaleFactor:1.5});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const result=await p.evaluate(async({record,width,module})=>{
  const {createViewer}=await import(module);document.body.replaceChildren();
  const c=document.createElement('canvas');c.style.cssText=`display:block;width:${width===400?400:914}px;height:${width===400?682:645}px`;document.body.append(c);
  const v=createViewer(c,{baseTexels:width===400?100:72});v.setSpeed(0);let checks=0,samples=0;
  const paint=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
  const contained=(boss=true)=>{const {width:w,height:h}=c.getBoundingClientRect(),rs=v.debugRects().filter(r=>r.hero||(boss&&r.kind==='goblin_warlord'));
   if(!rs.some(r=>r.hero)||boss&&!rs.some(r=>r.kind==='goblin_warlord'))throw Error('missing combatant');
   for(const r of rs){const drawn=r.drawn??r;if(drawn.x<-.5||drawn.y<-.5||drawn.x+drawn.w>w+.5||drawn.y+drawn.h>h+.5)throw Error('clipped '+JSON.stringify({r,w,h,stats:v.stats()}));}samples++;};
  const check=(ok,msg)=>{if(!ok)throw Error(msg);checks++;};
  try{
   v.load(record);v.setFrame('fight');await new Promise(r=>setTimeout(r,800));await paint();contained();checks++;
   const open={...record,w:64,h:64,tiles:Array(4096).fill('floor'),seen:Array(4096).fill(true),visible:Array(4096).fill(true),items:[],overlays:[],hero:{...record.hero,x:32,y:32},entities:[]};
   const boss={...record.entities.find(e=>e.kind==='goblin_warlord'),id:500,x:32,y:27};
   for(const [dx,dy] of [[0,-5],[5,0],[0,5],[-5,0]]){
    const s={...open,entities:[{...boss,x:32+dx,y:32+dy}]};v.load(s);v.setSpeed(0);await paint();contained();checks++;
    const scale=v.stats().k;v.apply([{k:'move',t:s.turn+1,id:500,x:33,y:32}]);v.seek(s.turn+1);await paint();contained();check(v.stats().k===scale,'approach must not repeatedly zoom in');
   }
   v.load({...open,entities:[boss]});await paint();const wide=v.stats().k;
   v.apply([{k:'die',t:open.turn+1,id:500,cause:'hero'}]);v.seek(open.turn+1);await paint();contained(false);check(v.stats().k===wide,'death holds encounter scale');
   v.load({...open,depth:9});await paint();const normal=v.stats().k;contained(false);check(normal>=wide,'new floor restores normal fight scale');
   for(const state of ['hidden','remembered','ally']){
    const s={...open,entities:[{...boss,remembered:state==='remembered',ally:state==='ally'}]};if(state==='hidden'){s.visible=Array(4096).fill(false);s.vision=1;}
    v.load(s);await paint();contained(false);check(v.stats().k===normal,state+' boss cannot alter zoom');
   }
   v.load({...open,entities:[boss]});v.setFrame('fight',{x:32,y:32,radius:3});await paint();contained(false);check(v.stats().k===normal,'fixed focus retains its scale');v.setFrame('fight');
   c.style.width='320px';c.style.height='280px';v.resize();v.load({...open,entities:[boss]});await paint();contained();checks++;
   c.style.width=width===400?'400px':'914px';c.style.height=width===400?'682px':'645px';v.resize();await paint();contained();checks++;
   v.load({...open,entities:[{...boss,id:600}]});v.apply([{k:'note',t:open.turn+100,text:'entrance sample'}]);v.setSpeed(1);
   let lifted=false;for(let i=0;i<40;i++){await paint();contained();lifted ||= v.debugRects().some(r=>r.kind==='goblin_warlord'&&r.drawn&&r.drawn.y<r.y-1);}check(lifted,'entrance samples must exercise actual drawn lift');
   v.setSpeed(0);await paint();const settled=v.stats().k;for(let i=0;i<5;i++){await paint();contained();check(v.stats().k===settled,'settled scale jitters');}
   return{checks,samples};
  }finally{v.dispose();}
 },{record,width,module:process.env.RIDDLE_RENDER_MODULE??'/src/render/index.ts'});console.log(width,result.checks,'boss framing cases',result.samples,'contained GPU samples PASS');await p.close();
}}finally{await b.close();}
