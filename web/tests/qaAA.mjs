#!/usr/bin/env node
// QA 524827b (qaAA) client gates, on the fake engine, headless at 400 × 800:
//   · a patch judged on whole runs: once `deathDeltas` lands, each tablet reads its whole-run move beside the moment's count (`death −8
//     ±8`, `risk fire`); a tablet that harms whole runs (`PatchWhole.harms`) is never the lead nor the gem's (it goes after the rest,
//     the light takes the first that does not harm; every one harming, the gem reads `edit`)
//   · `tied` when the top two survive alike (not only all three)
//   · after a lineage move the `vs sent` arrows are the shown number less the sent set's shown one (`D6 26→17%`, never `▼27`); an
//     unnamed change clears the last move's line (`drop · D6 …` gone after an unlock buy)
//
//   node web/tests/qaAA.mjs            (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen})`);
}

const heal = { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" }, origin: "player" };
const hit = { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" };
const unknown = { conds: [{ k: "hp<", n: 20 }], verb: { v: "drink", a: "unknown" } };
const rest = { conds: [{ k: "hp<", n: 20 }], verb: { v: "rest" } };
const ret = { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } };
const w = (death, harms, risk) => ({ reach: 0.1, reach_pm: 0.1, death, death_pm: 0.05, harms, ...(risk ? { risk } : {}) });

/** A death screen with `patches` (camp_pending) whose `deathDeltas` answers `filled` after 300 ms. */
async function deathWith(patches, filled) {
  await page.evaluate(([patches, filled, rows]) => {
    const r = window.__riddle;
    r.sets[r.active] = { ...r.sets[r.active], rows };
    r.engine.deathDeltas = () => new Promise((res) => setTimeout(() => res(filled), 300));
    r.go({ kind: "death", death: { run_id: 0, depth: 5, cause: "monkey", margin: "", verdict: "gap", baseline: 7 / 12, replays: 12, trace: { turns: [] }, morgue: "t1", rules: { rows }, patches } });
  }, [patches, filled, [heal, hit]]);
  await waitFor((x) => x?.screen === "death", "the death screen"); await sleep(150);
}
const tablets = () => page.evaluate(() => [...document.querySelectorAll(".patches button.patch")].map((b) => ({
  row: b.querySelector(".patch-main")?.textContent.trim(), rank: b.querySelector(".rank")?.textContent, top: b.classList.contains("top"),
  harms: b.classList.contains("harms"), whole: b.querySelector(".whole")?.textContent.trim() ?? "" })));
const head = () => page.evaluate(() => document.querySelector(".patches-moment")?.textContent.trim() ?? "");
const gemText = () => page.evaluate(() => document.querySelector(".patch-gem, .gem")?.textContent.trim() ?? "");

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=26`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250);

  // ---- a lead that harms whole runs yields; the whole-run move reads beside the count
  {
    const ps = [{ row: unknown, insert_at: 1, survive: 1, forecast_delta: 0.4, camp_pending: true },
                { row: rest, insert_at: 1, survive: 1, forecast_delta: 0.2, camp_pending: true },
                { row: ret, insert_at: 0, survive: 1, forecast_delta: 0, camp_pending: true, exits: true }];
    const filled = [{ ...ps[0], camp_pending: false, forecast_depth: 6, forecast_pm: 0.14, whole: w(0.12, true, "fire") },
                    { ...ps[1], camp_pending: false, forecast_depth: 6, forecast_pm: 0.14, whole: w(-0.08, false) },
                    { ...ps[2], camp_pending: false, forecast_depth: 6, forecast_pm: 0.1, whole: w(-0.8, false) }];
    await deathWith(ps, filled);
    const before = await tablets();
    check(before[0]?.top && /drink/.test(before[0].row ?? ""), `before the landing the verdict's lead is lit (${before.map((t) => t.row).join(" | ")})`);
    check(/· patches tie$/.test(await head()), `three alike read \`patches tie\` ("${await head()}")`);
    await sleep(700);
    const after = await tablets();
    const lead = after[0], last = after[after.length - 1];
    check(/rest/.test(lead?.row ?? "") && lead.top && lead.rank === "1.", `the landing: the harming tablet leaves the lead; the first that does not harm is lit (${after.map((t) => `${t.rank} ${t.row}${t.top ? "*" : ""}`).join(" | ")})`);
    check(/drink/.test(last?.row ?? "") && last.harms && /^death \+12 ±5 · risk fire$/.test(last.whole), `the harming tablet goes last, reading its whole-run move ("${last?.whole}")`);
    check(/^death −8 ±5$/.test(lead.whole), `a tablet reads its death move beside the count ("${lead.whole}")`);
  }

  // ---- every patch harms: the gem reads `edit`
  {
    const ps = [{ row: unknown, insert_at: 1, survive: 1, forecast_delta: 0.4, camp_pending: true }, { row: rest, insert_at: 1, survive: 10 / 12, forecast_delta: 0.2, camp_pending: true }];
    const filled = ps.map((p) => ({ ...p, camp_pending: false, forecast_depth: 6, forecast_pm: 0.1, whole: w(0.2, true) }));
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await waitFor((s) => s?.screen === "camp", "camp");
    await deathWith(ps, filled);
    check(!/tie/.test(await head()), `12/12 over 10/12 is no tie ("${await head()}")`);
    await sleep(700);
    const t = await tablets();
    check(!t.some((x) => x.top) && /edit/i.test(await gemText()), `every tablet harms: none lit, the gem reads edit ("${await gemText()}")`);
  }

  // ---- the top two tie, the third does not: `tied`
  {
    const ps = [{ row: unknown, insert_at: 1, survive: 1, forecast_delta: 0.4, camp_pending: true }, { row: rest, insert_at: 1, survive: 10 / 12, forecast_delta: 0.2, camp_pending: true },
                { row: ret, insert_at: 0, survive: 1, forecast_delta: 0, camp_pending: true, exits: true }];
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await waitFor((s) => s?.screen === "camp", "camp");
    await deathWith(ps, ps.map((p) => ({ ...p, camp_pending: false, whole: w(0, false) })));
    check(/· patches tie$/.test(await head()), `#1 and #3 both 12/12 read \`patches tie\` ("${await head()}")`);
  }

  // ---- vs sent after a lineage move: the shown numbers; an unnamed change clears the move line
  {
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await waitFor((s) => s?.screen === "camp", "camp"); await sleep(300);
    const r = await page.evaluate(() => {
      const a = window.__riddle;
      a.baseShown.clear(); a.baseShown.set("D6", 26);
      a.lastForecast = { ...a.lastForecast, sims: 50, refined: false };
      a.vs = { depths: [{ depth: 6, delta: 0.17 - 0.44, pm: 0.08, base: 0.44, abs_pm: 0.1 }], sims: 50, refined: false };
      a.baseMoved = true;
      const moved = a.vsShown()?.depths[0];
      a.baseMoved = false;
      const paired = a.vsShown()?.depths[0];
      return { moved, paired };
    });
    const pts = (m) => Math.round((m.base + m.delta) * 100) - Math.round(m.base * 100);
    check(r.moved && Math.round(r.moved.base * 100) === 26 && pts(r.moved) === -9, `after a purchase the move is the shown 17 less the sent set's shown 26 (${r.moved && pts(r.moved)}), not the re-measured 44`);
    check(r.paired && pts(r.paired) === -27, `with no lineage move the paired move stands (${r.paired && pts(r.paired)})`);
    const lm = await page.evaluate(async () => {
      const a = window.__riddle;
      a.lmove = { label: "drop", rules: JSON.stringify(a.rules.rows), depths: [{ depth: 6, delta: 0.02, pm: 0.06 }] };
      await a.mutate(() => a.engine.lineage());
      return a.lmove;
    });
    check(lm === null, "an unnamed lineage change clears the last move's line");
  }

} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  await browser.close();
}
for (const l of out) console.log(l);
for (const e of errors) console.log(`FAIL ${e}`);
const n = out.length;
console.log(`${failed || errors.length ? "FAIL" : "ok"}: ${n - failed}/${n} qaAA checks${errors.length ? ` · ${errors.length} errors` : ""}`);
process.exit(failed || errors.length ? 1 : 0);
