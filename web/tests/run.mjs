#!/usr/bin/env node
// `pnpm test`: every client gate at once, each in its own headless browser against the dev server
// (tools/dev.sh starts or reuses it). Output is printed per test as it finishes; exit 1 if any failed.
//   node tests/run.mjs [name…]        e.g. node tests/run.mjs fights cut12 clarity:paint
// A name `file:parts` runs `tests/<file>.mjs --part=<parts>` (clarity's four parts run side by side: the whole walk was one
// 4-minute test the suite waited on — docs/ITERATION_SPEED.md, round 3); a bare `clarity` runs its parts.
import { spawn } from "node:child_process";
import { readdirSync } from "node:fs";
import net from "node:net";
// Every tests/*.mjs but this runner, screen-time.mjs and town-gpu.mjs (timing gates the coordinator runs on the GPU).
const PARTS = { clarity: ["clarity:core,watch", "clarity:hold", "clarity:card", "clarity:paint", "clarity:deep"] };
const expand = (n) => PARTS[n] ?? [n];
const ALL = readdirSync("tests").filter((f) => f.endsWith(".mjs") && !["run.mjs", "screen-time.mjs", "town-gpu.mjs"].includes(f)).map((f) => f.slice(0, -4)).sort().flatMap(expand);
const names = process.argv.slice(2).length ? process.argv.slice(2).flatMap(expand) : ALL;
const t0 = Date.now();
// The suite's own dev server (tests/vite.test.config.ts: no watcher, no HMR) on a free port, for this run only — the shared
// :5219 server reloads every page on any edit in web/src (other agents' work) and a stale one serves old transforms. The
// tests find it through RIDDLE_PORT (tools/dev.sh); a RIDDLE_PORT set by the caller is used as it is.
let vite = null;
if (!process.env.RIDDLE_PORT) {
  const port = await new Promise((res) => { const s = net.createServer(); s.listen(0, () => { const p = s.address().port; s.close(() => res(p)); }); });
  vite = spawn(process.execPath, ["node_modules/vite/bin/vite.js", "--port", String(port), "--strictPort", "--config", "tests/vite.test.config.ts"], { stdio: "ignore" });
  process.env.RIDDLE_PORT = String(port);
  const up = () => fetch(`http://localhost:${port}/`).then((r) => r.ok, () => false);
  for (let i = 0; i < 120 && !(await up()); i++) await new Promise((r) => setTimeout(r, 250));
  // (the first page pre-bundles the dependencies: once, before the tests race for it)
  await fetch(`http://localhost:${port}/src/main.ts`).catch(() => {});
}
// A bounded pool, longest first: fourteen headless browsers rendering on the CPU at once starved the
// timing gates (clarity's rates, fights' holds) into flakes; the timing gates start first, then the rest
// longest first (`OWN`: each test's own seconds in a quiet suite, 2026-09-26 — cut25's real-wasm watch
// started last and ended the suite alone), sharing the remaining slots (`TEST_JOBS` overrides the width).
const TIMING = ["fights", "cut13", "clarity:deep", "clarity:card", "clarity:core,watch"];
const OWN = { cut25: 180, fights: 128, qaj: 88, ui: 87, "clarity:hold": 81, "clarity:deep": 66, cut13: 61, "clarity:card": 55, qa9: 52, screens: 47, "clarity:core,watch": 42, cut27: 38, cut23: 36, "clarity:paint": 30 };
const own = (n) => OWN[n] ?? 20;
const SOLO = ["clarity:paint"];   // (below: last, alone)
const order = [...names].sort((a, b) => (SOLO.includes(a) ? 1 : 0) - (SOLO.includes(b) ? 1 : 0) || (TIMING.includes(b) ? 1 : 0) - (TIMING.includes(a) ? 1 : 0) || (TIMING.includes(a) ? TIMING.indexOf(a) - TIMING.indexOf(b) : own(b) - own(a)));
const width = Number(process.env.TEST_JOBS ?? 7);
const run1 = (n) => new Promise((resolve) => {
  const start = Date.now();
  const [file, parts] = n.split(":");
  const p = spawn("node", [`tests/${file}.mjs`, ...(parts ? [`--part=${parts}`] : [])], { stdio: ["ignore", "pipe", "pipe"] });
  let out = "";
  p.stdout.on("data", (d) => (out += d)); p.stderr.on("data", (d) => (out += d));
  p.on("close", (code) => resolve({ n, code, out, ms: Date.now() - t0, own: Date.now() - start }));
});
// The wall-clock gates get headroom: while one runs, at most HEAVY_WIDTH (3) browsers run in all (7 CPU-rendered
// browsers beside them flaked them; the old runner kept clarity's whole walk and fights alone — 2 browsers — for
// 4 minutes). Their readings retry once when they fail on a loaded machine (tests/lib/load.mjs).
// SOLO: the real engine's edit timings (wasm in a worker, bars of 1 s and 3 s) read the machine even with three
// beside and a retry (1.3 s at a load of 31) — they run last, with nothing beside, ~30 s, on a server the other tests
// have warmed (first on a cold one its refine read 3.4 s where it reads 1.9 s warm). (The deep fixture's timings are
// notes unless RIDDLE_DEEP_GATE=1, which wants a quiet headed machine: `clarity:deep` runs as a heavy.)
const HEAVY = ["clarity:card", "clarity:deep", "clarity:core,watch", "fights", "cut13"], HEAVY_WIDTH = Number(process.env.TEST_HEAVY_WIDTH ?? 3);
const queue = [...order], done = [];
let heavyLive = 0, othersLive = 0, soloLive = 0;
const nextJob = () => {
  if (soloLive) return null;
  const live = heavyLive + othersLive;
  const i = queue.findIndex((n) => SOLO.includes(n) ? live === 0 : (heavyLive === 0 && !HEAVY.includes(n)) || live < HEAVY_WIDTH);
  return i < 0 ? null : queue.splice(i, 1)[0];
};
await Promise.all(Array.from({ length: Math.min(width, queue.length) }, async () => {
  while (queue.length) {
    const n = nextJob();
    if (!n) { await new Promise((r) => setTimeout(r, 200)); continue; }
    const kind = SOLO.includes(n) ? "solo" : HEAVY.includes(n) ? "heavy" : "other";
    if (kind === "solo") soloLive++; else if (kind === "heavy") heavyLive++; else othersLive++;
    done.push(await run1(n));
    if (kind === "solo") soloLive--; else if (kind === "heavy") heavyLive--; else othersLive--;
  }
}));
const results = names.map((n) => done.find((r) => r.n === n));
let failed = 0;
for (const r of results) {
  const last = r.out.trim().split("\n").at(-1) ?? "";
  // (the time it ended at, and its own time from its start)
  console.log(`${r.code === 0 ? "ok  " : "FAIL"} ${r.n.padEnd(15)} ${(r.ms / 1000).toFixed(1)}s (${(r.own / 1000).toFixed(1)}s)  ${last}`);
  if (r.code !== 0) { failed++; console.log(r.out.split("\n").filter((l) => /^(FAIL|fail|not ok|Error|\s+at )/.test(l) || /error|aborted/i.test(l)).slice(0, 20).map((l) => "     " + l).join("\n")); }
}
console.log(`${failed ? "FAIL" : "ok"}: ${results.length - failed}/${results.length} client gates in ${((Date.now() - t0) / 1000).toFixed(1)}s`);
vite?.kill();
process.exit(failed ? 1 : 0);
