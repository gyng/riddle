#!/usr/bin/env node
// Cut 9 §1 gate: the vocabulary sheet never offers a locked token. Runs on the GPU harness (tools/browser.mjs) against
// the dev server (tools/dev.sh, :5219) with the fake engine (`?engine=fake&dev=1`; a fresh fake lineage owns no cond
// unlock, so `alert ≥`, `turns >`, `loot ≥`, `on kill`, `on see` are in `Vocabulary.locked`).
//
//   node web/tests/locked-cond.mjs        (part of `pnpm test` in web/)
//
// Walk: boot fresh → the fake reports ≥ 1 locked cond → open R1's first cond chip → every locked token is on the sheet,
// dim, carrying its `needs` text and `⊘`, and is not a button → clicking one changes nothing (the sheet stays open, the row's
// text is unchanged, the engine's set is unchanged) → every offered token is a button and none of them is a locked one →
// the same on the `+` (second cond) sheet. Then the Cut 9 §4 card trigger: a `[card]` row reads `[card] pack break ·
// foe: pack` and, above a player row, the forecast's yours line reads `· card R1 first`. Exit 1 on any failed assertion,
// console error or page error.
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchGpu } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchGpu();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 3 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
/** The open sheet's tokens: offered (buttons) and locked (spans with ⊘ and a needs line). */
const sheet = () => page.evaluate(() => {
  const s = document.querySelector(".sheet-wrap:last-of-type .sheet-body");
  if (!s) return null;
  const text = (el) => el.textContent.replace(/\s+/g, " ").trim();
  return {
    offered: [...s.querySelectorAll("button.chip.cond")].map(text),
    locked: [...s.querySelectorAll(".chip.cond.locked.off")].map((el) => ({ text: text(el), tag: el.tagName, needs: el.querySelector(".needs")?.textContent ?? "", glyph: el.textContent.includes("⊘"), dim: getComputedStyle(el).opacity })),
    lockedButtons: s.querySelectorAll("button.chip.cond.locked.off").length,
  };
});
const rowText = (i) => page.evaluate((i) => document.querySelectorAll(".editor .row")[i]?.querySelector(".chips").innerText.replace(/\s+/g, " ").trim() ?? "", i);
const engineRows = () => page.evaluate(async () => JSON.stringify((await window.__riddle.engine.lineage()).sets[window.__riddle.active].rows));

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  const vocab = await page.evaluate(() => { const v = window.__riddle.vocab; return { locked: (v.locked ?? []).map((l) => ({ k: l.cond.k, needs: l.needs })), offered: v.conds.map((c) => c.k) }; });
  check(vocab.locked.length >= 1, `fake vocabulary carries ${vocab.locked.length} locked cond(s): ${vocab.locked.map((l) => `${l.k} (${l.needs})`).join(", ")}`);
  check(vocab.locked.every((l) => l.needs), "every locked cond carries a needs text");
  check(vocab.locked.every((l) => !vocab.offered.includes(l.k)), "no locked cond is also offered");

  const before = await rowText(0), engineBefore = await engineRows();
  await page.locator(".editor .row").first().locator("button.chip.cond").first().click({ timeout: 5000 });
  await sleep(200);
  const sh = await sheet();
  check(!!sh, "the cond sheet opened");
  check(sh.locked.length === vocab.locked.length, `the sheet shows all ${vocab.locked.length} locked tokens (${sh.locked.length})`);
  check(sh.lockedButtons === 0 && sh.locked.every((l) => l.tag === "SPAN"), "locked tokens are spans, not buttons (no handler)");
  check(sh.locked.every((l) => l.glyph && l.needs), `each locked token carries ⊘ and its needs: ${sh.locked.map((l) => l.text).join(" | ")}`);
  check(sh.locked.every((l) => Number(l.dim) < 0.6), "locked tokens render dim");
  const lockedNames = new Set(vocab.locked.map((l) => l.k));
  check(sh.offered.length === vocab.offered.length, `the sheet offers ${sh.offered.length} tokens (vocabulary: ${vocab.offered.length})`);
  const offeredKs = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap:last-of-type button.chip.cond")].map((b) => b.textContent.trim()));
  const lockedLabels = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap:last-of-type .chip.cond.locked.off")].map((el) => el.firstChild && el.childNodes[1] ? el.childNodes[1].textContent.trim() : ""));
  check(offeredKs.every((t) => !lockedLabels.includes(t)), "no offered button reads as a locked token");
  // clicking a locked token does nothing: the sheet stays, the row and the engine's set are unchanged
  await page.locator(".sheet-wrap .chip.cond.locked.off").first().click({ timeout: 5000, force: true });
  await sleep(300);
  check(!!(await sheet()), "the sheet is still open after tapping a locked token");
  check((await rowText(0)) === before, `R1 unchanged after the tap: "${before}"`);
  check((await engineRows()) === engineBefore, "the engine's set is unchanged");
  await page.keyboard.press("Escape"); await sleep(150);
  check(!(await sheet()), "escape closes the sheet");
  // the `+` (second cond) sheet says the same
  const plus = page.locator(".editor .row").first().locator("button.chip.cond.add");
  if (await plus.count()) {
    await plus.click({ timeout: 5000 }); await sleep(200);
    const sh2 = await sheet();
    check(sh2 && sh2.lockedButtons === 0 && sh2.locked.length === vocab.locked.length, "the second-cond sheet shows the locked tokens, none selectable");
    await page.keyboard.press("Escape"); await sleep(150);
  }
  check(lockedNames.size > 0, "locked set non-empty");

  // Cut 9 §4: a card row's trigger on its chip, and `· card R1 first` on the yours line when it sits above a player row
  await page.evaluate(async () => {
    const r = window.__riddle;
    r.unlockCat = await r.engine.unlocks();
    r.rules.rows.splice(0, r.rules.rows.length, { conds: [], verb: { v: "tactic", a: "pack_break" }, origin: "card" }, { conds: [{ k: "hp<", n: 40 }], verb: { v: "retreat" }, origin: "player" });
    r.rulesChanged(); r.go({ kind: "camp" });
  });
  await sleep(300);
  const cardRow = await rowText(0);
  check(/\[card\] pack break · foe: pack/.test(cardRow), `card chip reads its trigger: "${cardRow.split("\n")[0].slice(0, 60)}"`);
  const yours = await page.evaluate(() => document.querySelector(".fc-yours")?.textContent ?? "");
  check(/· card R1 first/.test(yours), `yours line: "${yours}"`);
  // the card below the player row: no warning
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows.reverse(); r.rulesChanged(); });
  await sleep(200);
  const yours2 = await page.evaluate(() => document.querySelector(".fc-yours")?.textContent ?? "");
  check(!/card R\d first/.test(yours2), `yours line without a card above: "${yours2}"`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`locked-cond: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`locked-cond: ok (${out.length} checks)`);
