#!/usr/bin/env node
// Cut 17 — the frame's gates (docs/CUT17.md, docs/UI.md §5–6), on the fake engine, headless at 400 × 800:
//   · a fresh lineage's camp shows ≤ 8 interactive elements: the bar ($ only), the tablets (compact), the shaft (D1), the gem SEND
//   · each step of the reveal ladder carves its tile on its trigger (and not before), glinting once: first death → edit · first gold
//     → loadout · first mark → unlocks (◆ on the bar); the class picker from the first death · an item kept → vault · a salvage → forge · a companion →
//     party · a 3rd row → the shaft's gems · 5 heirs → ledger, chronicle, set tabs
//   · the unlock panel is a short list (three, `more` for the catalogue)
//   · every sheet and panel carries a close stud, and the stud closes it
//   · every command tile has a pressed state (the tile sinks while held)
//   · every place (camp · watch · death · report) has the bar and the console, the primary action in the gem slot
//   · QA 23ed91f (player K): the death's line whole above its seal, the top patch above the console, the gem labelled `apply`; the
//     watch's last frame keeps the heir that ran and a gem (`verdict`); a `+1 vault` carves the vault tile; and the batch's smaller
//     fixes (trace columns, report order, bests, stall arithmetic, cage pick, keep legend, hp callout, alert, shaft, prices, …)
//
//   node web/tests/ui.mjs [--shots dir]        (part of `pnpm test` in web/)
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

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state();
    if (s?.screen === "exit" && s.vault) { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(100); continue; }
    if (pred(s)) return s;
    await sleep(80);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
async function settle(max = 15_000) {
  const t = Date.now(); let clear = 0;
  while (Date.now() - t < max) { const s = await state(); clear = s && s.booted && !s.busy ? clear + 1 : 0; if (clear >= 2) break; await sleep(120); }
  await sleep(150);
}
/** Patch the fake's saved lineage and reload it through the save (the camp remounts). */
const patchSave = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  new Function("e", src)(e);
  b.engine = JSON.stringify(e);
  return r.importSave(JSON.stringify(b));
}, `(${fn})(e)`);
/** The interactive elements a player sees above the fold (buttons, inputs, links), open sheets included. */
const interactive = () => page.evaluate(() => [...document.querySelectorAll("button, input, select, textarea, a[href], [role=button]")].filter((b) => {
  if (b.closest("[inert]") || !b.getClientRects().length || getComputedStyle(b).visibility === "hidden") return false;
  const r = b.getBoundingClientRect();
  return r.bottom > 0 && r.top < innerHeight && r.width > 0 && r.height > 0;
}).map((b) => (b.getAttribute("aria-label") || b.textContent || b.className).replace(/\s+/g, " ").trim().slice(0, 24)));
const tiles = () => page.evaluate(() => [...document.querySelectorAll(".console .cmd .tile:not(.empty)")].map((t) => ({ id: t.dataset.tile, reveal: t.classList.contains("reveal") })));
const tileIds = async () => (await tiles()).map((t) => t.id);

