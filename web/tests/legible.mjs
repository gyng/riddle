#!/usr/bin/env node
// c30-legible gates (the owner, a new player: "I didn't understand why there were new buildings or why the run ended early at like D3"),
// headless at 400 × 800:
//   ends    real engine, a fresh lineage, three watched runs (▶▶| as a player would): every run's end shows a reason beat under its sum
//           (the core's `ExitLine.reason`, ≤ 3 words — the first run's `banks every record`), and the report's end tile carries it
//   build   fake engine, each trigger in turn: every building's arrival names its cause (`first gold home → blacksmith`, ≤ 4 words + the
//           arrow) and draws the eye (its target glows); the staked plot's tag is always visible — the next building and its trigger
//           (`storehouse · first find kept`), from day 0 until the v1 set stands
//
//   node web/tests/legible.mjs [--shots dir] [--part=ends,build]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const words = (s) => s.split(/\s+/).filter((w) => w && w !== "·" && w !== "→").length;

const browser = await launchBrowser();
let page;
async function open() {
  page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
  page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
}
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`) }); };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp" && window.__town, "camp", 60_000); await sleep(300); };
const screen = () => page.evaluate(() => window.__riddle?.screen);

try {
  await open();
  // ---- the run's end: a reason beat, and the report carries it
  if (part("ends")) {
    await page.goto(`${url}?dev=1&fresh=1&seed=5101&speed=8`, { waitUntil: "domcontentloaded" });
    await camp();
    for (let k = 1; k <= 3; k++) {
      await page.evaluate(() => { window.__beatLog = []; window.__whyLog = []; });
      await page.locator("button.send").first().click();
      await until(() => window.__riddle.screen === "watch", "the watch");
      let whyShown = null;
      const t0 = Date.now();
      for (;;) {
        const s = await screen();
        if (s === "exit") { await page.locator(".sheet-wrap .vault-choice .chip").first().click().catch(() => {}); await page.locator(".sheet-wrap .btn.primary").first().click().catch(() => {}); await sleep(200); continue; }
        if (s !== "watch") break;
        whyShown ??= await page.evaluate(() => document.querySelector(".watch .beat-why.show")?.textContent ?? null);
        if (!whyShown) await page.evaluate(() => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent?.includes("▶▶|")) { b.click(); break; } });
        else if (k === 1) await shot("end-beat");
        if (Date.now() - t0 > 180_000) throw new Error(`run ${k} still going`);
        await sleep(60);
      }
      const log = await page.evaluate(() => (window.__beatLog ?? []).filter((b) => b.why));
      const wl = await page.evaluate(() => window.__whyLog ?? []);
      const why = whyShown ?? wl[wl.length - 1] ?? "";
      check(!!why && words(why) <= 3, `run ${k}: its end shows a reason beat ≤ 3 words ("${log[log.length - 1]?.text ?? "death"}" · "${why}"${whyShown ? "" : ", logged, not seen"})`);
      if (k === 1) check(why === "banks every record", `run 1: a fresh lineage's first end reads as the hero being sensible ("${why}")`);
      const s = await until(() => ["report", "death"].includes(window.__riddle.screen) && window.__riddle.screen, "the screen after", 30_000);
      if (s === "report") {
        const tw = await until(() => document.querySelector(".report .tile-why")?.textContent ?? null, "the report's reason", 8000).catch(() => null);
        check(tw === why, `run ${k}: the report's end tile carries the same reason ("${tw}")`);
        if (k === 1) await shot("report");
        await page.locator("main.report .gem").first().click();
      } else {
        check(/^slain|^starved/.test(why), `run ${k}: a death's beat names the killer ("${why}")`);
        await page.locator(".cmd button").filter({ hasText: /^camp$/ }).first().click();
      }
      await camp();
    }
  }

  // ---- building arrivals and the staked plot's tag (the fake's triggers, one stage at a time)
  if (part("build")) {
    await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3111`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.booted && ["camp", "report"].includes(window.__riddle.screen), "boot", 60_000);
    await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); for (const k of Object.keys(localStorage)) if (k.startsWith("riddle.town.")) localStorage.removeItem(k); });
    const STAGES = [
      (L) => Object.assign(L, { heir: 1, chronicle: [], marks: 0, gold: 0, gold_ledger: [], vault: [], party: [], kennel: [], best_depth: 0 }),
      (L) => Object.assign(L, { gold: 60, best_depth: 3 }),
      (L) => { L.vault = [{ id: 9001, kind: "sword", known: true, label: "sword" }]; },
      (L) => { L.kennel = [{ id: 77, kind: "jackal", name: "Ash", level: 1, tags: [], gen: 0, rules: { rows: [] }, max_rows: 2, hp: 5, max_hp: 5 }]; },
      (L) => { L.gold = 900; },
    ];
    const order = ["blacksmith", "storehouse", "kennel", "bank"];
    const toStage = (n) => page.evaluate(async ([n, src]) => {
      const r = window.__riddle; const save = JSON.parse(await r.engine.save());
      for (let i = 0; i <= n; i++) new Function("L", `return (${src[i]})(L)`)(save.lineage);
      r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
    }, [n, STAGES.map(String)]);
    for (let n = 0; n <= 4; n++) {
      await toStage(n); await camp();
      const st = await page.evaluate(() => {
        const tag = document.querySelector(".town-tag"), a = document.querySelector(".town-arrival");
        const r = tag?.getBoundingClientRect();
        return { tag: tag && !tag.hidden && r.width > 0 && r.top >= 0 && r.bottom <= innerHeight ? tag.textContent : null, next: window.__riddle.lineage.town?.next,
          arrived: a?.classList.contains("show") ? a.dataset.why : null, glow: [...document.querySelectorAll(".town-hit.arrived")].map((b) => b.dataset.building), opacity: a ? getComputedStyle(a).opacity : "0" };
      });
      const next = order[n];
      if (next) check(!!st.tag && st.tag.startsWith(next) && st.tag.includes("·") && words(st.tag) <= 5,
        `stage ${n}: the staked plot's tag is visible without a tap — what it becomes and its trigger ("${st.tag}")`);
      else check(!st.tag, `stage ${n}: no tag once the v1 set stands (${st.tag ?? "none"})`);
      if (n >= 1) {
        const b = order[n - 1];
        const why = st.arrived ?? "";
        check(why.endsWith(`→ ${b}`) && words(why.split("→")[0]) <= 4 && words(why.split("→")[0]) >= 1,
          `stage ${n}: the ${b}'s arrival names its cause ("${why}")`);
        check(st.glow.includes(b), `stage ${n}: the eye is drawn to the new ${b} (glowing: ${st.glow.join(",") || "none"})`);
      }
      if (n === 1) await shot("arrival");
    }
  }
} catch (e) {
  check(false, `threw: ${e.message}`);
}
const errs = errors.filter((e) => !/favicon|404|net::ERR/.test(e));
check(errs.length === 0, `no page errors (${errs.slice(0, 3).join(" | ")})`);
await browser.close();
console.log(out.join("\n"));
console.log(failed ? `legible: ${failed} failed` : "legible: all passed");
process.exit(failed ? 1 : 0);
