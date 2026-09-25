#!/usr/bin/env node
// QA on 778fa1b (players U and V) — the client's hand-offs and fixes, on the fake engine, headless at 400 × 800:
//   · an edit that touches only rows that never fire (`shadowed_by`) reads `≈`, whatever the sims' wobble
//   · the `vs sent` line is a row of its own under the well — it never covers a rule row or the shaft's gems
//   · exit lines: `−$37 swapped`, `heir purse +$40` (`ExitLine.wake`), `no top-up` (never `purse full`), the kept coins beside the
//     stolen items (`stolen scroll, $3`), a stolen flavour by its name now; a death's trace chip names its floor
//   · the repeat badge: `repeat short` (`repeat_unpaid`), `+heal at send` (`repeat_due`), else `repeat on · ≤$N`
//   · the death screen's first pass never moves the lit tablet; a flat reach reads `≈` alone; an exit patch says `return early`
//   · the vault tile under an open sheet leaves one surface up
//
//   node web/tests/qa778.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

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
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`) }); };
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen})`);
}
async function until(fn, label, timeout = 8000) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await fn(); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const txt = (sel) => page.evaluate((s) => { const e = document.querySelector(s); return e && !e.hidden && e.getClientRects().length ? e.textContent.replace(/\s+/g, " ").trim() : null; }, sel);

