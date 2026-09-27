#!/usr/bin/env node
// Cut 28 §2 client gates, the death screen (docs/CUT28.md), on the fake engine, headless at 400 × 800:
//   max hp  a trace whose hero's max moved (drain, hunger) reads every hp over the max he had then (`6/29`), each step a row of its own
//           (`max 44→29 · drain`), the run's arc over the table (`max hp 44→29 · drain −15`), and an `hp not <30%` reason its hp (`· 10/29`);
//           the core's `Trace.max_steps`, per-turn `TraceTurn.max_hp`, and the watched run's own `max_hp` events (the run log) alike
//   luck    a death most of whose unpatched replays live (or a `dice`) leads with the event that killed him and its odds (`goblin −6 at 6 hp ·
//           1 in 6`) above the stamp, which steps back; a gap most replays die of has no luck line
//   no gain a patch that reads `no gain` never carries a move in the gain colour (`reach D14 0→24%` neutral), nor once its whole run lands
//
//   node web/tests/cut28d.mjs [--shots dir]        (part of `pnpm test` in web/)
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
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: false }); };
async function waitScreen(name, timeout = 20_000) {
  const t = Date.now();
  while (Date.now() - t < timeout) { const s = await page.evaluate(() => window.__riddle ? { screen: window.__riddle.screen, booted: window.__riddle.booted } : null); if (s?.booted && s.screen === name) return; await sleep(40); }
  throw new Error(`timeout waiting for ${name}`);
}
const txt = (sel) => page.evaluate((s) => document.querySelector(s)?.textContent.replace(/\s+/g, " ").trim() ?? null, sel);

const ROWS = [{ conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" }, origin: "player" }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" }];
const turn = (t, row, hp, extra = {}) => ({ t, row, verb: row === 1 ? { v: "attack", a: "nearest" } : { v: "explore" }, hp, foes: 1, telegraphs: [], ...extra });
const deathOf = (o = {}) => ({ run_id: 0, depth: 9, cause: "gas", margin: "", verdict: "gap", baseline: 2 / 12, replays: 12, morgue: "heir 4 · fighter · brave", rules: { rows: ROWS }, patches: [],
  trace: { turns: [], blow: { t: 900, by: "gas", dmg: 3, hp: 0 } }, ...o });
