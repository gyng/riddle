import test from 'node:test';
import assert from 'node:assert/strict';
import { NativeRebuilder, nativeInput } from './native-watch.mjs';
import { setTimeout as delay } from 'node:timers/promises';

test('native edits coalesce, serialize and rebuild edits that arrive during a build', async () => {
  const pending = [], events = []; let builds = 0, active = 0, maxActive = 0;
  const queue = new NativeRebuilder({ debounceMs: 5, publish: s => events.push(s), build: async () => {
    builds++; active++; maxActive = Math.max(maxActive, active);
    await new Promise(resolve => pending.push(resolve)); active--;
  } });
  try {
    for (let i = 0; i < 10; i++) queue.request();
    assert.equal(queue.state.phase, 'queued');
    await delay(20); assert.equal(builds, 1);
    queue.request(); queue.request(); await delay(20); assert.equal(builds, 1);
    pending.shift()(); await delay(20); assert.equal(builds, 2);
    pending.shift()(); await delay(20);
    assert.equal(maxActive, 1); assert.equal(queue.state.phase, 'ready'); assert.equal(queue.state.revision, 12);
    assert.deepEqual(events.map(s => s.phase), ['queued', 'building', 'queued', 'building', 'ready']);
    assert.ok(queue.state.durationMs >= 0);
  } finally { queue.close(); }
});

test('failed builds surface errors, recover on an edit and stop after close', async () => {
  let builds = 0;
  const queue = new NativeRebuilder({ debounceMs: 5, build: async () => { if (++builds === 1) throw new Error('invalid Rust'); } });
  queue.request(); await delay(20);
  assert.equal(queue.state.phase, 'error'); assert.match(queue.state.error, /invalid Rust/);
  queue.request(); await delay(20); assert.equal(queue.state.phase, 'ready'); assert.equal(queue.state.error, null);
  queue.request(); queue.close(); await delay(20); assert.equal(builds, 2);
});

test('only compilation inputs trigger builds, not generated outputs or browser assets', () => {
  const root = '/tmp/project';
  for (const path of ['crates/riddle-core/src/turn.rs','crates/riddle-wasm/src/lib.rs','crates/riddle-core/presets/good.json','Cargo.lock','tools/native-codegen.mjs']) assert.equal(nativeInput(`${root}/${path}`, root), true, path);
  for (const path of ['target/native-dev/default/native_dev','web/src/engine/native-api.json','crates/riddle-core/examples/native_dev/generated.rs','web/src/runs.css','scratchpad/camp.json']) assert.equal(nativeInput(`${root}/${path}`, root), false, path);
});

test('test-only edits skip rebuilds only with current cfg declaration and compiler proof', async () => {
  const {mkdtempSync,mkdirSync,writeFileSync,rmSync}=await import('node:fs');
  const {tmpdir}=await import('node:os');const {join}=await import('node:path');
  const root=mkdtempSync(join(tmpdir(),'riddle-native-input-'));
  const src=join(root,'crates/riddle-core/src'),example=join(root,'target/fast/examples/native_dev');
  mkdirSync(src,{recursive:true});mkdirSync(join(root,'target/fast/examples'),{recursive:true});
  const lib=join(src,'lib.rs'),file=join(src,'tests_cut305.rs');
  const cfg='#[cfg(test)]\nmod tests_cut305;\n';
  const deps=extra=>writeFileSync(`${example}.d`,`${example}: ${lib}${extra?` ${extra}`:''}\n`);
  let builds=0;const queue=new NativeRebuilder({debounceMs:5,build:async()=>{builds++;}});
  const previous=process.env.CARGO_TARGET_DIR;delete process.env.CARGO_TARGET_DIR;
  try {
    writeFileSync(lib,cfg);assert.equal(nativeInput(file,root),true,'missing compiler proof rebuilds');
    deps();assert.equal(nativeInput(file,root),false,'confirmed unit-test module excluded');
    if(nativeInput(file,root))queue.request();await delay(20);assert.equal(builds,0);
    assert.equal(nativeInput(join(src,'turn.rs'),root),true);queue.request();await delay(20);assert.equal(builds,1,'runtime still rebuilds');
    deps(file);assert.equal(nativeInput(file,root),true,'compiled include of test-named file rebuilds');
    deps();writeFileSync(lib,'mod tests_cut305;\n');assert.equal(nativeInput(file,root),true,'now-runtime declaration rebuilds');
    writeFileSync(lib,cfg+'mod tests_cut305;\n');assert.equal(nativeInput(file,root),true,'ambiguous declaration rebuilds');
    writeFileSync(lib,cfg);deps();assert.equal(nativeInput(join(src,'tests_new.rs'),root),true,'unknown module rebuilds');
    assert.equal(nativeInput(join(src,'tests_cut305/helper.rs'),root),true,'nested test-like path stays conservative');
    writeFileSync(`${example}.d`,'invalid');assert.equal(nativeInput(file,root),true,'malformed compiler proof rebuilds');
    writeFileSync(`${example}.d`,`${example}: ${file}\n`);assert.equal(nativeInput(file,root),true,'incomplete proof rebuilds');
    writeFileSync(`${example}.d`,`${example}: ${lib} escaped\\ path\n`);assert.equal(nativeInput(file,root),true,'escaped proof stays conservative');
    deps();process.env.CARGO_TARGET_DIR='elsewhere';assert.equal(nativeInput(file,root),true,'custom target does not trust default proof');
    delete process.env.CARGO_TARGET_DIR;rmSync(lib);assert.equal(nativeInput(file,root),true,'unreadable declaration rebuilds');
  } finally {queue.close();rmSync(root,{recursive:true,force:true});previous===undefined?delete process.env.CARGO_TARGET_DIR:process.env.CARGO_TARGET_DIR=previous;}
});
