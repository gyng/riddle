#!/usr/bin/env node
// Blind 1fb7786 (A, twice: "REPORT gem unresponsive … only a reload fixed it", shots/stuck-report.png): the exit's keep sheet closed
// by anything but its `keep` button — a tap on the gem under its backdrop, Escape, the close stud — left the exit flow waiting on a
// `keep` that never came: the pump stopped, the gem's tap only shortened a walk-out already over. Every close of the keep sheet keeps
// the ticked picks and the flow goes on to the next screen.
//   node web/tests/exit-keep-close.mjs
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
const rules = encodeURIComponent("depth>=3 → return\nfoes>=1 → attack nearest");   // qa9's run that comes home with items
try {
  for (const how of (process.env.HOW ?? "gem,escape,stud").split(",")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
    page.on("pageerror", (e) => errors.push(`${how} pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=13&autosend=1&speed=fast&rules=${rules}`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "watch", null, { timeout: 30_000 });
    await page.evaluate(() => {
      const r = window.__riddle; r.__keep = []; r.__pick = null;
      const k = r.engine.keep.bind(r.engine), st = r.engine.step.bind(r.engine);
      r.engine.keep = async (ids) => { r.__keep.push(ids); return k(ids); };
      // a free slot and the automations' pick ticked: the sheet is a decision
      r.engine.step = async (n) => { const res = await st(n); if (res.exit_pending?.items?.length) { res.exit_pending.auto_keep = [res.exit_pending.items[0].id]; res.exit_pending.decide = true; r.__pick = res.exit_pending.items[0].id; } return res; };
    });
    const t = Date.now(); let open = false;
    while (Date.now() - t < 60_000) {
      const s = await page.evaluate(() => ({ screen: window.__riddle.screen, keep: !!document.querySelector(".sheet-wrap .keep-legend") }));
      if (s.keep) { open = true; break; }
      if (s.screen !== "watch") break;
      await page.evaluate(() => { for (const b of document.querySelectorAll(".sheet-wrap .vault-choice .chip")) { b.click(); break; } for (const b of document.querySelectorAll(".cmd .tile[data-tile=skip]")) b.click(); });
      await sleep(250);
    }
    check(open, `${how}: a return with loot opens the keep sheet`);
    if (!open) { await page.close(); continue; }
    if (how === "gem") {
      // the player's tap on the REPORT gem (it stands above the sheet's backdrop): it goes on
      const box = await page.locator(".gem-slot .gem").boundingBox();
      await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
    } else if (how === "escape") await page.keyboard.press("Escape");
    else await page.locator(".sheet-wrap .close-stud, .sheet-wrap .sheet-x").first().click();
    const left = await page.waitForFunction(() => !["watch", "exit"].includes(window.__riddle.screen), null, { timeout: 15_000 }).then(() => true, () => false);
    const k = await page.evaluate(() => ({ screen: window.__riddle.screen, keep: window.__riddle.__keep, pick: window.__riddle.__pick }));
    check(left, `${how}: closing the keep sheet goes on to the next screen (${k.screen})`);
    check(k.keep.length === 1 && k.keep[0].length === 1 && k.keep[0][0] === k.pick, `${how}: the close keeps the ticked pick once (${JSON.stringify(k.keep)}, pick ${k.pick})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`exit-keep-close: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`exit-keep-close: ok (${out.length} checks)`);
