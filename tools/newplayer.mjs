#!/usr/bin/env node
// A new player's first session (c30-legible): a fresh lineage, sent run after run as someone who has never seen the game would — watch
// each run to its end (no ▶▶|), read the screen after, tap the one obvious button, look at the town. Headed on the GPU by default (the
// shots are for a blind reader). Shots: the town at start, a mid-run frame, the run's end beat, the screen after (report / death), and
// the town on return (a building's arrival caught while it stands up, then built). Prints one line per shot and the end beats.
//
//   node tools/newplayer.mjs [--port 5368] [--seed N] [--runs 4] [--speed 2] [--out dir] [--headless]
import { mkdirSync, writeFileSync } from "node:fs";
import { join, resolve } from "node:path";
import { launchBrowser } from "./browser.mjs";

const arg = (k, d) => { const i = process.argv.indexOf(k); return i >= 0 ? process.argv[i + 1] : d; };
const port = Number(arg("--port", "5368")), seed = Number(arg("--seed", "4242")), runs = Number(arg("--runs", "4")), speed = Number(arg("--speed", "2"));
const out = resolve(arg("--out", "scratchpad/legible/walk"));
const headed = !process.argv.includes("--headless");
mkdirSync(out, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const t0 = Date.now();
const browser = await launchBrowser({ headed });
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: headed ? 3 : 2 });
const errors = [];
page.on("pageerror", (e) => errors.push(e.message));
let n = 0;
const lines = [];
async function shot(name, note = "") {
  const base = join(out, `${String(++n).padStart(2, "0")}-${name}`);
  await page.screenshot({ path: `${base}.png` });
  const tx = await page.evaluate(() => window.__riddle?.text?.() ?? document.body.innerText);
  writeFileSync(`${base}.txt`, tx);
  const line = `${base.slice(out.length + 1).padEnd(26)} +${((Date.now() - t0) / 1000).toFixed(1).padStart(6)}s ${note}`;
  console.log(line); lines.push(line);
}
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, ms = 60000) { const t = Date.now(); let s; while (Date.now() - t < ms) { s = await state(); if (pred(s)) return s; await sleep(100); } throw new Error(`timeout: ${label} (${s?.screen})`); }
async function settle() { let ok = 0; for (let i = 0; i < 60 && ok < 2; i++) { const s = await state(); ok = s?.booted && !s.busy ? ok + 1 : 0; await sleep(150); } }
const buildings = () => page.evaluate(() => window.__town?.stats?.().buildings ?? []);

await page.goto(`http://localhost:${port}/?dev=1&fresh=1&seed=${seed}&speed=${speed}`, { waitUntil: "domcontentloaded" });
await waitFor((s) => s?.booted && s.screen === "camp", "camp");
await settle(); await sleep(1200);
await shot("town-start");
let had = await buildings();
for (let k = 1; k <= runs; k++) {
  await page.locator("button.send").first().click({ timeout: 10000 });
  await waitFor((s) => s?.screen === "watch", "watch");
  const tw = Date.now(); let mid = false, ended = false, s;
  for (;;) {
    s = await state();
    if (s?.screen === "exit" && await page.locator(".sheet-wrap .vault-choice .chip").count()) { await page.locator(".sheet-wrap .vault-choice .chip").first().click().catch(() => {}); await sleep(300); continue; }
    if (s?.screen !== "watch") break;
    if (!mid && Date.now() - tw > 9000) { mid = true; await shot(`run${k}-mid`); }
    if (!ended) {
      const b = await page.evaluate(() => (window.__beatLog ?? []).slice(-1)[0]?.text ?? "");
      const exitBeat = await page.evaluate(() => { const l = window.__beatLog ?? []; const e = l.filter((x) => /BANKED|RETURNED|STALLED|REPELLED|LOST|DRIVEN/.test(x.text)); return e.length ? e[e.length - 1] : null; });
      const done = await page.evaluate(() => window.__riddle?.app?.watchEnded ?? false).catch(() => false);
      if (exitBeat && exitBeat.ms > 0 && (await page.evaluate((m) => performance.now() - m, exitBeat.ms)) < 4000 && !lines.some((l) => l.includes(`run${k}-end`))) {
        await sleep(700); ended = true; await shot(`run${k}-end`, `beat: ${exitBeat.text}${exitBeat.why ? ` · ${exitBeat.why}` : ""}`);
      }
      void b; void done;
    }
    if (Date.now() - tw > 600000) throw new Error("run too long");
    await sleep(150);
  }
  if (s?.screen === "exit") { await settle(); await shot(`run${k}-sheet`); await page.locator(".sheet-wrap .btn.primary").first().click().catch(() => {}); s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit", "after keep"); }
  await settle(); await sleep(600);
  s = await state();
  if (!ended && s.screen === "death") await shot(`run${k}-death`);
  else await shot(`run${k}-${s.screen}`);
  if (s.screen === "report") await page.locator("main.report .gem").first().click();
  else if (s.screen === "death") await page.locator('button[data-tile="camp"], .cmd button').filter({ hasText: /^camp$/ }).first().click().catch(async () => { await page.locator(".lever-gem").first().click(); });
  await waitFor((x) => x?.screen === "camp", "camp");
  await sleep(500);
  const now = await buildings();
  const fresh = now.filter((b) => !had.includes(b));
  if (fresh.length) { await sleep(700); await shot(`run${k}-town-arrival`, `new: ${fresh.join(",")}`); await sleep(1800); }
  else await sleep(1500);
  await shot(`run${k}-town`, `buildings: ${now.join(",") || "none"}`);
  had = now;
}
console.log(`done ${((Date.now() - t0) / 1000).toFixed(0)} s · ${n} shots · errors ${errors.length}${errors.length ? `: ${errors.slice(0, 3).join(" | ")}` : ""}`);
writeFileSync(join(out, "summary.txt"), lines.join("\n") + "\n");
await browser.close();
