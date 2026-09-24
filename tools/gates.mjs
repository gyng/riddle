#!/usr/bin/env node
// Bot gate table (examples/metrics.rs) on the `fast` cargo profile.
//   node tools/gates.mjs          quick: 8 seeds × 8 h × 3 verdicts, dayplayer 2 seeds alongside (~35–60 s)
//   node tools/gates.mjs --full   30 seeds × 8 h × 8 verdicts, dayplayer 3 seeds (~2.5 min); the number that counts
// Cut 13 §6: the wire invariants (examples/qa.rs, 30 seeds, ~35 s alone on 4 threads, ~80 s beside the table) run
// beside both as a third job; the run fails if they do. `METRICS_PHASES=1` prints the table's phase and job walls.
//   node tools/gates.mjs --fresh  ignore the cache (docs/ITERATION_SPEED.md §3.3: the printed table is kept under
//                                 target/gates/<sha1 of the three binaries + the presets>.txt; a hit reprints and
//                                 re-checks — a client-only commit skips the table entirely)
import { spawn, spawnSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, readdirSync, writeFileSync } from "node:fs";
import os from "node:os";
const full = process.argv.includes("--full");
const fresh = process.argv.includes("--fresh");
const extra = process.argv.slice(2).filter((a) => a !== "--full" && a !== "--fresh");
const b = spawnSync("cargo", ["build", "-q", "--profile", "fast", "-p", "riddle-core", "--example", "metrics", "--example", "dayplayer", "--example", "qa"], { stdio: "inherit" });
if (b.status !== 0) process.exit(b.status ?? 1);
// The binaries hash every input exactly (sources, deps, rustc); the presets are read at run time.
const hash = createHash("sha1");
for (const f of ["target/fast/examples/metrics", "target/fast/examples/dayplayer", "target/fast/examples/qa"]) hash.update(readFileSync(f));
for (const f of readdirSync("crates/riddle-core/presets").sort()) hash.update(readFileSync(`crates/riddle-core/presets/${f}`));
hash.update(JSON.stringify({ full, extra }));
const cacheFile = `target/gates/${hash.digest("hex").slice(0, 16)}${full ? "-full" : "-quick"}.txt`;
if (!fresh && !extra.length && existsSync(cacheFile)) {
  const cached = readFileSync(cacheFile, "utf8");
  process.stdout.write(cached);
  const ok = /gates: all PASS/.test(cached) && /qa: all PASS/.test(cached) && !/hard bars: FAIL/.test(cached);
  console.log(`gates: ${ok ? "cached PASS" : "cached FAIL"} (${cacheFile}; --fresh to rerun)`);
  process.exit(ok ? 0 : 1);
}
// The table fills every core seed by seed for ~40 s; the fourteen-day probe is a few long
// sequential chains (one per seed) that would otherwise run alone afterwards — they overlap.
const run = (bin, args) => new Promise((resolve) => {
  const p = spawn(bin, args, { stdio: ["ignore", "pipe", "inherit"] });
  let out = ""; p.stdout.on("data", (d) => (out += d));
  p.on("close", (status) => resolve({ status, stdout: out }));
});
// The dayplayer's chains start first. They were the critical path while the table left them
// their cores; with the table at ~2.5 min and the chains at ~1 min alone (docs/ITERATION_SPEED.md,
// 2026-09-24), the table is, so it takes every core but one and shares them while the others run.
const seeds = full ? 3 : 2;
const cores = os.availableParallelism();
const dayplayer = run("target/fast/examples/dayplayer", ["--gate", "--seeds", String(seeds)]);
// The invariants take four threads (~40 s): more would contend the table's quiet per-tick measurement.
const qaThreads = 4;
const qa = run("target/fast/examples/qa", ["--seeds", "30", "--threads", String(qaThreads)]);
const table = run("target/fast/examples/metrics", [...(full ? [] : ["--quick"]), "--threads", String(Math.max(4, cores - 1)), ...extra]);
const [r, p, q] = await Promise.all([table, dayplayer, qa]);
let printed = "";
const say = (t) => { printed += t; process.stdout.write(t); };
say(r.stdout ?? "");
if (r.status !== 0 || !/gates: all PASS/.test(r.stdout ?? "")) { console.error("gates: FAIL"); process.exit(1); }
say(q.stdout ?? "");
if (q.status !== 0 || !/qa: all PASS/.test(q.stdout ?? "")) { console.error("qa invariants: FAIL"); process.exit(1); }
// Fourteen-day pacing probe. Two of its bars (unlock days ≥ 10/14, stall ≤ 3 d) assume the full
// 30-floor dungeon; with v1's 16 floors a competent player finishes on day 4–9, so until M7 content
// lands the probe is printed and only its remaining bars fail the run (docs/CUT2.md deviation).
const out = p.stdout ?? ""; say(out.slice(out.lastIndexOf("bar ")));
const hardFails = [...out.matchAll(/^(Marks unspent|Empty check-ins|Class L10)[^\n]*FAIL/gm)].map((m) => m[0]);
if (hardFails.length) { console.error("dayplayer hard bars: FAIL\n" + hardFails.join("\n")); process.exit(1); }
say("dayplayer: hard bars pass (content bars informational until M7)\n");
if (!extra.length) { mkdirSync("target/gates", { recursive: true }); writeFileSync(cacheFile, printed); }
