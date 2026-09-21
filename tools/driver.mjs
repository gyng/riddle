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
//   ops: goto text html shot click tap type press fill eval js wait buttons log quit
//   compound ops (docs/ITERATION_SPEED.md §2 — one turn where it took four to fifteen):
//   tools/drive.sh 5347 '{"op":"state"}'                      screen (dev builds), busy, text, buttons in one call
//   tools/drive.sh 5347 '{"op":"act","label":"send","shot":"after-send"}'   click, settle until the engine is idle (not a fixed 600 ms), return the new text (+ a shot)
//   tools/drive.sh 5347 '{"op":"watch","ms":30000,"every":1000,"shots":4}'  sample the screen every N ms for M ms: the lines that appeared per sample + evenly spaced shots
//
// Console errors/warnings and page errors are kept in <dir>/console.log and returned by "log".
import http from "node:http";
import fs from "node:fs";
import { resolve } from "node:path";
import { launchBrowser } from "./browser.mjs";

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
const ctx = await browser.newContext(flag("--wide") ? { viewport: { width: 1280, height: 800 }, deviceScaleFactor: 1 } : { viewport: { width: 400, height: 800 }, deviceScaleFactor: dpr });
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
