#!/usr/bin/env node
// Scripted walk of the whole loop through the browser harness (tools/browser.mjs; headless by default,
// `--headed` for the GPU window when the walk is about how it looks or feels) against the dev server
// (tools/dev.sh, port 5219). Every screen is dumped as <out>/NN-<screen>.txt (innerText) + .png, one summary
// line per dump, total wall time at the end (also in <out>/summary.txt). Exit 1 on any console error, page
// error or unexpected reload. A Vite full reload from a parallel edit (pkg/, engine/, ui/) kills a walk: it is
// reported, and the walk is retried once from scratch.
//
//   node tools/playtest.mjs [--seed N] [--absent 8h] [--rules file.txt] [--out dir] [--wide] [--headed] [--dpr N]
//
// Walk: boot fresh(seed[, rules]) → camp → send → watch at 4× (two mid-run shots, then ▶▶| until the exit,
// the last frame kept as the final watch dump) → exit sheet (keep) → death (tap the first patch) | report (camp)
// → camp → reload ?absent → report (offline wait timed) → open the worst death → done.
// Dev URL params and window.__riddle are documented in web/src/main.ts and web/src/app.ts.
import { execFileSync } from "node:child_process";
import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "./browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const USAGE = "usage: node tools/playtest.mjs [--seed N] [--absent 8h] [--rules file.txt] [--out dir] [--wide] [--headed] [--dpr N]";
const MID_SHOTS_MS = [2500, 6000];   // two mid-run dumps at 4×, then ▶▶| to the exit
const SKIP_EVERY_MS = 40;            // ▶▶| cadence (a press while the pump is in flight is a no-op)
const FINAL_EVERY_MS = 1500;         // rolling "final frame" of the run
const WATCH_MAX_MS = 120_000, OFFLINE_MAX_MS = 180_000, STEP_MAX_MS = 30_000, SETTLE_MAX_MS = 10_000;
const ATTEMPTS = 2;                  // one retry, only after an unexpected reload

const opt = { seed: 1, absent: "8h", rules: null, out: null, wide: false, headed: undefined, dpr: undefined };
for (let i = 2; i < process.argv.length; i++) {
  const a = process.argv[i], v = process.argv[i + 1];
  if (a === "--seed") { opt.seed = Number(v); i++; }
  else if (a === "--absent") { opt.absent = v; i++; }
  else if (a === "--rules") { opt.rules = readFileSync(resolve(v), "utf8"); i++; }
  else if (a === "--out") { opt.out = resolve(v); i++; }
  else if (a === "--wide") opt.wide = true;
  else if (a === "--headed") opt.headed = true;
  else if (a === "--headless") opt.headed = false;
  else if (a === "--dpr") { opt.dpr = Number(v); i++; }
  else if (a === "--help" || a === "-h") { console.log(USAGE); process.exit(0); }
  else { console.error(`unknown argument ${a}\n${USAGE}`); process.exit(2); }
}
if (!Number.isFinite(opt.seed)) { console.error("--seed needs a number"); process.exit(2); }
const out = opt.out ?? join(ROOT, "docs/playtests", new Date().toISOString().replace(/[-:]/g, "").replace(/\..+/, "").replace("T", "-"));
mkdirSync(out, { recursive: true });

