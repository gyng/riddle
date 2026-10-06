import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const b=await launchBrowser();
try {for(const width of [400,1440]) {
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&juice=1`);await p.waitForFunction(()=>window.__riddle?.booted&&document.querySelector('.ambient-mote'));
 const count=await p.locator('.ambient-mote').count();assert.ok(count>=16&&count<=36);
 const positions=()=>p.locator('.ambient-mote').evaluateAll(es=>es.map(e=>e.style.transform));
 const before=await positions();await p.waitForTimeout(800);const after=await positions();assert.equal(before.filter((s,i)=>s!==after[i]).length,count);
 const result=await p.evaluate(async()=>{
  const {advanceMote}=await import('/src/ambient-motes.ts');
  const base={x:300,y:300,vx:0,vy:-10,phase:4,lift:12};
  const calm={...base},near={...base},far={...base},other={...base,phase:1};
  const idle={x:-1000,y:-1000,vx:0,vy:0,strength:0};
  for(let i=0;i<30;i++) {
   advanceMote(calm,1/30,i/30,idle);advanceMote(other,1/30,i/30,idle);
   advanceMote(near,1/30,i/30,{x:280,y:300,vx:100,vy:0,strength:1});
   advanceMote(far,1/30,i/30,{x:0,y:0,vx:100,vy:0,strength:1});
  }return {calm,near,far,other};
 });
 assert.ok(Math.abs(result.calm.x-300)>2);assert.ok(Math.abs(result.calm.x-result.other.x)>5);
 assert.ok(result.near.x-result.calm.x>10);assert.deepEqual(result.far,result.calm);
 assert.ok(Math.abs(result.near.vx)<=75&&Math.abs(result.near.vy)<=75);
 assert.equal(await p.locator('.ambient-motes').evaluate(e=>getComputedStyle(e).pointerEvents),'none');
 await p.getByRole('button',{name:'settings',exact:true}).click();await p.locator('.settings').waitFor();
 await p.keyboard.press('Escape');
 await p.evaluate(()=>{window.oldMote=document.querySelector('.ambient-mote');window.__riddle.go({kind:'camp'});});await p.waitForTimeout(150);assert.equal(await p.locator('.ambient-motes').count(),1);
 const detached=await p.evaluate(()=>({connected:window.oldMote.isConnected,transform:window.oldMote.style.transform}));
 assert.equal(detached.connected,false);await p.waitForTimeout(100);assert.equal(await p.evaluate(()=>window.oldMote.style.transform),detached.transform,'old animation stopped');
 await p.evaluate(()=>{Object.defineProperty(document,'hidden',{configurable:true,value:true});document.dispatchEvent(new Event('visibilitychange'));});
 await p.waitForTimeout(100);assert.equal(await p.locator('.ambient-motes').count(),0);
 await p.evaluate(()=>{delete document.hidden;document.dispatchEvent(new Event('visibilitychange'));});
 await p.waitForTimeout(100);assert.equal(await p.locator('.ambient-motes').count(),1);
 await p.emulateMedia({reducedMotion:'reduce'});await p.waitForFunction(()=>matchMedia('(prefers-reduced-motion: reduce)').matches&&!document.querySelector('.ambient-motes'),{},{timeout:1000});assert.equal(await p.locator('.ambient-motes').count(),0);
 await p.emulateMedia({reducedMotion:'no-preference'});await p.waitForFunction(()=>!matchMedia('(prefers-reduced-motion: reduce)').matches&&document.querySelectorAll('.ambient-motes').length===1,{},{timeout:1000});assert.equal(await p.locator('.ambient-motes').count(),1);
 await p.evaluate(()=>{const main=document.querySelector('main.frame');main.classList.add('watch');document.getElementById('app').appendChild(main);});await p.waitForTimeout(100);assert.equal(await p.locator('.ambient-motes').count(),0);
 assert.deepEqual(errors,[]);await p.goto(`${url}?engine=fake&fresh=1&runs=0&juice=0`);await p.waitForFunction(()=>window.__riddle?.booted);assert.equal(await p.locator('.ambient-motes').count(),0);
 console.log(width,'drift/wake/locality/bounds/lifecycle/reduced-motion/off PASS');await p.close();
}}finally{await b.close();}
