#!/usr/bin/env node
// Blind 5331f40 (both Mirror King kills ended off-view: `Run ended · Watching D33`, then the report; the kill was a 5-line reel): a boss's
// kill is a witnessed beat. Real wasm, the earned first-clear fixture, a numbered descent re-climbed to the King:
//   live — the watch is paused at the King's sighting while the world runs the run to its end; the hero tap (`riddle:focus-hero`,
//          a jump to live) then lands short of the kill, `MIRROR KING DOWN` holds its beat before `COLLECTED`, and only then the report;
//   away — a boss slain while away offers `watch kill` on the absence's report; it opens that run's replay seeked to the fall, which plays.
//   node web/tests/boss-kill-beat.mjs
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const source = readFileSync(new URL("./fixtures/earned-first-ascension-clear.json", import.meta.url), "utf8"), native = JSON.parse(source);
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = await launchBrowser();
async function fresh(page) {
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&fresh=1&runs=0`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
  check(await page.evaluate(() => window.__riddle.kind) === "wasm", "real wasm engine");
  check(await page.evaluate(async ({ source, loadout }) => window.__riddle.importSave(JSON.stringify({ v: 2, engine: source, loadout, last_seen: Date.now(), runs: 359 })), { source, loadout: native.loadout }), "the earned clear imports");
  await page.evaluate(async () => { const a = window.__riddle; a.runnerOn = false; await a.beginDescent(0); a.go({ kind: "camp" }); });
}
try {
  // ---- live: the picture reaches the kill and holds it before the exit flow
  {
    const page = await browser.newPage({ viewport: { width: 400, height: 860 } });
    await fresh(page);
    // the engine sends the re-climb (waystones lit as it goes) until a run stands on D33, the King's floor, not yet met
    const at = await page.evaluate(async () => {
      const a = window.__riddle; let s = null;
      for (let run = 0; run < 12; run++) {
        try { await a.engine.autoKeep?.(); } catch { /* nothing pending */ }
        const L = await a.engine.lineage();
        for (const d of [29, 28, 24, 23, 19, 18]) if (d <= L.best_depth) { try { await a.engine.setStart(d); break; } catch { /* unlit */ } }
        s = await a.engine.send(); let over = false;
        for (let i = 0; i < 6000; i++) { const x = await a.engine.step(50); s = x.snapshot; if (x.run_over) { over = true; break; } if (s.depth >= 33) break; }
        if (!over && s.depth >= 33) return s.depth;
      }
      return s?.depth ?? 0;
    });
    check(at === 33, `a live run stands on the King's floor (D${at})`);
    await page.evaluate(() => { window.__beatLog = []; const a = window.__riddle; a.watchMode = "fights"; a.go({ kind: "watch" }); });
    await page.waitForFunction(() => window.__riddle.screen === "watch", null, { timeout: 30_000 });
    // the King in view: pause, and let the world run the run out (the exit held: `Run ended`)
    const sighted = await page.waitForFunction(() => !!document.querySelector("main.watch")?.dataset.boss || (window.__beatLog ?? []).some((b) => /MIRROR KING DOWN/.test(b.text)), null, { timeout: 120_000, polling: 50 }).then(() => true, () => false);
    check(sighted, "the watch reaches the King");
    const downEarly = await page.evaluate(() => (window.__beatLog ?? []).some((b) => /MIRROR KING DOWN/.test(b.text)));
    if (!downEarly) {
      await page.locator("main.watch [aria-label='Pause watch']").first().click().catch(() => {});
      const ended = await page.waitForFunction(() => document.querySelector("main.watch")?.dataset.ending === "1", null, { timeout: 120_000 }).then(() => true, () => false);
      check(ended, "paused, the world ran the run to its end");
      // the hero tap: a jump to live — the kill not yet shown, the picture lands short of it
      await page.evaluate(() => window.dispatchEvent(new Event("riddle:focus-hero")));
      await page.locator("main.watch [aria-label='Resume watch']").first().click().catch(() => {});
    }
    await page.waitForFunction(() => window.__riddle.screen !== "watch", null, { timeout: 120_000 }).catch(() => {});
    const log = await page.evaluate(() => (window.__beatLog ?? []).map((b) => ({ text: b.text, ms: b.ms })));
    const down = log.findIndex((b) => /MIRROR KING DOWN/.test(b.text)), exit = log.findIndex((b) => /^(COLLECTED|RETURNED)/.test(b.text));
    check(down >= 0, `MIRROR KING DOWN is a beat (${log.map((b) => b.text).join(" | ")})`);
    check(down >= 0 && (exit < 0 || down < exit), "the kill's beat comes before the exit's");
    check(down >= 0 && exit >= 0 && log[exit].ms - log[down].ms >= 3500, `the kill holds its beat before the exit (${down >= 0 && exit >= 0 ? log[exit].ms - log[down].ms : "?"} ms)`);
    check(await page.evaluate(() => window.__riddle.screen) === "report", "then the report");
    await page.close();
  }
  // ---- away: the report's `watch kill` plays the run that slew him, seeked to the fall
  {
    const page = await browser.newPage({ viewport: { width: 400, height: 860 } });
    await fresh(page);
    await page.evaluate(() => window.__riddle.absence(4 * 3600));
    await page.waitForFunction(() => window.__riddle.screen === "report", null, { timeout: 180_000 });
    // a descent's absence re-slays the King but reports no first victory (`boss:` bests are the lineage's firsts): the report is shown
    // again as a first clear's would read — the runs, their held replays and the King's fall in them are the engine's own
    const held = await page.evaluate(() => { const a = window.__riddle, r = a.view.report; a.go({ kind: "report", report: { ...r, bests: [...r.bests, "boss:mirror_king"] }, absence: true }); return (a.lineage.replays ?? []).length; });
    check(held > 0, `the absence's runs are held for replay (${held})`);
    const btn = page.locator(".report-boss-watch").first();
    check(await btn.count() > 0, `a boss slain while away offers watch kill (${await page.locator(".report-boss-row").count()} bosses)`);
    if (await btn.count()) {
      const boss = await btn.getAttribute("data-boss");
      check(/watch kill/.test(await btn.textContent()), "the button reads watch kill");
      await btn.scrollIntoViewIfNeeded(); await btn.click();
      const opened = await page.waitForSelector(".run-replay[data-kill]", { timeout: 60_000 }).then(() => true, () => false);
      check(opened, `the kill's replay opens (${boss})`);
      await page.waitForFunction(() => (window.__runReplay?.floor ?? -1) >= 0, null, { timeout: 30_000 }).catch(() => {});
      const r = await page.evaluate(() => window.__runReplay);
      check(!!r?.kill && r.floor === r.kill.floor, `seeked to the kill's floor (floor ${r?.floor}, kill ${JSON.stringify(r?.kill)})`);
      const seen = await page.waitForFunction(() => window.__runReplay?.kill?.seen === true, null, { timeout: 30_000 }).then(() => true, () => false);
      check(seen, "the fall plays within seconds of the open");
      check(/DOWN/.test(await page.locator(".run-replay .rr-at").textContent()), `the replay names the kill (${await page.locator(".run-replay .rr-at").textContent()})`);
    }
    check(await page.evaluate(() => document.documentElement.scrollWidth <= innerWidth), "no horizontal scroll");
    await page.close();
  }
} catch (e) { errors.push(String(e?.stack ?? e)); } finally { await browser.close(); }
console.log(out.join("\n")); if (errors.length) console.log(errors.join("\n"));
console.log(failed || errors.length ? `boss-kill-beat: FAIL (${failed} assertion(s), ${errors.length} error(s))` : `boss-kill-beat: ok (${out.length} checks)`);
process.exit(failed || errors.length ? 1 : 0);
