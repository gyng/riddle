#!/usr/bin/env node
// Cut 22 client gates (docs/CUT22.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §3  no move before an edit; an edit's forecast paints first, then the paired move lands (`forecastVs(prev)` asked after the
//       paint, never before it): a line under the shaft `vs last · D8 +6 · bank +4`, a tiny `▲6`/`▼3`/`≈` on each notch and gem;
//       the absolute numbers are the forecast's own; a move inside its paired ± reads `≈`; the next edit clears the line at once;
//       a set switch shows none; a core that puts `vs` on the forecast itself is read the same (no call)
//   §4  the start picker: `D9 · bank +3 · death 61% · $90` (the current start its own levels); no delta anywhere reads `+N%` (a
//       chance) — cage picker, start picker, unlock cards, a patch's reach — and each is in the delta look (signed, a lighter weight)
//   AG/AH  the send gem names a remembered `fast` watch (`send` + `fast`); `fights` is a bare `send`
//   AH  the boss moment: while a boss is in view only his plate (and the allies') is drawn, and the counter banner never sits on
//       the hero's callout or the ticker's beat
//
//   node web/tests/cut22.mjs [--shots dir]        (part of `pnpm test` in web/)
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
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
async function until(fn, label, timeout = 8000) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await fn(); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const closeSheets = async () => { await page.keyboard.press("Escape"); await sleep(120); };
const txt = (sel) => page.evaluate((s) => { const e = document.querySelector(s); return e && !e.hidden && e.getClientRects().length ? e.textContent.replace(/\s+/g, " ").trim() : null; }, sel);
const patchLineage = (patch) => page.evaluate(async (p) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  Object.assign(e.lineage, p); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
}, patch);
const vsText = () => txt(".camp .shaft-vs-host .shaft-vs");
/** The shaft's notches: the absolute text (`54%±6`, the move's mark taken out) and the mark. */
const notches = () => page.evaluate(() => [...document.querySelectorAll(".camp .shaft .notch:not(.fold)")].map((n) => {
  const dp = n.querySelector(".dp"), m = dp?.querySelector(".vsm");
  return { d: Number(n.dataset.d), abs: dp ? [...dp.childNodes].filter((c) => c !== m).map((c) => c.textContent).join("").trim() : "", mark: m?.textContent ?? null };
}));
const pctOf = (x) => `${Math.round(x * 100)}%`;

