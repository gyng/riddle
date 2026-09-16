#!/usr/bin/env node
// Bot gate table (examples/metrics.rs) on the `fast` cargo profile.
//   node tools/gates.mjs          quick: 10 seeds × 2 h × 4 verdicts (~30 s)
//   node tools/gates.mjs --full   30 seeds × 8 h × 8 verdicts (~2 min); the number that counts
import { spawnSync } from "node:child_process";
const full = process.argv.includes("--full");
const extra = process.argv.slice(2).filter((a) => a !== "--full");
const b = spawnSync("cargo", ["build", "-q", "--profile", "fast", "-p", "riddle-core", "--example", "metrics"], { stdio: "inherit" });
if (b.status !== 0) process.exit(b.status ?? 1);
const r = spawnSync("target/fast/examples/metrics", [...(full ? [] : ["--quick"]), ...extra], { stdio: ["ignore", "pipe", "inherit"], encoding: "utf8" });
process.stdout.write(r.stdout ?? "");
if (r.status !== 0 || !/gates: all PASS/.test(r.stdout ?? "")) { console.error("gates: FAIL"); process.exit(1); }
