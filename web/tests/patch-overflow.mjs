#!/usr/bin/env node
// Cut 4 §1 gate: tapping a patch with full rows never changes any existing row and disables `send` until a
// row is removed; a patch the set already holds reads `at R1` and inserts nothing. Runs on the browser harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219)
// with the fake engine (`?engine=fake&dev=1`; the fake's max_rows is 4 without row unlocks).
//
//   node web/tests/patch-overflow.mjs        (or `pnpm test` in web/)
//
// Walk: boot fresh → fill the active set to max_rows → open a death screen carrying one insert patch (the death
// object is fabricated: the screen and the tap are the code under test) → tap the patch → assert on the camp:
// max+1 rows, every prior row's text unchanged, the counter reads `5/4` in red, the last row is marked `drop`,
// `send` is disabled → tap that row's × → `4/4`, `send` enabled. Then a stall-style replace patch on a full set:
// still max rows, never over. Exit 1 on any failed assertion, console error or page error.
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
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 3 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
/** What the editor shows: row texts, the counter, whether it is red, which rows are marked `drop`, `send` state. */
const editor = () => page.evaluate(() => ({
  rows: [...document.querySelectorAll(".editor .row")].map((r) => r.querySelector(".chips").innerText.replace(/\s+/g, " ").trim()),
  drop: [...document.querySelectorAll(".editor .row")].map((r) => r.classList.contains("drop")),
  count: document.querySelector(".rows-foot .num")?.textContent ?? "",
  countRed: !!document.querySelector(".rows-foot .num.over"),
  sendDisabled: document.querySelector("button.send")?.disabled ?? null,
  plus: !!document.querySelector(".rows-foot .btn.ghost"),
}));

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");

  // fill the set to max_rows with rows the fake's patch candidates never equal
  const max = await page.evaluate(() => {
    const r = window.__riddle; const filler = [{ conds: [{ k: "hp>", n: 90 }], verb: { v: "rest" } }, { conds: [{ k: "foes>=", n: 4 }], verb: { v: "retreat" } }, { conds: [{ k: "depth>=", n: 12 }], verb: { v: "bank" } }, { conds: [{ k: "alert>=", n: 5 }], verb: { v: "return" } }];
    r.rules.rows.splice(r.vocab.max_rows); let i = 0;
    while (r.rules.rows.length < r.vocab.max_rows) r.insertRow(filler[i++ % filler.length], r.rules.rows.length);
    r.go({ kind: "camp" });
    return r.vocab.max_rows;
  });
  const full = await editor();
  check(full.rows.length === max, `set full: ${full.rows.length}/${max} rows`);
  check(full.sendDisabled === false, "send enabled on a full set");
  check(!full.plus, "no + on a full set");

  // a death screen with one insert patch (insert_at 0) and one that would insert at the end
  const patch = { row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 0.75, forecast_delta: 0.05 };
  await page.evaluate((p) => {
    const r = window.__riddle;
    r.go({ kind: "death", death: { run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns: [] }, patches: [p], morgue: "" } });
  }, patch);
  await waitFor((s) => s?.screen === "death", "death");
  const shown = await page.locator("button.patch").count();
  check(shown === 1, `death screen offers ${shown} patch on a full set`);
  await page.locator("button.patch").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the patch");
  await sleep(200);
  const over = await editor();
  check(over.rows.length === max + 1, `after the tap: ${over.rows.length} rows (${max} + 1), nothing evicted`);
  check(JSON.stringify(over.rows.slice(1)) === JSON.stringify(full.rows), "every existing row's text unchanged (shifted down by one)");
  check(over.rows[0].includes("hp < 40%") && over.rows[0].includes("drink heal"), `the patch row is R1: "${over.rows[0]}"`);
  check(over.count === `${max + 1}/${max}` && over.countRed, `counter reads ${over.count} in red`);
  check(over.drop.length === max + 1 && over.drop[max] === true && over.drop.slice(0, max).every((d) => !d), "only the last row is marked as the one the engine would drop");
  check(over.sendDisabled === true, "send disabled while over budget");
  check(!over.plus, "no + while over budget");
  // the engine still holds a valid set (the forecast is not refreshed with an over-budget one)
  const engineRows = await page.evaluate(async () => (await window.__riddle.engine.lineage()).sets[window.__riddle.active].rows.length);
  check(engineRows <= max, `engine kept a valid set (${engineRows} rows)`);

  // resolve: × on the marked row
  await page.locator(".editor .row.drop .x").first().click({ timeout: 5000 });
  await sleep(200);
  const fixed = await editor();
  check(fixed.rows.length === max && fixed.count === `${max}/${max}` && !fixed.countRed, `after ×: ${fixed.count}, not red`);
  check(fixed.sendDisabled === false, "send enabled again");
  check(JSON.stringify(fixed.rows) === JSON.stringify([over.rows[0], ...full.rows.slice(0, max - 1)]), "the remaining rows are the patch + the first max−1 originals");

  // the same death opened again (the chronicle's old death, a second look): its patch row is already R1 — the tap opens the
  // camp on it and inserts nothing (QA on 952e306: "tapped patch → R1 inserted AGAIN → 5/4")
  await page.evaluate((p) => {
    const r = window.__riddle;
    r.go({ kind: "death", death: { run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns: [] }, patches: [p], morgue: "" } });
  }, patch);
  await waitFor((s) => s?.screen === "death", "death again");
  const heldText = await page.locator("button.patch .surv").first().innerText();
  check(heldText === "at R1", `a held patch reads where it sits: "${heldText}"`);
  await page.locator("button.patch").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the second tap");
  await sleep(200);
  const again = await editor();
  check(JSON.stringify(again.rows) === JSON.stringify(fixed.rows) && again.count === `${max}/${max}`, `the second tap inserted nothing: ${again.count}`);

  // a stall-style replace patch on a full set never overflows
  const rep = { row: { conds: [{ k: "hp<", n: 30 }], verb: { v: "return" } }, insert_at: 1, survive: 0.6, forecast_delta: 0.03, replace: true };
  await page.evaluate((p) => { const r = window.__riddle; r.go({ kind: "report", report: { elapsed_s: 60, runs: 4, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, stall: { row: 1, fired: 4, text: "R2 ended 4 runs at D3", patches: [p] } } }); }, rep);
  await waitFor((s) => s?.screen === "report", "report");
  await page.locator("button.patch").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the replace patch");
  await sleep(200);
  const replaced = await editor();
  check(replaced.rows.length === max && !replaced.countRed && replaced.sendDisabled === false, `replace on a full set: ${replaced.count}, send enabled`);
  check(replaced.rows[1].includes("hp < 30%") && replaced.rows[1].includes("return"), `R2 replaced: "${replaced.rows[1]}"`);
  check(replaced.rows[0] === fixed.rows[0] && replaced.rows[2] === fixed.rows[2], "the other rows untouched by the replace");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`patch-overflow: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`patch-overflow: ok (${out.length} checks)`);
