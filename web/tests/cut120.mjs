#!/usr/bin/env node
// Cut 120 — automation fill and chore reduction, the client half (docs/CUT120_AUTOMATION_FILL.md §8–11); headless, tools/browser.mjs.
//   sheet  — real wasm (the earned fixture, three bloodlines): one orders sheet holds every standing order — insure, the apprentice's
//            forge and sinks, heir, kennel, Legacy, ranks, the rank-II perk chips (setPerk) — each shareable row with `same for all`;
//            no ascend row before the King; the ladder's orders rung and the run setup tablet both open it.
//   audit  — the chore audit (§11), taps counted on the real engine, before and after:
//              buying a rank       before 2 a rank (works · promote)          after 0 (`ranks auto`: the report's `ranks` line)
//              spending Legacy     before 1 + 1 an upgrade (panel · buy)       after 0 (`balanced`: the report's `legacy` line)
//              one order, all      before 3 a bloodline (switch · sheet · chip) after 3 in all (sheet · same for all · chip)
//              routine check-in    3 bloodlines, `same for all` set: report → watch ≤ 2 taps
//   ledger — the While away Workers fold: one line per worker (none cut), the order lines (`ranks`, `legacy`), the purse each moved.
//   posts  — the town: the next worker's post shows `forge 2/3 · or 30h` (Works.next_worker's progress · eta_h), ≤ 3 markers.
//   news   — `k: "order"` news reaches the crier and the diary.
//   node web/tests/cut120.mjs [--part=sheet,audit,ledger,posts,news] [--shot=path.png]
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const ALL = "sheet,audit,ledger,posts,news";
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? ALL).split(",");
const shotPath = process.argv.find((a) => a.startsWith("--shot="))?.slice(7);
const engine = readFileSync(new URL("./fixtures/earned-gunner-home.json", import.meta.url), "utf8");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();

/** The earned fixture, three bloodlines (the town's own `addBloodline`, paid from the purse), at camp. */
async function earned(tag) {
  const page = await browser.newPage({ viewport: { width: 400, height: 860 }, reducedMotion: "reduce" });
  page.on("pageerror", (e) => errors.push(`${tag} pageerror: ${e.message}`));
  await page.goto(`${url}?fresh=1&seed=52`);
  await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
  const slots = await page.evaluate(async ({ engine }) => {
    const a = window.__riddle;
    if (!await a.importSave(JSON.stringify({ v: 2, engine, loadout: [], last_seen: Date.now(), runs: 0 }))) throw Error("import");
    const home = a.lineage.selected_bloodline ?? 1;
    for (let i = 0; i < 2; i++) await a.engine.addBloodline();
    await a.engine.selectBloodline(home);
    await a.refresh();
    a.go({ kind: "camp" });
    return a.lineage.hero_slots?.length ?? 0;
  }, { engine });
  await page.waitForTimeout(600);
  return { page, slots };
}
/** close every open sheet as a player does (Escape), until none is left */
async function closeSheets(page) {
  for (let i = 0; i < 4 && await page.locator(".sheet-wrap").count(); i++) { await page.keyboard.press("Escape"); await page.waitForTimeout(250); }
}
const sleep = (ms) => new Promise((res) => setTimeout(res, ms));
/** every bloodline's orders, read through the engine (the selection restored) */
const allOrders = (page) => page.evaluate(async () => {
  const a = window.__riddle, home = a.lineage.selected_bloodline ?? 1, ids = (a.lineage.hero_slots ?? []).map((s) => s.id), o = {};
  for (const id of ids) { await a.engine.selectBloodline(id); o[id] = (await a.engine.lineage()).orders; }
  await a.engine.selectBloodline(home); await a.refresh();
  return o;
});

