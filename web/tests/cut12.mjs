#!/usr/bin/env node
// Cut 12 gates, client side (docs/CUT12.md §1, §3, §6), on the fake engine (`?engine=fake&systems=none&dev=1`) through the GPU harness
// (tools/browser.mjs) against the dev server (tools/dev.sh, :5219):
//   §1  the editor chip counts own rows (`2/4 · 2 cards`); a bought card's row goes before the engagement row (`insert_at`);
//       card rows never count against `max_rows` (four own rows + two cards send; the fifth own row is `5/4 · drop one`, the
//       drop mark on an own row); picking a card verb clears the row's conds; a card another row holds is not offered
//   §3  the forecast panel's ends line: `bank 40% · return 35% · death 25% · ~$54` (Cut 13 §5: `death 25% ±4…` with its ± and the first paint's `…`)
//   §6  `+1 row ⊘ fill rows` (a requirement, never `rows full`) lifts once a rule edit fills the rows (no run needed); a free supply reads `leash · free` (QA 92eb880); a
//       supply line's `×` removes that line only; the combo is named (`gambler`), not counted
//
//   node web/tests/cut12.mjs        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows, openPanel } from "./lib/frame.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shots = process.argv.includes("--shots") ? resolve(ROOT, "scratchpad/cut12") : null;
if (shots) mkdirSync(shots, { recursive: true });
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
/** What the camp shows: the editor's rows (text, card?, drop?), the counter, `send`, the forecast lines, the shelf, the supplies. */
const camp = () => page.evaluate(() => ({
  rows: [...document.querySelectorAll(".editor .row")].map((r) => ({ text: r.querySelector(".chips").innerText.replace(/\s+/g, " ").trim(), card: r.classList.contains("locked"), drop: r.classList.contains("drop"), conds: r.querySelector(".chips").querySelectorAll(":scope > .chip.cond:not(.add)").length })),   // the row's own cond chips, not a card's inline rows,
  count: document.querySelector(".rows-foot .num")?.textContent ?? "",
  countRed: !!document.querySelector(".rows-foot .num.over"),
  send: document.querySelector("button.send")?.textContent ?? "", sendDisabled: document.querySelector("button.send")?.disabled ?? null,
  plus: !!document.querySelector(".rows-foot .btn.ghost"),
  yours: document.querySelector(".fc-yours")?.textContent ?? "",
  ends: document.querySelector(".fc-ends:not([hidden])")?.textContent ?? "",
  rowCard: [...document.querySelectorAll(".unlocks .card")].map((c) => c.innerText.replace(/\s+/g, " ").trim()).find((t) => t.startsWith("+1 rule")) ?? "",
  supplies: [...document.querySelectorAll(".supplies .chip.item")].map((c) => c.innerText.replace(/\s+/g, " ").trim()),
  supplyCount: document.querySelector(".supplies .row-label .num")?.textContent ?? "",
}));
const shot = async (name) => { if (shots) await page.screenshot({ path: `${shots}/${name}.png`, fullPage: true }); };

