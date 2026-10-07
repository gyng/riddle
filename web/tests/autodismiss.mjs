#!/usr/bin/env node
// Auto-continue gates (docs/UI.md §7), on the fake engine, headless at 400 × 800. The clocks run 4× fast (`?autodismiss=0.25`:
// the report and the death 3 s, a panel 5 s); without the param automation keeps them off (the older gates' meaning).
//   report    the report continues to the camp after its time, its gem ringed; nothing bought, nothing equipped
//   input     a key restarts the clock; a pointer resting on the gem holds it
//   hidden    a hidden tab does not advance it; the return starts it over full
//   toggle    `auto continue` off in settings: the report waits
//   death     Town until the Scout; afterward `send` (lever wait) → watch; `forge` / `wear` levers and a fix's apply never pressed
//             (the camp tile instead: no forge sheet, nothing worn, the rules unchanged); a death opened from the report goes back to it
//   panels    a sheet and a camp panel close after 20 s (5 s here) of no input; the watch has no clock
//   off       no param under automation: no clock at all
//   node web/tests/autodismiss.mjs [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const SCALE = 0.25, REPORT = 12_000 * SCALE, DEATH = 12_000 * SCALE, PANEL = 20_000 * SCALE;

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(60); }
  throw new Error(`timeout waiting for ${label}`);
}
const screen = () => page.evaluate(() => window.__riddle?.screen);
const live = () => page.evaluate(() => window.__autodismiss?.live() ?? []);
const withState = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const save = JSON.parse(await r.engine.save());
  new Function("e", src)(save);
  r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
}, `(${fn})(e)`);
/** What a timeout must never change: gold, marks, unlocks, the worn packages, the rules. */
const ledger = () => page.evaluate(async () => { const r = window.__riddle, L = await r.engine.lineage(); return JSON.stringify({ gold: L.gold, marks: L.marks, unlocks: L.unlocks, pk: L.packages ? { s: L.packages.stance, t: L.packages.tactics, m: L.packages.temperament } : null, rows: L.sets?.[L.active_set ?? 0]?.rows }); });
/** Boot to the absence's report (8 h away). */
async function toReport(q = `&autodismiss=${SCALE}`, seed = 4101) {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}&absent=8h${q}`, { waitUntil: "domcontentloaded" });
  await until(() => window.__riddle?.screen === "report" && document.querySelector(".report-sheet"), "the report", 60_000);
  return Date.now();
}
const boot = async (seed, q = `&autodismiss=${SCALE}`) => {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}${q}`, { waitUntil: "domcontentloaded" });
  await until(() => window.__riddle?.booted && window.__riddle.screen === "camp", "camp");
};
/** The last absence's report again (a fresh clock). */
const reReport = () => page.evaluate(() => { const r = window.__riddle; r.go({ kind: "report", report: r.lastAbsence.report, absence: true }); });