/** QA 23ed91f (player K): the batch's checks on fabricated screens (the fake engine; `/src/...` modules through the dev server). */
async function qaK() {
  const go = (screen) => page.evaluate((v) => window.__riddle.go(v), screen);
  const mod = (path, fn, arg) => page.evaluate(async ([p, f, a]) => { const m = await import(p); return new Function("m", "a", `return (${f})(m, a)`)(m, a); }, [path, String(fn), arg]);
  // ---- 1. the death: the whole line above the seal; the top patch above the console; the gem `100%` over `apply`; the trace's
  //      columns as they have content; a death's line without its repeated `keeps 0%`
  const turns = [7668, 7679, 7690, 7702, 7713].map((t) => ({ t, row: 1, verb: { v: "attack", a: "nearest" }, hp: 1, foes: 1, telegraphs: [], rows: [{ row: 0, why: "no item" }] }));
  const kDeath = { run_id: 0, depth: 5, cause: "monkey", margin: "1 hp short · 5 unknown unused", verdict: "gap", baseline: 0.25,
    notes: ["Trophy: no heal to D5.", "The monkey stole the ashen scroll?"],
    line: { text: "died $0 · $111 carried · keeps 0% · bones: 11 items on D5 · ◆+7 · +$40 wake", kept: 0, carried: 111, keep_pct: 0, spent: 0, spent_on: [] },
    trace: { turns }, morgue: "t10 a line", patches: [
      { row: { conds: [{ k: "hp<", n: 20 }], verb: { v: "rest" } }, insert_at: 0, survive: 1, forecast_delta: 0.08 },
      { row: { conds: [{ k: "hp<", n: 20 }, { k: "foes>=", n: 1 }], verb: { v: "retreat" } }, insert_at: 0, survive: 0.67, forecast_delta: 0.08 },
      { row: { conds: [{ k: "hp<", n: 20 }], verb: { v: "read", a: "unknown" } }, insert_at: 0, survive: 0.75, forecast_delta: -0.08 }] };
  await go({ kind: "death", death: kDeath }); await waitFor((x) => x?.screen === "death", "K's death"); await sleep(400);
  let d = await page.evaluate(() => {
    const r = (s) => document.querySelector(s)?.getBoundingClientRect();
    const cause = r(".death-line .cause"), seal = r(".death-line .verdict"), top = r("button.patch.top"), cons = r("main.death > footer.console"), well = document.querySelector(".death-well");
    const range = document.createRange(); range.selectNodeContents(document.querySelector(".death-line .cause")); const lines = [...range.getClientRects()];
    return { line: document.querySelector(".death-line .cause")?.textContent, causeBottom: Math.max(...lines.map((l) => l.bottom)), sealTop: seal.top, topPatch: top ? [top.top, top.bottom] : null, consoleTop: cons.top, scroll: well.scrollTop,
      gemN: document.querySelector(".gem.patch-gem .gem-n")?.textContent, gemW: document.querySelector(".gem.patch-gem .gem-w")?.textContent,
      heads: [...document.querySelectorAll(".death .trace thead th")].map((t) => t.textContent), ledger: document.querySelector(".death .ledger-line")?.textContent };
  });
  check(d.line === "monkey · D5 · 5 unknown unused", `the headline drops the hp margin inside a longer one ("${d.line}")`);
  check(d.causeBottom <= d.sealTop + 0.5, `the verdict line ends above the seal: nothing under it (line to ${Math.round(d.causeBottom)}, seal from ${Math.round(d.sealTop)})`);
  check(!!d.topPatch && d.topPatch[1] <= d.consoleTop && d.scroll === 0, `the top patch is whole above the console at 400 × 800, unscrolled (${d.topPatch?.map(Math.round).join("–")} ≤ ${Math.round(d.consoleTop)})`);
  check(d.gemN === "100%" && d.gemW === "apply", `the gem reads its number over the word \`apply\` ("${d.gemN}" / "${d.gemW}")`);
  check(d.heads.join(",") === "t,rule,hp,foes", `no empty \`tele\` column (${d.heads.join(",")})`);
  // QA 0c6e126 (qaZ: `died $0` read as "died carrying $0"): the kept and the carried named apart — `died · kept $0 · $111 carried`
  check(/^died · kept \$0 · \$111 carried · bones/.test(d.ledger ?? ""), `a death's line says \`kept $0\` once, no \`keeps 0%\` after it ("${d.ledger}")`);
  await shot("ui-qaK-death");
  // QA 912e135 (qaW: the seal, the banner, the trace rows and `R1 no item` answered no tap; the header's `$40` was no button here): each
  // is a button — the header opens GOLD, a row's name opens the editor on it
  const taps = await page.evaluate(() => ({ seal: !!document.querySelector(".death-line button.verdict"), cause: !!document.querySelector(".death-line button.cause-btn"),
    rows: [...document.querySelectorAll(".death button.row-link")].map((b) => b.textContent), gold: !!document.querySelector(".death .topbar button.stat.gold") }));
  check(taps.seal && taps.cause && taps.gold && taps.rows.length >= 6 && taps.rows.includes("drink unknown at 30%"), `the death's seal, banner, header \`$\` and row names are buttons (${JSON.stringify(taps)})`);
  await page.locator(".death .topbar button.stat.gold").click({ timeout: 3000 }); await sleep(250);
  check(!!(await page.locator(".sheet-wrap .gold-sheet").count()), "the death's header `$` opens the gold sheet");
  await page.keyboard.press("Escape"); await sleep(150);
  // QA 0c6e126 (qaZ: the row's name jumped to the editor with no way back): it opens the row's sheet over the death screen; its `edit`
  // opens the editor on the row
  await page.locator(".death button.row-link", { hasText: /^drink unknown at 30%$/ }).first().click({ timeout: 3000 });
  await sleep(250);
  const rs = await page.evaluate(() => ({ sheet: document.querySelector(".sheet-wrap .row-sheet")?.textContent ?? null }));
  check(!!rs.sheet && /^hp < 30% → drink unknown/.test(rs.sheet) && /acted \d+\/\d+/.test(rs.sheet), `a row's name opens its sheet over the death screen ("${rs.sheet}")`);
  await page.locator(".sheet-wrap .row-sheet button.row-edit").click({ timeout: 3000 });
  await waitFor((x) => x?.screen === "camp", "the camp from a row's sheet");
  check(await page.evaluate(() => window.__riddle.editing === true), "the row sheet's `edit` opens the editor");
  turns[4] = { ...turns[4], telegraphs: ["monkey reaches"] };
  await go({ kind: "death", death: { ...kDeath, trace: { turns } } }); await sleep(250);
  d = await page.evaluate(() => [...document.querySelectorAll(".death .trace thead th")].map((t) => t.textContent));
  check(d.includes("tele"), `the \`tele\` column is back when a turn has a telegraph (${d.join(",")})`);

  // the patches' reach is the camp's own, landing after the paint (`deathDeltas`): `reach …` first, then `reach D6 +8% ±3`
  await page.evaluate((d) => {
    const r = window.__riddle; r.__origDD = r.engine.deathDeltas;
    const filled = d.patches.map((p, i) => ({ ...p, camp_pending: false, forecast_depth: 6, forecast_delta: [0.08, 0.01, -0.05][i], forecast_pm: 0.03 }));
    r.engine.deathDeltas = () => new Promise((res) => setTimeout(() => res(filled), 700));
    r.go({ kind: "death", death: { ...d, patches: d.patches.map((p) => ({ ...p, camp_pending: true })) } });
  }, kDeath);
  await sleep(150);
  const pend = await page.evaluate(() => ({ reach: [...document.querySelectorAll("button.patch .delta")].map((x) => x.textContent), gem: document.querySelector(".gem.patch-gem .gem-n")?.textContent }));
  // (the measure lands 700 ms after the paint: polled up to 5 s — a fixed 1 s read `reach …` on a loaded machine)
  const readLanded = () => page.evaluate(() => [...document.querySelectorAll("button.patch .delta")].map((x) => x.textContent));
  let landed = await readLanded();
  for (let i = 0; i < 50 && (landed.length === 0 || landed.some((x) => /…/.test(x))); i++) { await sleep(100); landed = await readLanded(); }
  await page.evaluate(() => { const r = window.__riddle; r.engine.deathDeltas = r.__origDD; });
  // (QA 308f045, qaAD: the gem offers no apply until the whole-run measure lands — it reads `…` meanwhile)
  check(pend.reach.every((x) => x === "reach …") && pend.gem === "…", `the death paints at once, each reach pending (${pend.reach.join(" · ")}; gem ${pend.gem})`);
  check(landed.join(" · ") === "reach D6 +8 · reach D6 same · reach D6 −5", `the camp's reach lands on the tablets: the depth, the ±, +0% inside it (QA 92eb880: never \`~0\`) (${landed.join(" · ")})`);

  // ---- 2. the report: newest run first; bests named; a stall patch's numbers add up
  const L = await page.evaluate(() => window.__riddle.lineage);
  const ex = (i) => ({ carried: 50 + i, keep_pct: 60, kept: 30 + i, spent: 0, spent_on: [], text: `returned $${30 + i} · $${50 + i} carried · keeps 60%` });
  const rep = { elapsed_s: 3600, runs: 12, sampled: false, learned: [], bests: ["D7", "D8", "rank 2", "rank 3", "no heal to D5"], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [],
    xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 12, exits: Array.from({ length: 12 }, (_, i) => ex(i)),
    stall: { row: 2, fired: 10, text: "R3 return ended 10 runs, none past D7", patches: [{ row: { conds: [{ k: "hp<", n: 20 }], verb: { v: "rest" } }, insert_at: 2, replace: true, survive: 0.92, forecast_delta: 0.834 }] } };
  await go({ kind: "report", report: rep }); await waitFor((x) => x?.screen === "report", "K's report"); await sleep(300);
  d = await page.evaluate(() => ({ lines: [...document.querySelectorAll(".exit-lines .ledger-line")].map((l) => l.textContent.replace(/\s+/g, " ").trim()),
    bests: [...document.querySelectorAll(".report .rsec")].find((x) => /bests/i.test(x.querySelector(".label")?.textContent ?? ""))?.querySelectorAll("li"),
    patch: document.querySelector(".stall .patch")?.querySelector(".patch-nums")?.innerText.replace(/\s+/g, " ").trim() }));
  const bests = await page.evaluate(() => [...[...document.querySelectorAll(".report .rsec")].find((x) => /bests/i.test(x.querySelector(".label")?.textContent ?? ""))?.querySelectorAll("li") ?? []].map((l) => l.textContent));
  check(/^returned \$41/.test(d.lines[0] ?? "") && /^returned \$34/.test(d.lines[7] ?? "") && /earlier/.test(d.lines[8] ?? ""), `the run rows read newest first, \`· N earlier\` under them (${d.lines[0]?.slice(0, 12)} … ${d.lines[7]?.slice(0, 12)} · ${d.lines[8]})`);
  check(bests.join(" | ") === "new best D8 | ★ rank 3 | no heal to D5", `BESTS names its depth and its rank (${bests.join(" | ")})`);
  check(d.patch === "reach D8 92% · was 9% +83", `a stall patch's numbers add up, \`reach\` once ("${d.patch}")`);

  // ---- 3. strings: the hp lost names hp; the alert HUD
  check(await mod("/src/ui/watch.ts", (m) => m.hurtText(1, "monkey")) === "−1 hp · monkey", "the hero's hurt callout reads `−1 hp · monkey` (not a kill count)");

  // ---- 4. the camp: a `+1 vault` carves the vault tile (its prefs: `keep`, `cage`); trait copy; no `◆0`; busy label over the vista;
  //      the panel closes on a tap beside it; the unlock prices in the mark's blue; the empty slots recede; the verb picker lists the row's verb
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); localStorage.removeItem("riddle.unlocks.all"); });
  await patchSave((e) => { Object.assign(e.lineage, { heir: 2, best_depth: 4, unlocks: ["tame", "vault2"], marks: 3, gold: 0, gold_ledger: [], vault: [], forge: {}, party: [], kennel: [], eggs: [], graveyard: [{ heir: 1, depth: 2, cause: "rat", deeds: [] }], trait_offer: ["brave", "cowardly"], }); });
  await waitFor((x) => x?.screen === "camp", "camp"); await settle();
  const ids = await tileIds();
  check(ids.includes("vault"), `a bought \`+1 vault\` carves the vault tile before any item is kept (${ids.join(" · ")})`);
  if (ids.includes("vault")) {
    await page.locator(".cmd .tile[data-tile=vault]").click({ timeout: 5000 }); await sleep(200);
    const v = await page.evaluate(() => ({ prefs: [...document.querySelectorAll(".panel[data-panel=vault] .prefs > span.dim")].map((x) => x.textContent), slots: document.querySelectorAll(".panel[data-panel=vault] .chip.empty").length, auto: document.querySelector(".panel[data-panel=vault] .keep-auto")?.textContent }));
    // Cut 19 §1: the cage pref left the vault panel for its tablet beside the rules
    check(v.prefs.join(",") === "keep for heirs" && v.slots === 2 && /^keeps \w+/.test(v.auto ?? ""), `the vault panel: 2 empty slots, the \`home\` pref with what it keeps (\`${v.auto}\`), no \`cage\` row (${v.prefs.join(",")})`);
    const r = await page.evaluate(() => { const p = document.querySelector(".panel-host").getBoundingClientRect(), q = document.querySelector(".panel").getBoundingClientRect(); return { x: p.left + p.width / 2, y: p.top + 8, above: q.top > p.top + 16 }; });
    if (r.above) { await page.mouse.click(r.x, r.y); await sleep(200); }
    check(r.above && !(await page.locator(".panel-host .panel").count()), "a tap on the well beside an open panel closes it");
  }
  const trait = await page.evaluate(() => [...document.querySelectorAll(".chip.trait")].map((c) => c.textContent.replace(/\s+/g, " ").trim()));
  check(trait.some((t) => /^(✓ )?brave ?skips (a )?retreat( 1×\/floor)?$/.test(t)), `brave's rule reads \`skips retreat 1×/floor\` (the core's \`skips one retreat a floor\` in three words; ${trait.join(" | ")})`);
  const busy = await page.evaluate(async () => {
    let release; const held = window.__riddle.busy("forecast", () => new Promise((r) => { release = r; })); await new Promise((r) => setTimeout(r, 80));
    const st = document.querySelector(".camp-well .busy-strip"), q = st.getBoundingClientRect(), text = st.textContent, corner = document.querySelector(".busy-label:not([hidden])")?.textContent ?? null;
    st.style.pointerEvents = "auto";   // the strip lets taps through; the probe needs it hit-testable
    const hit = document.elementFromPoint(q.left + q.width / 2, q.top + q.height / 2); st.style.pointerEvents = ""; release(); await held;
    return { text, on: hit === st || st.contains(hit), w: q.width, corner, n: document.querySelectorAll(".busy-strip").length };
  });
  check(busy.text === "forecast" && busy.on && busy.corner === null, `the camp's busy label draws over the vista, not under it ("${busy.text}", on top ${busy.on}, corner "${busy.corner}", ${busy.n} strips)`);
  const empty = await page.evaluate(() => { const e = document.querySelector(".cmd .tile.empty"); return e ? Number(getComputedStyle(e).opacity) : 0; });
  check(empty <= 0.2, `the command card's empty slots recede (opacity ${empty})`);
  await page.evaluate(() => localStorage.setItem("riddle.unlocks.all", "1"));
  await page.locator(".cmd .tile[data-tile=unlocks]").click({ timeout: 5000 });
  await page.waitForFunction(() => document.querySelectorAll(".panel .unlocks .card").length > 0, null, { timeout: 15_000 });
  const u = await page.evaluate(() => ({ zero: [...document.querySelectorAll(".panel .unlocks .card .cost")].some((c) => c.textContent === "◆0"), color: getComputedStyle(document.querySelector(".panel .unlocks .card .cost")).color, bar: getComputedStyle(document.querySelector(".topbar .stat.marks")).color }));
  check(!u.zero, "no unlock card reads `◆0`");
  check(u.color === u.bar, `unlock prices in the bar's ◆ colour (${u.color} vs ${u.bar})`);
  await page.keyboard.press("Escape"); await sleep(100);
  await page.evaluate(() => { const r = window.__riddle; r.editing = true; r.rules.rows[0].verb = { v: "drink", a: "mystery" }; r.go({ kind: "camp" }); });
  await settle();
  await page.locator(".editor .row .chip.verb").first().click({ timeout: 5000 }); await sleep(200);
  const verbs = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chip.verb")].map((c) => ({ t: c.textContent, on: c.classList.contains("on") })));
  check(verbs[0]?.on && /mystery/.test(verbs[0].t), `the verb picker lists the row's own verb, lit, when the vocabulary lacks it (${verbs[0]?.t})`);
  await page.keyboard.press("Escape"); await sleep(100);
  await shot("ui-qaK-camp");

  // ---- 5. the shaft: each notch between 0 and 100% carries its ± (a move inside it is the sims); the forecast: a cause's share in
  //      the ends' unit (≤ the death share)
  const sh = await page.evaluate(() => { const f = window.__riddle.lastForecast; const want = (f?.depths ?? []).filter((d) => d.pm !== undefined && Math.round(d.reach * 100) > 0 && Math.round(d.reach * 100) < 100).map((d) => d.depth);
    const have = [...document.querySelectorAll(".shaft .notch")].filter((n) => n.querySelector(".pm")).map((n) => Number(n.dataset.d)); return { want, have }; });
  check(sh.want.every((d) => sh.have.includes(d)), `the shaft's notches carry their ± (D${sh.want.join(",D") || "–"} → ${sh.have.map((d) => `D${d}`).join(",") || "none"})`);
  await page.locator(".shaft").click({ timeout: 5000 }); await sleep(200);
  await page.waitForFunction(() => !window.__riddle.engineBusy, null, { timeout: 10_000 }).catch(() => {});
  const fc = await page.evaluate(() => { const f = window.__riddle.lastForecast; return { death: f?.ends ? Math.round(f.ends.death * 100) : null, causes: [...document.querySelectorAll(".panel .fc-causes b")].map((b) => parseInt(b.textContent)) }; });
  check(fc.death === null || fc.causes.every((c) => c <= fc.death + 1), `each killer's share is of the sends, never past \`death ${fc.death}%\` (${fc.causes.join(", ") || "none"})`);
  await page.keyboard.press("Escape"); await sleep(100);
  await qaL();
}

