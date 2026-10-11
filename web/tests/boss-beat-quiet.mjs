#!/usr/bin/env node
// A held beat hides the previous callout in the same task, including its actual
// DOM plate. Rendering another frame must not be required to remove the old line.
import { execFileSync } from 'node:child_process';
import { resolve } from 'node:path';
import { launchBrowser } from '../../tools/browser.mjs';

const url = execFileSync('bash', [resolve('../tools/dev.sh')], {encoding:'utf8'}).trim();
const browser = await launchBrowser();
try {
  for (const width of [400,1440]) {
    const page = await browser.newPage({viewport:{width,height:800},deviceScaleFactor:1});
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&runs=0`);
    await page.waitForFunction(() => window.__riddle?.booted);
    const result = await page.evaluate(async () => {
      const {createViewer} = await import('/src/render/index.ts');
      const s = await window.__riddle.engine.send();
      s.w=s.h=12;s.tiles=Array(144).fill('floor');s.seen=s.visible=Array(144).fill(true);
      s.items=[];s.overlays=[];s.turn=0;s.hero.x=s.hero.y=5;
      s.entities=[{id:500,kind:'goblin',name:'Krul',x:6,y:5,hp:8,max_hp:8,tags:[]}];
      const canvas=document.createElement('canvas');
      canvas.style.cssText='position:fixed;left:0;top:0;width:400px;height:660px;z-index:999';
      document.body.append(canvas);
      const viewer=createViewer(canvas,{baseTexels:100});
      viewer.setSpeed(0);viewer.load(s);viewer.setFrame('fight',{x:5,y:5,radius:4});
      const frame=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
      const visible=(selector)=>[...document.querySelectorAll(selector)].filter(el=>el.getClientRects().length&&getComputedStyle(el).display!=='none').map(el=>el.textContent);
      const shown=()=>({text:viewer.debugText(),plates:visible('.rcall')});
      let checks=0;
      const check=(ok,label,value)=>{if(!ok)throw Error(`${label}: ${JSON.stringify(value)}`);checks++;};
      try {
        for(let i=0;i<120&&!viewer.debugRects().some(rect=>rect.hero);i++)await frame();
        check(viewer.debugRects().some(rect=>rect.hero),'fixture renders hero',viewer.stats());
        for (const event of [
          {k:'callout',t:0,text:'rallied!'},
          {k:'rule',t:0,row:0,text:'attack nearest',verb:{v:'attack',who:'nearest'}}
        ]) {
          viewer.setQuiet(false);viewer.load(s);viewer.setSpeed(1);viewer.apply([event]);
          for(let i=0;i<12&&!shown().plates.length;i++)await frame();
          const before=shown(),tags=visible('.rtag');
          check(before.text.length===1&&before.plates.length===1,'message visible before hold',before);
          check(tags.length>0,'nameplate fixture visible',tags);
          viewer.setQuiet(true);
          const immediate=shown();
          check(immediate.text.length===0&&immediate.plates.length===0,'hold removes old text and DOM plate before next frame',immediate);
          check(JSON.stringify(visible('.rtag'))===JSON.stringify(tags),'hold retains monster nameplates',visible('.rtag'));
          for(let i=0;i<3;i++){await frame();const held=shown();check(!held.text.length&&!held.plates.length,'held frame has no second line',held);}
          viewer.setQuiet(false);await frame();const restored=shown();
          check(restored.text.length===1&&restored.plates.length===1,'unexpired message restores after release',restored);
        }
        viewer.setQuiet(true);
        await new Promise(resolve=>setTimeout(resolve,1600));
        viewer.setQuiet(false);await frame();const expired=shown();
        check(!expired.text.length&&!expired.plates.length,'expired message stays absent after release',expired);
        return {checks};
      } finally {viewer.dispose();canvas.remove();}
    });
    console.log(width,result.checks,'immediate callout/caption hold, frames, release, expiry and nameplates PASS');
    await page.close();
  }
} finally {await browser.close();}
