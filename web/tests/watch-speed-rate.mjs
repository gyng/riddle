// Blind c4705f9 (A, B: `fights only` persisted across runs at 16× and both raters believed they watched at 1×): the speed tile always
// carries the clock the picture plays at (`16×`, `2×`, `1×`), lit whenever it is not 1×, and a remembered mode reads so on the next run.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
const shots=process.env.SPEED_SHOTS;
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 const check=(v,l)=>{if(!v)throw Error(`${width}: ${l}`);};
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3330`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(()=>{window.__riddle.lastForecast=null;window.__riddle.go({kind:'watch'});});
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 const read=()=>p.evaluate(()=>{const t=document.querySelector('.console [data-tile=speed]'),r=t.querySelector('.watch-speed-rate'),l=t.querySelector('.tl');
  const a=r.getBoundingClientRect(),tb=t.getBoundingClientRect(),rg=document.createRange();rg.selectNodeContents(l);
  const clash=[...rg.getClientRects()].some(x=>x.width>0&&x.left<a.right&&x.right>a.left&&x.top<a.bottom&&x.bottom>a.top);
  return{rate:r.textContent,sped:t.classList.contains('sped'),mode:t.querySelector('.watch-speed-mode').textContent,visible:a.width>0&&getComputedStyle(r).visibility!=='hidden',inside:a.left>=tb.left-1&&a.right<=tb.right+1&&a.top>=tb.top-1,clash};});
 let s=await read();
 check(s.mode==='Fights only'&&/^\d+(\.\d)?×$/.test(s.rate)&&s.visible&&s.inside&&!s.clash,`default mode shows its clock on the tile (${JSON.stringify(s)})`);
 check(s.sped===(parseFloat(s.rate)>1),`lit exactly when not 1× (${JSON.stringify(s)})`);
 if(shots)await p.locator('.console').screenshot({path:`${shots}/speed-${width}.png`});
 await p.locator('[data-tile=speed]').click();await p.locator('.sheet [data-tile=one]').click();
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.mode==='one'&&!document.querySelector('.sheet'));
 await p.waitForFunction(()=>document.querySelector('.console [data-tile=speed] .watch-speed-rate')?.textContent==='1×',null,{timeout:5000});
 s=await read();check(s.mode==='Normal'&&!s.sped,`normal reads 1× unlit (${JSON.stringify(s)})`);
 await p.locator('[data-tile=speed]').click();await p.locator('.sheet [data-tile=fast]').click();
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.mode==='fast'&&!document.querySelector('.sheet'));
 await p.waitForFunction(()=>parseFloat(document.querySelector('.console [data-tile=speed] .watch-speed-rate')?.textContent)>1,null,{timeout:5000});
 s=await read();check(s.mode==='Fast'&&s.sped,`fast reads its clock lit (${JSON.stringify(s)})`);
 // paused: the tile keeps the mode's clock rather than going blank
 await p.locator('.console .gem').click();await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.speed==='0');
 s=await read();check(/×$/.test(s.rate)&&s.sped,`paused keeps the mode's clock (${JSON.stringify(s)})`);
 // the next run remembers `fast` and says so at once
 await p.evaluate(()=>{window.__riddle.go({kind:'watch'});});await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 s=await read();check(s.mode==='Fast'&&s.sped&&parseFloat(s.rate)>1,`a remembered mode reads lit on the next run (${JSON.stringify(s)})`);
 if(errors.length)throw Error(errors.join('\n'));console.log(width,'speed rate PASS');await p.close();
}}finally{await b.close();}
console.log('watch-speed-rate: PASS');
