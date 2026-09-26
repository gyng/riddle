#!/usr/bin/env node
// QA a946e04 (scratchpad/qaS, qaT) client gates, on the fake engine, headless at 400 × 800:
//   1  a toll the purse cannot pay: the start tablet reads `start → D5 · $50 short`, the picker dims its option (`$50 short`), and a
//      send that fell back says `from D1 · toll short` as the watch opens; the exit line carries it (`exitExtras`)
//   2  the shaft and the forecast panel paint one forecast (the same `data-fc`, the same numbers) on the first pass and on the refine
//   3  the `+1 row` sheet names the next step's price (`next ◆4 or $600`), not a rate
//   4  a card joins the set on its buy only with `auto_insert: true`; without it the card is owned, off the set, and the sheet says so
//   5  the watched report's SPENT carries the send's toll (`tollOf`); STOLEN leads with the carry's thefts; a report label stored while
//      unknown reads by its name now (`renamer`)
//   6  the repeat badge reads as a switch (`repeat on · $40` / `repeat off`)
//   7  a way-blocker `because` stamped on another floor than the death's is re-stamped to its turn on the death's floor (`restamp`)
//   8  `depth ≥` reads `Vocabulary.depth_max` (best + 2, never under 8)
//
//   node web/tests/qa21.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { openPanel } from "./lib/frame.mjs";

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
const txt = (sel) => page.evaluate((s) => { const e = document.querySelector(s); return e && !e.hidden && e.getClientRects().length ? e.textContent.replace(/\s+/g, " ").trim() : null; }, sel);
const patchLineage = (patch) => page.evaluate(async (p) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  Object.assign(e.lineage, p); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
}, patch);

