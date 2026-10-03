#!/usr/bin/env node
// Orchestration regression test with isolated fake workers. No game evidence is produced.
import assert from 'node:assert/strict';
import { mkdtempSync, mkdirSync, writeFileSync, readFileSync, readdirSync, existsSync, rmSync } from 'node:fs';
import { spawn } from 'node:child_process';
import { tmpdir } from 'node:os';
import { join } from 'node:path';
import { fileURLToPath } from 'node:url';
const dir = mkdtempSync(join(tmpdir(), 'riddle-gate-recovery-'));
const delay = (ms) => new Promise((r) => setTimeout(r, ms));
let active = null;
async function launch(finish = false) {
  active = spawn(process.execPath, ['gates.mjs'], { cwd: dir, env: { ...process.env, ...(finish ? { FINISH_NOW: '1' } : {}) }, stdio: 'ignore' });
  const child = active;
  const closed = new Promise((r) => child.on('close', r));
  return { child, closed };
}
try {
  for (const d of ['crates/riddle-core/src', 'crates/riddle-core/presets', 'eval/cards', 'target/fast/examples']) mkdirSync(join(dir, d), { recursive: true });
  for (const f of ['crates/riddle-core/src/lib.rs', 'crates/riddle-core/Cargo.toml', 'Cargo.toml', 'Cargo.lock', 'target/fast/examples/dayplayer', 'target/fast/examples/metrics', 'target/fast/examples/qa']) writeFileSync(join(dir, f), 'fixture');
  writeFileSync(join(dir, 'child.mjs'), `import {EventEmitter} from 'node:events'; import {appendFileSync} from 'node:fs';
export const spawnSync=(bin)=>({status:0,stdout:bin==='rustc'?'fixture compiler':''});
export function spawn(bin){const p=new EventEmitter();p.stdout=new EventEmitter();p.stderr=new EventEmitter();const name=bin.split('/').at(-1);appendFileSync('calls.txt',name+'\\n');setTimeout(()=>{if(name==='metrics')p.stderr.emit('data','metrics: quiet ticks measured\\n');},5);setTimeout(()=>{p.stdout.emit('data',name==='metrics'?'gates: all PASS\\n':name==='qa'?'qa: all PASS\\n':'bar fixture PASS\\ndayplayer: all PASS\\n');p.emit('close',0)},name==='dayplayer'&&!process.env.FINISH_NOW?60000:40);return p;}`);
  writeFileSync(join(dir, 'gates.mjs'), readFileSync(fileURLToPath(new URL('./gates.mjs', import.meta.url)), 'utf8').replace('from "node:child_process"', 'from "./child.mjs"'));
  const first = await launch();
  let files = [];
  for (let i = 0; i < 250; i++) {
    await delay(20);
    if (existsSync(join(dir, 'target/gates'))) files = readdirSync(join(dir, 'target/gates')).filter((f) => f.endsWith('.txt'));
    if (files.length === 2) break;
  }
  assert.equal(files.length, 2, 'completed legs saved while dayplayer is pending');
  assert.ok(files.some((f) => f.startsWith('metrics-')) && files.some((f) => f.startsWith('qa-')));
  for (const f of files) assert.equal(JSON.parse(readFileSync(join(dir, 'target/gates', f))).status, 0);
  first.child.kill('SIGTERM'); await first.closed;
  writeFileSync(join(dir, 'calls.txt'), '');
  const second = await launch(true);
  assert.equal(await second.closed, 0);
  assert.equal(readFileSync(join(dir, 'calls.txt'), 'utf8'), 'dayplayer\n', 'completed legs reused after interruption');
  const metrics = files.find((f) => f.startsWith('metrics-'));
  writeFileSync(join(dir, 'target/gates', metrics), 'interrupted older cache');
  writeFileSync(join(dir, 'calls.txt'), '');
  const third = await launch(true);
  assert.equal(await third.closed, 0);
  assert.equal(readFileSync(join(dir, 'calls.txt'), 'utf8'), 'metrics\n', 'corrupt cache is rerun; intact legs reused');
  assert.equal(JSON.parse(readFileSync(join(dir, 'target/gates', metrics))).status, 0);
  assert.ok(!readdirSync(join(dir, 'target/gates')).some((f) => f.includes('.tmp-')), 'atomic writes leave no pending file');
  console.log('PASS: gate orchestration preserves completed legs on interruption, reuses them, and reruns a corrupt cache. No game simulations were tested.');
} finally {
  active?.kill('SIGTERM');
  rmSync(dir, { recursive: true, force: true });
}
