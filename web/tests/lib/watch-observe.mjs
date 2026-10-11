// Prepare the paused-picture exit experiment using the same visible control a
// player presses. Recording and pausing share a task, avoiding IPC gaps.
export async function pauseWithExitRecord(page) {
  return page.evaluate(() => new Promise((resolve,reject) => {
    const began=performance.now();let readyAt=null;
    const poll=()=>{
      const now=performance.now(),r=window.__riddle,w=document.querySelector('.watch');
      const ready=r?.booted&&r.screen==='watch'&&Number.isFinite(Number(w?.dataset.tick));
      if(ready&&readyAt===null)readyAt=now;
      if(readyAt===null&&now-began>20000){reject(Error('timeout waiting for the fast watch'));return;}
      if(readyAt!==null&&now-readyAt>2000){reject(Error('timeout pressing the initial watch pause'));return;}
      const button=w?.querySelector('.gem.hud-btn'),box=button?.getBoundingClientRect();
      if(ready&&button&&!button.disabled&&box?.width&&box.height&&getComputedStyle(button).visibility!=='hidden'){
        const x=box.x+box.width/2,y=box.y+box.height/2;
        const hit=x>=0&&x<innerWidth&&y>=0&&y<innerHeight?document.elementFromPoint(x,y):null;
        if(hit&&button.contains(hit)){
          const original=r.engine.step.bind(r.engine);r.__exit=null;
          r.engine.step=async n=>{const res=await original(n),exit=res.events.find(e=>e.k==='exit');if(exit)r.__exit={t:exit.t,depth:res.snapshot.depth,tier:exit.tier};return res;};
          button.click();resolve();return;
        }
      }
      requestAnimationFrame(poll);
    };
    requestAnimationFrame(poll);
  }));
}

// At most10Hz, on actual browser frames. Preserve the coverage interval and
// horizon while removing per-sample protocol round trips.
export async function sampleCardHud(page) {
  return page.evaluate(() => new Promise(resolve => {
    const began=performance.now(),samples=[];let last=-Infinity;
    const poll=()=>{
      const now=performance.now(),screen=window.__riddle.screen;
      if(screen!=='watch'||now-began>=20000){resolve(samples);return;}
      if(now-last>=100){
        const card=document.querySelector('.interstitial');
        samples.push({at:Math.round(now-began),screen,
          depth:document.querySelector('.watch .depth')?.textContent??'',
          stake:document.querySelector('.watch .stake')?.getAttribute('aria-label')??'',
          card:card&&!card.hidden?card.textContent:null});last=now;
      }
      requestAnimationFrame(poll);
    };
    requestAnimationFrame(poll);
  }));
}
