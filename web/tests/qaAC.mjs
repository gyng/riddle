#!/usr/bin/env node
// QA 308f045 (qaAC, qaAD) client gates, on the fake engine, headless at 400 × 800:
//   · the death gem offers no one-tap apply until the whole-run measure lands (`…`); a lit tablet that harms reads `harms` on the stone
//   · the landed measure is matched patch by patch (the core re-ranks: a tablet never shows another patch's numbers)
//   · an exit's price reads beside its word (`return early · D6 54→30%`); a death move reads from→to (`death 100→0%`)
//   · the chain: an older row's reason reads under its row (`R2 drink heal · no item ← R4 drank heal`); a reason the row outlived
//     (`← never met` for a row that fired after it) is off the chain
//   · the repeat switch asks before it refunds a packed shelf (`refund 2?`, then off)
//   · an empty set is no send (`no rows`)
//   · a verdict opened from a report leads back to it (`report`)
//   · the route sheet: one "on" look (the unpicked stair a plain border); the untaken lane reads `untried`
//   · a free card is added by the sheet's one button (`add`)
//
//   node web/tests/qaAC.mjs            (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { deathDetails } from "./lib/frame.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
await deathDetails(page);   // death v2: this suite reads the trace, the ledger and the tablets under `details`
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen})`);
}
const camp = async () => { await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await waitFor((s) => s?.screen === "camp", "camp"); await sleep(200); };

const heal = { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" }, origin: "player" };
const hit = { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" };
const gas = { conds: [{ k: "foe_tag", t: "gas" }, { k: "adj>=", n: 1 }], verb: { v: "retreat" } };
const tele = { conds: [{ k: "foe_tag", t: "telegraph" }], verb: { v: "retreat" } };
const ret = { conds: [{ k: "hp<", n: 50 }], verb: { v: "return" } };

/** A death screen with `patches` (camp_pending) whose `deathDeltas` answers `filled` after `ms`. */
async function deathWith(patches, filled, ms = 600, extra = {}) {
  await page.evaluate(([patches, filled, rows, ms, extra]) => {
    const r = window.__riddle;
    r.sets[r.active] = { ...r.sets[r.active], rows };
    r.engine.deathDeltas = () => new Promise((res) => setTimeout(() => res(filled), ms));
    r.go({ kind: "death", death: { run_id: 0, depth: 8, cause: "gas", margin: "", verdict: "gap", baseline: 0, replays: 12, trace: { turns: [] }, morgue: "t1", rules: { rows }, patches, ...extra } });
  }, [patches, filled, [heal, hit], ms, extra]);
  await waitFor((x) => x?.screen === "death", "the death screen"); await sleep(120);
}
const tablets = () => page.evaluate(() => [...document.querySelectorAll(".patches button.patch")].map((b) => ({
  row: b.querySelector(".patch-main")?.textContent.trim(), top: b.classList.contains("top"), harms: b.classList.contains("harms"),
  delta: b.querySelector(".delta")?.textContent.replace(/\s+/g, " ").trim() ?? "", whole: b.querySelector(".whole")?.textContent.trim() ?? "" })));
const gemText = () => page.evaluate(() => document.querySelector(".patch-gem, .gem")?.textContent.replace(/\s+/g, " ").trim() ?? "");

try {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=26`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250);

  // ---- the gem waits for the whole-run measure; the landing is matched patch by patch; an exit reads its price
  {
    const ps = [{ row: gas, insert_at: 0, survive: 1, forecast_delta: 0, camp_pending: true },
                { row: ret, insert_at: 0, survive: 1, forecast_delta: 0, camp_pending: true, exits: true },
                { row: tele, insert_at: 0, survive: 10 / 12, forecast_delta: 0, camp_pending: true }];
    const whole = (reach, from, depth, death, deathFrom, harms) => ({ reach, reach_pm: 0.13, reach_from: from, reach_to: from + reach, death, death_pm: 0.02, depth, death_from: deathFrom, ...(harms ? { harms } : {}) });
    // the core re-ranked: the return first, then the gas retreat, then the telegraph retreat
    const filled = [{ ...ps[1], camp_pending: false, forecast_depth: 5, forecast_delta: -0.62, forecast_pm: 0.13, whole: whole(-0.62, 0.86, 5, -1, 1, false) },
                    { ...ps[0], camp_pending: false, forecast_depth: 6, forecast_delta: -0.24, forecast_pm: 0.13, whole: whole(-0.24, 0.54, 6, 0, 1, true) },
                    { ...ps[2], camp_pending: false, forecast_depth: 6, forecast_delta: -0.32, forecast_pm: 0.13, whole: whole(-0.32, 0.54, 6, 0, 1, true) }];
    await deathWith(ps, filled, 900);
    const g0 = await gemText();
    check(/…/.test(g0) && !/apply/.test(g0), `before the measure lands the gem offers no apply ("${g0}")`);
    const pend = await page.evaluate(async () => { const g = document.querySelector(".patch-gem"); const before = window.__riddle.rules.rows.length; g?.click(); await new Promise((r) => setTimeout(r, 100)); return window.__riddle.rules.rows.length === before && window.__riddle.screen === "death"; });
    check(pend, "a tap on the waiting gem applies nothing");
    await sleep(1200);
    const t = await tablets();
    const byRow = (re) => t.find((x) => re.test(x.row ?? ""));
    const g = byRow(/gas/), r = byRow(/return/), te = byRow(/telegraph/);
    check(g && /^reach D6 54→30%/.test(g.delta) && g.harms, `the gas retreat reads its own numbers (${g?.delta}), harming`);
    check(r && /^return early · reach D5 86→24%$/.test(r.delta) && /^death 100→0%$/.test(r.whole) && !r.harms, `the return reads its price and its death move ("${r?.delta}" · "${r?.whole}")`);
    check(te && /^reach D6 54→22%/.test(te.delta), `the telegraph retreat reads its own numbers (${te?.delta})`);
    check(r?.top && /12\/12\s*apply/i.test(await gemText()), `the lit tablet is the one that does not harm; the gem applies it ("${await gemText()}")`);
    await page.locator(".patches button.patch", { hasText: "gas" }).first().click(); await sleep(150);
    check(/risky/.test(await gemText()), `a lit tablet that harms says so on the stone ("${await gemText()}")`);
  }

  // ---- the chain: an older row's reason under its row; a reason the row outlived is off
  {
    const turn = (t, row, rows) => ({ t, row, verb: row === 0 ? { v: "attack", a: "tag:boss" } : { v: "attack", a: "nearest" }, hp: 10, foes: 2, rule_foes: 2, telegraphs: [], rows });
    const turns = [turn(100, 2, [{ row: 0, why: "not in view", because: { text: "never met", t: 100, depth: 8 } }, { row: 1, why: "no item", because: { text: "R4 drank heal at 17/36 hp", t: 60, depth: 8 } }]),
      turn(110, 0, []), turn(120, 0, []), turn(130, 0, []), turn(140, 0, []), turn(150, 0, []), turn(160, 0, []),
      turn(170, 2, [{ row: 0, why: "not in view", because: { text: "bloat mother slain D8", t: 165, depth: 8 } }])];
    await camp();
    await page.evaluate((turns) => {
      const r = window.__riddle;
      const rows = [{ conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "tag:boss" } }, { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } }];
      r.go({ kind: "death", death: { run_id: 0, depth: 8, cause: "gas", margin: "", verdict: "gap", baseline: 0, replays: 12, trace: { turns }, morgue: "t1", rules: { rows }, patches: [],
        chain: [{ text: "bloat mother slain D8", t: 165, depth: 8 }, { text: "never met", t: 100, depth: 8 }, { text: "R4 drank heal at 17/36 hp", t: 60, depth: 8 }] } });
    }, turns);
    await waitFor((x) => x?.screen === "death", "the chain death"); await sleep(200);
    const lines = await page.evaluate(() => [...document.querySelectorAll(".chain .chain-row")].map((l) => [...l.children].map((c) => c.textContent.trim()).join(" ").replace(/\s+/g, " ").trim()));
    check(!lines.some((l) => /never met/.test(l)), `a reason the row outlived (it fired after it) is off the chain (${lines.join(" | ")})`);
    check(lines.some((l) => /drink heal at 30% no item ← drank heal at 17\/36 hp/.test(l)), `an older row's reason reads under its row (${lines.join(" | ")})`);
    check(!lines.some((l) => /^← /.test(l)), `no bare \`←\` line (${lines.join(" | ")})`);
  }

  // ---- the repeat switch asks before it refunds a packed shelf
  {
    await camp();
    await page.evaluate(() => {
      const r = window.__riddle; r.__restock = [];
      r.engine.setRestock = async (on) => { r.__restock.push(on); return { ...r.lineage, repeat: on, supplies: [] }; };
      r.lineage = { ...r.lineage, gold: 300, repeat: true, repeat_kinds: ["heal"], repeat_gold: 48,
        supplies: [{ id: 901, kind: "heal", label: "heal potion", known: true, free: false }, { id: 902, kind: "blink", label: "blink scroll", known: true, free: false }] };
      r.go({ kind: "camp" });
    });
    await sleep(300);
    const sel = ".cmd .tile[data-tile=loadout] .repeat-badge";
    const b0 = await page.locator(sel).textContent().catch(() => null);
    await page.locator(sel).click({ timeout: 5000 }).catch(() => {}); await sleep(200);
    const b1 = await page.locator(sel).textContent().catch(() => null), c1 = await page.evaluate(() => window.__riddle.__restock.join());
    await page.locator(sel).click({ timeout: 5000 }).catch(() => {}); await sleep(300);
    const c2 = await page.evaluate(() => window.__riddle.__restock.join());
    check(/^repeat on/.test(b0 ?? "") && b1 === "refund 2?" && c1 === "" && c2 === "false", `a packed shelf: the first tap asks (\`${b0}\` → \`${b1}\`, setRestock ${c1 || "—"}), the second turns it off (${c2})`);
  }

  // ---- an empty set is no send
  {
    await camp();
    const s = await page.evaluate(async () => {
      const r = window.__riddle; const keep = r.sets[r.active].rows; r.sets[r.active].rows = []; r.rulesChanged(); r.go({ kind: "camp" });
      await new Promise((f) => setTimeout(f, 300));
      const g = document.querySelector(".gem.send"); const out = { text: g?.textContent.trim(), disabled: g?.disabled };
      r.sets[r.active].rows = keep; r.rulesChanged(); r.go({ kind: "camp" });
      return out;
    });
    check(s.disabled === true && s.text === "no rules", `an empty set's gem is off and says so ("${s.text}", disabled ${s.disabled})`);
  }

  // ---- a verdict opened from a report leads back to it
  {
    await camp();
    await page.evaluate(() => {
      const r = window.__riddle;
      const report = { elapsed_s: 60, runs: 1, sampled: false, learned: [], bests: [], found: [], pending: [], deaths: [{ cause: "gas", n: 1 }], reel: [], marks_earned: 0, hatched: [], lost: [], banked: 0, returned: 0 };
      r.go({ kind: "death", death: { run_id: 0, depth: 8, cause: "gas", margin: "", verdict: "gap", baseline: 0, replays: 12, trace: { turns: [] }, morgue: "t1", patches: [] }, kept: true, from: { report } });
    });
    await waitFor((x) => x?.screen === "death", "the kept death"); await sleep(200);
    const tile = page.locator(".cmd .tile[data-tile=report]");
    const has = await tile.count();
    if (has) { await tile.click(); await sleep(300); }
    check(has === 1 && (await state())?.screen === "report", `the verdict's \`report\` tile leads back (${has} tile, screen ${(await state())?.screen})`);
  }

  // ---- the route sheet: one "on" look; the untaken lane reads `untried`
  {
    await camp();
    const r = await page.evaluate(async () => {
      const a = window.__riddle;
      a.lineage = { ...a.lineage, facts: [...new Set([...(a.lineage.facts ?? []), "fork:4", "fork D4"])].filter((f) => f !== "biome:fens"), forks: [4] };
      a.go({ kind: "camp" });
      await new Promise((f) => setTimeout(f, 400));
      const front = document.querySelector(".shaft .frontier")?.textContent.trim() ?? null;
      const tab = document.querySelector(".route-line"); tab?.click();
      await new Promise((f) => setTimeout(f, 600));
      const opts = [...document.querySelectorAll(".route-picker .route-opt")].map((b) => ({ on: b.classList.contains("on"), border: getComputedStyle(b).borderTopColor, outline: getComputedStyle(b).outlineStyle }));
      document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
      return { front, opts };
    });
    const on = r.opts.filter((o) => o.on), off = r.opts.filter((o) => !o.on);
    check(r.front === null || /untried$/.test(r.front), `the untaken lane reads \`untried\` ("${r.front}")`);
    check(r.opts.length < 2 || (on.length === 1 && off.every((o) => o.outline === "none" && o.border !== "rgb(79, 138, 90)")), `one stair lit, the other a plain border (${JSON.stringify(r.opts)})`);
  }

  // ---- a free card: one button, `add`
  {
    await camp();
    const b = await page.evaluate(async () => {
      const a = window.__riddle;
      const u = (a.unlockCat ?? []).find((x) => /^(kite_archers|gas_step|thief_guard|pack_break)$/.test(x.id) && !x.owned);
      if (!u) return null;
      const { openUnlockSheet } = await import("/src/ui/unlocks.ts");
      openUnlockSheet(a, { ...u, cost: 0, gold: 0, available: true, needs: undefined, label: "card: kite archers", gated: false });
      await new Promise((f) => setTimeout(f, 300));
      const btns = [...document.querySelectorAll(".sheet-wrap .buy-pair button")].map((x) => x.textContent.trim());
      document.dispatchEvent(new KeyboardEvent("keydown", { key: "Escape" }));
      return btns;
    });
    check(b === null || (b.length === 1 && b[0] === "add"), `a free card's sheet has one button, \`add\` (${JSON.stringify(b)})`);
  }
} catch (e) {
  errors.push(String(e?.stack ?? e));
} finally {
  await browser.close();
}
for (const l of out) console.log(l);
for (const e of errors) console.log(`FAIL ${e}`);
const n = out.length;
console.log(`${failed || errors.length ? "FAIL" : "ok"}: ${n - failed}/${n} qaAC checks${errors.length ? ` · ${errors.length} errors` : ""}`);
process.exit(failed || errors.length ? 1 : 0);