/** QA 23ed91f (player L): the second report's client items. */
async function qaL() {
  const mod = (path, fn, arg) => page.evaluate(async ([p, f, a]) => { const m = await import(p); return new Function("m", "a", `return (${f})(m, a)`)(m, a); }, [path, String(fn), arg]);
  // strings and tables: a callout ≤ 3 words; the depth cond offers every floor to 8; a restock of two is ×2 and says restock
  const cap = await mod("/src/ui/watch.ts", (m) => [m.rowCallout(3, "pack break goblin"), m.rowCallout(1, "attack nearest")]);
  const cap2 = await mod("/src/render/state.ts", (m) => m.capWords("pack break goblin", 2));
  check(cap.join(" | ") === "pack break goblin | attack nearest" && cap2 === "pack break", `a row's callout is ≤ 3 words, the target dropped first (${cap.join(" | ")} · caption ${cap2})`);
  const depths = await mod("/src/ui/tokens.ts", (m) => m.NUMS["depth>="]);
  check([4, 6, 7].every((d) => depths.includes(d)), `\`depth ≥\` offers D4, D6, D7 (${depths.join(" ")})`);
  const spent = await mod("/src/ui/watch.ts", (m) => m.spentOf([{ t: 1, delta: 40, why: "returned D5" }, { t: 1, delta: -80, why: "restock heal" }, { t: 1, delta: -30, why: "leash" }], [{ kind: "heal", label: "heal", price: 40 }, { kind: "leash", label: "leash", price: 30 }]));
  check(spent.map((r) => `${r.kind} ×${r.n} $${r.gold}`).join(" · ") === "restock heal ×2 $80 · leash ×1 $30", `SPENT counts a restock of two at the supply's price, and says restock (${spent.map((r) => `${r.kind} ×${r.n} $${r.gold}`).join(" · ")})`);

  // the camp: the gems sum (a stall gem); the killer sits where the deaths are; a stale forecast is not painted; `rest` is no
  // control; the forge marks are no controls; the gear closes an open sheet
  await patchSave((e) => { Object.assign(e.lineage, { heir: 3, best_depth: 8, marks: 30, rest_left_s: 1200, forge: { sword: { salvaged: 1, craftable: false, tier: 0 }, axe: { salvaged: 5, craftable: true, tier: 1 } } }); });
  await waitFor((x) => x?.screen === "camp", "camp"); await settle();
  await page.evaluate(() => { const r = window.__riddle; while (r.ownRows() < 3) r.insertRow({ conds: [{ k: "hp<", n: 40 + r.rules.rows.length }], verb: { v: "retreat" } }, r.rules.rows.length); r.go({ kind: "camp" }); });
  await settle();
  const fake = (f) => page.evaluate((f) => { const r = window.__riddle; const b = r.lastForecast; const x = { ...b, ...f }; r.lastForecast = x; for (const fn of r.fcListeners) fn(x); }, f);
  const depthsF = Array.from({ length: 9 }, (_, i) => ({ depth: i + 1, reach: i === 0 ? 0.5 : i === 1 ? 0.05 : 0, pm: 0.02 }));
  await fake({ depths: depthsF, known_to: 9, causes: [{ cause: "rat", share: 1 }], ends: { bank: 0, return: 0.04, death: 0.66, stall: 0.3, gold: 3, pm: 0.03 }, refined: true });
  await sleep(150);
  const ends = await page.evaluate(() => [...document.querySelectorAll(".shaft .shaft-ends .end")].map((e) => e.textContent.replace(/\s+/g, " ").trim()));
  const sum = ends.map((e) => /(<?)(\d+)%$/.exec(e)).filter(Boolean).reduce((a, m) => a + (m[1] ? 0 : Number(m[2])), 0);   // Cut 23 §2: a `<N%` share is a 0 sampled
  check(ends.some((e) => e === "stall 30%") && sum === 100, `the shaft's gems sum to 100 with a stall gem (${ends.join(" · ")})`);
  await page.locator(".shaft").click({ timeout: 5000 }); await sleep(200);
  // (the panel's bars, polled up to 3 s; a camp measure landing late on a loaded machine repaints the real forecast over the
  // injected one — then it is injected once more and read again)
  const readKiller = () => page.evaluate(() => [...document.querySelectorAll(".panel .fc-bars .bar")].filter((b) => /rat/.test(b.textContent)).map((b) => b.querySelector(".d")?.textContent));
  let killer = await readKiller();
  for (let i = 0; i < 30 && killer.length === 0; i++) { await sleep(100); killer = await readKiller(); }
  if (killer.length === 0) {
    await settle();
    await fake({ depths: depthsF, known_to: 9, causes: [{ cause: "rat", share: 1 }], ends: { bank: 0, return: 0.04, death: 0.66, stall: 0.3, gold: 3, pm: 0.03 }, refined: true });
    for (let i = 0; i < 30 && killer.length === 0; i++) { await sleep(100); killer = await readKiller(); }
  }
  check(killer.length === 1 && killer[0] !== "D9", `the killer sits on the floor where the reach falls, not on D9 (${killer.join(",") || "none"})`);
  await page.keyboard.press("Escape"); await sleep(100);
  const stale = await page.evaluate(async () => {
    const r = window.__riddle, orig = r.engine.forecast.bind(r.engine); let n = 0; const got = [];
    r.engine.forecast = () => new Promise((res) => setTimeout(() => orig().then((f) => res({ ...f, __n: ++n })), 300));
    const off = r.onForecast((f) => got.push(f.__n ?? "refine"));
    const a = r.emitForecast(); await new Promise((x) => setTimeout(x, 60)); r.emitForecast(); await a;
    await new Promise((x) => setTimeout(x, 50)); off(); r.engine.forecast = orig; return got;
  });
  check(stale.join(",") === "2", `a forecast whose rules changed while it ran is not painted; the next one is (painted: ${stale.join(",")})`);
  const rest = await page.evaluate(() => { const e = document.querySelector(".rest-line .rest"); return e && !e.hidden ? { tag: e.tagName, border: getComputedStyle(e).borderTopStyle } : null; });
  check(!!rest && rest.tag !== "BUTTON" && rest.border === "none", `\`rest … · send skips\` is a note, not a boxed control (${JSON.stringify(rest)})`);
  await page.locator(".cmd .tile[data-tile=forge]").click({ timeout: 5000 }); await sleep(200);
  const forge = await page.evaluate(() => ({ text: document.querySelector(".sheet-wrap .forge .salvage")?.textContent ?? "", buttons: [...document.querySelectorAll(".sheet-wrap .forge .salvage button")].filter((b) => !b.classList.contains("close-stud")).length }));
  check(!/○/.test(forge.text) && forge.buttons === 0, `the forge's marks are marks: no \`○\`, no control in the salvage ladder (Cut 23: the kit steps above it are buys) (${forge.buttons} buttons)`);
  await page.keyboard.press("Escape"); await sleep(100);
  await page.locator(".cmd .tile[data-tile=unlocks]").click({ timeout: 5000 }); await sleep(200);
  await page.locator("button.gear").click({ timeout: 5000 }); await sleep(250);
  const sheets = await page.evaluate(() => ({ n: document.querySelectorAll(".sheet-wrap").length, settings: !!document.querySelector(".sheet-wrap .settings"), panels: document.querySelectorAll(".panel-host .panel").length }));
  check(sheets.n === 1 && sheets.settings && sheets.panels === 0, `the gear's settings sheet closes the open UNLOCKS panel: one frame at a time (${sheets.n} sheet, ${sheets.panels} panel)`);
  for (let i = 0; i < 3; i++) { await page.keyboard.press("Escape"); await sleep(80); }

  // the report: LEARNED without bones; PENDING the next three
  const L = await page.evaluate(() => window.__riddle.lineage);
  await page.evaluate((L) => window.__riddle.go({ kind: "report", report: { elapsed_s: 60, runs: 1, sampled: false, learned: ["bones:7", "foe:rat"], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, bones_found: ["bones:7:4"] } }), L);
  await page.waitForFunction(() => document.querySelectorAll(".report .cards .card").length > 0, null, { timeout: 10_000 }).catch(() => {});
  const rp = await page.evaluate(() => { const sec = (w) => [...document.querySelectorAll(".report .rsec")].find((x) => (x.querySelector(".label")?.textContent ?? "").toLowerCase() === w);
    return { learned: sec("learned")?.querySelectorAll(".chip").length ?? 0, learnedText: sec("learned")?.textContent ?? "", pending: sec("pending")?.querySelectorAll(".card").length ?? 0 }; });
  check(rp.learned === 1 && !/bones/.test(rp.learnedText), `LEARNED holds facts, not bones (${rp.learned} chip: ${rp.learnedText.replace(/\s+/g, " ").trim()})`);
  check(rp.pending >= 1 && rp.pending <= 3, `PENDING shows the next three unlocks at most, not the shop (${rp.pending})`);

  // the death: the ledger line wraps inside its outline
  await page.evaluate(() => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 5, cause: "jackal", margin: "1 hp short · 7 unknown unused", verdict: "gap", baseline: 0.2, trace: { turns: [] }, patches: [], morgue: "",
    line: { text: "died $0 · $78 carried · keeps 0% · bones: 10 items on D5 · ◆+7 · +$40 wake · restocked heal ×2 · refund leash", kept: 0, carried: 78, keep_pct: 0, spent: 0, spent_on: [] } } }));
  await sleep(250);
  const lb = await page.evaluate(() => { const b = document.querySelector(".death .ledger-btn"); return b ? { sw: b.scrollWidth, cw: b.clientWidth, right: b.getBoundingClientRect().right } : null; });
  check(!!lb && lb.sw <= lb.cw + 1 && lb.right <= 400, `the death's ledger line wraps inside its outline (${JSON.stringify(lb)})`);
}

