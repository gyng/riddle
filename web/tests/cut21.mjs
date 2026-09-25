#!/usr/bin/env node
// Cut 21 client gates (docs/CUT21.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §1  no waystone lit, no start tablet; the first lit carves `start → D1` beside the rules and the cage; the picker lists D1 and each
//       lit waystone with its forecast move and toll (`D9 · bank +12% · $90`, `…` until `startForecast()` lands); a tap is
//       `setStart(9)`, the tablet reads `start → D9 · $90`, the shaft starts at D9; the fake's send pays `waystone D9` and the watch's
//       floor card names it (`D9 · the Fens · waystone`)
//   §2  an exit that shelved a found supply says so (`found heal ×2 → shelf`) on its report line and in `shelved`; a shelf line an exit put
//       there reads `· found`, and with the repeat on a kind no row names reads `· no row`
//   §3  the `depth ≥` picker offers every depth to best + 2 (D22 on a best of D20); `picked clean` reads `thinned`, `restock capped` reads
//       `restock ≤ income` and opens the gold sheet; the trace's chain carries the earlier ticks' reasons and their `because` links
//
//   node web/tests/cut21.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows } from "./lib/frame.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`) }); };
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const go = (screen) => page.evaluate((x) => window.__riddle.go(x), screen);
const closeSheets = async () => { await page.keyboard.press("Escape"); await sleep(120); };
const txt = (sel) => page.evaluate((s) => { const e = document.querySelector(s); return e && !e.hidden && e.getClientRects().length ? e.textContent.replace(/\s+/g, " ").trim() : null; }, sel);
/** Patch the fake's saved lineage and reload it through the save (the camp remounts). */
const patchLineage = (patch) => page.evaluate(async (p) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  Object.assign(e.lineage, p); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
}, patch);

