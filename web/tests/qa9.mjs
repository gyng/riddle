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
// QA lapses on build e0f87e7 (scratchpad/qaC, qaD):
//  11  the report's `· N more` is a button that expands to every exit line; runs without a line read `· N unlisted` (dim,
//      inert); a recovered bones pile reads `found ♟3's bones · D8 · 11 items` (never a pile still lying there)
//  12  the watch mode chosen (`fast`) is remembered: the save blob carries it and the next watch starts in it
//  13  the exit sheet's counter is `keep 0/1` (picks against free slots, not the camp's `vault 1/2`); a tap past the free
//      slots swaps the oldest pick out
//  14  SALVAGED agrees with the gold sheet: the report's rows sum to the ledger's salvage lines of that exit — with the sheet
//      (the kept item absent from the rows) and with the vault full (no sheet; the rows are still built)
//  15  in `fights` the interstitial names the HUD's floor whenever both show (the card repaints as each step lands)
//
//   node web/tests/qa9.mjs        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows, openPanel } from "./lib/frame.mjs";

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
  return { label: w.querySelector(".label")?.textContent.trim() ?? "", buttons: [...w.querySelectorAll("button:not(.close-stud)")].map((b) => b.textContent.trim()), text: w.innerText.replace(/\s+/g, " ").trim(), first: w.querySelector(".sheet-body")?.firstElementChild?.outerHTML.slice(0, 60) ?? "" };
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
  await editRows(page);   // Cut 17: the tablets carry their chips, ▲▼ and × (the `edit` tile, remembered)
  check(await setLineage([], false), "the lineage took marks, gold, the thief fact and an empty chronicle");
  await waitFor((s) => s?.screen === "camp", "camp again");
  await page.waitForFunction(() => window.__riddle.unlockCat.length > 0, null, { timeout: 10_000 });
  // Cut 17: the unlock panel lists the whole catalogue from here on (`more`, remembered); the panel closes over the tablets
  await openPanel(page, "unlocks", { all: true }); await page.keyboard.press("Escape");
  await sleep(600);

  // 4 · 5 · 6: sheet titles, the empty chronicle, the seed, the empty vault slot
  await page.locator(".cmd .tile[data-tile=chronicle]").first().click({ timeout: 5000 }); await sleep(200);
  let s = await sheet();
  check(s?.label === "chronicle" && !/·/.test(s.text.replace("chronicle", "")) && s.buttons.length === 0, `an empty chronicle shows its label and nothing else: "${s?.text}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.locator(".cmd .tile[data-tile=ledger]").first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "ledger" && /seen/i.test(s.text), `the ledger sheet is titled: "${s?.label}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.locator(".cmd .tile[data-tile=forge]").first().click({ timeout: 5000 }); await sleep(200);
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
  await openPanel(page, "unlocks");
  const rowCard = page.locator(".unlocks .card", { hasText: "+1 row" }).first();
  const cardText = (await rowCard.innerText()).replace(/\s+/g, " ");
  check(/⊘ fill rows/.test(cardText) && !/rows full/.test(cardText), `the +1 row card reads a requirement: "${cardText}"`);
  await rowCard.click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(/⊘ fill rows/.test(s?.text ?? "") && !/rows full/.test(s?.text ?? ""), `so does its sheet: "${s?.text}"`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.keyboard.press("Escape"); await sleep(150);   // the panel

  // 7: the cond picker's × leads the sheet; it removes the cond
  await page.locator(".editor .row").first().locator(".chip.cond").first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "cond" && s.buttons[0] === "×", `the cond picker's × is its first control (under the sheet's title): ${s?.first} · [${s?.buttons[0]}]`);   // Cut 13 §6: every sheet is titled
  await page.locator(".sheet-wrap .sheet-body > button.btn.ghost.wide").first().click({ timeout: 5000 }); await sleep(300);
  let rs = await rows();
  check((await sheets()) === 0 && rs[0].conds === 0, `× removed R1's cond: "${rs[0].text}"`);

  // 2: an owned card's chip, its sheet, `insert` after the row is dropped
  await page.evaluate(() => window.__riddle.buy("thief_guard"));
  await sleep(800);
  rs = await rows();
  check(rs.length === 3 && rs[1].card && /thief guard/.test(rs[1].text), `the bought card sits at R2: ${rs.map((r) => r.text.slice(0, 20)).join(" | ")}`);
  await openPanel(page, "unlocks");
  const chip = page.locator(".unlocks .chip.owned", { hasText: "thief guard" }).first();
  const chipText = (await chip.innerText()).replace(/\s+/g, " ").trim();
  check(chipText === "card: thief guard · owned", `the owned chip reads owned: "${chipText}"`);
  await chip.click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "card: thief guard" && !s.buttons.includes("insert") && !s.buttons.includes("buy"), `the owned sheet is titled, no insert while the set holds the row: [${s?.buttons.join(", ")}]`);
  await page.keyboard.press("Escape"); await sleep(150);
  await page.keyboard.press("Escape"); await sleep(150);   // the panel
  await page.locator(".editor .row").nth(1).locator(".x").click({ timeout: 5000 }); await sleep(400);
  rs = await rows();
  check(rs.length === 2 && !rs.some((r) => r.card), "the card row dropped");
  await openPanel(page, "unlocks");
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
  await page.locator(".cmd .tile[data-tile=forge]").first().click({ timeout: 5000 }).catch(() => {});   // under the settings sheet: the backdrop takes it
  await fakeDeath(false);
  await waitFor((s) => s?.screen === "death", "death");
  check((await sheets()) === 0, "the settings sheet closed with the screen change");
  await page.locator("main.death button", { hasText: /^morgue$/ }).first().click({ timeout: 5000 }); await sleep(150);
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
  await page.locator(".cmd .tile[data-tile=chronicle]").first().click({ timeout: 5000 }); await sleep(200);
  s = await sheet();
  check(s?.label === "chronicle" && s.buttons.length === 1 && /▸/.test(s.buttons[0]), `the chronicle line is a button: [${s?.buttons.join(", ")}]`);
  await page.locator(".sheet-wrap button.cline.kept").first().click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "death", "the kept death");
  await sleep(200);
  check((await sheets()) === 0, "the chronicle sheet closed under the death screen");
  check((await page.locator("main.death button", { hasText: /^edit$/ }).count()) === 1, "the kept death offers edit (the camp)");
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
  await openPanel(page, "unlocks");
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
  await openPanel(page, "unlocks");
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

  // 11: `· N more` expands the exit lines; `· N unlisted` names the runs with no line; bones piles read as found
  await page.evaluate(() => {
    const r = window.__riddle; const L = r.lineage;
    const exits = Array.from({ length: 12 }, (_, i) => ({ carried: 10 + i, keep_pct: 60, kept: 6 + i, spent: 0, spent_on: [], text: `returned $${6 + i} · $${10 + i} carried · keeps 60%` }));
    r.go({ kind: "report", report: { elapsed_s: 3600, runs: 15, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 15, exits, bones_found: ["heir 3 · D8 · 11 items", "D5 · 7 items", "bones:6:4"] }, absence: true });
  });
  await sleep(300);
  const exitsDom = () => page.evaluate(() => ({
    lines: document.querySelectorAll(".report .exit-lines .ledger-line button.ledger-btn").length,
    more: document.querySelector(".report .exit-lines button.ledger-more")?.textContent ?? null,
    unlisted: [...document.querySelectorAll(".report .exit-lines .ledger-line.unlisted")].map((e) => ({ text: e.textContent, tag: e.tagName, dim: e.classList.contains("dim") })),
    bones: [...document.querySelectorAll(".report .rsec")].find((s) => s.querySelector(".label")?.textContent === "bones")?.querySelectorAll("li") ?? [],
  }));
  let ex = await exitsDom();
  check(ex.lines === 8 && ex.more === "· 4 earlier", `12 exits over 15 runs: 8 lines and an earlier button (${ex.lines} lines, "${ex.more}")`);
  check(ex.unlisted.length === 1 && ex.unlisted[0].text === "· 3 unlisted" && ex.unlisted[0].tag === "DIV" && ex.unlisted[0].dim, `the runs with no line: "${ex.unlisted[0]?.text}" (${ex.unlisted[0]?.tag}, dim ${ex.unlisted[0]?.dim})`);
  await page.locator(".report .exit-lines button.ledger-more").click({ timeout: 5000 }); await sleep(200);
  ex = await exitsDom();
  check(ex.lines === 12 && ex.more === null && ex.unlisted.length === 1, `more expands to every line, the unlisted count stays (${ex.lines} lines, more ${ex.more}, ${ex.unlisted.length} unlisted)`);
  const bones = await page.evaluate(() => [...document.querySelectorAll(".report .rsec")].filter((s) => s.querySelector(".label")?.textContent === "bones").flatMap((s) => [...s.querySelectorAll("li")].map((l) => l.textContent)));
  check(bones.length === 3 && bones[0] === "found ♟3's bones · D8 · 11 items" && bones[1] === "found bones · D5 · 7 items" && bones[2] === "found bones · D6 · 4 items", `bones piles read as found: ${JSON.stringify(bones)}`);
  // no line without a line: 8 exits over 8 runs show neither `more` nor `unlisted`
  await page.evaluate(() => {
    const r = window.__riddle; const L = r.lineage;
    const exits = Array.from({ length: 8 }, (_, i) => ({ carried: 10 + i, keep_pct: 60, kept: 6 + i, spent: 0, spent_on: [], text: `returned $${6 + i} · $${10 + i} carried · keeps 60%` }));
    r.go({ kind: "report", report: { elapsed_s: 3600, runs: 8, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 8, exits } });
  });
  await sleep(200);
  ex = await exitsDom();
  check(ex.lines === 8 && ex.more === null && ex.unlisted.length === 0, `8 exits over 8 runs: no more, no unlisted (${ex.lines} lines)`);

  // 12 · 13 · 14: a fake run that returns with items (seed 13, return at D3), watched in `fast`
  const rules = encodeURIComponent("depth>=3 → return\nfoes>=1 → attack nearest");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=13&rules=${rules}&autosend=1`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
  // 15: in `fights` the interstitial names the HUD's floor whenever both show (QA on e0f87e7: "`D1 · 16 rooms · $18` while the
  // HUD reads `32/40 D2`") — sampled through the drive below, ▶▶| pressed every 300 ms as the QA player did
  const cardSample = () => page.evaluate(() => { const c = document.querySelector(".interstitial"); const card = c && !c.hidden ? c.textContent : null; const hud = document.querySelector(".watch .depth")?.textContent ?? ""; return { card, hud, mismatch: !!card && !!hud && !card.startsWith(`${hud} `) }; });
  const cardSamples = [];
  const ledgerSalvage = () => page.evaluate(() => {
    const g = window.__riddle.lineage.gold_ledger ?? []; let i = g.length - 1;
    while (i >= 0 && !/^(returned|banked|died|lost|stalled)\b/.test(g[i].why)) i--;
    return { exit: g[i]?.why ?? null, salvage: g.slice(i + 1).filter((x) => /^salvage/.test(x.why)).reduce((a, x) => a + x.delta, 0) };
  });
  const reportSalvaged = () => page.evaluate(() => [...document.querySelectorAll(".report .rsec")].filter((s) => s.querySelector(".label")?.textContent === "salvaged").flatMap((s) => [...s.querySelectorAll("li")].map((l) => ({ text: l.textContent.replace(/\s+/g, " ").trim(), gold: Number(/\$(\d+)/.exec(l.textContent)?.[1] ?? 0) }))));
  // 12: `fast` is chosen during the run (a DOM click) once two cards have shown (Cut 15 §4: the card is ≤ 1.2 s, so one card and
  // four samples do too): the run's end kills the mode buttons
  // (QA on 50bb162: "fights · fast · ▶▶| · bail still live on a dead hero"), so the choice cannot come from the exit sheet
  let picked = false;
  const pickFast = () => page.evaluate(() => { for (const b of document.querySelectorAll("main.watch .cmd .hud-btn")) if (b.textContent === "fast" && !b.disabled) b.click(); });
  const drive = async (sample = false) => { const t0 = Date.now(); while (Date.now() - t0 < 120_000) { const s = await state(); if (!s || s.screen !== "watch") return s; if (sample) cardSamples.push(await cardSample()); if (sample && !picked && (cardSamples.filter((c) => c.card).length >= 2 || (cardSamples.some((c) => c.card) && cardSamples.length >= 4))) { picked = true; await pickFast(); } await page.locator(".cmd .hud-btn", { hasText: "▶▶|" }).click({ timeout: 1000 }).catch(() => {}); await sleep(300); } return state(); };
  let s2 = await drive(true);
  check(s2?.screen === "exit", `the run ended on the keep sheet (${s2?.screen})`);
  const shownCards = cardSamples.filter((c) => c.card), mism = cardSamples.filter((c) => c.mismatch);
  check(shownCards.length > 0 && mism.length === 0, `the card named the HUD's floor in every sample it showed (${shownCards.length} samples${mism.length ? `; off: ${mism.map((m) => `${m.card} | ${m.hud}`).join(", ")}` : ""})`);
  // 12: the `fast` chosen mid-run is the next run's mode; at the exit the mode buttons are dead
  if (!picked) await pickFast();   // the run ended before three cards showed: the click then lands on a dead button (the check says so)
  const saved = await page.evaluate(() => ({ mode: document.querySelector("main.watch")?.dataset.mode, blob: JSON.parse(window.__riddle.exportSave()).watch, dead: [...document.querySelectorAll("main.watch .cmd .hud-btn")].every((b) => b.disabled) }));
  check(saved.mode === "fast" && saved.blob === "fast", `fast chosen mid-run: the watch is in it and the save blob carries it (${saved.mode}, ${saved.blob}, picked ${picked})`);
  check(saved.dead, "at the exit the mode buttons, ▶▶| and bail are dead");
  const keepSheet = () => page.evaluate(() => { const w = [...document.querySelectorAll(".sheet-wrap")].pop(); const lab = w?.querySelector(".label.row-label"); return { label: lab?.firstChild?.textContent?.trim() ?? "", count: lab?.querySelector(".num")?.textContent ?? "", on: [...(w?.querySelectorAll(".chips .chip.item") ?? [])].map((c) => c.classList.contains("on")), n: w?.querySelectorAll(".chips .chip.item").length ?? 0 }; });
  let ks = await keepSheet();
  check(ks.label === "keep" && ks.count === "0/1" && ks.n >= 3, `the sheet counts picks against free slots as keep: "${ks.label} ${ks.count}" over ${ks.n} chips`);
  // QA 23ed91f (K: "`$5` on each item: a cost to keep, or a sale price?"): the sheet says the prices are what the unkept sell for
  const legend = await page.evaluate(() => document.querySelector(".sheet-wrap .keep-legend")?.textContent ?? null);
  check(legend === "unkept → salvage", `the keep sheet names its prices: "${legend}"`);
  const kchip = (i) => page.locator(".sheet-wrap .chips .chip.item").nth(i);
  await kchip(0).click({ timeout: 5000 }); await sleep(100); ks = await keepSheet();
  check(ks.count === "1/1" && ks.on[0] && !ks.on[1], `one pick: "${ks.count}", chip 1 on`);
  await kchip(1).click({ timeout: 5000 }); await sleep(100); ks = await keepSheet();
  check(ks.count === "1/1" && !ks.on[0] && ks.on[1], `a tap past the free slots swaps the oldest pick out: "${ks.count}", chip 2 on, chip 1 off`);
  await kchip(1).click({ timeout: 5000 }); await sleep(100); ks = await keepSheet();
  check(ks.count === "0/1" && !ks.on.some(Boolean), `a tap on a pick lets it go: "${ks.count}"`);
  const keptKind = await page.evaluate(() => document.querySelector(".sheet-wrap .chips .chip.item")?.textContent.replace(/\s+\$\d+$/, "").trim().split(" ")[0]);
  await kchip(0).click({ timeout: 5000 }); await sleep(100);
  await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 });
  s2 = await waitFor((x) => x && x.screen === "report" && !x.busy, "the report after keep", 30_000);
  await sleep(300);
  let ls = await ledgerSalvage(), sv = await reportSalvaged();
  let sum = sv.reduce((a, r) => a + r.gold, 0);
  check(ls.salvage > 0 && sv.length > 0 && sum === ls.salvage, `with the sheet: SALVAGED sums to the ledger's salvage after "${ls.exit}" ($${sum} vs $${ls.salvage}): ${sv.map((r) => r.text).join(" · ")}`);
  check(keptKind && !sv.some((r) => r.text.startsWith(keptKind)), `the kept ${keptKind} is not among the salvaged rows`);
  const vaultN = await page.evaluate(() => window.__riddle.lineage.vault.length);
  check(vaultN === 1, `the vault holds the kept item (${vaultN}/1: full)`);
  // 12: the next watch starts in `fast`; 14: with the vault full the sheet is skipped and SALVAGED is still built
  await page.evaluate(() => window.__riddle.go({ kind: "watch" }));
  await waitFor((s) => s?.screen === "watch", "the second watch");
  const mode2 = await page.evaluate(() => ({ mode: document.querySelector("main.watch")?.dataset.mode, on: [...document.querySelectorAll(".cmd .hud-btn.on")].map((b) => b.textContent) }));
  check(mode2.mode === "fast" && mode2.on.join() === "fast", `the next watch starts in the remembered mode (${mode2.mode}, on: ${mode2.on.join()})`);
  // the fake's worths equal the client's table, so the ledger is skewed through the lineage the client refreshes at the exit:
  // one more `+$9 salvage` line at the exit's tick — the rows must move to it (the largest row takes the difference)
  await page.evaluate(() => { const e = window.__riddle.engine; const real = e.lineage.bind(e); e.lineage = async () => { const L = await real(); const g = L.gold_ledger ?? []; const last = g[g.length - 1]; if (last && /^salvage/.test(last.why) && !g.some((x) => x.why === "salvage")) g.push({ t: last.t, delta: 9, why: "salvage" }); return L; }; });
  s2 = await drive();
  check(s2?.screen === "report", `the vault full: the run went straight to the report (${s2?.screen})`);
  await waitFor((x) => x && !x.busy, "the engine idle", 30_000); await sleep(300);
  ls = await ledgerSalvage(); sv = await reportSalvaged(); sum = sv.reduce((a, r) => a + r.gold, 0);
  check(ls.salvage > 0 && sv.length > 0 && sum === ls.salvage, `vault full, no sheet: SALVAGED sums to the ledger's salvage after "${ls.exit}" ($${sum} vs $${ls.salvage}): ${sv.map((r) => r.text).join(" · ")}`);
  // seed 13's second run lets go teleport ×2 ($14 at the table) · sword $12 · aggravate $7 · enchant $7 · fire $5: the +$9 lands on the teleports
  const top = [...sv].sort((a, b) => b.gold - a.gold)[0];
  check(top?.text === "teleport ×2 · $23" && sv.find((r) => r.text.startsWith("sword"))?.gold === 12, `the skew (+$9) landed on the largest row alone: ${sv.map((r) => r.text).join(" · ")}`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`qa9: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`qa9: ok (${out.length} checks)`);
