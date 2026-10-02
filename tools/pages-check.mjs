#!/usr/bin/env node
// Production smoke check; serve a built site first (with the same RIDDLE_BASE as the build).
// node tools/pages-check.mjs http://localhost:5359/riddle/ [screenshot-dir]
import assert from "node:assert/strict";
import { mkdir } from "node:fs/promises";
import { launchBrowser } from "./browser.mjs";
const target = new URL(process.argv[2] ?? "http://localhost:5359/riddle/");
const out = process.argv[3];
if (out) await mkdir(out, { recursive: true });
const browser = await launchBrowser();
try {
  const context = await browser.newContext({ viewport: { width: 400, height: 800 } });
  const page = await context.newPage(), errors = [];
  page.on("pageerror", (e) => errors.push(e.message));
  page.on("response", (r) => { if (r.status() >= 400) errors.push(`${r.status()} ${r.url()}`); });
  page.on("request", (r) => { if (new URL(r.url()).origin === target.origin && !r.url().startsWith(target.href)) errors.push(`outside base: ${r.url()}`); });
  await page.goto(target.href);
  await page.waitForSelector(".build-banner");
  await page.evaluate(() => navigator.serviceWorker.ready);
  assert.match(await page.locator(".build-banner").innerText(), /alpha\s*build \d{4}-\d{2}-\d{2}/i);
  assert.equal(await page.evaluate(() => navigator.serviceWorker.controller?.scriptURL), new URL("sw.js", target).href);
  for (const width of [400, 1440]) {
    await page.setViewportSize({ width, height: width === 400 ? 800 : 900 });
    assert.equal(await page.evaluate(() => document.documentElement.scrollWidth), width, `overflow at ${width}px`);
    assert.deepEqual(await page.evaluate(() => [...document.images].filter((i) => i.complete && !i.naturalWidth).map((i) => i.src)), []);
    if (out) await page.screenshot({ path: `${out}/${width}.png` });
  }
  await context.setOffline(true);
  await page.reload();
  await page.waitForSelector(".build-banner");
  assert.deepEqual(errors, []);
  console.log("pages: scoped assets, banner, mobile/desktop layout, offline reload PASS");
} finally { await browser.close(); }