try {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await editRows(page);   // Cut 17: the tablets carry their chips, ▲▼ and × (the `edit` tile, remembered)
  // a lineage with marks, the card gates met, a potion identified (so the supply catalogue sells it) and gold
  const ok = await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    e.lineage.marks = 20; e.lineage.gold = 300; e.lineage.best_depth = 3;
    e.lineage.facts.push("foe:monkey:thief", "foe:jackal:pack", "foe:goblin_archer:ranged", "item:red=heal", "item:blue=strength");
    b.engine = JSON.stringify(e);
    return r.importSave(JSON.stringify(b));
  });
  check(ok, "the lineage took marks, facts and gold");
  await waitFor((s) => s?.screen === "camp", "camp again");
  await page.waitForFunction(() => window.__riddle.unlockCat.length > 0, null, { timeout: 10_000 });
  // Cut 17: the unlock panel lists the whole catalogue from here on (`more`, remembered); the panel closes over the tablets
  await openPanel(page, "unlocks", { all: true }); await page.keyboard.press("Escape");
  await sleep(600);
  await openPanel(page, "loadout");   // the shelf is read as the player sees it, in its panel
  let c = await camp();
  await page.keyboard.press("Escape"); await sleep(100);   // the panel closes over the tablets
  check(c.count === "2/4" && c.rows.length === 2, `the preset reads ${c.count}, ${c.rows.length} rows`);
  check(/^written: 0 of 2 rules · gambler$/.test(c.yours), `the yours line names the combo: "${c.yours}"`);
  check(/^ends bank <?\d+% · return <?\d+%( · stall <?\d+%)? · death <?\d+%( ±\d+…?)? · avg \$\d+\/run$/.test(c.ends), `the forecast's ends line: "${c.ends}"`);
  check(/⊘ fill rules/.test(c.rowCard) && !/rows full/.test(c.rowCard), `+1 row waits on the rows at 2/4, as a requirement: "${c.rowCard}"`);
  check(c.supplies.length === 1 && /^leash · free ×$/.test(c.supplies[0]), `the kennel's leash reads free (QA 92eb880: kennel was unexplained): "${c.supplies[0]}"`);

  // §1: a bought card's row goes before the engagement row (the attack row), and never counts
  const at = await page.evaluate(() => window.__riddle.unlockCat.find((u) => u.id === "thief_guard")?.insert_at);
  check(at === 1, `the catalogue says the card goes before the attack row (insert_at ${at})`);
  await page.evaluate(() => window.__riddle.buy("thief_guard"));
  await sleep(700);
  c = await camp();
  check(c.rows.length === 3 && c.rows[1].card && /thief guard/.test(c.rows[1].text) && /attack/.test(c.rows[2].text), `the card sits at R2, above attack: ${c.rows.map((r) => r.text.slice(0, 24)).join(" | ")}`);
  check(c.count === "2/4 rules + 1 tactic", `the chip counts own rows and the card beside: "${c.count}"`);
  check(/^written: 0 of 2 rules/.test(c.yours) && !/card R\d+ first/.test(c.yours), `yours counts own rows, no "card R2 first" (a card sits before attack on purpose): "${c.yours}"`);
  await page.evaluate(() => window.__riddle.buy("pack_break"));
  await sleep(700);
  c = await camp();
  check(c.rows.length === 4 && c.rows[2].card && /pack break/.test(c.rows[2].text) && /attack/.test(c.rows[3].text), `the second card also sits above attack: ${c.rows.map((r) => r.text.slice(0, 24)).join(" | ")}`);
  check(c.count === "2/4 rules + 2 tactics" && !c.countRed && c.sendDisabled === false, `two cards, still 2/4: "${c.count}", send ${c.sendDisabled ? "disabled" : "enabled"}`);
  await shot("01-cards-above-attack");

  // four own rows + two cards: full, sends; the fifth own row is refused as `5/4 · drop one` with the mark on an own row
  // (a row inserted from outside the editor — a patch — reaches the camp through `go`, as the death screen does)
  await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [{ k: "hp>", n: 90 }], verb: { v: "rest" } }, r.rules.rows.length); r.insertRow({ conds: [{ k: "depth>=", n: 3 }], verb: { v: "return" } }, r.rules.rows.length); r.go({ kind: "camp" }); });
  await page.waitForFunction(() => window.__riddle.unlockCat.length > 0, null, { timeout: 10_000 });
  await sleep(1200);
  c = await camp();
  check(/^ends bank <?\d+% · return [1-9]\d*%( · stall <?\d+%)? · death <?\d+%( ±\d+…?)? · avg \$\d+\/run$/.test(c.ends), `with a return row the ends line shows a return share: "${c.ends}"`);
  check(c.count === "4/4 rules + 2 tactics" && !c.countRed && c.sendDisabled === false && !c.plus, `4 own + 2 cards: "${c.count}", send enabled, no +`);
  check(!/fill rules|rows full/.test(c.rowCard) && /\+1 rule/.test(c.rowCard), `+1 row lifted at 4/4 without a run: "${c.rowCard}"`);
  const engineRows = await page.evaluate(async () => (await window.__riddle.engine.lineage()).sets[window.__riddle.active].rows.length);
  check(engineRows === 6, `the engine took all six rows (${engineRows})`);
  await shot("02-four-own-two-cards");
  await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [{ k: "alert>=", n: 5 }], verb: { v: "return" } }, r.rules.rows.length); r.go({ kind: "camp" }); });
  await sleep(600);
  c = await camp();
  check(c.count === "5/4 rules + 2 tactics" && c.countRed && c.sendDisabled === true && c.send === "5/4 · drop one", `the fifth own row: "${c.count}" red, send "${c.send}"`);
  check(c.rows.filter((r) => r.drop).length === 1 && c.rows[6].drop && !c.rows[6].card, "the drop mark is on the last own row, never a card");
  await shot("03-fifth-own-row");
  await page.locator(".editor .row.drop .x").first().click({ timeout: 5000 });   // QA 0c6e126 (qaY): a row's × takes two taps — the first arms it
  await sleep(150);
  await page.locator(".editor .row.drop .x").first().click({ timeout: 5000 });
  await sleep(300);
  c = await camp();
  check(c.count === "4/4 rules + 2 tactics" && c.sendDisabled === false, `after ×: "${c.count}", send enabled`);

  // §1: the verb picker — a card another row holds is not offered; picking a card verb clears the row's conds
  await page.locator(".editor .row").nth(4).locator(".chip.verb").click({ timeout: 5000 });   // R5 `hp > 90% → rest`, an own row
  await sleep(200);
  let verbs = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chip.verb")].map((b) => b.textContent.trim()));
  check(!verbs.includes("tactic thief guard") && !verbs.includes("tactic pack break"), `cards the set holds are not offered (${verbs.filter((v) => v.startsWith("tactic")).join(", ") || "no cards"})`);
  await page.keyboard.press("Escape"); await sleep(200);
  await page.locator(".editor .row").nth(1).locator(".x").click({ timeout: 5000 }); await sleep(150);   // (two taps: arm, then drop)
  await page.locator(".editor .row").nth(1).locator(".x").click({ timeout: 5000 });   // drop the thief guard row
  await sleep(300);
  await page.locator(".editor .row").nth(3).locator(".chip.verb").click({ timeout: 5000 });   // R4 `hp > 90% → rest`
  await sleep(200);
  verbs = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chip.verb")].map((b) => b.textContent.trim()));
  check(verbs.includes("tactic thief guard") && !verbs.includes("tactic pack break"), "the freed card is offered again, the held one is not");
  await page.locator(".sheet-wrap .chip.verb", { hasText: "tactic thief guard" }).first().click({ timeout: 5000 });
  await sleep(400);
  c = await camp();
  check(c.rows[3].card && c.rows[3].conds === 0 && /thief guard/.test(c.rows[3].text) && !/hp >/.test(c.rows[3].text), `picking a card verb cleared the conds: "${c.rows[3].text.slice(0, 40)}"`);
  check(c.count === "3/4 rules + 2 tactics", `the row moved from own to card: "${c.count}"`);
  await shot("04-card-verb-cleared-conds");

  // §6: a supply line's × removes that line only (the fake has `dropSupply`); the fallback (an engine without it) rebuys the rest
  await openPanel(page, "loadout");   // Cut 17: the supplies are the loadout panel
  await page.locator(".supplies .chip.buy", { hasText: "heal potion" }).first().click({ timeout: 5000 });
  await sleep(400);
  await page.locator(".supplies .chip.buy", { hasText: "strength potion" }).first().click({ timeout: 5000 });
  await sleep(400);
  c = await camp();
  check(c.supplyCount === "3/3" && c.supplies.length === 3, `three lines on the shelf (${c.supplies.join(" / ")})`);
  await page.locator(".supplies .chip.item", { hasText: "heal potion" }).locator(".x").click({ timeout: 5000 });
  await sleep(400);
  c = await camp();
  check(c.supplies.length === 2 && /leash · free/.test(c.supplies[0]) && /strength/.test(c.supplies[1]), `× took the heal only: ${c.supplies.join(" / ")}`);
  await shot("05-supplies-one-line");
  await page.evaluate(() => { window.__riddle.engine.dropSupply = () => Promise.reject(new Error("wasm: dropSupply")); });
  await page.locator(".supplies .chip.item", { hasText: "strength" }).locator(".x").click({ timeout: 5000 });
  await sleep(600);
  c = await camp();
  // the fallback clears the shelf and rebuys the bought lines; a free line has no price to rebuy at (the core hands it back at the
  // next exit) — the reason `dropSupply` is a core item
  check(c.supplies.length === 0 && c.supplyCount === "0/3", `without dropSupply: the strength line is gone, the free leash with it until the next exit (${c.supplies.join(" / ") || "empty"})`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut12: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut12: ok (${out.length} checks)`);
