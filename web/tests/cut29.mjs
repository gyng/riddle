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
    await sleep(2400);
    const n2 = await page.evaluate(() => (window.__riddle.lineage.systems ?? []).filter((s) => s.new).map((s) => s.id));
    check(n2.length === 0, `once shown the camp clears it (seenSystems): new ${JSON.stringify(n2)}`);
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
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(e);
const bad = failed + errors.filter((e) => !/favicon|Failed to load resource/.test(e)).length;
console.log(bad ? `cut29: FAIL (${failed} assertion(s), ${errors.length} error(s))` : `cut29: ok (${out.length} checks)`);
process.exit(bad ? 1 : 0);
