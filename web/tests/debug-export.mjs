import { execFileSync } from 'node:child_process';
import { launchBrowser } from '../../tools/browser.mjs';
import { readDebug } from '../../tools/debug-export.mjs';
import assert from 'node:assert/strict';
import {mkdirSync,writeFileSync} from 'node:fs';
const shotsIndex=process.argv.indexOf('--shots'),shots=shotsIndex>=0?process.argv[shotsIndex+1]:process.env.RIDDLE_SHOTS;if(shots)mkdirSync(shots,{recursive:true});
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try { for(const width of [400,1440]) {
 const page=await browser.newPage({viewport:{width,height:900},acceptDownloads:true});
 await page.goto(`${url}?fresh=1&seed=5&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 const result=await page.evaluate(async()=>{
  const app=window.__riddle, {captureDebug,debugNote}=await import('/src/debug.ts');let n=0;
  const check=(ok,name)=>{if(!ok)throw Error(name);n++;};
  check(app.kind==='wasm','actual shipping engine');
  app.lineage=await app.engine.buildTown('house');
  // A rich fixture through the real loader, without spending test time earning gold.
  const fixture=JSON.parse(await app.engine.save());fixture.lineage.gold=1000;
  app.lineage=await app.engine.load(JSON.stringify(fixture));
  app.lineage=await app.engine.addBloodline();app.lineage=await app.engine.selectBloodline(2);app.adoptSets();
  app.lastSave='stale checkpoint';
  localStorage.setItem('unrelated-secret','do-not-export');
  const original=app.engine,before=await original.save(),calls=[];
  app.engine=new Proxy(original,{get:(target,key)=>typeof target[key]==='function'? (...args)=>{calls.push(key);return target[key](...args);} : target[key]});
  debugNote('error','test https://user:password@example.org/path?token=secret');
  const bundle=await captureDebug(app);app.engine=original;
  check(calls.join(',')==='save','export calls save only');
  check(bundle.save.engine===before,'fresh save equals real current state');
  check(await original.save()===before,'export does not mutate engine');
  const raw=JSON.parse(bundle.save.engine);
  check(raw.bloodlines_v===1&&Object.keys(raw.others).length===1&&raw.selected===2,'both bloodlines and selected hero retained');
  check(!JSON.stringify(bundle).includes('do-not-export')&&!JSON.stringify(bundle).includes('token=secret')&&!JSON.stringify(bundle).includes('user:password'),'unrelated storage and URL secrets excluded');
  check(bundle.build.commit!=='unknown'&&bundle.build.version&&bundle.context.bloodline===2,'build and hero context');
  check(bundle.capture.context==='at-request'&&bundle.capture.state==='fresh','context provenance explicit');
  const pkg=await import('/src/engine/pkg/riddle_wasm.js');await pkg.default();
  const expected=pkg.Game.fromSave(before),actual=pkg.Game.fromSave(bundle.save.engine);
  check(actual.save()===before,'shipping save round trip exact');
  const e=expected.runOfflineQuick(1800),a=actual.runOfflineQuick(1800);
  check(e===a&&expected.save()===actual.save(),'subsequent offline results and complete state exact');expected.free();actual.free();
  app.engine={...original,save:async()=>{throw Error('capture failure');}};
  let failed=false;try{await captureDebug(app);}catch{failed=true;}check(failed,'failure never returns stale save');
  app.engine=original;
  let attempts=0;app.engine={...original,save:async()=>{attempts++;if(attempts===1)app.sets[app.active].name='edited during capture';return before;}};
  const retried=await captureDebug(app);check(attempts===2&&retried.context.editing_sets[app.active].name==='edited during capture','changed editing state retries coherent capture');
  app.engine={...original,save:async()=>{app.sets[app.active].name=String(Math.random());return before;}};
  let unstable=false;try{await captureDebug(app);}catch{unstable=true;}check(unstable,'continually changing edits fail explicitly');
  app.engine=original;
  const getItem=Storage.prototype.getItem;Storage.prototype.getItem=function(){throw Error('storage unavailable');};
  let optional;try{optional=await captureDebug(app);}finally{Storage.prototype.getItem=getItem;}
  check(optional.save.engine===before&&optional.capture.unavailable.includes('environment'),'optional diagnostics cannot discard valid save');
  for(let i=0;i<100;i++)debugNote('test',String(i));check((await captureDebug(app)).diagnostics.length===64,'diagnostic history bounded');
  await app.importSave(JSON.stringify(bundle.save));
  check(await app.engine.save()===before,'existing app importer restores complete engine exactly');
  const offer=app.lineage.kit.find(k=>k.next?.affordable);
  check(!!offer,'real purchasable Forge step');
  app.lineage=await original.buyKit(offer.slot);app.adoptSets();
  const bought=await captureDebug(app);
  check(bought.save.engine===await original.save(),'immediate purchase captured exactly');
  const report=await original.runOfflineQuick(1800);app.lineage=await original.lineage();app.adoptSets();app.go({kind:'report',report});
  const offline=await captureDebug(app);check(offline.save.engine===await original.save()&&offline.context.view.kind==='report','offline catch-up and report captured exactly');
  app.go({kind:'watch'});
  return {n,bundle};
 });
 readDebug(JSON.stringify(result.bundle));const legacy=structuredClone(result.bundle);legacy.save.v=1;readDebug(JSON.stringify(legacy));
 for(const mutate of [b=>b.schema=999,b=>b.save.engine='broken',b=>b.capture.state='cached',b=>b.save.v=9]){
  const bad=structuredClone(result.bundle);mutate(bad);assert.throws(()=>readDebug(JSON.stringify(bad)));
 }
 await page.locator('.watch canvas').waitFor({state:'visible'});
 await page.evaluate(async()=>{const {openSettings}=await import('/src/ui/settings.ts');openSettings(window.__riddle);});
 const downloadPromise=page.waitForEvent('download');await page.locator('.debug-export').click();const download=await downloadPromise;
 const stream=await download.createReadStream();let downloaded='';for await(const chunk of stream)downloaded+=chunk;const liveDump=readDebug(downloaded);assert.equal(liveDump.bundle.context.view.kind,'watch');assert.equal(liveDump.engine.selected,2);assert.ok(liveDump.engine.run,'captured an actual in-flight run');
 await page.evaluate(async raw=>{const pkg=await import('/src/engine/pkg/riddle_wasm.js');await pkg.default();const restored=pkg.Game.fromSave(raw);try{if(restored.save()!==raw)throw Error('live restore mismatch');}finally{restored.free();}},liveDump.save.engine);
 if(shots){writeFileSync(`${shots}/${width}-dump.json`,downloaded);await page.locator('.debug-export').scrollIntoViewIfNeeded();await page.mouse.move(2,2);await page.screenshot({path:`${shots}/${width}-settings.png`});}
 assert.match(download.suggestedFilename(),/^riddle-debug-.*\.json$/);assert.equal(await page.locator('.debug-copy').isVisible(),true);
 await page.evaluate(()=>{window.goodDebugSave=window.__riddle.engine.save;window.__riddle.engine.save=async()=>{throw Error('test failure');};});
 await page.locator('.debug-export').click();await page.waitForFunction(()=>document.querySelector('.debug-status').textContent==='Export failed');
 assert.equal(await page.locator('.debug-export').isEnabled(),true);assert.equal(await page.locator('.settings textarea').inputValue(),'');
 await page.evaluate(()=>{window.__riddle.engine.save=window.goodDebugSave;window.goodDownload=HTMLAnchorElement.prototype.click;HTMLAnchorElement.prototype.click=function(){throw Error('blocked download');};});
 await page.locator('.debug-export').click();await page.waitForFunction(()=>document.querySelector('.debug-status').textContent==='Copy dump');
 readDebug(await page.locator('.settings textarea').inputValue());assert.equal(await page.locator('.debug-copy').isVisible(),true);
 await page.evaluate(()=>{HTMLAnchorElement.prototype.click=window.goodDownload;});
 assert.equal(await page.evaluate(()=>document.documentElement.scrollWidth>innerWidth),false);
 console.log(width,'shipping debug export, restore, live watch and recovery PASS');await page.close();
}}finally{await browser.close();}
