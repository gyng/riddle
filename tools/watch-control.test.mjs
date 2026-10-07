// Exercise visible Speed navigation, including Skip opening a new decision.
import assert from 'node:assert/strict';
import {launchBrowser} from './browser.mjs';
import {pressWatchControl} from './watch-control.mjs';
const b=await launchBrowser();
try{const p=await b.newPage();
for(const [label,id] of [['normal','one'],['fights only','fights'],['fast','fast'],['▶▶|','skip']]){
 await p.setContent('<main class="watch"><footer class="console"><button data-tile="speed">Speed</button></footer></main>');
 await p.evaluate(()=>{document.querySelector('[data-tile=speed]').onclick=()=>{
  const wrap=document.createElement('div');wrap.className='sheet-wrap';wrap.innerHTML='<button class="close-stud">Close</button><div class="watch-options"><button data-tile="one">normal</button><button data-tile="fights">fights only</button><button data-tile="fast">fast</button><button data-tile="skip">▶▶|</button></div>';document.body.append(wrap);
  wrap.querySelector('.close-stud').onclick=()=>wrap.remove();for(const btn of wrap.querySelectorAll('[data-tile]'))btn.onclick=()=>{document.body.dataset.chosen=btn.dataset.tile;if(btn.dataset.tile==='skip'){const keep=document.createElement('div');keep.className='sheet-wrap keep';keep.textContent='Choose loot';document.body.append(keep);}else wrap.remove();};
 };});
 assert.equal(await pressWatchControl(p,label),true);assert.equal(await p.locator('body').getAttribute('data-chosen'),id);assert.equal(await p.locator('.watch-options').count(),0);assert.equal(await p.locator('.keep').count(),id==='skip'?1:0);
}
await p.setContent('<main class="watch"><footer class="console"><button data-tile="speed" disabled>Speed</button></footer></main>');assert.equal(await pressWatchControl(p,'fast'),false);
await p.setContent('<main class="watch"><button data-tile="fast" disabled>fast</button></main>');assert.equal(await pressWatchControl(p,'fast'),false);
await p.setContent('<main class="watch"><button data-tile="fast">fast</button></main>');assert.equal(await pressWatchControl(p,'fast'),true);
await p.setContent('<main class="watch"><footer class="console"><button data-tile="speed">Speed</button></footer></main><div class="sheet-wrap">Caller choice</div>');assert.equal(await pressWatchControl(p,'fast'),false);assert.equal(await p.locator('.sheet-wrap').innerText(),'Caller choice');
await p.setContent('<main class="camp"></main>');assert.equal(await pressWatchControl(p,'fast'),false);assert.equal(await pressWatchControl(p,'unknown'),false);
console.log('watch controls: 4 modes, own-menu closure, new keep preservation, disabled/inline/caller/no-watch/unknown PASS');
}finally{await b.close();}
