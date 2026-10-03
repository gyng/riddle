#!/usr/bin/env node
// The frame's layout gates (the owner's IA: docs/UI.md §2).
// (a) Phone 400×800: on every screen (camp, watch, death, report) and with each sheet / panel open (the why sheet from a tablet, the
//     forecast, the loadout, the unlocks, the oath board), with the divergence scene playing and the fold line up where reached, the
//     console, the portrait and the primary gem are on screen and NOTHING covers the gem (elementFromPoint at its centre and four inner
//     points is the gem or inside it); no horizontal scroll.
// (b) Desktop 1440×900 (web/src/wide.css): the three columns — the rules left of the well, the shaft right of it — the console across
//     ≥ 90 % of the width at the bottom, nothing wider than the viewport, and a sheet opened from a tablet stands beside it (never over it).
//
//   node web/tests/layout.mjs        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0, checks = 0;
const check = (ok, what) => { checks++; out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
async function newPage(w, h) {
  const page = await browser.newPage({ viewport: { width: w, height: h }, deviceScaleFactor: 1 });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  return page;
}
const state = (page) => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; }).catch(() => null);
async function waitFor(page, pred, label, timeout = 60_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state(page);
    if (s?.screen === "exit") { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 1500 }).catch(() => {}); await page.locator(".sheet-wrap .btn.primary").first().click({ timeout: 1500 }).catch(() => {}); }
    if (pred(s)) return s;
    await sleep(100);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen})`);
}
async function settle(page, max = 15_000) { const t = Date.now(); let c = 0; while (Date.now() - t < max) { const s = await state(page); c = s && s.booted && !s.busy ? c + 1 : 0; if (c >= 3) break; await sleep(150); } await sleep(300); }
const closeSheets = async (page) => { for (let i = 0; i < 3; i++) { const x = page.locator(".sheet-x:visible, .close-stud:visible").first(); if (await x.count()) await x.click({ timeout: 2000 }).catch(() => {}); else await page.keyboard.press("Escape"); await sleep(150); } };
const tile = async (page, id) => { const l = page.locator(`button[data-tile="${id}"]`).first(); if (!(await l.count())) return false; await l.click({ timeout: 3000 }).catch(() => {}); await sleep(250); return true; };

/** The console, the portrait and the gem on screen; nothing over the gem; no horizontal scroll. */
async function gemClear(page, what) {
  const r = await page.evaluate(() => {
    const on = (el) => { if (!el) return false; const b = el.getBoundingClientRect(); return b.width > 4 && b.height > 4 && b.top >= -1 && b.left >= -1 && b.bottom <= innerHeight + 1 && b.right <= innerWidth + 1; };
    const con = [...document.querySelectorAll("footer.console")].find((e) => e.offsetParent !== null) ?? null;
    const gem = con?.querySelector(".gem-slot > *") ?? null, por = con?.querySelector(".well-slot > *") ?? null;
    const cover = [];
    if (gem) { const b = gem.getBoundingClientRect(); for (const [fx, fy] of [[.5, .5], [.3, .3], [.7, .3], [.3, .7], [.7, .7]]) { const hit = document.elementFromPoint(b.left + b.width * fx, b.top + b.height * fy); if (hit && hit !== gem && !gem.contains(hit)) cover.push(`${hit.tagName.toLowerCase()}.${String(hit.className?.baseVal ?? hit.className ?? "").split(/\s+/).slice(0, 2).join(".")}`); } }
    return { con: on(con), gem: on(gem), por: on(por), cover: [...new Set(cover)], wide: document.documentElement.scrollWidth - innerWidth };
  });
  check(r.con && r.gem && r.por, `${what}: console, portrait, gem on screen`);
  check(!r.cover.length, `${what}: nothing over the gem${r.cover.length ? ` (${r.cover.join(", ")})` : ""}`);
  check(r.wide <= 1, `${what}: no horizontal scroll${r.wide > 1 ? ` (${r.wide}px)` : ""}`);
}
async function toRunEnd(page) {
  const t = Date.now();
  while (Date.now() - t < 150_000) {
    const s = await state(page); if (s?.screen !== "watch" && s?.screen !== "exit") break;
    if (s?.screen === "exit") { await waitFor(page, (x) => x && x.screen !== "exit" && x.screen !== "watch", "after exit"); break; }
    await page.evaluate(() => document.querySelector('button[data-tile="skip"]')?.click()); await sleep(150);
  }
  await settle(page);
}
const RICH = JSON.stringify({ rows: [
  { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } },
  { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } },
  { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "boss" } },
  { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } } ] });

try {
  // ---- (a) the phone ----
  const page = await newPage(400, 800);
  await page.goto(`${url}?dev=1&fresh=1&seed=4242`);
  await waitFor(page, (s) => s?.booted && s.screen === "camp", "camp"); await settle(page);
  await gemClear(page, "phone camp (fresh)");
  // a run: the watch (and its fold line when one comes up)
  await page.locator(".gem-slot .gem").first().click();
  await waitFor(page, (s) => s?.screen === "watch", "watch"); await sleep(1500);
  await gemClear(page, "phone watch");
  let fold = false;
  for (let i = 0; i < 40 && (await state(page))?.screen === "watch"; i++) {
    if (!fold && await page.locator(".watch .fold-line:not([hidden]):not(.docked)").count()) { fold = true; await gemClear(page, "phone watch, the fold line up"); }
    await page.evaluate(() => document.querySelector('button[data-tile="skip"]')?.click()); await sleep(250);
  }
  await toRunEnd(page);
  const after = (await state(page))?.screen;
  if (after === "death") await gemClear(page, "phone death (the first run)");
  // The absence's layout belongs to the scout phase; manual sends wait at home before he is hired (Cut 30.5).
  await page.evaluate(async () => {
    const r = window.__riddle, blob = JSON.parse(r.exportSave()), save = JSON.parse(blob.engine);
    for (const id of ["porter", "scout"]) if (!save.lineage.tree.hired.some(([worker]) => worker === id)) save.lineage.tree.hired.push([id, save.lineage.day ?? 0]);
    blob.engine = JSON.stringify(save);
    if (!await r.importSave(JSON.stringify(blob))) throw new Error("Could not load the scout-phase layout fixture");
    if (!r.lineage.tree.auto_send) throw new Error("The absence layout fixture requires the scout");
  });
  // the absence: the report, its worst death
  await page.goto(`${url}?dev=1&absent=8h`);
  await waitFor(page, (s) => s?.booted && (s.screen === "report" || s.screen === "ending"), "report", 240_000); await settle(page);
  await gemClear(page, "phone report");
  if (await tile(page, "open")) {
    await waitFor(page, (s) => s?.screen === "death", "death"); await settle(page);
    await gemClear(page, "phone death");
    await tile(page, "camp"); await waitFor(page, (s) => s?.screen === "camp", "camp");
  }
  await settle(page);
  if ((await state(page))?.screen === "report") { await page.locator("main.report .gem").first().click().catch(() => {}); await waitFor(page, (s) => s?.screen === "camp", "camp"); await settle(page); }
  await gemClear(page, "phone camp");
  // the divergence scene over the well
  await page.evaluate((t) => window.__riddle.setRulesText(t), RICH);
  { const t = Date.now(); let seen = false; while (Date.now() - t < 30_000) { if (await page.locator(".div-scene:not([hidden])").count()) { seen = true; break; } await sleep(100); }
    if (seen) { await sleep(300); await gemClear(page, "phone camp, the scene playing"); } else out.push("note no scene played (move too small)"); }
  await sleep(4000); await settle(page);
  // each sheet and panel
  const sheets = [
    ["the why / edit sheet", async () => { await page.locator("main.camp .row.tablet:not(.oath-tab)").first().click({ timeout: 3000 }); }],
    ["the forecast", async () => { await page.locator(".shaft").first().click({ timeout: 3000, force: true }); }],
    ["the loadout", () => tile(page, "loadout")],
    ["the unlocks", () => tile(page, "unlocks")],
    ["the oath board", async () => { const o = page.locator(".oath-tab:not([hidden])").first(); if (!(await o.count())) return false; await o.click({ timeout: 3000, force: true }); }],
  ];
  for (const [name, open] of sheets) {
    await closeSheets(page);
    const r = await open().catch(() => false);
    if (r === false) { out.push(`note ${name}: not on this camp`); continue; }
    await sleep(500);
    await gemClear(page, `phone camp, ${name} open`);
  }
  await closeSheets(page);
  await page.close();

  // ---- (b) the desktop ----
  const d = await newPage(1440, 900);
  await d.goto(`${url}?dev=1&fresh=1&seed=4243`);
  await waitFor(d, (s) => s?.booted && s.screen === "camp", "desktop camp"); await settle(d);
  const cols = (sel) => d.evaluate((sel) => {
    const r = (q) => { const e = [...document.querySelectorAll(q)].find((x) => x.offsetParent !== null || getComputedStyle(x).position === "fixed"); if (!e) return null; const b = e.getBoundingClientRect(); return { l: b.left, r: b.right, t: b.top, b: b.bottom, w: b.width, h: b.height }; };
    const con = r("footer.console");
    let over = 0; for (const e of document.querySelectorAll("main.frame *")) { const b = e.getBoundingClientRect(); if (b.width > innerWidth + 1) over++; }
    return { left: r(sel.left), well: r(sel.well), right: r(sel.right), con, W: innerWidth, H: innerHeight, over, scroll: document.documentElement.scrollWidth - innerWidth };
  }, sel);
  const three = async (what, sel) => {
    const c = await cols(sel);
    check(!!c.left && !!c.well && !!c.right && c.left.w > 100 && c.well.w > 300 && c.right.w > 100, `${what}: three columns (${[c.left?.w, c.well?.w, c.right?.w].map((x) => Math.round(x ?? 0)).join(" · ")} px)`);
    check(!!c.left && !!c.well && !!c.right && c.left.r <= c.well.l + 2 && c.well.r <= c.right.l + 2, `${what}: rules left of the well, the shaft right of it`);
    check(!!c.con && c.con.w >= 0.9 * c.W && c.con.b >= c.H - 2, `${what}: the console spans the bottom (${Math.round(c.con?.w ?? 0)} of ${c.W})`);
    check(c.over === 0 && c.scroll <= 1, `${what}: nothing wider than the viewport`);
  };
  await three("desktop camp", { left: "main.camp .tablets", well: "main.camp .town", right: "main.camp .shaft" });   // Cut 30 §3: the town stands where the vista stood
  await gemClear(d, "desktop camp");
  // a sheet from a tablet stands beside it
  const tab = d.locator("main.camp .row.tablet:not(.oath-tab)").first();
  await tab.click({ timeout: 3000 }); await sleep(500);
  // a fresh set has no why: the tap opens the editor — its row's chip opens the sheet, anchored to that row
  if (!(await d.locator(".sheet-wrap > .sheet").count())) { await d.locator("main.camp .editor .row .chip.verb").first().click({ timeout: 3000 }).catch(() => {}); await sleep(500); }
  const beside = await d.evaluate(() => {
    const t = document.querySelector("main.camp .sheet-anchor") ?? document.querySelector("main.camp .row.tablet");
    const p = [...document.querySelectorAll(".sheet-wrap > .sheet")].pop();
    if (!t || !p) return { none: true, sheet: !!p, anchor: !!t, editing: !!document.querySelector("main.camp .editor:not(.compact)") };
    const a = t.getBoundingClientRect(), b = p.getBoundingClientRect();
    const hit = a.left < b.right && a.right > b.left && a.top < b.bottom && a.bottom > b.top;
    return { hit, a: [a.left, a.top, a.right, a.bottom].map(Math.round), b: [b.left, b.top, b.right, b.bottom].map(Math.round) };
  });
  if (beside.none) out.push(`note desktop: the tablet opened no sheet (${JSON.stringify(beside)})`);
  else check(!beside.hit, `desktop: the sheet stands beside its tablet (tablet ${beside.a} · sheet ${beside.b})`);
  await closeSheets(d);
  // the watch: the same columns around the stage
  await d.locator(".gem-slot .gem").first().click();
  await waitFor(d, (s) => s?.screen === "watch", "desktop watch"); await sleep(1500);
  await three("desktop watch", { left: "main.watch > .rules-col", well: "main.watch > .stage", right: "main.watch > .side-col" });
  await gemClear(d, "desktop watch");
  await toRunEnd(d);
  const ds = (await state(d))?.screen;
  if (ds === "death") { await three("desktop death", { left: "main.death > .rules-col", well: "main.death > .well", right: "main.death > .side-col" }); await gemClear(d, "desktop death"); }
  await d.close();
} catch (e) {
  failed++; out.push(`FAIL ${e.message}`);
}
await browser.close();
for (const e of errors) out.push(`FAIL ${e}`);
failed += errors.length;
console.log(out.join("\n"));
console.log(failed ? `layout: FAIL (${failed} of ${checks})` : `layout: ok (${checks})`);
process.exit(failed ? 1 : 0);