try {
  const rules = encodeURIComponent("foes>=1 → attack nearest\ndepth>=12 → bank");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=31&rules=${rules}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await sleep(300);

  // ---- 8, 5, 7, 6 (units, off the dev server's modules)
  const units = await page.evaluate(async () => {
    const tok = await import("/src/ui/tokens.ts"), watch = await import("/src/ui/watch.ts"), rep = await import("/src/ui/report.ts"), chain = await import("/src/ui/chain.ts"), death = await import("/src/ui/death.ts");
    const ledger = [{ t: 1, delta: 40, why: "banked D4" }, { t: 2, delta: -40, why: "repeat heal" }, { t: 3, delta: -50, why: "waystone D5" }, { t: 9, delta: 71, why: "banked D7" }, { t: 10, delta: -40, why: "repeat caustic" }];
    const nm = rep.renamer({ facts: ["item:blue=poison"], renamed: { "red potion?": "heal" } });
    const b = { text: "chase given up", t: 2640, depth: 2 };
    return {
      depth8: tok.depthNums(5, { conds: [], depth_max: 8 }).slice(-1)[0], depth22: tok.depthNums(20, { conds: [], depth_max: 22 }).slice(-1)[0], depthOld: tok.depthNums(7).slice(-1)[0],
      toll: JSON.stringify(watch.tollOf(ledger)), tollNone: JSON.stringify(watch.tollOf(ledger.slice(0, 2))),
      named: [nm("blue potion?"), nm("red potion?"), nm("violet potion?"), nm("goblin_archer")].join(" | "),
      restamped: JSON.stringify(chain.restamp(b, 7840, 4)), kept: JSON.stringify(chain.restamp({ text: "den took the heal, D3", t: 900, depth: 3 }, 7840, 4)), same: JSON.stringify(chain.restamp({ ...b, depth: 4 }, 7840, 4)),
      extras: death.exitExtras({ text: "died $0 · $74 carried", start: 1, start_short: true }),
    };
  });
  check(units.depth8 === 8 && units.depth22 === 22 && units.depthOld === 9, `\`depth ≥\` reads \`depth_max\` (best 5 → 8, best 20 → 22; without it best 7 → ${units.depthOld})`);
  check(units.toll === JSON.stringify([{ kind: "waystone D5", n: 1, gold: 50 }]) && units.tollNone === "[]", `the send's toll is a SPENT row of this run (${units.toll}; none before a second exit: ${units.tollNone})`);
  check(units.named === "poison | heal | violet potion? | goblin archer", `a stored unknown label reads by its name now (${units.named})`);
  check(units.restamped === JSON.stringify({ text: "chase given up", t: 7840, depth: 4 }) && units.kept === JSON.stringify({ text: "den took the heal, D3", t: 900, depth: 3 }) && units.same === JSON.stringify({ text: "chase given up", t: 2640, depth: 4 }),
    `a way blocker from another floor is the turn's own moment (${units.restamped}); an event link stays (${units.kept})`);
  check(units.extras === " · from D1 · toll short", `the exit line says the start fell back ("${units.extras}")`);

  // ---- 7 on the death screen: `← chase given up` links the death's floor, not a D2 jackal 5 000 ticks back
  await page.evaluate(() => {
    const d = { run_id: 999, depth: 4, cause: "goblin_archer", margin: "", verdict: "gap", baseline: 0.42, morgue: "slain by goblin_archer",
      trace: { turns: [{ t: 7830, row: -2, verb: { v: "explore" }, hp: 2, foes: 1, telegraphs: [] },
        { t: 7840, row: 0, verb: { v: "attack", a: "nearest" }, hp: 2, foes: 1, telegraphs: [], rows: [{ row: 0, why: "no target", because: { text: "chase given up", t: 2640, depth: 2 } }] }] },
      chain: [{ text: "chase given up", t: 2640, depth: 2 }],
      patches: [{ row: { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } }, insert_at: 0, survive: 1, forecast_delta: 0 }, { row: { conds: [{ k: "hp<", n: 20 }], verb: { v: "retreat" } }, insert_at: 0, survive: 0.42, forecast_delta: 0.05 }] };
    window.__riddle.go({ kind: "death", death: d });
  });
  await sleep(400);
  const ch = await page.evaluate(() => ({ at: [...document.querySelectorAll(".chain .chain-row .at")].map((e) => e.textContent.trim()), text: document.querySelector(".chain")?.textContent ?? "",
    patches: [...document.querySelectorAll(".patch.tablet")].map((b) => ({ text: b.textContent.replace(/\s+/g, " ").trim(), cls: b.className })) }));
  check(ch.at.length > 0 && ch.at.every((a) => a === "D4 · t7840") && !/D2/.test(ch.text), `the link names the death's floor and turn (${ch.at.join(" | ")})`);
  const ret = ch.patches.find((p) => /return/.test(p.text)), noGain = ch.patches.find((p) => /retreat/.test(p.text));
  // QA 778fa1b (qaU: `hp < 40% → return · survives 100%` looked best and cost D5 −54): an exit row's patch names its cost — `return early` beside the reach
  check(!!ret && /return early/.test(ret.text) && !/reach D\d/.test(ret.text), `an exit row's patch says it goes home early, its cost in a word, no reach number — Cut 23 §3 ("${ret?.text}")`);
  check(!!noGain && /survives 42% · no gain/.test(noGain.text) && /\bbelow\b/.test(noGain.cls), `a patch that survives no more than the base is dim, \`no gain\` ("${noGain?.text}")`);
  await shot("qa21-death");

  // ---- 1: the toll the purse cannot pay
  await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
  await waitFor((s) => s?.screen === "camp", "camp");
  await patchLineage({ best_depth: 12, heir: 3, gold: 20, waystones: [5, 9], start: 5, facts: ["item:leash", "vault"] });   // Cut 22: the toll is $5 × depth
  // QA 778fa1b (qaV; core): the toll is going — with the wire's toll at 0 nothing is short and the tablet names no price
  const tolls = await page.evaluate(async () => Object.fromEntries((await window.__riddle.engine.startForecast()).map((o) => [o.start, o.toll ?? 0])));
  if (!tolls[5] && !tolls[9]) {
    await sleep(500);
    const tab0 = await txt(".camp .start-tab");
    check(tab0 === "start → D5", `a free start names no price ("${tab0}")`);
    await page.locator(".camp .start-tab").click({ timeout: 5000 }); await sleep(900);
    const opts0 = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .start-opt")].map((b) => ({ text: b.textContent.replace(/\s+/g, " ").trim(), short: b.classList.contains("short") })));
    check(opts0.length === 3 && opts0.every((o) => !o.short && !/\$\d+( short)?$/.test(o.text.replace(/~\$-?\d+$/, ""))), `no start is short, none shows a toll (${opts0.map((o) => o.text).join(" | ")})`);
    await page.keyboard.press("Escape"); await sleep(150);
  } else {
    await sleep(500);
    const tab = await txt(".camp .start-tab");
    check(tab === "start → D5 · $25 short", `the tablet says the toll is short ("${tab}")`);
    await page.locator(".camp .start-tab").click({ timeout: 5000 }); await sleep(900);
    const opts = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .start-opt")].map((b) => ({ d: b.dataset.start, text: b.textContent.replace(/\s+/g, " ").trim(), short: b.classList.contains("short"), disabled: b.disabled })));
    const o5 = opts.find((o) => o.d === "5"), o9 = opts.find((o) => o.d === "9"), o1 = opts.find((o) => o.d === "1");
    check(o5?.short && /\$25 short$/.test(o5.text) && o9?.short && o9.disabled && /\$45 short$/.test(o9.text) && o1 && !o1.short, `the picker dims what the purse cannot pay (${opts.map((o) => o.text).join(" | ")})`);
    await shot("qa21-start-short");
    await page.keyboard.press("Escape"); await sleep(150);
    // the shaft starts where the sims do (the fake's forecast: D1, the toll unpaid)
    await page.evaluate(() => window.__riddle.emitForecast()); await sleep(600);
    const firstNotch = await page.evaluate(() => { const n = document.querySelector(".shaft .notch"); return n?.dataset.from ?? n?.dataset.d; });
    check(firstNotch === "1", `the shaft starts on the floor the forecast ran from (D${firstNotch})`);
    await page.evaluate(() => { window.__riddle.dev.speed = "fights"; });
    await page.locator(".gem.send").click({ timeout: 5000 });
    await waitFor((s) => s?.screen === "watch", "the watch");
    const banner = await page.evaluate(() => new Promise((res) => { const t0 = performance.now(); const poll = () => { const b = document.querySelector(".watch .banner.show"); if (b?.textContent) { res(b.textContent.trim()); return; } if (performance.now() - t0 > 6000) { res(null); return; } requestAnimationFrame(poll); }; poll(); }));
    check(banner === "from D1 · toll short", `the watch opens saying the start fell back ("${banner}")`);

  await shot("qa21-toll-short-watch");
  }
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=32&rules=${rules}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");

  // ---- 2: one forecast on the shaft and the panel — the first pass, then the refine
  await patchLineage({ best_depth: 5, heir: 3 }); await sleep(300);
  await page.evaluate(() => window.__riddle.insertRow({ conds: [{ k: "hp<", n: 30 }], verb: { v: "retreat" } }, 0));
  const samePaint = () => page.evaluate(() => {
    const fc = document.querySelector(".forecast"), sh = document.querySelector(".shaft");
    const panel = Object.fromEntries([...fc.querySelectorAll(".fc-bars .bar")].map((b) => [b.querySelector(".d")?.textContent, b.querySelector(".n")?.firstChild?.textContent?.trim()]));
    const shaft = Object.fromEntries([...sh.querySelectorAll(".notch:not(.fold)")].map((n) => [`D${n.dataset.d}`, n.querySelector(".dp")?.firstChild?.textContent?.trim()]));
    const both = Object.keys(shaft).filter((k) => panel[k] !== undefined && shaft[k] !== "?");
    return { fcP: fc.dataset.fc, fcS: sh.dataset.fc, refined: fc.dataset.refined, diff: both.filter((k) => panel[k] !== shaft[k]).map((k) => `${k} ${panel[k]}/${shaft[k]}`), n: both.length };
  });
  await page.waitForFunction(() => document.querySelector(".forecast")?.dataset.refined === "0", null, { timeout: 15000 }).catch(() => {});
  const p1 = await samePaint();
  await page.waitForFunction(() => document.querySelector(".forecast")?.dataset.refined === "1", null, { timeout: 15000 }).catch(() => {});
  const p2 = await samePaint();
  check(p1.refined === "0" && p1.fcP === p1.fcS && p1.n > 0 && !p1.diff.length, `the first pass: shaft and panel one forecast (#${p1.fcS}/#${p1.fcP}, ${p1.n} floors${p1.diff.length ? `; off ${p1.diff.join(", ")}` : ""})`);
  check(p2.refined === "1" && p2.fcP === p2.fcS && p2.n > 0 && !p2.diff.length, `the refine: shaft and panel one forecast (#${p2.fcS}/#${p2.fcP}, ${p2.n} floors${p2.diff.length ? `; off ${p2.diff.join(", ")}` : ""})`);

  // ---- 3: the `+1 row` sheet names the next step's price
  await patchLineage({ marks: 9, gold: 1000, heir: 6, best_depth: 6, facts: ["item:leash", "foe:goblin_archer", "foe:goblin_archer:ranged"] });
  await sleep(400);
  await page.evaluate(() => localStorage.setItem("riddle.unlocks.all", "1"));
  await openPanel(page, "unlocks", { all: true }); await sleep(300);
  await page.locator(".unlocks .card", { hasText: "+1 row" }).first().click({ timeout: 5000 }); await sleep(250);
  const next = await txt(".sheet-wrap .next-price"), climb = await txt(".sheet-wrap .gold-climb");
  check(next === "next ◆4 or $600 · each $ buy +$150" && climb === null, `the \`+1 row\` sheet names the next step's price ("${next}"; no rate: ${climb})`);
  await shot("qa21-row-next");
  await page.keyboard.press("Escape"); await sleep(150);

  // ---- 4: a card joins only on `auto_insert: true`
  const buyCard = (auto) => page.evaluate(async (auto) => {
    const r = window.__riddle; const e = r.engine;
    const real = e.unlocks.bind(e); e.unlocks = async () => (await real()).map((u) => u.id === "kite_archers" ? { ...u, auto_insert: auto } : u);
    e.unlockDeltas = async () => (await real()).map((u) => u.id === "kite_archers" ? { ...u, auto_insert: auto } : u);
    r.unlockCat = await e.unlocks();
    const before = r.rules.rows.filter((x) => x.verb.v === "tactic").length;
    const ok = await r.buy("kite_archers");
    return { ok, before, after: r.rules.rows.filter((x) => x.verb.v === "tactic").length };
  }, auto);
  // the sheet first (unowned): what the buy will do
  await page.evaluate(() => { const r = window.__riddle; const e = r.engine; const real = e.unlocks.bind(e); e.unlocks = async () => (await real()).map((u) => u.id === "kite_archers" ? { ...u, auto_insert: undefined } : u); e.unlockDeltas = e.unlocks; r.go({ kind: "camp" }); });
  await sleep(500);
  await openPanel(page, "unlocks", { all: true }); await sleep(300);
  await page.locator(".unlocks .card", { hasText: "kite archers" }).first().click({ timeout: 5000 }); await sleep(250);
  const joins = await txt(".sheet-wrap .card-joins");
  check(joins === "buy, then add to rules", `a card that will not join says so on its sheet (QA 778fa1b, qaV: unowned, never owned) ("${joins}")`);
  await shot("qa21-card-sheet");
  await page.keyboard.press("Escape"); await sleep(150);
  const b1 = await buyCard(undefined);
  check(b1.ok && b1.after === b1.before, `bought without \`auto_insert\`: owned, not in the set (${b1.before} → ${b1.after} card rows)`);
  const addChip = await page.evaluate(async () => { await new Promise((r) => setTimeout(r, 400)); return !!document.querySelector(".unlocks .add-card[data-card=kite_archers]"); });
  check(addChip, "its owned chip offers `add`");

  // ---- 6: the repeat badge is a switch
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, repeat: true, repeat_kinds: ["heal"], repeat_gold: 40, supplies: [{ id: 9001, kind: "heal", known: true, label: "heal" }] }; r.go({ kind: "camp" }); });
  await sleep(300);
  const badge = await txt(".cmd .tile .repeat-badge");
  check(badge === "repeat on · held ≤$40", `the repeat badge reads as a switch ("${badge}")`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`qa21: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`qa21: ok (${out.length} checks)`);
