import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:800}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(()=>{window.__riddle.watchMode='one';window.__riddle.go({kind:'watch'});});await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 await p.locator('.console .gem').click();await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.speed==='0');
 await p.evaluate(()=>{
  const stack=document.querySelector('.watch-messages');window.messages=stack;
  for(const name of ['ticker','beat-why','why-tip']){const e=stack.querySelector(`.${name}`);e.classList.remove('show');}
  const list=stack.querySelector('.combat-lines');list.replaceChildren();
 });
 assert.equal(await p.locator('.combat-log').isVisible(),false);
 assert.equal(await p.locator('.ticker').isVisible(),false);
 assert.equal(await p.locator('.beat-why').isVisible(),false);
 for(const cause of ['slain · spectral blade','slain · exceptionally ancient spectral blade']){
  await p.evaluate(cause=>{
   const stack=window.messages;
   for(const [name,text] of [['ticker','D6 · died $0 · $102 lost'],['beat-why',cause],['why-tip','return blocked → foes too near']]){
    const e=stack.querySelector(`.${name}`);e.textContent=text;e.classList.add('show');
   }
   stack.querySelector('.combat-lines').replaceChildren(...['D6 · spectral blade → Wren Ash · −1 hp','D6 · Wren Ash · fell','D6 · died $0 · $102 lost · bones: 10 items on D6'].map(text=>{const li=document.createElement('li');li.textContent=text;return li;}));
  },cause);
  await p.waitForTimeout(250);
  const geometry=await p.evaluate(()=>{
   const stack=window.messages, rect=e=>{const r=e.getBoundingClientRect();return {top:r.top,bottom:r.bottom,left:r.left,right:r.right,height:r.height};};
   return {rows:[...stack.children].filter(e=>getComputedStyle(e).display!=='none').map(rect),stage:rect(document.querySelector('.stage')),background:getComputedStyle(stack).backgroundColor,log:getComputedStyle(stack.querySelector('.combat-log')).backgroundColor,overflow:document.documentElement.scrollWidth>innerWidth};
  });
  assert.equal(geometry.rows.length,4);
  for(let i=1;i<geometry.rows.length;i++)assert.ok(geometry.rows[i].top-geometry.rows[i-1].bottom>=7.5,JSON.stringify(geometry));
  for(const r of geometry.rows)assert.ok(r.left>=geometry.stage.left&&r.right<=geometry.stage.right&&r.top>=geometry.stage.top&&r.bottom<=geometry.stage.bottom,JSON.stringify(geometry));
  assert.equal(geometry.background,'rgba(0, 0, 0, 0)');assert.equal(geometry.log,'rgba(0, 0, 0, 0)');assert.equal(geometry.overflow,false);
 }
 // Suppressed fight captions and cut tickers must not leave blank rows.
 await p.evaluate(()=>{document.querySelector('.watch').dataset.frame='fight';window.messages.querySelector('.ticker').className='ticker show kill';});
 assert.equal(await p.locator('.ticker').isVisible(),false);
 await p.evaluate(()=>{document.querySelector('.watch').dataset.frame='map';window.messages.querySelector('.ticker').className='ticker show cut';});
 assert.equal(await p.locator('.ticker').isVisible(),false);
 await p.evaluate(()=>window.messages.querySelector('.ticker').classList.remove('cut'));
 // Interactive ticker descendants still receive clicks through the transparent stack.
 await p.evaluate(()=>{const t=window.messages.querySelector('.ticker');t.classList.add('has-why');t.onclick=()=>{window.tickerTapped=true;};});
 await p.locator('.watch-messages .ticker').click();assert.equal(await p.evaluate(()=>window.tickerTapped),true);
 await p.evaluate(()=>{const list=window.messages.querySelector('.combat-lines');list.append(...Array.from({length:20},()=>{const li=document.createElement('li');li.textContent='D6 · gold';return li;}));list.scrollTop=0;});
 await p.locator('.combat-lines').hover();await p.mouse.wheel(0,90);await p.waitForTimeout(150);assert.ok(await p.locator('.combat-lines').evaluate(e=>e.scrollTop>0));
 await p.locator('.console [data-tile=speed]').click();await p.locator('.watch-options').waitFor();await p.keyboard.press('Escape');
 assert.deepEqual(errors,[]);console.log(width,'wrapped messages/gaps/bounds/plain overlay/ticker/history/controls PASS');await p.close();
}}finally{await b.close();}
