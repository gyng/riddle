import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,360,400,840,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=3330`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(()=>{window.__riddle.lastForecast=null;window.__riddle.go({kind:'watch'});});
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.tick);
 const result=await p.evaluate(()=>{
  const buttons=[...document.querySelectorAll('.console .cmd button')],rect=e=>e.getBoundingClientRect();
  const inside=(a,b)=>a.left>=b.left-1&&a.right<=b.right+1&&a.top>=b.top-1&&a.bottom<=b.bottom+1;
  const overlap=(a,b)=>a.left<b.right&&a.right>b.left&&a.top<b.bottom&&a.bottom>b.top;
  const labels=buttons.map(b=>{const l=b.querySelector('.tl'),r=document.createRange();r.selectNodeContents(l);return{label:l.textContent,fits:[...r.getClientRects()].every(x=>inside(x,rect(b))),touch:rect(b).width>=44&&rect(b).height>=44};});
  const extras=[...document.querySelectorAll('.console .portrait,.console .gem')];
  return{labels,fillers:document.querySelectorAll('.console .cmd .empty').length,overlap:buttons.some(b=>extras.some(e=>overlap(rect(b),rect(e)))),overflow:document.documentElement.scrollWidth>innerWidth};
 });
 if(result.labels.length!==2||result.fillers||result.overlap||result.overflow||result.labels.some(x=>!x.fits||!x.touch))throw Error(JSON.stringify({width,result}));
 if(await p.locator('.watch-speed-mode').innerText()!=='Fights only')throw Error('Default mode missing');
 await p.locator('[data-tile=speed]').click();await p.locator('.sheet [data-tile=fast]').click();
 await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.mode==='fast'&&!document.querySelector('.sheet'));
 if(await p.locator('.watch-speed-mode').innerText()!=='Fast')throw Error('Selected mode missing');
 await p.locator('.console .gem').click();await p.waitForFunction(()=>document.querySelector('.watch')?.dataset.speed==='0');
 await p.locator('.console .gem').click();await p.waitForFunction(()=>Number(document.querySelector('.watch')?.dataset.speed)>0);
 await p.locator('.console [data-tile=town]').click();await p.waitForFunction(()=>window.__riddle.screen==='camp');
 if(errors.length)throw Error(errors.join('\n'));console.log(width,'labels/touch/geometry/speed/pause/Town PASS');await p.close();
}}finally{await b.close();}
