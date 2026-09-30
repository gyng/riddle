#!/usr/bin/env node
// Cohort 24 (AW: `CONTEXT_LOST_WEBGL` — "Web page caused context loss and was blocked" — froze the watch for 7 minutes on a
// placeholder view, ▶▶| dead until a reload). The renderer handles `webglcontextlost` / `webglcontextrestored` and the watch keeps
// advancing with a 2D view while the GL context is gone:
//   (a) the real engine's watch: `WEBGL_lose_context.loseContext()` mid-watch → `stats().glLost`, the 2D overlay drawn (not blank),
//       the tick advancing; `restoreContext()` → GL frames again, the overlay hidden; lost again → ▶▶| reaches the run's exit.
//   (b) no GL context at all (getContext("webgl*") → null, a page Chrome blocked): the watch mounts the renderer's 2D viewer, its
//       tick advances, ▶▶| reaches the exit, no page error.
//   node web/tests/ctxloss.mjs        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
async function open(init) {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  if (init) await page.addInitScript(init);
  return page;
}
const screen = (p) => p.evaluate(() => window.__riddle?.screen ?? null).catch(() => null);
const tick = (p) => p.evaluate(() => Number(document.querySelector(".watch")?.dataset.tick ?? NaN)).catch(() => NaN);
const stats = (p) => p.evaluate(() => { const s = window.__viewer?.stats?.(); return s ? { glLost: !!s.glLost, calls: s.calls, tick: s.tick } : null; }).catch(() => null);
async function waitFor(p, pred, label, ms = 30000) { const t = Date.now(); while (Date.now() - t < ms) { if (await pred()) return true; await sleep(100); } throw new Error(`timeout: ${label}`); }
/** ▶▶| pressed until the run leaves the watch (an exit's vault pick taken); ms to get there, or -1. */
async function skipToExit(p, ms = 60000) {
  const t = Date.now();
  while (Date.now() - t < ms) {
    const s = await screen(p);
    if (s !== "watch") {
      if (s === "exit") { await p.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 1000 }).catch(() => {}); }
      return Date.now() - t;
    }
    await p.evaluate(() => document.querySelector('button[data-tile="skip"]')?.click());
    await sleep(250);
  }
  return -1;
}
/** the 2D overlay's pixels: shown, and more than one colour (a map drawn, not a cleared canvas) */
const overlay = (p) => p.evaluate(() => {
  const c = document.querySelector(".watch canvas.view-2d");
  if (!c) return { shown: false, colours: 0 };
  const g = c.getContext("2d"), d = g.getImageData(0, 0, c.width, c.height).data, set = new Set();
  for (let i = 0; i < d.length; i += 4 * 97) set.add((d[i] << 16) | (d[i + 1] << 8) | d[i + 2]);
  return { shown: getComputedStyle(c).display !== "none", colours: set.size };
});
/** a fresh lineage's camp → send (the gem) → the watch */
async function toWatch(p, label) {
  await waitFor(p, async () => ["camp", "watch"].includes(await screen(p)), `${label}: the camp`, 60000);
  const t = Date.now();
  while ((await screen(p)) === "camp" && Date.now() - t < 20000) { await p.evaluate(() => (document.querySelector("button.send") ?? document.querySelector(".gem-slot .gem"))?.click()); await sleep(700); }
  await waitFor(p, async () => (await screen(p)) === "watch" && !!(await stats(p)), label);
}
const lose = (p, restore = false) => p.evaluate((restore) => {
  const c = document.querySelector(".watch canvas.view");
  const gl = c.getContext("webgl2") ?? c.getContext("webgl");
  window.__lc ??= gl.getExtension("WEBGL_lose_context");
  if (restore) window.__lc.restoreContext(); else window.__lc.loseContext();
}, restore);

