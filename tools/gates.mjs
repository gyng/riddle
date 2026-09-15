#!/usr/bin/env node
// Runs the Rust bot-gate table (examples/metrics.rs) in release and fails on any FAIL line.
import { spawnSync } from "node:child_process";
const r = spawnSync("cargo", ["run", "-q", "-p", "riddle-core", "--release", "--example", "metrics", "--", ...process.argv.slice(2)], { stdio: ["ignore", "pipe", "inherit"], encoding: "utf8" });
process.stdout.write(r.stdout ?? "");
if (r.status !== 0 || /\bFAIL\b/.test(r.stdout ?? "")) { console.error("gates: FAIL"); process.exit(1); }
console.log("gates: all PASS");
