#!/usr/bin/env node
// Cut 118 — lessons from the idle genre; client half (headless, tools/browser.mjs; the fake engine, the ui modules direct):
//   taps     — a routine return: the report's `collect & send` opens the chest, takes the routine buys and sends — ≤ 2 taps from the
//              report to the watch; the orders and the buys fold under it.
//   decide   — at most one decision prompt in the report's card (the pick, else the new choices, else the preparation); the rest
//              wait under `details` and move up when the first is taken.
//   ladder   — weights → orders → workers → pen: lit as the lineage has them, the first dark rung's condition shown; gone when all lit.
//   crier    — one line per notable act of the last absence; ≤ 3 words on the town, the scroll longer; read once, quiet.
//   king     — `King · ~day N` (crude, its tip says so; none before D13): later for a slower pace, none once he is slain; the away report's
//              time away carries `uncapped` in its tip, the King's line waits in its fold (owner IA pass 2026-10-10).
//   node web/tests/cut118.mjs [--part=taps,decide,ladder,crier,king]
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "taps,decide,ladder,crier,king").split(",");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
const fresh = async (tag) => {
  const page = await browser.newPage({ viewport: { width: 400, height: 860 } });
  page.on("pageerror", (e) => errors.push(`${tag} pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&engine=fake&systems=all&fresh=1&seed=5&runs=0`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
  await page.waitForTimeout(300);
  await page.evaluate(() => { const a = window.__riddle; a.engine.lineage = async () => a.lineage; });
  return page;
};
const REPORT = { elapsed_s: 28800, runs: 12, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 14, gold: { home: 900, salvage: 40, wake: 0, spent: 0 } };
try {
  if (parts.includes("taps")) {
    const page = await fresh("taps");
    const r = await page.evaluate(async (REP) => {
      const app = window.__riddle;
      const wait = (ms) => new Promise((res) => setTimeout(res, ms));
      const step = (label, price) => ({ label, price, affordable: true });
      const kit = [{ slot: "weapon", owned: 1, steps: [], next: step("sword +2", 300) }, { slot: "pack", owned: 0, steps: [], next: step("satchel", 150) }];
      const tree = { ...(app.lineage.tree ?? { nodes: [], waits: false, sent: false, ledger: 0 }), chest: 120, auto_send: true };
      const orders = { keep: "best_weapon", cage: "weapon", start: 1, repeat: true, insure: false, wall: "bank" };
      app.lineage = { ...app.lineage, kit, tree, orders, gold: 5000, live: null, ended: false, town: { ...(app.lineage.town ?? { buildings: [], bank: 0, bank_cap: 0, interest: 0 }), home: true } };
      const calls = [];
      app.engine.openChest = () => { calls.push("chest"); return { ...app.lineage, gold: app.lineage.gold + app.lineage.tree.chest, tree: { ...app.lineage.tree, chest: 0 } }; };
      app.engine.buyKit = (slot) => { calls.push(slot); return { ...app.lineage, kit: app.lineage.kit.map((k) => (k.slot === slot ? { ...k, next: undefined } : k)) }; };
      app.go({ kind: "report", report: REP, absence: true });
      await wait(500);
      const sec = document.querySelector(".report .collect-send");
      const btn = () => document.querySelector(".report .collect-go");
      const label = btn()?.textContent;
      const fold = [...(sec?.querySelectorAll(".collect-lines li") ?? [])].map((li) => li.textContent);
      // the action stands after the highlights, before any decision
      const order = [...document.querySelector(".report-sheet").children].map((c) => c.className.split(" ")[0]);
      let taps = 0;
      btn().click(); taps++;
      const armed = btn()?.textContent, before = calls.length;
      btn().click(); taps++;
      for (let i = 0; i < 60 && app.screen !== "watch"; i++) await wait(100);
      return { label, armed, before, calls, taps, screen: app.screen, fold, order };
    }, REPORT);
    check(r.label === "collect & send" && r.armed === "ok $450" && r.before === 0, `one action, armed before it pays (${r.label} → ${r.armed}, ${r.before} calls)`);
    check(r.calls.join() === "chest,weapon,pack" && r.taps === 2 && r.screen === "watch", `two taps: chest, the routine buys, the send (${r.calls} · ${r.taps} taps · ${r.screen})`);
    check(r.fold.some((t) => t === "keep weapon") && r.fold.some((t) => t === "chest +$120") && r.fold.some((t) => /sword \+2/.test(t)), `the orders and the buys fold under it (${r.fold.join(" | ")})`);
    check(r.order.indexOf("collect-send") > r.order.indexOf("report-summary") && r.order.indexOf("collect-send") < r.order.indexOf("details-fold"), `the action follows the highlights (${r.order.join(" ")})`);
    // nothing to pay: one tap
    const page2 = await fresh("taps1");
    const one = await page2.evaluate(async (REP) => {
      const app = window.__riddle;
      app.lineage = { ...app.lineage, kit: [], tree: { ...(app.lineage.tree ?? { nodes: [], waits: false, sent: false, ledger: 0 }), chest: 0, auto_send: true }, live: null, ended: false, town: { ...(app.lineage.town ?? { buildings: [], bank: 0, bank_cap: 0, interest: 0 }), home: true } };
      app.go({ kind: "report", report: REP, absence: true });
      await new Promise((res) => setTimeout(res, 400));
      const b = document.querySelector(".report .collect-go");
      const label = b?.textContent;
      b.click();
      for (let i = 0; i < 60 && app.screen !== "watch"; i++) await new Promise((res) => setTimeout(res, 100));
      return { label, screen: app.screen };
    }, REPORT);
    check(one.label === "send" && one.screen === "watch", `nothing to collect: one tap sends (${one.label} → ${one.screen})`);
    await page.close(); await page2.close();
  }
  if (parts.includes("decide")) {
    const page = await fresh("decide");
    const r = await page.evaluate(async (REP) => {
      const app = window.__riddle;
      const wait = (ms) => new Promise((res) => setTimeout(res, ms));
      const offer = { id: "health", rank: 0, cap: 10, price: 10, effect: "+5 hp", affordable: true };
      const pick = { size: 1, offers: [{ id: "drill", title: "Steady", line: "+3 runs", available: true }, { id: "legacy", title: "Legacy", line: "+4", available: true }, { id: "forge", title: "sword +2", line: "−20%", price: 240, available: true }] };
      app.lineage = { ...app.lineage, live: null, selected_bloodline: 1, bloodline: { points: 33, spent: 0, upgrades: {} }, legacy_upgrades: [offer], return_pick: pick };
      app.engine.upgradeHero = async () => app.lineage;
      app.engine.takeReturnPick = () => ({ ...app.lineage, return_pick: undefined });
      app.go({ kind: "report", report: REP, absence: true });
      await wait(500);
      const card = () => [".return-pick", ".report-choices", ".report-upgrade-host"].filter((s) => { const el = document.querySelector(`.report-sheet ${s}`); return el && !el.hidden && !el.closest(".report-details"); });
      const def0 = document.querySelector(".report .return-pick .tile[data-default]")?.dataset.offer;
      const first = card(), later = !!document.querySelector(".report-details .report-upgrade-host");
      document.querySelector('.report .return-pick .tile[data-offer="legacy"]').click();
      await wait(500);
      const def = document.querySelector(".report .return-pick .tile[data-default]")?.dataset.offer;
      return { first, later, after: card(), def: def0 };
    }, REPORT);
    check(r.first.length === 1 && r.first[0] === ".return-pick" && r.later, `one decision in the card, the pick; the preparation waits under details (${r.first} · ${r.later})`);
    check(r.after.length === 1 && r.after[0] === ".report-upgrade-host", `the pick taken, the next decision moves up (${r.after})`);
    check(r.def === "legacy", `the one prompt shows its default (${r.def})`);
    await page.close();
  }
  if (parts.includes("ladder")) {
    const page = await fresh("ladder");
    const r = await page.evaluate(async () => {
      const app = window.__riddle;
      const { ladderRungs, controlLadder } = await import("/src/ui/ladder.ts");
      const base = app.lineage;
      const pk = (pen, stances) => ({ ...(base.packages ?? { all: [], stance: "steady" }), literal: false, pen_open: pen, all: stances ? [{ id: "steady", kind: "stance", owned: true }, { id: "guarded", kind: "stance", owned: true }] : [{ id: "steady", kind: "stance", owned: true }] });
      const W = (hired) => ({ nodes: [{ id: "quartermaster", kind: "worker", name: "quartermaster", state: "done" }, { id: "porter", kind: "worker", name: "porter", state: hired ? "done" : "lit", chore: "chest", count: 2, need: 3 }], chest: 0, waits: false, sent: false, auto_send: false, ledger: 0 });
      const sys = [{ id: "stances", open: false, trigger: "first bank" }, { id: "pen", open: false, trigger: "meet Mother" }, { id: "loadout", open: false, trigger: "first salvage" }];
      const early = { ...base, best_depth: 4, packages: pk(false, false), tree: W(false), systems: sys, orders: undefined };
      const mid = { ...early, tree: W(true), orders: { keep: "best_weapon", cage: "weapon", start: 1, repeat: true, insure: true } };
      const all = { ...mid, packages: pk(true, true), systems: sys.map((s) => ({ ...s, open: true })) };
      const has = (s) => s === "loadout";
      const ids = (L, h) => ladderRungs(L, h).map((x) => `${x.id}${x.lit ? "+" : "-"}`).join(" ");
      const e = ladderRungs(early, () => false);
      app.lineage = early;
      const lad = controlLadder(app); document.body.appendChild(lad.el);
      const shown = { lit: lad.el.dataset.ladder, next: lad.el.querySelector(".ladder-next")?.textContent, rungs: [...lad.el.querySelectorAll(".ladder-rung")].map((x) => x.textContent) };
      app.lineage = all; app.emitChange?.();
      await new Promise((res) => setTimeout(res, 50));
      const gone = lad.el.hidden;
      app.lineage = base; app.emitChange?.();
      const w = ladderRungs(mid, has).find((x) => x.id === "workers");
      return { early: ids(early, () => false), mid: ids(mid, has), all: ids(all, has), cond: e.find((x) => !x.lit)?.cond, shown, gone, w: { retires: w.retires, litBy: w.litBy }, every: ladderRungs(all, has).every((x) => x.retires && x.litBy) };
    });
    check(r.early === "weights- orders- workers- pen-" && r.cond === "first bank", `a fresh ladder is dark, the first rung names its condition (${r.early} · ${r.cond})`);
    check(r.mid === "weights- orders+ workers+ pen-" && r.all === "weights+ orders+ workers+ pen+", `rungs light as earned, in any order (${r.mid} | ${r.all})`);
    check(r.w.retires === "chests by hand" && r.w.litBy === "3 chests" && r.every, `each rung names the chore it retires and what lit it (${JSON.stringify(r.w)})`);
    check(r.shown.rungs.join() === "tactics,orders,workers,pen" && r.shown.next === "first bank" && r.gone, `the rail: four words and one condition; gone when all lit (${JSON.stringify(r.shown)} · ${r.gone})`);
    await page.close();
  }
  if (parts.includes("crier")) {
    const page = await fresh("crier");
    const r = await page.evaluate(async (REP) => {
      const app = window.__riddle;
      const wait = (ms) => new Promise((res) => setTimeout(res, ms));
      const { cries } = await import("/src/ui/crier.ts");
      const rep = { ...REP, bests: ["boss: goblin_warlord", "D9", "D10"], packages: ["built bank", "QUEST DONE · reach D10"], workers: [{ id: "porter", what: "+3 hauls", n: 3, first: true }], bounty: { depth: 10, taken: true, gold: 80 },
        exits: [{ carried: 0, keep_pct: 100, kept: 0, spent: 0, spent_on: [], text: "banked $0", news: [{ k: "named", text: "avenged Zelul" }] }] };
      try { localStorage.removeItem(`riddle.diary.${app.lineage.seed ?? 0}`); } catch {}
      const L = { ...app.lineage, walls: [{ boss: "goblin_warlord", title: "Warlord", depth: 8, slain: true, known: true, fact: "" }] };
      const cs = cries(rep, L);
      const words = (t) => t.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length;
      app.lineage = { ...L, town: { ...(app.lineage.town ?? { buildings: [], bank: 0, bank_cap: 0, interest: 0 }), home: true } };
      app.lastAbsence = { report: rep, played: false };
      app.go({ kind: "camp" });
      await wait(600);
      const el = document.querySelector(".town-crier");
      const chrome = { shown: !!el && !el.hidden, text: el?.querySelector(".crier-cry")?.textContent, ringing: el?.classList.contains("ringing") };
      el?.click();
      await wait(300);
      const scroll = [...document.querySelectorAll(".crier-scroll .crier-line")].map((li) => li.textContent);
      const { closeAllSheets } = await import("/src/ui/sheet.ts"); closeAllSheets();
      app.go({ kind: "camp" });
      await wait(400);
      const quiet = document.querySelector(".town-crier")?.hidden;
      return { ks: cs.map((c) => c.k), shortMax: Math.max(...cs.map((c) => words(c.short))), chrome, scroll, quiet };
    }, REPORT);
    check(r.ks.join() === "boss,record,avenged,quest,built,worker", `one line per notable act from the real events, the boss first, ≤ 6 (${r.ks})`);
    check(r.shortMax <= 3 && r.chrome.shown && r.chrome.text === "slew the Warlord +5" && r.chrome.ringing, `the hero's voice, ≤ 3 words on the town, ringing once (${JSON.stringify(r.chrome)})`);
    check(r.scroll.length === 6 && /^day \d+ I slew the Warlord on D8$/.test(r.scroll[0]) && r.quiet, `the diary: dated lines, longer; read once, he falls quiet (${r.scroll.join(" | ")} · quiet ${r.quiet})`);
    await page.close();
  }
  if (parts.includes("king")) {
    const page = await fresh("king");
    const r = await page.evaluate(async (REP) => {
      const app = window.__riddle;
      const { kingEta } = await import("/src/ui/king-eta.ts");
      const L = (age_h, best, extra = {}) => ({ age_h, best_depth: best, ended: false, walls: [], runs: [], trophies: [], ...extra });
      const idle = kingEta(L(48, 13)), picked = kingEta(L(24, 13)), d23 = kingEta(L(12 * 24, 23)), early = kingEta(L(24, 8));
      const slain = kingEta(L(240, 33, { walls: [{ boss: "mirror_king", slain: true }] })), young = kingEta(L(0.5, 3));
      app.lineage = { ...app.lineage, age_h: 30, best_depth: 13, king_eta_h: undefined, king_pct: undefined, tree: { ...(app.lineage.tree ?? { nodes: [], waits: false, sent: false, ledger: 0 }), chest: 0, auto_send: true } };
      app.go({ kind: "report", report: REP, absence: true });
      await new Promise((res) => setTimeout(res, 400));
      const k = document.querySelector(".report .report-king");
      const { kingLine } = await import("/src/ui/king-eta.ts");
      const { wallPreview } = await import("/src/ui/wall-preview.ts");
      const walls = [{ boss: "goblin_warlord", title: "Goblin Warlord", depth: 8, slain: true, known: true, fact: "" }, { boss: "lich", title: "Lich", depth: 18, slain: false, known: false, fact: "" }, { boss: "bloat_mother", title: "Bloat Mother", depth: 13, slain: false, known: false, fact: "", learn: "see her" }];
      const wp = wallPreview({ ...app.lineage, walls });
      const away = document.querySelector(".report-summary .report-away");
      return { idle: idle?.day, picked: picked?.day, d23: d23?.day, slain, young, early, text: k?.textContent, pct: k?.dataset.pct, tip: k?.title, folded: !!k?.closest(".report-details [data-group=way]"), uncapped: away?.dataset.uncapped === "1" && /uncapped/.test(away?.title ?? "") ? "uncapped" : null, away: away?.textContent,
        slainText: kingLine(L(240, 33, { walls: [{ boss: "mirror_king", slain: true }] }))?.textContent, wall: wp && { boss: wp.dataset.boss, text: wp.textContent } };
    }, REPORT);
    check(r.idle > 14 && r.picked < r.idle && r.d23 > 12 && r.slain === null && r.young === null && r.early === null, `a pace, not a promise: idle day ${r.idle}, a faster lineage day ${r.picked}, D23 at day 12 → day ${r.d23}; none slain, too young or before D13`);
    check(/^King · ~day \d+39% · next: harder dungeon$/.test(r.text ?? "") && r.pct === "39" && /crude/.test(r.tip ?? "") && r.folded && r.uncapped === "uncapped" && r.away === "8h", `the away report: ${r.text} (${r.tip}) · bar ${r.pct}% · in the fold ${r.folded} · ${r.away} ${r.uncapped}`);
    check(r.slainText === "King · slain100% · next: harder dungeon", `slain, what comes after him (${r.slainText})`);
    check(r.wall?.boss === "bloat_mother" && /Bloat Mother D13/.test(r.wall.text) && /counter · \? · see her/.test(r.wall.text), `the chart's next wall: the shallowest unslain boss and his counter (${r.wall?.text})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut118: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut118: ok (${out.length} checks)`);
