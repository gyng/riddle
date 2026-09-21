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