/** QA 23ed91f (L): ▶▶| always moves the picture — a world that never ends (a summoner stall) and never opens a fight: each press
 *  returns within its wall budget and lands further on; a stalling run is watched at the flat rate. */
async function stallSkip() {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=fights`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "watch", "the watch for the stall", 30_000);
  await page.evaluate(() => {
    const r = window.__riddle, orig = r.engine.step.bind(r.engine); let S = null, T = 0;
    r.engine.step = async (n) => {
      if (S) { T += n; await new Promise((x) => setTimeout(x, 30)); return { snapshot: { ...S, turn: T, stake: { ...(S.stake ?? {}), stalling: true } }, events: [], run_over: false }; }
      const res = await orig(n);
      if (res.snapshot.turn > 20 && !res.run_over && !res.events.some((e) => e.k === "exit" || e.k === "die")) { S = res.snapshot; T = S.turn; res.snapshot = { ...S, stake: { ...(S.stake ?? {}), stalling: true } }; }
      return res;
    };
  });
  await page.waitForFunction(() => /stalling/.test(document.querySelector("main.watch .stake")?.textContent ?? ""), null, { timeout: 20_000 }).catch(() => {});
  const tick = () => page.evaluate(() => Number(document.querySelector("main.watch")?.dataset.tick ?? 0));
  const res = [];
  for (let k = 0; k < 2; k++) {
    const t0 = await tick(), w0 = Date.now();
    await page.locator(".cmd .tile[data-tile=skip]").click({ timeout: 5000 });
    await page.waitForFunction((t0) => Number(document.querySelector("main.watch")?.dataset.tick ?? 0) > t0 + 300, t0, { timeout: 9000 }).catch(() => {});
    res.push({ dt: (await tick()) - t0, ms: Date.now() - w0 });
  }
  check(res.every((x) => x.dt > 300 && x.ms < 8000), `▶▶| in a never-ending stall moves the picture each press, within its wall budget (${res.map((x) => `+${x.dt} ticks in ${(x.ms / 1000).toFixed(1)} s`).join(" · ")})`);
  await sleep(600);
  const sp = await page.evaluate(() => Number(document.querySelector("main.watch")?.dataset.speed ?? 0));
  check(sp >= 16, `a stalling run plays at the flat rate (${sp}×)`);
  await page.locator(".cmd .tile[data-tile=fast]").click({ timeout: 5000 }); await sleep(600);
  const sp2 = await page.evaluate(() => Number(document.querySelector("main.watch")?.dataset.speed ?? 0));
  check(sp2 >= 16, `\`fast\` runs flat through a stall (${sp2}×)`);
}

/** QA 23ed91f (L: `auto: keep weapon+armour` owned, the vault full → everything salvaged): a watched exit whose keep sheet is skipped
 *  (the vault full) resolves by `autoKeep` (the preference and the automations), never `keep([])`; with a free slot the sheet opens
 *  with the automations' picks (`ExitPending.auto_keep`) ticked. */
async function autoKeepCheck() {
  const rules = encodeURIComponent("depth>=3 → return\nfoes>=1 → attack nearest");   // qa9's run that comes home with items
  const run = async (full) => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=13&autosend=1&speed=fast&rules=${rules}`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch for the keep");
    await page.evaluate((full) => {
      const r = window.__riddle; r.__ak = 0; r.__keep = []; r.__pending = null;
      // the exit reads the engine's vault (QA e75ec29: the camp's copy counted a brought item twice): a full vault is the engine's
      if (full) { r.lineage.vault = [{ id: 7001, kind: "axe", known: true, label: "axe" }]; const lg = r.engine.lineage.bind(r.engine); r.engine.lineage = async () => ({ ...(await lg()), vault: [{ id: 7001, kind: "axe", known: true, label: "axe" }] }); }
      const ak = r.engine.autoKeep?.bind(r.engine), k = r.engine.keep.bind(r.engine), st = r.engine.step.bind(r.engine);
      r.engine.autoKeep = async () => { r.__ak++; return ak ? ak() : k([]); };
      r.engine.keep = async (ids) => { r.__keep.push(ids); return k(ids); };
      r.engine.step = async (n) => { const res = await st(n); if (res.exit_pending?.items?.length) { res.exit_pending.auto_keep = [res.exit_pending.items[0].id]; r.__pending = res.exit_pending.items.length; if (full) res.exit_pending.decide = false; } return res; };
    }, full);
    const t = Date.now(); let s = null;
    while (Date.now() - t < 60_000) {
      s = await page.evaluate(() => ({ screen: window.__riddle.screen, keep: !!document.querySelector(".sheet-wrap .keep-legend") }));
      if (s.keep || !["watch"].includes(s.screen)) break;
      await page.evaluate(() => { for (const b of document.querySelectorAll(".sheet-wrap .vault-choice .chip")) { b.click(); break; } for (const b of document.querySelectorAll(".cmd .tile[data-tile=skip]")) b.click(); });
      await sleep(250);
    }
    return s;
  };
  let s = await run(false);
  const pending = await page.evaluate(() => window.__riddle.__pending);
  if (s?.keep) {
    const on = await page.evaluate(() => ({ on: [...document.querySelectorAll(".sheet-wrap .chips .chip.item")].map((c) => c.classList.contains("on")), count: document.querySelector(".sheet-wrap .label.row-label .num")?.textContent }));
    check(on.on[0] === true && on.count === "1/1", `the keep sheet opens with the automations' pick ticked (${on.count}, ${on.on.map((x) => (x ? "on" : "·")).join(" ")})`);
    await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 }).catch(() => {});
  } else check(false, `a bank with loot opens the keep sheet (screen ${s?.screen}, pending ${pending})`);
  s = await run(true);
  await page.waitForFunction(() => window.__riddle.screen !== "watch", null, { timeout: 30_000 }).catch(() => {});
  const k = await page.evaluate(() => ({ ak: window.__riddle.__ak, keep: window.__riddle.__keep, pending: window.__riddle.__pending, screen: window.__riddle.screen }));
  check(!k.pending || (k.ak === 1 && !k.keep.some((ids) => ids.length === 0)), `the vault full, the skipped sheet keeps by \`autoKeep\`, not \`keep([])\` (autoKeep ×${k.ak}, keep ${JSON.stringify(k.keep)}, ${k.pending ?? 0} pending)`);
}

/** Cut 18 §4–5: the death's gem applies the top patch and lands on the camp with its tablet lit — it never sends (rater Z: "APPLY also
 *  sent the next heir immediately"; two taps on the gem slot, the second on the camp's `send`); the unlock tiles carry both prices
 *  (`◆3 · $450`) and glow when the gold buys them; a card whose reach is noise names its situation (`reach ~0 at R1 · vs archers`). */
/** Cut 19: the cage tablet and its picker's deltas; the loadout's `repeat · $120` toggle; the pinned `+1 row`; `restock capped`;
 *  the `row` verdict (seal ROW, the headline names R2); `+ drop R5` on a full set; the stake's `returning` on a return row. */