const t0 = Date.now();
const secs = (ms) => `${(ms / 1000).toFixed(1)}s`;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const url = execFileSync("bash", [join(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();

const errors = [], warnings = [], summary = [];
let n = 0, walkStart = 0, offlineWait = 0;
const first = (s) => s.split("\n").map((x) => x.trim()).filter(Boolean).slice(0, 4).join(" · ").slice(0, 60);
const name = (screen) => join(out, `${String(++n).padStart(2, "0")}-${screen}`);
const log = (line) => { console.log(line); summary.push(line); };

async function attempt() {
  let navigating = false;
  const headed = opt.headed ?? (process.env.RIDDLE_BROWSER === "headed");
  const browser = await launchBrowser({ headed });
  // Headless WebGL is SwiftShader and pixel-bound (14 fps at 3×, 32 at 2×, 60 at 1× on a 400×800
  // watch); the phone walk renders at 2× headless, 3× on the GPU — the same layout either way.
  const dpr = opt.dpr ?? (headed ? 3 : 2);
  const page = await browser.newPage(opt.wide ? { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 } : { viewport: { width: 400, height: 800 }, deviceScaleFactor: dpr });
  page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); else if (m.type() === "warning") warnings.push(m.text()); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  page.on("framenavigated", (f) => { if (f === page.mainFrame() && !navigating) errors.push(`unexpected reload at ${secs(Date.now() - walkStart)} (a Vite full reload from a parallel edit?) → ${f.url()}`); });

  const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
  const text = () => page.evaluate(() => window.__riddle?.text() ?? "");
  async function waitFor(pred, label, timeout = STEP_MAX_MS) {
    const t = Date.now(); let s = null;
    while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(120); }
    throw new Error(`timeout after ${secs(timeout)} waiting for ${label} (screen=${s?.screen} booted=${s?.booted} busy=${s?.busy})`);
  }
  /** Wait until the progress bar has been down for 300 ms (forecast / verdict / offline slice), then a paint. */
  async function settle() {
    const t = Date.now(); let clear = 0;
    while (Date.now() - t < SETTLE_MAX_MS) { const s = await state(); clear = s && s.booted && !s.busy ? clear + 1 : 0; if (clear >= 2) break; await sleep(150); }
    await sleep(150);
  }
  async function goto(q) {
    navigating = true;
    try { await page.goto(`${url}?dev=1&${q}`, { waitUntil: "domcontentloaded" }); } finally { await sleep(50); navigating = false; }
  }
  /** Write NN-<screen>.txt + .png and print the summary line. Full-page unless it is the run or a sheet. */
  async function dump(screen, { base = name(screen), full = screen !== "watch" && screen !== "exit", note = "" } = {}) {
    const tx = await text();
    writeFileSync(`${base}.txt`, tx);
    await page.screenshot({ path: `${base}.png`, fullPage: full });
    log(`${base.slice(out.length + 1).padEnd(14)} +${secs(Date.now() - walkStart).padStart(6)}  ${String(tx.length).padStart(5)} ch  ${first(tx)}${note ? `  [${note}]` : ""}`);
    return base;
  }
  const press = (label) => page.evaluate((l) => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === l) { b.click(); return true; } return false; }, label);
  const clickBtn = async (sel, label) => { const loc = label ? page.locator(sel).filter({ hasText: new RegExp(`^${label}$`) }).first() : page.locator(sel).first(); await loc.click({ timeout: 5000 }); };

  try {
    // 1. boot fresh with the seed (and rules), camp
    walkStart = Date.now();
    await goto(`fresh=1&seed=${opt.seed}&speed=4${opt.rules ? `&rules=${encodeURIComponent(opt.rules)}` : ""}`);
    await waitFor((s) => s?.booted && s.screen === "camp", "camp");
    await settle();
    await dump("camp");

    // 2. send; watch at 4×: two mid-run dumps, then ▶▶| until the exit, keeping the last frame
    await clickBtn("button.send");
    await waitFor((s) => s?.screen === "watch", "watch");
    const tw = Date.now(); const mids = [...MID_SHOTS_MS];
    let finalBase = null, lastFinal = 0, lastSkip = 0, presses = 0, s;
    let vaults = 0;
    for (;;) {
      s = await state();
      // a mid-run vault choice (Cut 5) opens a sheet over the watch: pick the first item and carry on
      if (s?.screen === "exit" && await page.locator(".sheet-wrap .vault-choice .chip").count()) {
        if (vaults++ === 0) await dump("vault");
        await page.locator(".sheet-wrap .vault-choice .chip").first().click().catch(() => {});
        await sleep(300); continue;
      }
      if (s?.screen !== "watch") break;
      const el = Date.now() - tw;
      if (el > WATCH_MAX_MS) throw new Error(`run still going after ${secs(WATCH_MAX_MS)}`);
      if (mids.length) { if (el >= mids[0]) { mids.shift(); await dump("watch"); } await sleep(100); continue; }
      if (Date.now() - lastSkip >= SKIP_EVERY_MS) { lastSkip = Date.now(); if (await press("▶▶|")) presses++; }
      if (Date.now() - lastFinal >= FINAL_EVERY_MS) {
        lastFinal = Date.now(); finalBase ??= name("watch");
        writeFileSync(`${finalBase}.txt`, await text()); await page.screenshot({ path: `${finalBase}.png` });
      }
      await sleep(SKIP_EVERY_MS / 2);
    }
    if (!s) throw new Error("the page lost window.__riddle during the run");
    if (finalBase) { const tx = readFileSync(`${finalBase}.txt`, "utf8"); log(`${finalBase.slice(out.length + 1).padEnd(14)} +${secs(Date.now() - walkStart).padStart(6)}  ${String(tx.length).padStart(5)} ch  ${first(tx)}  [final frame · run ${secs(Date.now() - tw)} · ${presses} skips]`); }

    // 3. exit sheet → keep; then death (first patch) | report (camp) | ending
    if (s.screen === "exit") {
      await settle(); await dump("exit");
      await clickBtn(".sheet-wrap .btn.primary");
      s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit", "the screen after keep");
    }
    await settle();
    if (s.screen === "death") {
      await dump("death");
      if (await page.locator("button.patch").count()) { await clickBtn("button.patch"); await waitFor((x) => x?.screen === "camp", "camp after the patch"); await settle(); await dump("camp", { note: "patched" }); }
      else { await clickBtn("button.btn.primary", "edit"); await waitFor((x) => x?.screen === "camp", "camp after edit"); await settle(); await dump("camp", { note: "no patch offered" }); }
    } else if (s.screen === "report") {
      await dump("report", { note: "returned" });
      await clickBtn("button.btn.primary", "camp"); await waitFor((x) => x?.screen === "camp", "camp after the report"); await settle(); await dump("camp");
    } else if (s.screen === "ending") await dump("ending");
    else await dump(s.screen, { note: "unexpected" });

    // 4. come back after the absence: offline report (timed), then the worst death
    const ta = Date.now();
    await goto(`absent=${encodeURIComponent(opt.absent)}`);
    await waitFor((x) => x?.booted && (x.screen === "report" || x.screen === "ending"), `the ${opt.absent} offline report`, OFFLINE_MAX_MS);
    offlineWait = Date.now() - ta;
    await settle();
    s = await state();
    await dump(s.screen, { note: `offline ${opt.absent} → ${secs(offlineWait)}` });
    if (s.screen === "report") {
      if (await page.locator("button.btn").filter({ hasText: /^open$/ }).count()) {
        await clickBtn("button.btn", "open"); await waitFor((x) => x?.screen === "death", "the worst death"); await settle(); await dump("death", { note: "worst" });
      } else log("(no worst death to open)");
    }
  } catch (e) {
    errors.push(`walk aborted: ${e.message}`);
  } finally {
    await browser.close().catch(() => {});
  }
}

for (let a = 1; a <= ATTEMPTS; a++) {
  errors.length = 0; warnings.length = 0; summary.length = 0; n = 0; offlineWait = 0;
  await attempt();
  const reload = errors.find((e) => e.startsWith("unexpected reload"));
  if (reload && a < ATTEMPTS) { console.log(`retrying once after: ${reload}\n`); continue; }
  break;
}

log(`total ${secs(Date.now() - t0)} (walk ${secs(Date.now() - walkStart)}, offline ${opt.absent} ${secs(offlineWait)}) · ${n} dumps · seed ${opt.seed} · ${opt.wide ? "wide" : "phone"} → ${out}`);
if (warnings.length) console.log(`${warnings.length} console warning(s): ${[...new Set(warnings)].slice(0, 5).join(" | ").slice(0, 400)}`);
let code = 0;
if (errors.length) { code = 1; console.error(`FAIL: ${errors.length} error(s)`); for (const e of errors) console.error("  " + e); summary.push("FAIL", ...errors); }
else console.log("ok: no console errors, no page errors");
writeFileSync(join(out, "summary.txt"), summary.join("\n") + "\n");
process.exit(code);