const go = async (d) => { await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d }), d); await waitScreen("death"); await sleep(300); };

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3102`, { waitUntil: "domcontentloaded" });
  await waitScreen("camp");
  await page.evaluate((rows) => { const r = window.__riddle; r.rules.rows = rows; r.rulesChanged(); }, ROWS);

  // ---- max hp in the trace: the core's steps
  {
    const why = [{ row: 0, why: "hp not <30%" }];
    const trace = { turns: [turn(700, 1, 16), turn(760, 1, 13), turn(820, 1, 10, { rows: why }), turn(860, 1, 10, { rows: why })],
      blows: [{ t: 880, by: "gas", dmg: 3, hp: 7 }, { t: 890, by: "gas", dmg: 3, hp: 4 }, { t: 900, by: "gas", dmg: 4, hp: 0 }], blow: { t: 900, by: "gas", dmg: 4, hp: 0 },
      max_steps: [{ t: 300, max: 38, delta: -6, cause: "drain" }, { t: 740, max: 32, delta: -6, cause: "drain" }, { t: 800, max: 29, delta: -3, cause: "hunger" }] };
    await go(deathOf({ trace }));
    const m = await page.evaluate(() => {
      const t = document.querySelector(".death .trace-panel");
      return { head: t?.querySelector(".hp-max")?.textContent.replace(/\s+/g, " ").trim(), steps: [...(t?.querySelectorAll("tr.max-step") ?? [])].map((r) => [...r.children].map((c) => c.textContent).join(" ").replace(/\s+/g, " ").trim()),
        hps: [...(t?.querySelectorAll("tbody tr:not(.max-step)") ?? [])].map((r) => r.children[2]?.textContent), why: (t?.querySelector(".chain-row .hp-at, .rows-line")?.textContent ?? "").replace(/\s+/g, " ").trim(),
        order: [...(t?.querySelectorAll("tbody tr") ?? [])].map((r) => r.children[0]?.textContent) };
    });
    check(m.head === "max hp 44→29 · drain −12 · hunger −3", `the run's max-hp arc reads over the table ("${m.head}")`);
    check(m.steps.length === 2 && /^740 max 38→32 drain −6$/.test(m.steps[0]) && /^800 max 32→29 hunger −3$/.test(m.steps[1]), `each step in the window is its own row (${m.steps.join(" | ")})`);
    check(m.hps[0] === "16/38" && m.hps[1] === "13/32" && m.hps[2] === "10/29" && m.hps.at(-1) === "0/29", `every hp reads over the max he had then (${m.hps.join(" · ")})`);
    check(m.order.every((t, i, a) => i === 0 || Number(t) >= Number(a[i - 1])), `the table's rows stay in tick order (${m.order.join(" ")})`);
    check(/10\/29/.test(m.why), `an \`hp not <30%\` reason reads his hp over the drained max ("${m.why}")`);
    await shot("cut28d-max-hp");
  }
  // ---- the per-turn max (`TraceTurn.max_hp`) and no step at all
  {
    const trace = { turns: [turn(10, 1, 20, { max_hp: 40 }), turn(20, 1, 12, { max_hp: 34 }), turn(30, 1, 9, { max_hp: 34 })], blow: { t: 40, by: "wraith", dmg: 9, hp: 0 } };
    await go(deathOf({ trace, cause: "wraith" }));
    const hps = await page.evaluate(() => [...document.querySelectorAll(".death .trace-panel tbody tr:not(.max-step)")].map((r) => r.children[2]?.textContent));
    check(hps.join(" ") === "20/40 12/34 9/34 0/34", `per-turn maxima read the same (${hps.join(" · ")})`);
    await go(deathOf({ trace: { turns: [turn(10, 1, 20), turn(20, 1, 12)], blow: { t: 30, by: "goblin", dmg: 12, hp: 0 } }, cause: "goblin" }));
    const plain = await page.evaluate(() => ({ hps: [...document.querySelectorAll(".death .trace-panel tbody tr")].map((r) => r.children[2]?.textContent), head: !!document.querySelector(".death .hp-max"), steps: document.querySelectorAll(".death tr.max-step").length }));
    check(plain.hps.join(" ") === "20 12 0" && !plain.head && !plain.steps, `a max that never moved leaves the table as it was (${plain.hps.join(" · ")})`);
  }
  // ---- the watched run's own `max_hp` events (no wire field): the run log of that run
  {
    const got = await page.evaluate(async () => {
      const { recordRun } = await import("/src/ui/runlog.ts"), { traceTable } = await import("/src/ui/trace.ts");
      const hero = { id: 1, kind: "hero", x: 0, y: 0, hp: 30, max_hp: 30, tags: [] };
      const v = recordRun({ load() {}, apply() {} }, 4242, 0);
      v.load({ turn: 0, depth: 3, hero, entities: [], items: [], seen: [] });
      v.apply([{ t: 50, k: "max_hp", id: 1, max: 27, delta: -3, cause: "drain" }]);
      const els = traceTable({ turns: [{ t: 40, row: 1, verb: { v: "attack" }, hp: 22, foes: 1, telegraphs: [] }, { t: 60, row: 1, verb: { v: "attack" }, hp: 11, foes: 1, telegraphs: [] }] }, { runId: 4242 });
      const box = document.createElement("div"); box.append(...els);
      return { head: box.querySelector(".hp-max")?.textContent.replace(/\s+/g, " ").trim(), hps: [...box.querySelectorAll("tbody tr:not(.max-step)")].map((r) => r.children[2]?.textContent), steps: box.querySelectorAll("tr.max-step").length };
    });
    check(got.head === "max hp 30→27 · drain −3" && got.hps.join(" ") === "22/30 11/27" && got.steps === 1, `a watched run's max-hp events read in its trace ("${got.head}"; ${got.hps.join(" · ")})`);
  }

  // ---- luck: a death most replays live leads with the event and its odds
  {
    const blow = { t: 900, by: "goblin", dmg: 6, hp: 0 };
    await go(deathOf({ cause: "goblin", depth: 8, baseline: 10 / 12, trace: { turns: [turn(880, 1, 6)], blow } }));
    const a = await page.evaluate(() => {
      const cloth = document.querySelector(".death .banner-cloth"), lead = cloth?.querySelector(".luck-lead"), line = cloth?.querySelector(".death-line");
      return { lead: lead?.textContent.replace(/\s+/g, " ").trim(), first: !!lead && !!line && !!(lead.compareDocumentPosition(line) & Node.DOCUMENT_POSITION_FOLLOWING),
        seal: document.querySelector(".death-line .verdict")?.className, sealW: document.querySelector(".death-line .verdict")?.getBoundingClientRect().width, leanTxt: document.querySelector(".death-line .lean")?.textContent };
    });
    check(a.lead === "goblin −6 at 6 hp · 1 in 6" && a.first, `a gap 10/12 replays live leads with its event and odds ("${a.lead}", above the stamp: ${a.first})`);
    check(/lean-seal/.test(a.seal ?? "") && (a.sealW ?? 99) < 70 && a.leanTxt === "10/12 replays survive", `the stamp steps back beside its count (${a.seal}, ${Math.round(a.sealW ?? 0)} px; "${a.leanTxt}")`);
    await shot("cut28d-luck");
    await go(deathOf({ cause: "goblin", depth: 8, baseline: 2 / 12, trace: { turns: [turn(880, 1, 6)], blow } }));
    check((await txt(".death .luck-lead")) === null, "a gap most replays die of has no luck line");
    await go(deathOf({ cause: "ogre", depth: 7, verdict: "dice", baseline: 12 / 12, trace: { turns: [turn(880, 1, 9)], blow: { t: 900, by: "ogre", dmg: 9, hp: 0 } } }));
    check((await txt(".death .luck-lead")) === "ogre −9 at 9 hp · 1 in 12+", `a dice no replay died of reads its odds as rarer than the replays ("${await txt(".death .luck-lead")}")`);
    await go(deathOf({ cause: "ogre", depth: 7, verdict: "dice", baseline: 0.5, lean: "dice", luck: { text: "a crit at 6 hp", one_in: 6 }, trace: { turns: [], blow: { t: 900, by: "ogre", dmg: 9, hp: 0 } } }));
    check((await txt(".death .luck-lead")) === "a crit at 6 hp · 1 in 6", `the core's own event and odds win ("${await txt(".death .luck-lead")}")`);
    await go(deathOf({ verdict: "stall", baseline: 1, trace: { turns: [turn(10, 1, 20)] } }));
    check((await txt(".death .luck-lead")) === null, "a stall has no luck line");
  }

  // ---- `no gain` never beside a move in the gain colour
  {
    await page.evaluate(() => { const r = window.__riddle; r.lastForecast = { ...(r.lastForecast ?? { causes: [] }), known_to: 14, depths: Array.from({ length: 14 }, (_, i) => ({ depth: i + 1, reach: i + 1 === 14 ? 0 : 0.5 })) }; });
    const p = (o) => ({ row: { conds: [{ k: "hp<", n: 50 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 0, forecast_delta: 0.24, forecast_depth: 14, forecast_pm: 0.03, ...o });
    await go(deathOf({ baseline: 0, trace: { turns: [turn(10, 1, 5)], blow: { t: 20, by: "gas", dmg: 5, hp: 0 } },
      patches: [p({}), p({ row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "drink", a: "heal" } }, whole: { reach: 0.24, reach_pm: 0.03, death: -0.2, death_pm: 0.02, reach_from: 0, reach_to: 0.24, death_from: 0.3 } })] }));
    const g = await page.evaluate(() => [...document.querySelectorAll(".death button.patch")].map((b) => ({ surv: b.querySelector(".surv")?.textContent, green: [...b.querySelectorAll(".delta.up, .dlt.up")].map((e) => e.textContent), reach: b.querySelector(".delta")?.textContent.replace(/\s+/g, " ").trim(), color: getComputedStyle(b.querySelector(".delta")).color, ok: getComputedStyle(document.body).getPropertyValue("--ok") })));
    check(g.length === 2 && g.every((x) => /no gain/.test(x.surv ?? "")), `both patches read \`no gain\` (${g.map((x) => x.surv).join(" | ")})`);
    check(g.every((x) => x.green.length === 0 && /D14 0→24%/.test(x.reach ?? "")), `no \`no gain\` tablet carries a gain-coloured move; the number stays (${g.map((x) => `${x.reach} [${x.green.join(",")}]`).join(" | ")})`);
    await shot("cut28d-no-gain");
    // a gainful patch keeps its green
    await go(deathOf({ baseline: 0, trace: { turns: [turn(10, 1, 5)], blow: { t: 20, by: "gas", dmg: 5, hp: 0 } }, patches: [p({ survive: 9 / 12 })] }));
    check((await page.evaluate(() => document.querySelectorAll(".death button.patch .delta.up").length)) === 1, "a patch that gains keeps its move in the gain colour");
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}\n${e.stack}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut28d: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut28d: ok (${out.length} checks)`);