async function cut19() {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp for Cut 19"); await settle();
  // ---- §1: no cage seen, no tablet; a `vault` fact carves it (`cage pick → weapon`)
  const cageTab = () => page.evaluate(() => { const t = document.querySelector(".camp .cage-tab"); return t && !t.hidden && t.getClientRects().length ? t.textContent.replace(/\s+/g, " ").trim() : null; });
  await page.evaluate(() => { const r = window.__riddle; r.lineage = { ...r.lineage, facts: r.lineage.facts.filter((f) => f !== "vault") }; r.go({ kind: "camp" }); }); await sleep(250);
  const before = await cageTab();
  await page.evaluate(() => {
    const r = window.__riddle; r.__prefs = [];
    const opts = (cur) => ["weapon", "armour", "potion", "scroll"].map((pref) => { const d = { weapon: 0, armour: 0.36, potion: 0.004, scroll: -0.05 }[pref] - ({ weapon: 0, armour: 0.36, potion: 0.004, scroll: -0.05 }[cur]);
      return { pref, current: pref === cur, depth: 7, reach: 0.5, reach_delta: d, bank: 0.54 + d, bank_delta: d, gold: 100, gold_delta: 0, delta: d, pm: 0.03 }; });
    r.engine.cageForecast = () => new Promise((res) => setTimeout(() => res(opts(r.lineage.vault_pref ?? "weapon")), 400));
    const setPref = r.engine.setVaultPref.bind(r.engine); r.engine.setVaultPref = async (p) => { r.__prefs.push(p); return setPref(p); };
    r.lineage = { ...r.lineage, facts: [...r.lineage.facts, "vault"], vault_pref: "weapon" }; r.go({ kind: "camp" });
  });
  await sleep(250);
  const after = await cageTab();
  check(before === null && after === "from cages → weapon", `the cage tablet is carved by a seen cage, beside the rules (${before} → ${after})`);
  const inCol = await page.evaluate(() => !!document.querySelector(".camp-main .tablets .editor + .cage-tab"));
  check(inCol, "the cage tablet sits under the rule tablets");
  await page.locator(".camp .cage-tab").click({ timeout: 5000 }); await sleep(120);
  const opt = () => page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .cage-picker .cage-opt")].map((b) => `${b.textContent.replace(/\s+/g, " ").trim()}${b.classList.contains("on") ? "*" : ""}`));
  const pending = await opt();
  await sleep(600);
  const landed = await opt();
  check(pending.includes("armour …") && landed.join(" · ") === "weapon bank 54%* · armour bank 90% ▲36 · potion bank 54% · scroll bank 49% ▼5", `the picker shows each preference's level on one scale, its move a mark (QA 0c6e126, qaZ) (${pending.join(" · ")} → ${landed.join(" · ")})`);
  await shot("ui-cut19-cage-picker");
  await page.locator(".sheet-wrap .cage-opt[data-pref=armour]").click({ timeout: 5000 }); await sleep(400);
  const picked = await cageTab(), prefs = await page.evaluate(() => window.__riddle.__prefs);
  check(picked === "from cages → armour" && prefs.join() === "armour" && !(await page.locator(".sheet-wrap .cage-picker").count()), `a tap on an option sets the cage (${prefs.join()}; tablet \`${picked}\`)`);
  await shot("ui-cut19-cage");
  // ---- §3: the loadout tile carries `repeat · $120`; a tap on it clears the repeat (the shelf stays shut); a second turns it back on
  await page.evaluate(() => {
    const r = window.__riddle; r.__restock = [];
    r.engine.setRestock = async (on) => { r.__restock.push(on); return { ...r.lineage, repeat: on }; };
    // (an empty shelf: the switch flips in one tap — a shelf of bought supplies asks first, qaAC.mjs)
    r.lineage = { ...r.lineage, gold: 300, repeat: true, repeat_kinds: ["heal"], repeat_gold: 120, supplies: [] }; r.go({ kind: "camp" });
  });
  await sleep(300);
  const badge = () => page.evaluate(() => document.querySelector(".cmd .tile[data-tile=loadout] .repeat-badge")?.textContent ?? null);
  const b0 = await badge();
  await shot("ui-cut19-repeat");
  await page.locator(".cmd .tile[data-tile=loadout] .repeat-badge").click({ timeout: 5000 }).catch(() => {}); await sleep(300);
  const b1 = await badge(), panel = await page.locator(".panel[data-panel=loadout]").count();
  await page.locator(".cmd .tile[data-tile=loadout] .repeat-badge").click({ timeout: 5000 }).catch(() => {}); await sleep(300);
  const b2 = await badge(), calls = await page.evaluate(() => window.__riddle.__restock);
  check(b0 === "repeat on · held ≤$120" && b1 === "repeat off" && b2 === "repeat on · held ≤$120" && calls.join() === "false,true" && panel === 0, `the loadout tile: \`${b0}\` → \`${b1}\` → \`${b2}\` (setRestock ${calls.join()}; shelf ${panel ? "opened" : "shut"})`);
  // ---- §3: the short list always carries the pinned `+1 row`, ranked last or not
  await page.evaluate(() => {
    const r = window.__riddle;
    const cat = [
      { id: "vault2", cost: 1, owned: false, available: true, gold: 150 }, { id: "cond_alert", cost: 1, owned: false, available: true, gold: 150 },
      { id: "kennel", cost: 1, owned: false, available: true, gold: 150 }, { id: "insure", cost: 2, owned: false, available: true, gold: 300 },
      { id: "row5", cost: 9, owned: false, available: false, needs: "◆9 more", pinned: true },
    ];
    r.engine.unlocks = async () => cat; r.engine.unlockDeltas = async () => cat;
    r.lineage = { ...r.lineage, marks: 3, gold: 0, unlocks: [...r.lineage.unlocks.filter((u) => u !== "row5")] };
    localStorage.setItem("riddle.unlocks.all", "0"); r.go({ kind: "camp" });
  });
  await sleep(400); await openPanel(page, "unlocks"); await sleep(300);
  const cards = await page.evaluate(() => [...document.querySelectorAll(".panel .unlocks .cards .card .card-main > span:first-child")].map((c) => c.textContent));
  check(cards.length === 3 && cards.includes("+1 rule slot"), `the short list keeps the pinned \`+1 row\` (${cards.join(" · ")})`);
  await page.keyboard.press("Escape"); await sleep(100);
  // ---- §3: the report says `restock capped`
  const L = await page.evaluate(() => window.__riddle.lineage);
  const rep = { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [],
    xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 3, spent: [{ kind: "restock heal", n: 2, gold: 80 }],
    gold: { home: 90, salvage: 0, wake: 0, spent: 80 }, restock_capped: true };
  await page.evaluate((v) => window.__riddle.go(v), { kind: "report", report: rep }); await waitFor((x) => x?.screen === "report", "the capped report"); await sleep(300);
  const gl = await page.evaluate(() => document.querySelector(".report .gold-line")?.textContent.replace(/\s+/g, " ").trim() ?? "");
  check(/restock ≤ \$90 earned$/.test(gl), `the report says the repeat was capped (restock ≤ $90 earned) ("${gl}")`);
  // ---- §4: a `row` verdict — the seal reads ROW, the headline names the row; an insert on a full set reads `+ drop R5`
  const rows = [
    { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "unknown" }, origin: "player" }, { conds: [{ k: "adj>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" },
    { conds: [], verb: { v: "explore" }, origin: "player" }, { conds: [{ k: "hp<", n: 50 }], verb: { v: "rest" }, origin: "player" },
    { conds: [{ k: "depth>=", n: 8 }], verb: { v: "bank" }, origin: "player" },
  ];
  await page.evaluate((rows) => { const r = window.__riddle; r.sets[r.active] = { ...r.sets[r.active], rows }; r.vocab = { ...r.vocab, max_rows: 5 }; }, rows);
  await page.evaluate((rows) => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 6, cause: "goblin_archer", margin: "", verdict: "row", cause_row: 0, baseline: 0.2, trace: { turns: [] }, morgue: "t1", rules: { rows },
    patches: [{ row: rows[0].row ?? rows[0], insert_at: 0, remove: true, survive: 0.8, forecast_delta: 0.1 },
              { row: { conds: [{ k: "foe_tag", t: "ranged" }], verb: { v: "retreat" } }, insert_at: 1, survive: 0.7, forecast_delta: 0.05, drops: 4 }] } }), rows);
  await waitFor((x) => x?.screen === "death", "the row death"); await sleep(300);
  const dd = await page.evaluate(() => ({ cause: document.querySelector(".death-line .cause")?.textContent, seal: document.querySelector(".death-line .verdict") ? getComputedStyle(document.querySelector(".death-line .verdict")).textTransform + ":" + document.querySelector(".death-line .verdict").textContent : "",
    targets: [...document.querySelectorAll("button.patch .target")].map((t) => t.textContent.trim()) }));
  check(/^goblin archer · D6 · drink unknown at 30%$/.test(dd.cause ?? "") && dd.seal === "uppercase:rule", `the row verdict: the headline names the row, the seal reads ROW ("${dd.cause}", ${dd.seal})`);
  check(dd.targets.join(" | ") === "cut | · drops bank at D8", `the cut leads, the insert names the row it drops (${dd.targets.join(" | ")})`);
  await shot("ui-cut19-row");
  // QA 1a2a4a9: a tablet tap lights it; the gem applies the lit one
  await page.locator("button.patch[data-full]").first().click({ timeout: 5000 }); await sleep(150);
  const lit = await page.evaluate(() => ({ top: document.querySelector("button.patch.top")?.dataset.full === "1", sheet: !!document.querySelector(".sheet-wrap .drop-sheet") }));
  check(lit.top && !lit.sheet, `a tablet tap lights the tablet and applies nothing (${JSON.stringify(lit)})`);
  await page.locator(".patch-gem").click({ timeout: 5000 }); await sleep(200);
  // QA 778fa1b (qaU friction: `apply` opened the drop sheet though the patch said `drops R4`): the named drop applies as stated
  const applied = await page.evaluate(() => ({ screen: window.__riddle.screen, sheet: !!document.querySelector(".sheet-wrap .drop-sheet"), rows: window.__riddle.rules.rows.map((r) => r.verb.v).join(",") }));
  check(applied.screen === "camp" && !applied.sheet && applied.rows === "drink,retreat,attack,explore,rest", `the gem applies the named drop, no sheet (${JSON.stringify(applied)})`);
  // ---- §2: a return row firing turns the stake's `return at 20%` into `returning`
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp for the return walk"); await settle();
  await page.evaluate(() => {
    const r = window.__riddle, orig = r.engine.step.bind(r.engine); let n = 0;
    r.sets[r.active] = { ...r.sets[r.active], rows: [{ conds: [{ k: "hp<", n: 20 }], verb: { v: "return" }, origin: "player" }, ...r.rules.rows] };
    r.engine.step = async (k) => { const res = await orig(k); n++; if (res.snapshot.stake) res.snapshot.stake.return_row = 0;
      if (n > 6 && !res.run_over && !res.events.some((e) => e.k === "exit")) res.events.push({ k: "rule", t: res.snapshot.turn, row: 0, verb: { v: "return" }, text: "return" }); return res; };
    r.watchMode = "fast"; r.go({ kind: "watch" });
  });
  let st = ""; const t1 = Date.now(), seen = [];
  while (Date.now() - t1 < 15_000) { st = await page.evaluate(() => document.querySelector(".watch .stake")?.textContent ?? ""); if (st && !seen.includes(st)) seen.push(st); if (/returning/.test(st) || !["watch"].includes((await state())?.screen)) break; await sleep(60); }
  check(seen.some((x) => /return at 20%/.test(x)) && /returning/.test(st), `the stake reads \`return at 20%\`, then \`returning\` once the row fires (${seen.slice(0, 2).join(" | ")} → ${st})`);
}

