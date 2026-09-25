#!/usr/bin/env node
// Cut 23 client gates (docs/CUT23.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §1  the forge: no tile before a kit step is affordable (a fresh purse); the ladder carves it (a badge counts the affordable
//       steps); the sheet has a tablet per slot — pips, the next step `kit · D6 +3 · $225` (its measured move, `…` until it lands),
//       every later step's price; an unaffordable next says its nights; a buy takes two taps, the armed chip does not disarm on its
//       own, and nothing on the sheet moves under the finger (the next step takes the bought one's place); the report's PENDING
//       `forge sword +1 · $300` opens the forge
//   §2  a share the sims sampled at 0 prints `<N%` (the core's `low`), never `0%` — the ends line, the gems, the notches, the bars
//   §3  a tablet with a why-not opens it on tap (`0/40 · blocked · no item`, the reason's gloss) beside the tablet, never over it,
//       with `edit`; a tablet without one edits at once; a grip tap (no drag) opens it while editing; a `✗` callout and a shouted
//       word carry their reason on tap (the ticker or the picture), a death chain's reason its gloss; a paid card names what it holds (`carries`)
//   §4  a chip's option sheet never covers the row it edits (top row and bottom row); a tap outside closes it and does nothing
//       else; the free supply's × is ≥ 32 px and its armed `drop` never undoes itself; buying a supply moves no chip under the
//       finger
//
//   node web/tests/cut23.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { openPanel, editRows } from "./lib/frame.mjs";

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
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
async function until(fn, label, timeout = 8000) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await fn(); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const closeSheets = async () => { await page.keyboard.press("Escape"); await sleep(120); };
const txt = (sel) => page.evaluate((s) => { const e = document.querySelector(s); return e && !e.hidden && e.getClientRects().length ? e.textContent.replace(/\s+/g, " ").trim() : null; }, sel);
const rect = (sel) => page.evaluate((s) => { const e = document.querySelector(s); if (!e) return null; const r = e.getBoundingClientRect(); return { x: r.x, y: r.y, w: r.width, h: r.height, top: r.top, bottom: r.bottom, left: r.left, right: r.right }; }, sel);
const overlap = (a, b) => !!a && !!b && a.left < b.right && b.left < a.right && a.top < b.bottom && b.top < a.bottom;
/** Patch the save (lineage fields and the fake's own state) and reload the camp on it. */
const patchSave = (lin, st = {}) => page.evaluate(async ([p, q]) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  Object.assign(e.lineage, p); Object.assign(e, q); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
}, [lin, st]);
const camp = async () => { await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250); };

