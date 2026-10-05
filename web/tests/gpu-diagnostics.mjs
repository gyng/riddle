// Query lifecycle regression: WebGL2 constants, delayed/disjoint results and driver failure.
import {execFileSync} from 'node:child_process';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{const p=await b.newPage();await p.goto(url);const n=await p.evaluate(async()=>{
 const {GpuTimer}=await import('/src/render/gputimer.ts');let checks=0;
 const check=(v,label)=>{if(!v)throw Error(label);checks++;};
 const driver=(supported=true)=>{const queries=[],ext={TIME_ELAPSED_EXT:35007,GPU_DISJOINT_EXT:36795};let active=null,ended=0,deleted=0,lost=false,disjoint=false,allocation=true;
 const gl={QUERY_RESULT:34918,QUERY_RESULT_AVAILABLE:34919,getExtension:()=>supported?ext:null,isContextLost:()=>lost,getError:()=>{throw Error('must not drain unrelated GL errors');},getParameter:k=>{check(k===ext.GPU_DISJOINT_EXT,'disjoint enum');return disjoint;},createQuery:()=>{if(!allocation)return null;const q={ready:false,ns:4000000,pending:false};queries.push(q);return q;},beginQuery:(k,q)=>{check(k===ext.TIME_ELAPSED_EXT&&!active&&!q.pending,'no query reuse before result');active=q;},endQuery:k=>{check(k===ext.TIME_ELAPSED_EXT&&!!active,'end active query');active.pending=true;active=null;ended++;},deleteQuery:q=>{check(lost||q!==active,'end query before deletion');deleted++;},getQueryParameter:(q,k)=>{check(k===gl.QUERY_RESULT||k===gl.QUERY_RESULT_AVAILABLE,'WebGL2 result enum');if(k===gl.QUERY_RESULT_AVAILABLE)return q.ready;check(q.ready,'never block on an unavailable result');q.pending=false;return q.ns;}};
 return{gl,queries,get ended(){return ended;},get deleted(){return deleted;},set lost(v){lost=v;active=null;},set disjoint(v){disjoint=v;},set allocation(v){allocation=v;}};};
 {const d=driver(false),t=new GpuTimer(d.gl);t.begin();t.end();check(!t.available&&Number.isNaN(t.ms)&&Number.isNaN(t.pct(.95))&&d.queries.length===0,'missing extension');}
 {const d=driver(),t=new GpuTimer(d.gl);t.begin();t.end();check(Number.isNaN(t.ms)&&t.resolved===0,'pending sample');d.queries[0].ready=true;t.begin();t.end();check(t.ms===4&&t.pct(.95)===4&&t.resolved===1,'nanoseconds converted to milliseconds');d.queries[1].ready=true;d.queries[1].ns=9000000;t.begin();t.end();check(t.ms===9&&t.pct(0)===4&&t.pct(.95)===9,'rolling percentiles');t.dispose();check(!t.available&&Number.isNaN(t.ms)&&Number.isNaN(t.pct(.95))&&d.deleted===d.queries.length,'cleanup clears readings');}
 {const d=driver(),t=new GpuTimer(d.gl);for(let i=0;i<130;i++){t.begin();t.end();}check(!t.available&&d.queries.length===16&&d.deleted===16,'full pending ring reaches bounded timeout');}
 {const d=driver(),t=new GpuTimer(d.gl);t.begin();t.end();d.queries[0].ready=true;t.begin();t.end();for(let i=0;i<130;i++){t.begin();t.end();}check(!t.available&&t.resolved===1,'stall after prior progress also disables');}
 {const d=driver(),t=new GpuTimer(d.gl);t.begin();t.end();d.queries[0].ready=true;t.begin();t.end();d.disjoint=true;t.begin();t.end();check(Number.isNaN(t.ms)&&Number.isNaN(t.pct(.95))&&t.disjoint>0,'disjoint discards samples and unresolved queries');d.disjoint=false;t.begin();t.end();d.queries.at(-1).ready=true;t.begin();t.end();check(t.ms===4,'timing recovers after disjoint');t.dispose();}
 {const d=driver(),t=new GpuTimer(d.gl);t.begin();t.dispose();check(d.ended===1&&d.deleted===1&&!t.available,'dispose active query');t.dispose();check(d.deleted===1,'dispose idempotent');}
 {const d=driver();d.allocation=false;const t=new GpuTimer(d.gl);t.begin();t.end();check(!t.available&&d.queries.length===0,'allocation failure disables cleanly');}
 {const d=driver(),t=new GpuTimer(d.gl);t.begin();d.lost=true;t.end();check(!t.available&&d.ended===0&&d.deleted===1,'context loss does not end lost query');}
 // Real GL allocation: upload before derivation, then grow/shrink the source.
 {const THREE=await import('/node_modules/three/build/three.module.js'),{SpriteNormals}=await import('/src/render/normals.ts');const canvas=document.createElement('canvas'),src=document.createElement('canvas');canvas.width=canvas.height=64;src.width=src.height=32;
  const renderer=new THREE.WebGLRenderer({canvas,antialias:false}),sprite=new THREE.CanvasTexture(src),normals=new SpriteNormals(sprite),gl=renderer.getContext();
  const initial=normals.material.uniforms.nmap.value;check(initial.image.width===32&&initial.image.height===32,'normal canvas starts at atlas size');renderer.initTexture(initial);check(gl.getError()===gl.NO_ERROR,'initial normal upload');
  let version=0,disposed=0;initial.addEventListener('dispose',()=>disposed++);
  for(const [w,h] of [[32,32],[64,48],[16,16]]){src.width=w;src.height=h;const ctx=src.getContext('2d');ctx.fillStyle='#fff';ctx.fillRect(2,2,w-4,h-4);const sheet={canvas:src,version:++version};normals.sync(sheet,version*1000);normals.sync(sheet,version*1000+401);const tex=normals.material.uniforms.nmap.value;
   renderer.initTexture(tex);check(gl.getError()===gl.NO_ERROR,'derived/resized normal upload has no GL error');check(tex.image.width===w&&tex.image.height===h&&tex.image.getContext('2d').getImageData(w/2,h/2,1,1).data[3]>0,'derived normals match current atlas');
   if(version===1)check(tex===initial&&disposed===0,'same dimensions reuse texture');else check(tex!==initial&&disposed===1,'resize disposes original texture once');
  }
  normals.dispose();sprite.dispose();renderer.dispose();renderer.forceContextLoss();
 }
 return checks;
});console.log('GPU diagnostics:',n,'protocol checks PASS');}finally{await b.close();}
