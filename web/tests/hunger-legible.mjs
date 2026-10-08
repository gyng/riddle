// blind c4705f9 (B: `hp 7/25 · max -26` "from hunger I never understood"): on the watch a hunger bite names its cause once a floor
// (`starving · no light`) and the HUD's hp keeps what the drain took this run (`starving −N`), never a bare `max −N`.
import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
let n=0;const check=(ok,m)=>{if(!ok)throw Error(m);n++;};
try{for(const width of [400,1440]){
 const p=await b.newPage({viewport:{width,height:900}});
 await p.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=9&fake_drain=1`,{waitUntil:'domcontentloaded'});
 await p.waitForFunction(()=>window.__riddle?.booted&&window.__riddle.screen==='camp');await p.waitForTimeout(250);
 await p.evaluate(()=>{window.__riddle.watchMode='fast';window.__riddle.go({kind:'watch'});});
 const r=await p.evaluate(()=>new Promise((res)=>{
  const words=new Set();const t0=performance.now();
  const obs=new MutationObserver(()=>{for(const el of document.querySelectorAll('.watch .callout, .watch .ticker, .watch [class*=callout]')){const t=el.textContent.trim();if(t)words.add(t);}});
  obs.observe(document.body,{subtree:true,childList:true,characterData:true});
  let keep=null;const poll=()=>{const loss=document.querySelector('.watch .hp-text .hp-max-loss');if(loss)keep={text:loss.textContent.trim(),title:loss.title,hp:loss.parentElement.textContent};
   if((loss&&[...words].some(w=>/no light/.test(w)))||performance.now()-t0>45000){obs.disconnect();res({loss:keep?.text??null,title:keep?.title??null,hp:keep?.hp??document.querySelector('.watch .hp-text')?.textContent??'',screen:window.__riddle.screen,words:[...words]});return;}
   requestAnimationFrame(poll);};poll();}));
 check(r.loss&&/^starving −\d+$/.test(r.loss),`${width}: HUD names what took the max (${r.loss} in "${r.hp}" · ${r.screen} · ${r.words.slice(0,12).join(' | ')})`);
 check(/max hp −\d+ · starving/.test(r.title??''),`${width}: the loss chip's tip (${r.title})`);
 check(r.words.some(w=>/starving · no light/.test(w)),`${width}: the bite's callout names its cause (${r.words.filter(w=>/starv|hunger/.test(w)).join(' | ')})`);
 check(!r.words.some(w=>/hunger −1 max|^max [−-]\d/.test(w)),`${width}: never a bare bite or a bare max`);
 const box=await p.evaluate(()=>{const e=document.querySelector('.watch .hp-text .hp-max-loss').getBoundingClientRect();return {l:e.left,r:e.right,w:innerWidth};});
 check(box.l>=0&&box.r<=box.w,`${width}: the loss chip sits on screen`);
 console.log(width,'hunger legible',r.loss,'|',r.words.filter(w=>/starv/.test(w))[0]);await p.close();
}}finally{await b.close();}
console.log(n,'hunger legibility checks PASS');
