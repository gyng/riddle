#!/usr/bin/env node
// Persistent browser driver for rating and QA sessions: one browser context that lives across
// hundreds of agent turns, commanded over local HTTP (one JSON op per POST). Headless by default
// (DOM text, clicks, screenshots — fast, no desktop window); `--headed` for the GPU window when
// the session judges feel, frame rate or the render itself (docs: eval/RATING.md).
//
//   node tools/driver.mjs --dir scratchpad/raterQ [--port 5347] [--headed] [--wide] [--dpr N] &
//   tools/drive.sh 5347 '{"op":"goto","url":"http://localhost:5219/?dev=1&seed=157&fresh=1"}'
//   tools/drive.sh 5347 '{"op":"text"}'                       body innerText
//   tools/drive.sh 5347 '{"op":"click","label":"send"}'      by visible label (exact; "nth", "selector", "force", "wait" ms)
//   tools/drive.sh 5347 '{"op":"shot","name":"camp-1","full":true}'   → <dir>/shots/camp-1.png
//   tools/drive.sh 5347 '{"op":"js","body":"return await page.evaluate(() => document.title)"}'   async fn (page, ctx, browser, dir)
//   ops: goto text html shot click tap type press fill eval js wait buttons log quit state act watch send_and_watch sheets
//   compound ops (docs/ITERATION_SPEED.md §2 — one turn where it took four to fifteen):
//   tools/drive.sh 5347 '{"op":"state"}'                      screen (dev builds), busy, text, buttons in one call
//   tools/drive.sh 5347 '{"op":"act","label":"send","shot":"after-send"}'   click, settle until the engine is idle (not a fixed 600 ms), return the new text (+ a shot)
//   tools/drive.sh 5347 '{"op":"watch","ms":30000,"every":1000,"shots":4}'  sample the screen every N ms for M ms: the lines that appeared per sample + evenly spaced shots
//   tools/drive.sh 5347 '{"op":"send_and_watch","mode":"fights","every":2000,"shots":4}'   a whole send in one call: tap `send`, tap the
//        watch's `mode` button if given ("fights" / "fast" — your choice, at its own speed), sample the screen every `every` ms (the lines
//        that appeared, as `watch`), a shot every `shotEvery` ms (default 15000) up to `shots`, and stop at the run's end — a sheet over the
//        watch (the exit sheet, the cage) or the watch's controls gone (the death / report screen) — or after `max` ms (default 300000).
//        `"skip":true` taps `▶▶|` (`skipLabel`) every `skipEvery` ms (default 400) as a player would. Returns the samples, the shots, the
//        end (`sheet` | `screen` | `timeout`), the final text and buttons, the open sheet's text, and an `-end` shot. Nothing but the page.
//   tools/drive.sh 5347 '{"op":"sheets"}'                     every sheet this screen opens, in one call: in a throwaway copy of the page (the same
//        saved lineage in a second browser context, the main page untouched) each visible control is tapped once; a `.sheet-wrap` that opens is
//        read (title = its first line) and closed; a control that changed the screen instead (an in-page panel, a navigation, a mutation) is
//        listed with the lines it added, and the copy is rebuilt from the save. `max` controls (default 40). Nothing but the page.
//
// Console errors/warnings and page errors are kept in <dir>/console.log and returned by "log".
import http from "node:http";
import fs from "node:fs";
import { resolve } from "node:path";
import { launchBrowser } from "./browser.mjs";
import { pressWatchControl } from "./watch-control.mjs";

const argv = process.argv.slice(2);
const flag = (k) => argv.includes(k);
const val = (k, d) => { const i = argv.indexOf(k); return i >= 0 ? argv[i + 1] : d; };
const dir = resolve(val("--dir", "scratchpad/driver"));
const port = Number(val("--port", "5347"));
const headed = flag("--headed") ? true : flag("--headless") ? false : undefined;
fs.mkdirSync(`${dir}/shots`, { recursive: true });

