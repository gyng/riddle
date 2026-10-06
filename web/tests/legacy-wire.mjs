import {execFileSync} from 'node:child_process';
import {launchBrowser} from '../../tools/browser.mjs';
import assert from 'node:assert/strict';
const url=execFileSync('bash',['tools/dev.sh'],{cwd:new URL('../../',import.meta.url),encoding:'utf8'}).trim();
const browser=await launchBrowser();
try {
 const page=await browser.newPage(),errors=[];page.on('pageerror',e=>errors.push(e.message));
 await page.goto(`${url}?fresh=1&seed=3&runs=0`);await page.waitForFunction(()=>window.__riddle?.booted);
 const result=await page.evaluate(async()=>{
  const app=window.__riddle;app.runnerOn=false;
  if(app.kind!=='wasm')throw Error('real WASM worker required');
  if(typeof app.engine.respecLegacy!=='function')throw Error('missing worker respec bridge');
  const before=await app.engine.save();let reason='';
  try {await app.engine.respecLegacy();}catch(error){reason=String(error);}
  return {reason,unchanged:before===await app.engine.save(),offer:(await app.engine.lineage()).legacy_respec};
 });
 assert.match(result.reason,/build a house|no upgrades/);
 assert.equal(result.unchanged,true);assert.equal(result.offer.available,false);assert.equal(result.offer.refund,0);
 assert.deepEqual(errors,[]);
 console.log('real WASM worker respec bridge, Rust refusal and complete save unchanged PASS');
}finally {await browser.close();}
