// Blind 5331f40 item 4 — numbers that disagree: (i) the vs line's `to` is the bar's own share (A: tactics head `floor 29 <13%` beside
// `vs last run · D29 0→38%`); (ii) a pending `fired in 0 of N runs` line is dropped for a row the same report's meter saw act (A:
// `attack nearest · fired in 0 of 1 runs` beside RULES `attack nearest 23%`), the meter's % titled as activations; (iii) a purse fall
// reads beside the `$` with what spent it (B: `$19193 → $14139` with no word).
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim(), b = await launchBrowser();
let checks = 0; const check = (ok, label) => { if (!ok) throw Error(`FAIL ${label}`); checks++; };
try {
  const p = await b.newPage({ viewport: { width: 400, height: 900 } }), errors = []; p.on("pageerror", (e) => errors.push(e.message));
  await p.goto(`${url}?engine=fake&fresh=1&runs=0&seed=5331`); await p.waitForFunction(() => window.__riddle?.booted);

  // (i) the pin: a move paired before a package swap reads against the share painted now
  const pin = await p.evaluate(async () => {
    const { pinnedVs, moveOf } = await import("/src/ui/forecast.ts");
    const vs = { depths: [{ depth: 29, base: 0, delta: 0.38, pm: 0.1 }, { depth: 28, base: 0.75, delta: 0, pm: 0.1 }], death: { base: 0.88, delta: -0.38, pm: 0.1 }, sims: 8 };
    const f = { depths: [{ depth: 28, reach: 0.75 }, { depth: 29, reach: 0 }], ends: { bank: 0, return: 0.1, death: 0.9, gold: 0 }, sims: 8 };
    const out = pinnedVs(vs, f);
    const to = (m) => Math.round((m.base + m.delta) * 100);
    return { d29: to(out.depths[0]), d28same: out.depths[1] === vs.depths[1], death: to(out.death), text: moveOf(out.depths[0]).text, untouched: pinnedVs(vs, null) === vs };
  });
  check(pin.d29 === 0 && pin.text === "same", `the D29 term reads the bar's <13% share, not 38 (${JSON.stringify(pin)})`);
  check(pin.d28same && pin.death === 90 && pin.untouched, `a term already on the bar's share stands; an end pins too (${JSON.stringify(pin)})`);

  // (ii) the pending audit beside the meter
  await p.evaluate(() => { const a = window.__riddle; if (a.lineage.packages) a.lineage.packages.pen_open = true;
    a.rules.rows.splice(0, a.rules.rows.length, { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } }); });
  const L = await p.evaluate(() => window.__riddle.lineage);
  const meters = { seconds: 60, time_s: { fight: 30, travel: 20, chores: 5, rest: 5 }, dealt: { hero: 100, pets: 0 }, taken: { hero: 40, pets: 0 }, dps_dealt: { hero: 2, pets: 0 }, dps_taken: { hero: 1, pets: 0 },
    hps: 0, healed: [], hits_hero: 9, supplies: {}, gold: 0, gold_per_min: 0, rows: [{ row: 1, fires: 23, share: 0.23 }, { row: -2, fires: 77, share: 0.77 }] };
  const report = { elapsed_s: 1200, runs: 1, deepest: 14, sampled: false, bests: [], found: [], deaths: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], banked: 1, returned: 0,
    xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, learned: [], meters,
    pending: ["R1 fired 0 of 1 runs: hp < 30% → drink heal", "R2 fired 0 of 1 runs: foes ≥ 1 → attack nearest"],
    exits: [{ carried: 300, keep_pct: 100, kept: 300, spent: 0, spent_on: [], text: "banked $300 · $300 carried" }], gold: { home: 300, salvage: 0, wake: 0, spent: 0 } };
  await p.evaluate((r) => window.__riddle.go({ kind: "report", report: r }), report);
  await p.waitForSelector(".report"); await p.waitForTimeout(400);
  const rep = await p.evaluate(() => ({ pending: [...document.querySelectorAll(".report .rsec li")].map((l) => l.textContent), rules: [...document.querySelectorAll(".meters .mrow .mlab")].map((x) => x.textContent) }));
  check(rep.pending.some((x) => /drink heal · fired in 0 of 1 runs/.test(x)), `a row the meter never saw keeps its audit line (${rep.pending.join(" | ")})`);
  check(!rep.pending.some((x) => /attack nearest · fired in 0/.test(x)), `no \`fired in 0\` for a row the meter saw act (${rep.pending.join(" | ")})`);
  check(await p.evaluate(() => /% of recorded activations/.test(document.querySelector(".meters .mrule")?.title ?? "")), `the meter's % says what it counts (${rep.rules.join(" | ")})`);

  // (iii) the purse's fall, named from the ledger
  await p.evaluate(() => window.__riddle.go({ kind: "camp" })); await p.waitForSelector(".topbar .stat.gold");
  await p.evaluate(() => { const a = window.__riddle, L = a.lineage; L.gold_ledger = [...(L.gold_ledger ?? []), { t: 900000, delta: 19193 - L.gold, why: "returned D33" }]; L.gold = 19193; a.emitChange(); });
  await p.waitForFunction(() => /\$19193/.test(document.querySelector(".topbar .stat.gold")?.textContent ?? ""));
  check(!(await p.$(".topbar .gold-drop")), "no fall, no chip");
  await p.evaluate(() => { const a = window.__riddle, L = a.lineage;
    L.gold_ledger = [...L.gold_ledger, { t: 900100, delta: -4800, why: "forge sword +6" }, { t: 900100, delta: -254, why: "repeat heal", n: 4 }]; L.gold = 14139; a.emitChange(); });
  await p.waitForFunction(() => !!document.querySelector(".topbar .gold-drop"));
  const chip = await p.evaluate(() => ({ text: document.querySelector(".topbar .gold-drop").textContent, title: document.querySelector(".topbar .gold-drop").title }));
  check(chip.text === "−$5054 · forge" && chip.title === "forge −$4800 · restock −$254", `the fall names what spent it (${JSON.stringify(chip)})`);
  await p.evaluate((r) => window.__riddle.go({ kind: "report", report: r }), report); await p.waitForSelector(".report");
  await p.evaluate(() => window.__riddle.go({ kind: "camp" })); await p.waitForSelector(".topbar .stat.gold");
  check(await p.evaluate(() => document.querySelector(".topbar .gold-drop")?.textContent === "−$5054 · forge"), "the word survives a screen change");
  await p.evaluate(() => { const a = window.__riddle, L = a.lineage; L.gold_ledger = [...L.gold_ledger, { t: 900200, delta: 500, why: "returned D20" }]; L.gold = 14639; a.emitChange(); });
  await p.waitForFunction(() => !document.querySelector(".topbar .gold-drop"));
  checks++;
  if (await p.evaluate(() => document.documentElement.scrollWidth > innerWidth)) throw Error("overflow"); checks++;
  if (errors.length) throw Error(errors.join("\n"));
  console.log(checks, "numbers agree: vs pinned to the bar · pending vs meter · purse fall named PASS");
} finally { await b.close(); }