const browser = await launchBrowser({ headed });
const isHeaded = headed ?? process.env.RIDDLE_BROWSER === "headed";
// Phone at 3× on the GPU, 2× headless (SwiftShader is pixel-bound; the layout is the same).
const dpr = Number(val("--dpr", isHeaded ? "3" : "2"));
const ctxOpts = flag("--wide") ? { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 } : { viewport: { width: 400, height: 800 }, deviceScaleFactor: dpr };
const ctx = await browser.newContext(ctxOpts);
const page = await ctx.newPage();
const log = [];
const note = (line) => { const l = `[${new Date().toISOString()}] ${line}`; log.push(l); fs.appendFileSync(`${dir}/console.log`, l + "\n"); };
page.on("console", (m) => { if (m.type() === "error" || m.type() === "warning") note(`console.${m.type()}: ${m.text()}`); });
page.on("pageerror", (e) => note(`pageerror: ${e.message}`));
const AsyncFunction = Object.getPrototypeOf(async function () {}).constructor;
let shotN = 0;

// The dev inspection surface when the page has it (`window.__riddle`, dev builds / `?dev=1`).
const state = async () => page.evaluate(() => {
  const r = window.__riddle;
  const buttons = [...document.querySelectorAll("button,[role=button]")].filter((b) => b.offsetWidth || b.offsetHeight).map((b) => (b.innerText || "").trim().slice(0, 40)).filter(Boolean);
  return { screen: r?.screen ?? null, busy: !!document.querySelector(".busy"), text: document.body.innerText, buttons };
});
// Settle: a frame, then until no progress bar is up (bounded), then one more frame.
async function idle(timeout) {
  const t0 = Date.now();
  await page.waitForTimeout(120);
  while (Date.now() - t0 < timeout && (await page.evaluate(() => !!document.querySelector(".busy")))) await page.waitForTimeout(100);
  await page.waitForTimeout(120);
  return Date.now() - t0;
}

// What the page shows, for the compound ops below — DOM only (the rater profile: no `__riddle`, no wire).
// A control matches a label when one of its visible text lines is that label, case aside (a tile is its icon + its one word).
// `all`: disabled ones too (the watch greys its controls through the ending; they are still its controls).
const pageButtons = (p, all = false) => p.evaluate((all) => [...document.querySelectorAll("button,[role=button]")]
  .filter((b) => (b.offsetWidth || b.offsetHeight) && (all || !b.disabled)).map((b) => (b.innerText || "").trim()).filter(Boolean), all);
const tapLabel = (p, label) => p.evaluate((l) => {
  for (const b of document.querySelectorAll("button,[role=button]")) {
    if (!(b.offsetWidth || b.offsetHeight) || b.disabled) continue;
    const t = (b.innerText || "").trim().toLowerCase();   // innerText carries CSS text-transform (`SEND`)
    if (t === l.toLowerCase() || t.split("\n").map((x) => x.trim()).includes(l.toLowerCase())) { b.click(); return true; }
  }
  return false;
}, label);
const sheetText = (p) => p.evaluate(() => { const w = [...document.querySelectorAll(".sheet-wrap")].pop(); return w ? w.innerText : null; });
const bodyText = (p) => p.evaluate(() => document.body.innerText);
/** Until the text has held still for two polls and no progress bar is up (bounded). */
async function steady(p, timeout = 15000) {
  const t0 = Date.now(); let last = null, still = 0;
  while (Date.now() - t0 < timeout) {
    const cur = await p.evaluate(() => (document.querySelector(".busy") ? "\u0000busy" : "") + document.body.innerText);
    still = cur === last && !cur.startsWith("\u0000busy") ? still + 1 : 0;
    if (still >= 2) break;
    last = cur; await p.waitForTimeout(250);
  }
  return Date.now() - t0;
}
const digitless = (t) => t.replace(/\d+/g, "#");
const lineDiff = (before, after) => { const b = new Set(before.split("\n")); return after.split("\n").filter((l) => l.trim() && !b.has(l)); };

