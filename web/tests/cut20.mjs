#!/usr/bin/env node
// Cut 20 client gates (docs/CUT20.md), on the fake engine, headless at 400 × 800:
//   §2  the party: a tap on a kennel card takes it into the party; a second tap never dismisses it — its `×` does
//   §4  the stake names the exit that keeps and what a death keeps: `carry $N · bank keeps $N · death $0 · bank at D9` (`keeps` never
//       alone); a death's ledger line carries the loadout's re-pack (`repeat −$80`), and its gold sheet lists it
//   §5  the shaft's bounty notch reads `D{best+2} ×2` (a gold glint); the report names the night's bounty (`bounty D12 · taken $412`,
//       `bounty D12 · missed`)
//   AD  the report's `died · trace` chip opens the trace, never the gold sheet (the chip sits in its own column beside the line);
//       `nothing beats base` folds the candidates (dim, behind `others`), the gem `edit`
//
//   node web/tests/cut20.mjs [--shots dir]        (part of `pnpm test` in web/)
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
const sheetLabel = () => page.evaluate(() => document.querySelector(".sheet-wrap .label")?.textContent?.trim() ?? "");
const closeSheets = async () => { await page.keyboard.press("Escape"); await sleep(120); };

try {
  // ---- §4 the stake line, watched: a bank row (`depth>=9 → bank`) and the fake's death keep (0)
  const bankRules = encodeURIComponent("foes>=1 → attack nearest\ndepth>=9 → bank");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=157&autosend=1&speed=fast&rules=${bankRules}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
  const stakes = await page.evaluate(() => new Promise((res) => {
    const seen = new Set(), t0 = performance.now();
    const poll = () => { const s = document.querySelector(".watch .stake"); if (s && !s.hidden && s.textContent) seen.add(s.textContent.replace(/\s+/g, " ").trim());
      if (performance.now() - t0 > 4000 || window.__riddle.screen !== "watch") { res([...seen]); return; } requestAnimationFrame(poll); };
    poll();
  }));
  const full = stakes.filter((x) => /^carry \$\d+( −\$\d+(?: \w+)?)? · bank keeps \$\d+ · death \$0( · [^·]+)* · bank at D9$/.test(x));
  check(stakes.length > 0 && full.length > 0, `the stake reads \`carry $N · bank keeps $N · death $0 · bank at D9\` (${stakes.slice(0, 2).join(" | ") || "never shown"})`);
  check(stakes.every((x) => !/\bkeeps\b/.test(x) || /· death \$\d+/.test(x)), `\`keeps\` never shows without the death's share (${stakes.filter((x) => !/death \$/.test(x)).slice(0, 2).join(" | ") || "none alone"})`);
  await shot("cut20-stake");

  // ---- the camp: the bounty notch on the shaft (the fake's bounty floor is best + 2)
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  // a fresh lineage has no best, so no bounty (its shaft is D1 alone): a lineage with best D4 through the save
  await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    Object.assign(e.lineage, { best_depth: 4, heir: 3 }); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
  });
  await page.waitForFunction(() => !!document.querySelector(".shaft .notch.bounty"), null, { timeout: 10_000 }).catch(() => {});
  const notch = await page.evaluate(() => { const n = document.querySelector(".shaft .notch.bounty"); return { text: n?.querySelector(".dl")?.textContent ?? "", best: window.__riddle.lineage.best_depth, bounty: window.__riddle.lineage.bounty?.depth, glint: n ? getComputedStyle(n.querySelector(".hex")).boxShadow : "" }; });
  check(notch.text === `D${notch.best + 2} bounty $×2` && notch.bounty === notch.best + 2 && /rgb/.test(notch.glint), `the shaft's bounty notch reads \`D${notch.best + 2} ×2\` with a glint ("${notch.text}", lineage bounty D${notch.bounty})`);
  await shot("cut20-shaft");

  // ---- §2 the party: a tap selects, a second tap keeps it, `×` drops it (a companion put in the kennel through the save)
  await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    const pet = (id, name) => ({ id, kind: "jackal", name, level: 2, tags: ["pack", "fast"], gen: 1, rules: { rows: [] }, max_rows: 2, hp: 12, max_hp: 12 });
    const L = e.lineage; L.kennel = [pet(7001, "Thix"), pet(7002, "Skog")]; L.party = [];
    b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
  });
  await sleep(400);
  const party = async () => page.evaluate(() => ({ ids: window.__riddle.lineage.party.map((c) => c.id), cards: [...document.querySelectorAll(".party .card.comp")].map((c) => ({ name: c.querySelector(".name small")?.textContent?.trim(), on: c.classList.contains("on"), x: !!c.querySelector(".drop-pet") })) }));
  await page.locator(".cmd .tile[data-tile=party]").click({ timeout: 5000 }).catch(() => {});
  await sleep(200);
  const cardBtn = (name) => page.locator(".party .card.comp", { hasText: name }).locator(".comp-main");
  await cardBtn("Thix").click({ timeout: 5000 }); await sleep(300);
  const p1 = await party();
  check(p1.ids.includes(7001) && p1.cards.find((c) => c.name === "Thix")?.x, `a tap takes Thix into the party, his card carries \`×\` (party ${p1.ids.join(",")})`);
  await cardBtn("Thix").click({ timeout: 5000 }); await sleep(300);
  const p2 = await party();
  check(p2.ids.includes(7001), `a second tap does not dismiss him (party ${p2.ids.join(",") || "empty"})`);
  await shot("cut20-party");
  await page.locator(".party .card.comp", { hasText: "Thix" }).locator(".drop-pet").click({ timeout: 5000 }); await sleep(300);
  const p3 = await party();
  check(!p3.ids.includes(7001), `\`×\` drops him (party ${p3.ids.join(",") || "empty"})`);
  await closeSheets();

  // ---- §4 a death's repeat charge: the ledger after the death's exit line carries `repeat heal −$80`
  const turns = [10, 20, 30].map((t) => ({ t, row: 1, verb: { v: "attack", a: "nearest" }, hp: 2, foes: 1, telegraphs: [] }));
  const P = (row, survive, extra = {}) => ({ row, insert_at: 0, survive, forecast_delta: 0, ...extra });
  await page.evaluate(() => {
    const L = window.__riddle.lineage;
    L.gold_ledger = [{ t: 100, delta: -40, why: "bought heal" }, { t: 200, delta: 0, why: "died D5" }, { t: 200, delta: 40, why: "wake pay" }, { t: 200, delta: -80, why: "repeat heal" }];
  });
  const death = (extra = {}) => ({ run_id: 0, depth: 5, cause: "jackal", margin: "", verdict: "gap", baseline: 0.4, trace: { turns }, morgue: "Riddle morgue · seed 1 · heir 2 · run 7",
    line: { carried: 78, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $78 carried · keeps 0% · +$40 wake" },
    patches: [P({ conds: [{ k: "hp<", n: 30 }], verb: { v: "rest" } }, 0.9)], ...extra });
  await go({ kind: "death", death: death() }); await waitFor((s) => s?.screen === "death", "the death"); await sleep(300);
  const rep = await page.evaluate(() => document.querySelector(".death .repeat-line")?.textContent ?? "");
  check(rep === "repeat −$80", `the death's gold carries the re-pack: "${rep || "absent"}"`);
  await shot("cut20-repeat");
  if (rep) {
    await page.locator(".death .repeat-line").click({ timeout: 3000 }); await sleep(250);
    const gold = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .gold-sheet .lrow")].map((l) => l.textContent.replace(/\s+/g, " ").trim()));
    check(gold.some((g) => /^−\$80\s*repeat heal$/.test(g)), `its gold sheet lists the re-pack (${gold.join(" | ")})`);
    await closeSheets();
  }
  // no re-pack: no line
  await page.evaluate(() => { window.__riddle.lineage.gold_ledger = [{ t: 200, delta: 0, why: "died D5" }]; });
  await go({ kind: "death", death: death() }); await sleep(300);
  check(!(await page.locator(".death .repeat-line").count()), "no re-pack, no `repeat` line");

  // ---- AD: `nothing beats base` folds the candidates (dim, behind `others`); the gem is `edit`
  await go({ kind: "death", death: death({ verdict: "dice", baseline: 1, nothing_beats_base: true, patches: [P({ conds: [{ k: "hp<", n: 30 }], verb: { v: "rest" } }, 1, { below_bar: true }), P({ conds: [], verb: { v: "return" } }, 1), P({ conds: [{ k: "foes>=", n: 2 }], verb: { v: "retreat" } }, 1)] }) });
  await sleep(300);
  const nb = await page.evaluate(() => { const b = document.querySelector(".death .patches"); const vis = [...b.querySelectorAll("button.patch")].filter((x) => x.getClientRects().length);
    return { head: b.querySelector(".patches-head")?.textContent, visible: vis.length, all: b.querySelectorAll("button.patch").length, dim: [...b.querySelectorAll("button.patch")].every((x) => x.classList.contains("below")), more: b.querySelector(".patches-more")?.textContent?.replace(/\s+/g, " ").trim(), gem: document.querySelector(".death .gem")?.textContent?.trim() }; });
  check(nb.head === "nothing beats unpatched 100%" && nb.visible === 0 && nb.all === 3 && nb.dim && /^others 3$/.test(nb.more ?? "") && /edit/i.test(nb.gem ?? ""), `under \`nothing beats base\` no patch shows as one: folded behind \`${nb.more}\`, dim, the gem \`${nb.gem}\` (${nb.visible}/${nb.all} visible)`);
  await shot("cut20-nothing-beats");
  await page.locator(".death .patches-more").click({ timeout: 3000 }); await sleep(200);
  const nb2 = await page.evaluate(() => [...document.querySelectorAll(".death .patches button.patch")].filter((x) => x.getClientRects().length).length);
  check(nb2 === 3, `\`others\` unfolds them (${nb2} shown)`);

  // ---- §5 + AD: the report — the bounty line; the `died · trace` chip opens the trace
  const trace = { turns };
  const x = (text, kept, pct) => ({ carried: 80, keep_pct: pct, kept, spent: 0, spent_on: [], text, trace, run_id: 1 });
  const report = (bounty) => ({ elapsed_s: 100, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "goblin", n: 1 }], pending: [], reel: [], marks_earned: 1, tamed: [], hatched: [], lost: [],
    xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 1, returned: 1, bounty,
    exits: [x("returned $22 · $37 carried · keeps 60%", 22, 60), x("died $0 · $80 carried · keeps 0% · bones: 8 items on D8 · ◆+1", 0, 0), x("banked $106 · $106 carried · keeps 100% · ◆+3 (1 frontier)", 106, 100)] });
  await go({ kind: "report", report: report({ depth: 12, taken: true, gold: 412 }) }); await waitFor((s) => s?.screen === "report", "the report"); await sleep(300);
  const b1 = await page.evaluate(() => document.querySelector(".report .bounty-line")?.textContent ?? "");
  check(b1 === "bounty D12 · taken $412", `the report names the bounty taken: "${b1}"`);
  await shot("cut20-report");
  const chip = await page.evaluate(() => {
    const line = [...document.querySelectorAll(".exit-lines .ledger-line")].find((l) => /^died/.test(l.querySelector(".ledger-btn")?.textContent ?? ""));
    const c = line.querySelector(".chip").getBoundingClientRect(), b = line.querySelector(".ledger-btn").getBoundingClientRect();
    return { label: line.querySelector(".chip").textContent, cx: c.x + c.width / 2, cy: c.y + c.height / 2, top: c.y, left: c.x, btnRight: b.right, apart: c.left >= b.right - 0.5 || c.top >= b.bottom + 6 };
  });
  check(chip.apart, `the \`${chip.label}\` chip stands apart from its line's button (chip left ${Math.round(chip.left)} ≥ line right ${Math.round(chip.btnRight)})`);
  await page.mouse.click(chip.cx, chip.cy); await sleep(250);
  const lbl1 = await sheetLabel();
  check(/^trace/.test(lbl1), `a tap on \`${chip.label}\` opens the trace, not the gold sheet (sheet "${lbl1}")`);
  await closeSheets();
  await page.mouse.click(chip.cx, chip.top + 2); await sleep(250);   // a tap on the chip's top edge (AD's finger) is still the chip's
  const lbl2 = await sheetLabel();
  check(/^trace/.test(lbl2), `a tap on the chip's top edge opens the trace too (sheet "${lbl2}")`);
  await closeSheets();
  await go({ kind: "report", report: report({ depth: 12, taken: false, gold: 0 }) }); await sleep(300);
  const b2 = await page.evaluate(() => document.querySelector(".report .bounty-line")?.textContent ?? "");
  check(b2 === "bounty D12 · missed", `…or missed: "${b2}"`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut20: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut20: ok (${out.length} checks)`);
