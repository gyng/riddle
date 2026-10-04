#!/usr/bin/env node
// Compare complete shipping bridge replies against the native dev bridge.
import { NativeHost } from './native-host.mjs';
import { launchGpu } from './browser.mjs';
import { readFileSync, mkdirSync, writeFileSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';
const [binary, out, ...saves] = process.argv.slice(2);
assert.ok(binary && out && saves.length, 'usage: native-parity.mjs BINARY OUT SAVE...');
const sha = b => createHash('sha256').update(b).digest('hex');
const wasm = readFileSync('web/src/engine/pkg/riddle_wasm_bg.wasm');
const bridge = readFileSync('web/src/engine/pkg/riddle_wasm.js');
const threads = Number(process.env.RIDDLE_NATIVE_THREADS ?? 8);
const host = new NativeHost({ binary, threads });
const browser = await launchGpu(); const rows = [];
try {
  const page = await browser.newPage();
  await page.route('http://riddle.parity.invalid/**', r => {
    const p = new URL(r.request().url()).pathname;
    return r.fulfill(p === '/engine.mjs' ? {contentType:'text/javascript',body:bridge} : p === '/engine.wasm' ? {contentType:'application/wasm',body:wasm} : {contentType:'text/html',body:'<!doctype html><title>Native parity</title>'});
  });
  await page.goto('http://riddle.parity.invalid/');
  await page.evaluate(async()=>{const m=await import('/engine.mjs');await m.default({module_or_path:'/engine.wasm'});window.engineModule=m;});
  for (let i=0;i<saves.length;i++) {
    const input = readFileSync(saves[i],'utf8'), id=`camp-${i}`;
    await host.call(id,'version',[]);
    await page.evaluate(()=>{window.parityGame=new window.engineModule.Game(1);});
    const calls=[['load',[input]],['lineage',[]],['vocabulary',[]],['forecast',[]],['packageOptions',[12]],['wallEdit',[]],['save',[]],['runOfflineQuick',[8*3600]],['save',[]]];
    for (const [method,args] of calls) {
      const start=performance.now(), n=await host.call(id,method,args), native=(performance.now()-start)/1000;
      const w=await page.evaluate(({method,args})=>{const start=performance.now();const r=window.parityGame[method](...args);return {r,seconds:(performance.now()-start)/1000};},{method,args});
      // JSON floats may have different last-bit formatting; compare parsed values except raw saves.
      if (n !== w.r) {
        mkdirSync(out,{recursive:true});
        writeFileSync(join(out,'mismatch-native.json'), typeof n === 'string' ? n : JSON.stringify(n));
        writeFileSync(join(out,'mismatch-wasm.json'), typeof w.r === 'string' ? w.r : JSON.stringify(w.r));
        throw new Error(`${saves[i]} ${method}: differing reply (saved in ${out})`);
      }
      rows.push({save:resolve(saves[i]),inputSha256:sha(input),method,nativeSeconds:native,wasmSeconds:w.seconds,replySha256:sha(JSON.stringify(n))});
    }
    await page.evaluate(()=>window.parityGame.free());
  }
  mkdirSync(out,{recursive:true});writeFileSync(join(out,'parity.json'),JSON.stringify({binarySha256:sha(readFileSync(binary)),wasmSha256:sha(wasm),bridgeSha256:sha(bridge),harnessSha256:sha(readFileSync(new URL(import.meta.url))),browserVersion:browser.version(),threads,scope:'Each round loads fresh games; calls run in the recorded order. Native timing includes ordered process IPC; WASM timing covers the direct bridge call. No whole-app speed claim.',rows},null,2)+'\n');
  console.log(JSON.stringify(rows.filter(r=>['forecast','packageOptions','wallEdit','runOfflineQuick'].includes(r.method)),null,2));
} finally {host.close();await browser.close();}