async function sendAndWatch(cmd) {
  const every = cmd.every ?? 2000, max = cmd.max ?? 300000, shots = cmd.shots ?? 4, shotEvery = cmd.shotEvery ?? 15000;
  const skipLabel = cmd.skipLabel ?? "▶▶|", skipEvery = cmd.skipEvery ?? 400, name = cmd.name ?? "run";
  // The worker shortcut can also say "send"; the actual run command is the gem.
  if (await sheetText(page) !== null) return { error: "close the current choice before sending" };
  const send = cmd.send ? await tapLabel(page,cmd.send) : await page.evaluate(() => {
    const button = document.querySelector('main.camp button.send');
    if (!button || button.disabled || !button.getClientRects().length) return false;
    button.click(); return true;
  });
  if (!send) return { error: `no visible control labelled ${cmd.send ?? "send"}` };
  const t0 = Date.now();
  while (Date.now() - t0 < 15000 && !await page.locator('main.watch:visible').count()) await page.waitForTimeout(200);
  if (!await page.locator('main.watch:visible').count()) return { error: "the watch did not open after the tap", text: await bodyText(page) };
  if (cmd.mode && !await pressWatchControl(page,cmd.mode)) return { error: `no enabled watch control labelled ${cmd.mode}`, buttons: await pageButtons(page) };
  const samples = [], paths = [];
  let last = await bodyText(page), lastSkip = 0, nextSample = Date.now() + every, nextShot = Date.now() + shotEvery, ended = "timeout";
  const shoot = async (tag) => { const path = `${dir}/shots/${name}-${tag}.png`; await page.screenshot({ path }); paths.push({ t: Math.round((Date.now() - t0) / 100) / 10, path }); };
  while (Date.now() - t0 < max) {
    await page.waitForTimeout(100);
    if (await sheetText(page) !== null) { ended = "sheet"; break; }
    if (!await page.locator('main.watch:visible').count()) { ended = "screen"; break; }
    if (cmd.skip && Date.now() - lastSkip >= skipEvery) { lastSkip = Date.now(); await pressWatchControl(page, skipLabel); }
    if (Date.now() >= nextSample) {
      nextSample += every;
      const text = await bodyText(page);
      const added = lineDiff(last, text);
      if (added.length) samples.push({ t: Math.round((Date.now() - t0) / 100) / 10, added });
      last = text;
    }
    if (paths.length < shots && Date.now() >= nextShot) { nextShot += shotEvery; await shoot(String(paths.length + 1).padStart(2, "0")); }
  }
  await steady(page);
  await shoot("end");
  return { ended, ms: Date.now() - t0, samples, shots: paths, sheet: await sheetText(page), text: await bodyText(page), buttons: await pageButtons(page) };
}

