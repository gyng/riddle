// Centered management windows must survive expanded content and viewport changes.
import {execFileSync} from 'node:child_process';
import assert from 'node:assert/strict';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();let checks=0;const errors=[];
try {for(const width of [320,400,768,1440]) {
 const p=await browser.newPage({viewport:{width,height:900},deviceScaleFactor:1});p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&seed=3002&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{window.__riddle.go({kind:'camp'});await document.fonts.ready;});
 for(const [file,fn] of [['packages','openPackages'],['town','openHero'],['forge','openForge'],['works','openWorks'],['settings','openSettings'],['chronicle','openChronicle'],['gold','openGoldSheet']]){
  await p.evaluate(async([file,fn])=>{(await import('/src/ui/sheet.ts')).closeAllSheets();(await import(`/src/ui/${file}.ts`))[fn](window.__riddle);},[file,fn]);
  await p.waitForTimeout(550);
  // Headed GPU windows can defer the opening animation while another window is active.
  await p.evaluate(async()=>{const panel=document.querySelector('.sheet-wrap:not(.under) .sheet');await Promise.all(panel.getAnimations().map(a=>a.finished.catch(()=>{})));});
  const geometry=async()=>p.evaluate(()=>{const panel=document.querySelector('.sheet-wrap:not(.under) .sheet');const r=panel.getBoundingClientRect();const bar=document.querySelector('.topbar').getBoundingClientRect(),c=document.querySelector('.console').getBoundingClientRect();return {centerX:Math.abs(r.left+r.width/2-innerWidth/2),centerY:Math.abs(r.top+r.height/2-(bar.bottom+c.top)/2),top:r.top,bottom:r.bottom,bar:bar.bottom,console:c.top,overflow:panel.scrollWidth>panel.clientWidth+1||document.documentElement.scrollWidth>innerWidth,fontSizes:[...document.querySelectorAll('.cmd .tile .tl')].map(e=>getComputedStyle(e).fontSize)};});
  const centered=()=>p.waitForFunction(()=>{
   const panel=document.querySelector('.sheet-wrap:not(.under) .sheet');
   const bar=document.querySelector('.topbar'),console=document.querySelector('.console');
   if(!panel||!bar||!console)return false;
   const r=panel.getBoundingClientRect();
   return Math.abs(r.top+r.height/2-(bar.getBoundingClientRect().bottom+console.getBoundingClientRect().top)/2)<=2;
  },null,{timeout:2000});
  const g=await geometry();assert.ok(g.centerX<=2&&g.centerY<=2,`${width} ${file} center ${JSON.stringify(g)}`);checks++;
  assert.ok(g.top>=g.bar&&g.bottom<=g.console,`${width} ${file} overlaps chrome`);checks++;
  assert.equal(g.overflow,false,`${width} ${file} overflow`);checks++;
  assert.equal(new Set(g.fontSizes).size,1,`${width} menu fonts differ`);checks++;
  await p.evaluate(()=>{const d=document.querySelector('.sheet-wrap:not(.under) details');if(d)d.open=true;});await p.waitForTimeout(100);
  const grown=await geometry();assert.ok(grown.centerY<=2&&!grown.overflow,`${width} ${file} expansion`);checks++;
  if(file==='packages'){
   await p.evaluate(()=>{const bar=document.querySelector('main.frame > .topbar');bar.style.minHeight=`${bar.getBoundingClientRect().height+20}px`;});await centered();
   assert.ok((await geometry()).centerY<=2,`${width} HUD resize recenters`);checks++;
   await p.evaluate(()=>document.querySelector('main.frame > .topbar').style.removeProperty('min-height'));await centered();
   const restored=await geometry();assert.ok(restored.centerY<=2,`${width} HUD restore recenters ${JSON.stringify(restored)}`);checks++;
  }
  if(file==='packages'){const icons=await p.locator('.sheet-wrap .pkg .icon-socket').count();assert.ok(icons>0,'styles have icon sockets');checks++;}
  await p.keyboard.press('Escape');assert.equal(await p.locator('.sheet-wrap:not(.under)').count(),0,'Escape closes window');checks++;
 }
 // Widening an open management window recenters.
 await p.evaluate(async()=>{const {openWindow}=await import('/src/ui/sheet.ts');openWindow(()=>{const b=document.createElement('div');b.textContent='Resize';return b;});});
 await p.setViewportSize({width:width===1440?400:1440,height:800});await p.waitForTimeout(550);
 const resize=await p.evaluate(()=>{const r=document.querySelector('.sheet-wrap:not(.under) .sheet').getBoundingClientRect();return Math.abs(r.left+r.width/2-innerWidth/2);});assert.ok(resize<=2,'resize recenters');checks++;
 await p.close();
}assert.deepEqual(errors,[]);console.log(`chrome: ${checks} checks PASS`);}finally{await browser.close();}