try {
  // ---- §1: the start tablet
  const bankRules = encodeURIComponent("foes>=1 → attack nearest\ndepth>=12 → bank");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=21&rules=${bankRules}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await sleep(300);
  const before = await txt(".camp .start-tab");
  await patchLineage({ best_depth: 12, heir: 3, gold: 500, waystones: [5, 9], start: 1, facts: ["item:leash", "vault"] });
  await sleep(400);
  const after = await txt(".camp .start-tab");
  const inCol = await page.evaluate(() => !!document.querySelector(".camp-main .tablets .cage-tab + .start-tab"));
  check(before === null && after === "start → D1", `a lit waystone carves the start tablet (${before} → ${after})`);
  check(inCol, "the start tablet sits beside the rules and the cage tablet");
  await shot("cut21-start-tablet");
  // the picker: the engine's moves land after a beat; `…` meanwhile
  await page.evaluate(() => {
    const r = window.__riddle; r.__starts = [];
    const real = r.engine.startForecast.bind(r.engine);
    r.engine.startForecast = () => new Promise((res) => setTimeout(() => res(real()), 400));
    const set = r.engine.setStart.bind(r.engine); r.engine.setStart = async (d) => { r.__starts.push(d); return set(d); };
  });
  await page.locator(".camp .start-tab").click({ timeout: 5000 }); await sleep(120);
  const opts = () => page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .start-picker .start-opt")].map((b) => `${b.textContent.replace(/\s+/g, " ").trim()}${b.classList.contains("on") ? "*" : ""}`));
  const pending = await opts();
  await sleep(700);
  const landed = await opts();
  // QA 778fa1b (qaV; core): the toll is the wire's only (none at 0, no client guess) — `T(d)` is what the engine charges
  const tolls = await page.evaluate(async () => Object.fromEntries((await window.__riddle.engine.startForecast()).map((o) => [o.start, o.toll ?? 0])));
  const T = (d) => (tolls[d] ? ` · $${tolls[d]}` : "");
  check(pending.length === 3 && pending[1] === "D5 …" && pending[2] === "D9 …", `the picker lists D1 and each lit waystone, \`…\` until measured (${pending.join(" | ")})`);
  // QA 778fa1b (qaV: `D1 · bank 100%` beside `D5 · bank −10` read as −$10): every option in one absolute form — its level, its death share,
  // the gold a send brings home net of the toll (`~$N`), then the toll (Cut 22: $5 × depth)
  const shape = /^D1 · (bank|D\d+) \d+%( · death \d+%)? · ~\$-?\d+\*$/.test(landed[0] ?? "") && new RegExp(`^D5 · (bank|D\\d+) \\d+%( · death \\d+%)? · ~\\$-?\\d+${T(5).replace("$", "\\$")}$`).test(landed[1] ?? "") && new RegExp(`^D9 · (bank|D\\d+) \\d+%( · death \\d+%)? · ~\\$-?\\d+${T(9).replace("$", "\\$")}$`).test(landed[2] ?? "");
  check(shape, `each option: its forecast move and its toll (${landed.join(" | ")})`);
  await shot("cut21-start-picker");
  await page.locator(".sheet-wrap .start-opt[data-start='9']").click({ timeout: 5000 }); await sleep(500);
  const picked = await txt(".camp .start-tab"), calls = await page.evaluate(() => window.__riddle.__starts);
  check(picked === `start → D9${T(9)}` && calls.join() === "9", `a tap sets the start (setStart ${calls.join()}; tablet \`${picked}\`)`);
  const notches = await page.evaluate(() => [...document.querySelectorAll(".shaft .notch .dl")].map((n) => n.textContent.trim()));
  check(/^D9\b/.test(notches[0] ?? ""), `the shaft starts at the chosen start (${notches.join(" · ")})`);
  await shot("cut21-start-shaft");
  // the send: the toll on the ledger, the watch's first floor names the waystone
  const gold0 = await page.evaluate(() => window.__riddle.lineage.gold);
  await page.evaluate(() => { window.__riddle.dev.speed = "fights"; });
  await page.locator(".gem.send").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "watch", "the watch");
  const cards = await page.evaluate(() => new Promise((res) => {
    const seen = new Set(), t0 = performance.now();
    const poll = () => { const c = document.querySelector(".watch .interstitial"); if (c && !c.hidden && c.textContent) seen.add(c.textContent.replace(/\s+/g, " ").trim());
      if (seen.size || performance.now() - t0 > 8000) { res([...seen]); return; } requestAnimationFrame(poll); };
    poll();
  }));
  check(cards[0] === "D9 · the Fens · waystone", `the watch's floor card names the waystone (${cards.join(" | ") || "no card"})`);
  await shot("cut21-watch-waystone");
  // the engine's lineage (the app adopts it at the exit)
  const toll = await page.evaluate(async () => ((await window.__riddle.engine.lineage()).gold_ledger ?? []).find((g) => /^waystone D9$/.test(g.why)));
  const gold1 = await page.evaluate(async () => (await window.__riddle.engine.lineage()).gold);
  check(tolls[9] ? toll?.delta === -tolls[9] || gold0 - gold1 === tolls[9] : !toll, `the send pays the toll the wire named (${tolls[9] ?? 0}) (${toll ? `${toll.delta} ${toll.why}` : `$${gold0} → $${gold1}`})`);

  // ---- §2: found supplies to the shelf; `no row`
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=22`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  const x = (text, kept, pct, extra = {}) => ({ carried: 80, keep_pct: pct, kept, spent: 0, spent_on: [], text, run_id: 1, ...extra });
  const report = { elapsed_s: 100, runs: 2, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, tamed: [], hatched: [], lost: [],
    xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [{ kind: "sword", n: 1, gold: 6 }], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 2, returned: 0,
    picked: [3], restock_capped: true, gold: { home: 212, salvage: 6, wake: 0, spent: 80 }, spent: [{ kind: "heal", n: 2, gold: 80 }],
    shelved: [{ kind: "heal", n: 3 }],
    exits: [x("banked $106 · $106 carried · keeps 100%", 106, 100, { shelved: [{ kind: "heal", n: 2 }] }), x("banked $106 · $106 carried · keeps 100%", 106, 100, { shelved: [{ kind: "heal", n: 1 }] })] };
  await go({ kind: "report", report }); await waitFor((s) => s?.screen === "report", "the report"); await sleep(300);
  const lines = await page.evaluate(() => [...document.querySelectorAll(".report .exit-lines .ledger-btn")].map((b) => b.textContent.replace(/\s+/g, " ").trim()));
  check(lines.some((l) => /· found heal ×2 → shelf$/.test(l)), `the exit line says what went to the shelf (${lines.join(" | ")})`);
  const shelf = await page.evaluate(() => { const s = [...document.querySelectorAll(".report .rsec")].find((x) => x.querySelector(".label")?.textContent === "shelved"); return s ? [...s.querySelectorAll(".chip")].map((c) => c.textContent.replace(/\s+/g, " ").trim()) : []; });
  check(shelf.join() === "found heal ×3 → shelf", `the report's \`shelved\` (${shelf.join(" · ") || "absent"})`);
  // §3 on the same report: the words
  const pk = await txt(".report .picked-line"), gl = await txt(".report .gold-line");
  check(pk === "D3 · thinned", `\`picked clean\` reads \`thinned\` ("${pk}")`);
  check(/restock ≤ income$/.test(gl ?? ""), `\`restock capped\` reads \`restock ≤ income\` ("${gl}")`);
  await shot("cut21-report");
  await page.locator(".report .gold-line .ledger-link").first().click({ timeout: 3000 }); await sleep(250);
  const goldSheet = await page.evaluate(() => !!document.querySelector(".sheet-wrap .gold-sheet"));
  check(goldSheet, "a tap on it opens the gold sheet");
  await closeSheets();
  // the loadout: a shelved line `· found`; a kind no row names `· no row`
  await page.evaluate(() => {
    const r = window.__riddle; const L = r.lineage;
    r.lineage = { ...L, gold: 300, repeat: true, repeat_kinds: ["heal"], repeat_gold: 40, supplies: [
      { id: 9001, kind: "heal", known: true, label: "heal" }, { id: 9002, kind: "strength", known: true, label: "strength" }, { id: 9003, kind: "heal", known: true, label: "heal", found: true }] };
    r.go({ kind: "camp" });
  });
  await sleep(300);
  await page.locator(".cmd .tile[data-tile=loadout]").click({ timeout: 5000 }).catch(() => {}); await sleep(250);
  const sup = await page.evaluate(() => [...document.querySelectorAll(".panel .supplies .chip.item")].map((c) => c.textContent.replace(/\s+/g, " ").replace(/\s*×$/, "").trim()));
  check(sup.join(" | ") === "heal | strength · no row | heal · found", `the shelf: \`· no row\` on a kind the repeat skips, \`· found\` on a shelved one (${sup.join(" | ")})`);
  await shot("cut21-loadout");
  await closeSheets();

  // ---- §3: the depth picker to best + 2
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=23&rules=${encodeURIComponent("foes>=1 → attack nearest\ndepth>=8 → bank")}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await patchLineage({ best_depth: 20, heir: 3 }); await sleep(400);
  await editRows(page); await sleep(200);
  await page.locator(".editor .row .chip.cond", { hasText: "depth" }).first().click({ timeout: 5000 }); await sleep(200);
  const nums = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .grid.nums .chip.num")].map((b) => Number(b.textContent)));
  const every = Array.from({ length: 21 }, (_, i) => i + 2);
  check(nums.join() === every.join(), `\`depth ≥\` offers every depth to best + 2 (${nums[0]}…${nums[nums.length - 1]}, ${nums.length} values)`);
  await shot("cut21-depth-picker");
  await page.locator(".sheet-wrap .grid.nums .chip.num", { hasText: /^22$/ }).click({ timeout: 5000 }); await sleep(200);
  const row = await page.evaluate(() => window.__riddle.rules.rows.find((r) => r.verb.v === "bank")?.conds[0]?.n);
  check(row === 22, `a tap sets it (depth ≥ ${row})`);

  // ---- §3: the chain carries each tick's reason and its because, not only the last
  const b = (text, t, depth = 13) => ({ text, t, depth });
  const turns = [
    { t: 800, row: 3, verb: { v: "attack", a: "boss" }, hp: 9, foes: 2, telegraphs: [], rows: [{ row: 0, why: "no item", because: b("drank the last heal", 700) }, { row: 1, why: "no way", because: b("foes held the way", 790) }] },
    { t: 810, row: 3, verb: { v: "attack", a: "boss" }, hp: 9, foes: 2, telegraphs: [], rows: [{ row: 0, why: "no item", because: b("drank the last heal", 700) }, { row: 1, why: "no way", because: b("foes held the way", 790) }] },
    { t: 820, row: 3, verb: { v: "attack", a: "boss" }, hp: 7, foes: 2, telegraphs: [], rows: [{ row: 0, why: "no item", because: b("drank the last heal", 700) }, { row: 1, why: "no way", because: b("foes held the way", 790) }] },
    { t: 830, row: 1, verb: { v: "bank" }, hp: 3, foes: 2, telegraphs: [], rows: [{ row: 0, why: "no item", because: b("drank the last heal", 700) }] },
  ];
  const rules = { rows: [{ conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }, { conds: [{ k: "hp<", n: 30 }], verb: { v: "bank" } }, { conds: [], verb: { v: "explore" } }, { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "boss" } }] };
  await go({ kind: "death", death: { run_id: 0, depth: 13, cause: "gas", margin: "", verdict: "gap", baseline: 0.4, trace: { turns }, morgue: "m", rules, patches: [] } });
  await waitFor((s) => s?.screen === "death", "the death"); await sleep(300);
  const chain = await page.evaluate(() => [...document.querySelectorAll(".death .chain .chain-row")].map((r) => ({ cls: r.className, text: [...r.childNodes].map((c) => c.textContent.trim()).filter(Boolean).join(" ").replace(/\s+/g, " ") })));
  const ticks = chain.filter((c) => /\btick\b/.test(c.cls)).map((c) => c.text);
  check(ticks.some((t) => /^t800–820 R2 bank no way ← foes held the way/.test(t)), `an earlier stretch's reason carries its because (${ticks.join(" | ") || "none"})`);
  check(!ticks.some((t) => /R1 drink heal/.test(t)), "a reason the last tick repeats stays on the last tick's line");
  check(chain.some((c) => /^R1 drink heal no item ← drank the last heal/.test(c.text)) && chain.some((c) => /fired/.test(c.cls) && /^R2 bank fired$/.test(c.text)), `the last tick's lines stand (${chain.filter((c) => !/tick/.test(c.cls)).map((c) => c.text).join(" | ")})`);
  await shot("cut21-chain");
} catch (e) {
  errors.push(`exception: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(`error ${e}`);
const bad = failed + errors.filter((e) => !/favicon|ResizeObserver/.test(e)).length;
console.log(bad ? `FAIL cut21: ${failed} checks failed, ${errors.length} errors` : `ok cut21: ${out.length} checks`);
process.exit(bad ? 1 : 0);
