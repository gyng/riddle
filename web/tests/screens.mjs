#!/usr/bin/env node
// Cut 13 §6 — the screen lint: the QA brief's itinerary walked on the fake engine (`?engine=fake&dev=1`, headless through
// tools/browser.mjs against the dev server, tools/dev.sh :5219) and, on EVERY screen and sheet it opens, the checks the four
// QA players reconciled by hand:
//   · the text holds no `undefined` / `NaN` / `[object`
//   · no line repeats a ` · `-separated segment (`returned $20 · returned $20`)
//   · every sheet opened carries a title (`.label` / `.sheet-head`)
//   · every visible enabled button on the screen, clicked, changes the screen (its text, its `on` marks, its sheets) — or is a
//     documented toggle (mute · fights · fast · ⏸ · ▶▶| · bail · ▲ · ▼ · the drag grip · an already-selected chip); the buttons
//     that lead somewhere (send · keep · patch · edit · camp · open · buy · insert · ok) are walked by the itinerary itself
//   · the camp header's `$` equals the gold sheet's total
// Itinerary: fresh → camp (its sheets) → send (fights) → ▶▶| → exit/keep → death (patch) | report (camp) → camp → edit two
// rows → buy the first affordable unlock and open its sheet → drop a supply → `?absent=8h` → report (open the worst death).
// Cut 14 §4: the report's trace chips carry their exit (`D5 · died · trace`); the stalled tile carries its cost (`2 STALLED · $201 lost`).
//
//   node web/tests/screens.mjs [--verbose]        (part of `pnpm test` in web/; ~60 s)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const verbose = process.argv.includes("--verbose");
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0, lints = 0, clicks = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const note = (what) => { if (verbose) out.push(`     ${what}`); };