try {
  // ---- §3: the edit's paired move
  const rules3 = encodeURIComponent("hp<30% → drink heal\nfoes>=1 → attack nearest\ndepth>=6 → bank");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=221&rules=${rules3}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await patchLineage({ best_depth: 5, heir: 3 });
  await until(() => page.evaluate(() => document.querySelector(".camp .shaft")?.dataset.fc), "the first forecast");
  await sleep(300);
  check((await vsText()) === null, "no move before an edit");
  // the order the worker sees: the forecast (painted) before the paired call, which carries the set before the edit
  await page.evaluate(() => {
    const r = window.__riddle, e = r.engine; r.__log = [];
    const fc = e.forecast.bind(e), vs = e.forecastVs.bind(e);
    e.forecast = async () => { const f = await fc(); r.__log.push("forecast"); return f; };
    e.forecastVs = async (prev) => { r.__log.push(`vs:${document.querySelector(".camp .shaft")?.dataset.fc}:${prev.rows.length}`); return vs(prev); };
  });
  const fc0 = await page.evaluate(() => Number(document.querySelector(".camp .shaft").dataset.fc));
  await page.evaluate(() => window.__riddle.insertRow({ conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } }, 0));
  check((await vsText()) === null, "an edit shows no move while its forecast runs");
  const line = await until(vsText, "the move under the shaft");
  const log = await page.evaluate(() => window.__riddle.__log);
  check(log[0] === "forecast" && log[1] === `vs:${fc0 + 1}:3`, `the paired call waits for the paint and measures against the set before the edit (${log.join(" → ")})`);
  check(/^vs last · D\d+ ([+−]\d+|≈) · bank ([+−]\d+|≈)( · death [+−]\d+)?$/.test(line), `the line under the shaft: "${line}"`);
  const ns = await notches(), f = await page.evaluate(() => window.__riddle.lastForecast);
  const marked = ns.filter((n) => n.mark !== null);
  check(marked.every((n) => /^[▲▼]\d+$/.test(n.mark)) && ns.every((n) => n.mark === null || !/≈/.test(n.mark)), `a notch carries its move when it clears its ± (nothing inside it) (${ns.map((n) => `D${n.d} ${n.mark ?? "-"}`).join(" · ")})`);
  const absOk = ns.every((n) => { const d = f.depths.find((x) => x.depth === n.d); return !d || n.abs.startsWith(pctOf(d.reach)); });
  check(absOk, `the notches' absolute numbers are the forecast's own (${ns.map((n) => n.abs).join(" · ")})`);
  const gems = await page.evaluate(() => [...document.querySelectorAll(".camp .shaft-ends .end .vsm")].map((m) => m.closest(".end").className.replace("end ", "") + " " + m.textContent));
  check(gems.every((g) => /^(bank|return|stall|death) [▲▼]$/.test(g)), `a gem carries a bare arrow when it moves (${gems.join(" · ") || "none"})`);
  const look = await page.evaluate(() => { const b = document.querySelector(".camp .shaft-vs-host .vs-term b"), n = document.querySelector(".camp .shaft .notch .dp"); return b ? { w: Number(getComputedStyle(b).fontWeight), fs: parseFloat(getComputedStyle(b).fontSize), nfs: parseFloat(getComputedStyle(n).fontSize) } : null; });
  check(!!look && look.w <= 400, `the move is in the delta look (weight ${look?.w})`);
  await shot("cut22-vs-line");
  // the forecast panel carries the same line
  await openPanel(page, "forecast");
  const panelVs = await txt(".panel .forecast .fc-vs .shaft-vs");
  check(panelVs === line, `the forecast panel shows the same move ("${panelVs}")`);
  await shot("cut22-vs-panel");
  await closeSheets(); await page.locator(".camp .shaft").click().catch(() => {}); await sleep(100);
  if (await page.locator(".panel[data-panel=forecast]").count()) await closeSheets();
  // a move inside its paired ± reads `≈`; a move outside it keeps its sign — the engine's numbers, as sent
  await page.evaluate(() => {
    const e = window.__riddle.engine;
    e.forecastVs = async () => ({ depths: [1, 2, 3, 4, 5, 6].map((depth) => ({ depth, delta: depth === 6 ? -0.07 : 0.03, pm: depth === 6 ? 0.02 : 0.05 })), bank: { delta: 0.04, pm: 0.02 }, death: { delta: 0.01, pm: 0.03 } });
  });
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows[0].conds[0].n = 40; r.rulesChanged(); });
  check((await vsText()) === null, "the next edit clears the line at once");
  const line2 = await until(vsText, "the second move");
  check(line2 === "vs last · D6 −7 · bank +4", `the largest move outside its ± heads the line, a death inside its ± is left out ("${line2}")`);
  const gems2 = await page.evaluate(() => [...document.querySelectorAll(".camp .shaft-ends .end .vsm")].map((m) => m.closest(".end").className.replace("end ", "") + " " + m.textContent));
  check(gems2.join() === "bank ▲", `the bank gem carries its arrow, a death inside its ± none (${gems2.join(" · ") || "none"})`);
  const ns2 = await notches();
  check(ns2.filter((n) => n.d < 6).every((n) => n.mark === null) && ns2.find((n) => n.d === 6)?.mark === "▼7", `a notch inside its paired ± is unmarked, one outside it \`▼7\` (${ns2.map((n) => `D${n.d} ${n.mark ?? "-"}`).join(" · ")})`);
  // nothing clears: the frontier's `≈`
  await page.evaluate(() => { window.__riddle.engine.forecastVs = async () => ({ depths: [1, 2, 3, 4, 5, 6].map((depth) => ({ depth, delta: 0.02, pm: 0.04 })), bank: { delta: 0.004, pm: 0.03 } }); });
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows[0].conds[0].n = 45; r.rulesChanged(); });
  const line3 = await until(vsText, "the flat move");
  check(line3 === "vs last · D6 ≈ · bank ≈", `an edit that moves nothing reads ≈ on the frontier ("${line3}")`);
  await shot("cut22-vs-flat");
  // a set switch is not an edit
  await page.evaluate(() => window.__riddle.selectSet(1));
  await until(() => page.evaluate(() => !document.querySelector(".camp .shaft").classList.contains("stale")), "the other set's forecast");
  await sleep(300);
  check((await vsText()) === null, "a set switch shows no move");
  await page.evaluate(() => window.__riddle.selectSet(0)); await sleep(400);
  // a core that sends `vs` on the forecast itself: read, no call
  await page.evaluate(() => {
    const r = window.__riddle, e = r.engine; r.__vsCalls = 0;
    e.forecastVs = async () => { r.__vsCalls++; return { depths: [] }; };
    const fc = e.forecast; e.forecast = async () => ({ ...(await fc()), vs: { depths: [{ depth: 6, delta: 0.12, pm: 0.03 }], bank: 0.09 } });
  });
  await page.evaluate(() => { const r = window.__riddle; r.rules.rows[0].conds[0].n = 25; r.rulesChanged(); });
  const line4 = await until(vsText, "the inline move");
  const calls = await page.evaluate(() => window.__riddle.__vsCalls);
  check(line4 === "vs last · D6 +12 · bank +9" && calls === 0, `a forecast's own \`vs\` is read without a call ("${line4}", ${calls} calls)`);

  // ---- §4: the start picker's death share; signed deltas
  const bankRules = encodeURIComponent("foes>=1 → attack nearest\ndepth>=12 → bank");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=222&rules=${bankRules}`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await patchLineage({ best_depth: 12, heir: 3, gold: 500, waystones: [5, 9], start: 1, facts: ["item:leash", "vault"] });
  await until(() => page.evaluate(() => document.querySelector(".camp .shaft")?.dataset.fc), "the forecast"); await sleep(300);
  await page.locator(".camp .start-tab").click({ timeout: 5000 }); await sleep(150);
  const opts = await until(async () => { const o = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .start-picker .start-opt")].map((b) => `${b.textContent.replace(/\s+/g, " ").trim()}${b.classList.contains("on") ? "*" : ""}`)); return o.length === 3 && !o.some((x) => x.includes("…")) ? o : null; }, "the measured starts");
  check(/^D1 · (bank|D\d+) \d+% · death \d+%\*$/.test(opts[0]), `the current start: its own levels and its death share (${opts[0]})`);
  check(/^D5 · (bank|D\d+) [+−]\d+ · death \d+% · \$50$/.test(opts[1]) && /^D9 · (bank|D\d+) [+−]\d+ · death \d+% · \$90$/.test(opts[2]), `each start: the signed move, the death share, the toll (${opts.slice(1).join(" | ")})`);
  const deathWarn = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .start-opt .start-death")].map((d) => `${d.textContent.trim()}${d.classList.contains("warn") ? "!" : ""}`));
  check(deathWarn.every((d) => { const n = Number(/(\d+)%/.exec(d)?.[1]); return (n >= 50) === d.endsWith("!"); }), `a death share of half or more is marked (${deathWarn.join(" ")})`);
  await shot("cut22-start-picker");
  await closeSheets();
  // the cage picker
  await page.locator(".camp .cage-tab").click({ timeout: 5000 }); await sleep(150);
  const cage = await until(async () => { const o = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .cage-opt")].map((b) => b.textContent.replace(/\s+/g, " ").trim())); return o.length === 4 && !o.some((x) => x.includes("…")) ? o : null; }, "the cage picker");
  check(cage.filter((c) => /[+−]/.test(c)).length >= 1 && cage.every((c) => !/[+−]\d+%/.test(c)), `the cage picker's moves are signed points (${cage.join(" | ")})`);
  await closeSheets();
  // every delta the camp shows: signed, never `%`, in the delta look
  await page.evaluate(() => {
    const r = window.__riddle, row = (t) => [{ conds: [{ k: "foe_tag", t }], verb: { v: "retreat" } }];
    const cat = [
      { id: "kite_archers", cost: 1, owned: false, available: true, gold: 150, delta: 0.12, pm: 0.03, insert_at: 0, situation: "ranged", rows: row("ranged") },
      { id: "gas_dodge", cost: 1, owned: false, available: true, gold: 150, delta: -0.08, pm: 0.03, stall: 0.11, insert_at: 1, situation: "gas", rows: row("gas") },
      { id: "thief_guard", cost: 1, owned: false, available: true, gold: 150, delta: 0.01, pm: 0.03, insert_at: 0, situation: "thief", rows: row("thief") },
    ];
    r.engine.unlocks = async () => cat; r.engine.unlockDeltas = async () => cat;
    r.lineage = { ...r.lineage, marks: 5 }; localStorage.setItem("riddle.unlocks.all", "1"); r.go({ kind: "camp" });
  });
  await sleep(400);
  await openPanel(page, "unlocks", { all: true });
  await sleep(600);
  const deltas = await page.evaluate(() => [...document.querySelectorAll(".delta, .dlt")].filter((e) => e.getClientRects().length && e.textContent.trim()).map((e) => ({ text: e.textContent.replace(/\s+/g, " ").trim(), w: Number(getComputedStyle(e).fontWeight) })));
  check(deltas.length > 0 && deltas.every((d) => !/[+−]\d+%/.test(d.text)), `no delta reads as a chance (${deltas.slice(0, 6).map((d) => d.text).join(" · ")})`);
  check(deltas.every((d) => d.w <= 400), `every delta in the delta look (weights ${[...new Set(deltas.map((d) => d.w))].join(",")})`);
  await shot("cut22-unlock-deltas");
  await closeSheets();

  // ---- AG/AH: the send gem names a remembered `fast`
  await page.evaluate(() => { const r = window.__riddle; r.watchMode = "fast"; r.go({ kind: "camp" }); }); await sleep(250);
  const sendFast = await page.evaluate(() => { const b = document.querySelector(".gem.send"); return { t: b?.textContent.replace(/\s+/g, " ").trim(), mode: b?.dataset.mode }; });
  check(sendFast.t === "sendfast" && sendFast.mode === "fast", `a remembered \`fast\` is on the send gem (${JSON.stringify(sendFast)})`);
  await shot("cut22-send-fast");
  await page.evaluate(() => { const r = window.__riddle; r.watchMode = "fights"; r.go({ kind: "camp" }); }); await sleep(250);
  const sendFights = await page.evaluate(() => document.querySelector(".gem.send")?.textContent.trim());
  check(sendFights === "send", `\`fights\` is a bare send ("${sendFights}")`);

  // ---- AH: the boss moment — one line wins
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=223`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  // the engine's first batch after tick 20 puts the warlord and a horde of goblins around the hero, the warlord's rally and his break
  await page.evaluate(() => {
    const r = window.__riddle, e = r.engine; let done = false;
    const step = e.step.bind(e);
    e.step = async (n) => {
      const res = await step(n);
      const s = res.snapshot; if (done || !s || s.turn < 20) return res;
      done = true; const hh = s.hero, t = s.turn;
      const horde = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, 1], [1, -1], [-1, -1], [2, 0]];
      const ok = (x, y) => x >= 0 && y >= 0 && x < s.w && y < s.h;
      const put = (id, kind, dx, dy, tags) => { const x = hh.x + dx, y = hh.y + dy; if (!ok(x, y)) return; const ent = { id, kind, x, y, hp: 30, max_hp: 30, tags };
        res.events.push({ t: t - 6, k: "spawn", e: ent }); s.entities.push(ent); const i = y * s.w + x; s.visible[i] = true; s.seen[i] = true; };
      put(96000, "goblin_warlord", horde[0][0], horde[0][1], ["boss"]);
      horde.slice(1).forEach(([dx, dy], k) => put(96001 + k, "goblin", dx, dy, []));
      res.events.push({ t: t - 5, k: "telegraph", id: 96000, what: "rallies" }, { t: t - 4, k: "callout", text: "rallied!" },
        { t: t - 3, k: "hurt", id: 96000, dmg: 16, hp: 14, cause: "hero" }, { t: t - 3, k: "callout", text: "warlord breaks" });
      return res;
    };
    r.dev.speed = "fights";
  });
  await page.locator(".gem.send").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "watch", "the watch");
  // sample the stage while the warlord is in view: the drawn plates, the banner, the counter line, the ticker (node-side, so the
  // moment can be shot)
  const sample = () => page.evaluate(() => {
    const w = document.querySelector(".watch"); if (!w?.dataset.boss) return null;
    const tags = [...document.querySelectorAll(".watch .rtag")].filter((t) => t.getClientRects().length && getComputedStyle(t).display !== "none" && getComputedStyle(t).visibility !== "hidden").map((t) => t.textContent.trim());
    const rect = (sel) => { const e = document.querySelector(sel); if (!e || e.hidden || !e.getClientRects().length || getComputedStyle(e).opacity === "0") return null; const r = e.getBoundingClientRect(); return { top: r.top, bottom: r.bottom, text: e.textContent.trim(), cls: e.className }; };
    const canvas = (window.__viewer?.debugText?.() ?? []).map((x) => x.text);
    return { tags, canvas, banner: rect(".watch .banner.show"), bossLine: rect(".watch .boss-hp .counter"), ticker: rect(".watch .ticker.show"), frame: w.dataset.frame };
  });
  const samples = []; let shotTaken = false;
  for (const t0 = Date.now(); Date.now() - t0 < 12_000 && samples.length < 120;) {
    const x = await sample(); if (x) { samples.push(x); if (!shotTaken && x.frame === "fight" && x.bossLine && x.ticker) { shotTaken = true; await shot("cut22-boss"); } }
    await sleep(60);
  }
  const seen = samples.filter((s) => s.frame === "fight");
  check(seen.length > 0, `the warlord's fight is framed (${samples.length} samples, ${seen.length} in the fight frame)`);
  const plates = seen.map((s) => s.tags);
  check(seen.length > 0 && plates.every((t) => t.every((x) => !/^goblin$/.test(x))), `while the boss is in view only his plate is drawn (${[...new Set(plates.flat())].join(" · ") || "none"})`);
  const midBanner = seen.filter((s) => s.banner && /counter/.test(s.banner.text));
  check(midBanner.length === 0, `the counter never floats over the fight (${midBanner.length} samples with a mid-stage banner)`);
  const lines = [...new Set(seen.map((s) => s.bossLine?.text).filter(Boolean))];
  check(lines.length === 1 && /^counter: /.test(lines[0]), `the counter is the boss bar's second line (${lines.join(" | ") || "never"})`);
  const overlap = seen.filter((s) => s.bossLine && s.ticker && s.bossLine.bottom > s.ticker.top && s.bossLine.top < s.ticker.bottom);
  check(overlap.length === 0, `the counter line and the ticker never overlap (${overlap.length})`);
  const twice = seen.filter((s) => s.ticker && s.canvas.some((c) => c.toUpperCase() === s.ticker.text.toUpperCase()));
  const broke = seen.some((s) => s.ticker?.text === "WARLORD BREAKS");
  const crowded = seen.filter((s) => s.ticker?.text === "WARLORD BREAKS" && s.canvas.length);
  check(broke && twice.length === 0 && crowded.length === 0, `the break is one line — the beat under the fight, nothing drawn over the hero meanwhile (${broke ? "beat seen" : "no beat"}; ${twice.length} doubled; ${crowded.map((s) => s.canvas.join("+")).join(" | ") || "0 crowded"})`);
  if (!shotTaken) await shot("cut22-boss");

} catch (e) {
  errors.push(`exception: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(`error ${e}`);
const bad = failed + errors.filter((e) => !/favicon|ResizeObserver/.test(e)).length;
console.log(bad ? `FAIL cut22: ${failed} checks failed, ${errors.length} errors` : `ok cut22: ${out.length} checks`);
process.exit(bad ? 1 : 0);
