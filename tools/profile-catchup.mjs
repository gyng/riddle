#!/usr/bin/env node
// Capture the actual WASM catch-up in headed Chromium. Open *.cpuprofile in DevTools.
import { launchGpu } from './browser.mjs';
import { readFileSync, writeFileSync, mkdirSync } from 'node:fs';
import { createHash } from 'node:crypto';
import { resolve, join } from 'node:path';
import assert from 'node:assert/strict';

const [savePath, ...flags] = process.argv.slice(2);
if (!savePath) throw new Error('usage: node tools/profile-catchup.mjs SAVE --out DIR [--pkg DIR] [--hours N] [--runs N]');
let out, pkg = 'web/src/engine/pkg', hours = 8, runs = 3;
for (let i = 0; i < flags.length; i += 2) {
  const value = flags[i + 1];
  if (flags[i] === '--out' && value) out = resolve(value);
  else if (flags[i] === '--pkg' && value) pkg = value;
  else if (flags[i] === '--hours') hours = Number(value);
  else if (flags[i] === '--runs') runs = Number(value);
  else throw new Error(`unknown or incomplete option: ${flags[i]}`);
}
assert.ok(out && Number.isSafeInteger(hours) && hours > 0 && Number.isSafeInteger(runs) && runs > 0);
const input = readFileSync(savePath, 'utf8');
const wasm = readFileSync(join(pkg, 'riddle_wasm_bg.wasm'));
const bridge = readFileSync(join(pkg, 'riddle_wasm.js'));
const hash = bytes => createHash('sha256').update(bytes).digest('hex');
mkdirSync(out, { recursive: true });
const browser = await launchGpu();
try {
  const page = await browser.newPage();
  // Route everything locally: no server, stale dev transforms or service-worker cache.
  await page.route('http://riddle.perf.invalid/**', route => {
    const path = new URL(route.request().url()).pathname;
    if (path === '/engine.mjs') return route.fulfill({ contentType: 'text/javascript', body: bridge });
    if (path === '/engine.wasm') return route.fulfill({ contentType: 'application/wasm', body: wasm });
    return route.fulfill({ contentType: 'text/html', body: '<!doctype html><title>Riddle catch-up profile</title>' });
  });
  await page.goto('http://riddle.perf.invalid/');
  await page.evaluate(async ({ input, seconds }) => {
    const mod = await import('/engine.mjs');
    await mod.default({ module_or_path: '/engine.wasm' });
    window.catchupProfile = { mod, input };
    // Module compilation and one warm-up expedition batch are outside the profile.
    const game = new mod.Game(1);
    try { game.load(input); game.runOfflineQuick(seconds); } finally { game.free(); }
  }, { input, seconds: hours * 3600 });
  const cdp = await page.context().newCDPSession(page);
  await cdp.send('Profiler.enable');
  await cdp.send('Profiler.setSamplingInterval', { interval: 1000 });
  await cdp.send('Profiler.start');
  const results = await page.evaluate(({ runs, seconds }) => {
    const rows = [], { mod, input } = window.catchupProfile;
    for (let i = 0; i < runs; i++) {
      const game = new mod.Game(1);
      try {
        game.load(input);
        const start = performance.now(), report = game.runOfflineQuick(seconds);
        const wallSeconds = (performance.now() - start) / 1000;
        rows.push({ wallSeconds, report, save: game.save() });
      } finally { game.free(); }
    }
    return rows;
  }, { runs, seconds: hours * 3600 });
  const { profile } = await cdp.send('Profiler.stop');
  writeFileSync(join(out, 'catchup.cpuprofile'), JSON.stringify(profile));
  const nodes = new Map(profile.nodes.map(node => [node.id, node]));
  const totals = new Map();
  for (let i = 0; i < (profile.samples ?? []).length; i++) {
    const frame = nodes.get(profile.samples[i]).callFrame;
    const key = JSON.stringify([frame.functionName, frame.url]);
    totals.set(key, (totals.get(key) ?? 0) + (profile.timeDeltas?.[i] ?? 1000));
  }
  const total = [...totals.values()].reduce((a, b) => a + b, 0);
  const leaf = [...totals].map(([key, micros]) => {
    const [functionName, url] = JSON.parse(key);
    return { functionName, url, milliseconds: micros / 1000, percent: micros / total * 100 };
  }).sort((a, b) => b.milliseconds - a.milliseconds);
  const outcomes = results.map(row => ({ seconds: row.wallSeconds, reportSha256: hash(row.report), saveSha256: hash(row.save) }));
  for (const row of outcomes) assert.deepEqual([row.reportSha256, row.saveSha256], [outcomes[0].reportSha256, outcomes[0].saveSha256], 'profile runs changed outcomes');
  const summary = {
    fixture: resolve(savePath), fixtureSha256: hash(input), pkg: resolve(pkg),
    wasmSha256: hash(wasm), bridgeSha256: hash(bridge), harnessSha256: hash(readFileSync(new URL(import.meta.url))),
    browserVersion: browser.version(), hours, runs, samples: profile.samples?.length, intervalMicros: 1000,
    scope: 'Profile includes save loading, catch-up and save serialization; timings cover runOfflineQuick only. Compilation and warm-up excluded. Inlined work is attributed to the containing function. Shipping builds may show only WASM function numbers. This profiles the engine, not the app catch-up UI or chunk scheduler.',
    outcomes, leaf,
  };
  writeFileSync(join(out, 'summary.json'), JSON.stringify(summary, null, 2) + '\n');
  console.log(JSON.stringify({ samples: summary.samples, outcomes, leaf: leaf.slice(0, 10), out }, null, 2));
} finally { await browser.close(); }