async function cut18() {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp for Cut 18"); await settle();
  // ---- APPLY: two quick taps on the gem slot
  const rows0 = await page.evaluate(() => window.__riddle.rules.rows.length);
  await page.evaluate(() => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 3, cause: "goblin", margin: "", verdict: "gap", baseline: 0.3, trace: { turns: [] }, morgue: "t1",
    patches: [{ row: { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } }, insert_at: 0, survive: 0.9, forecast_delta: 0.1 }] } }));
  await waitFor((x) => x?.screen === "death", "the death for APPLY"); await sleep(300);
  const box = await page.locator("main.death .gem-slot > .gem").boundingBox();
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);
  await sleep(120);
  await page.mouse.click(box.x + box.width / 2, box.y + box.height / 2);   // the second tap lands on the camp's gem, `send`
  await sleep(1500);
  const ap = await page.evaluate(() => ({ screen: window.__riddle.screen, rows: window.__riddle.rules.rows.length, lit: document.querySelectorAll(".editor .row.hl, .editor .hl").length, first: window.__riddle.rules.rows[0]?.verb.v }));
  check(ap.screen === "camp" && ap.rows === rows0 + 1 && ap.first === "return", `the gem applies the top patch and lands on the camp — it does not send (${ap.screen}, rows ${rows0} → ${ap.rows}, R1 ${ap.first})`);
  check(ap.lit >= 1, `the applied tablet is lit on the camp (${ap.lit})`);
  const sent = await page.evaluate(() => new Promise((res) => setTimeout(() => res(window.__riddle.screen), 200)));
  await page.locator("main.camp .gem-slot > .gem").click({ timeout: 5000 }); await sleep(600);
  const s2 = await page.evaluate(() => window.__riddle.screen);
  check(sent === "camp" && s2 === "watch", `the player sends: a later tap on \`send\` starts the run (${s2})`);
  // ---- the unlock tiles: both prices, the gold glow, a noise card's situation
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "the camp for the tiles"); await settle();
  await page.evaluate(() => {
    const r = window.__riddle;
    const cat = [
      { id: "kite_archers", cost: 3, owned: false, available: false, needs: "◆3 more", gold: 450, delta: 0.01, pm: 0.03, insert_at: 0, situation: "ranged", rows: [{ conds: [{ k: "foe_tag", t: "ranged" }], verb: { v: "retreat" } }] },
      { id: "vault2", cost: 3, owned: false, available: false, needs: "◆3 more", gold: 300 },
      { id: "cond_alert", cost: 2, owned: false, available: false, needs: "fact: alert", gold: 300 },
    ];
    r.engine.unlocks = async () => cat; r.engine.unlockDeltas = async () => cat;
    r.lineage = { ...r.lineage, gold: 400, marks: 0, unlocks: [...r.lineage.unlocks, "row5"] };
    localStorage.setItem("riddle.unlocks.all", "1");
    r.go({ kind: "camp" });
  });
  await sleep(400);
  await openPanel(page, "unlocks", { all: true });
  await sleep(300);
  const tiles = await page.evaluate(() => [...document.querySelectorAll(".unlocks .cards .card")].map((c) => ({ label: c.querySelector(".card-main > span")?.textContent, cost: c.querySelector(".cost")?.textContent, cls: c.className, delta: c.querySelector(".delta")?.textContent ?? "" })));
  const t = (label) => tiles.find((x) => x.label === label);
  check(t("rule: kite archers")?.cost === "◆3 or $450" && t("+1 vault")?.cost === "◆3 or $300", `an unlock tile shows both prices (${tiles.map((x) => `${x.label} ${x.cost}`).join(" · ")})`);
  check(/\bbuyable\b/.test(t("+1 vault")?.cls ?? "") && !/\bbuyable\b/.test(t("rule: kite archers")?.cls ?? "") && !/\bbuyable\b/.test(t("condition: alert")?.cls ?? ""), `a tile the gold buys glows like one the marks buy; one short of gold or gated does not (${tiles.map((x) => `${x.label}: ${x.cls}`).join(" · ")})`);
  check(t("rule: kite archers")?.delta === "vs archers", `a card's \`~0\` names its situation ("${t("rule: kite archers")?.delta}")`);
  await shot("ui-cut18-unlocks");
  // ---- §3: a wall says it is a wall — `ForecastDepth.wall` on D9 (best D8): the notch `D9 · warlord`, the panel's row `D9 0% · sealed by warlord`
  await page.evaluate(() => { const r = window.__riddle; r.engine.unlocks = async () => []; r.engine.unlockDeltas = async () => []; r.lineage = { ...r.lineage, best_depth: 8 }; r.go({ kind: "camp" }); });
  await settle();
  await page.evaluate(() => { const r = window.__riddle; const depths = Array.from({ length: 9 }, (_, i) => ({ depth: i + 1, reach: i < 8 ? 0.9 - i * 0.05 : 0, pm: 0.02, ...(i === 8 ? { wall: "goblin_warlord", cause: "goblin_warlord" } : {}) }));
    const x = { ...r.lastForecast, depths, known_to: 9, causes: [{ cause: "goblin_warlord", share: 1 }], refined: true }; r.lastForecast = x; for (const fn of r.fcListeners) fn(x); });
  await sleep(200);
  const notch = await page.evaluate(() => { const n = document.querySelector('.shaft .notch[data-d="9"] .dl'); const r = n?.getBoundingClientRect(), sh = document.querySelector(".shaft")?.getBoundingClientRect(); return { text: n?.textContent, inside: !!r && !!sh && r.right <= sh.right + 0.5 }; });
  check(notch.text === "D9 · behind warlord" && notch.inside, `the shaft's notch names the wall ("${notch.text}", inside the shaft ${notch.inside})`);
  await shot("ui-cut18-shaft");
  await openPanel(page, "forecast"); await sleep(200);
  const row = await page.evaluate(() => [...document.querySelectorAll(".panel .fc-bars .bar")].map((b) => b.textContent.replace(/\s+/g, " ").trim()).find((t) => /^D9/.test(t)));
  check(/^D9 ?(0%|<\d+%)( ±\d+)? · behind warlord( · counter: [a-z ]+| · warlord: \?)?$/.test(row ?? ""), `the panel's row reads \`D9 0% · sealed by warlord\` and its counter (Cut 28 §1) ("${row}")`);
  await shot("ui-cut18-wall");
  await page.keyboard.press("Escape"); await sleep(100);
  // ---- §4: a stall's cause names the rows' loop (`R2 retreat ↔ explore`): the loop whole on one line; QA 1a2a4a9 (P: the headline
  // `STALLED · R2 RETREAT ↔ EXPLORE · D6 · KEEPS $0` ran off both edges at 400 px): it wraps between segments, inside the screen
  await page.evaluate(() => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 6, cause: "stalled · R2 retreat ↔ explore", margin: "keeps $0", verdict: "stall", baseline: 0.08, trace: { turns: [] }, morgue: "t1",
    patches: [{ row: { conds: [{ k: "foe_tag", t: "gas" }], verb: { v: "retreat" } }, insert_at: 1, replace: true, survive: 1, forecast_delta: 0 }] } }));
  await waitFor((x) => x?.screen === "death", "the stall's verdict"); await sleep(400);
  const head = await page.evaluate(() => {
    const c = document.querySelector(".death-line .cause"), loop = [...c.querySelectorAll(".seg")].find((x) => /↔/.test(x.textContent));
    const lh = parseFloat(getComputedStyle(c).fontSize) * 1.6, segs = [...c.querySelectorAll(".seg")].map((x) => x.getBoundingClientRect());
    return { text: c?.textContent, loopOne: !!loop && loop.getBoundingClientRect().height < lh, inside: segs.every((r) => r.left >= 0 && r.right <= innerWidth),
      face: !document.querySelector(".death .portrait.dead"), surv: document.querySelector("button.patch .surv")?.textContent };
  });
  check(head.text === "stalled · retreat ↔ explore · D6 · keeps $0" && head.loopOne && head.inside, `a stall's headline: the loop whole on one line, every segment on screen ("${head.text}", loop one line ${head.loopOne}, inside ${head.inside})`);
  check(head.face && head.surv === "unstuck 100% · was 8%", `a stall is no death: the face lit, the patch reads what it ends ("${head.surv}", lit ${head.face})`);
  await shot("ui-stall-head");
  // QA 1a2a4a9: the core's new reasons arrive verbatim and read whole, one line each
  await page.evaluate(() => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 4, cause: "jackal", margin: "", verdict: "gap", baseline: 0.3, morgue: "t1", patches: [],
    trace: { turns: [{ t: 10, row: -1, verb: { v: "explore" }, hp: 9, foes: 2, telegraphs: [], rows: [{ row: 0, why: "foes appeared after" }, { row: 1, why: "foes fleeing" }, { row: 2, why: "going home" }, { row: 3, why: "repeat short" }] }] } } }));
  await waitFor((x) => x?.screen === "death", "the reasons' death"); await sleep(300);
  const whys = await page.evaluate(() => [...document.querySelectorAll(".rows-line .rw, .chain-row .why")].map((e) => ({ t: e.textContent, one: e.getClientRects().length === 1 })));
  check(["foes appeared after", "foes fleeing", "going home", "repeat short"].every((w) => whys.some((x) => x.t.endsWith(w) && x.one)), `the new reasons render whole, one line each (${whys.map((x) => x.t).join(" · ")})`);
  await shot("ui-cut18-stall");
}