// buttons the itinerary walks (they lead to another screen) and the documented toggles (a click need not change the text)
const NAV = new Set(["send", "keep", "edit", "camp", "open", "buy", "insert", "ok", "import", "export", "reset", "again", "trace", "watch"]);
const TOGGLES = new Set(["mute", "fights", "fast", "⏸", "▶", "▶▶|", "bail", "▲", "▼", "≡"]);
const INERT_SEL = ".grip, .interstitial, .prefs .chip.on, .tabs .tab.on, .classes .chip.on, .chip.trait.on, button.patch, .cline.kept, .chip.mini.trace, .bar.try";

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, sheets: document.querySelectorAll(".sheet-wrap").length, vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state();
    if (s?.screen === "exit" && s.vault) { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(100); continue; }
    if (pred(s)) return s;
    await sleep(80);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted} busy=${s?.busy})`);
}
/** Wait until the progress bar has been down for two samples, then a paint. */
async function settle(max = 15_000) {
  const t = Date.now(); let clear = 0;
  while (Date.now() - t < max) { const s = await state(); clear = s && s.booted && !s.busy ? clear + 1 : 0; if (clear >= 2) break; await sleep(120); }
  await sleep(120);
}
const closeSheets = async () => { for (let i = 0; i < 6 && (await state())?.sheets; i++) { await page.keyboard.press("Escape"); await sleep(120); } };

/** The text lint on a string: no undefined / NaN / [object; no repeated ` · ` segment on a line. Returns the lapses. */
function lintString(text) {
  const bad = [];
  for (const m of text.match(/\bundefined\b|\bNaN\b|\[object/g) ?? []) bad.push(`"${m}" in the text`);
  for (const raw of text.split("\n")) {
    const line = raw.trim(); if (!line.includes(" · ")) continue;
    const segs = line.split(" · ").map((s) => s.trim()).filter((s) => s.length > 1);
    const seen = new Set();
    for (const s of segs) { if (seen.has(s)) { bad.push(`repeated segment "${s}" in "${line.slice(0, 80)}"`); break; } seen.add(s); }
  }
  return bad;
}
/** Lint the screen's text (the top sheet's, when one is up). */
async function lintScreen(where) {
  lints++;
  const text = await page.evaluate(() => document.body.innerText);
  const bad = lintString(text);
  check(bad.length === 0, `${where}: text lint${bad.length ? ` — ${bad.slice(0, 3).join("; ")}` : ""}`);
}
/** Lint the top sheet: a title and its text. */
async function lintSheet(where) {
  const s = await page.evaluate(() => {
    const w = [...document.querySelectorAll(".sheet-wrap")].pop(); if (!w) return null;
    const title = w.querySelector(".label, .sheet-head");
    return { title: title?.textContent.trim() ?? "", text: w.innerText };
  });
  if (!s) { check(false, `${where}: no sheet opened`); return; }
  const bad = lintString(s.text);
  check(!!s.title && bad.length === 0, `${where}: sheet titled "${s.title.slice(0, 24)}"${bad.length ? ` — ${bad.slice(0, 3).join("; ")}` : ""}`);
}
/** A hash of what the screen shows: its text, which chips/tabs are on, hidden marks, open sheets. */
const screenHash = () => page.evaluate(() => {
  const marks = [...document.querySelectorAll(".on, [aria-pressed], [hidden], .stale, .drop, .over")].map((e) => `${e.tagName}.${e.className}:${(e.textContent ?? "").slice(0, 20)}`).join("|");
  const s = `${document.body.innerText}\n${marks}\n${document.querySelectorAll(".sheet-wrap").length}`;
  let h = 2166136261; for (let i = 0; i < s.length; i++) { h ^= s.charCodeAt(i); h = Math.imul(h, 16777619); } return h >>> 0;
});
/** The screen's candidate buttons, as descriptors the click step can find again after a repaint. */
const buttonsOf = (inertSel) => page.evaluate((inertSel) => {
  const all = [...document.querySelectorAll("main button")].filter((b) => !b.disabled && !b.closest(".sheet-wrap") && !b.matches(inertSel) && b.getClientRects().length && getComputedStyle(b).visibility !== "hidden");
  const key = (b) => `${b.className}::${(b.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 40)}`;
  const counts = new Map();
  return all.map((b) => { const k = key(b); const n = counts.get(k) ?? 0; counts.set(k, n + 1); return { key: k, nth: n, text: (b.textContent ?? "").replace(/\s+/g, " ").trim(), cls: b.className }; });
}, inertSel);
const clickButton = (d) => page.evaluate(({ key, nth }) => {
  const all = [...document.querySelectorAll("main button")].filter((b) => !b.closest(".sheet-wrap"));
  const k = (b) => `${b.className}::${(b.textContent ?? "").replace(/\s+/g, " ").trim().slice(0, 40)}`;
  const hits = all.filter((b) => k(b) === key);
  const b = hits[nth]; if (!b || b.disabled) return false;
  b.scrollIntoView({ block: "center" }); b.click(); return true;
}, d);
/** Cut 13 §6: every visible enabled button on the screen changes it when clicked (or is a documented toggle); a sheet it opens
 *  is linted and closed; a screen it leaves is noted and the walk returns to it. */
async function lintButtons(where, expectScreen) {
  const list = await buttonsOf(INERT_SEL);
  const inert = [];
  let tried = 0;
  for (const d of list) {
    if (NAV.has(d.text) || TOGGLES.has(d.text)) continue;
    const before = await screenHash();
    if (!(await clickButton(d))) continue;   // gone after an earlier click (a removed row, a bought card)
    tried++; clicks++;
    await sleep(280);
    const s = await state();
    if (s.sheets) await lintSheet(`${where} · ${d.text || d.cls}`);
    const after = await screenHash();
    if (before === after) inert.push(`${d.text || d.cls}`);
    await closeSheets();
    if ((await state())?.screen !== expectScreen) {
      note(`${where}: "${d.text}" led to ${(await state())?.screen}; back`);
      await page.evaluate((k) => window.__riddle.go({ kind: k }), expectScreen);
      await waitFor((x) => x?.screen === expectScreen, `${expectScreen} again`); await settle();
    }
  }
  check(inert.length === 0, `${where}: ${tried} buttons each change the screen${inert.length ? ` — inert: ${inert.slice(0, 6).join(", ")}` : ""}`);
}
const openAndLint = async (sel, label, hasText) => {
  const loc = hasText ? page.locator(sel, { hasText }).first() : page.locator(sel).first();
  if (!(await loc.count())) { check(false, `${label}: nothing to open (${sel})`); return false; }
  await loc.click({ timeout: 5000 }); await sleep(250);
  await lintSheet(label);
  return true;
};
const rowsText = () => page.evaluate(() => [...document.querySelectorAll(".editor .row .chips")].map((c) => c.innerText.replace(/\s+/g, " ").trim()));

const t0 = Date.now();
try {
  // 1. fresh camp (marks and gold so the shelf and the shop have something to sell)
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  const seeded = await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    e.lineage.marks = 12; e.lineage.gold = 300; e.lineage.facts.push("item:red=heal");
    b.engine = JSON.stringify(e);
    return r.importSave(JSON.stringify(b));
  });
  check(seeded, "the lineage took marks, gold and an identified heal");
  await waitFor((s) => s?.screen === "camp", "camp again");
  await page.waitForFunction(() => window.__riddle.unlockCat.length > 0 && document.querySelectorAll(".supplies .chip.buy").length > 0, null, { timeout: 15_000 });
  await settle();
  await lintScreen("camp");
  // 2. the camp's sheets
  for (const [sel, label, has] of [["button.mini", "chronicle", "chronicle"], ["button.mini", "ledger", "ledger"], ["button.mini", "forge", "forge"], ["button.gear", "settings"], ["button.cls", "class"], [".strip button.gold", "gold"], [".editor .row .chip.cond", "cond picker"], [".editor .row .chip.verb", "verb picker"], [".unlocks .card", "unlock card"], [".tabs .tab.edit", "rename"]]) {
    if (label === "gold") {
      await openAndLint(sel, label, has);
      const g = await page.evaluate(() => ({ header: document.querySelector(".strip button.gold")?.textContent, sheet: document.querySelector(".sheet-wrap .label .gold")?.textContent, lineage: `$${window.__riddle.lineage.gold}` }));
      check(g.header === g.sheet && g.sheet === g.lineage, `the header's ${g.header} equals the gold sheet's ${g.sheet}`);
    } else await openAndLint(sel, label, has);
    await closeSheets();
  }
  // 3. every button on the camp
  await lintButtons("camp", "camp");
  // 4. send → fights → ▶▶| to the exit (the run's text linted twice on the way)
  await page.locator("button.send").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "watch", "the watch");
  await sleep(1500); await lintScreen("watch");
  let s = await state(); const tw = Date.now(); let presses = 0;
  while (s?.screen === "watch" && Date.now() - tw < 90_000) {
    await page.evaluate(() => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === "▶▶|") b.click(); }); presses++;
    await sleep(250); s = await waitFor((x) => x, "state");
    if (presses === 8) await lintScreen("watch (later)");
  }
  check(s?.screen !== "watch", `▶▶| reached the run's end (${presses} presses, ${s?.screen})`);
  // 5. exit/keep
  if (s.screen === "exit") {
    await lintSheet("keep sheet");
    await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 });
    s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after keep", 30_000);
  }
  await settle();
  // 6. death (patch) | report (camp)
  if (s.screen === "death") {
    await lintScreen("death");
    await lintButtons("death", "death");
    if (await page.locator("button.patch").count()) { await page.locator("button.patch").first().click({ timeout: 5000 }); await waitFor((x) => x?.screen === "camp", "camp after the patch"); }
    else { await page.locator("main.death button.btn.primary", { hasText: "edit" }).click({ timeout: 5000 }); await waitFor((x) => x?.screen === "camp", "camp after edit"); }
  } else if (s.screen === "report") {
    await lintScreen("report (a run)");
    await lintButtons("report", "report");
    await page.locator("main.report button.btn.primary", { hasText: "camp" }).click({ timeout: 5000 }); await waitFor((x) => x?.screen === "camp", "camp after the report");
  } else check(false, `an unexpected screen after the run: ${s.screen}`);
  await settle();
  await lintScreen("camp (after the run)");
  // 7. edit two rows: R1's first cond through the picker, R2's verb through the picker
  const before = await rowsText();
  if (!(await page.locator(".editor .row").count())) { await page.locator(".rows-foot .btn.ghost").click({ timeout: 5000 }); await sleep(200); }
  if ((await page.locator(".editor .row").count()) < 2) { await page.locator(".rows-foot .btn.ghost").click({ timeout: 5000 }); await sleep(200); }
  await page.locator(".editor .row").first().locator(".chip.cond").first().click({ timeout: 5000 }); await sleep(200);
  await lintSheet("cond picker (edit)");
  await page.locator(".sheet-wrap .grid .chip.cond:not(.on):not(.locked)").first().click({ timeout: 5000 }); await sleep(200);
  if ((await state())?.sheets) { await lintSheet("number picker"); await page.locator(".sheet-wrap .grid.nums .chip").nth(1).click({ timeout: 5000 }); await sleep(200); }
  await page.locator(".editor .row:not(.locked)").nth(1).locator(".chip.verb").first().click({ timeout: 5000 }); await sleep(200);
  await lintSheet("verb picker (edit)");
  await page.locator(".sheet-wrap .grid .chip.verb:not(.on)").first().click({ timeout: 5000 }); await sleep(300);
  const after = await rowsText();
  check((await state())?.sheets === 0 && after[0] !== before[0] && after[1] !== before[1], `two rows edited through the pickers: "${after[0]}" · "${after[1]}"`);
  await settle(); await lintScreen("camp (edited)");
  // 8. buy the first affordable unlock, open its sheet (the owned chip's, when it has rows)
  const card = page.locator(".unlocks .card:not(.off):not(.gated)").first();
  check(await card.count() > 0, "an affordable unlock is on the shelf");
  const bought = (await card.innerText()).split("\n")[0].trim();
  await card.click({ timeout: 5000 }); await sleep(200);
  await lintSheet(`unlock sheet · ${bought}`);
  // Cut 15 §2: both prices on the title (`◆3 · $450`) and two buys, `◆ buy` · `$ buy`
  const pair = await page.evaluate(() => ({ cost: document.querySelector(".sheet-wrap .unlock-sheet .cost")?.textContent.trim(), buys: [...document.querySelectorAll(".sheet-wrap .buy-pair button")].map((b) => b.textContent.trim()) }));
  check(/^◆\d+ · \$\d+$/.test(pair.cost ?? "") && pair.buys.join(" | ") === "◆ buy | $ buy", `the sheet shows both prices and two buys ("${pair.cost}": ${pair.buys.join(" | ")})`);
  await page.locator(".sheet-wrap button.buy.marks").click({ timeout: 5000 });
  await waitFor((x) => x?.sheets === 0, "the buy sheet closed"); await settle();
  const marksAfter = await page.evaluate(() => window.__riddle.lineage.marks);
  check(marksAfter < 12, `bought "${bought}" (marks ${marksAfter})`);
  await lintScreen("camp (bought)");
  // Cut 15 §2: a gold buy — the lineage gets $5000; the next card's `$ buy` spends gold, not marks, and the header's $ and ◆ read
  // the returned lineage; the next gold price climbs
  {
    await page.evaluate(async () => { const r = window.__riddle; const save = JSON.parse(await r.engine.save()); save.lineage.gold = 5000; r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" }); });
    await settle();
    const head = () => page.evaluate(() => ({ gold: document.querySelector(".strip .gold")?.textContent.trim(), marks: document.querySelector(".strip .marks")?.textContent.trim(), L: { gold: window.__riddle.lineage.gold, marks: window.__riddle.lineage.marks } }));
    const h0 = await head();
    const gcard = page.locator(".unlocks .card").filter({ hasNot: page.locator(".needs") }).first();
    const any = (await gcard.count()) ? gcard : page.locator(".unlocks .card").first();
    await any.click({ timeout: 5000 }); await sleep(200);
    const price = await page.evaluate(() => Number(/\$(\d+)/.exec(document.querySelector(".sheet-wrap .unlock-sheet .gold-price")?.textContent ?? "")?.[1] ?? NaN));
    const enabled = await page.evaluate(() => !document.querySelector(".sheet-wrap button.buy.gold")?.disabled);
    await lintSheet("unlock sheet (gold)");
    if (enabled) await page.locator(".sheet-wrap button.buy.gold").click({ timeout: 5000 });
    await waitFor((x) => x?.sheets === 0, "the gold buy sheet closed"); await settle();
    const h1 = await head();
    check(enabled && h1.L.gold === h0.L.gold - price && h1.L.marks === h0.L.marks && h1.gold === `$${h1.L.gold}` && h1.marks === `◆${h1.L.marks}`, `$ buy spent $${price}, not marks: header ${h0.gold} ${h0.marks} → ${h1.gold} ${h1.marks}`);
    const next = await page.evaluate(async () => (await window.__riddle.engine.unlocks()).find((u) => !u.owned && u.gold)?.gold ?? 0);
    const cost0 = await page.evaluate(async () => { const u = (await window.__riddle.engine.unlocks()).find((x) => !x.owned && x.gold); return u ? 150 * u.cost : 0; });
    check(next > cost0, `the next gold price climbs ($${next} > $${cost0} at the base rate)`);
  }
  if (await page.locator(".unlocks .chip.owned").count()) { await openAndLint(".unlocks .chip.owned", "owned sheet"); await closeSheets(); }
  // 9. drop a supply (the button lint may have dropped the leash already: buy one from the shop first then)
  if (!(await page.locator(".supplies .chip.item").count())) { await page.locator(".supplies .chip.buy:not(.off)").first().click({ timeout: 5000 }); await settle(); }
  const supplies = await page.locator(".supplies .chip.item").count();
  await page.locator(".supplies .chip.item .x").first().click({ timeout: 5000 }); await settle();
  check((await page.locator(".supplies .chip.item").count()) === supplies - 1, `a supply dropped (${supplies} → ${supplies - 1})`);
  await lintScreen("camp (supply dropped)");
  await page.evaluate(() => window.__riddle.flush());
  // 10. the absence: the report, its buttons, the worst death
  await page.goto(`${url}?dev=1&engine=fake&absent=8h`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && (x.screen === "report" || x.screen === "ending"), "the 8 h report", 120_000);
  await settle();
  await lintScreen("report (8 h)");
  if (s.screen === "report") {
    // Cut 14 §4: every exit's trace chip names its exit (`D5 · died · trace`; the depth off the ledger line the exit claims)
    // (the walk's picker edits can leave a set that returns on its first action — a traceless exit has no chip; step 11 checks
    // the label on fabricated lines whatever the walk did)
    const chips = await page.evaluate(() => [...document.querySelectorAll(".report .exit-lines .chip.mini")].map((c) => c.textContent.replace(/\s+/g, " ").trim()));
    const traced = await page.evaluate(() => (window.__riddle.view.report?.exits ?? []).some((x) => x.trace?.turns.length));
    const labelled = chips.filter((c) => /^D\d+ · (banked|returned|died) · trace$/.test(c)).length;
    if (traced || chips.length) check(chips.length > 0 && chips.every((c) => /^(D\d+ · )?(banked|returned|died) · trace$/.test(c)) && labelled * 2 >= chips.length, `the ${chips.length} trace chips carry their exit (${chips.slice(0, 3).join(" · ")}${labelled < chips.length ? ` · ${chips.length - labelled} without a depth` : ""})`);
    else note("report (8 h): no traced exit to label (the walk's set returns at once)");
    await lintButtons("report (8 h)", "report");
    if (await page.locator("main.report button.btn", { hasText: /^open$/ }).count()) {
      await page.locator("main.report button.btn", { hasText: /^open$/ }).click({ timeout: 5000 });
      await waitFor((x) => x?.screen === "death", "the worst death"); await settle();
      await lintScreen("death (worst)");
      await lintButtons("death (worst)", "death");
    } else note("no worst death to open");
  }
  // 11. Cut 14 §4: the stalled tile carries what the stalls cost — the stalled lines' `carried` (a fabricated report: the tile is
  // the code under test); the lines' trace chips read `returned · trace` without a ledger line to take a depth from
  {
    const turns = [{ t: 10, row: 0, verb: { v: "explore" }, hp: 20, foes: 0, telegraphs: [] }];
    const ex = (text, carried, kept, keep_pct) => ({ text, carried, kept, keep_pct, spent: 0, spent_on: [], trace: { turns } });
    await page.evaluate((exits) => { const r = window.__riddle; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: r.lineage.live ?? null, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 3, stalled: 2, exits } }); },
      [ex("returned $0 · $161 carried · keeps 0% · stalled", 161, 0, 0), ex("returned $30 · $50 carried · keeps 60%", 50, 30, 60), ex("returned $0 · $40 carried · keeps 0% · stalled", 40, 0, 0)]);
    await waitFor((x) => x?.screen === "report", "the fabricated report"); await sleep(200);
    const t = await page.evaluate(() => { const tile = [...document.querySelectorAll(".report .tile")].find((x) => x.querySelector(".label")?.textContent === "stalled"); return { tile: tile && [...tile.children].map((c) => c.textContent.trim()).join(" "), chips: [...document.querySelectorAll(".report .exit-lines .chip.mini")].map((c) => c.textContent.trim()) }; });
    check(t.tile === "2 stalled $201 lost", `the stalled tile carries its cost: "${t.tile}"`);
    // (a fabricated line can still claim a real ledger line of the same tier and sum, and then carries its depth)
    check(t.chips.length === 3 && t.chips.every((c) => /^(D\d+ · )?returned · trace$/.test(c)), `the lines' chips read their exit: ${t.chips.join(" · ")}`);
    await lintScreen("report (stalls)");
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
const secs = ((Date.now() - t0) / 1000).toFixed(1);
if (failed || errors.length) { console.error(`screens: FAIL (${failed} assertion(s), ${errors.length} error(s), ${lints} screens, ${clicks} clicks, ${secs}s)`); process.exit(1); }
console.log(`screens: ok (${out.length} checks · ${lints} screens linted · ${clicks} buttons clicked · ${secs}s)`);