try {
  if (parts.includes("sheet") || parts.includes("audit") || parts.includes("ledger")) {
    const { page, slots } = await earned("sheet");
    check(slots === 3, `three bloodlines in the town (${slots})`);

    // ---- before: buying a rank by hand (ranks off, the fixture's own) — the pill opens the works, the promote buys it: 2 taps
    const before = await page.evaluate(async () => {
      const a = window.__riddle, L = a.lineage, id = L.tree?.lit_rank, r0 = L.tree?.nodes.find((n) => n.id === id)?.rank ?? 0;
      return { id, r0, ranks: L.orders?.ranks, legacy: L.orders?.legacy, due: (L.tree?.nodes ?? []).filter((n) => n.state === "done" && n.rank_wait_d === 0 && n.rank_price).length };
    });
    let rankTaps = 0;
    await page.locator(".next-pill:visible").click(); rankTaps++;
    await page.locator(`.works-sheet .promote-btn[data-node="${before.id}"]`).click(); rankTaps++;
    await page.waitForFunction(({ id, r0 }) => (window.__riddle.lineage.tree?.nodes.find((n) => n.id === id)?.rank ?? 0) > r0, before, { timeout: 5000 }).catch(() => {});
    const r1 = await page.evaluate((id) => window.__riddle.lineage.tree?.nodes.find((n) => n.id === id)?.rank ?? 0, before.id);
    check(before.ranks === "off" && before.legacy === "off" && r1 === before.r0 + 1 && rankTaps === 2,
      `before: a rank by hand is ${rankTaps} taps (${before.id} ${before.r0} → ${r1}; ${before.due} ranks due → ${before.due * 2} taps)`);
    await closeSheets(page);
    await page.waitForTimeout(200);

    // ---- the sheet: from the ladder's orders rung (when the ladder shows), else the run setup tablet
    let sheetTaps = 0;
    const door = page.locator('.control-ladder:not([hidden]) .ladder-door[data-door="orders"]');
    const viaLadder = await door.count() > 0 && await door.isVisible();
    if (viaLadder) await door.click(); else await page.locator(".orders-tab:visible").click();
    sheetTaps++;
    await page.waitForSelector(".orders-sheet .order-row");
    const rows = await page.evaluate(() => [...document.querySelectorAll(".orders-sheet .order-row")].map((r) => ({
      label: r.querySelector(".olab")?.textContent ?? "", order: r.dataset.order ?? "", perkOf: r.dataset.perkOf ?? "", share: !!r.querySelector("[data-share]"),
      chips: [...r.querySelectorAll(".chip.order:not(.share)")].map((c) => `${c.textContent.trim()}${c.classList.contains("on") ? "*" : ""}`) })));
    const byOrder = Object.fromEntries(rows.filter((r) => r.order).map((r) => [r.order, r]));
    const shown = await page.evaluate(async () => { const { kennelShown } = await import("/src/ui/pets.ts"), { kingSlain } = await import("/src/ui/king-eta.ts"); return { kennel: kennelShown(window.__riddle.lineage), king: kingSlain(window.__riddle.lineage) }; });
    check(["insure", "forge", "sink", "heir", "legacy", "ranks", ...(shown.kennel ? ["kennel"] : [])].every((k) => byOrder[k]) && !byOrder.wall,
      `one sheet: every standing order a chip row, the retired wall order none (${rows.map((r) => r.order || r.perkOf || r.label).join(", ")})`);
    check(byOrder.legacy?.chips.join(",") === "balanced,health,damage,armour,off*" && byOrder.ranks?.chips.join(",") === "auto,off*",
      `Legacy and ranks orders, the save's off lit (${byOrder.legacy?.chips} · ${byOrder.ranks?.chips})`);
    check(Object.values(byOrder).every((r) => r.share), `each shareable row offers same for all with three bloodlines (${Object.values(byOrder).filter((r) => r.share).length}/${Object.keys(byOrder).length})`);
    check(!!byOrder.ascend === shown.king && (!shown.king || byOrder.ascend.chips.join(",") === "off*,on"), `the ascend row only after the King (King slain: ${shown.king} · ${byOrder.ascend?.chips ?? "no row"})`);
    const perks = rows.filter((r) => r.perkOf);
    check(perks.map((r) => r.perkOf).sort().join(",") === "apprentice,scout" && perks.every((r) => / II$/.test(r.label) && r.chips.length === 2 && r.chips[0].endsWith("*")),
      `the rank-II perk chips on the workers' rows, the first the default (${perks.map((r) => `${r.label}: ${r.chips.join(" · ")}`).join(" | ")})`);

    if (shotPath) { await page.locator('.orders-sheet [data-order="legacy"]').scrollIntoViewIfNeeded(); await page.screenshot({ path: shotPath, fullPage: false }); }

    // ---- a perk chip: setPerk
    await page.locator('.orders-sheet [data-perk-of="scout"] [data-perk="safer start"]').click();
    await page.waitForTimeout(400);
    const perk = await page.evaluate(async () => (await window.__riddle.engine.lineage()).tree?.nodes.find((n) => n.id === "scout")?.perk);
    check(perk === "safer start", `a perk chip sets the worker's perk (setPerk → ${perk})`);
    await page.locator('.orders-sheet [data-perk-of="scout"] [data-perk="shorter rest"]').click();
    await page.waitForTimeout(300);

    // ---- one order for all bloodlines: same for all, then the chip (the sheet already open: 1 tap)
    let allTaps = sheetTaps;
    await page.locator('.orders-sheet [data-share="legacy"]').click(); allTaps++;
    await page.waitForTimeout(300);
    await page.locator('.orders-sheet [data-legacy="balanced"]').click(); allTaps++;
    await page.waitForTimeout(400);
    let o = await allOrders(page);
    check(Object.values(o).length === 3 && Object.values(o).every((x) => x.legacy === "balanced" && x.shared?.includes("legacy")),
      `one order set for all bloodlines in ${allTaps} taps, before ${3 * 3} (${Object.entries(o).map(([id, x]) => `${id}:${x.legacy}`).join(" ")})`);
    // the ranks order likewise (the sheet still open: 2 taps), so the routine below runs on both orders in every bloodline
    await page.locator('.orders-sheet [data-share="ranks"]').click(); await page.waitForTimeout(300);
    await page.locator('.orders-sheet [data-ranks="auto"]').click(); await page.waitForTimeout(400);
    o = await allOrders(page);
    check(Object.values(o).every((x) => x.ranks === "auto" && x.shared?.includes("ranks")), `ranks auto shared to every bloodline (${Object.values(o).map((x) => x.ranks).join(",")})`);
    // a later change follows; then back
    await closeSheets(page);
    await page.locator(".orders-tab:visible").click();
    await page.waitForSelector('.orders-sheet [data-legacy="damage"]');
    const sharedLit = await page.evaluate(() => document.querySelector('.orders-sheet [data-share="legacy"]')?.getAttribute("aria-pressed"));
    await page.locator('.orders-sheet [data-legacy="damage"]').click(); await page.waitForTimeout(400);
    o = await allOrders(page);
    const sum = await page.evaluate(() => document.querySelector(".orders-sum")?.textContent ?? "");
    check(sharedLit === "true" && Object.values(o).every((x) => x.legacy === "damage") && /legacy damage/.test(sum), `a later change follows to every bloodline; the summary names the focus (${sum})`);
    await page.locator('.orders-sheet [data-legacy="balanced"]').click(); await page.waitForTimeout(400);
    await closeSheets(page);

    // ---- the absence: ranks and Legacy bought with no tap; the report's ledger; the routine return ≤ 2 taps
    const pre = await page.evaluate(() => { const L = window.__riddle.lineage; return { ranks: Object.fromEntries((L.tree?.nodes ?? []).filter((n) => n.rank).map((n) => [n.id, n.rank])), spent: L.bloodline?.spent ?? 0 }; });
    const rep = await page.evaluate(async () => {
      const a = window.__riddle, rep = await a.engine.runOfflineQuick(8 * 3600);
      await a.refresh();
      window.__rep120 = rep;
      return { workers: rep.workers ?? [], feats: rep.feats ?? [], packages: rep.packages ?? [] };
    });
    const post = await page.evaluate(() => { const L = window.__riddle.lineage; return { ranks: Object.fromEntries((L.tree?.nodes ?? []).filter((n) => n.rank).map((n) => [n.id, n.rank])), spent: L.bloodline?.spent ?? 0 }; });
    const ranksLine = rep.workers.find((w) => w.id === "ranks"), legacyLine = rep.workers.find((w) => w.id === "legacy");
    const rose = Object.keys(post.ranks).filter((k) => post.ranks[k] > (pre.ranks[k] ?? 0));
    check(!!ranksLine && ranksLine.n > 0 && rose.length > 0, `after: ranks bought with 0 taps (${ranksLine?.what ?? "no line"} · ${rose.join(",")}; before ${2 * (ranksLine?.n ?? 0)} taps)`);
    check(!!legacyLine && legacyLine.n > 0 && post.spent > pre.spent, `after: Legacy spent with 0 taps (${legacyLine?.what ?? "no line"} · spent ${pre.spent} → ${post.spent}; before ${1 + (legacyLine?.n ?? 0)} taps)`);
    check(rep.feats.some((f) => f.k === "order"), `the orders announced as news (${rep.feats.filter((f) => f.k === "order").map((f) => f.text).join(", ")})`);

    // the While away report: the Workers fold is the ledger, one line per worker
    await page.evaluate(() => window.__riddle.go({ kind: "report", report: window.__rep120, absence: true }));
    await page.waitForTimeout(700);
    const led = await page.evaluate(() => {
      const fold = document.querySelector('.report-details [data-group="workers"] .works-ledger');
      return { lines: [...(fold?.querySelectorAll(".work-act:not(.chest)") ?? [])].map((x) => ({ id: x.dataset.worker, text: x.textContent.replace(/\s+/g, " ").trim() })),
        more: !!fold?.querySelector(".dim"), firsts: [...document.querySelectorAll(".report-first-workers .work-act")].map((x) => x.dataset.worker) };
    });
    const orderLine = (id) => id === "ranks" || id === "legacy";
    const routine = rep.workers.filter((w) => (!w.first || orderLine(w.id)) && (w.n > 0 || w.what));
    check(led.lines.length === routine.length && !led.more && new Set(led.lines.map((l) => l.id)).size === led.lines.length,
      `the Workers fold: one line per worker, none cut (${led.lines.length}/${routine.length})`);
    const rl = led.lines.find((l) => l.id === "ranks"), ll = led.lines.find((l) => l.id === "legacy");
    check(!!rl && /^ranks · /.test(rl.text) && /−\$\d/.test(rl.text) && !!ll && /^legacy · /.test(ll.text),
      `the order lines in the ledger (${rl?.text} | ${ll?.text})`);
    check(rep.workers.filter((w) => w.first && !orderLine(w.id)).every((w) => led.firsts.includes(w.id) && led.lines.every((l) => l.id !== w.id)) && !led.firsts.some(orderLine),
      `a worker's first act stays a highlight, the order lines stay in the ledger (${led.firsts.join(",") || "none this absence"})`);

    // the routine return, three bloodlines, same for all set: report → watch
    const back = await page.evaluate(async () => {
      const a = window.__riddle, btn = () => document.querySelector(".report .collect-go");
      let taps = 0;
      for (let i = 0; i < 3 && a.screen !== "watch" && btn(); i++) {
        btn().click(); taps++;
        for (let k = 0; k < 15 && a.screen !== "watch"; k++) await new Promise((res) => setTimeout(res, 100));
      }
      for (let i = 0; i < 60 && a.screen !== "watch"; i++) await new Promise((res) => setTimeout(res, 100));
      return { taps, screen: a.screen, slots: a.lineage.hero_slots?.length };
    });
    check(back.taps <= 2 && back.screen === "watch" && back.slots === 3, `a routine 3-bloodline check-in: ${back.taps} taps report → watch, no order or rank touched (${back.screen})`);
    out.push(`     audit: rank ${2}→0 · Legacy ${1 + (legacyLine?.n ?? 0)}→0 · one order for all ${3 * 3}→${allTaps} · check-in ${back.taps}`);
    await page.close();
  }

  if (parts.includes("posts")) {
    const { page } = await earned("posts");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, L = a.lineage;
      a.engine.lineage = async () => a.lineage;   // (the fixture below is the real lineage with the next worker counting)
      const nodes = L.tree.nodes.map((n) => n.id === "apprentice" ? { ...n, state: "open", progress: "2/3", count: 2, need: 3, chore: "forge", eta_h: 30, fallback_h: 48 } : n);
      a.lineage = { ...L, tree: { ...L.tree, nodes, next_worker: "apprentice", lit: undefined, lit_rank: undefined }, town: { ...L.town, workers: (L.town?.workers ?? []).filter((w) => w.id !== "apprentice" && !w.lit) } };
      a.go({ kind: "report", report: { elapsed_s: 0, runs: 0, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 0 } });
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 900));
      const mk = [...document.querySelectorAll(".town-hit .town-marker")].filter((m) => m.getClientRects().length);
      const next = document.querySelector('.town-hit .town-marker[data-marker="next"]');
      return { text: next?.textContent ?? "", at: next?.closest(".town-hit")?.dataset.building, n: mk.length, kinds: mk.map((m) => m.dataset.marker) };
    });
    check(r.text === "forge 2/3 · or 30h" && r.at === "blacksmith", `the next worker's post: ${r.text} at the ${r.at}`);
    check(r.n <= 3, `≤ 3 markers on the town (${r.kinds.join(",")})`);
    const k = await page.evaluate(async () => {
      const { nextWorkerMark } = await import("/src/ui/town.ts");
      const keeper = { id: "kennel_keeper", kind: "worker", name: "kennel keeper", state: "open", branch: "character", post: "kennel", chore: "breed", progress: "1/2", eta_h: 12 };
      const lit = { ...keeper, state: "lit" };
      return { open: nextWorkerMark({ tree: { nodes: [keeper], next_worker: "kennel_keeper" } }), lit: nextWorkerMark({ tree: { nodes: [lit], next_worker: "kennel_keeper" } }) };
    });
    check(k.open?.text === "breed 1/2 · or 12h" && k.open.post === "kennel" && k.lit === null, `the kennel keeper's post is the kennel (${k.open?.text}); a lit worker carries his price instead`);
    await page.close();
  }

  if (parts.includes("news")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 860 } });
    page.on("pageerror", (e) => errors.push(`news pageerror: ${e.message}`));
    await page.goto(`${url}?engine=fake&fresh=1&seed=5&runs=0`);
    await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 60_000 });
    const r = await page.evaluate(async () => {
      const { cries, writeDiary } = await import("/src/ui/crier.ts");
      const rep = { elapsed_s: 0, runs: 0, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 0,
        feats: [{ k: "order", text: "legacy balanced", day: 3 }, { k: "order", text: "ranks auto", day: 3 }] };
      try { localStorage.removeItem("riddle.diary.991"); } catch {}
      const cs = cries(rep, { walls: [] });
      const diary = writeDiary({ seed: 991, age_h: 80 }, cs);
      return { cries: cs.map((c) => `${c.k}:${c.short}`), diary: diary.map((d) => `${d.day}:${d.text}`) };
    });
    check(r.cries.join() === "order:legacy balanced,order:ranks auto", `order news cried (${r.cries.join(", ")})`);
    check(r.diary.some((d) => d === "3:New order in town · legacy balanced"), `order news in the diary, dated (${r.diary.join(" | ")})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut120: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut120: ok (${out.length} checks)`);
