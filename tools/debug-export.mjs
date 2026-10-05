#!/usr/bin/env node
// Extract an importable save without booting or advancing the game.
import { readFileSync, writeFileSync, existsSync } from 'node:fs';
import { pathToFileURL } from 'node:url';
export function readDebug(text) {
  const b = JSON.parse(text);
  if (b?.format !== 'riddle-debug' || b.schema !== 1) throw Error('Unsupported diagnostic format/schema');
  if (b.capture?.state !== 'fresh' || !Number.isFinite(Date.parse(b.captured_at))) throw Error('Invalid capture provenance');
  const s = b.save;
  if (!s || ![1, 2].includes(s.v) || typeof s.engine !== 'string' || !s.engine ||
      !Array.isArray(s.loadout) || !s.loadout.every(Number.isSafeInteger) || !Number.isFinite(s.last_seen)) throw Error('Invalid save envelope');
  const engine = JSON.parse(s.engine);
  if (!engine || typeof engine !== 'object' || Array.isArray(engine)) throw Error('Invalid engine save');
  if (!b.build || typeof b.build.commit !== 'string' || typeof b.build.engine !== 'string') throw Error('Missing build identity');
  return { bundle: b, save: s, engine };
}
if (process.argv[1] && import.meta.url === pathToFileURL(process.argv[1]).href) {
  try {
    const args = process.argv.slice(2), file = args[0];
    if (!file || args.length !== 3 || args[1] !== '--save') throw Error('Usage: node tools/debug-export.mjs dump.json --save /tmp/repro-save.json');
    const { bundle: b, save, engine } = readDebug(readFileSync(file, 'utf8'));
    const contextFile = `${args[2]}.context.json`;
    if (existsSync(args[2]) || existsSync(contextFile)) throw Error('Output already exists; choose a new save filename');
    writeFileSync(args[2], JSON.stringify(save, null, 2), { flag: 'wx' });
    writeFileSync(contextFile, JSON.stringify({ build: b.build, capture: b.capture, captured_at: b.captured_at,
      context: b.context, diagnostics: b.diagnostics }, null, 2), { flag: 'wx' });
    console.log(JSON.stringify({ build: b.build, captured_at: b.captured_at, capture: b.capture,
      bloodlines: engine.bloodlines_v ? 1 + Object.keys(engine.others ?? {}).length : 1,
      save_file: args[2], context_file: contextFile }, null, 2));
  } catch (e) { console.error(e.message); process.exitCode = 1; }
}
