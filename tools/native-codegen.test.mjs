import test from 'node:test';
import assert from 'node:assert/strict';
import { readFileSync } from 'node:fs';
import { generate, writeGenerated } from './native-codegen.mjs';
const source = readFileSync(new URL('../crates/riddle-wasm/src/lib.rs', import.meta.url), 'utf8');
test('native bridge covers every client method and rejects unsupported bindings', () => {
  writeGenerated(true);
  const api = JSON.parse(generate(source).metadata);
  const proxy = readFileSync(new URL('../web/src/engine/proxy.ts', import.meta.url), 'utf8');
  const methods = [...proxy.slice(proxy.indexOf('const METHODS'), proxy.indexOf('];')).matchAll(/"(\w+)"/g)].map(m => m[1]);
  assert.deepEqual(api.methods.map(m => m.name).sort(), methods.sort());
  const wasm = readFileSync(new URL('../web/src/engine/wasm.ts', import.meta.url), 'utf8');
  for (const name of methods) assert.match(wasm, new RegExp(`\\b${name}\\(`), `missing WASM adapter method ${name}`);
  assert.throws(() => generate(source.replace('turns: u32', 'turns: Vec<u32>')), /unsupported/);
  assert.throws(() => generate(source.replace('pub fn forecast(&self) -> String', 'pub fn forecast(&self) -> u64')), /unsupported/);
  assert.throws(() => generate(source + '\npub fn extra(arg: (u32, u32)) {}'), /unsupported/);
});
test('checkpoint memento accounts for every serde-skipped engine field', () => {
  const engine = readFileSync(new URL('../crates/riddle-core/src/engine.rs', import.meta.url), 'utf8');
  const memento = readFileSync(new URL('../crates/riddle-core/examples/dayplayer_checkpoint/mod.rs', import.meta.url), 'utf8');
  for (const name of ['Game','Run','DeathRec','Batch']) {
    const body = engine.split(`pub struct ${name} {`)[1].split('\n}')[0];
    for (const m of body.matchAll(/#\[serde\([^)]*\bskip\b[^)]*\)\]\s*pub (\w+)/g))
      assert.match(memento, new RegExp(`\\b${m[1]}\\b`), `unaccounted runtime field ${name}.${m[1]}`);
  }
});
