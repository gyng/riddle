#!/usr/bin/env node
// Cut 29 client gates (docs/CUT29.md), on the fake engine (its Cut 29 stand-ins), headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   merge   `mergeReports` carries the Cut 29 fields — night marks summed, systems opened unioned in order, kept oaths and the fallen
//           concatenated, the meters merged (totals add; rates recomputed from the summed seconds)
//   §4      the keep sheet only when it is a decision (`ExitPending.decide`); a settled exit says its one line (`kept leather +1`)
//   §2      the systems open one at a time: a fresh lineage's camp shows no tile, compact tablets, no vs line; each trigger opens its
//           system (the edit tile at the first death, glinting; the order's ▲▼ at the first plateau; the forge at the Warlord slain); the
//           report plaques the systems an absence opened; the camp clears the core's `new` once shown (`seenSystems`)
//
//   node web/tests/cut29.mjs [--shots dir] [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: shots ? 2 : 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: false }); };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp", "camp"); await sleep(300); };
/** The fake's whole state edited through a save round-trip (`e.lineage`, `e.sys29` the curriculum), then the camp again. */
const withState = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const save = JSON.parse(await r.engine.save());
  new Function("e", src)(save);
  r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
}, `(${fn})(e)`);
const tiles = () => page.evaluate(() => [...document.querySelectorAll(".cmd .tile:not(.empty)")].map((t) => t.dataset.tile + (t.classList.contains("reveal") ? "*" : "")));
const boot = async (seed) => {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}`, { waitUntil: "domcontentloaded" });
  await camp();
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); });
};

try {
  // ---- merge: the offline report's slices carry the Cut 29 fields
  if (part("merge")) {
    await boot(2900);
    const m = await page.evaluate(async () => {
      const { mergeReports } = await import("/src/app.ts");
      const meter = (s, dealt, heal, rows, gold) => ({ seconds: s, dealt: { hero: dealt, pets: 0, foes: 5 }, taken: { hero: 4, pets: 0, foes: dealt }, dps_dealt: { hero: dealt / s, pets: 0, foes: 5 / s }, dps_taken: { hero: 4 / s, pets: 0, foes: dealt / s },
        healed: [{ src: "potion", total: heal, per_s: heal / s }], hps: heal / s, time: { fight: 10, travel: 20, chores: 0, rest: 5 }, time_s: { fight: 1, travel: 2, chores: 0, rest: 0.5 },
        rows: rows.map(([row, fires]) => ({ row, fires, share: 0 })), actions: rows.reduce((a, [, f]) => a + f, 0), supplies: { heal: 1 }, gold, gold_per_min: gold / (s / 60), hits_hero: 2, hits_pets: 0, fights: 1 });
      const base = (o) => ({ elapsed_s: 100, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, live: false, ...o });
      const a = base({ night_marks: 1, systems_opened: ["edit", "death"], oaths_kept: [{ kind: "card", id: "x", label: "slay Warlord" }], fallen: [{ name: "Greth", kind: "ogre", level: 5, depth: 12, why: "fell D12 to lurker", heir: 3 }], meters: meter(10, 30, 6, [[0, 3], [1, 1]], 40) });
      const b = base({ night_marks: 0, systems_opened: ["death", "exits"], fallen: [{ name: "Ashar", kind: "jackal", level: 2, depth: 4, why: "fell D4 to rat", heir: 4 }], meters: meter(30, 60, 0, [[1, 4], [2, 2]], 20) });
      const r = mergeReports(a, b);
      return { marks: r.night_marks, sys: r.systems_opened, oaths: r.oaths_kept?.length, fallen: r.fallen?.map((f) => f.name), m: r.meters };
    });
    check(m.marks === 1 && JSON.stringify(m.sys) === JSON.stringify(["edit", "death", "exits"]) && m.oaths === 1 && JSON.stringify(m.fallen) === JSON.stringify(["Greth", "Ashar"]),
      `merge: marks ${m.marks}, systems ${m.sys?.join(" · ")}, kept oaths ${m.oaths}, fallen ${m.fallen?.join(" · ")}`);
    const mm = m.m;
    check(mm && mm.seconds === 40 && mm.dealt.hero === 90 && mm.dps_dealt.hero === 2.25 && mm.actions === 10 && JSON.stringify(mm.rows.map((x) => [x.row, x.fires, x.share])) === JSON.stringify([[0, 3, 0.3], [1, 5, 0.5], [2, 2, 0.2]])
      && mm.healed[0].total === 6 && mm.hps === 0.15 && mm.gold === 60 && mm.gold_per_min === 90 && mm.supplies.heal === 2 && mm.fights === 2,
      `merge: the meters add up and their rates are the summed totals' (${mm && `${mm.seconds} s · dealt ${mm.dealt.hero} · ${mm.dps_dealt.hero} dps · rows ${mm.rows.map((x) => `${x.row}:${x.fires}/${x.share}`).join(" ")} · ${mm.hps} hp/s · $${mm.gold_per_min}/min`})`);
  }

  // ---- §4: the keep sheet only when it is a decision
  if (part("keep")) {
    await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=2904&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.booted && window.__riddle.screen === "watch", "the watch");
    await page.evaluate(() => {
      const r = window.__riddle, st = r.engine.step.bind(r.engine), ak = r.engine.autoKeep?.bind(r.engine);
      r.__ak = 0; r.__banners = [];
      r.engine.autoKeep = async () => { r.__ak++; return ak(); };
      r.engine.step = async (n) => { const res = await st(n); if (res.exit_pending) { res.exit_pending.decide = false; res.exit_pending.note = "kept leather +1"; } return res; };
      new MutationObserver(() => { for (const b of document.querySelectorAll(".keep-note")) if (!r.__banners.includes(b.textContent)) r.__banners.push(b.textContent); }).observe(document.body, { subtree: true, childList: true, characterData: true });
    });
    let sheet = false; const t0 = Date.now();
    while (Date.now() - t0 < 90_000) {
      const s = await page.evaluate(() => ({ screen: window.__riddle.screen, keep: !!document.querySelector(".sheet-wrap .keep-legend") }));
      if (s.keep) { sheet = true; break; }
      if (await page.evaluate(() => window.__riddle.__ak > 0 && window.__riddle.__banners.length > 0)) break;
      // a run that died or brought nothing home: send again
      if (s.screen !== "watch") { await page.evaluate(() => window.__riddle.go({ kind: "watch" })); await sleep(400); continue; }
      await page.evaluate(() => { for (const b of document.querySelectorAll(".sheet-wrap .vault-choice .chip")) { b.click(); break; } for (const b of document.querySelectorAll(".cmd .tile[data-tile=skip]")) b.click(); });
      await sleep(200);
    }
    const k = await page.evaluate(() => ({ ak: window.__riddle.__ak, banners: window.__riddle.__banners, screen: window.__riddle.screen }));
    check(!sheet && k.ak >= 1, `a find that is no decision opens no keep sheet: settled by autoKeep (×${k.ak}, sheet ${sheet}, ${k.screen})`);
    check(k.banners.includes("kept leather +1"), `the settled exit says its one line (${JSON.stringify(k.banners)})`);
  }

  // ---- §2: the systems open one at a time
  if (part("systems")) {
    await boot(2901);
    await withState((e) => {
      delete e.sys29;
      Object.assign(e.lineage, { heir: 1, chronicle: [], unlocks: ["tame"], marks: 0, gold: 0, gold_ledger: [], graveyard: [], vault: [], forge: {}, party: [], kennel: [], eggs: [], rank: 0, renown: 0, best_depth: 0, trophies: [], facts: [],
        supplies: [{ id: 100000, kind: "leash", known: true, label: "leash", free: true }] });
    });
    await camp();
    const d0 = await page.evaluate(() => ({ sys: (window.__riddle.lineage.systems ?? []).filter((s) => s.open).map((s) => s.id), tiles: [...document.querySelectorAll(".cmd .tile:not(.empty)")].length,
      compact: !!document.querySelector(".editor.compact"), vs: !!document.querySelector(".shaft-vs"), route: !!document.querySelector(".route-line:not([hidden])") }));
    check(d0.sys.join() === "send,dial,headline" && d0.tiles === 0 && d0.compact && !d0.vs && !d0.route, `day 0: only ${d0.sys.join(" · ")} open — ${d0.tiles} tiles, compact tablets ${d0.compact}, no vs line (${d0.vs})`);
    await shot("cut29-day0");
    // the first death opens the edit (and the verdicts): the tile glints on its arrival
    await withState((e) => { e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 2, cause: "rat", deeds: [] }]; });
    await camp();
    const t1 = await tiles();
    check(t1.includes("edit*"), `the first death opens the edit tile, glinting (${t1.join(" · ")})`);
    const n1 = await page.evaluate(() => (window.__riddle.lineage.systems ?? []).filter((s) => s.new).map((s) => s.id));
    check(n1.includes("edit"), `the core marks it new until the camp has shown it (${n1.join(" · ")})`);
    await sleep(2000);
    await page.evaluate(async () => { const r = window.__riddle; r.go({ kind: "watch" }); });   // the camp left for a send after the glint
    await until(() => window.__riddle.screen === "watch", "the watch");
    const n2 = await until(() => { const n = (window.__riddle.lineage.systems ?? []).filter((s) => s.new).map((s) => s.id); return n.length ? null : n; }, "the new marks cleared", 5000).catch(() => ["still new"]);
    check(n2.length === 0, `once shown, the next send clears it (seenSystems): new ${JSON.stringify(n2)}`);
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
    // editing before the first plateau: no order yet (no ▲▼, the grip does not drag); the exits' verbs and the foe tags not offered
    await page.locator(".cmd .tile[data-tile=edit]").click();
    await until(() => !document.querySelector(".editor.compact") && document.querySelector(".editor .row.tablet"), "the editor");
    const e1 = await page.evaluate(() => ({ updown: document.querySelectorAll(".editor .updown").length, still: document.querySelectorAll(".editor .grip.still").length }));
    check(e1.updown === 0 && e1.still > 0, `before the first plateau the tablets carry no ▲▼ (${e1.updown}) and a still grip (${e1.still})`);
    await page.locator(".editor .row.tablet .chip.verb").first().click();
    const verbs = await until(() => { const s = [...document.querySelectorAll(".sheet-wrap .chip.verb")].map((c) => c.textContent.trim()); return s.length ? s : null; }, "the action sheet");
    check(!verbs.some((v) => /^(bank|rest)\b/.test(v)), `the action sheet offers no exit verb before the first gold home (${verbs.join(" · ")})`);
    await page.keyboard.press("Escape"); await sleep(200);
    // the first plateau opens the order and the vs line
    await withState((e) => { e.sys29.plateau = true; });
    await camp();
    await page.evaluate(() => { window.__riddle.editing = true; window.__riddle.go({ kind: "camp" }); }); await camp();
    const e2 = await page.evaluate(() => document.querySelectorAll(".editor .updown").length);
    check(e2 > 0, `the first plateau opens the order: ▲▼ on the tablets (${e2})`);
    // the Warlord slain opens the forge
    await withState((e) => { e.lineage.best_depth = 9; e.lineage.gold = 400; e.lineage.gold_ledger = [{ t: 1, delta: 400, why: "bank D9" }]; });
    await camp();
    const t3 = await tiles();
    check(t3.some((t) => t.startsWith("forge")) && t3.some((t) => t.startsWith("loadout")), `the Warlord slain opens the forge, the first gold home the loadout (${t3.join(" · ")})`);
    await shot("cut29-forge-open");
    // the report plaques what an absence opened
    await page.evaluate(() => { const r = window.__riddle; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 4, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 1, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 10, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, live: false, banked: 2, returned: 1, stalled: 0, driven: 0, systems_opened: ["send", "forge", "oaths"] } }); });
    await until(() => window.__riddle.screen === "report", "the report");
    const pl = await until(() => { const p = [...document.querySelectorAll(".sys-opened .sys-plaque")].map((x) => x.dataset.sys + ":" + x.textContent.trim() + (x.classList.contains("reveal") ? "*" : "")); return p.length ? p : null; }, "the opened plaques");
    check(pl.join() === "forge:forge*,oaths:oaths*", `the report plaques the systems opened, glinting (${pl.join(" · ")})`);
    await shot("cut29-report-opened");
  }
  // ---- §3: the meters — the watch's toggle, the death's fight, the report's night, the camp's two runs; units always
  if (part("meters")) {
    await boot(2903);
    await page.evaluate(() => { localStorage.removeItem("riddle.meters"); window.__riddle.go({ kind: "watch" }); });
    await until(() => window.__riddle.screen === "watch" && document.querySelector(".cmd .tile[data-tile=meters]"), "the watch's meters tile");
    const off = await page.evaluate(() => document.querySelector(".meter-box")?.hidden);
    await page.locator(".cmd .tile[data-tile=meters]").click();
    const line = await until(() => { const b = document.querySelector(".meter-box"); return b && !b.hidden && b.textContent.trim() ? b.textContent.replace(/\s+/g, " ").trim() : null; }, "the compact meter", 20_000);
    check(off === true && /^(fight|run) dealt [\d.]+ dps.* taken [\d.]+ dps/.test(line), `the watch's meter is off until toggled, then one line with its units ("${line}")`);
    const kept = await page.evaluate(() => localStorage.getItem("riddle.meters"));
    check(kept === "1", `the toggle is remembered (${kept})`);
    await shot("cut29-watch-meter");
    const fightM = { seconds: 8.4, dealt: { hero: 31, pets: 0, foes: 44 }, taken: { hero: 38, pets: 0, foes: 31 }, dps_dealt: { hero: 3.7, pets: 0, foes: 5.2 }, dps_taken: { hero: 4.5, pets: 0, foes: 3.7 },
      healed: [{ src: "potion", total: 12, per_s: 1.4 }], hps: 1.4, time: { fight: 84, travel: 0, chores: 0, rest: 0 }, time_s: { fight: 8.4, travel: 0, chores: 0, rest: 0 },
      rows: [{ row: 1, fires: 7, share: 0.7 }, { row: 0, fires: 3, share: 0.3 }], actions: 10, supplies: { heal: 1 }, gold: 0, gold_per_min: 0, hits_hero: 9, hits_pets: 0, fights: 1 };
    await page.evaluate((m) => { const r = window.__riddle; r.go({ kind: "death", death: { run_id: 0, depth: 7, cause: "ogre", margin: "", verdict: "gap", baseline: 0.25, replays: 12, trace: { turns: [] }, patches: [], morgue: "t1", fight: m, rules: r.rules } }); }, fightM);
    const dm = await until(() => { const x = document.querySelector(".fight-meters"); return x ? [...x.querySelectorAll(".mrow")].map((r) => r.textContent.replace(/\s+/g, " ").trim()) : null; }, "the death's fight meter");
    const rules = await page.evaluate(() => window.__riddle.rules.rows.length);
    check(dm.some((x) => /^dealt 3\.7 dps/.test(x)) && dm.some((x) => /^taken 4\.5 dps .*9 hits/.test(x)) && dm.some((x) => /^healed 1\.4 hp\/s .*potion 12 hp/.test(x)) && dm.some((x) => /^rules .*70%/.test(x) && !/\bR\d/.test(x)),
      `the death screen breaks down the fight, units on every figure, rules by their words (${dm.join(" | ")}; ${rules} rules)`);
    await shot("cut29-death-fight");
    await page.evaluate((m) => { const r = window.__riddle; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 6, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, live: false, banked: 3, returned: 3, meters: { ...m, seconds: 900, time_s: { fight: 120, travel: 600, chores: 80, rest: 100 }, gold: 300, gold_per_min: 20 } } }); }, fightM);
    await until(() => window.__riddle.screen === "report", "the report");
    await page.locator(".details-fold").click();
    const rm = await until(() => { const x = document.querySelector(".report-details .meters"); return x ? x.textContent.replace(/\s+/g, " ").trim() : null; }, "the night's meter");
    check(/^this night · 15m/.test(rm) && /gold \$20\/min/.test(rm) && /fight 13%/.test(rm) && /travel 67%/.test(rm), `the report meters the night ("${rm.slice(0, 160)}")`);
    // the camp: the last two runs side by side (the forecast panel on a phone)
    await withState((e) => { e.sys29 ??= { open: ["send", "dial", "headline"], fresh: [], plateau: false, works: [], meters: [], insure: true };
      const m = (dps, g) => ({ seconds: 60, dealt: { hero: dps * 60, pets: 0, foes: 10 }, taken: { hero: 30, pets: 0, foes: 10 }, dps_dealt: { hero: dps, pets: 0, foes: 0.2 }, dps_taken: { hero: 0.5, pets: 0, foes: 0.2 }, healed: [], hps: 0,
        time: { fight: 100, travel: 500, chores: 0, rest: 0 }, time_s: { fight: 10, travel: 50, chores: 0, rest: 0 }, rows: [], actions: 0, supplies: {}, gold: g, gold_per_min: g, hits_hero: 3, hits_pets: 0, fights: 2 });
      e.sys29.meters = [m(4, 20), m(6, 35)]; });
    await camp();
    await page.locator(".shaft").first().click();
    const cmp = await until(() => { const x = document.querySelector(".forecast .mcmp"); return x ? [...x.querySelectorAll(".mcmp-row")].map((r) => r.textContent.replace(/\s+/g, " ").trim()) : null; }, "the two-run comparison");
    check(cmp.some((x) => /^dealt 4 → 6 dps \+2$/.test(x)) && cmp.some((x) => /^gold 20 → 35 \$\/min \+15$/.test(x)), `the camp compares the last two runs (${cmp.join(" | ")})`);
    await shot("cut29-camp-compare");
  }
  // ---- §4: the standing orders in one panel (setOrders)
  if (part("orders")) {
    await boot(2905);
    await withState((e) => {
      e.sys29 = { open: ["send", "dial", "headline", "edit", "death", "exits", "loadout", "unlocks", "cage", "forge", "start"], fresh: [], plateau: false, works: [], meters: [], insure: true };
      Object.assign(e.lineage, { heir: 3, gold: 300, gold_ledger: [{ t: 1, delta: 300, why: "bank D9" }], facts: [...(e.lineage.facts ?? []), "vault"], waystones: [5], best_depth: 9 });
    });
    await camp();
    const tabs = await page.evaluate(() => ({ orders: document.querySelector(".orders-tab:not([hidden])")?.textContent.replace(/\s+/g, " ").trim() ?? null, cage: !!document.querySelector(".cage-tab:not([hidden])"), start: !!document.querySelector(".start-tab:not([hidden])") }));
    check(tabs.orders && /orders keep weapon · cages → weapon/.test(tabs.orders) && !tabs.cage && !tabs.start, `one orders tablet stands for the keep, the cage and the start ("${tabs.orders}"; cage tablet ${tabs.cage}, start tablet ${tabs.start})`);
    await page.locator(".orders-tab").click();
    const rows = await until(() => { const r = [...document.querySelectorAll(".sheet-wrap .order-row")].map((x) => x.textContent.replace(/\s+/g, " ").trim()); return r.length ? r : null; }, "the orders sheet");
    check(rows.length === 5 && /^keep for heirs/.test(rows[0]) && rows.some((r) => /^from cages/.test(r)) && rows.some((r) => /^start D1/.test(r)) && rows.some((r) => /^repeat pack on off/.test(r)) && rows.some((r) => /^insure kit on off/.test(r)),
      `the sheet holds the five orders (${rows.join(" | ")})`);
    await shot("cut29-orders");
    await page.locator(".sheet-wrap .order-row").first().locator(".chip", { hasText: "armour" }).click();
    await until(() => window.__riddle.lineage.orders?.keep === "best_armour", "the keep order set");
    await page.locator(".sheet-wrap .order-row", { hasText: "repeat" }).locator(".chip", { hasText: "off" }).click();
    await until(() => window.__riddle.lineage.orders?.repeat === false, "the repeat order set");
    const after = await page.evaluate(() => ({ on: [...document.querySelectorAll(".sheet-wrap .chip.order.on")].map((c) => c.textContent.trim()), tab: document.querySelector(".orders-tab")?.textContent.replace(/\s+/g, " ").trim() }));
    check(after.on.includes("armour") && after.on.includes("off") && /keep armour/.test(after.tab) && /no repeat/.test(after.tab), `a tap sets an order through setOrders; the sheet and the tablet follow (${after.on.join(" · ")}; "${after.tab}")`);
  }
  // ---- §1/§5: the oath board's slots, the draw, the works
  if (part("oaths")) {
    await boot(2906);
    await withState((e) => {
      e.sys29 = { open: ["send", "dial", "headline", "edit", "death", "exits", "loadout", "unlocks", "oaths", "walls"], fresh: [], plateau: true, works: [], meters: [], insure: true };
      Object.assign(e.lineage, { heir: 3, gold: 5000, best_depth: 9, marks: 5, unlocks: [...e.lineage.unlocks, "oath_slot_2"], gold_ledger: [{ t: 1, delta: 5000, why: "bank D9" }] });
    });
    await camp();
    await page.locator(".oath-tab").click();
    await until(() => document.querySelectorAll(".sheet-wrap .oath.tablet").length === 3, "the oath board");
    const swear = page.locator(".sheet-wrap .oath.tablet .chip.swear").first();
    await swear.click(); await sleep(120); await page.locator(".sheet-wrap .oath.tablet .chip.swear").first().click();
    await until(() => (window.__riddle.lineage.sworn ?? []).length === 1, "one oath sworn");
    const b1 = await page.evaluate(() => ({ slots: document.querySelector(".sheet-wrap .oath-slots")?.textContent.replace(/\s+/g, " ").trim(), open: [...document.querySelectorAll(".sheet-wrap .oath.tablet:not(.sworn) .chip.swear")].filter((b) => !b.disabled).length,
      draw: document.querySelector(".sheet-wrap .chip.draw")?.textContent.trim(), works: document.querySelector(".sheet-wrap .chip.commission")?.textContent.trim() }));
    check(b1.slots === "sworn 1/2" && b1.open === 2, `with a second slot one sworn oath leaves the others swearable (${b1.slots}; ${b1.open} open)`);
    await shot("cut29-oaths");
    const m0 = await page.evaluate(() => window.__riddle.lineage.marks);
    await page.locator(".sheet-wrap .chip.draw").click(); await sleep(120); await page.locator(".sheet-wrap .chip.draw").click();
    await until((m) => window.__riddle.lineage.marks === m - 2, "the draw paid", 10_000, m0);
    check(/^draw ◆2$/.test(b1.draw ?? ""), `a fresh oath is drawn for ◆2 ("${b1.draw}", marks ${m0} → ${m0 - 2})`);
    const g0 = await page.evaluate(() => window.__riddle.lineage.gold);
    await page.locator(".sheet-wrap .chip.commission").click(); await sleep(120); await page.locator(".sheet-wrap .chip.commission").click();
    const built = await until(() => (window.__riddle.lineage.works ?? []).length ? window.__riddle.lineage.works : null, "the work built");
    const g1 = await page.evaluate(() => window.__riddle.lineage.gold);
    check(/^build .+ \$\d+$/.test(b1.works ?? "") && g1 < g0 && built.length === 1, `a work is commissioned for gold ("${b1.works}": $${g0} → $${g1}, works ${built.join(" · ")})`);
  }
  // ---- §4: the repeat's missing kind is one tap
  if (part("repeat")) {
    await boot(2907);
    await withState((e) => {
      e.sys29 = { open: ["send", "dial", "headline", "edit", "death", "exits", "loadout"], fresh: [], plateau: false, works: [], meters: [], insure: true };
      Object.assign(e.lineage, { heir: 2, gold: 500, gold_ledger: [{ t: 1, delta: 500, why: "bank D3" }], supplies: [], facts: [...e.lineage.facts, "item:ruby=fire"] });
      e.rules.rows.push({ conds: [{ k: "foes>=", n: 2 }], verb: { v: "throw", a: "fire" }, origin: "player" });
    });
    await camp();
    const chip = await until(() => document.querySelector(".repeat-add:not([hidden]) .repeat-add-chip")?.textContent.replace(/\s+/g, " ").trim() ?? null, "the repeat's one tap");
    check(chip === "+ fire · for throw fire", `a throw row's kind the repeat lacks is offered as one tap ("${chip}")`);
    await shot("cut29-repeat-add");
    await page.locator(".repeat-add-chip").click();
    await until(() => (window.__riddle.lineage.supplies ?? []).some((s) => s.kind === "fire"), "fire packed");
    const gone = await page.evaluate(() => !document.querySelector(".repeat-add:not([hidden]) .repeat-add-chip"));
    check(gone, `one tap packs it (buySupply) and the offer leaves (${gone})`);
  }
  // ---- §6: the passage apart from a run's gold; the fallen by name
  if (part("lines")) {
    await boot(2908);
    await withState((e) => { e.sys29 = { open: ["send", "dial", "headline", "edit", "death", "exits", "loadout", "walls"], fresh: [], plateau: false, works: [], meters: [], insure: true }; Object.assign(e.lineage, { best_depth: 9, heir: 3 }); });
    await camp();
    await until(() => window.__riddle.lastForecast?.ends, "the forecast's ends");
    await page.evaluate(() => { const r = window.__riddle;
      for (const m of ["forecast", "forecastRefine"]) { const o = r.engine[m]?.bind(r.engine); if (o) r.engine[m] = async (...a) => { const f = await o(...a); return f?.ends ? { ...f, ends: { ...f.ends, gold: 260, passage: 135 } } : f; }; }
      r.rulesChanged(); r.go({ kind: "camp" }); });
    await camp();
    const g = await until(() => { const x = document.querySelector(".shaft .end.gold"); return x && /passage$/.test(x.textContent.trim()) ? x.textContent.replace(/\s+/g, " ").trim() : null; }, "the passage on the shaft", 8000).catch(() => null);
    check(g === "~$125/run +$135 passage", `the shaft's gold is a run's, the passage apart ("${g}")`);
    await page.evaluate(() => { window.__riddle.go({ kind: "report", report: { elapsed_s: 3600, runs: 4, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, live: false, banked: 2, returned: 2,
      fallen: [{ name: "Greth", kind: "ogre", level: 5, depth: 12, why: "fell D12 to lurker", heir: 3 }] } }); });
    const f = await until(() => document.querySelector(".fallen-sec .fallen-line")?.textContent.replace(/\s+/g, " ").trim() ?? null, "the fallen line");
    check(f === "Greth · ogre L5 · fell D12 to lurker", `a fallen companion is a named line among the decisions ("${f}")`);
    await shot("cut29-fallen");
  }
  // ---- §1 (E1): the wall's edit lands as a patch after the report paints, and applies the whole measured set
  if (part("wall")) {
    await boot(2909);
    const t = await page.evaluate(async () => {
      const r = window.__riddle; const rows = r.rules.rows;
      const set = { rows: [...rows.slice(0, 1), { conds: [{ k: "hp<", n: 90 }], verb: { v: "rest" }, origin: "patch" }, ...rows.slice(1)] };
      r.engine.wallEdit = () => new Promise((res) => setTimeout(() => res({ depth: 17, edits: ["R1 → hp < 90% → rest"], rules: set, before: 0.12, after: 0.38, sims: 48 }), 700));
      const t0 = performance.now();
      r.go({ kind: "report", report: { elapsed_s: 3600, runs: 4, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, tamed: [], hatched: [], lost: [], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, live: false, banked: 2, returned: 2 } });
      return { painted: !!document.querySelector("main.report .tiles, main.report .report-sheet"), wallAtOnce: !!document.querySelector(".wall-edit"), t0 };
    });
    check(t.painted && !t.wallAtOnce, `the report paints without waiting on the wall search (painted ${t.painted}, tablet at once ${t.wallAtOnce})`);
    const w = await until(() => document.querySelector(".wall-host .wall-edit")?.textContent.replace(/\s+/g, " ").trim() ?? null, "the wall's tablet");
    check(/^wall D17 · 48 sends/.test(w) && /past D17 12%→38%/.test(w) && !/\bR\d/.test(w), `the wall's edit is a patch tablet, its rule named by its words ("${w}")`);
    await shot("cut29-wall");
    await page.locator(".wall-apply").click();
    await camp();
    const rows = await page.evaluate(() => window.__riddle.rules.rows.map((r) => `${r.conds.map((c) => c.k + (c.n ?? "")).join("&")}>${r.verb.v}`));
    check(rows[1] === "hp<90>rest", `apply takes the measured set (${rows.join(" · ")})`);
  }
  // ---- §4: a long press lifts a tablet and it moves any distance
  if (part("reorder")) {
    await boot(2910);
    await withState((e) => {
      e.sys29 = { open: ["send", "dial", "headline", "edit", "death", "reorder", "vs"], fresh: [], plateau: true, works: [], meters: [], insure: true };
      Object.assign(e.lineage, { heir: 2, graveyard: [{ heir: 1, depth: 2, cause: "rat", deeds: [] }], unlocks: [...e.lineage.unlocks, "row5"] });
      e.rules.rows.push({ conds: [{ k: "foes>=", n: 3 }], verb: { v: "retreat" }, origin: "player" }, { conds: [{ k: "hp<", n: 50 }], verb: { v: "rest" }, origin: "player" });
    });
    await camp();
    await page.evaluate(() => { window.__riddle.editing = true; window.__riddle.go({ kind: "camp" }); }); await camp();
    const before = await page.evaluate(() => window.__riddle.rules.rows.map((r) => r.verb.v));
    const last = page.locator(".editor .row.tablet").last(), first = page.locator(".editor .row.tablet").first();
    const lb = await last.locator(".chip.verb").boundingBox(), fb = await first.boundingBox();
    await page.mouse.move(lb.x + lb.width / 2, lb.y + lb.height / 2); await page.mouse.down();
    const lifted = await until(() => !!document.querySelector(".editor .row.tablet.lifted"), "the lift", 3000).catch(() => false);
    for (let k = 1; k <= 8; k++) { await page.mouse.move(lb.x + lb.width / 2, lb.y + (fb.y + 4 - lb.y) * k / 8); await sleep(30); }
    await page.mouse.up(); await sleep(400);
    const after = await page.evaluate(() => ({ rows: window.__riddle.rules.rows.map((r) => r.verb.v), sheet: !!document.querySelector(".sheet-wrap") }));
    check(lifted && after.rows[0] === before[before.length - 1] && after.rows.length === before.length && !after.sheet,
      `a long press lifts the last tablet and drops it at the top (lifted ${lifted}; ${before.join(" · ")} → ${after.rows.join(" · ")}; sheet ${after.sheet})`);
  }
  // ---- owner: a world concept's icon and its caption, once
  if (part("concepts")) {
    await boot(2911);
    await page.evaluate(() => localStorage.removeItem("riddle.concepts"));
    await page.reload({ waitUntil: "domcontentloaded" }); await camp();   // (the module keeps what it read at its first caption)
    await withState((e) => { Object.assign(e.lineage, { marks: 3, heir: 2 }); });
    await camp();
    const c1 = await page.evaluate(() => document.querySelector(".topbar .stat.marks .concept-cap")?.dataset.cap ?? null);
    check(c1 === "buys unlocks", `the marks' first appearance carries its caption ("${c1}")`);
    await shot("cut29-concept");
    await sleep(2600);
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
    const c2 = await page.evaluate(() => ({ cap: !!document.querySelector(".topbar .stat.marks .concept-cap"), ico: !!document.querySelector(".topbar .stat.marks .ico") }));
    check(!c2.cap && c2.ico, `once seen, the icon stands alone (caption ${c2.cap}, icon ${c2.ico})`);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(e);
const bad = failed + errors.filter((e) => !/favicon|Failed to load resource/.test(e)).length;
console.log(bad ? `cut29: FAIL (${failed} assertion(s), ${errors.length} error(s))` : `cut29: ok (${out.length} checks)`);
process.exit(bad ? 1 : 0);
