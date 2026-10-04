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