try {
  const rules = encodeURIComponent("hp<30% → drink heal\nfoes>=1 → attack nearest\nfoe:ranged → throw fire");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=231&rules=${rules}`, { waitUntil: "domcontentloaded" });
  await camp();
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); localStorage.setItem("riddle.editing", "0"); });

  // ---- §1 the forge
  await patchSave({ heir: 2, best_depth: 4, gold: 0, forge: {}, graveyard: [{ heir: 1, depth: 3, cause: "rat", deeds: [] }] });
  await camp();
  check(!(await page.locator(".cmd .tile[data-tile=forge]").count()), "no forge tile while no kit step is affordable (and nothing salvaged)");
  await patchSave({ gold: 500 });
  await camp();
  const tile = await page.evaluate(() => { const t = document.querySelector(".cmd .tile[data-tile=forge]"); return t ? { badge: t.querySelector(".kit-n")?.textContent ?? null } : null; });
  check(!!tile && /^\d$/.test(tile.badge ?? ""), `the first affordable kit step carves the forge tile, its badge counts them (${JSON.stringify(tile)})`);
  await page.locator(".cmd .tile[data-tile=forge]").click({ timeout: 5000 });
  await until(() => page.evaluate(() => document.querySelectorAll(".sheet-wrap .forge .kit-slot").length === 3), "the forge's three slots");
  const moves = await until(() => page.evaluate(() => { const t = [...document.querySelectorAll(".sheet-wrap .forge .kit-next")].map((b) => b.textContent.replace(/\s+/g, " ").trim()); return t.every((x) => !/…/.test(x)) ? t : null; }), "the steps' measured moves");
  check(moves.length === 3 && moves.every((m) => /^[a-z][\w +]* · (D\d+ ([+−]\d+|≈)|(bank|death) [+−]\d+) · \$\d+( · \d+ nights?)?$/.test(m)), `each next step reads its kit, its move and its price (${moves.join(" | ")})`);
  const later = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .forge .kit-later")].map((l) => l.textContent.trim()));
  check(later.every((l) => /\$\d+/.test(l)), `every later step shows its price (${later.join(" | ")})`);
  await shot("cut23-forge");
  // a buy: two taps, the armed chip holds, nothing moves
  const sel = ".sheet-wrap .forge .kit-slot[data-slot=weapon] .kit-next";
  const gold0 = await page.evaluate(() => window.__riddle.lineage.gold);
  await sleep(300);   // the sheet's unfold (120 ms) is over
  const r0 = await rect(sel), price = Number(/\$(\d+)/.exec(moves[0])?.[1] ?? 0);
  await page.locator(sel).click({ timeout: 5000 });
  await sleep(3600);
  const armed = await page.evaluate((s) => { const b = document.querySelector(s); return b ? { armed: b.classList.contains("armed"), text: b.textContent } : null; }, sel);
  check(armed?.armed && armed.text === `ok $${price}`, `the first tap arms the step and it stays armed (3.6 s later: "${armed?.text}")`);
  await page.locator(sel).click({ timeout: 5000 });
  await until(() => page.evaluate((g) => window.__riddle.lineage.gold < g, gold0), "the buy");
  await sleep(200);
  const after = await page.evaluate(() => ({ gold: window.__riddle.lineage.gold, pips: document.querySelectorAll(".sheet-wrap .forge .kit-slot[data-slot=weapon] .pip.on").length, kit: window.__riddle.lineage.kit.find((k) => k.slot === "weapon").owned }));
  const r1 = await rect(sel);
  check(after.gold === gold0 - price && after.pips === 1 && after.kit === 1, `the second tap buys the step ($${gold0} → $${after.gold}, pips ${after.pips})`);
  check(!!r0 && !!r1 && Math.abs(r0.top - r1.top) < 1 && Math.abs(r0.h - r1.h) < 1, `the next step takes the bought one's place (top ${r0?.top} → ${r1?.top})`);
  const armOff = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .forge .kit-next.armed")].length);
  const offNext = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .forge .kit-next.off")].map((b) => b.textContent.replace(/\s+/g, " ").trim()));
  check(offNext.length > 0 && offNext.every((t) => /\d+ nights?$/.test(t)), `a step the purse cannot buy is off and says its nights (${offNext.join(" | ")})`);
  await shot("cut23-forge-bought");
  await closeSheets();
  check(armOff === 0, "nothing stays armed after the buy");

  // ---- §2 the low end: a set that banks at once never dies — `death <N%`, never `0%`
  await page.evaluate(() => window.__riddle.setRulesText("depth>=1 → bank"));
  await until(() => page.evaluate(() => window.__riddle.lastForecast && !document.querySelector(".camp .shaft").classList.contains("stale") && window.__riddle.rules.rows.length === 1), "the bank set's forecast");
  await sleep(300);
  const low = await page.evaluate(() => { const f = window.__riddle.lastForecast; return { low: f.low, death: f.ends?.death, gem: document.querySelector(".camp .shaft-ends .end.death b")?.firstChild?.textContent ?? null, dps: [...document.querySelectorAll(".camp .shaft .notch .dp")].map((d) => d.firstChild?.textContent ?? "") }; });
  check(low.low > 0 && low.death === 0 && low.gem === `<${low.low}%`, `a death sampled at 0 of N reads \`<${low.low}%\` on its gem ("${low.gem}")`);
  check(low.dps.every((d) => d !== "0%"), `no notch reads 0% (${low.dps.join(" · ")})`);
  await openPanel(page, "forecast");
  const ends = await txt(".panel .forecast .fc-ends");
  const bars = await page.evaluate(() => [...document.querySelectorAll(".panel .forecast .bar .n")].map((n) => n.firstChild?.textContent ?? ""));
  check(!!ends && /death <\d+%/.test(ends) && !/(^|[^<\d])0%/.test(ends), `the ends line: "${ends}"`);
  check(bars.every((b) => b !== "0%"), `no bar reads 0% (${bars.join(" · ")})`);
  await shot("cut23-low-end");
  await closeSheets(); if (await page.locator(".panel[data-panel=forecast]").count()) await page.locator(".camp .shaft").click().catch(() => {});

  // ---- §3 a row's why-not
  await patchSave({ unlocks: ["tame", "throw", "row5"] }, { runCounter: 2 });
  await camp();
  // the set the sends ran under (the engine's why-not is per row of the set it holds)
  const set = await page.evaluate(async () => { const r = window.__riddle; await r.setRulesText("hp<30 → drink heal\nfoes>=2 → throw fire\nfoes>=1 → attack nearest"); await new Promise((x) => setTimeout(x, 200)); await r.refresh(); return r.lineage.sets[r.lineage.active_set].rows.length; });
  check(set === 3, `the engine holds the three-row set (${set})`);
  await page.evaluate(() => localStorage.setItem("riddle.editing", "0")); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
  const whys = await page.evaluate(() => window.__riddle.rowWhy().map((w) => w?.text ?? null));
  check(whys.every((w) => typeof w === "string"), `every row of a set that ran has its why-not (${whys.join(" | ")})`);
  const blockedAt = await page.evaluate(() => window.__riddle.rowWhy().findIndex((w) => w?.blocked));
  const tab = `.editor.compact .row.tablet[data-i="${blockedAt}"]`;
  await page.locator(tab).click({ timeout: 5000 }); await sleep(250);
  const why = await page.evaluate(() => { const s = document.querySelector(".sheet-wrap .row-why"); return s ? { line: s.querySelector(".why-line")?.textContent ?? "", gloss: [...s.querySelectorAll(".why-gloss")].map((g) => g.textContent.replace(/\s+/g, " ").trim()), edit: !!s.querySelector(".why-edit") } : null; });
  const tr = await rect(tab), sr = await rect(".sheet-wrap .sheet");
  check(!!why && /^\d+\/\d+ · blocked · no item( · \d+ sends?)?$/.test(why.line) && why.gloss.some((g) => /^no item · none in pack$/.test(g)) && why.edit, `a blocked row's tablet opens its why-not with the reason's gloss (${JSON.stringify(why)})`);
  check(!overlap(tr, sr), `the why sheet sits beside the tablet, never over it (tablet ${Math.round(tr?.top)}–${Math.round(tr?.bottom)}, sheet ${Math.round(sr?.top)}–${Math.round(sr?.bottom)})`);
  await shot("cut23-row-why");
  await page.locator(".sheet-wrap .why-edit").click({ timeout: 5000 }); await sleep(250);
  check(!(await page.locator(".editor.compact").count()) && !(await page.locator(".sheet-wrap").count()), "its `edit` opens the row for editing");
  // while editing, a tap on the grip (no drag) opens it too
  const grip = page.locator(`.editor .row[data-i="0"] .grip`);
  const gb = await grip.boundingBox();
  await page.mouse.move(gb.x + gb.width / 2, gb.y + gb.height / 2); await page.mouse.down(); await page.mouse.up(); await sleep(250);
  check(!!(await txt(".sheet-wrap .row-why .why-line")), "a tap on a row's grip opens its why-not while editing");
  await closeSheets();
  // a fresh set's tablet edits at once (no why yet)
  await patchSave({}, { runCounter: 0 }); await camp();
  await page.evaluate(() => localStorage.setItem("riddle.editing", "0")); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
  await page.locator(".editor.compact .row.tablet").first().click({ timeout: 5000 }); await sleep(250);
  check(!(await page.locator(".sheet-wrap").count()) && !(await page.locator(".editor.compact").count()), "a tablet with no why-not yet edits at once");

  // a paid card names what it holds
  await patchSave({ marks: 20, gold: 5000 }); await camp();
  await openPanel(page, "unlocks", { all: true });
  const carries = await page.evaluate(() => [...document.querySelectorAll(".unlocks .card")].map((c) => ({ label: c.querySelector(".card-main > span")?.textContent ?? "", carries: c.querySelector(".carries")?.textContent ?? null })).filter((c) => c.carries));
  check(carries.length > 0 && carries.every((c) => c.carries.split(/\s+/).length <= 6), `paid cards name what they hold (${carries.map((c) => `${c.label}: ${c.carries}`).join(" | ")})`);
  await closeSheets(); if (await page.locator(".panel[data-panel=unlocks]").count()) await page.locator(".cmd .tile[data-tile=unlocks]").click().catch(() => {});

  // ---- §4 the option sheet never covers its row; an outside tap only closes
  await editRows(page); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
  const nRows = await page.locator(".editor .row").count();
  for (const i of [0, nRows - 1]) {
    const row = `.editor .row[data-i="${i}"]`;
    await page.locator(`${row} .chip.verb`).first().click({ timeout: 5000 }); await sleep(250);
    const a = await rect(row), s = await rect(".sheet-wrap .sheet");
    check(!!s && !overlap(a, s), `R${i + 1}'s verb sheet leaves the row in view (row ${Math.round(a.top)}–${Math.round(a.bottom)}, sheet ${Math.round(s?.top)}–${Math.round(s?.bottom)})`);
    if (i === nRows - 1) await shot("cut23-sheet-beside-row");
    await closeSheets();
  }
  // an outside tap: over another row's chip — the sheet closes, no other opens, the rules hold
  const before = await page.evaluate(() => JSON.stringify(window.__riddle.rules.rows));
  await page.locator(`.editor .row[data-i="0"] .chip.cond`).first().click({ timeout: 5000 }); await sleep(250);
  const other = await rect(`.editor .row[data-i="${nRows - 1}"] .chip.verb`);
  const sheetR = await rect(".sheet-wrap .sheet");
  const target = !overlap(other, sheetR) ? other : await rect(".camp .shaft");
  await page.mouse.click(target.x + target.w / 2, target.y + target.h / 2); await sleep(300);
  const afterTap = await page.evaluate(() => ({ sheets: document.querySelectorAll(".sheet-wrap").length, rows: JSON.stringify(window.__riddle.rules.rows), panel: !!document.querySelector(".panel-host.open") }));
  check(afterTap.sheets === 0 && afterTap.rows === before && !afterTap.panel, `a tap outside the sheet closes it and does nothing else (${afterTap.sheets} sheets, rules ${afterTap.rows === before ? "held" : "changed"}, panel ${afterTap.panel})`);

  // the free supply's × and its confirm; a buy moves nothing
  await patchSave({ gold: 400, supplies: [{ id: 100001, kind: "leash", known: true, label: "leash", free: true }] }); await camp();
  await openPanel(page, "loadout");
  await until(() => page.evaluate(() => document.querySelectorAll(".panel .supplies .chip.buy").length > 0), "the shop");
  const x = await rect(".panel .supplies .chip.item .x");
  check(!!x && x.w >= 32 && x.h >= 32, `the supply × is ≥ 32 px (${Math.round(x?.w)}×${Math.round(x?.h)})`);
  const buysBefore = await page.evaluate(() => [...document.querySelectorAll(".panel .supplies .chip.buy")].map((b) => { const r = b.getBoundingClientRect(); return `${Math.round(r.left)},${Math.round(r.top)},${Math.round(r.width)},${Math.round(r.height)}`; }));
  const firstBuy = page.locator(".panel .supplies .chip.buy:not(.off)").first();
  const nPacked = await page.locator(".panel .supplies .chip.item").count();
  await firstBuy.click({ timeout: 5000 });
  await until(() => page.evaluate((n) => document.querySelectorAll(".panel .supplies .chip.item").length > n, nPacked), "the bought line");
  await sleep(200);
  const buysAfter = await page.evaluate(() => [...document.querySelectorAll(".panel .supplies .chip.buy")].map((b) => { const r = b.getBoundingClientRect(); return `${Math.round(r.left)},${Math.round(r.top)},${Math.round(r.width)},${Math.round(r.height)}`; }));
  check(buysBefore.length > 0 && buysBefore.join("|") === buysAfter.join("|"), `a buy moves no shop chip (${buysBefore.length} chips, ${buysBefore.filter((b, i) => b !== buysAfter[i]).length} moved)`);
  await shot("cut23-supplies");
  const leashX = page.locator(".panel .supplies .chip.item", { hasText: "free" }).locator(".x");
  await leashX.click({ timeout: 5000 }); await sleep(3600);
  const armedX = await page.evaluate(() => { const c = [...document.querySelectorAll(".panel .supplies .chip.item")].find((e) => /free/.test(e.textContent)); return c?.querySelector(".x")?.textContent ?? null; });
  check(armedX === "drop", `the free line's × arms \`drop\` and holds it (3.6 s later: "${armedX}")`);
  await leashX.click({ timeout: 5000 });
  await until(() => page.evaluate(() => !window.__riddle.lineage.supplies.some((s) => s.free)), "the leash dropped");
  check(true, "the second tap drops the free line");
  await closeSheets();

  // ---- §1 the report's PENDING `forge sword +1 · $300` opens the forge
  await page.evaluate(async () => { const r = window.__riddle; const rep = await r.engine.runOfflineQuick(1800); rep.pending = [...(rep.pending ?? []), "forge sword +1 · $300"]; r.go({ kind: "report", report: rep }); });
  await waitFor((s) => s?.screen === "report", "the report");
  const fl = page.locator(".report .forge-pending .forge-line").first();
  const flText = (await fl.count()) ? (await fl.textContent()).trim() : null;
  if (flText) { await fl.click({ timeout: 5000 }); await sleep(250); }
  check(flText === "forge sword +1 · $300" && (await page.locator(".sheet-wrap .forge .kit-slot").count()) === 3, `PENDING's forge line opens the forge ("${flText}")`);
  await closeSheets();
  await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();

  // ---- §3 a reason in a death's chain carries its gloss on tap
  await page.evaluate(() => window.__riddle.go({ kind: "death", death: { run_id: 999, depth: 3, cause: "rat", margin: "1 hp short", verdict: "gap", baseline: 0.5, morgue: "", patches: [],
    trace: { turns: [{ t: 10, row: 1, verb: { v: "attack", a: "nearest" }, hp: 1, foes: 1, telegraphs: [], rows: [{ row: 0, why: "no item", because: { text: "den took the heal", t: 5, depth: 3 } }] }] } } }));
  await waitFor((s) => s?.screen === "death", "the death screen");
  const reason = page.locator(".death .chain-row button.why.has-gloss").first();
  const hasReason = await reason.count();
  if (hasReason) { await reason.click({ timeout: 5000 }); await sleep(150); }
  const glossed = hasReason ? (await reason.textContent()).replace(/\s+/g, " ").trim() : null;
  check(glossed === "no item · none in pack", `a chain's reason takes its gloss on tap ("${glossed}")`);
  await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();

  // ---- §3 a `✗` callout and a shouted word carry their reason on tap
  await page.evaluate(() => {
    const r = window.__riddle, e = r.engine, step = e.step.bind(e); let n = 0;
    e.step = async (k) => { const res = await step(k); if (n++ === 1) { const t = res.events[0]?.t ?? res.snapshot.turn; res.events.push({ t, k: "callout", text: "read ✗ no use" }); } return res; };
  });
  await page.evaluate(() => window.__riddle.go({ kind: "watch" }));
  await waitFor((s) => s?.screen === "watch", "the watch");
  // the line may be under a card or a held beat when it is released: the reason is there on a tap for a while all the same — tap the
  // picture until it answers (a tap there does nothing else)
  let shown = null, tip = null;
  const t0 = Date.now();
  while (Date.now() - t0 < 20_000 && !tip) {
    shown ??= await page.evaluate(() => { const t = document.querySelector(".watch .ticker"); return t && t.classList.contains("has-why") ? t.textContent : null; });
    await page.locator(".watch canvas.view").click({ position: { x: 200, y: 300 }, timeout: 2000 }).catch(() => {});
    await sleep(150);
    tip = await txt(".watch .why-tip.show");
    if (!tip) await sleep(250);
  }
  check(tip === "read ✗ no use → no effect now", `a \`✗\` refusal's reason on tap ("${tip}")`);
  check(shown === null || shown === "read ✗ no use", `the ticker marks a reasoned line (has-why: "${shown}")`);
  await shot("cut23-why-tip");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut23: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut23: ok (${out.length} checks)`);
