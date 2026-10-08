#!/usr/bin/env node
// Blind cohort 77030eb fixes, client half (headless, tools/browser.mjs):
//   sink   — B ("$27 119 with 'Kit complete' and nothing to buy"): a complete kit with gold names the next town work on the forge (a
//            two-tap `build $N` that spends it) and badges the forge tile `$` (fake engine).
//   train  — B ("tapping 'Training · Warlord tactic' did nothing"): a training plaque with an opener is a button and the tap opens it.
//   level  — B ("levelling corridor fighting L1→L5 dropped D33 75%→25% with no reason"): the core sends a level's rows (`level_adds`)
//            and the tactics panel shows them beside the level button (`L3 + …`); `compare outcomes` opens the styles AND the tactics
//            choosers (real wasm, the 307dbed fixture).
//   node web/tests/blind-77030eb.mjs [--part=sink,train,level]
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "sink,train,level").split(",");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
const patchSave = (page, lin, st = {}) => page.evaluate(async ([p, q]) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  for (const [k, v] of Object.entries(p)) e.lineage[k] = v && typeof v === "object" && !Array.isArray(v) && e.lineage[k] && typeof e.lineage[k] === "object" ? { ...e.lineage[k], ...v } : v;
  Object.assign(e, q); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
}, [lin, st]);
const camp = (page) => page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
try {
  if (parts.includes("sink")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
    page.on("pageerror", (e) => errors.push(`sink pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=231&runs=0`, { waitUntil: "domcontentloaded" });
    await camp(page);
    await patchSave(page, { heir: 2, best_depth: 12, gold: 60000, graveyard: [{ heir: 1, depth: 3, cause: "rat", deeds: [] }] }, { kit: { weapon: 3, armour: 4, pack: 4 } });
    await camp(page); await page.waitForTimeout(300);
    const L = await page.evaluate(() => ({ c: window.__riddle.lineage.commission, kit: window.__riddle.lineage.kit?.map((k) => !!k.next) }));
    check(!!L.c && L.kit.every((x) => !x), `the fake lineage: kit complete, a commission on the wire (${JSON.stringify(L)})`);
    const badge = await page.evaluate(() => document.querySelector(".cmd .tile[data-tile=forge] .kit-n, .cmd .tile[data-tile=blacksmith] .kit-n")?.textContent ?? null);
    check(badge === "$", `the forge tile badges what gold still buys (${badge})`);
    await page.locator(".cmd .tile[data-tile=forge], .cmd .tile[data-tile=blacksmith]").first().click();
    await page.waitForSelector(".forge [data-sink=commission]", { timeout: 5000 });
    const text = await page.locator(".forge").innerText();
    check(/Kit complete/.test(text) && /town upgrades/i.test(text) && text.includes(L.c.label), `the forge names the next town work under "Kit complete" (${text.replace(/\s+/g, " ").slice(0, 120)})`);
    const gold0 = await page.evaluate(() => window.__riddle.lineage.gold);
    await page.locator(".forge [data-sink=commission] button.commission").click();
    await page.locator(".forge [data-sink=commission] button.commission").click();
    await page.waitForFunction((g) => window.__riddle.lineage.gold < g, gold0, { timeout: 5000 });
    const after = await page.evaluate(() => ({ gold: window.__riddle.lineage.gold, works: window.__riddle.lineage.works }));
    check(gold0 - after.gold === L.c.price && after.works?.includes(L.c.label), `two taps build it: −$${L.c.price} (${gold0} → ${after.gold}; ${after.works})`);
    await page.close();
  }
  if (parts.includes("train")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
    page.on("pageerror", (e) => errors.push(`train pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&runs=0`, { waitUntil: "domcontentloaded" });
    await camp(page);
    const r = await page.evaluate(async () => {
      const { trainingBlock } = await import("/src/ui/tracks.ts");
      const inert = trainingBlock(["DRILLED · Warlord"]);
      let opened = null;
      const live = trainingBlock(["DRILLED · Warlord", "corridor fighting L3"], (_a, beat) => { opened = beat; });
      document.body.append(live);
      const b = live.querySelector("button.training-open");
      b?.click();
      const res = { inertButtons: inert.querySelectorAll("button").length, buttons: live.querySelectorAll("button.training-open").length, opened, label: b?.textContent };
      live.remove(); return res;
    });
    check(r.inertButtons === 0, "without an opener the plaques stay plaques");
    check(r.buttons === 2 && r.opened === "DRILLED · Warlord", `with one, each plaque is a button and a tap opens it (${JSON.stringify(r)})`);
    await page.close();
  }
  if (parts.includes("level")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
    page.on("pageerror", (e) => errors.push(`level pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&seed=307&fresh=1&runs=0`, { waitUntil: "domcontentloaded" });
    await camp(page);
    const fixture = readFileSync(new URL("crates/riddle-core/src/fixtures/save_307dbed.json", root), "utf8");
    await page.evaluate(async (e) => { const r = window.__riddle; const b = JSON.parse(r.exportSave()); b.engine = e; await r.importSave(JSON.stringify(b)); }, fixture);
    await camp(page); await page.waitForTimeout(500);
    // corridor fighting owned and worn at L2, with marks for its L3
    await page.evaluate(async () => {
      const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine); const p = e.lineage.pkg;
      p.owned = [...new Set([...(p.owned ?? []), "corridor_fighting", "kite_archers", "guarded"])]; p.tactics = ["corridor_fighting"];
      p.runs = { ...(p.runs ?? {}), corridor_fighting: 10 }; e.lineage.marks = 40;
      b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
    });
    await camp(page); await page.waitForTimeout(500);
    const pk = await page.evaluate(() => window.__riddle.lineage.packages?.all.find((p) => p.id === "corridor_fighting"));
    check(pk?.level === 2 && (pk.level_adds?.length ?? 0) === 1 && pk.level_adds[0].verb.v === "back_corridor", `the core names L3's row (${JSON.stringify(pk?.level_adds)} at L${pk?.level})`);
    await page.locator(".cmd .tile[data-tile=packages]").click();
    await page.waitForSelector(".pkg-panel", { timeout: 5000 });
    const adds = await page.evaluate(() => document.querySelector('.pkg-sec[data-kind=tactic] .pkg-level-adds')?.textContent ?? null);
    check(!!adds && /^L3 \+ .*→/.test(adds), `the level button carries its why (${adds})`);
    await page.locator(".pkg-compare").click();
    await page.waitForTimeout(300);
    const open = await page.evaluate(() => [...document.querySelectorAll(".pkg-sec")].filter((s) => s.querySelector(".pkg-choices:not([hidden])")).map((s) => s.dataset.kind));
    check(open.includes("stance") && open.includes("tactic"), `compare outcomes opens the styles and the tactics (${open})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`blind-77030eb: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`blind-77030eb: ok (${out.length} checks)`);