try {
  const rules = encodeURIComponent("hp<30% → drink heal\nfoes>=1 → attack nearest\ndepth>=6 → bank");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=778&rules=${rules}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await until(() => page.evaluate(() => document.querySelector(".camp .shaft")?.dataset.fc), "the first forecast");
  await sleep(300);

  // ---- a dead edit: the new row is shadowed (the forecast says so) and the engine's move wobbles — the line reads ≈
  await page.evaluate(() => {
    const r = window.__riddle, e = r.engine; delete e.forecastVsRefined;
    const fc = e.forecast; e.forecast = async () => { const f = await fc(); return r.rules.rows.length === 4 ? { ...f, shadowed_by: [null, null, 1, null] } : f; };
    e.forecastVs = async () => ({ depths: [1, 2, 3, 4, 5, 6].map((depth) => ({ depth, delta: -0.1, pm: 0.02 })), bank: { delta: -0.1, pm: 0.02 }, death: { delta: 0.1, pm: 0.02 } });
    r.insertRow({ conds: [{ k: "hp<", n: 50 }], verb: { v: "attack", a: "lowest" } }, 2);
  });
  const dead = await until(async () => { const t = await txt(".camp .shaft-vs-host .shaft-vs"); return t && t !== "vs sent…" ? t : null; }, "the dead edit's line");
  const marks = await page.evaluate(() => document.querySelectorAll(".camp .shaft .vsm").length);
  check(/^vs sent · D\d+ ≈ · bank ≈$/.test(dead) && marks === 0, `a row that never fires moves nothing: "${dead}", ${marks} marks`);
  // the strip is its own row: under the well, over nothing
  const lay = await page.evaluate(() => {
    const well = document.querySelector(".camp .camp-well").getBoundingClientRect(), strip = document.querySelector(".camp .shaft-vs-host").getBoundingClientRect();
    const cons = document.querySelector(".camp .console").getBoundingClientRect();
    return { wellBottom: Math.round(well.bottom), top: Math.round(strip.top), bottom: Math.round(strip.bottom), cons: Math.round(cons.top) };
  });
  check(lay.top >= lay.wellBottom - 1 && lay.bottom <= lay.cons + 1, `the vs strip sits between the well and the console (well ≤ ${lay.wellBottom}, strip ${lay.top}–${lay.bottom}, console ${lay.cons})`);
  await shot("qa778-dead-edit");
  // a live edit's move on the death gem: a fall reads good (`▼`, the up colour)
  await page.evaluate(() => { const r = window.__riddle; r.engine.forecastVs = async () => ({ depths: [{ depth: 6, delta: 0.2, pm: 0.02 }], bank: { delta: 0.2, pm: 0.02 }, death: { delta: -0.3, pm: 0.02 } }); const row = r.rules.rows.find((x) => x.conds[0]?.n !== undefined); if (row) row.conds[0].n += 5; else r.rules.rows.pop(); r.rulesChanged(); });
  await until(async () => { const t = await txt(".camp .shaft-vs-host .shaft-vs"); return t && /death/.test(t) ? t : null; }, "the live move");
  const tone = await page.evaluate(() => ({ line: document.querySelector('.camp .shaft-vs .vs-term[data-k="death"] b')?.className, gem: document.querySelector(".camp .shaft-ends .death .vsm")?.className ?? null }));
  check(/\bup\b/.test(tone.line ?? "") && (tone.gem === null || /\bup\b/.test(tone.gem)), `a fall in death is drawn as good (${JSON.stringify(tone)})`);

  // ---- the repeat badge
  const badge = async (patch) => {
    await page.evaluate((p) => { const r = window.__riddle; r.engine.setRestock ??= async () => r.lineage; r.lineage = { ...r.lineage, supplies: [{ id: 1, kind: "leash", known: true, label: "leash" }], repeat: true, repeat_kinds: ["heal"], repeat_gold: 26, repeat_due: [], repeat_unpaid: [], ...p }; r.go({ kind: "camp" }); }, patch);
    await sleep(150);
    return txt(".camp .cmd .repeat-badge");
  };
  const b1 = await badge({ repeat_unpaid: ["heal"] }), b2 = await badge({ repeat_due: ["heal"] }), b3 = await badge({});
  check(b1 === "repeat short" && b2 === "+heal at send" && b3 === "repeat on · ≤$26", `the repeat badge: unpaid \`${b1}\`, due \`${b2}\`, else \`${b3}\``);

  // ---- exit lines on the report
  const L = await page.evaluate(() => window.__riddle.lineage);
  const exits = [
    { carried: 66, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $66 carried · bones: 4 items on D6", purse_full: true, trace: { turns: [{ t: 10, row: 0, verb: { v: "attack", a: "nearest" }, hp: 0, foes: 1, telegraphs: [] }] } },
    { carried: 20, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $20 carried", wake: 40 },
    { carried: 63, keep_pct: 60, kept: 38, spent: 0, spent_on: [], text: "returned $38 · $63 carried · keeps 60%", swapped: 37, stolen: ["amber potion?"], stolen_gold: 3 },
  ];
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, renamed: { "amber potion?": "caustic" } }; });
  await page.evaluate(({ exits, cls }) => window.__riddle.go({ kind: "report", report: { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "jackal", n: 2 }], pending: [], reel: [{ pattern: "note", score: 0, t: 1, run_id: 1, text: "A jackal chased him to 1 HP; R1 returned; died to jackal." }], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [],
    xp: { class: cls, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 1, exits, swapped: 37, gold: { home: 38, salvage: 0, wake: 40, spent: 0, wake_n: 1 } } }), { exits, cls: L.class });
  await waitFor((s) => s?.screen === "report", "the report"); await sleep(300);
  const rep = await page.evaluate(() => ({ lines: [...document.querySelectorAll(".report .exit-row .ledger-btn")].map((b) => b.textContent.replace(/\s+/g, " ").trim()),
    chips: [...document.querySelectorAll(".report .exit-row .chip")].map((b) => b.textContent.replace(/\s+/g, " ").trim()), gold: document.querySelector(".report .gold-line")?.textContent.replace(/\s+/g, " ").trim(), reel: document.querySelector(".report")?.textContent ?? "" }));
  const all = rep.lines.join(" | ");
  check(/· −\$37 swapped/.test(all) && /stolen caustic, \$3/.test(all), `a swap and the coins thieves kept are on the line, a flavour by its name now (${all})`);
  check(/heir purse \+\$40/.test(all) && /no top-up/.test(all) && !/purse full/.test(all + rep.gold), `the purse reads \`heir purse +$40\` / \`no top-up\`, never \`purse full\` (${all})`);
  check(/−\$37 swapped/.test(rep.gold ?? "") && /no top-up/.test(rep.gold ?? ""), `the gold line counts the swaps and the deaths with no top-up ("${rep.gold}")`);
  check(rep.chips.some((c) => /^D6 · died · trace$/.test(c)), `a death's trace chip names its floor (${rep.chips.join(" | ")})`);
  check(/R1 return too late; died to jackal/.test(rep.reel), "a return that did not get him home reads `too late`");
  await shot("qa778-report");

  // ---- the death screen: the reach lands; nothing moves; `≈` alone; `return early`
  await page.evaluate(() => {
    const r = window.__riddle;
    const pat = (conds, v, survive, extra = {}) => ({ row: { conds, verb: { v } }, insert_at: 0, survive, forecast_delta: 0, camp_pending: true, ...extra });
    const ps = [pat([{ k: "hp<", n: 40 }], "return", 1, { exits: true }), pat([{ k: "hp<", n: 20 }], "read", 0.67), pat([{ k: "foes>=", n: 2 }], "retreat", 0.25)];
    r.engine.deathDeltas = () => new Promise((res) => setTimeout(() => res(ps.map((p, i) => ({ ...p, camp_pending: false, forecast_depth: 6, forecast_delta: [-0.49, 0.02, 0.1][i], forecast_pm: [0.03, 0.14, 0.05][i] }))), 500));
    r.go({ kind: "death", death: { run_id: 0, depth: 5, cause: "goblin_captain", margin: "", verdict: "gap", baseline: 0, trace: { turns: [] }, morgue: "", patches: ps } });
  });
  await waitFor((s) => s?.screen === "death", "the death"); await sleep(150);
  const pick = () => page.evaluate(() => ({ top: [...document.querySelectorAll(".patches > button.patch")].findIndex((b) => b.classList.contains("top")), gem: document.querySelector(".patch-gem .gem-n")?.textContent,
    order: [...document.querySelectorAll(".patches > button.patch .chips-inline")].map((c) => c.textContent.replace(/\s+/g, " ").trim()).join(" | "), reach: [...document.querySelectorAll(".patches > button.patch .delta")].map((d) => d.textContent.replace(/\s+/g, " ").trim()) }));
  const d0 = await pick(); await sleep(900); const d1 = await pick();
  check(d0.top === d1.top && d0.gem === d1.gem && d0.order === d1.order, `the reach lands and nothing moves (lit ${d0.top} → ${d1.top}, gem ${d0.gem} → ${d1.gem})`);
  check(d1.reach[0] === "return early · reach D6 −49 ±3" && d1.reach[1] === "reach D6 ≈" && d1.reach[2] === "reach D6 +10 ±5", `an exit names its cost, a flat reach is \`≈\` alone (${d1.reach.join(" · ")})`);
  await shot("qa778-death");

  // ---- one surface: the vault tile under an open sheet
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, vault: [{ id: 91, kind: "sword", known: true, label: "sword +1" }] }; r.go({ kind: "camp" }); }); await sleep(250);
  const hasVault = await page.locator(".cmd .tile[data-tile=vault]").count();
  if (hasVault) {
    await page.locator(".camp .strip button.gold, .camp .bar button.gold").first().click({ timeout: 5000 }).catch(() => {}); await sleep(200);
    const sheet0 = await page.locator(".sheet-wrap").count();
    await page.locator(".cmd .tile[data-tile=vault]").click({ timeout: 5000 }); await sleep(200);
    const after = await page.evaluate(() => ({ sheets: document.querySelectorAll(".sheet-wrap").length, panel: document.querySelector(".panel")?.dataset.panel ?? null }));
    check(sheet0 === 1 && after.sheets === 0 && after.panel === "vault", `a tile under an open sheet takes the tap and closes it (sheets ${sheet0} → ${after.sheets}, panel ${after.panel})`);
  } else check(false, "the vault tile is carved (a kept item)");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.log(e);
console.log(`qa778: ${failed || errors.length ? "FAIL" : "ok"} (${out.length} checks${failed ? `, ${failed} failed` : ""}${errors.length ? `, ${errors.length} errors` : ""})`);
process.exit(failed || errors.length ? 1 : 0);
