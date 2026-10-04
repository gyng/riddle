import test from 'node:test';
import assert from 'node:assert/strict';
import { mkdtempSync, writeFileSync, chmodSync, rmSync, utimesSync } from 'node:fs';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { NativeHost, nativePlugin } from './native-host.mjs';
test('native sessions isolate, order calls, recover from request errors and preserve state across rebuild', async () => {
  const dir = mkdtempSync(join(tmpdir(), 'riddle-native-host-'));
  const binary = join(dir, 'engine');
  writeFileSync(binary, `#!/usr/bin/env node
const rl = require('node:readline').createInterface({input:process.stdin}); let state=0;
rl.on('line',line=>{ const {id,method,args}=JSON.parse(line); let r=null,e;
if(method==='version')r='test'; else if(method==='add')r=state+=args[0];
else if(method==='save')r=String(state); else if(method==='load')state=Number(args[0]); else e='unknown';
console.log(JSON.stringify({id,ok:!e,r,e})); });
`); chmodSync(binary, 0o755);
  const host = new NativeHost({ binary, threads: 1 });
  try {
    assert.throws(() => host.call('missing', 'save', []), /unavailable/);
    await Promise.all(['a','b'].map(id => host.call(id, 'version', [])));
    assert.deepEqual(await Promise.all([host.call('a','add',[2]), host.call('a','add',[3])]), [2,5]);
    assert.equal(await host.call('b','save',[]), '0');
    await assert.rejects(host.call('a','bad',[]), /unknown/);
    utimesSync(binary, new Date(), new Date(Date.now()+1000));
    assert.equal(await host.call('a','add',[7]), 12);
    assert.equal(await host.call('b','save',[]), '0');
    host.closeSession('b');
    assert.throws(() => host.call('b','save',[]), /unavailable/);
    assert.throws(() => host.call('b','version',[]), /unavailable/);
  } finally { host.close(); rmSync(dir, { recursive: true, force: true }); }
});
test('native transport rejects remote connections and cross-origin requests', async () => {
  const before = { enabled: process.env.RIDDLE_NATIVE_DEV, binary: process.env.RIDDLE_NATIVE_BIN };
  process.env.RIDDLE_NATIVE_DEV = '1'; process.env.RIDDLE_NATIVE_BIN = process.execPath;
  let middleware, close;
  try {
    nativePlugin().configureServer({ httpServer: {once:(_event,fn)=>{close=fn;}}, middlewares:{use:(_path,fn)=>{middleware=fn;}} });
    async function request(address, origin, host='localhost:5367', method='POST', url='/rpc') {
      let result;
      const res={setHeader(){},statusCode:200,end(body){result={status:this.statusCode,body:JSON.parse(body)};}};
      await middleware({socket:{remoteAddress:address},headers:{host,origin},method,url},res);
      return result;
    }
    assert.equal((await request('10.0.0.2','http://localhost:5367')).status,403);
    assert.equal((await request('127.0.0.1','https://evil.invalid')).status,403);
    assert.equal((await request('127.0.0.1','http://evil.invalid','evil.invalid')).status,403);
    assert.equal((await request('127.0.0.1',undefined,'localhost:5367','GET','/health')).body.native,true);
  } finally {
    close?.();
    for (const [key,value] of [['RIDDLE_NATIVE_DEV',before.enabled],['RIDDLE_NATIVE_BIN',before.binary]]) value===undefined ? delete process.env[key] : process.env[key]=value;
  }
});
