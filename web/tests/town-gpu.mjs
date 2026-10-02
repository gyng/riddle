#!/usr/bin/env node
// Cut 30 §3 — the town's frame gates on the GPU harness (`launchGpu`: headed Chromium on the real GPU; headless is pixel-bound and
// says nothing about frame time). Not part of `pnpm test` (like screen-time.mjs): run it on a quiet machine.
//   busy     the v1 town with 24 walkers (the stress knob), input arriving: every frame the display gives (≥ 95 % of its rAF rate; WSLg's
//            compositor runs ~58.5), the frame's CPU p95 < 8 ms
//   idle     no input for 10 s: ≤ 30 frames a second (the scene keeps breathing at its idle rate)
//   hidden   the page hidden (the window minimised; WSLg keeps it `visible`, so the page is told): 0 frames
//   node web/tests/town-gpu.mjs [--shots dir]
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchGpu } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const out = []; let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchGpu();
const ctx = browser.contexts?.()[0] ?? await browser.newContext({ viewport: { width: 400, height: 800 } });
const page = await ctx.newPage();
await page.setViewportSize({ width: 400, height: 800 });
try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3020`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp" && window.__town, null, { timeout: 60_000 });
  await page.evaluate(async () => {
    const r = window.__riddle; const save = JSON.parse(await r.engine.save()); const L = save.lineage;
    Object.assign(L, { gold: 900, best_depth: 4, vault: [{ id: 9001, kind: "sword", known: true, label: "sword" }],
      kennel: [{ id: 77, kind: "jackal", name: "Ash", level: 1, tags: [], gen: 0, rules: { rows: [] }, max_rows: 2, hp: 5, max_hp: 5 }] });
    r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
  });
  await page.waitForFunction(() => window.__town?.stats().buildings.length === 4, null, { timeout: 10_000 });
  const probe = await page.evaluate(() => { const c = document.createElement("canvas").getContext("webgl2"); const d = c?.getExtension("WEBGL_debug_renderer_info"); return d ? c.getParameter(d.UNMASKED_RENDERER_WEBGL) : "?"; });
  out.push(`     renderer: ${probe}`);
  await page.evaluate(() => window.__town.stress(24));
  const frames = () => page.evaluate(() => window.__town.stats().frames);
  // busy: input arriving every 150 ms for 4 s
  await sleep(800);
  let f0 = await frames(), t0 = Date.now();
  for (let i = 0; i < 27; i++) { await page.mouse.move(100 + (i % 9) * 20, 300 + (i % 5) * 10); await sleep(150); }
  const busy = ((await frames()) - f0) / ((Date.now() - t0) / 1000);
  const raf = await page.evaluate(() => new Promise((res) => { let n = 0; const t = performance.now(); const f = () => { n++; if (performance.now() - t < 2000) requestAnimationFrame(f); else res(n / ((performance.now() - t) / 1000)); }; requestAnimationFrame(f); }));
  const s = await page.evaluate(() => window.__town.stats());
  if (shots) await page.screenshot({ path: resolve(shots, "town-gpu-24.png") });
  // 60 fps = every frame the display gives (WSLg's compositor runs ~58.5): ≥ 95 % of the display's own rate, the frame's CPU far under it
  check(busy >= 0.95 * raf && s.walkers >= 24 && s.mode === "gl" && s.cpuP95 < 8, `busy: ${busy.toFixed(1)} fps of the display's ${raf.toFixed(1)}, cpu p95 ${s.cpuP95.toFixed(2)} ms, with ${s.walkers} walkers on screen (${s.mode}, k ${s.k})`);
  // idle: nothing for 10 s, then 4 s measured
  await sleep(10_500);
  f0 = await frames(); t0 = Date.now(); await sleep(4000);
  const idle = ((await frames()) - f0) / ((Date.now() - t0) / 1000);
  check(idle <= 30 && idle > 0, `idle (no input 10 s): ${idle.toFixed(1)} fps`);
  // hidden: the window minimised (CDP; a second tab in front opens its own window under WSLg and leaves this one visible)
  const cdp = await ctx.newCDPSession(page);
  const { windowId } = await cdp.send("Browser.getWindowForTarget");
  await cdp.send("Browser.setWindowBounds", { windowId, bounds: { windowState: "minimized" } });
  await sleep(1000);
  let hid = await page.evaluate(() => document.visibilityState);
  if (hid !== "hidden") {
    // (WSLg keeps a minimised window `visible`: the page is told it is hidden — the town's own loop must stop while rAF still runs)
    hid = await page.evaluate(() => { Object.defineProperty(document, "hidden", { configurable: true, get: () => true }); Object.defineProperty(document, "visibilityState", { configurable: true, get: () => "hidden" }); document.dispatchEvent(new Event("visibilitychange")); return `${document.visibilityState} (emulated)`; });
  }
  f0 = await frames(); await sleep(3000);
  const hidden = (await frames()) - f0;
  check(hidden === 0 && hid.startsWith("hidden"), `hidden (${hid}): ${hidden} frames in 3 s`);
  await cdp.send("Browser.setWindowBounds", { windowId, bounds: { windowState: "normal" } });
} catch (e) {
  check(false, `threw: ${e.message}`);
}
await browser.close();
console.log(out.join("\n"));
console.log(failed ? `town-gpu: ${failed} failed` : "town-gpu: all passed");
process.exit(failed ? 1 : 0);