async function sheets(cmd) {
  const max = cmd.max ?? 40, workers = cmd.workers ?? 4;
  const url = page.url();
  const storage = await ctx.storageState();
  const mainText = await bodyText(page);
  // A copy of this page: the same saved lineage in its own context. Its save is stamped "seen now" so the
  // reload does not run the absence it would otherwise see.
  async function copy(old) {
    await old?.pctx.close().catch(() => {});
    const pctx = await browser.newContext({ ...ctxOpts, storageState: storage });
    await pctx.addInitScript(() => {
      try {
        for (let i = 0; i < localStorage.length; i++) {
          const k = localStorage.key(i), v = localStorage.getItem(k);
          if (!v || !v.includes('"last_seen"')) continue;
          const o = JSON.parse(v);
          if (typeof o.last_seen === "number") { o.last_seen = Date.now(); localStorage.setItem(k, JSON.stringify(o)); }
        }
      } catch { /* not a save */ }
    });
    const p = await pctx.newPage();
    await p.goto(url, { waitUntil: "load" });
    // Past the camp's second pass: the forecast refines once the set has been still for 2 s, and until it
    // lands its first paint's `±` trails `…` (on the page).
    await refined(p);
    // Number the controls this screen shows (the same order every copy: the same save).
    const labels = await p.evaluate(() => [...document.querySelectorAll("button,[role=button]")]
      .filter((b) => (b.offsetWidth || b.offsetHeight) && !b.disabled && !b.closest(".sheet-wrap"))
      .map((b, i) => { b.dataset.probeI = String(i); return (b.innerText || b.getAttribute("aria-label") || "").trim().replace(/\s*\n\s*/g, " "); }));
    return { pctx, p, labels, text: await bodyText(p) };
  }
  // A screen that repaints its forecast paints it twice (the second pass lands a quiet 2 s + its sims later):
  // settle until the text has held still for `quiet` ms.
  const quiet = cmd.quiet ?? 4500;
  async function refined(p) {
    await steady(p);
    let last = await bodyText(p), since = Date.now();
    for (const t0 = Date.now(); Date.now() - t0 < 20000 && Date.now() - since < quiet;) {
      await p.waitForTimeout(250);
      const cur = await bodyText(p);
      if (cur !== last) { last = cur; since = Date.now(); }
    }
  }
  const found = [], changed = [], inert = [];
  let first = null;
  // `workers` copies side by side, each tapping every `workers`-th control; a copy the tap changed is rebuilt.
  const work = async (w) => {
    let c = await copy(), retried = -1;
    first ??= c;
    try {
      for (let i = w; i < Math.min(c.labels.length, max); i += workers) {
        const label = c.labels[i] || `#${i}`;
        const loc = c.p.locator(`[data-probe-i="${i}"]`);
        if (!(await loc.count())) { if (retried !== i) { retried = i; c = await copy(c); i -= workers; } continue; }
        const pre = await bodyText(c.p);
        await loc.first().click({ timeout: 3000 }).catch(() => {});
        await c.p.waitForTimeout(150);
        await steady(c.p, 8000);
        const sheet = await sheetText(c.p);
        if (sheet !== null) {
          found.push({ i, control: label, title: sheet.split("\n").map((x) => x.trim()).find(Boolean) ?? "", text: sheet });
          for (let k = 0; k < 4 && (await sheetText(c.p)) !== null; k++) { await c.p.keyboard.press("Escape"); await c.p.waitForTimeout(120); }
          await steady(c.p, 4000);
          if (digitless(await bodyText(c.p)) !== digitless(pre)) c = await copy(c);
          continue;
        }
        await refined(c.p);
        const post = await bodyText(c.p);
        // Numbers alone moving (the forecast's second pass landing, a clock) is not the tap's doing.
        if (digitless(post) === digitless(pre)) { inert.push({ i, control: label }); continue; }
        changed.push({ i, control: label, added: lineDiff(pre, post).slice(0, 40), ...(cmd.debug ? { gone: lineDiff(post, pre).slice(0, 40) } : {}) });
        c = await copy(c);
      }
    } finally {
      await c.pctx.close().catch(() => {});
    }
  };
  await Promise.all(Array.from({ length: workers }, (_, w) => work(w)));
  const byI = (a, b) => a.i - b.i;
  return { same_screen: first?.text === mainText, controls: first?.labels.length ?? 0, sheets: found.sort(byI), changed: changed.sort(byI), inert: inert.sort(byI).map((x) => x.control) };
}