try {
  // (a) a lost context mid-watch (the fake engine: its early floors last), restored, lost again and skipped to the exit
  {
    const p = await open();
    await p.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1`, { waitUntil: "domcontentloaded" });
    await waitFor(p, async () => (await screen(p)) === "watch" && !!(await stats(p)), "the watch (fake)");
    await p.evaluate(() => document.querySelector('button[data-tile="one"]')?.click());   // normal: the run lasts
    await sleep(1200);
    await lose(p);
    await sleep(600);
    const s1 = await stats(p), o1 = await overlay(p), t1 = await tick(p);
    check(s1?.glLost === true, `lost: stats().glLost (${s1?.glLost})`);
    check(o1.shown && o1.colours >= 4, `lost: the 2D view is drawn over the canvas (shown ${o1.shown}, ${o1.colours} colours)`);
    await sleep(2000);
    const t2 = await tick(p), sc = await screen(p);
    check(sc === "watch" && t2 > t1, `lost: the watch keeps advancing (tick ${t1} → ${t2}, ${sc})`);
    await lose(p, true);
    await sleep(800);
    const s2 = await stats(p), o2 = await overlay(p);
    check(s2?.glLost === false && s2.calls > 0, `restored: GL frames again (glLost ${s2?.glLost}, ${s2?.calls} draw calls)`);
    check(!o2.shown, `restored: the 2D view hidden (${o2.shown})`);
    const t3 = await tick(p); await sleep(1000); const t4 = await tick(p);
    check((await screen(p)) !== "watch" || t4 > t3, `restored: the watch advances (tick ${t3} → ${t4})`);
    if ((await screen(p)) === "watch") { await lose(p); await sleep(300); }
    const ms = await skipToExit(p);
    check(ms >= 0, `lost again: ▶▶| reaches the run's exit (${ms} ms, screen ${await screen(p)})`);
    await p.close();
  }
  // (a') the real engine: lost as the watch opens, ▶▶| still takes the run to its end
  {
    const p = await open();
    await p.goto(`${url}?dev=1&fresh=1&seed=4242`, { waitUntil: "domcontentloaded" });
    await toWatch(p, "the watch");
    await lose(p);
    await sleep(500);
    check((await stats(p))?.glLost === true || (await screen(p)) !== "watch", `real engine, lost: stats().glLost`);
    const ms = await skipToExit(p);
    check(ms >= 0, `real engine, lost: ▶▶| reaches the run's exit (${ms} ms, screen ${await screen(p)})`);
    await p.close();
  }
  // (b) no GL context can be made at all: the renderer's 2D viewer
  {
    const p = await open(() => {
      const get = HTMLCanvasElement.prototype.getContext;
      HTMLCanvasElement.prototype.getContext = function (type, ...a) { return /webgl/i.test(type) ? null : get.call(this, type, ...a); };
    });
    await p.goto(`${url}?dev=1&fresh=1&seed=4243`, { waitUntil: "domcontentloaded" });
    await toWatch(p, "the watch (no GL)");
    const s0 = await stats(p);
    check(s0?.glLost === true, `no GL: the 2D viewer mounted (glLost ${s0?.glLost})`);
    await p.evaluate(() => document.querySelector('button[data-tile="one"]')?.click());
    const t1 = await tick(p); await sleep(2000); const t2 = await tick(p);
    check((await screen(p)) !== "watch" || t2 > t1, `no GL: the tick advances (${t1} → ${t2})`);
    const drawn = await p.evaluate(() => { const c = document.querySelector(".watch canvas.view"); const d = c.getContext("2d")?.getImageData(0, 0, c.width, c.height).data; if (!d) return 0; const s = new Set(); for (let i = 0; i < d.length; i += 4 * 97) s.add((d[i] << 16) | (d[i + 1] << 8) | d[i + 2]); return s.size; });
    check(drawn >= 4, `no GL: the map is drawn in 2D (${drawn} colours)`);
    const ms = await skipToExit(p);
    check(ms >= 0, `no GL: ▶▶| reaches the run's exit (${ms} ms, screen ${await screen(p)})`);
    await p.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`ctxloss: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`ctxloss: ok (${out.length} checks)`);
