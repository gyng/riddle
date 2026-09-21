#!/usr/bin/env node
// `pnpm test`: every client gate at once, each in its own headless browser against the dev server
// (tools/dev.sh starts or reuses it). Output is printed per test as it finishes; exit 1 if any failed.
//   node tests/run.mjs [name…]        e.g. node tests/run.mjs fights cut12
import { spawn } from "node:child_process";
import { readdirSync } from "node:fs";
// Every tests/*.mjs but this runner and screen-time.mjs (a timing gate the coordinator runs on the GPU).
const ALL = readdirSync("tests").filter((f) => f.endsWith(".mjs") && !["run.mjs", "screen-time.mjs"].includes(f)).map((f) => f.slice(0, -4)).sort();
const names = process.argv.slice(2).length ? process.argv.slice(2) : ALL;
const t0 = Date.now();
const results = await Promise.all(names.map((n) => new Promise((resolve) => {
  const p = spawn("node", [`tests/${n}.mjs`], { stdio: ["ignore", "pipe", "pipe"] });
  let out = "";
  p.stdout.on("data", (d) => (out += d)); p.stderr.on("data", (d) => (out += d));
  p.on("close", (code) => resolve({ n, code, out, ms: Date.now() - t0 }));
})));
let failed = 0;
for (const r of results) {
  const last = r.out.trim().split("\n").at(-1) ?? "";
  console.log(`${r.code === 0 ? "ok  " : "FAIL"} ${r.n.padEnd(15)} ${(r.ms / 1000).toFixed(1)}s  ${last}`);
  if (r.code !== 0) { failed++; console.log(r.out.split("\n").filter((l) => /^(FAIL|fail|not ok|Error|\s+at )/.test(l) || /error/i.test(l)).slice(0, 20).map((l) => "     " + l).join("\n")); }
}
console.log(`${failed ? "FAIL" : "ok"}: ${results.length - failed}/${results.length} client gates in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
process.exit(failed ? 1 : 0);
