// Isolate name placement from the watch's current floor, queued actors and
// quiet beat. The live watch has its own tests; these assertions need two foes.
export async function tagCollisionFixture(page) {
  return page.evaluate(async () => {
    const {createViewer} = await import('/src/render/index.ts');
    const snapshot = await window.__riddle.engine.send();
    snapshot.w=snapshot.h=16;snapshot.turn=0;
    snapshot.tiles=Array(256).fill('floor');snapshot.seen=snapshot.visible=Array(256).fill(true);
    snapshot.items=[];snapshot.overlays=[];snapshot.entities=[];
    snapshot.hero.x=snapshot.hero.y=6;
    const host=document.createElement('div');
    host.style.cssText='position:fixed;left:0;top:0;width:400px;height:600px;z-index:999';
    const canvas=document.createElement('canvas');canvas.style.cssText='width:400px;height:600px';
    host.append(canvas);document.body.append(host);
    const viewer=createViewer(canvas,{baseTexels:100});
    const frame=()=>new Promise(resolve=>requestAnimationFrame(()=>requestAnimationFrame(resolve)));
    try {
      viewer.load(snapshot);viewer.setSpeed(0);viewer.setFrame('fight');viewer.setQuiet(false);
      viewer.apply([
        {k:'spawn',t:0,e:{id:90011,kind:'goblin',name:'Captain Tain',x:5,y:6,hp:5,max_hp:5,tags:[]}},
        {k:'spawn',t:0,e:{id:90012,kind:'goblin',name:'Ashar Monkey',x:4,y:6,hp:5,max_hp:5,tags:[]}}
      ]);
      viewer.seek(0);
      for(let i=0;i<120&&!viewer.debugRects().some(rect=>rect.hero);i++)await frame();
      await frame();
      const initial={frame:viewer.frame(),labels:viewer.debugLabels(),rects:viewer.debugRects(),entities:viewer.debugEnts()};
      viewer.setQuiet(true);await frame();const quiet=viewer.debugLabels();
      viewer.setQuiet(false);await frame();const restored=viewer.debugLabels();
      return {...initial,quiet,restored};
    } finally {viewer.dispose();host.remove();}
  });
}
