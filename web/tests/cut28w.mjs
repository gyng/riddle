#!/usr/bin/env node
// Cut 28 client gates, the watch's half (docs/CUT28.md §3–§4), on the fake engine, headless at 400 × 800:
//   §4  the watch's pixel text (the callout over the hero, the fight frame's caption) never lands on the DOM over the canvas — the
//       HUD line, the docked fold line's head and chips, the banner, the ticker — nor on a foe's name plate; a name plate never lands
//       on those chips either (AU: `R3 BANK` over `gas>pack · bp 5/40 · 14 finds`; `archer` across `R7 ATTACK ARCHER`; AV: `R2 RETURN`
//       over `stolen darkness scroll · hp 11/38`). Read every frame of watched runs (a folded start, so the fold chips dock), from the
//       renderer's own boxes (`__viewer.debugText()` / `debugLabels()`, canvas CSS px) against the DOM's rects.
//   §3  the send gem names the watch's mode always (`▸ fights` too); a tap on the pill steps the mode, never sends; it is remembered
//       across a reload.
//
//   node web/tests/cut28w.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
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
const gemText = () => page.evaluate(() => { const g = document.querySelector(".gem.send"); return { text: g?.textContent.replace(/\s+/g, " ").trim(), mode: g?.dataset.mode }; });

try {
  // ---- §3: the send gem names the mode, the pill steps it (never a send), a reload keeps it
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=41`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp");
  await sleep(300);
  const g0 = await gemText();
  check(g0.text === "send▸ fights only" && g0.mode === "fights", `a fresh camp's gem names the default mode (${JSON.stringify(g0)})`);
  await page.locator(".gem.send .send-mode").click();
  await sleep(200);
  const g1 = await gemText(), s1 = await screen();
  check(s1.screen === "camp" && g1.text === "send▸ normal" && g1.mode === "one", `a tap on the pill steps the mode to normal and does not send (${JSON.stringify(g1)}, ${s1.screen})`);
  await shot("cut28w-send-mode");
  await page.evaluate(() => window.__riddle.flush?.());
  await sleep(1200);   // the save's debounce
  await page.goto(`${url}?dev=1&engine=fake&systems=none&seed=41`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp after a reload");
  await sleep(300);
  const g2 = await gemText();
  check(g2.mode === "one" && g2.text === "send▸ normal", `the chosen mode is remembered across a reload (${JSON.stringify(g2)})`);
  await page.evaluate(() => { window.__riddle.watchMode = "fights"; window.__riddle.persist(); });

  // ---- §4: no pixel text on the DOM chips, no plate on either
  const tally = { frames: 0, callouts: 0, captions: 0, plates: 0, docked: 0, hits: [] };
  for (const [seed, fold] of [[26, 2], [157, 0], [5, 2]]) {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}${fold ? `&fake_fold=${fold}` : ""}`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", `the camp (seed ${seed})`);
    if (fold) await page.waitForFunction(() => window.__riddle.lastForecast?.fold_to !== undefined, null, { timeout: 15_000 }).catch(() => {});
    await page.evaluate(() => document.querySelector("button.gem.send")?.click());
    await waitFor((s) => s?.booted && s.screen === "watch", `the watch (seed ${seed})`);
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