async function run(cmd) {
  const { op } = cmd;
  const settle = async (ms) => page.waitForTimeout(ms ?? 600);
  switch (op) {
    case "goto": await page.goto(cmd.url, { waitUntil: "load" }); await settle(cmd.wait ?? 1500); return { ok: true, url: page.url() };
    case "text": return { text: await page.evaluate(() => document.body.innerText) };
    case "html": return { html: await page.evaluate(() => document.body.innerHTML) };
    case "shot": { const p = `${dir}/shots/${cmd.name ?? String(++shotN).padStart(3, "0")}.png`; await page.screenshot({ path: p, fullPage: !!cmd.full }); return { path: p }; }
    case "click": {
      const loc = cmd.selector ? page.locator(cmd.selector) : page.getByText(cmd.label, { exact: cmd.exact ?? true });
      const n = await loc.count();
      if (!n) return { error: `no element for ${cmd.label ?? cmd.selector}` };
      await loc.nth(cmd.nth ?? 0).click({ timeout: 5000, force: !!cmd.force });
      await settle(cmd.wait);
      return { ok: true, matched: n };
    }
    case "tap": await page.mouse.click(cmd.x, cmd.y); await settle(cmd.wait); return { ok: true };
    case "type": await page.keyboard.type(cmd.text, { delay: 30 }); return { ok: true };
    case "press": await page.keyboard.press(cmd.key); await settle(300); return { ok: true };
    case "fill": await page.locator(cmd.selector).fill(cmd.text); return { ok: true };
    case "eval": return { value: await page.evaluate(cmd.js) };
    case "js": return { value: await new AsyncFunction("page", "ctx", "browser", "dir", cmd.body)(page, ctx, browser, dir) };
    case "wait": await page.waitForTimeout(cmd.ms); return { ok: true };
    case "buttons": return { buttons: await page.evaluate(() => [...document.querySelectorAll("button,[role=button],a,input,select,textarea")].map((b) => ({ tag: b.tagName, text: (b.innerText || b.value || b.placeholder || "").trim().slice(0, 60), cls: b.className, dis: b.disabled ?? false, vis: !!(b.offsetWidth || b.offsetHeight) })).filter((b) => b.vis)) };
    case "state": return await state();
    case "act": {
      const loc = cmd.selector ? page.locator(cmd.selector) : page.getByText(cmd.label, { exact: cmd.exact ?? true });
      const n = await loc.count();
      if (!n) return { error: `no element for ${cmd.label ?? cmd.selector}` };
      await loc.nth(cmd.nth ?? 0).click({ timeout: 5000, force: !!cmd.force });
      const waited = await idle(cmd.timeout ?? 10000);
      const out = { ok: true, matched: n, waited, text: await page.evaluate(() => document.body.innerText) };
      if (cmd.shot) { out.shot = `${dir}/shots/${cmd.shot}.png`; await page.screenshot({ path: out.shot }); }
      return out;
    }
    case "watch": {
      const ms = cmd.ms ?? 30000, every = cmd.every ?? 1000, shots = cmd.shots ?? 0;
      const t0 = Date.now(); const samples = []; const paths = [];
      let last = new Set((await page.evaluate(() => document.body.innerText)).split("\n"));
      let nextShot = shots > 0 ? 0 : Infinity, shotK = 0;
      while (Date.now() - t0 < ms) {
        await page.waitForTimeout(every);
        const t = Date.now() - t0;
        const text = await page.evaluate(() => document.body.innerText);
        const lines = text.split("\n"); const cur = new Set(lines);
        const added = lines.filter((l) => l.trim() && !last.has(l));
        if (added.length) samples.push({ t: Math.round(t / 100) / 10, added });
        last = cur;
        if (t >= nextShot && shotK < shots) {
          const p = `${dir}/shots/${cmd.name ?? "watch"}-${String(++shotK).padStart(2, "0")}.png`;
          await page.screenshot({ path: p }); paths.push({ t: Math.round(t / 100) / 10, path: p });
          nextShot = shots > 1 ? (shotK * ms) / (shots - 1) : Infinity;
        }
      }
      return { ms, samples, shots: paths, end: await state() };
    }
    case "send_and_watch": return await sendAndWatch(cmd);
    case "sheets": return await sheets(cmd);
    case "log": return { log: log.splice(0) };
    case "quit": setTimeout(async () => { await browser.close(); process.exit(0); }, 200); return { ok: true };
    default: return { error: `unknown op ${op}` };
  }
}

http.createServer((req, res) => {
  let body = "";
  req.on("data", (c) => (body += c));
  req.on("end", async () => {
    let out;
    try { out = await run(JSON.parse(body || "{}")); } catch (e) { out = { error: String(e.message ?? e) }; }
    res.setHeader("content-type", "application/json");
    res.end(JSON.stringify(out));
  });
}).listen(port, "127.0.0.1", () => console.log(`driver ready on ${port} (${isHeaded ? "headed" : "headless"}, ${dpr}×) · ${dir}`));
