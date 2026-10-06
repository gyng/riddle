#!/usr/bin/env node
// Cut 28 client gates, the watch's half (docs/CUT28.md §3–§4), on the fake engine, headless at 400 × 800:
//   §4  the watch's pixel text (the callout over the hero, the fight frame's caption) never lands on the DOM over the canvas — the
//       HUD line, the docked fold line's head and chips, the banner, the ticker — nor on a foe's name plate; a name plate never lands
//       on those chips either (AU: `R3 BANK` over `gas>pack · bp 5/40 · 14 finds`; `archer` across `R7 ATTACK ARCHER`; AV: `R2 RETURN`
//       over `stolen darkness scroll · hp 11/38`). Read every frame of watched runs (a folded start, so the fold chips dock), from the
//       renderer's own boxes (`__viewer.debugText()` / `debugLabels()`, canvas CSS px) against the DOM's rects.
//   §3  owner compact-control amendment: Speed names the mode, changes it without sending, and remembers it across a reload.
//
//   node web/tests/cut28w.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync, readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const GOOD = JSON.parse(readFileSync(resolve(ROOT, "crates/riddle-core/presets/good.json"), "utf8")).rows.slice(0, 4);
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: false }); };
const screen = () => page.evaluate(() => window.__riddle ? { screen: window.__riddle.screen, booted: window.__riddle.booted } : null);
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await screen(); if (pred(s)) return s; await sleep(60); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen})`);
}

try {
  // Current owner contract: one Speed selector in watch; selection persists without a new Send.
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=41&autosend=1`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
  check(await page.locator(".watch-speed-mode").innerText() === "Fights only", "a fresh watch names the default mode");
  await page.locator(".console .gem").click();
  await page.waitForFunction(() => document.querySelector(".watch")?.dataset.speed === "0");
  await page.locator(".console [data-tile=speed]").click();
  // Sample the actual selection synchronously: pausing the viewer does not pause the world.
  const chosen = await page.evaluate(() => {
    const r = window.__riddle, before = JSON.stringify(r.lineage);
    document.querySelector('.watch-options [data-tile=one]').click();
    return { mode: document.querySelector('.watch')?.dataset.mode,
      label: document.querySelector('.watch-speed-mode')?.textContent,
      menu: !!document.querySelector('.sheet-wrap .watch-options'), unchanged: JSON.stringify(r.lineage) === before };
  });
  check(chosen.mode === "one" && chosen.label === "Normal" && !chosen.menu && chosen.unchanged,
    `Speed changes the viewer mode, closes the menu and never sends (${JSON.stringify(chosen)})`);
  await shot("cut28w-speed-mode");
  await page.locator(".console [data-tile=town]").click();
  await waitFor((s) => s.screen === "camp", "Town menu");
  await page.evaluate(() => window.__riddle.flush?.());
  await sleep(1200);
  await page.goto(`${url}?dev=1&engine=fake&systems=none&seed=41`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp after a reload");
  check(await page.evaluate(() => window.__riddle.watchMode) === "one", "the chosen watch mode survives a reload");

  // ---- §4: no pixel text on the DOM chips, no plate on either
  const tally = { frames: 0, callouts: 0, captions: 0, plates: 0, docked: 0, hits: [] };
  for (const [seed, fold] of [[26, 2], [157, 0], [5, 2]]) {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}${fold ? `&fake_fold=${fold}` : ""}`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", `the camp (seed ${seed})`);
    if (await page.evaluate(() => window.__riddle.town?.home === false)) {
      await page.locator('.town-tag[data-next="house"]').click();
      await page.waitForFunction(() => window.__riddle.town?.home === true);
    }
    // First-load camp defers forecasts. Request it explicitly; use the supported client fold path to exercise the overlay (core fold correctness is cut27).
    if (fold) {
      await page.evaluate(async rows => { const r = window.__riddle; r.rules.rows = rows; r.rulesChanged(); await r.engine.setRules(r.rules); const off = r.onForecast(() => {}); try { await r.emitForecast(); } finally { off(); } r.engine.fold = undefined; }, GOOD);
      await page.waitForFunction(() => window.__riddle.forecastOfRules()?.fold_to !== undefined, null, { timeout: 15_000 });
    }
    await page.evaluate(() => document.querySelector("button.gem.send")?.click());
    await waitFor((s) => s?.booted && s.screen === "watch", `the watch (seed ${seed})`);
    if (fold) check(await page.evaluate(() => !!document.querySelector(".watch")?.dataset.foldPlan), `seed ${seed}: genuine forecast populates the fold plan`);
    const r = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(), acc = { frames: 0, callouts: 0, captions: 0, plates: 0, docked: 0, hits: [] };
      const inter = (a, b) => a.x < b.x + b.w - 1 && a.x + a.w > b.x + 1 && a.y < b.y + b.h - 1 && a.y + a.h > b.y + 1;
      const poll = () => {
        const w = document.querySelector("main.watch"), v = window.__viewer, cv = w?.querySelector("canvas");
        if (w && v?.debugText && cv && window.__riddle?.screen === "watch" && w.dataset.ending !== "1") {
          const c = cv.getBoundingClientRect();
          const fold = w.querySelector(".fold-line.docked:not([hidden])");
          const moving = !!fold && fold.getAnimations().some((a) => a.playState === "running");
          if (!moving) {
            acc.frames++;
            const dom = [...w.querySelectorAll(".hud.top > *"), ...(fold ? [fold.querySelector(".fold-head"), ...fold.querySelectorAll(".fchip")] : []), ...w.querySelectorAll(".banner.show, .ticker.show")]
              .filter((e) => e && !e.hidden && getComputedStyle(e).opacity !== "0" && getComputedStyle(e).visibility !== "hidden")
              .map((e) => { const r = e.getBoundingClientRect(); return { what: (e.textContent || e.className).trim().slice(0, 24), x: r.left - c.left, y: r.top - c.top, w: r.width, h: r.height }; })
              .filter((r) => r.w > 0 && r.h > 0 && (r.what.length > 0));
            if (fold) acc.docked++;
            const texts = v.debugText().filter((t) => t.w !== undefined);
            const plates = (v.debugLabels?.() ?? []).filter((l) => l.w).map((l) => ({ what: l.text, x: l.x - l.w / 2, y: l.y - l.h, w: l.w, h: l.h }));
            acc.callouts += texts.filter((t) => t.kind === "callout").length; acc.captions += texts.filter((t) => t.kind === "caption").length; acc.plates += plates.length;
            for (const t of texts) {
              for (const d of dom) if (inter(t, d)) acc.hits.push(`${t.kind} "${t.text}" on "${d.what}"`);
              for (const p of plates) if (inter(t, p)) acc.hits.push(`${t.kind} "${t.text}" on plate "${p.what}"`);
            }
            for (const p of plates) for (const d of dom) if (inter(p, d)) acc.hits.push(`plate "${p.what}" on "${d.what}"`);
          }
        }
        if (performance.now() - t0 > 30_000 || (window.__riddle?.screen !== "watch" && performance.now() - t0 > 2000)) { res(acc); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    for (const k of ["frames", "callouts", "captions", "plates", "docked"]) tally[k] += r[k];
    tally.hits.push(...r.hits);
  }
  const uniq = [...new Set(tally.hits)];
  check(tally.frames > 200 && tally.callouts + tally.captions > 20 && tally.plates > 20 && tally.docked > 0 && tally.hits.length === 0,
    `no callout or caption on the HUD, the docked fold chips, the banner, the ticker or a name plate; no plate on the chips (${tally.frames} frames · ${tally.callouts} callouts · ${tally.captions} captions · ${tally.plates} plates · ${tally.docked} docked; ${tally.hits.length} overlaps${uniq.length ? `: ${uniq.slice(0, 6).join(" | ")}` : ""})`);
  await shot("cut28w-watch");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut28w: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut28w: ok (${out.length} checks)`);