try {
  if (part("report")) {
    const t0 = await toReport();
    const before = await ledger();
    const L = await live();
    const ring = await page.evaluate(() => !!document.querySelector(".report .console .gem .ad-ring"));
    check(L.length === 1 && /gem/.test(L[0].cls) && ring, `the report's gem carries the ring (${L.map((x) => `${x.cls.split(" ")[0]} ${x.ms} ms`).join(", ")})`);
    await until(() => window.__riddle.screen === "camp", "the camp after the report's time", REPORT + 6000);
    const dt = Date.now() - t0;
    check(dt >= REPORT - 300 && dt <= REPORT + 4000, `the report continued to the camp after its time (${dt} ms, bar ${REPORT})`);
    check((await ledger()) === before, "nothing bought, spent or worn by the timeout");
    check((await live()).length === 0, "the camp itself has no clock");
  }

  if (part("input")) {
    await toReport(); await reReport();
    // a key every half clock: the screen stays
    for (let i = 0; i < 6; i++) { await sleep(REPORT / 2); await page.keyboard.press("Shift"); }
    check((await screen()) === "report", `keys restart the clock (still the report after ${(3 * REPORT / 1000).toFixed(1)} s)`);
    // the pointer resting on the gem holds it
    const g = await page.locator(".report .console .gem").boundingBox();
    await page.mouse.move(g.x + g.width / 2, g.y + g.height / 2);
    await sleep(REPORT * 1.6);
    const held = await live();
    check((await screen()) === "report" && held[0]?.paused, `a pointer resting on the gem holds it (paused ${held[0]?.paused})`);
    await page.mouse.move(5, 300);
    const t = Date.now();
    await until(() => window.__riddle.screen === "camp", "the camp once the pointer left", REPORT + 6000);
    check(Date.now() - t >= REPORT - 300, `the clock ran again from full once the pointer left (${Date.now() - t} ms)`);
  }

  if (part("hidden")) {
    await toReport(); await reReport();
    await page.evaluate(() => { Object.defineProperty(document, "hidden", { configurable: true, get: () => true }); document.dispatchEvent(new Event("visibilitychange")); });
    await sleep(REPORT * 1.8);
    check((await screen()) === "report", "a hidden tab does not advance the clock");
    await sleep(REPORT * 0.5);
    await page.evaluate(() => { Object.defineProperty(document, "hidden", { configurable: true, get: () => false }); document.dispatchEvent(new Event("visibilitychange")); });
    const back = await live();
    check(back[0] && back[0].left >= back[0].ms - 400, `the return starts it over full (${back[0]?.left} of ${back[0]?.ms} ms)`);
    await until(() => window.__riddle.screen === "camp", "the camp after the return", REPORT + 6000);
  }

  if (part("toggle")) {
    await boot(4104);
    await page.locator("button.gear").click();
    await until(() => document.querySelector(".settings .auto-continue"), "the settings toggle");
    const was = await page.evaluate(() => document.querySelector(".settings .auto-continue").textContent);
    await page.locator(".settings .auto-continue").click();
    const now = await page.evaluate(() => ({ t: document.querySelector(".settings .auto-continue").textContent, ls: localStorage.getItem("riddle.autoContinue") }));
    check(was === "on" && now.t === "off" && now.ls === "0", `\`auto continue\` defaults on and turns off (${was} → ${now.t}, stored ${now.ls})`);
    await page.keyboard.press("Escape");
    await page.evaluate(() => { const r = window.__riddle; r.go({ kind: "report", report: { runs: 1, banked: 1, returned: 0, deaths: [], learned: [], bests: [], found: [], pending: [], reel: [], gold: 0, marks: 0 } }); });
    await sleep(REPORT * 2);
    const ring = await page.evaluate(() => getComputedStyle(document.querySelector(".report .gem .ad-ring") ?? document.body).display);
    check((await screen()) === "report" && ring === "none", `off: the report waits, no ring (${await screen()}, ring ${ring})`);
    await page.evaluate(() => localStorage.setItem("riddle.autoContinue", "1"));
  }

  if (part("death")) {
    await boot(4105);
    await withState((e) => { e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 4, cause: "goblin" }]; e.lineage.best_depth = 6; e.st305.hired = e.st305.hired.filter(id => id !== "scout"); });
    const base = { run_id: 7, depth: 6, cause: "goblin_archer", margin: "3 over", verdict: "gap", baseline: 0.42, replays: 12, trace: { turns: [] },
      patches: [{ row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "retreat" } }, insert_at: 0, survive: 0.9, forecast_delta: 0.2, forecast_depth: 6 }], morgue: "slain" };
    const go = (d) => page.evaluate((x) => window.__riddle.go({ kind: "death", death: x }), d);
    // Before the Scout, no clock is allowed to manufacture a manual send.
    const manualBefore = await ledger();
    await page.evaluate(() => { const e = window.__riddle.engine, send = e.send.bind(e); window.__sendCalls = 0; e.send = async (...args) => { window.__sendCalls++; return send(...args); }; });
    check(await page.evaluate(() => window.__riddle.lineage.tree.auto_send === false), "manual-send fixture has no hired Scout");
    await go({ ...base, lever: { kind: "wait", text: "Steady L2" }, package: "Steady · HP<20% → return" });
    const manual = await until(() => window.__riddle.screen === "death" && window.__autodismiss.live()[0], "the manual death's clock");
    check(/tile/.test(manual.cls) && manual.text === "town", "before the Scout, the death clock returns to Town");
    await until(() => window.__riddle.screen === "camp", "Town after the manual death's time", DEATH + 6000);
    check(await page.evaluate(() => window.__sendCalls === 0) && await ledger() === manualBefore, "manual death timeout sends nothing and changes no gold, choices or rules");
    await withState((e) => { e.st305.hired.push("scout"); });
    check(await page.evaluate(() => window.__riddle.lineage.tree.auto_send === true), "automatic-send fixture has a hired Scout");
    // After the Scout, the lever `wait` gem reads `send` and is pressed.
    await go({ ...base, lever: { kind: "wait", text: "Steady L2" }, package: "Steady · HP<20% → return" });
    const w = await until(() => window.__riddle.screen === "death" && window.__autodismiss.live()[0], "the death's clock");
    check(/gem/.test(w.cls) && /send/.test(w.text), `before the pen, lever \`wait\`: the ring on the \`send\` gem (${w.text})`);
    const t = Date.now();
    await until(() => window.__riddle.screen === "watch", "the watch after the death's time", DEATH + 6000);
    check(Date.now() - t >= DEATH - 400, `the death sent again after its time (${Date.now() - t} ms)`);
    check((await live()).length === 0, "the watch has no clock");
    await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
    // `forge` and `wear` are decisions: the camp tile carries the ring; no forge sheet, nothing worn
    for (const lever of [{ kind: "spend", text: "sword +2" }, { kind: "package", text: "Steady" }]) {
      const before = await ledger();
      await go({ ...base, lever });
      const c = await until(() => window.__riddle.screen === "death" && window.__autodismiss.live()[0], "the death's clock");
      check(/tile/.test(c.cls) && c.text === "town", `lever \`${lever.kind}\`: the ring on the Town tile, not the gem (${c.cls.split(" ")[0]} ${c.text})`);
      await until(() => window.__riddle.screen === "camp", "the camp after the death's time", DEATH + 6000);
      const forge = await page.evaluate(() => !!document.querySelector(".sheet .forge, .forge-sheet"));
      check(!forge && (await ledger()) === before, `lever \`${lever.kind}\`: to the camp, nothing opened, bought or worn`);
    }
    // the pen open: a fix's `apply` is never pressed
    await withState((e) => { e.lineage.best_depth = 14; });
    const before = await ledger();
    await go(base);
    const p = await until(() => window.__riddle.screen === "death" && document.querySelector(".death button.patch") && window.__autodismiss.live()[0], "the pen's death");
    check(!/gem/.test(p.cls), `with the pen: the ring is not on the fix's gem (${p.cls.split(" ")[0]} ${p.text})`);
    await until(() => window.__riddle.screen !== "death", "the pen's death continued", DEATH + 6000);
    check((await ledger()) === before, `no fix applied by the timeout (now ${await screen()})`);
    // a death opened from the report goes back to it after a panel's time
    await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d, from: { report: { runs: 1, banked: 0, returned: 0, deaths: [], learned: [], bests: [], found: [], pending: [], reel: [], gold: 0, marks: 0 } } }), base);
    const o = await until(() => window.__riddle.screen === "death" && window.__autodismiss.live()[0], "the opened death's clock");
    check(/report/.test(o.text) && o.ms === PANEL, `a death opened from the report: back to it after ${o.ms} ms (${o.text})`);
    await until(() => window.__riddle.screen === "report", "back to the report", PANEL + 6000);
    check(true, "the opened death went back to the report");
  }

  if (part("panels")) {
    await boot(4106);
    // A fresh town hides its forecast; this panel fixture represents an earned camp.
    await withState((e) => { e.lineage.best_depth = 6; });
    // a sheet (the purse's ledger)
    await page.locator(".topbar .stat.gold").click();
    await until(() => document.querySelector(".sheet-wrap .ad-ring"), "the sheet's ring");
    const t = Date.now();
    await until(() => !document.querySelector(".sheet-wrap"), "the sheet closed", PANEL + 6000);
    check(Date.now() - t >= PANEL - 500, `a sheet closes after its time of no input (${Date.now() - t} ms, bar ${PANEL})`);
    // a camp panel (the forecast, from the shaft)
    await page.locator(".camp .shaft:visible").click(); await page.mouse.move(5, 5);
    await until(() => document.querySelector(".panel-host.open .panel .close-stud .ad-ring"), "the panel's ring");
    const t2 = Date.now();
    await until(() => !document.querySelector(".panel-host.open"), "the panel closed", PANEL + 6000);
    check(Date.now() - t2 >= PANEL - 500, `a camp panel closes after its time of no input (${Date.now() - t2} ms)`);
  }

  if (part("off")) {
    await toReport("", 4107);
    await sleep(REPORT * 2);
    const st = await page.evaluate(() => ({ s: window.__riddle.screen, on: window.__autodismiss.enabled() }));
    check(st.s === "report" && !st.on, `under automation without the param: off (${st.s}, enabled ${st.on})`);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(e);
const n = out.filter((l) => /^(ok|FAIL)/.test(l)).length;
console.log(errors.length || failed ? `autodismiss: ${failed} of ${n} failed${errors.length ? `, ${errors.length} errors` : ""}` : `autodismiss: ${n}/${n} ok`);
process.exit(errors.length || failed ? 1 : 0);
