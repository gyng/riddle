#!/usr/bin/env node
// Cut 4 §1 gate, rewritten for Cut 14 §4: tapping an insert patch on a full set never leaves the set over `max_rows` — the
// chip reads `↑ R3` (the own row that fired least: the run's counts, else the trace's fired rows, the lowest among equals) and the
// tap drops that row and lands the patch where it was measured; a patch the set already holds reads `at R1` and inserts nothing.
// Runs on the browser harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219) with the fake engine
// (`?engine=fake&dev=1`; the fake's max_rows is 4 without row unlocks).
//
//   node web/tests/patch-overflow.mjs        (or `pnpm test` in web/)
//
// Walk: boot fresh → fill the active set to max_rows → open a death screen carrying one insert patch (the death object is
// fabricated: the screen and the tap are the code under test) → the chip names the last row (nothing fired) → tap → assert on
// the camp: max rows, the named row gone, the patch at R1, the counter `4/4`, `send` enabled → a death whose trace shows R1, R2
// and R4 firing names R3 → tap → R3 gone. Then a held patch (`at R1`, inserts nothing) and a stall-style replace patch on a
// full set: still max rows, never over. Exit 1 on any failed assertion, console error or page error.
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

  // a death screen with one insert patch (insert_at 0); nothing is known to have fired, so the chip names the last row
  const patch = { row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 0.75, forecast_delta: 0.05 };
  const death = (p, turns = []) => ({ run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns }, patches: [p], morgue: "" });
  await page.evaluate((d) => { window.__riddle.rowFires = null; window.__riddle.go({ kind: "death", death: d }); }, death(patch));
  await waitFor((s) => s?.screen === "death", "death");
  const shown = await page.locator("button.patch").count();
  check(shown === 1, `death screen offers ${shown} patch on a full set`);
  const chip = await page.evaluate(() => ({ target: document.querySelector("button.patch .target")?.textContent.trim(), drop: document.querySelector("button.patch")?.dataset.drop }));
  check(chip.target === `↑ R${max}` && chip.drop === String(max - 1), `the chip names the row it replaces: "${chip.target}"`);
  await page.locator("button.patch").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the patch");
  await sleep(200);
  const over = await editor();
  check(over.rows.length === max, `after the tap: ${over.rows.length} rows (${max}), never over`);
  check(JSON.stringify(over.rows.slice(1)) === JSON.stringify(full.rows.slice(0, max - 1)), `the named row went; the others' text unchanged (shifted down by one)`);
  check(over.rows[0].includes("hp < 40%") && over.rows[0].includes("drink heal"), `the patch row is R1: "${over.rows[0]}"`);
  check(over.count === `${max}/${max}` && !over.countRed, `counter reads ${over.count}, not red`);
  check(over.drop.every((d) => !d), "no row is marked to drop");
  check(over.sendDisabled === false, "send enabled");
  const engineRows = await page.evaluate(async () => (await window.__riddle.engine.lineage()).sets[window.__riddle.active].rows.length);
  check(engineRows === max, `the engine took the set (${engineRows} rows)`);
  const fixed = over;

  // a second patch (a different row) on a death whose trace shows R1, R2 and R4 firing: R3 fired least and is the one named
  const patch2 = { row: { conds: [{ k: "foes>=", n: 2 }], verb: { v: "retreat" } }, insert_at: 1, survive: 0.7, forecast_delta: 0.04 };
  const turn = (row) => ({ t: 10 * row, row, verb: { v: "attack" }, hp: 20, foes: 1, telegraphs: [] });
  await page.evaluate((d) => { window.__riddle.rowFires = null; window.__riddle.go({ kind: "death", death: d }); }, death(patch2, [turn(0), turn(1), turn(3), turn(0)]));
  await waitFor((s) => s?.screen === "death", "the second death");
  const chip2 = await page.evaluate(() => document.querySelector("button.patch .target")?.textContent.trim());
  check(chip2 === "↑ R3", `with R1 · R2 · R4 in the trace the chip names R3: "${chip2}"`);
  await page.locator("button.patch").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the second patch");
  await sleep(200);
  const second = await editor();
  check(second.rows.length === max && second.count === `${max}/${max}` && !second.countRed, `still ${second.count}`);
  check(second.rows[1].includes("foes") && second.rows[1].includes("retreat") && second.rows[0] === fixed.rows[0] && second.rows[2] === fixed.rows[1] && second.rows[3] === fixed.rows[3], `R3 went and the patch sits at R2: "${second.rows[1]}"`);
  // the watched run's counts win over the trace: R1 never fired → `↑ R1`
  await page.evaluate((d) => { window.__riddle.rowFires = [0, 5, 3, 2]; window.__riddle.go({ kind: "death", death: d }); }, death({ ...patch2, row: { conds: [{ k: "alert>=", n: 3 }], verb: { v: "retreat" } } }, [turn(0)]));
  await waitFor((s) => s?.screen === "death", "the third death");
  const chip3 = await page.evaluate(() => document.querySelector("button.patch .target")?.textContent.trim());
  check(chip3 === "↑ R1", `the run's own counts name the row that never fired: "${chip3}"`);
  await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
  await waitFor((s) => s?.screen === "camp", "camp");

  // the same death opened again (the chronicle's old death, a second look): its patch row is already R1 — the tap opens the
  // camp on it and inserts nothing (QA on 952e306: "tapped patch → R1 inserted AGAIN → 5/4")
  await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d }), death(patch));
  await waitFor((s) => s?.screen === "death", "death again");
  const heldText = await page.locator("button.patch .surv").first().innerText();
  const heldTarget = await page.evaluate(() => document.querySelector("button.patch .target")?.textContent.trim() ?? "");
  check(heldText === "at R1" && heldTarget === "", `a held patch reads where it sits: "${heldText}" (no ↑)`);
  await page.locator("button.patch").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the second tap");
  await sleep(200);
  const again = await editor();
  check(JSON.stringify(again.rows) === JSON.stringify(second.rows) && again.count === `${max}/${max}`, `the second tap inserted nothing: ${again.count}`);

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
  check(replaced.rows[0] === second.rows[0] && replaced.rows[2] === second.rows[2], "the other rows untouched by the replace");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`patch-overflow: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`patch-overflow: ok (${out.length} checks)`);
