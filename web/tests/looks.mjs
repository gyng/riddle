#!/usr/bin/env node
// Hero looks gate: the heir's cosmetic look (`Lineage.look`: male | female | cat) is swapped from the camp's portrait stud, persists
// through the save (a reload), and every surface draws it — the camp's portrait well and the bar's mini portrait paint
// `hero_<class>_<look>`, and the watch's renderer draws the hero entity as `hero_<class>_<look>`. Real wasm engine against the dev
// server (tools/dev.sh, :5219).
//
//   node web/tests/looks.mjs        (part of `pnpm test` in web/)
//
// Walk: boot fresh (seed 31: no stud) → a 1 h absence (a best depth carves the stud) → the lineage's look is the class's own and the well paints it → the stud opens a sheet with three
// headshots (`hero_fighter_{male,female,cat}`) → one tap on `cat` swaps (the sheet closes, the well and the mini portrait repaint,
// the engine says `cat`) → reload (no fresh) → still `cat` → send (`?autosend=1`) → the renderer's hero is `hero_fighter_cat` and
// the watch's well paints it → back to `female` from the camp, the engine and the well follow. Exit 1 on any failed assertion,
// console error or page error.
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
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted } : null; });
async function waitFor(pred, label, timeout = 30_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
async function until(fn, label, timeout = 10_000) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await fn(); if (v) return v; await sleep(100); }
  return v;
}
const look = () => page.evaluate(async () => (await window.__riddle.engine.lineage()).look ?? "");
const wellArt = (sel = "main .portrait .face") => page.evaluate((sel) => document.querySelector(sel)?.dataset.art ?? "", sel);
const miniArt = () => page.evaluate(() => document.querySelector(".mini-portrait .face")?.dataset.art ?? "");

try {
  await page.goto(`${url}?dev=1&fresh=1&seed=31`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  // a fresh camp stays ≤ 8 controls: on packages the stud is carved from the first run (a best depth; camp.ts — the Cut 30 idle
  // floor rarely dies on Steady), behind the pen at the first death
  check(!(await page.locator("main .portrait .look-stud").count()), "a fresh camp has no look stud");
  // a 1 h absence returns runs (a best depth), which is how the real game reveals the stud now — not a death
  await page.goto(`${url}?dev=1&fresh=1&seed=31&absent=1h`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen !== "boot", "after an absence", 60_000);
  await page.evaluate(() => window.__riddle.go?.({ kind: "camp" }));
  await waitFor((s) => s?.screen === "camp", "camp");
  const L1 = await page.evaluate(async () => { const L = await window.__riddle.engine.lineage(); return { best: L.best_depth ?? 0 }; });
  check(L1.best > 0, `the absence earned a best depth (D${L1.best}): the stud's condition`);
  const cls = await page.evaluate(() => window.__riddle.lineage?.class ?? "fighter");
  const l0 = await look();
  check(["male", "female", "cat"].includes(l0), `a fresh lineage wears a look (${cls} · ${l0})`);
  const w0 = await until(async () => (await wellArt()) === `hero_${cls}_${l0}` && wellArt(), "well");
  check(w0 === `hero_${cls}_${l0}`, `the camp's well paints hero_${cls}_${l0} (${w0})`);

  // the stud opens the sheet: three headshots of the class
  await page.locator("main .portrait .look-stud").click({ timeout: 5000 });
  await page.waitForSelector(".look-sheet .look", { timeout: 5000 });
  const faces = await page.evaluate(() => [...document.querySelectorAll(".look-sheet .look")].map((b) => ({ look: b.dataset.look, art: b.querySelector(".face")?.dataset.art ?? "", on: b.classList.contains("on") })));
  check(faces.length === 3 && faces.every((f) => f.art === `hero_${cls}_${f.look}`), `the sheet shows three headshots (${faces.map((f) => f.art).join(", ")})`);
  check(faces.filter((f) => f.on).map((f) => f.look).join() === l0, `the worn look is lit (${faces.filter((f) => f.on).map((f) => f.look)})`);
  const target = l0 === "cat" ? "female" : "cat";
  await page.locator(`.look-sheet .look[data-look=${target}]`).click({ timeout: 5000 });
  await until(async () => !(await page.locator(".look-sheet").count()), "sheet closed", 5000);
  check(!(await page.locator(".look-sheet").count()), "one tap swaps and closes the sheet");
  check((await look()) === target, `the engine wears ${target} (${await look()})`);
  const w1 = await until(async () => (await wellArt()) === `hero_${cls}_${target}` && wellArt(), "well repaint");
  check(w1 === `hero_${cls}_${target}`, `the well repaints hero_${cls}_${target} (${w1})`);
  const m1 = await until(async () => (await miniArt()) === `hero_${cls}_${target}` && miniArt(), "mini repaint");
  check(m1 === `hero_${cls}_${target}`, `the bar's mini portrait follows (${m1})`);

  // the save carries it
  await page.evaluate(() => window.__riddle.flush?.());   // the save is debounced (1 s)
  await page.waitForTimeout(1500);
  await page.goto(`${url}?dev=1&seed=31`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp after reload");
  check((await look()) === target, `a reload keeps ${target} (${await look()})`);
  const w2 = await until(async () => (await wellArt()) === `hero_${cls}_${target}` && wellArt(), "well after reload");
  check(w2 === `hero_${cls}_${target}`, `the well after a reload paints hero_${cls}_${target} (${w2})`);

  // the renderer draws it
  await page.goto(`${url}?dev=1&seed=31&autosend=1`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.screen === "watch", "the watch");
  const kind = await until(() => page.evaluate(() => window.__viewer?.debugPos?.().find((e) => e.hero)?.kind ?? ""), "hero kind", 15_000);
  check(kind === `hero_${cls}_${target}`, `the renderer's hero is hero_${cls}_${target} (${kind || "—"})`);
  const atlasHas = await page.evaluate(async (id) => { const a = await (await fetch("/art/atlas.json")).json(); return !!a.frames[id]; }, `hero_${cls}_${target}`);
  check(atlasHas, `the atlas packs hero_${cls}_${target}`);
  const ww = await until(async () => (await wellArt(".watch .portrait .face")) === `hero_${cls}_${target}` && wellArt(".watch .portrait .face"), "watch well");
  check(ww === `hero_${cls}_${target}`, `the watch's well paints hero_${cls}_${target} (${ww})`);

  // and back, from the camp
  await page.goto(`${url}?dev=1&seed=31`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp again");
  await page.evaluate(() => window.__riddle.go?.({ kind: "camp" }));
  await page.locator("main .portrait .look-stud").click({ timeout: 5000 });
  await page.locator(".look-sheet .look[data-look=female]").click({ timeout: 5000 });
  const w3 = await until(async () => (await wellArt()) === `hero_${cls}_female` && wellArt(), "well female");
  check((await look()) === "female" && w3 === `hero_${cls}_female`, `swapped back to female (${await look()} · ${w3})`);
} catch (e) {
  check(false, `walk: ${e.message}`);
}
await browser.close();
for (const e of errors) check(false, e);
console.log(out.join("\n"));
console.log(failed ? `FAIL: ${failed} failed` : `ok: ${out.length} checks`);
process.exit(failed ? 1 : 0);
