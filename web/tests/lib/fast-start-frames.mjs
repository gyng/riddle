// Keep observation in the browser across the travel-to-fight transition.
// Each state is read from the DOM on an actual animation frame; no game edits.
export async function fastStartFrames(page) {
  return page.evaluate(() => new Promise((resolve, reject) => {
    const began=performance.now();let travel=null,fightAt=0;
    const sample=()=>{
      const r=window.__riddle,w=document.querySelector('.watch');
      return {screen:r?.screen,mode:w?.dataset.mode,frame:w?.dataset.frame,
        speed:Number(w?.dataset.speed),tick:Number(w?.dataset.tick),
        ending:w?.dataset.ending==='1'};
    };
    const poll=()=>{
      const now=performance.now(),s=sample();
      if(!travel){
        if(now-began>8000){reject(Error('timeout waiting for32× travel'));return;}
        if(s.screen!=='watch'||s.speed>=32){travel=s;fightAt=now;}
      }
      if(travel){
        if(now-fightAt>30000){reject(Error('timeout waiting for a fight in fast'));return;}
        if(s.screen!=='watch'||(s.frame==='fight'&&s.speed===4)){
          resolve({travel,fight:s,travelMs:Math.round(fightAt-began),fightMs:Math.round(now-fightAt)});return;
        }
      }
      requestAnimationFrame(poll);
    };
    requestAnimationFrame(poll);
  }));
}
