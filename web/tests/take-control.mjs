#!/usr/bin/env node
// Take control (owner 2026-10-08, a secondary mode), on the real wasm engine: in a live watched run the panel's toggle takes the
// hero; the world waits (engine tick still) until an action; each arrow key is one hero action (the picture never runs past the
// frontier); Release hands him back to the rules (the world moves again) and the panel reads off.
//   node web/tests/take-control.mjs
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = await launchBrowser();
try {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&fresh=1&seed=611`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
  check(await page.evaluate(() => window.__riddle.kind) === "wasm", "real wasm engine");
  const house = page.locator(".town-tag[data-next=house]");
  if (await house.count()) { await house.first().click(); await sleep(800); const b = page.locator("button", { hasText: /^build|Build/ }); if (await b.count()) await b.first().click(); await sleep(4000); }
  await page.locator(".camp-gem, .gem").first().click();
  await page.waitForFunction(() => window.__riddle.screen === "watch", null, { timeout: 30_000 });
  await sleep(1500);
  const st = () => page.evaluate(() => { const m = document.querySelector(".ctl"), w = document.querySelector("main.watch"); return { on: m?.dataset.on, awaiting: m?.dataset.awaiting, tick: Number(w?.dataset.tick), eng: Number(w?.dataset.engineTick ?? NaN) }; });
  check(await page.locator(".ctl-toggle").isVisible(), "the watch offers take control");
  await page.click(".ctl-toggle");
  await page.waitForFunction(() => document.querySelector(".ctl")?.dataset.awaiting === "1", null, { timeout: 20_000 });
  await sleep(500); const a = await st(); await sleep(2000); const b = await st();
  check(a.on === "1" && b.eng === a.eng, `the world waits for an order (engine ${a.eng} → ${b.eng})`);
  check(/Hand control/.test(await page.locator(".live-badge").textContent()), "the badge reads hand control");
  let moved = 0;
  for (const k of ["ArrowRight", "ArrowLeft", "ArrowDown", "ArrowUp", "."]) { const s0 = await st(); await page.keyboard.press(k); await sleep(900); const s1 = await st(); if (s1.eng > s0.eng) moved++; check(s1.tick <= s1.eng, `the picture stays at the frontier after ${k} (${s1.tick} ≤ ${s1.eng})`); }
  check(moved === 5, `each key is one action (${moved}/5 advanced the world)`);
  // blind b58b431 (A: arrows and `descend` inert in gas): every order resolves on the panel — done, or refused and why
  const note = (await page.locator(".ctl-note").textContent()) ?? "";
  check(/^(moved|waited|can't · .+|paralysed · \d+)$/.test(note), `the last order reads its outcome ("${note}")`);
  await page.click('.ctl-verb[data-verb="drink heal"]').catch(() => {});
  await page.click('.ctl-verb[data-verb="descend"]'); await sleep(900);
  const n2 = (await page.locator(".ctl-note").textContent()) ?? "";
  check(/^(descend ✓|can't · no stairs)$/.test(n2), `descend resolves visibly ("${n2}")`);
  // blind b58b431 (B: 40 identical attack taps): `attack until` is one order
  check(await page.locator('.ctl-verb[data-verb="attack until"]').isVisible(), "the panel offers attack until");
  await page.click('.ctl-verb[data-verb="attack until"]'); await sleep(900);
  const n3 = (await page.locator(".ctl-note").textContent()) ?? "";
  check(/^(attack until|foe down|hp under \d+%|can't · (no foe|hp low|no way)|paralysed · \d+)$/.test(n3), `attack until resolves visibly ("${n3}")`);
  await page.click(".ctl-toggle");
  await sleep(3000);
  const c = await st();
  check(c.on === "0", "released: the panel reads off");
  check(c.tick > b.tick + 40, `released: the rules play on (${b.tick} → ${c.tick})`);
  // blind 7f7fc2b (B: `Hand control · awaits order` with the run over — arrows and `descend` inert): in hand, `return` ordered and the
  // walk home driven a key a turn — the panel goes once the run is over or the exit holds, and the watch moves on
  await page.click(".ctl-toggle");
  await page.waitForFunction(() => document.querySelector(".ctl")?.dataset.awaiting === "1", null, { timeout: 20_000 });
  await page.click('.ctl-verb[data-verb="return"]');
  let ended = false, panelOff = false, stuckPanel = 0;
  for (let i = 0; i < 120 && !ended; i++) {
    const w = await page.evaluate(() => { const m = document.querySelector(".ctl"), w = document.querySelector("main.watch"); return { screen: window.__riddle.screen, over: w?.dataset.ending === "1" || !!w?.dataset.exit, hidden: !m || m.hidden || getComputedStyle(m).display === "none", awaiting: m?.dataset.awaiting }; });
    if (w.screen !== "watch") { ended = true; break; }
    if (w.over) { if (w.hidden) panelOff = true; else stuckPanel++; }
    // the walk home: `return` again each turn he awaits (as a player would) until the run is over
    if (!w.over && w.awaiting === "1") await page.click('.ctl-verb[data-verb="return"]').catch(() => {});
    await sleep(500);
  }
  check(ended, "in hand, the run that ended leaves the watch (no dead state)");
  check(panelOff || ended, "the panel is gone once the run is over");
  check(stuckPanel <= 1, `no take-control panel over a run that is over (${stuckPanel} reads)`);
} catch (e) { errors.push(String(e?.stack ?? e)); } finally { await browser.close(); }
console.log(out.join("\n")); if (errors.length) console.log(errors.join("\n"));
console.log(failed || errors.length ? `take-control: FAIL (${failed} assertion(s), ${errors.length} error(s))` : `take-control: ok (${out.length} checks)`);
process.exit(failed || errors.length ? 1 : 0);
