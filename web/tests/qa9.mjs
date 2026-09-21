#!/usr/bin/env node
// QA lapses on build 952e306 (scratchpad/qaA, qaB), client side, on the fake engine (`?engine=fake&dev=1`) through the GPU
// harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219):
//   1  a screen change closes every open sheet; a kept death (the chronicle's ▸) goes back to the camp on Escape (a fresh death does not)
//   2  an owned card's chip reads `card: thief guard · owned`; its sheet carries the title and, once the row is dropped, `insert`
//      puts the row back where a buy would (before the engagement row) and opens the camp on it
//   3  the `+1 row` gate reads `⊘ fill rows` on the card and in its sheet (never `rows full`)
//   4  the chronicle, ledger and forge sheets carry their one-word title; an empty chronicle shows nothing else (no lone `·`)
//   5  settings: `lineage seed 7` (decimal)
//   6  an empty vault slot is not a tap target
//   7  the cond picker's `×` leads the sheet
//   8  the supply shop and the unlock shelf stay painted while the forecast computes (from the last catalogue, the fetches ahead of the forecast)
//   9  an unlock sheet's `buy` is off while gated — the engine's `needs`, or the marks short against the live lineage (a stale `available`)
//  10  the yours line has no `· card RN first` suffix (a card sits before the engagement row on purpose)
//
//   node web/tests/qa9.mjs        (part of `pnpm test` in web/)
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
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const sheets = () => page.evaluate(() => document.querySelectorAll(".sheet-wrap").length);
/** The top sheet as text: its title label, its buttons, its body text. */
const sheet = () => page.evaluate(() => {
  const w = [...document.querySelectorAll(".sheet-wrap")].pop(); if (!w) return null;
  return { label: w.querySelector(".label")?.textContent.trim() ?? "", buttons: [...w.querySelectorAll("button")].map((b) => b.textContent.trim()), text: w.innerText.replace(/\s+/g, " ").trim(), first: w.querySelector(".sheet-body")?.firstElementChild?.outerHTML.slice(0, 60) ?? "" };
});
const rows = () => page.evaluate(() => [...document.querySelectorAll(".editor .row")].map((r) => ({ text: r.querySelector(".chips").innerText.replace(/\s+/g, " ").trim(), card: r.classList.contains("locked"), conds: r.querySelector(".chips").querySelectorAll(":scope > .chip.cond:not(.add)").length })));
const count = () => page.evaluate(() => document.querySelector(".rows-foot .num")?.textContent ?? "");
/** A lineage with marks, gold and the thief fact (the card's gate), a chronicle as given, and a grave for heir 1 that keeps its death. */
const setLineage = (chronicle, grave) => page.evaluate(async ({ chronicle, grave }) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  e.lineage.marks = 20; e.lineage.gold = 300; e.lineage.best_depth = 3;
  e.lineage.facts.push("foe:monkey:thief");
  e.lineage.chronicle = chronicle;
  e.lineage.graveyard = grave ? [{ heir: 1, depth: 3, cause: "jackal", deeds: [], death_id: 0 }] : [];
  b.engine = JSON.stringify(e);
  return r.importSave(JSON.stringify(b));
}, { chronicle, grave });
const fakeDeath = (kept) => page.evaluate((kept) => {
  const r = window.__riddle;
  r.go({ kind: "death", death: { run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns: [] }, patches: [], morgue: "t10 a line" }, kept });
}, kept);

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  check(await setLineage([], false), "the lineage took marks, gold, the thief fact and an empty chronicle");
  await waitFor((s) => s?.screen === "camp", "camp again");
  await page.waitForFunction(() => window.__riddle.unlockCat.length > 0, null, { timeout: 10_000 });
  await sleep(600);

  // 4 · 5 · 6: sheet titles, the empty chronicle, the seed, the empty vault slot
  await page.locator("button.mini", { hasText: "chronicle" }).first().click({ timeout: 5000 }); await sleep(200);
  let s = await sheet();
  check(s?.label === "chronicle" && !/·/.test(s.text.replace("chronicle", "")) && s.buttons.length === 0, `an empty chronicle shows its label and nothing else: "${s?.text}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.locator("button.mini", { hasText: "ledger" }).first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "ledger" && /seen/i.test(s.text), `the ledger sheet is titled: "${s?.label}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.locator("button.mini", { hasText: "forge" }).first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "forge" && /craft/i.test(s.text), `the forge sheet is titled: "${s?.label}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.locator("button.gear").click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(/lineage seed 7\b/i.test(s?.text ?? "") && !/#7\b/.test(s?.text ?? ""), `settings name the seed in decimal: "${/lineage \S+ \S+/i.exec(s?.text ?? "")?.[0]}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  const slot = await page.evaluate(() => { const e = document.querySelector(".vault .chip.empty"); return e ? { tag: e.tagName, pe: getComputedStyle(e).pointerEvents, inButton: !!e.closest("button") } : null; });
  check(slot && slot.tag === "SPAN" && slot.pe === "none" && !slot.inButton, `the empty vault slot is a plain marker (${slot?.tag}, pointer-events ${slot?.pe})`);

  // 3: the gate on the card and in its sheet
  const rowCard = page.locator(".unlocks .card", { hasText: "+1 row" }).first();
  const cardText = (await rowCard.innerText()).replace(/\s+/g, " ");
  check(/⊘ fill rows/.test(cardText) && !/rows full/.test(cardText), `the +1 row card reads a requirement: "${cardText}"`);
  await rowCard.click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(/⊘ fill rows/.test(s?.text ?? "") && !/rows full/.test(s?.text ?? ""), `so does its sheet: "${s?.text}"`);
  await page.keyboard.press("Escape"); await sleep(150);

  // 7: the cond picker's × leads the sheet; it removes the cond
  await page.locator(".editor .row").first().locator(".chip.cond").first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(/^<button class="btn ghost wide"/.test(s?.first ?? "") && s.buttons[0] === "×", `the cond picker's × is its first control: ${s?.first}`);
  await page.locator(".sheet-wrap .sheet-body > button.btn.ghost.wide").first().click({ timeout: 5000 }); await sleep(300);
  let rs = await rows();
  check((await sheets()) === 0 && rs[0].conds === 0, `× removed R1's cond: "${rs[0].text}"`);

  // 2: an owned card's chip, its sheet, `insert` after the row is dropped
  await page.evaluate(() => window.__riddle.buy("thief_guard"));
  await sleep(800);
  rs = await rows();
  check(rs.length === 3 && rs[1].card && /thief guard/.test(rs[1].text), `the bought card sits at R2: ${rs.map((r) => r.text.slice(0, 20)).join(" | ")}`);
  const chip = page.locator(".unlocks .chip.owned", { hasText: "thief guard" }).first();
  const chipText = (await chip.innerText()).replace(/\s+/g, " ").trim();
  check(chipText === "card: thief guard · owned", `the owned chip reads owned: "${chipText}"`);
  await chip.click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "card: thief guard" && !s.buttons.includes("insert") && !s.buttons.includes("buy"), `the owned sheet is titled, no insert while the set holds the row: [${s?.buttons.join(", ")}]`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.locator(".editor .row").nth(1).locator(".x").click({ timeout: 5000 }); await sleep(400);
  rs = await rows();
  check(rs.length === 2 && !rs.some((r) => r.card), "the card row dropped");
  await page.locator(".unlocks .chip.owned", { hasText: "thief guard" }).first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "card: thief guard" && s.buttons.includes("insert"), `the owned sheet offers insert once the row is gone: [${s?.buttons.join(", ")}]`);
  await page.locator(".sheet-wrap button", { hasText: "insert" }).first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "camp", "camp after insert");
  await sleep(500);
  rs = await rows();
  // where a buy puts it: before the engagement row (R2 `attack`), never the end the stale catalogue would say
  check((await sheets()) === 0 && rs.length === 3 && rs[1].card && /thief guard/.test(rs[1].text) && /attack/.test(rs[2].text), `insert put the card back above attack: ${rs.map((r) => r.text.slice(0, 20)).join(" | ")}`);
  check((await count()) === "2/4 · 1 card", `the counter: "${await count()}"`);

  // 1: a screen change closes every sheet; Escape on a kept death goes to the camp, on a fresh death it stays
  await page.locator("button.gear").click({ timeout: 5000 }); await sleep(150);
  await page.locator("button.mini", { hasText: "forge" }).first().click({ timeout: 5000 }).catch(() => {});   // under the settings sheet: the backdrop takes it
  await fakeDeath(false);
  await waitFor((s) => s?.screen === "death", "death");
  check((await sheets()) === 0, "the settings sheet closed with the screen change");
  await page.locator("button.btn", { hasText: "morgue" }).first().click({ timeout: 5000 }); await sleep(150);
  check((await sheets()) === 1, "the morgue sheet is up");
  await page.keyboard.press("Escape"); await sleep(150);
  check((await sheets()) === 0 && (await state()).screen === "death", "Escape closed the morgue; a fresh death stays");
  await page.keyboard.press("Escape"); await sleep(150);
  check((await state()).screen === "death", "Escape again: a fresh death still stays (edit is the way out)");
  await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
  await waitFor((s) => s?.screen === "camp", "camp");
  await sleep(300);
  // the chronicle's ▸ on a kept death: the sheet closes, the death mounts, Escape leads to the camp
  check(await setLineage(["♟1 the curious fighter · D3 · fell to a jackal · left bones on D3."], true), "the lineage took a chronicle line whose heir keeps its death");
  await waitFor((s) => s?.screen === "camp", "camp again");
  await sleep(400);
  await page.locator("button.mini", { hasText: "chronicle" }).first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "chronicle" && s.buttons.length === 1 && /▸/.test(s.buttons[0]), `the chronicle line is a button: [${s?.buttons.join(", ")}]`);
  await page.locator(".sheet-wrap button.cline.kept").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "death", "the kept death");
  await sleep(200);
  check((await sheets()) === 0, "the chronicle sheet closed under the death screen");
  check((await page.locator("main.death button.btn.primary", { hasText: "edit" }).count()) === 1, "the kept death offers edit (the camp)");
  await page.keyboard.press("Escape");
  await waitFor((s) => s?.screen === "camp", "camp after Escape on the kept death", 5000);
  check(true, "Escape on the kept death leads back to the camp");
  await sleep(600);

  // 10: the yours line, with the card above attack, carries no `card R2 first`
  const yours = await page.evaluate(() => document.querySelector(".fc-yours")?.textContent ?? "");
  rs = await rows();
  check(rs.some((r) => r.card) && /^yours: \d+ of \d+ rows/.test(yours) && !/card R\d+ first/.test(yours), `no card-first suffix with a card at R2: "${yours}"`);

  // 9: a gated unlock's sheet has buy off — the engine's `needs` (caster: boss 2), and a card that lies `available` while the
  // marks are short (the stale catalogue of a card painted before a buy)
  await page.locator(".unlocks .card", { hasText: "class: caster" }).first().click({ timeout: 5000 }); await sleep(200);
  let gate = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap button.buy"); return { disabled: b?.disabled, off: b?.classList.contains("off"), needs: document.querySelector(".sheet-wrap .needs-line")?.textContent.trim() }; });
  check(gate.disabled === true && gate.off && /^⊘ /.test(gate.needs ?? ""), `a gated unlock's buy is off: "${gate.needs}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.evaluate(() => {
    const e = window.__riddle.engine; const real = e.unlocks.bind(e);
    e.unlocks = async () => (await real()).map((u) => (u.id === "ranger" ? { ...u, cost: 40, available: true, needs: undefined } : u));
    e.unlockDeltas = () => Promise.reject(new Error("no deltas"));
    window.__riddle.go({ kind: "camp" });
  });
  await sleep(800);
  await page.locator(".unlocks .card", { hasText: "class: ranger" }).first().click({ timeout: 5000 }); await sleep(200);
  gate = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap button.buy"); return { disabled: b?.disabled, off: b?.classList.contains("off"), needs: document.querySelector(".sheet-wrap .needs-line")?.textContent.trim(), marks: window.__riddle.lineage.marks }; });
  check(gate.disabled === true && gate.off && gate.needs === `⊘ ◆${40 - gate.marks} more`, `a stale available with the marks short (◆${gate.marks} of 40): buy off, "${gate.needs}"`);
  await page.keyboard.press("Escape"); await sleep(150);

  // 8: the shop and the shelf are up while the forecast computes — the worker answers in order, so the engine is wrapped to answer
  // in order with a slow forecast; the caches are cleared so only the fetch order can keep them painted
  await page.evaluate(() => {
    const r = window.__riddle; const e = r.engine; let q = Promise.resolve();
    const ser = (m, ms) => { const f = e[m].bind(e); e[m] = (...a) => { const p = q.then(() => new Promise((res) => setTimeout(res, ms))).then(() => f(...a)); q = p.catch(() => {}); return p; }; };
    ser("forecast", 2500); ser("unlocks", 0); ser("supplyCatalogue", 0);
    r.unlockCat = []; r.supplyCat = [];
    r.go({ kind: "camp" });
  });
  await sleep(700);
  let during = await page.evaluate(() => ({ busy: window.__riddle.engineBusy, fc: document.querySelector(".fc-bars .d")?.textContent, shop: document.querySelectorAll(".supplies .chip.buy").length, cards: document.querySelectorAll(".unlocks .card").length }));
  check(during.busy && during.fc === "…" && during.shop > 0 && during.cards > 0, `while the forecast computes (${during.fc}): ${during.shop} shop chips, ${during.cards} unlock cards`);
  // and from the caches alone (the fetches queued behind a forecast in flight): a repaint mid-forecast keeps them
  await page.evaluate(() => { const r = window.__riddle; r.engine.unlocks = () => new Promise(() => {}); r.engine.supplyCatalogue = () => new Promise(() => {}); r.go({ kind: "camp" }); });
  await sleep(300);
  during = await page.evaluate(() => ({ shop: document.querySelectorAll(".supplies .chip.buy").length, cards: document.querySelectorAll(".unlocks .card").length }));
  check(during.shop > 0 && during.cards > 0, `from the last catalogue with the fetches hung: ${during.shop} shop chips, ${during.cards} unlock cards`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`qa9: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`qa9: ok (${out.length} checks)`);
