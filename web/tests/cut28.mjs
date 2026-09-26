#!/usr/bin/env node
// Cut 28 client gates (docs/CUT28.md), on the fake engine (its Cut 28 stand-ins), headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §1  the oath board: carved by the reveal ladder only once an oath is affordable; three carved tablets, each its constraint chips
//       (≤ 3 words a chip), its reward (icon + word) and its swear price; two taps swear one (the purse pays), the tablet and the shaft
//       then carry it with the forecast's share (`oath → D10 no drink 34%`); a band boss's counter reads where the wall is (`mother: ?`);
//       the bounty says what it pays and needs (`bounty · D13 · $×2 · item · reach`)
//   §2  the forecast move names its cause: after a run whose state changed (the stand-in: the party), the state's part is its own line
//       under the shaft (`party −1 jackal · death +6`) with no row edited and no divergence scene; a row edit adds the rows' `vs sent` line
//       and the state line stays; the report leads with decisions (the core's `lead`: the oath first), the oath's tally on its tablet, and
//       the ledger (exits, salvage, bones, spent, the reel) folded under one `details` tap
//   §4  the class picker opens from the portrait — while the wake's class chips stand too (AV)
//
//   node web/tests/cut28.mjs [--shots dir]        (part of `pnpm test` in web/)
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
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: shots ? 2 : 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
page.on("console", (m) => { if (/forecastMove|unavailable|listener|refused|failed/.test(m.text())) out.push(`note: ${m.text()}`); });
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: false }); };
const screen = () => page.evaluate(() => (window.__riddle?.booted ? window.__riddle.screen : "boot"));
async function until(pred, label, timeout = 20_000) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp", "camp"); await sleep(300); };
/** The lineage edited through a save round-trip (the fake's state), then the camp again. */
const withLineage = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const save = JSON.parse(await r.engine.save());
  new Function("L", src)(save.lineage);
  r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
}, `(${fn})(L)`);
const words = (s) => s.trim().split(/\s+/).filter((w) => /\p{L}/u.test(w)).length;

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=2801`, { waitUntil: "domcontentloaded" });
  await camp();
  // ---- §1: the ladder — a fresh purse affords no oath, so no board
  const fresh = await page.evaluate(() => ({ tab: !!document.querySelector(".oath-tab:not([hidden])"), board: (window.__riddle.lineage.oaths ?? []).length, gold: window.__riddle.lineage.gold, price: Math.min(...(window.__riddle.lineage.oaths ?? []).map((o) => o.price)) }));
  check(fresh.board === 3 && !fresh.tab && fresh.gold < fresh.price, `a fresh lineage ($${fresh.gold}, oaths from $${fresh.price}) shows no oath tablet (board ${fresh.board}, tablet ${fresh.tab})`);

  await withLineage((L) => { L.gold = 2000; L.best_depth = Math.max(L.best_depth, 9); L.heir = Math.max(L.heir, 2); });
  await camp();
  const tab = await until(() => { const t = document.querySelector(".oath-tab:not([hidden])"); return t ? t.textContent.replace(/\s+/g, " ").trim() : null; }, "the oath tablet");
  check(/^oaths 3$/.test(tab.replace(/^\S*\s*/, "")) || /oaths\s*3/.test(tab), `an affordable oath carves the oath tablet ("${tab}")`);
  await page.click(".oath-tab");
  await page.waitForSelector(".sheet-wrap .oath-board", { timeout: 5000 });
  await sleep(200);
  const board = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .oath.tablet")].map((t) => ({
    chips: [...t.querySelectorAll(".chip.oath-c")].map((c) => c.textContent.trim()),
    reward: t.querySelector(".oath-reward")?.textContent.trim() ?? "", ico: !!t.querySelector(".oath-reward .ico"),
    swear: t.querySelector(".chip.swear")?.textContent.trim() ?? "", counter: t.querySelector(".oath-counter")?.textContent.trim() ?? "" })));
  check(board.length === 3, `the board holds three oaths (${board.length})`);
  check(board.every((o) => o.chips.length >= 1 && o.chips.every((c) => words(c) <= 3)), `each oath is its constraint as chips, ≤ 3 words a chip (${board.map((o) => o.chips.join(" | ")).join(" ; ")})`);
  check(board.every((o) => o.reward && o.ico && words(o.reward) <= 3), `each carries its reward, an icon and ≤ 3 words (${board.map((o) => o.reward).join(" · ")})`);
  check(board.every((o) => /^swear \$\d+$/.test(o.swear)), `each carries its swear price (${board.map((o) => o.swear).join(" · ")})`);
  const slayer = board.find((o) => o.counter);
  if (slayer) check(/^mother: (\?|fire)$/.test(slayer.counter), `an oath at a band boss names its counter as a fact ("${slayer.counter}")`);
  await shot("cut28-oath-board");
  const gold0 = await page.evaluate(() => window.__riddle.lineage.gold);
  const price = Number(board[0].swear.replace(/\D/g, ""));
  await page.locator(".sheet-wrap .oath.tablet .chip.swear").first().click();
  const armed = await page.evaluate(() => document.querySelector(".sheet-wrap .oath.tablet .chip.swear")?.textContent.trim());
  check(/^ok \$\d+$/.test(armed ?? ""), `the first tap arms the swear ("${armed}")`);
  await page.locator(".sheet-wrap .oath.tablet .chip.swear").first().click();
  await until(() => window.__riddle.lineage.oath, "the sworn oath");
  const gold1 = await page.evaluate(() => window.__riddle.lineage.gold);
  check(gold0 - gold1 === price, `swearing pays its price ($${gold0} → $${gold1}, price $${price})`);
  await page.keyboard.press("Escape"); await sleep(200);
  const onShaft = await until(() => { const s = document.querySelector(".shaft .shaft-oath .oath-share:not(.pending)"); return s ? { shaft: document.querySelector(".shaft .shaft-oath").textContent.replace(/\s+/g, " ").trim(), tab: document.querySelector(".oath-tab").textContent.replace(/\s+/g, " ").trim(), sworn: document.querySelector(".oath-tab").classList.contains("sworn") } : null; }, "the oath's share on the shaft", 15_000);
  check(/\d+%|<\d+%/.test(onShaft.shaft), `the sworn oath rides the shaft with its share ("${onShaft.shaft}")`);
  check(onShaft.sworn && /^.*oath → .*\d+%/.test(onShaft.tab), `the tablet carries the sworn oath ("${onShaft.tab}")`);
  await shot("cut28-oath-shaft");

  // the wall's path and the bounty's terms (the pure readers, on the lineage's walls and bounty)
  const wb = await page.evaluate(async () => {
    const m = await import("/src/ui/forecast.ts");
    const app = { lineage: { counters: [], walls: [{ boss: "bloat_mother", title: "Mother", depth: 13, slain: false, known: false, fact: "mother: ?", learn: "meet her" }] } };
    return { wall: m.wallCounter(app, "bloat_mother"), known: m.wallCounter({ lineage: { counters: [{ boss: "goblin_warlord", text: "attack boss" }] } }, "goblin_warlord"),
      bounty: m.bountyText({ depth: 13, pays: "$×2 · item", needs: "reach" }), bare: m.bountyText({ depth: 13 }) };
  });
  check(wb.wall === "mother: ?" && wb.known === "warlord: attack", `the wall names its counter fact ("${wb.wall}", known "${wb.known}")`);
  check(wb.bounty === "bounty · D13 · $×2 · item · reach" && wb.bare === "bounty · D13 · $×2 · reach", `the bounty says what it pays and needs ("${wb.bounty}"; bare "${wb.bare}")`);

  // ---- §2: a run whose state changed — the state's line, no row edited, no scene
  await page.evaluate(() => { window.__riddle.watchMode = "fast"; });
  await page.evaluate(() => document.querySelector("button.gem.send")?.click());
  await until(() => window.__riddle.screen === "watch", "the watch");
  for (let i = 0; i < 300; i++) {
    const s = await screen();
    if (s === "camp") break;
    if (s === "watch") await page.evaluate(() => document.querySelector('[data-tile="skip"]')?.click());
    else if (s === "exit") await page.evaluate(() => (document.querySelector(".sheet-wrap .btn.primary") ?? document.querySelector(".sheet-wrap button:not(.stud)"))?.click());
    else if (s === "death") await page.evaluate(() => document.querySelector('.console [data-tile="camp"]')?.click());
    else if (s === "report") await page.evaluate(() => document.querySelector(".gem.camp-gem")?.click());
    await sleep(250);
  }
  await camp();
  const st = await until(() => { const l = document.querySelector(".shaft-vs-host .shaft-state"); return l ? { text: l.textContent.replace(/\s+/g, " ").trim(), k: l.dataset.k, vs: !!document.querySelector(".shaft-vs-host .shaft-vs"), scene: !(document.querySelector(".div-line")?.hidden ?? true) } : null; }, "the state's line", 15_000).catch(() => null);
  check(!!st && /^party [−+]\d+\b.* · (death|bank|D\d+) [−+]\d+$/.test(st.text), `a state change reads as its own line ("${st?.text}")`);
  check(!!st && !st.vs && !st.scene, `with no row edited there is no rows' move and no scene (vs ${st?.vs}, scene ${st?.scene})`);
  await shot("cut28-attributed-move");
  // a row edit: the rows' `vs sent` joins it, the state's line stays
  await page.evaluate(() => { const r = window.__riddle; const row = r.rules.rows[0]; const c = row.conds.find((x) => typeof x.n === "number"); if (c) c.n = c.n >= 50 ? 30 : c.n + 20; else r.rules.rows.reverse(); r.rulesChanged(); });
  const both = await until(() => { const a = document.querySelector(".shaft-vs-host .shaft-state"), b = document.querySelector(".shaft-vs-host .shaft-vs:not(.pending)"); return a && b ? { state: a.textContent.replace(/\s+/g, " ").trim(), rows: b.textContent.replace(/\s+/g, " ").trim() } : null; }, "both lines", 15_000).catch(() => null);
  if (!both || !/^party/.test(both.state)) out.push(`note: ${await page.evaluate(() => JSON.stringify({ fm: window.__riddle.fmove && window.__riddle.fmove.parts.map((p) => p.kind), due: window.__riddle.moveDue, off: window.__riddle.moveOff, has: typeof window.__riddle.engine.forecastMove }))}`);
  check(!!both && /^party/.test(both.state) && /^vs sent/.test(both.rows), `after a row edit each part has its line ("${both?.state}" · "${both?.rows}")`);
  if (both) await shot("cut28-attributed-move-rows");

  // ---- §2: the report leads with decisions (an absence with the oath sworn)
  // (in the page: the fake's stand-in keeps the sworn oath in its live state, not its save)
  await page.evaluate(async () => { const r = window.__riddle; const rep = await r.runOfflineChunked(8 * 3600); await r.refresh(); r.adoptSets(); r.go({ kind: "report", report: rep, absence: true }); });
  await until(() => window.__riddle?.booted && window.__riddle.screen === "report", "the absence report", 60_000);
  await sleep(400);
  const rep = await page.evaluate(() => {
    const sheet = document.querySelector(".report-sheet"), kids = [...sheet.children];
    const idx = (el) => (el ? kids.indexOf(el.closest(".report-sheet > *")) : -1);
    const details = sheet.querySelector(".report-details"), fold = sheet.querySelector(".details-fold");
    const secs = (root) => [...root.querySelectorAll(".rsec > .label")].map((l) => l.textContent.trim());
    return { lead: [...sheet.querySelectorAll(".news-line.decision")].map((l) => ({ k: l.dataset.k, text: l.textContent })),
      oath: sheet.querySelector(".rsec.oath-sec")?.textContent.replace(/\s+/g, " ").trim() ?? null, oathAt: idx(sheet.querySelector(".rsec.oath-sec")), foldAt: idx(fold),
      hidden: details?.hidden, fold: fold?.textContent.trim(), inDetails: details ? secs(details) : [], above: secs(sheet).filter((s) => !details || !secs(details).includes(s)),
      exitsFolded: !!details?.querySelector(".exit-lines") || !sheet.querySelector(".exit-lines") };
  });
  const dbg = await page.evaluate(() => ({ oath: window.__riddle.lineage.oath, view: JSON.stringify(window.__riddle.view?.report?.lead ?? null), ro: JSON.stringify(window.__riddle.view?.report?.oath ?? null) }));
  if (!rep.lead.length) out.push(`note: ${JSON.stringify(dbg)}`);
  check(rep.lead.length >= 1 && rep.lead[0].k === "oath", `the report leads with a decision, the oath first (${rep.lead.map((l) => `${l.k}: ${l.text}`).join(" · ")})`);
  check(!!rep.oath && /kept \d+\/\d+/.test(rep.oath) && rep.oathAt >= 0 && rep.oathAt < rep.foldAt, `the oath's tablet is above the fold with its tally ("${rep.oath}")`);
  check(rep.hidden === true && /details$/.test(rep.fold ?? ""), `the ledger is folded under \`details\` (hidden ${rep.hidden}, "${rep.fold}")`);
  const LEDGER = ["salvaged", "bones", "spent", "found", "kept", "reel", "stolen", "shelved", "renown"];
  check(rep.above.every((s) => !LEDGER.includes(s)) && rep.exitsFolded, `no ledger section above the fold (above: ${rep.above.join(", ") || "none"}; folded: ${rep.inDetails.join(", ")})`);
  await shot("cut28-report");
  await page.click(".report .details-fold");
  const open = await page.evaluate(() => !document.querySelector(".report-details").hidden && document.querySelector(".details-fold").getAttribute("aria-expanded") === "true");
  check(open, "a tap on `details` unfolds the ledger");
  await page.click(".gem.camp-gem"); await camp();

  // ---- §4: a refine that never answers (AU's `kite archers` card) leaves the first pass standing — its `…` marks go
  await page.evaluate(() => { const r = window.__riddle; r.refineStuckMs = 1500; r.engine.forecastRefine = () => new Promise(() => {}); const row = r.rules.rows[r.rules.rows.length - 1]; r.rules.rows.push({ ...row, conds: [...row.conds] }); r.rules.rows.pop(); r.rules.rows.reverse(); r.rulesChanged(); });
  const rough = await until(() => document.querySelector(".shaft.rough") ? 1 : null, "the first pass", 10_000).catch(() => null);
  const settled = await until(() => { const sh = document.querySelector(".shaft"); return sh && !sh.classList.contains("rough") && !sh.classList.contains("stale") && !/…/.test(sh.querySelector(".notches")?.textContent ?? "") ? 1 : null; }, "the first pass standing", 8_000).catch(() => null);
  check(!!rough && !!settled, `a refine with no answer leaves the first pass standing, no \`…\` (first pass ${!!rough}, settled ${!!settled})`);
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows.reverse(); r.go({ kind: "camp" }); });
  await camp();

  // ---- §4: the class picker opens from the portrait while the wake's class chips stand
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, heir: Math.max(2, r.lineage.heir), class_offer: [{ class: r.lineage.class, signature: "shield_bash", level: 1, opens: 1 }, { class: "rogue", signature: "vanish", level: 1, opens: 1 }] }; r.go({ kind: "camp" }); });
  await camp();
  const pr = await page.evaluate(() => { const p = document.querySelector(".console .portrait"); return { tag: p?.tagName, chips: document.querySelectorAll(".classes-offer .chip").length }; });
  await page.click(".console .portrait", { position: { x: 30, y: 30 } });
  const sheet = await page.waitForSelector(".sheet-wrap .classes", { timeout: 3000 }).then(() => true).catch(() => false);
  check(pr.tag === "BUTTON" && pr.chips >= 2 && sheet, `the portrait opens the class picker beside the wake's chips (${pr.tag}, ${pr.chips} chips, sheet ${sheet})`);
} catch (e) {
  check(false, `threw: ${e.message}`);
}
const real = errors.filter((e) => !/favicon|Failed to load resource|WebGL|GPU stall|swiftshader/i.test(e));
check(real.length === 0, `no console errors (${real.slice(0, 3).join(" | ")})`);
await browser.close();
console.log(out.join("\n"));
console.log(`cut28: ${out.length - failed}/${out.length} ok`);
process.exit(failed ? 1 : 0);
