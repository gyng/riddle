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
// Fourteen-day pacing probe. Two of its bars (unlock days ≥ 10/14, stall ≤ 3 d) assume the full
// 30-floor dungeon; with v1's 16 floors a competent player finishes on day 4–9, so until M7 content
// lands the probe is printed and only its remaining bars fail the run (docs/CUT2.md deviation).
const d = spawnSync("cargo", ["build", "-q", "--profile", "fast", "-p", "riddle-core", "--example", "dayplayer"], { stdio: "inherit" });
if (d.status !== 0) process.exit(d.status ?? 1);
const p = spawnSync("target/fast/examples/dayplayer", ["--gate", "--seeds", full ? "3" : "2"], { stdio: ["ignore", "pipe", "inherit"], encoding: "utf8" });
const out = p.stdout ?? ""; process.stdout.write(out.slice(out.lastIndexOf("bar ")));
const hardFails = [...out.matchAll(/^(Marks unspent|Empty check-ins|Class L10)[^\n]*FAIL/gm)].map((m) => m[0]);
if (hardFails.length) { console.error("dayplayer hard bars: FAIL\n" + hardFails.join("\n")); process.exit(1); }
console.log("dayplayer: hard bars pass (content bars informational until M7)");