const t0 = Date.now();
try {
  // ---- a fresh lineage: heir 1, nothing earned (the fake seeds a chronicle and a free unlock; strip them)
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); localStorage.removeItem("riddle.unlocks.all"); });
  check(await patchSave((e) => {
    const L = e.lineage;
    // as the core's new_lineage: `tame` owned and a free leash on the shelf (neither earned)
    Object.assign(L, { heir: 1, chronicle: [], unlocks: ["tame"], marks: 0, gold: 0, gold_ledger: [], graveyard: [], vault: [], forge: {}, party: [], kennel: [], eggs: [], rank: 0, renown: 0, best_depth: 0, trophies: [],
      supplies: [{ id: 100000, kind: "leash", known: true, label: "leash", free: true }] });
  }), "a fresh lineage (heir 1, nothing earned)");
  await waitFor((s) => s?.screen === "camp", "the fresh camp"); await settle();
  await shot("ui-fresh-camp");
  {
    const els = await interactive();
    check(els.length <= 8, `a fresh camp shows ≤ 8 interactive elements at 400 × 800 (${els.length}: ${els.join(" | ")})`);
    const f = await page.evaluate(() => ({
      stats: [...document.querySelectorAll(".topbar .stats > *")].map((e) => e.className),
      tablets: document.querySelectorAll(".editor .row.tablet.compact").length,
      notches: [...document.querySelectorAll(".shaft .notch")].map((n) => n.querySelector(".dl")?.textContent),
      ends: !document.querySelector(".shaft .shaft-ends")?.hidden,
      gem: document.querySelector(".console .gem")?.textContent.trim(),
      tiles: document.querySelectorAll(".console .cmd .tile:not(.empty)").length,
      tabs: !!document.querySelector(".tabs:not([hidden]) .tab"),
      cls: !!document.querySelector("button.cls"),
    }));
    check(f.stats.length === 1 && /\bgold\b/.test(f.stats[0]), `the bar shows $ only (${f.stats.join(", ")})`);
    check(f.tablets === 2 && f.notches.join(",") === "D1" && !f.ends, `two compact tablets, the shaft at D1 alone, no gems (${f.tablets} tablets, notches ${f.notches.join(",")}, ends ${f.ends})`);
    check(/^send(▸ fights only)?$/i.test(f.gem ?? "") && f.tiles === 0 && !f.tabs && !f.cls, `the gem SEND and nothing else: no tile, no set tab, no class picker (gem "${f.gem}", ${f.tiles} tiles)`);
    // a tablet is one tap: it opens the tablets for editing at that row
    await page.locator(".editor .row.tablet.compact").first().click({ timeout: 5000 }); await sleep(200);
    check(await page.locator(".editor .row .chip.cond").count() > 0, "a tap on a compact tablet opens it for editing (its chips)");
  }

  // ---- the ladder: each tile on its trigger, not before; the first appearance glints
  const steps = [
    ["edit", "first death", (e) => { e.lineage.graveyard.push({ heir: 1, depth: 1, cause: "rat", deeds: [] }); }],
    ["loadout", "first gold home", (e) => { e.lineage.gold = 40; e.lineage.gold_ledger = [{ t: 10, delta: 40, why: "returned D1" }]; }],
    ["unlocks", "first mark", (e) => { e.lineage.marks = 1; }],
    ["vault", "an item worth keeping", (e) => { e.lineage.vault = [{ id: 901, kind: "sword", label: "sword +1" }]; }],
    ["forge", "first salvage", (e) => { e.lineage.forge = { sword: { salvaged: 1, craftable: false, tier: 0 } }; }],
    ["party", "a companion", (e) => { e.lineage.eggs = [{ id: 902, kind: "jackal", tags: [], gen: 1, hatch_in: 3, from_loss: false }]; }],
  ];
  const seen = [];
  for (const [id, why, fn] of steps) {
    const before = await tileIds();
    check(!before.includes(id), `before ${why}: no \`${id}\` tile (${before.join(" · ") || "none"})`);
    check(await patchSave(fn), `the lineage took ${why}`);
    await waitFor((s) => s?.screen === "camp", "camp"); await settle();
    const after = await tiles();
    const t = after.find((x) => x.id === id);
    check(!!t && t.reveal && seen.every((s) => after.some((x) => x.id === s)), `${why} carves the \`${id}\` tile, glinting (${after.map((x) => `${x.id}${x.reveal ? "*" : ""}`).join(" · ")})`);
    seen.push(id);
    if (id === "edit") check(await page.locator("button.cls").count() === 1, "the first death puts the class picker on the portrait");
    if (id === "unlocks") {
      const u = await page.evaluate(() => document.querySelector(".strip .marks")?.textContent);
      check(u === "◆1", `the first mark puts ◆ on the bar (${u})`);
    }
  }
  // a later repaint is no reveal: the tiles stand, none glints (a glint lasts its 1.5 s)
  await sleep(1700); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await settle();
  check((await tiles()).every((t) => !t.reveal) && (await tileIds()).length === seen.length, `a repaint keeps the carved tiles without a glint (${(await tileIds()).join(" · ")})`);
  // a 3rd row: the shaft's gems
  check(await page.evaluate(() => document.querySelector(".shaft .shaft-ends")?.hidden !== false), "before a 3rd row: the shaft has no gems");
  await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [{ k: "hp<", n: 50 }], verb: { v: "attack", a: "nearest" } }, 2); r.go({ kind: "camp" }); });
  await page.waitForFunction(() => document.querySelector(".shaft .shaft-ends")?.hidden === false, null, { timeout: 15_000 }).catch(() => {});
  const ends = await page.evaluate(() => [...document.querySelectorAll(".shaft .shaft-ends .end")].map((e) => e.textContent.replace(/\s+/g, " ").trim()));
  check(ends.length === 4 && /^bank <?\d+%$/.test(ends[0]) && /^return <?\d+%$/.test(ends[1]) && /^death [<>]?\d+%$/.test(ends[2]) && /^avg \$\d+\/run…?$/.test(ends[3]), `a 3rd row lights the shaft's gems (QA 778fa1b: \`…\` on the first pass): ${ends.join(" · ")}`);
  // 5 heirs: ledger, chronicle, the set tabs
  check(!(await tileIds()).includes("ledger") && !(await page.locator(".tabs:not([hidden]) .tab").count()), "before the 5th heir: no ledger, no chronicle, no set tabs");
  check(await patchSave((e) => { e.lineage.heir = 5; }), "the lineage took its 5th heir");
  await waitFor((s) => s?.screen === "camp", "camp"); await settle();
  const ids = await tileIds();
  check(ids.includes("ledger") && ids.includes("chronicle") && (await page.locator(".tabs:not([hidden]) .tab").count()) >= 3, `the 5th heir carves ledger and chronicle and the set tabs (${ids.join(" · ")})`);
  check(ids.length === 8, `the command card is full at the ladder's top (${ids.length} tiles)`);
  await shot("ui-ladder-camp");

  // ---- the unlock panel: the next three, `more` for the catalogue
  await patchSave((e) => { e.lineage.marks = 30; });
  await waitFor((s) => s?.screen === "camp", "camp"); await settle();
  await page.locator(".cmd .tile[data-tile=unlocks]").click({ timeout: 5000 });
  await page.waitForFunction(() => document.querySelectorAll(".panel .unlocks .card").length > 0, null, { timeout: 15_000 });
  const cards0 = await page.locator(".panel .unlocks .card").count();
  const more = await page.locator(".panel .unlocks button.more").count();
  check(cards0 === 3 && more === 1, `the unlock panel lists the next three, \`more\` under them (${cards0} cards, more ${more})`);
  await page.locator(".panel .unlocks button.more").click({ timeout: 5000 }); await sleep(200);
  const cards1 = await page.locator(".panel .unlocks .card").count();
  check(cards1 > 3 && !(await page.locator(".panel .unlocks button.more").count()), `\`more\` opens the catalogue (${cards1} cards)`);
  await shot("ui-unlocks-panel");

  // ---- every panel and sheet carries a close stud that closes it
  for (const p of ["unlocks", "loadout", "vault", "party"]) {
    const open = await page.locator(".panel-host .panel").count();
    if (!open || (await page.locator(`.panel[data-panel=${p}]`).count()) === 0) await page.locator(`.cmd .tile[data-tile=${p}]`).click({ timeout: 5000 });
    await sleep(150);
    const has = await page.locator(`.panel[data-panel=${p}] .close-stud`).count();
    await page.locator(`.panel[data-panel=${p}] .close-stud`).click({ timeout: 5000 }).catch(() => {}); await sleep(150);
    check(has === 1 && !(await page.locator(".panel-host .panel").count()), `the ${p} panel has a close stud, and it closes the panel`);
  }
  await page.locator(".shaft").click({ timeout: 5000 }); await sleep(150);
  check(await page.locator(".panel[data-panel=forecast] .close-stud").count() === 1, "the shaft opens the forecast panel, with its stud");
  await page.keyboard.press("Escape"); await sleep(150);
  check(!(await page.locator(".panel-host .panel").count()), "Escape closes the open panel");
  const sheets = [
    ["settings", "button.gear"], ["gold", ".strip button.gold"], ["forge", ".cmd .tile[data-tile=forge]"], ["ledger", ".cmd .tile[data-tile=ledger]"],
    ["chronicle", ".cmd .tile[data-tile=chronicle]"], ["class", "button.cls"], ["cond picker", ".editor .row .chip.cond"], ["verb picker", ".editor .row .chip.verb"], ["rename", ".tabs .tab.edit"],
  ];
  for (const [name, sel] of sheets) {
    await page.locator(sel).first().click({ timeout: 5000 }); await sleep(200);
    const s = await page.evaluate(() => { const w = [...document.querySelectorAll(".sheet-wrap")].pop(); return w ? { stud: w.querySelectorAll(".sheet > .close-stud, .sheet .sheet-x").length } : null; });
    if (s?.stud) { await page.locator(".sheet-wrap .sheet > .close-stud, .sheet-wrap .sheet .sheet-x").last().click({ timeout: 5000 }); await sleep(150); }
    const left = await page.locator(".sheet-wrap").count();
    check(!!s && s.stud === 1 && left === 0, `the ${name} sheet has one close stud, and it closes the sheet`);
    if (left) for (let i = 0; i < 4; i++) await page.keyboard.press("Escape");
  }
  await page.locator(".cmd .tile[data-tile=unlocks]").click({ timeout: 5000 }); await sleep(200);
  await page.locator(".panel .unlocks .card").first().click({ timeout: 5000 }); await sleep(200);
  check(await page.locator(".sheet-wrap .sheet > .close-stud").count() === 1, "an unlock card's sheet has its close stud");
  await page.keyboard.press("Escape"); await page.keyboard.press("Escape"); await sleep(150);

  // ---- every tile has a pressed state: held, it sinks (and darkens)
  const pressed = [];
  for (const id of await tileIds()) {
    const loc = page.locator(`.cmd .tile[data-tile=${id}]`);
    const box = await loc.boundingBox(); if (!box) { pressed.push(`${id}: no box`); continue; }
    const rest = await loc.evaluate((b) => getComputedStyle(b).transform), on = await loc.evaluate((b) => b.classList.contains("on"));   // an `on` tile rests pressed
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down(); await sleep(120);
    const down = await loc.evaluate((b) => ({ t: getComputedStyle(b).transform, img: getComputedStyle(b).borderImageSource }));
    await page.mouse.move(200, 400); await page.mouse.up(); await sleep(60);
    if (!/matrix\(1, 0, 0, 1, 0, 2\)/.test(down.t) || (!on && down.t === rest)) pressed.push(`${id}: ${rest} → ${down.t}`);
    else if (/tile\.png/.test(down.img)) pressed.push(`${id}: the raised frame while held`);
  }
  for (let i = 0; i < 3; i++) { await page.keyboard.press("Escape"); await sleep(60); }
  check(pressed.length === 0, `every tile sinks while held (${pressed.join("; ") || (await tileIds()).length + " tiles"})`);

  // ---- every place: the bar, the console, the primary in the gem
  const frame = () => page.evaluate(() => {
    const m = document.querySelector("#app > main");
    const g = m?.querySelector(":scope > footer.console .gem-slot > .gem");
    return { main: m?.className ?? "", bar: !!m?.querySelector(":scope > header.topbar"), console: !!m?.querySelector(":scope > footer.console"), gem: g ? { text: g.textContent.trim(), cls: g.className, visible: !!g.getClientRects().length && getComputedStyle(g).visibility !== "hidden" } : null };
  });
  let f = await frame();
  check(f.bar && f.console && f.gem?.visible && /\bsend\b/.test(f.gem.cls) && /^send▸ (fights only|fast|normal)$/.test(f.gem.text), `camp: bar + console, \`send\` in the gem (${JSON.stringify(f.gem)})`);
  await page.locator("button.send").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "watch", "the watch"); await sleep(1500);
  // QA 23ed91f: the last frame — record the bar's heir and the gem the moment the run is over (the exit's refresh follows)
  await page.evaluate(() => {
    const w = document.querySelector("main.watch"); const heir0 = w.querySelector(".topbar .heir")?.textContent;
    window.__lastFrame = { heir0, at: [] };
    const look = () => { const m = document.querySelector("main.watch"); if (/!/.test(m?.querySelector(".hud .alert")?.textContent ?? "")) window.__lastFrame.bang = true; if (!m || m.dataset.over !== "1") return; window.__lastFrame.at.push({ heir: m.querySelector(".topbar .heir")?.textContent, gem: m.querySelector(".gem-slot > .gem")?.textContent ?? null, dis: !!m.querySelector(".gem-slot > .gem")?.disabled, corner: !!document.querySelector(".busy-label:not([hidden])") }); };
    new MutationObserver(look).observe(document.body, { subtree: true, childList: true, attributes: true, characterData: true });
  });
  f = await frame();
  check(f.bar && f.console && f.gem?.visible && ["⏸", "▶"].includes(f.gem.text), `watch: bar + console, ⏸ in the gem (${JSON.stringify(f.gem)})`);
  const cmd = await page.evaluate(() => [...document.querySelectorAll(".console .cmd .tile:not(.empty)")].map((b) => b.textContent.trim()));
  check(cmd.join(" · ") === "fights only · fast · normal · ▶▶| · bail · meters", `watch: the command card is fights only · fast · normal · ▶▶| · bail · meters (Cut 25 §3, Cut 29 §3) (${cmd.join(" · ")})`);
  await shot("ui-watch");
  let s = await state(); const tw = Date.now();
  while (s?.screen === "watch" && Date.now() - tw < 90_000) { await page.evaluate(() => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === "▶▶|") b.click(); }); await sleep(250); s = await state(); }
  if (s?.screen === "exit") {
    // QA 23ed91f (K: "`$5` on each item: a cost to keep, or a sale price?"): the keep sheet says what the prices are
    const legend = await page.evaluate(() => document.querySelector(".sheet-wrap .keep-legend")?.textContent ?? null);
    check(legend === "unkept → salvage", `the keep sheet says its prices are the salvage of what is not kept ("${legend}")`);
    await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 });
  }
  s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after the run", 30_000); await settle();
  {
    const lf = await page.evaluate(() => window.__lastFrame);
    const heirs = [...new Set(lf.at.map((a) => a.heir))], gems = [...new Set(lf.at.map((a) => a.gem))];
    check(lf.at.length > 0 && heirs.length === 1 && heirs[0] === lf.heir0, `the watch's last frame keeps the heir that ran on the bar (${lf.heir0} → ${heirs.join(" · ") || "no frame seen"})`);
    check(!lf.bang, "the HUD names the alert (`alert 3`), never `!!!`");
    // QA 92eb880 (N: "VERDICT appears while the hero is still up"): through the walk-out the stilled pause (disabled), then `verdict` / `report`
    const last = lf.at.at(-1)?.gem;
    check(lf.at.length > 0 && (last === "verdict" || last === "report") && lf.at.every((a) => a.gem === last || (a.gem === "⏸" && a.dis) || a.gem === "▶") && lf.at.every((a) => !a.corner), `the last frame's gem slot holds the stilled pause through the walk-out, then a gem, \`verdict\` / \`report\`, no corner label (${gems.join(" · ")})`);
  }
  if (s.screen !== "death") {   // a report: the gem is `camp`; then a fabricated death for the death's frame
    f = await frame();
    check(f.bar && f.console && f.gem?.visible && f.gem.text === "camp", `report: bar + console, \`camp\` in the gem (${JSON.stringify(f.gem)})`);
    const r = await page.evaluate(async () => { const r = window.__riddle; const L = await r.engine.lineage(); const g = L.graveyard.at(-1); return g?.death_id; });
    if (r !== undefined) { await page.evaluate(async (id) => { const r = window.__riddle; r.go({ kind: "death", death: await r.engine.death(id), kept: true }); }, r); await waitFor((x) => x?.screen === "death", "a death"); await settle(); }
  }
  if ((await state())?.screen === "death") {
    f = await frame();
    const patch = await page.locator("button.patch.top").count();
    check(f.bar && f.console && f.gem?.visible && (patch ? /\bpatch-gem\b/.test(f.gem.cls) : f.gem.text === "edit"), `death: bar + console, the top patch in the gem (${JSON.stringify(f.gem)}, ${patch} lit patch)`);
    const d = await page.evaluate(() => ({ banner: !!document.querySelector(".banner-cloth .death-line"), seal: !!document.querySelector(".banner-cloth .death-line .verdict"), tiles: [...document.querySelectorAll(".console .cmd .tile:not(.empty)")].map((t) => t.textContent.trim()) }));
    check(d.banner && d.seal && d.tiles.includes("morgue") && d.tiles.includes("camp"), `death: the line on the banner, the verdict in the seal, morgue · camp on the card (${d.tiles.join(" · ")})`);
    await shot("ui-death");
    // the morgue's sheet carries its stud too
    await page.locator(".cmd .tile[data-tile=morgue]").click({ timeout: 5000 }); await sleep(200);
    check(await page.locator(".sheet-wrap .sheet > .close-stud").count() === 1, "the morgue sheet has its close stud");
    await page.keyboard.press("Escape"); await sleep(100);
  } else check(false, "no death to check the death's frame on");
  await page.goto(`${url}?dev=1&engine=fake&systems=none&absent=2h`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && (x.screen === "report" || x.screen === "ending"), "the report", 120_000); await settle();
  f = await frame();
  const plaques = await page.locator(".report .parchment .tile.plaque").count();
  check(f.bar && f.console && f.gem?.visible && f.gem.text === "camp" && plaques >= 4, `report: bar + console, \`camp\` in the gem, the counts as plaques on parchment (${plaques} plaques)`);
  await shot("ui-report");
  // QA 23ed91f (K: the walk printed `(no worst death to open)` over 16 deaths): the report's `open` tile opens the worst death
  if (await page.evaluate(() => !!document.querySelector(".cmd .tile[data-tile=open]"))) {
    await page.locator(".cmd .tile[data-tile=open]").click({ timeout: 5000 });
    const o = await waitFor((x) => x?.screen === "death", "the worst death", 10_000).catch(() => null);
    check(o?.screen === "death", "the report's `open` tile opens the worst death");
  } else check(await page.evaluate(() => { const r = window.__riddle; return r.view?.report ? !r.view.report.worst_death : true; }), "no `open` tile: the report has no worst death");
  await qaK();
  await stallSkip();
  await autoKeepCheck();
  await cut18();
  await cut19();
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
const secs = ((Date.now() - t0) / 1000).toFixed(1);
if (failed || errors.length) { console.error(`ui: FAIL (${failed} assertion(s), ${errors.length} error(s), ${secs}s)`); process.exit(1); }
console.log(`ui: ok (${out.length} checks · ${secs}s)`);
