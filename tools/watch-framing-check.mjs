#!/usr/bin/env node
// Camera containment on the actual renderer/GPU; independent of game simulation.
// node tools/watch-framing-check.mjs (not a frame-rate benchmark)
import {execFileSync} from 'node:child_process';import {launchGpu} from './browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{encoding:'utf8'}).trim(),b=await launchGpu();
try{for(const width of [400,1440]){const p=await b.newPage({viewport:{width,height:900},deviceScaleFactor:1.5});await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 const result=await p.evaluate(async({width,renderPath})=>{
  const {createViewer}=await import(renderPath),s=await window.__riddle.engine.send();
  s.w=s.h=64;s.tiles=Array(4096).fill('floor');s.seen=s.visible=Array(4096).fill(true);s.items=[];s.overlays=[];s.entities=[];s.turn=0;s.hero.x=s.hero.y=2;
  const c=document.createElement('canvas');c.style.cssText=`position:fixed;left:0;top:0;width:${width<1024?400:920}px;height:660px;z-index:999`;document.body.append(c);
  const v=createViewer(c,{baseTexels:width<1024?100:72});v.setSpeed(0);v.load(s);let checks=0,samples=0;
  const frame=()=>new Promise(r=>requestAnimationFrame(()=>requestAnimationFrame(r)));
  const visible=(label)=>{const h=v.debugRects().find(r=>r.hero),r=c.getBoundingClientRect();if(!h||h.x<-.5||h.y<-.5||h.x+h.w>r.width+.5||h.y+h.h>r.height+.5)throw Error(`${label}: ${JSON.stringify({h,view:[r.width,r.height],stats:v.stats()})}`);samples++;return h;};
  try{for(let i=0;i<180&&!v.debugRects().some(r=>r.hero);i++)await frame();visible('loaded');checks++;
   for(const mode of ['map','fight']){
    v.load(s);v.setFrame(mode,mode==='fight'?{x:55,y:55,radius:3}:undefined);v.setSpeed(32);
    const path=[[55,2],[55,55],[2,55],[2,2],[55,55],[2,55],[55,2],[32,32]];
    v.apply(path.map(([x,y],i)=>({k:'move',id:s.hero.id,x,y,t:(i+1)*80})));
    const positions=new Set();for(let i=0;i<32;i++){await frame();visible(`${mode} fast`);positions.add(v.debugPos().find(e=>e.hero).x+','+v.debugPos().find(e=>e.hero).y);}
    if(positions.size<3)throw Error('movement fixture did not exercise travel');checks++;
    v.setSpeed(0);for(const t of [80,400,640]){v.seek(t);await frame();visible(`${mode} seek ${t}`);checks++;}
    c.style.height='280px';c.style.width=width<1024?'320px':'720px';v.resize();await frame();visible(`${mode} resize`);checks++;
    v.load(s);await frame();const first=visible('stationary');for(let i=0;i<5;i++){await frame();const h=visible('stationary');if(h.x!==first.x||h.y!==first.y)throw Error('stationary camera jitters '+JSON.stringify({mode,first,h,stats:v.stats(),pos:v.debugPos()}));}checks++;
    c.style.height='660px';c.style.width=width<1024?'400px':'920px';v.resize();await frame();visible(`${mode} restore`);checks++;
   }
   return{checks,samples};
  }finally{v.dispose();c.remove();}
 },{width,renderPath:process.env.RIDDLE_RENDER_MODULE??'/src/render/index.ts'});console.log(width,result.checks,'camera cases',result.samples,'contained GPU samples PASS');await p.close();}}finally{await b.close();}
