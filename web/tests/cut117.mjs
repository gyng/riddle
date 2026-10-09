#!/usr/bin/env node
// Cut 117 — numbers that agree, advice that holds; client half (headless, tools/browser.mjs; the fake engine, the ui modules direct):
//   try    — a death's tactic fix names what it takes off (`try · boss focus ← Kite archers`), applies on its one tap only, never on
//            its own; a variant fix names the worn variant; an open slot names nothing. A move with no ± under 5 points reads `same`.
//   focus  — the `Warlord tactic` plaque opens the tactics panel on that drill (the fold open, the row ringed); a level plaque on
//            its package.
//   taps   — the quest swap and a breed say what happened (a toast, the card / the egg lit).
//   checkin — the routine buys of a return (an unbranched forge step, a lit hire) are one chip, two taps; branches stay out.
//   gold   — the report's gold head reconciles: earned − spent = purse (the core's `gold.ledger`; an old save derived).
//   wire   — the core's Cut 117 fields: `Death.moment_hp`, `PkgOption.even/noise`, `Forecast.hold`, `supply_budget` / `WorkerAct.reason`.
//   node web/tests/cut117.mjs [--part=try,focus,taps,checkin,gold,wire]
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "try,focus,taps,checkin,gold,wire").split(",");
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
  // the stubs below hand the app a lineage of their own: a re-read keeps it (the fake's own would undo the fixture)
  await page.evaluate(() => { const a = window.__riddle; a.engine.lineage = async () => a.lineage; });
  return page;
};
// a packages wire: two slots worn (corridor fighting, kite archers), boss focus owned with two variants, the Warlord drilled
const PACKAGES = {
  stance: "steady", tactics: ["corridor_fighting", "kite_archers"], tactic_slots: 2,
  all: [
    { id: "steady", name: "Steady", kind: "stance", level: 2, runs: 4, owned: true },
    { id: "corridor_fighting", name: "Corridor fighting", kind: "tactic", level: 3, runs: 9, owned: true, slot: 0, variants: ["hold", "press"], variant: 0 },
    { id: "kite_archers", name: "Kite archers", kind: "tactic", level: 1, runs: 2, owned: true, slot: 1 },
    { id: "boss_focus", name: "Boss focus", kind: "tactic", level: 1, runs: 0, owned: true, variants: ["burst", "hunt"], variant: 0 },
  ],
  drills: [{ boss: "goblin_warlord", revoked: false, rows: [], scar: 0 }],
};
try {
  if (parts.includes("try")) {
    const page = await fresh("try");
    const r = await page.evaluate(async (P) => {
      const app = window.__riddle;
      const { pickTablet } = await import("/src/ui/death.ts");
      const { fixReplaces, priceOf } = await import("/src/ui/packages.ts");
      const { moveOf } = await import("/src/ui/forecast.ts");
      app.lineage = { ...app.lineage, packages: P };
      let taken = 0;
      app.engine.takeFix = (id, v) => { taken++; return app.lineage; };
      const go = app.go.bind(app); app.go = () => {};   // stay on the screen
      const tab = pickTablet(app, { kind: "tactic", text: "boss focus · hunt", id: "boss_focus", variant: 1 });
      document.body.appendChild(tab);
      await new Promise((res) => setTimeout(res, 1500));
      const before = taken;
      tab.click();
      await new Promise((res) => setTimeout(res, 300));
      app.go = go;
      const open = { ...P, tactics: ["corridor_fighting"] };
      return {
        text: tab.textContent, tag: tab.querySelector(".lever-replaces")?.dataset.replaces ?? null, before, after: taken,
        variant: fixReplaces(P, "corridor_fighting", 1), sameVariant: fixReplaces(P, "corridor_fighting", 0), openSlot: fixReplaces(open, "boss_focus", 1),
        small: moveOf({ delta: 0.03 })?.text, big: moveOf({ delta: 0.06 })?.text, inside: moveOf({ delta: 0.08, pm: 0.1 })?.text,
        price: priceOf({ d_past: 0.02, d_death: 0, d_bank: 0, past: 0.5, death: 0.2, bank: 0.3 }, 400).text,
      };
    }, PACKAGES);
    check(r.tag === "Kite archers" && /try.*boss focus · hunt ← Kite archers/.test(r.text), `TRY names what it replaces (${r.text})`);
    check(r.before === 0 && r.after === 1, `the fix applies on its one tap, never on its own (${r.before} → ${r.after})`);
    check(r.variant === "hold" && r.sameVariant === null && r.openSlot === null, `a variant fix names the worn variant; an open slot nothing (${r.variant}, ${r.sameVariant}, ${r.openSlot})`);
    check(r.small === "same" && r.big === "+6" && r.inside === "same", `a move with no ± under 5 points is no call (${r.small}, ${r.big}, ${r.inside})`);
    check(r.price === "—", `a rounded price under the band is no call (${r.price})`);
    await page.close();
  }
  if (parts.includes("focus")) {
    const page = await fresh("focus");
    const r = await page.evaluate(async (P) => {
      const app = window.__riddle;
      const { openPackages } = await import("/src/ui/packages.ts");
      const { trainingFocus } = await import("/src/ui/tracks.ts");
      app.lineage = { ...app.lineage, packages: P };
      const f = trainingFocus("DRILLED · Warlord");
      openPackages(app, null, undefined, f);
      await new Promise((res) => setTimeout(res, 400));
      const ring = document.querySelector(".pkg-panel .pkg-drill.pkg-focus");
      const fold = document.querySelector(".pkg-panel .pkg-advanced");
      const box = ring?.getBoundingClientRect(), sheet = document.querySelector(".sheet")?.getBoundingClientRect();
      const drill = { f, boss: ring?.dataset.boss ?? null, open: !!fold?.open, seen: !!box && !!sheet && box.top >= sheet.top - 1 && box.bottom <= sheet.bottom + 1 };
      document.querySelector(".sheet .sheet-x, .sheet .close-stud")?.click();
      const { closeAllSheets } = await import("/src/ui/sheet.ts"); closeAllSheets();
      await new Promise((res) => setTimeout(res, 200));
      const g = trainingFocus("Boss focus L2");
      openPackages(app, null, undefined, g);
      await new Promise((res) => setTimeout(res, 400));
      const lit = document.querySelector(".pkg-panel .chip.pkg.pkg-focus");
      return { drill, g, lit: lit?.dataset.pkg ?? null, chooserOpen: !document.querySelector('.pkg-sec[data-kind=tactic] .pkg-choices')?.hidden };
    }, PACKAGES);
    check(r.drill.f.boss === "goblin_warlord" && r.drill.boss === "goblin_warlord" && r.drill.open && r.drill.seen, `the Warlord plaque opens the sheet on his drill (${JSON.stringify(r.drill)})`);
    check(r.g.pkg === "boss_focus" && r.lit === "boss_focus" && r.chooserOpen, `a level plaque opens on its package, the chooser open when not worn (${JSON.stringify(r)})`);
    await page.close();
  }
  if (parts.includes("taps")) {
    const page = await fresh("taps");
    const r = await page.evaluate(async () => {
      const app = window.__riddle;
      const wait = (ms) => new Promise((res) => setTimeout(res, ms));
      // the quest: a board with a swap left
      const quest = { goal: "reach D10 · no return", reward: "title", progress: 0.2, done: false, swap: true };
      app.lineage = { ...app.lineage, town: { ...(app.lineage.town ?? {}), quest, quests_done: 0 } };
      app.engine.swapQuest = () => ({ ...app.lineage, town: { ...app.lineage.town, quest: { ...quest, goal: "slay the Lich", swap: false } } });
      const { openQuest } = await import("/src/ui/quest.ts");
      openQuest(app, null);
      await wait(200);
      delete document.body.dataset.toast;
      document.querySelector(".quest-swap").click();
      await wait(300);
      const q = { toast: document.body.dataset.toast ?? null, live: document.querySelector(".toast")?.textContent ?? null, lit: document.querySelector(".quest-card")?.dataset.swapped ?? null, goal: document.querySelector(".quest-goal")?.textContent };
      const { closeAllSheets } = await import("/src/ui/sheet.ts"); closeAllSheets();
      // a breed: two level-2 companions in the kennel
      const pet = (id, kind) => ({ id, kind, name: kind, level: 2, gen: 1, tags: ["bite"], hp: 10, max_hp: 10, rules: { rows: [] }, max_rows: 3 });
      app.lineage = { ...app.lineage, party: [], kennel: [pet(1, "jackal"), pet(2, "hound")], eggs: [] };
      app.engine.breed = () => ({ ...app.lineage, eggs: [{ id: 77, kind: "jackal", tags: ["bite"], gen: 2, hatch_in: 3 }] });
      const { renderParty } = await import("/src/ui/party.ts");
      const party = renderParty(app);
      document.body.appendChild(party.el);
      party.el.querySelector(".row-label button, .label button").click();
      const mode = document.body.dataset.toast;
      party.el.querySelectorAll(".comp-main")[0].click();
      const one = document.body.dataset.toast;
      party.el.querySelectorAll(".comp-main")[1].click();
      await wait(400);
      return { q, mode, one, bred: document.body.dataset.toast, egg: !!party.el.querySelector('.chip.egg[data-egg="77"]') };
    });
    check(r.q.toast === "new quest" && r.q.lit === "1" && r.q.goal === "slay the Lich", `the quest swap shows its result (${JSON.stringify(r.q)})`);
    check(r.mode === "pick two" && r.one === "pick a mate" && r.bred === "egg laid" && r.egg, `a breed shows each step and its egg (${JSON.stringify(r)})`);
    await page.close();
  }
  if (parts.includes("checkin")) {
    const page = await fresh("checkin");
    const r = await page.evaluate(async () => {
      const app = window.__riddle;
      const { routineItems, checkinBatch } = await import("/src/ui/checkin.ts");
      const step = (label, price) => ({ label, price, affordable: true });
      const kit = [
        { slot: "weapon", owned: 1, steps: [], next: step("sword +2", 300) },
        { slot: "armour", owned: 1, steps: [], next: step("mail +1", 200), branches: [{ id: "plate", label: "plate" }, { id: "pace", label: "pace" }] },
        { slot: "pack", owned: 0, steps: [], next: step("satchel", 150) },
        { slot: "gun_sidearm", owned: 0, steps: [], next: step("pistol", 100) },
      ];
      const tree = { nodes: [{ id: "porter", kind: "worker", branch: "trunk", name: "porter", state: "lit", price: 90, affordable: true }, { id: "cook", kind: "worker", branch: "trunk", name: "cook", state: "shut" }], chest: 0, waits: false, sent: true, auto_send: true, ledger: 0 };
      app.lineage = { ...app.lineage, kit, tree, gold: 5000, live: undefined };
      const items = routineItems(app.lineage).map((x) => `${x.kind}:${x.id}`);
      const calls = [];
      // each buy settles its step: the next read no longer offers it
      app.engine.buyKit = (slot) => { calls.push(slot); return { ...app.lineage, kit: app.lineage.kit.map((k) => (k.slot === slot ? { ...k, next: undefined } : k)) }; };
      app.engine.hire = (id) => { calls.push(id); return { ...app.lineage, tree: { ...app.lineage.tree, nodes: app.lineage.tree.nodes.map((n) => (n.id === id ? { ...n, state: "done" } : n)) } }; };
      const b = checkinBatch(app);
      document.body.appendChild(b.el);
      const chip = b.el.querySelector(".checkin-take");
      const label = chip?.textContent;
      let taps = 0;
      chip.click(); taps++;
      const armed = b.el.querySelector(".checkin-take")?.textContent;
      const none = calls.length;
      b.el.querySelector(".checkin-take").click(); taps++;
      await new Promise((res) => setTimeout(res, 800));
      return { items, label, armed, none, calls, taps, toast: document.body.dataset.toast, hidden: b.el.hidden };
    });
    check(r.items.join() === "kit:weapon,kit:pack,hire:porter", `routine is the unbranched steps and the lit hire, never a branch or the backup (${r.items})`);
    check(r.label === "take 3" && r.armed === "ok $540" && r.none === 0, `one chip, armed before it buys (${r.label} → ${r.armed}, ${r.none} bought)`);
    check(r.calls.join() === "weapon,pack,porter" && r.taps === 2 && r.toast === "3 taken" && r.hidden, `two taps take the three routine buys (${r.calls} · ${r.taps} taps · ${r.toast})`);
    await page.close();
  }
  if (parts.includes("gold")) {
    const page = await fresh("gold");
    const r = await page.evaluate(async () => {
      const a = window.__riddle;
      const gold = { home: 0, salvage: 0, wake: 39, spent: 78, lost: 2620, net: -6789 };
      const rep = { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "mirror_king", depth: 34, n: 1 }], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 34, gold,
        workers: [{ id: "apprentice", n: 2, what: "+2 steps", spent: 6750 }] };
      a.go({ kind: "report", report: rep, absence: true });
      await new Promise((res) => setTimeout(res, 600));
      const led = document.querySelector(".report-ledger");
      return { text: led?.textContent ?? null, e: +led?.dataset.earned, s: +led?.dataset.spent, n: +led?.dataset.net, head: document.querySelector(".report-gold b")?.textContent, lost: document.querySelector(".report-lost")?.textContent ?? null };
    });
    check(r.e - r.s === r.n && r.n === -6789 && r.head === `$${r.e}`, `earned − spent = purse, the head the earned term (${r.text} · head ${r.head})`);
    check(/earned \$39 − spent \$6828 \(forge \$6750\) = purse −\$6789/.test(r.text ?? ""), `an old save derives the terms, the forge inside the spent (${r.text})`);
    // the core's ledger (`gold.ledger`): its sums read as sent, each signed term named in the tip, inflows first
    const c = await page.evaluate(async () => {
      const a = window.__riddle;
      const terms = [{ label: "carried", amount: 2620 }, { label: "heir", amount: 39 }, { label: "lost", amount: -2620 }, { label: "apprentice", amount: -6750 }, { label: "supplies", amount: -78 }];
      const ledger = { earned: 2659, spent: 9448, net: -6789, terms };
      const gold = { home: 0, salvage: 0, wake: 39, spent: 78, lost: 2620, net: -6789, ledger };
      const rep = { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "mirror_king", depth: 34, n: 1 }], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 34, gold,
        workers: [{ id: "apprentice", n: 2, what: "+2 steps", spent: 6750 }] };
      a.go({ kind: "report", report: rep, absence: true });
      await new Promise((res) => setTimeout(res, 600));
      const led = document.querySelector(".report-ledger");
      return { text: led?.textContent ?? null, e: +led?.dataset.earned, s: +led?.dataset.spent, n: +led?.dataset.net, terms: +led?.dataset.terms, head: document.querySelector(".report-gold b")?.textContent };
    });
    await page.hover(".report-gold");
    await page.waitForFunction(() => document.querySelectorAll("#kw-tip .ledger-term").length > 0, null, { timeout: 5000 }).catch(() => {});
    c.tip = await page.evaluate(() => [...document.querySelectorAll("#kw-tip .ledger-term")].map((x) => x.textContent));
    check(c.e === 2659 && c.s === 9448 && c.n === -6789 && c.e - c.s === c.n && c.head === "$2659" && c.terms === 5, `the core's ledger reads as sent (${c.text} · head ${c.head})`);
    check(/earned \$2659 − spent \$9448 \(forge \$6750\) = purse −\$6789/.test(c.text ?? ""), `the forge part from the apprentice term (${c.text})`);
    check(c.tip.join("|") === "carried · +$2620|heir · +$39|lost · −$2620|apprentice · −$6750|supplies · −$78", `each term named in the tip, inflows first (${c.tip.join(" | ")})`);
    await page.close();
  }
  if (parts.includes("wire")) {
    const page = await fresh("wire");
    const r = await page.evaluate(async () => {
      const { momentHp } = await import("/src/ui/death.ts");
      const { priceOf } = await import("/src/ui/packages.ts");
      const { holdLine, endsOf } = await import("/src/ui/forecast.ts");
      const { supplyReason, supplyLimit } = await import("/src/ui/report-supplies.ts");
      const { workersBlock } = await import("/src/ui/works.ts");
      const app = window.__riddle;
      const blow = { hp: 0, dmg: 18 };
      const hp = { core: momentHp({ trace: { blows: [blow] }, moment_hp: 5, moment_max_hp: 50 }), old: momentHp({ trace: { blows: [blow] } }) };
      const base = { d_past: 0.6, d_death: 0, d_bank: 0, past: 0.7, death: 0.2, bank: 0.3 };
      const price = { even: priceOf({ ...base, n: 8, better: 6, worse: 0, even: true }).text, wide: priceOf({ ...base, noise: 0.7 }, 400).text, tight: priceOf({ ...base, noise: 0.01 }, 400).text };
      const ends = { bank: 0.4, return: 0.3, death: 0.3, gold: 100 };
      const held = { ...ends, bank: 0.85, death: 0.1, return: 0.05 };
      const f = { depths: [{ depth: 33, reach: 0.88 }], causes: [], known_to: 33, ends, sims: 100, hold: { depth: 33, stop: 32, order: "bank", share: 0.88, ends: held } };
      const line = holdLine(f);
      const carry = holdLine({ ...f, hold: { ...f.hold, order: "carry" } })?.textContent;
      const sb = { gold: { home: 0, salvage: 0, wake: 0, spent: 0 }, supply_budget: { income: 0, spent: 0, left: 0, reason: "no_income" } };
      const lim = supplyLimit(app, sb, true);
      const L = app.lineage;
      const w = workersBlock(L, { workers: [{ id: "apprentice", n: 1, what: "+1 step", first: false, reason: "purse_short" }] });
      return { hp, price, hold: line?.textContent, holdTitle: line?.title, carry, none: holdLine({ ...f, hold: undefined }), ends: endsOf(f).bank, old: endsOf({ ...f, hold: undefined }).bank,
        reasons: ["no_income", "income_spent", "purse_short", "x"].map((c) => supplyReason(c)?.text ?? null), lim: lim.textContent, limReason: lim.dataset.reason,
        worker: w?.querySelector(".work-reason")?.textContent, workerTip: w?.querySelector(".work-reason")?.title };
    });
    check(r.hp.core === 5 && r.hp.old === 18, `the header hp is the core's moment, an old save the blow's (${JSON.stringify(r.hp)})`);
    check(r.price.even === "same" && r.price.wide === "—" && r.price.tight !== "—", `an even move reads same; the core's noise is the band (${JSON.stringify(r.price)})`);
    check(r.hold === "reach D33 88% · banks D32" && /banks at D32/.test(r.holdTitle) && r.carry === "reach D33 88% · secures D32" && r.none === null, `the forecast names the scout's order (${r.hold} · ${r.carry})`);
    check(r.ends === 0.85 && r.old === 0.4, `the ends are what the send will do (${r.ends}, ${r.old})`);
    check(r.reasons.join() === "no income,income spent,purse short," && /Supplies limited · \$0 budget · no income$/.test(r.lim), `the $0 budget says why (${r.lim} · ${r.reasons})`);
    check(r.limReason === "no_income" && r.worker === " · purse short" && /Purse could not pay/.test(r.workerTip ?? ""), `the apprentice's line says why, its gloss on hover (${r.worker} · ${r.workerTip})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut117: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut117: ok (${out.length} checks)`);
