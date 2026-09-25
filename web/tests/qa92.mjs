#!/usr/bin/env node
// QA on 92eb880 (players M and N; eval/qa/92eb880.qaM.md, .qaN.md) — the client batch, on the fake engine, headless at 400 × 800:
//   · the core's hand-offs: a first-pass forecast reads rough (dim, `…`) on the shaft and the panel; `Death.nothing_beats_base` heads the
//     patch block `nothing beats base · base 100%` and no patch reads `below bar`; a shadowed row is a dim tablet with `↑ R1`
//   · the death: the camp's reach lands on the patches (QA 778fa1b: the order and the lit tablet stay; a loss dim); `+0%` inside the ±, not `~0`; no
//     `saved him` / cage-loot note; the bar names the hero who died; a lost ally is a line, not a chip
//   · the camp: a 0 % notch dims (label and all), a `depth ≥ N → bank` row caps the shaft (`D3 · bank`); a numeric cond chip opens on its
//     values; the verb picker names another row's unoffered verb; `· free` on the kennel's leash, its × arms before it drops; the edit
//     tile does not toggle the editor off; the chosen trait reads chosen; the unlock tiles (no `⊘` a gold price covers, `free`, the owned
//     row lists a verb, `+1 row` says `rows 4 → 5`); the empty gold sheet reads `no movements`
//   · the report: one tile order; a `shadowed by R1` pending line keeps its reason; LEARNED reads `blink (ashen)` on its own row and a foe's
//     tags after ` · `; the gold line says banked / returned, never `home`; a plateau patch names its floor
//   · core hand-offs (c46f60a): a card's `UnlockInfo.stall` on its tile (`stall +11%`); the XP bar off `classes[c].next`
//   · the watch's helpers: the speed badge rounds (`1.3`, `16`); an unopened cage near the hero is seen
//
//   node web/tests/qa92.mjs [--shots dir]        (part of `pnpm test` in web/)
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
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen})`);
}
async function settle(max = 15_000) {
  const t = Date.now(); let clear = 0;
  while (Date.now() - t < max) { const s = await state(); clear = s && s.booted && !s.busy ? clear + 1 : 0; if (clear >= 2) break; await sleep(120); }
  await sleep(150);
}
const go = (v) => page.evaluate((x) => window.__riddle.go(x), v);
const mod = (path, fn, arg) => page.evaluate(async ([p, f, a]) => { const m = await import(p); return new Function("m", "a", `return (${f})(m, a)`)(m, a); }, [path, String(fn), arg]);
/** Paint a fabricated forecast through the app's own listeners (the shaft, the panel, the editor's marks). */
const paintForecast = (f) => page.evaluate((f) => { const r = window.__riddle; const x = { ...r.lastForecast, ...f }; r.lastForecast = x; r.shadow = x.shadowed_by ?? []; for (const fn of r.fcListeners) fn(x); }, f);
const text = (sel) => page.evaluate((s) => [...document.querySelectorAll(s)].map((e) => e.textContent.replace(/\s+/g, " ").trim()), sel);

const t0 = Date.now();
try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=11`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await page.evaluate(() => { localStorage.removeItem("riddle.unlocks.all"); });
  await settle();
  // a lineage past the reveal ladder's first steps (edit, loadout, unlocks, gems), best D6
  await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    Object.assign(e.lineage, { heir: 3, best_depth: 6, marks: 1, gold: 700, graveyard: [{ heir: 1, depth: 2, cause: "rat", deeds: [] }, { heir: 2, depth: 4, cause: "rat", deeds: [] }],
      gold_ledger: [{ t: 10, delta: 40, why: "returned D1" }], trait_offer: ["brave", "cowardly"], trait: "brave",
      supplies: [{ id: 100000, kind: "leash", known: true, label: "leash", free: true }], unlocks: [...new Set([...(e.lineage.unlocks ?? []), "throw", "row5"])] });
    b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
  });
  await waitFor((s) => s?.screen === "camp", "camp"); await settle();

  // ---- 1a. a first pass is rough on the shaft and the panel; the refine is not
  const depths = (rs) => rs.map((reach, i) => ({ depth: i + 1, reach, pm: 0.04 }));
  await paintForecast({ depths: depths([1, 0.95, 0.8, 0.6, 0.4, 0.2, 0]), known_to: 7, causes: [{ cause: "rat", share: 1 }], ends: { bank: 0.2, return: 0.3, death: 0.5, gold: 20, pm: 0.05 }, refined: false });
  await sleep(150);
  let f = await page.evaluate(() => ({ rough: document.querySelector(".shaft")?.classList.contains("rough"), pm: [...document.querySelectorAll(".shaft .notch .pm")].map((x) => x.textContent), panel: document.querySelector(".forecast")?.classList.contains("rough"), panelPm: [...document.querySelectorAll(".forecast .fc-bars .pm")].map((x) => x.textContent) }));
  check(f.rough && f.pm.length > 0 && f.pm.every((x) => x.endsWith("…")) && f.panel && f.panelPm.every((x) => x.endsWith("…")), `a first-pass forecast reads rough on the shaft and the panel, each ± trailing \`…\` (${f.pm.join(" ")} · panel ${f.panel})`);
  await paintForecast({ refined: true });
  await sleep(150);
  f = await page.evaluate(() => ({ rough: document.querySelector(".shaft")?.classList.contains("rough"), pm: [...document.querySelectorAll(".shaft .notch .pm")].map((x) => x.textContent), panel: document.querySelector(".forecast")?.classList.contains("rough") }));
  check(!f.rough && !f.panel && f.pm.every((x) => !x.includes("…")), `the refine lands plain: no \`…\`, not dim (${f.pm.join(" ")})`);
  // ---- N5: a 0 % notch dims, label and all
  const z = await page.evaluate(() => { const n = document.querySelector('.shaft .notch[data-d="7"]'); return { zero: n?.classList.contains("zero"), color: n ? getComputedStyle(n.querySelector(".dl")).color : "", acc: getComputedStyle(document.documentElement).getPropertyValue("--acc").trim() }; });
  check(z.zero && !/255, 2[0-9]{2}, /.test(z.color), `the next floor at 0 % is dim, not gold (${z.color})`);
  // a `depth ≥ 5 → bank` row caps the shaft: `D5 · bank`, D6+ capped (Cut 20 §5: the bounty notch past best + 1 is its own)
  await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [{ k: "depth>=", n: 5 }], verb: { v: "bank" } }, r.rules.rows.length); });
  await sleep(200);
  const cap = await page.evaluate(() => ({ d5: document.querySelector('.shaft .notch[data-d="5"] .dl')?.textContent, capped: [...document.querySelectorAll(".shaft .notch.capped:not(.bounty)")].map((n) => n.dataset.d) }));
  check(cap.d5 === "D5 · bank" && cap.capped.join(",") === "6,7", `a bank row marks its floor and dims the ones past it (${cap.d5}; capped ${cap.capped.join(",")})`);
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows.pop(); r.rulesChanged(); });
  await settle();

  // ---- 1c. a shadowed row: the engine's read of the set (the fake mirrors `RuleSet::shadowed_by`) → a dim tablet `↑ R1`
  await page.evaluate(() => { const r = window.__riddle; r.editing = true; r.rules.rows.splice(0, r.rules.rows.length,
    { conds: [{ k: "hp<", n: 30 }], verb: { v: "return" }, origin: "player" }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" }, { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" }, origin: "player" });
    r.rulesChanged(); r.go({ kind: "camp" }); });
  await settle(); await sleep(600);
  let sh = await page.evaluate(() => [...document.querySelectorAll(".editor .row")].map((r) => ({ on: r.classList.contains("shadowed"), mark: r.querySelector(".shadow-mark")?.textContent ?? "", op: getComputedStyle(r.querySelector(":scope > .chips")).opacity })));
  check(sh.length === 3 && !sh[0].on && !sh[1].on && sh[2].on && sh[2].mark === "↑ R1" && Number(sh[2].op) < 0.8, `R3 under R1 (both hp < 30%) is a dim tablet marked \`↑ R1\` (${sh.map((x) => `${x.on ? "dim" : "·"}${x.mark ? ` ${x.mark}` : ""}`).join(" | ")})`);
  await shot("qa92-shadow");
  // an edit clears the mark until the forecast of the rules now lands; moving R3 to the top frees it
  await page.evaluate(() => { const r = window.__riddle; const [x] = r.rules.rows.splice(2, 1); r.rules.rows.unshift(x); r.rulesChanged(); r.go({ kind: "camp" }); });
  await settle(); await sleep(600);
  sh = await page.evaluate(() => [...document.querySelectorAll(".editor .row")].map((r) => r.classList.contains("shadowed")));
  check(sh.every((x) => !x), `the drink row moved above the return: nothing shadowed (${sh.join(",")})`);
  // the compact tablets carry the mark too
  await paintForecast({ shadowed_by: [null, null, 1] });
  await page.evaluate(() => { const r = window.__riddle; r.editing = false; r.go({ kind: "camp" }); });
  await sleep(300);
  await paintForecast({ shadowed_by: [null, null, 1] });
  await sleep(150);
  const compact = await text(".editor.compact .row.shadowed .shadow-mark");
  check(compact.join() === "↑ R2", `a compact tablet carries the mark (${compact.join() || "none"})`);
  await page.evaluate(() => { window.__riddle.editing = true; window.__riddle.go({ kind: "camp" }); });
  await settle();

  // ---- friction: a numeric cond chip opens on its values — a threshold is one tap from the sheet
  await page.locator(".editor .row").first().locator("button.chip.cond").first().click({ timeout: 5000 }); await sleep(200);
  const nums = await text(".sheet-wrap .grid.nums.now .chip.num");
  const lit = await text(".sheet-wrap .grid.nums.now .chip.num.on");
  check(nums.length > 3 && lit.length === 1, `the cond sheet opens on the chip's values, the current one lit (${nums.join(" ")}; lit ${lit.join()})`);
  const pick = nums.find((n) => n !== lit[0]);
  await page.locator(".sheet-wrap .grid.nums.now .chip.num", { hasText: new RegExp(`^${pick}$`) }).first().click({ timeout: 5000 }); await sleep(200);
  const r1 = await page.evaluate(() => window.__riddle.rules.rows[0].conds[0]);
  check(`${r1.n}%` === pick || `${r1.n}` === pick, `one tap on a value sets it (${JSON.stringify(r1)} ← ${pick})`);
  // the verb picker: another row's verb the vocabulary does not offer sits dim with its reason
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows[0].verb = { v: "drink", a: "mystery" }; r.go({ kind: "camp" }); });
  await settle();
  await page.locator(".editor .row").nth(1).locator(".chip.verb").first().click({ timeout: 5000 }); await sleep(200);
  const lockedV = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chip.verb.locked.off")].map((c) => ({ t: c.textContent.replace(/\s+/g, " ").trim(), tag: c.tagName })));
  check(lockedV.some((v) => /mystery/.test(v.t) && /unknown/.test(v.t) && v.tag === "SPAN"), `another row's unoffered verb is listed dim with why, never a button (${lockedV.map((v) => v.t).join(" | ")})`);
  await page.keyboard.press("Escape"); await sleep(150);

  // ---- the edit tile never turns the editor off; the chosen trait reads chosen
  await page.evaluate(() => { const r = window.__riddle; r.editing = false; r.go({ kind: "camp" }); }); await settle();
  const editTile = page.locator(".cmd .tile[data-tile=edit]");
  await editTile.click({ timeout: 5000 }); await sleep(150); await editTile.click({ timeout: 5000 }); await sleep(150);
  check(!(await page.locator(".editor.compact").count()) && await page.locator(".editor .row .chip.cond").count() > 0, "a second tap on `edit` leaves the editor open");
  // QA 1a2a4a9 (O): the tick is text — a text dump reads which trait is his; the others carry none
  const tick = await page.evaluate(() => ({ on: document.querySelector(".chip.trait.on")?.textContent ?? null, off: [...document.querySelectorAll(".chip.trait:not(.on)")].map((x) => x.textContent) }));
  check(!!tick.on && tick.on.startsWith("✓ ") && tick.off.every((t) => !t.includes("✓")), `the chosen trait carries a tick in its text (${tick.on} · ${tick.off.join(" · ")})`);

  // ---- the loadout: the kennel's leash reads `· free`; its × arms (`drop`), then drops
  await openPanel(page, "loadout");
  let sup = await text(".supplies .chip.item");
  check(sup.length === 1 && /^leash · free ?×$/.test(sup[0]), `the free leash reads \`· free\` ("${sup[0]}")`);
  await page.locator(".supplies .chip.item .x").first().click({ timeout: 5000 }); await sleep(250);
  sup = await text(".supplies .chip.item");
  check(sup.length === 1 && /drop$/.test(sup[0]), `the first tap on its × arms it (\`drop\`), nothing dropped ("${sup[0]}")`);
  await page.locator(".supplies .chip.item .x").first().click({ timeout: 5000 }); await sleep(400);
  sup = await text(".supplies .chip.item");
  check(sup.length === 0, `the second tap drops it (${sup.join(" / ") || "shelf empty"})`);
  await page.keyboard.press("Escape"); await sleep(100);

  // ---- the unlock tiles and sheets
  await page.evaluate(() => { const r = window.__riddle; const cat = [
    { id: "auto_supply", cost: 3, owned: false, available: false, needs: "◆2 more", gold: 600 },
    { id: "rogue", cost: 0, owned: false, available: false, needs: "bank once", gold: 0 },
    { id: "row6", cost: 2, owned: false, available: false, needs: "◆1 more", gold: 300 },
    { id: "throw", cost: 2, owned: true, available: false, gold: 0 },
    { id: "row5", cost: 2, owned: true, available: false, gold: 0 },
    { id: "corridor_fighting", cost: 3, owned: false, available: false, needs: "◆2 more", gold: 450, delta: 0.001, pm: 0.02, stall: 0.11, insert_at: 5 } ];
    r.engine.unlocks = async () => cat; r.engine.unlockDeltas = async () => cat; localStorage.setItem("riddle.unlocks.all", "1"); r.go({ kind: "camp" }); });
  await settle();
  await openPanel(page, "unlocks"); await sleep(200);
  const tiles = await page.evaluate(() => [...document.querySelectorAll(".unlocks .card")].map((c) => ({ label: c.querySelector(".card-main > span")?.textContent, needs: c.querySelector(".needs")?.textContent ?? "", cost: c.querySelector(".cost")?.textContent })));
  const tt = (l) => tiles.find((x) => x.label === l);
  check(tt("auto: restock")?.needs === "◆2 more", `a marks shortfall the gold covers carries no \`⊘\` ("${tt("auto: restock")?.needs}")`);
  const corr = await page.evaluate(() => [...document.querySelectorAll(".unlocks .card")].find((c) => /corridor/.test(c.textContent))?.querySelector(".stall-risk")?.textContent);
  check(corr === "stall +11", `a card that raises the stall share says so on its tile ("${corr}")`);
  check(tt("class: rogue")?.cost === "free", `a door that costs nothing reads \`free\` ("${tt("class: rogue")?.cost}")`);
  const owned = await text(".unlocks .chips.owned .chip");
  check(owned.some((o) => /verb: throw/.test(o)) && !owned.some((o) => /\+1 row/.test(o)), `the owned row lists a bought verb, not the counted steps (${owned.join(" · ")})`);
  await page.locator(".unlocks .card", { hasText: "+1 row" }).first().click({ timeout: 5000 }); await sleep(200);
  const eff = await text(".sheet-wrap .unlock-sheet .effect-line");
  const vmax = await page.evaluate(() => window.__riddle.vocab.max_rows);
  check(eff.join() === `rows ${vmax} → ${vmax + 1}`, `the \`+1 row\` sheet says what it gives ("${eff.join()}")`);
  await page.keyboard.press("Escape"); await sleep(100);
  await page.locator(".unlocks .card", { hasText: "auto: restock" }).first().click({ timeout: 5000 }); await sleep(200);
  const needsLine = await text(".sheet-wrap .unlock-sheet .needs-line:not(.gold-short)");
  check(needsLine.join() === "◆2 more", `the sheet's marks line drops its \`⊘\` while \`$ buy\` is on ("${needsLine.join()}")`);
  await page.keyboard.press("Escape"); await sleep(100);
  await page.keyboard.press("Escape"); await sleep(100);
  await page.evaluate(() => localStorage.removeItem("riddle.unlocks.all"));

  // ---- the XP bar is the engine's ladder (`classes[c].next`), not a client one
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, trait_offer: [], class_offer: [], classes: { ...r.lineage.classes, [r.lineage.class]: { level: 4, xp: 240, next: 960 } } }; r.go({ kind: "camp" }); });
  await sleep(250);
  const xpw = await page.evaluate(() => document.querySelector(".console .portrait .xp .fill")?.style.width);
  check(xpw === "25%", `the camp's XP bar reads the engine's next-level cost (240 / 960 → ${xpw})`);
  // ---- the gold sheet on an empty ledger
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, gold_ledger: [] }; r.go({ kind: "camp" }); }); await settle();
  await page.locator(".strip button.gold").click({ timeout: 5000 }); await sleep(200);
  const gold = await text(".sheet-wrap .gold-sheet .gold-lines");
  check(gold.join() === "no movements", `an empty gold sheet says so, no lone \`·\` ("${gold.join()}")`);
  await page.keyboard.press("Escape"); await sleep(100);

  // ---- 1b + N6: the death — nothing beats base; the camp's reach re-ranks; notes; the dead hero's bar; the lost ally a line
  const turns = [10, 20, 30].map((t) => ({ t, row: 1, verb: { v: "attack", a: "nearest" }, hp: 2, foes: 1, telegraphs: [] }));
  const P = (row, survive, extra = {}) => ({ row, insert_at: 0, survive, forecast_delta: 0, ...extra });
  const dice = { run_id: 0, depth: 8, cause: "ogre", margin: "", verdict: "dice", baseline: 1, trace: { turns }, morgue: "Riddle morgue · seed 1 · heir 2 · run 7\nclass fighter L1 · trait curious", nothing_beats_base: true,
    notes: ["R2 attack saved him.", "The cage opens: one is his.", "Took the axe +1 from the cage."],
    patches: [P({ conds: [{ k: "foe_tag", t: "telegraph" }], verb: { v: "retreat" } }, 1, { below_bar: true }), P({ conds: [{ k: "hp<", n: 30 }], verb: { v: "rest" } }, 1, { below_bar: true })] };
  await go({ kind: "death", death: dice, lost: ["ally hound"] }); await waitFor((s) => s?.screen === "death", "the dice death"); await sleep(300);
  let d = await page.evaluate(() => ({ head: document.querySelector(".patches .patches-head")?.textContent, surv: [...document.querySelectorAll("button.patch .surv")].map((x) => x.textContent), gem: document.querySelector(".death .gem")?.textContent,
    notes: [...document.querySelectorAll(".death-notes .note")].map((n) => n.textContent), heir: document.querySelector(".death .topbar .heir")?.textContent, trait: document.querySelector(".death .topbar .trait")?.textContent,
    egg: document.querySelector(".death .eggs-line")?.textContent, eggBtn: !!document.querySelector(".death .eggs-line button, .death .chip.egg") }));
  check(d.head === "nothing beats unpatched 100%" && d.surv.every((s) => s === "survives 100%") && /edit/i.test(d.gem ?? ""), `a dice death nothing beats: the block says so, no \`below bar\`, the gem is \`edit\` (${d.head} · ${d.surv.join(" | ")} · gem ${d.gem})`);
  check(d.notes.length === 0, `no \`saved him\` and no cage loot over a death (${d.notes.join(" | ") || "none"})`);
  check(d.heir === "♟2" && d.trait === "curious", `the bar names the hero who died (${d.heir} · ${d.trait})`);
  check(/ally hound fell/.test(d.egg ?? "") && !d.eggBtn, `a lost ally is a line, not a chip ("${d.egg}")`);
  await shot("qa92-dice");
  // the camp's reach lands: a loss drops to the bottom, dim, never the gem's; inside the ± the higher survival leads
  await page.evaluate(() => {
    const r = window.__riddle; r.__dd = r.engine.deathDeltas;
    const pat = (conds, v, survive) => ({ row: { conds, verb: { v } }, insert_at: 0, survive, forecast_delta: 0, camp_pending: true });
    const ps = [pat([{ k: "foe_tag", t: "telegraph" }], "retreat", 1), pat([{ k: "hp<", n: 20 }], "rest", 0.75), pat([{ k: "hp<", n: 20 }], "return", 1), pat([{ k: "foes>=", n: 3 }], "retreat", 0.6)];
    const deltas = [-0.04, 0.01, 0, 0.09];
    r.engine.deathDeltas = () => new Promise((res) => setTimeout(() => res(ps.map((p, i) => ({ ...p, camp_pending: false, forecast_depth: 8, forecast_delta: deltas[i], forecast_pm: 0.02 }))), 400));
    r.go({ kind: "death", death: { run_id: 0, depth: 7, cause: "gas", margin: "", verdict: "gap", baseline: 0, trace: { turns: [] }, morgue: "", patches: ps } });
  });
  await sleep(150);
  const before = await page.evaluate(() => document.querySelector(".gem.patch-gem .gem-n")?.textContent);
  await sleep(900);
  d = await page.evaluate(() => ({ rows: [...document.querySelectorAll(".patches > button.patch")].map((b) => ({ t: b.querySelector(".chips-inline")?.textContent.replace(/\s+/g, " ").trim(), neg: b.classList.contains("neg"), top: b.classList.contains("top"), reach: b.querySelector(".delta")?.textContent })), gem: document.querySelector(".gem.patch-gem .gem-n")?.textContent }));
  await page.evaluate(() => { const r = window.__riddle; r.engine.deathDeltas = r.__dd; });
  const order = d.rows.map((x) => x.t).join(" | ");
  // QA 778fa1b (qaU: the lit tablet and the gem moved 1 → 2 on their own ~5 s after arrival): the landing fills the reach and dims a loss,
  // never re-orders the list or moves the lit tablet (the core's `rank_patches` order stands)
  check(order === "foe: telegraph → retreat | hp < 20% → rest | hp < 20% → return | foes ≥ 3 → retreat", `the landing keeps the core's order (${order})`);
  check(d.rows[0]?.neg && d.rows[0]?.top && !d.rows.slice(1).some((x) => x.neg) && d.gem === "100%" && before === "100%", `a loss is dim and says its move; the lit tablet and the gem stay (gem ${before} → ${d.gem}; ${d.rows.map((x) => x.reach).join(" · ")})`);
  check(/^reach D8 ≈ ±\d+$/.test(d.rows[1]?.reach ?? "") && d.rows[2]?.reach === "return early", `a move inside the ± reads \`≈ ±N\` (Cut 24 §4; was \`≈\` alone, QA 778fa1b); an exit says so ("${d.rows[1]?.reach}" · "${d.rows[2]?.reach}")`);
  await shot("qa92-rerank");

  // ---- the report: one tile order; the shadowed pending line; LEARNED; the gold words; the plateau's floor
  const L = await page.evaluate(() => window.__riddle.lineage);
  const base = { elapsed_s: 3600, runs: 16, sampled: false, bests: [], found: [], deaths: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [],
    xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 },
    learned: ["foe:stray:lock", "item:ashen=blink", "item:amber=speed", "alert:rising"],
    pending: ["R3 fired 0 of 16 runs: hp < 30% → drink heal · shadowed by R1"],
    exits: [{ carried: 100, keep_pct: 100, kept: 100, spent: 0, spent_on: [], text: "banked $100 · $100 carried" }, { carried: 50, keep_pct: 60, kept: 30, spent: 0, spent_on: [], text: "returned $30 · $50 carried · keeps 60%" }],
    gold: { home: 130, salvage: 5, wake: 0, spent: 0 },
    stall: { row: 2, fired: 13, text: "R3 bank ended 13 runs, none past D6", patches: [{ row: { conds: [{ k: "depth>=", n: 7 }], verb: { v: "bank" } }, insert_at: 2, replace: true, survive: 0.17, forecast_delta: 0.17 }] } };
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows.splice(0, r.rules.rows.length, { conds: [{ k: "hp<", n: 30 }], verb: { v: "return" } }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } }, { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }); });
  const tileOrder = [];
  for (const [b, rt] of [[1, 12], [11, 5]]) {
    await go({ kind: "report", report: { ...base, banked: b, returned: rt } }); await waitFor((s) => s?.screen === "report", "report"); await sleep(250);
    tileOrder.push((await text(".report .tiles .tile .label")).join(" "));
  }
  check(tileOrder.every((t) => t === "runs best marks banked returned deaths"), `the tiles keep one order whichever leads (${tileOrder.join(" / ")})`);
  const rep = await page.evaluate(() => {
    const sec = (l) => [...document.querySelectorAll(".report .rsec")].find((x) => new RegExp(l, "i").test(x.querySelector(".label")?.textContent ?? ""));
    return { pending: [...(sec("pending")?.querySelectorAll("li") ?? [])].map((l) => l.textContent), items: [...(sec("learned")?.querySelectorAll(".chips.items .chip") ?? [])].map((c) => c.textContent),
      foes: [...(sec("learned")?.querySelectorAll(".chips:not(.items) .chip") ?? [])].map((c) => c.textContent), gold: document.querySelector(".report .gold-line")?.textContent, stall: document.querySelector(".stall .patch .surv")?.textContent };
  });
  check(rep.pending.some((p) => /R3 fired 0 of 16 runs: hp < 30% → drink heal · shadowed by R1$/.test(p)), `PENDING keeps the shadowing row (${rep.pending.join(" | ")})`);
  check(rep.items.join(" | ") === "blink (ashen) | speed (amber)" && rep.foes.includes("stray · lock") && rep.foes.includes("alert rises · alert ≥ open"), `LEARNED: identities on their own row, a foe's tags after \` · \` (${rep.items.join(" | ")} · ${rep.foes.join(" | ")})`);
  check(/^\+\$100 banked · \+\$30 returned · \+\$5 salvage$/.test(rep.gold ?? "") && !/home/.test(rep.gold ?? ""), `the gold line says banked / returned, never \`home\` ("${rep.gold}")`);
  check(/^reach D7 17% · base 0%$/.test(rep.stall ?? ""), `a plateau patch names its floor ("${rep.stall}")`);
  await shot("qa92-report");

  // ---- the watch's helpers
  const rates = await mod("/src/ui/watch.ts", (m) => [1.332247798006322, 2, 4, 16, 1].map(m.rateText));
  check(rates.join(" ") === "1.3 2 4 16 1", `the speed badge rounds (${rates.join(" ")})`);
  const cage = await mod("/src/ui/watch.ts", (m) => {
    const w = 30, h = 20, tiles = Array(w * h).fill("floor"), seen = Array(w * h).fill(true);
    const snap = (cx) => { const t = [...tiles]; t[5 * w + cx] = "vault"; return { w, h, tiles: t, seen, hero: { x: 3, y: 5 } }; };
    return [m.cageNear(snap(10)), m.cageNear(snap(25))];
  });
  check(cage[0] === true && cage[1] === false, `an unopened cage near the hero is seen, a far one not (${cage.join(",")})`);
} catch (e) {
  errors.push(`walk aborted: ${e.stack ?? e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
const secs = ((Date.now() - t0) / 1000).toFixed(1);
if (failed || errors.length) { console.error(`qa92: FAIL (${failed} assertion(s), ${errors.length} error(s), ${secs}s)`); process.exit(1); }
console.log(`qa92: ok (${out.length} checks · ${secs}s)`);
