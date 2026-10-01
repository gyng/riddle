#!/usr/bin/env node
// Cut 4 §1 gate, rewritten for Cut 15 §3 (Cut 14 §4's silent `↑ R3` withdrawn): tapping an insert patch on a full set never takes
// a row by itself — the chip reads `+ drop one` and the tap opens a sheet of the set's own rows (card rows excluded), each with its
// fired count when known (`R1 … · 0/10`), the least-fired marked; no row leaves before a row is tapped; the tapped row is the one
// gone and the patch lands at its measured `insert_at` (one up when the dropped row sat above it); dismissing the sheet leaves the
// set whole on the death screen. A patch the set already holds reads `at R1` and inserts nothing; a stall `replace` never overflows.
// Runs on the browser harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219) with the fake engine
// (`?engine=fake&systems=none&dev=1`; the fake's max_rows is 4 without row unlocks).
//
//   node web/tests/patch-overflow.mjs        (or `pnpm test` in web/)
//
// Exit 1 on any failed assertion, console error or page error.
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows, deathDetails } from "./lib/frame.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 3 });
await deathDetails(page);   // death v2: this suite reads the trace, the ledger and the tablets under `details`
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
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await editRows(page);   // Cut 17: the tablets carry their chips, ▲▼ and × (the `edit` tile, remembered)

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
  const setNow = () => page.evaluate(() => window.__riddle.rules.rows.map((r) => JSON.stringify([r.conds, r.verb])).join("|"));
  const sheet = () => page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .drop-sheet .drop-row")].map((b) => ({ row: Number(b.dataset.row), text: b.textContent.replace(/\s+/g, " ").trim(), least: b.classList.contains("least"), fired: b.querySelector(".fired")?.textContent.trim() ?? "" })));
  // QA 1a2a4a9: on the death screen a tablet tap lights it; the gem applies the lit one
  const applyTop = async () => { await page.locator("button.patch").first().click({ timeout: 5000 }); await page.locator(".patch-gem").click({ timeout: 5000 }); };
  const openSheets = () => page.evaluate(() => document.querySelectorAll(".sheet-wrap").length);

  // a death screen with one insert patch (insert_at 0); nothing is known to have fired
  const patch = { row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 0.75, forecast_delta: 0.05 };
  const death = (p, turns = []) => ({ run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns }, patches: [p], morgue: "" });
  await page.evaluate((d) => { window.__riddle.rowFires = null; window.__riddle.go({ kind: "death", death: d }); }, death(patch));
  await waitFor((s) => s?.screen === "death", "death");
  check((await page.locator("button.patch").count()) === 1, "the death screen offers the patch on a full set");
  const chip = await page.evaluate(() => ({ target: document.querySelector("button.patch .target")?.textContent.trim(), full: document.querySelector("button.patch")?.dataset.full }));
  check(chip.target === "· drops one" && chip.full === "1", `the chip asks for a drop: "${chip.target}"`);
  const set0 = await setNow();
  await applyTop();
  await sleep(200);
  let rows = await sheet();
  check((await state()).screen === "death" && rows.length === max, `the tap opens the drop sheet on the death screen (${rows.length} rows)`);
  check((await setNow()) === set0, "no row left before a row is tapped");
  check(rows.every((r, i) => r.row === i && / → /.test(r.text) && r.fired === ""), `the sheet lists the set's rows, no count when none is known (${rows.map((r) => r.text.slice(0, 18)).join(" · ")})`);
  check(rows.filter((r) => r.least).length === 0, "nothing fired: all rows tie, none is marked (QA on 3d71c33: the last row was an arbitrary pick)");
  // dismissed: the set whole, the death screen up
  await page.keyboard.press("Escape"); await sleep(200);
  check((await openSheets()) === 0 && (await state()).screen === "death" && (await setNow()) === set0, "Escape closes the sheet; the set is whole and the death screen stays");
  await applyTop(); await sleep(200);
  await page.mouse.click(200, 20); await sleep(200);   // the backdrop
  check((await openSheets()) === 0 && (await state()).screen === "death" && (await setNow()) === set0, "a backdrop tap closes it too; nothing changed");
  // tap R3: R3 gone, the patch at R1
  await applyTop(); await sleep(200);
  await page.locator(".sheet-wrap .drop-row[data-row='2']").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the drop");
  await sleep(200);
  const over = await editor();
  check(over.rows.length === max, `after the drop: ${over.rows.length} rows (${max}), never over`);
  check(over.rows[0].includes("hp < 40%") && over.rows[0].includes("drink heal"), `the patch row is R1: "${over.rows[0]}"`);
  check(JSON.stringify(over.rows.slice(1)) === JSON.stringify([full.rows[0], full.rows[1], full.rows[3]]), "the tapped row (R3) is the one gone; the others unchanged");
  check(over.count === `${max}/${max}` && !over.countRed && over.drop.every((d) => !d) && over.sendDisabled === false, `counter ${over.count}, send enabled`);
  const engineRows = await page.evaluate(async () => (await window.__riddle.engine.lineage()).sets[window.__riddle.active].rows.length);
  check(engineRows === max, `the engine took the set (${engineRows} rows)`);

  // the run's counts: `· 0/10` per row, R1 (never fired) marked; a drop below the patch's place keeps insert_at
  const patch2 = { row: { conds: [{ k: "foes>=", n: 2 }], verb: { v: "retreat" } }, insert_at: 1, survive: 0.7, forecast_delta: 0.04 };
  await page.evaluate((d) => { window.__riddle.rowFires = [0, 5, 3, 2]; window.__riddle.go({ kind: "death", death: d }); }, death(patch2));
  await waitFor((s) => s?.screen === "death", "the second death");
  await applyTop(); await sleep(200);
  rows = await sheet();
  check(rows.map((r) => r.fired).join(" ") === "· 0/10 fires · 5/10 fires · 3/10 fires · 2/10 fires" && rows[0].least && rows.filter((r) => r.least).length === 1, `each row carries its fired count, the least-fired marked (${rows.map((r) => `${r.text.slice(0, 2)}${r.fired}${r.least ? "↓" : ""}`).join(" ")})`);
  const before2 = over;   // the camp as the last drop left it
  await page.locator(".sheet-wrap .drop-row[data-row='3']").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the second drop"); await sleep(200);
  const second = await editor();
  check(second.rows.length === max && second.rows[1].includes("foes") && second.rows[1].includes("retreat") && second.rows[0] === before2.rows[0] && second.rows[2] === before2.rows[1] && second.rows[3] === before2.rows[2], `R4 went and the patch sits at its measured R2: "${second.rows[1]}"`);
  // a drop above the patch's place: insert_at 2, R1 dropped → the patch lands at R2 (one up)
  const patch3 = { row: { conds: [{ k: "alert>=", n: 3 }], verb: { v: "retreat" } }, insert_at: 2, survive: 0.7, forecast_delta: 0.04 };
  await page.evaluate((d) => { window.__riddle.rowFires = null; window.__riddle.go({ kind: "death", death: d }); }, death(patch3));
  await waitFor((s) => s?.screen === "death", "the third death");
  await applyTop(); await sleep(200);
  await page.locator(".sheet-wrap .drop-row[data-row='0']").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after the third drop"); await sleep(200);
  const third = await editor();
  check(third.rows.length === max && third.rows[1].includes("alert") && third.rows[0] === second.rows[1] && third.rows[2] === second.rows[2] && third.rows[3] === second.rows[3], `R1 went and the patch moved up one to R2: "${third.rows[1]}"`);
  // a card row is never offered for the drop (it sits outside max_rows)
  await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [], verb: { v: "tactic", a: "kite_archers" } }, 1, "card"); });
  await page.evaluate((d) => { window.__riddle.rowFires = null; window.__riddle.go({ kind: "death", death: d }); }, death(patch));
  await waitFor((s) => s?.screen === "death", "the fourth death");
  await applyTop(); await sleep(200);
  rows = await sheet();
  check(rows.length === max && !rows.some((r) => r.row === 1), `the card row is not in the sheet (rows ${rows.map((r) => r.row + 1).join(", ")})`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows.splice(1, 1); r.rulesChanged(); r.go({ kind: "camp" }); });
  await waitFor((s) => s?.screen === "camp", "camp");
  const fixed = await editor();

  // a death whose patch row the set already holds: `at R1`, the tap opens the camp on it and inserts nothing
  const heldPatch = { ...patch3, insert_at: 0 };
  await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d }), death(heldPatch));
  await waitFor((s) => s?.screen === "death", "death again");
  const heldText = await page.locator("button.patch .surv").first().innerText();
  const heldTarget = await page.evaluate(() => document.querySelector("button.patch .target")?.textContent.trim() ?? "");
  check(heldText === "already written" && heldTarget === "", `a held patch reads where it sits: "${heldText}" (no drop)`);
  await applyTop();
  await waitFor((s) => s?.screen === "camp", "camp after the held tap");
  await sleep(200);
  const again = await editor();
  check(JSON.stringify(again.rows) === JSON.stringify(fixed.rows) && again.count === `${max}/${max}`, `the held tap inserted nothing: ${again.count}`);

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
