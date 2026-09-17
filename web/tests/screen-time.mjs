#!/usr/bin/env node
// Cut 10 §1 gate (coordinator-run, not in `pnpm test`): a DEFAULT run's screen time in `fights` mode, from send to the
// exit, median over 30 seeds ≤ 90 s with ≥ 3 fights shown. Runs on the GPU harness (tools/browser.mjs) against the dev
// server (tools/dev.sh, :5219) with the real engine (`--engine fake` for the UI fake), several pages in parallel (the
// viewer's clock is wall time, so pages do not slow each other unless the machine starves).
//
//   node web/tests/screen-time.mjs [--seeds 30] [--from 1] [--parallel 4] [--engine wasm|fake] [--max 420] [--out file.json]
//
// Per seed: boot fresh(seed) with `autosend=1` (the watch mounts straight from boot; that mount is the send), answer any
// vault-choice sheet with its first chip (part of the run), stop the clock the moment the screen is no longer the watch
// (the exit sheet, the death or the report) and read `data-fights` off the watch element (fights shown). Prints one line
// per seed, then the median, the spread and the gate verdict; exit 1 when the gate fails or a page errors.
import { execFileSync } from "node:child_process";
import { writeFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchGpu } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const opt = { seeds: 30, from: 1, parallel: 4, engine: "wasm", max: 420, out: null, median: 90, fights: 3 };
for (let i = 2; i < process.argv.length; i++) {
  const a = process.argv[i], v = process.argv[i + 1];
  if (a === "--seeds") { opt.seeds = Number(v); i++; }
  else if (a === "--from") { opt.from = Number(v); i++; }
  else if (a === "--parallel") { opt.parallel = Number(v); i++; }
  else if (a === "--engine") { opt.engine = v; i++; }
  else if (a === "--max") { opt.max = Number(v); i++; }
  else if (a === "--out") { opt.out = resolve(v); i++; }
  else { console.error(`unknown argument ${a}`); process.exit(2); }
}
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [];

const browser = await launchGpu();
async function measure(seed) {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
  page.on("console", (m) => { if (m.type() === "error") errors.push(`seed ${seed}: console.error: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`seed ${seed}: pageerror: ${e.message}`));
  const row = { seed, screenS: null, fights: 0, exit: null, depth: null, vaults: 0, error: null };
  try {
    await page.goto(`${url}?dev=1&fresh=1&seed=${seed}&autosend=1${opt.engine === "fake" ? "&engine=fake" : ""}`, { waitUntil: "domcontentloaded" });
    const st = () => page.evaluate(() => {
      const r = window.__riddle, w = document.querySelector(".watch");
      return r ? { screen: r.screen, booted: r.booted, fights: Number(w?.dataset.fights ?? 0), depth: document.querySelector(".watch .depth")?.textContent ?? null, vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null;
    });
    let t0 = 0, s = null;
    const tBoot = Date.now();
    while (Date.now() - tBoot < 60_000) { s = await st(); if (s?.screen === "watch") { t0 = Date.now(); break; } await sleep(50); }
    if (!t0) throw new Error(`no watch after boot (screen=${s?.screen})`);
    let fights = 0, depth = null;
    for (;;) {
      s = await st();
      if (s?.screen === "watch") { fights = s.fights; depth = s.depth; }
      else if (s?.screen === "exit" && s.vault) { row.vaults++; await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(150); continue; }
      else if (s && s.screen !== "watch") { row.exit = s.screen; break; }
      if (Date.now() - t0 > opt.max * 1000) { row.exit = "timeout"; break; }
      await sleep(100);
    }
    row.screenS = (Date.now() - t0) / 1000; row.fights = fights; row.depth = depth;
  } catch (e) { row.error = e.message; errors.push(`seed ${seed}: ${e.message}`); }
  finally { await page.close().catch(() => {}); }
  return row;
}

const seeds = Array.from({ length: opt.seeds }, (_, i) => opt.from + i);
const rows = [];
let next = 0;
async function worker() { while (next < seeds.length) { const seed = seeds[next++]; const r = await measure(seed); rows.push(r); console.log(`seed ${String(seed).padStart(3)}  ${r.screenS === null ? "  --" : r.screenS.toFixed(1).padStart(6)} s  fights ${String(r.fights).padStart(2)}  ${r.depth ?? ""}  → ${r.exit ?? r.error}`); } }
const t = Date.now();
await Promise.all(Array.from({ length: Math.max(1, opt.parallel) }, worker));
await browser.close().catch(() => {});

const ok = rows.filter((r) => r.screenS !== null && r.exit !== "timeout").sort((a, b) => a.screenS - b.screenS);
const med = (xs) => (xs.length ? xs[Math.floor((xs.length - 1) / 2)] : NaN);
const times = ok.map((r) => r.screenS), fights = ok.map((r) => r.fights).sort((a, b) => a - b);
const medianS = med(times), medianF = med(fights);
const fewFights = ok.filter((r) => r.fights < opt.fights).length;
console.log(`\n${ok.length}/${rows.length} runs · screen time median ${medianS.toFixed(1)} s (min ${Math.min(...times).toFixed(1)}, max ${Math.max(...times).toFixed(1)}) · fights median ${medianF} (min ${fights[0]}, max ${fights[fights.length - 1]}) · ${fewFights} run(s) under ${opt.fights} fights · ${((Date.now() - t) / 1000).toFixed(0)} s wall, ${opt.engine}`);
const pass = ok.length === rows.length && medianS <= opt.median && medianF >= opt.fights && !errors.length;
console.log(`gate: median ≤ ${opt.median} s and ≥ ${opt.fights} fights (median) → ${pass ? "ok" : "FAIL"}`);
if (opt.out) writeFileSync(opt.out, JSON.stringify({ engine: opt.engine, rows, medianS, medianF, pass }, null, 2));
for (const e of errors) console.error(e);
process.exit(pass ? 0 : 1);
