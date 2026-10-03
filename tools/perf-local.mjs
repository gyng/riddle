#!/usr/bin/env node
// Alternate unchanged baseline/candidate binaries on one saved camp; never time compilation.
import assert from "node:assert/strict";
import { createHash } from "node:crypto";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { resolve, join } from "node:path";
import { spawnSync } from "node:child_process";

const [baseline, candidate, fixture, ...flags] = process.argv.slice(2);
if (!fixture) throw new Error("usage: node tools/perf-local.mjs BASELINE CANDIDATE SAVE [--pairs N] [--hours N] [--out DIR]");
let pairs = 7, hours = 8, out;
for (let i = 0; i < flags.length; i += 2) {
  const value = flags[i + 1];
  if (flags[i] === "--pairs") pairs = Number(value);
  else if (flags[i] === "--hours") hours = Number(value);
  else if (flags[i] === "--out" && value) out = resolve(value);
  else throw new Error(`unknown or incomplete option: ${flags[i]}`);
}
assert.ok(Number.isSafeInteger(pairs) && pairs > 0 && Number.isSafeInteger(hours) && hours > 0);
const binaries = { before: resolve(baseline), after: resolve(candidate) };
const sha = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");
const provenance = { fixture: resolve(fixture), fixtureSha256: sha(fixture), binaries: Object.fromEntries(Object.entries(binaries).map(([key, path]) => [key, { path, sha256: sha(path) }])) };
const rows = [];
let expected;
for (let pair = 0; pair < pairs; pair++) {
  for (const which of pair % 2 ? ["after", "before"] : ["before", "after"]) {
    const run = spawnSync(binaries[which], [resolve(fixture), "1", String(hours)], { encoding: "utf8", timeout: 300_000 });
    assert.equal(run.status, 0, `${which} failed: ${run.error ?? run.stderr}`);
    const match = /sample 0: ([\d.]+) s; output (\w+); ticks (\d+)/.exec(run.stdout);
    assert.ok(match, `missing benchmark output: ${run.stdout}`);
    const result = { output: match[2], ticks: Number(match[3]) };
    expected ??= result;
    assert.deepEqual(result, expected, "performance candidate changed outcomes");
    rows.push({ pair, which, seconds: Number(match[1]), ...result, stdout: run.stdout });
    console.log(`${which} pair ${pair}: ${match[1]} s · ${result.output}`);
  }
}
const median = (which) => {
  const values = rows.filter((r) => r.which === which).map((r) => r.seconds).sort((a, b) => a - b);
  const middle = Math.floor(values.length / 2);
  return values.length % 2 ? values[middle] : (values[middle - 1] + values[middle]) / 2;
};
const before = median("before"), after = median("after");
const result = { ...provenance, pairs, hours, before, after, improvementPct: (1 - after / before) * 100, rows };
if (out) { mkdirSync(out, { recursive: true }); writeFileSync(join(out, "paired.json"), JSON.stringify(result, null, 2) + "\n"); }
console.log(`median ${before.toFixed(6)} → ${after.toFixed(6)} s · ${result.improvementPct.toFixed(2)}% faster · outcomes identical`);
