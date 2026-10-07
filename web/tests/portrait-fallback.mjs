import {execFileSync} from 'node:child_process';import {mkdirSync} from 'node:fs';import {launchBrowser} from '../../tools/browser.mjs';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim(),b=await launchBrowser();
try{for(const width of [320,400,1440]){
 const p=await b.newPage({viewport:{width,height:900}}),errors=[];p.on('pageerror',e=>errors.push(e.message));
 await p.goto(`${url}?engine=fake&fresh=1&runs=0`);await p.waitForFunction(()=>window.__riddle?.booted);
 await p.evaluate(async()=>{const a=window.__riddle;a.lineage.class='fighter';a.lineage.packages.pen_open=false;window.__portraitSave=await a.engine.save();window.__portraitDeath={run_id:1,depth:8,cause:'goblin',margin:'1 hp short',verdict:'gap',baseline:0,replays:12,patches:[],morgue:'heir1',trace:{turns:[]},hero:{name:'Corin Ash',bloodline_id:1,heir:1,class:'rogue'}};a.go({kind:'death',death:window.__portraitDeath,kept:true});});
 await p.locator('.death-hero img').evaluate(el=>el.decode());
 if(await p.locator('.death-hero .unit-icon').getAttribute('data-unit')!=='hero_rogue')throw Error('Healthy class identity');
 await p.route('**/ui/portraits/hero_rogue.webp',route=>route.abort());
 await p.evaluate(()=>window.__riddle.go({kind:'death',death:{...window.__portraitDeath},kept:true}));
 await p.waitForFunction(()=>document.querySelector('.death-hero .unit-icon')?.dataset.art==='hero_rogue');
 const icon=p.locator('.death-hero .unit-icon');if(await icon.evaluate(el=>el.tagName)!=='SPAN')throw Error('Missing same-unit fallback');
 if(!await icon.evaluate(el=>getComputedStyle(el).backgroundImage.includes('/art/atlas.png')))throw Error('Atlas art missing');
 const frame=await p.locator('.death-hero .unit-portrait').boundingBox();if(frame.width!==64||frame.height!==64)throw Error('Frame shifted');
 if(await p.locator('.death-hero').textContent()!=='Corin Ash · rogue · Bloodline 1')throw Error('Historical identity changed');
 if(process.env.RIDDLE_PORTRAIT_SHOTS){mkdirSync(process.env.RIDDLE_PORTRAIT_SHOTS,{recursive:true});await p.screenshot({path:`${process.env.RIDDLE_PORTRAIT_SHOTS}/fallback-${width}.png`});}
 // The same component retains a primitive when even the atlas lacks this identity.
 await p.evaluate(async()=>{const original=window.__portraitDeath;window.__riddle.go({kind:'death',death:{...original,hero:{...original.hero,class:'future_class'}},kept:true});});
 if(await p.locator('.death-hero .unit-icon').getAttribute('data-glyph')!=='?')throw Error('Missing primitive');
 if(await p.locator('.death-hero .unit-icon').evaluate(el=>getComputedStyle(el,'::before').content)!=='"?"')throw Error('Invisible primitive');
 if(await p.evaluate(()=>document.documentElement.scrollWidth>innerWidth))throw Error('Overflow');
 if(!await p.evaluate(async()=>await window.__riddle.engine.save()===window.__portraitSave))throw Error('Save changed');
 if(errors.length)throw Error(errors.join('\n'));console.log(width,'healthy portrait/failure same-unit fallback/primitive/frame/historical identity/save PASS');await p.close();
}}finally{await b.close();}
