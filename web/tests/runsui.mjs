#!/usr/bin/env node
// RUNS_UI client gates (docs/RUNS_UI.md §8) — the run lanes, the live watch, the runs log, the replay; headless at 400 × 800 (and
// 1440 × 900 for the desktop part), RIDDLE_BROWSER=headed for the GPU. Real wasm unless named (`?runs=1`: the open app's clock on under
// automation).
//   manual    day 0 before the scout: the lane `waits ▸ send`, `auto` greyed with the scout's count, the gem lit `SEND 0/3`, the lane inert,
//             the log hidden; ≤ 12 elements above the fold; ≤ 5 surfaces in the well and console (AUTOMATION_TREE §4's day 0)
//   live      SEND → the watch; `town ↻` leaves it mid-run: the lane reads `live D…` with hp, the run's tick moves on in the town (the
//             run counter advances); the lane opens the watch on the same run; left again, the run ends in the town and the log holds
//             it (`via town`), the lane `waits` again (the manual phase: one run a send); the log has one entry from run 1
//   rests     the scout hired: home after a run the lane reads `rests Nm` with `↻ auto` lit; the gem `send`; due, the next run goes down
//             by itself (the lane turns `live`)
//   log       an absence (2 h) after the scout: the log folds it as `away · N runs` (one line until opened), the town's runs as `here`;
//             the newest fold open; entries carry #, the end, D, $, length; a death's entry opens its verdict (when the fold holds one)
//   replay    a watched run's events (every step and fold the engine returned, its exit line and trace left out) hash the same as its
//             replay's; the log's ▶ opens the replay sheet and plays its floors to the end
//   density   the camp with lanes: ≤ 12 elements above the fold at 400 × 800; the lane's labels ≤ 2 words; no sentences; the new terms tip
//   heroes    (fake) the lanes render N rows from an array: 1 and 3 heroes; a 4th folds into `+1`; desktop shows them in the centre column
//
//   node web/tests/runsui.mjs [--shots dir] [--part=a,b]      (part of `pnpm test` in web/)
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
const words = (s) => String(s ?? "").split(/\s+/).filter((w) => /\p{L}/u.test(w)).length;
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
let page = null;
async function open(q, viewport = { width: 400, height: 800 }) {
  if (page) await page.close();
  page = await browser.newPage({ viewport, deviceScaleFactor: shots ? 2 : 1 });
  page.on("console", (m) => { if (m.type() === "error" && !/Failed to load resource/.test(m.text())) errors.push(`console.error: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&runs=1&${q}`);
  await until(() => window.__riddle?.booted, "boot", 60_000);
}
const shot = async (name, ms = 400) => { if (shots) { await page.mouse.move(1, 300); await sleep(ms); await page.screenshot({ path: resolve(shots, `${name}.png`) }); } };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(100); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp" && !!document.querySelector(".lanes"), "camp", 30_000); await sleep(300); };
/** the lane block as the player sees it */
const lanes = () => page.evaluate(() => {
  const q = (s) => [...document.querySelectorAll(s)];
  return { n: Number(document.querySelector(".lanes")?.dataset.n ?? 0), rows: q(".lanes .lane:not(.lane-more)").map((l) => ({ state: l.dataset.state, tag: l.tagName, text: l.textContent.replace(/\s+/g, " ").trim(), auto: l.querySelector(".lane-auto")?.dataset.auto, gauge: l.querySelector(".lane-gauge")?.dataset.pct })),
    more: document.querySelector(".lanes .lane-more")?.dataset.more ?? null, log: (() => { const b = document.querySelector(".lanes .lanes-log"); return b && !b.hidden ? b.textContent.trim() : null; })(),
    gem: document.querySelector(".console .gem")?.textContent.replace(/\s+/g, " ").trim() ?? "", pulse: !!document.querySelector(".console .gem.pulse") };
});
const L = () => page.evaluate(() => { const l = window.__riddle.lineage; return { live: l.live ?? null, runs: l.runs ?? [], replays: l.replays ?? [], waits: !!l.tree?.waits, auto: !!l.tree?.auto_send, rest: l.rest_left_s ?? 0, heir: l.heir }; });
/** the interactive elements above the fold, one per function (cut305's reading) */
const SAME = { forge: "blacksmith", vault: "storehouse", party: "kennel", bank: "bank", loadout: "crate" };
const fold = (scope = "") => page.evaluate(([SAME, scope]) => {
  const shown = (b) => {
    const r = b.getBoundingClientRect(); let top = Math.max(0, r.top), bottom = Math.min(innerHeight, r.bottom);
    for (let p = b.parentElement; p && bottom > top; p = p.parentElement) { const o = getComputedStyle(p).overflowY; if (o === "visible") continue; const q = p.getBoundingClientRect(); top = Math.max(top, q.top); bottom = Math.min(bottom, q.bottom); }
    return bottom - top > 1 && r.width > 0;
  };
  const els = [...document.querySelectorAll(`${scope} :is(button, input, select, textarea, a[href], [role=button])`)].filter((b) => !b.closest("[inert]") && b.getClientRects().length && getComputedStyle(b).visibility !== "hidden" && !b.hidden && shown(b));
  // (a run under way: the gem, the mouth and the main hero's live lane all watch it — one function)
  const live = document.querySelector(".console .gem[data-live='1']");
  const key = (b) => live && (b === live || b.dataset.building === "mouth" || (b.matches(".lane[data-state=live]") && b === document.querySelector(".lanes .lane"))) ? "watch"
    : b.classList.contains("next-pill") || b.dataset.building === "worker" ? "works" : b.dataset.building ?? (b.dataset.tile ? SAME[b.dataset.tile] ?? `tile:${b.dataset.tile}` : (b.getAttribute("aria-label") || b.textContent || b.className).replace(/\s+/g, " ").trim().slice(0, 24));
  return [...new Set(els.map(key))];
}, [SAME, scope]);
/** send by the gem; resolves on the watch with the run's id (the engine's `send` answer, tapped) */
async function sendWatch() {
  await page.evaluate(() => { const r = window.__riddle; if (r.__sendTap) return; const o = r.engine.send.bind(r.engine); r.engine.send = async () => { const s = await o(); window.__sentRun = s.run.id; return s; }; r.__sendTap = true; });
  await page.evaluate(() => { window.__sentRun = undefined; });
  await page.locator(".console .gem").click();
  await until(() => window.__riddle.screen === "watch" && window.__sentRun !== undefined, "the watch", 30_000);
  return page.evaluate(() => window.__sentRun);
}
/** FNV-1a over each event's JSON, an exit's line and trace left out, up to and including the exit */
const HASH = `(evs) => { let h = 0xcbf29ce484222325n; const P = 0x100000001b3n, M = (1n << 64n) - 1n;
  for (const e of evs) { const x = e.k === "exit" ? { ...e, line: undefined, trace: undefined } : e; const s = JSON.stringify(x); for (let i = 0; i < s.length; i++) { h ^= BigInt(s.charCodeAt(i) & 0xff); h = (h * P) & M; } if (e.k === "exit") break; }
  return h.toString(16).padStart(16, "0"); }`;

try {
  // ---- the manual phase: day 0, the lane waits
  if (part("manual") || part("live") || part("replay")) {
    await open(`seed=${Number(process.env.RUNSUI_SEED ?? 4101)}&fresh=1`);
    await camp();
    const s = await lanes(), l = await L();
    await shot("manual-day0");
    check(s.n === 1 && s.rows[0]?.state === "waits" && /waits/.test(s.rows[0].text) && /send/.test(s.rows[0].text), `day 0: one lane, \`waits ▸ send\` (${s.rows[0]?.text})`);
    check(s.rows[0]?.auto === "0" && /auto\s*0\/3/.test(s.rows[0].text), `day 0: \`auto\` greyed with the scout's count (${s.rows[0]?.text})`);
    check(s.pulse && /send/i.test(s.gem) && /0\/3/.test(s.gem), `day 0: the gem lit, \`SEND 0/3\` (${s.gem})`);
    check(s.rows[0]?.tag === "DIV" && s.log === null && l.runs.length === 0, `day 0: the lane is no surface, the log hidden (${s.rows[0]?.tag}, log ${s.log})`);
    const surf = await fold(":is(.well-wrap, .console)"), all = await fold();
    check(surf.length <= 5, `day 0: ≤ 5 surfaces in the well and console (${surf.length}: ${surf.join(" | ")})`);
    check(all.length <= 12, `day 0: ≤ 12 elements above the fold (${all.length}: ${all.join(" | ")})`);
  }

  // ---- live: the watch, out to the town (he keeps going), back in, out; the run ends in the town
  let watchedRun = 0;
  if (part("live")) {
    const run = await sendWatch();
    await until(() => !!document.querySelector(".console .tile[data-tile=town]:not([disabled])"), "the town tile", 15_000);
    await sleep(1500);
    await shot("live-watch");
    const townTile = await page.evaluate(() => document.querySelector(".console .tile[data-tile=town]")?.textContent.trim());
    check(townTile === "town", `the watch carries \`town\` (${townTile})`);
    await page.locator(".console .tile[data-tile=town]").click();
    await camp();
    const a = await until(() => { const l = window.__riddle.lineage.live; return l && l.turn > 0 ? l : null; }, "the run live in the town", 15_000);
    const s = await lanes();
    await shot("live-town");
    check(a.run_id === run && s.rows[0]?.state === "live" && /live D\d+/.test(s.rows[0].text) && Number(s.rows[0].gauge) > 0, `left mid-run: the lane reads live with his floor and hp (${s.rows[0]?.text}; run ${a.run_id} = ${run})`);
    check(/watch/i.test(s.gem), `a run under way: the gem reads \`watch\` (${s.gem})`);
    const b = await until((t0) => { const l = window.__riddle.lineage.live; return l && l.turn > t0 + 10 ? l : (window.__riddle.lineage.runs ?? []).some((r) => r.id === window.__riddle.lineage.live?.run_id) ? l : null; }, "the run's tick moving in the town", 20_000, a.turn);
    check(b.turn > a.turn, `leaving the watch keeps the run going: tick ${a.turn} → ${b.turn} in the town`);
    // back into the same run from the lane
    await page.evaluate(() => { window.__sentRun = undefined; });
    await page.locator(".lanes .lane[data-state=live]").click();
    await until(() => window.__riddle.screen === "watch" && window.__sentRun !== undefined, "the watch from the lane", 20_000);
    const again = await page.evaluate(() => window.__sentRun);
    check(again === run, `the lane opens the watch on the run in flight (run ${again} = ${run})`);
    await until(() => !!document.querySelector(".console .tile[data-tile=town]:not([disabled])"), "the town tile again", 15_000);
    await page.locator(".console .tile[data-tile=town]").click();
    await camp();
    const rec = await until((id) => (window.__riddle.lineage.runs ?? []).find((r) => r.id === id) ?? null, "the run's end in the log", 240_000, run);
    await sleep(1200);
    const s2 = await lanes(), l2 = await L();
    await shot("live-ended");
    check(rec.via === "town", `the run left mid-watch ended in the town: the log holds it as \`town\` (${rec.via}, D${rec.depth}, ${rec.tier})`);
    check(s2.rows[0]?.state === "waits" && !l2.auto, `before the scout one send is one run: the lane waits again (${s2.rows[0]?.text})`);
    check(s2.log !== null && /log/.test(s2.log) && l2.runs.filter((r) => r.id > 0).length === 1, `the log from run 1: one entry, the stud shown (${s2.log})`);
    watchedRun = run;
  }

  // ---- replay: a watched run, its events tapped, hashes the same as its replay; the log's ▶ plays it
  if (part("replay")) {
    await page.evaluate(() => {
      const r = window.__riddle; window.__evs = []; window.__tapOn = true;
      if (!r.__stepTap) {
        const st = r.engine.step.bind(r.engine), fo = r.engine.fold?.bind(r.engine);
        r.engine.step = async (n) => { const x = await st(n); if (window.__tapOn) window.__evs.push(...x.events); return x; };
        if (fo) r.engine.fold = async () => { const x = await fo(); if (window.__tapOn) window.__evs.push(...x.step.events); return x; };
        r.__stepTap = true;
      }
    });
    const run = await sendWatch();
    await page.evaluate(() => { for (const b of document.querySelectorAll(".console .tile[data-tile=fast]")) b.click(); });
    // the run to its end (▶▶| as a player would), the exit sheet kept, the report, the camp
    for (let i = 0; i < 600; i++) {
      const sc = await page.evaluate(() => window.__riddle.screen);
      if (sc === "camp") break;
      await page.evaluate(() => { const b = [...document.querySelectorAll("button")].filter((x) => x.offsetWidth && !x.disabled); const t = (s) => b.find((x) => x.textContent.trim() === s); (t("keep") ?? t("▶▶|") ?? t("REPORT") ?? t("report") ?? t("CAMP") ?? t("camp") ?? t("VERDICT") ?? t("verdict") ?? document.querySelector(".console .gem:not([disabled])"))?.click(); });
      await sleep(400);
    }
    await page.evaluate(() => { window.__tapOn = false; });
    if ((await page.evaluate(() => window.__riddle.screen)) !== "camp") await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
    await camp();
    const live = await page.evaluate(new Function(`return (${HASH})(window.__evs)`));
    const rep = await page.evaluate(async (id) => { const x = await window.__riddle.engine.replay(id); return x && { hash: x.hash, floors: x.floors.length, n: x.floors.reduce((a, f) => a + f.events.length, 0), evs: x.floors.flatMap((f) => f.events) }; }, run);
    const replayHash = rep ? await page.evaluate(new Function("evs", `return (${HASH})(evs)`), rep.evs) : null;
    const nLive = await page.evaluate(() => { const i = window.__evs.findIndex((e) => e.k === "exit"); return i < 0 ? window.__evs.length : i + 1; });
    check(!!rep && replayHash === live, `the watched run (${nLive} events) replays identically: ${live} = ${replayHash} (core's ${rep?.hash}, ${rep?.floors} floors)`);
    // the log's ▶ on it
    await page.locator(".lanes .lanes-log").click();
    await until(() => !!document.querySelector(".runs-sheet"), "the log", 5000);
    await shot("log-manual");
    const btn = page.locator(`.runs-sheet .re-play[data-run="${run}"]`);
    check((await btn.count()) === 1, `the log's entry for run ${run} carries ▶`);
    if (await btn.count()) {
      await btn.click();
      await until(() => !!document.querySelector(".run-replay") && (window.__runReplay?.floor ?? -1) >= 0, "the replay sheet", 15_000);
      await sleep(1500); await shot("replay");
      const r1 = await page.evaluate(() => window.__runReplay);
      check(r1.run === run && r1.hash === rep?.hash, `the replay sheet plays run ${r1.run} (hash ${r1.hash})`);
      // to the end: a tap on the picture goes to the next floor
      for (let i = 0; i < 40 && !(await page.evaluate(() => window.__runReplay?.done)); i++) { await page.locator(".run-replay-view").click(); await sleep(250); }
      check(await page.evaluate(() => window.__runReplay?.done), `the replay plays its ${r1.floors} floors to the end`);
      await page.keyboard.press("Escape"); await sleep(200); await page.keyboard.press("Escape"); await sleep(200);
    }
    void watchedRun;
  }

  // ---- rests and the log by absence: the scout hired, an absence, the town's runs
  if (part("rests") || part("log") || part("density")) {
    await open(`seed=${Number(process.env.RUNSUI_SEED2 ?? 4102)}&fresh=1`);
    await camp();
    // the manual phase done by the save: the scout hired (as `hire` would), two hours away
    await page.evaluate(async () => { const r = window.__riddle; for (let i = 0; i < 3 && !r.lineage.tree.auto_send; i++) { try { r.lineage = await r.engine.hire("scout"); } catch { /* not lit: send by hand */ break; } } });
    let l = await L();
    for (let i = 0; i < 4 && !l.auto; i++) {
      // three sends by hand (the trunk's), each played out unwatched by the open app's clock
      await page.evaluate(async () => { const r = window.__riddle; await r.engine.send(); r.lineage = await r.engine.lineage(); for (let k = 0; k < 400 && r.lineage.live; k++) { await r.engine.advance(20_000); r.lineage = await r.engine.lineage(); } try { r.lineage = await r.engine.openChest(); } catch { /* none */ } try { r.lineage = await r.engine.hire(r.lineage.tree.lit ?? ""); } catch { /* not yet */ } r.go({ kind: "camp" }); });
      l = await L();
    }
    check(l.auto, `the scout hired by the trunk's sends (auto ${l.auto})`);
    await camp();
    // a run ends; home, he rests
    await page.evaluate(async () => { const r = window.__riddle; await r.engine.send(); for (let k = 0; k < 400; k++) { const a = await r.engine.advance(20_000); if (a.ended.length) break; } r.lineage = await r.engine.lineage(); r.go({ kind: "camp" }); });
    await camp();
    const s = await lanes();
    await shot("rests");
    check(s.rows[0]?.state === "rests" && /rests \d+m/.test(s.rows[0].text) && s.rows[0].auto === "1", `home after a run: \`rests Nm\` with \`↻ auto\` lit (${s.rows[0]?.text})`);
    check(/send/i.test(s.gem) && !/watch/i.test(s.gem), `resting: the gem sends (${s.gem})`);
    // due: the rest left to a second, the open app's clock sends him down by itself
    await page.evaluate(async () => { const r = window.__riddle; const a = await r.engine.advance(Math.max(0, (r.lineage.rest_left_s ?? 0) - 2) * 1000); void a; r.lineage = await r.engine.lineage(); r.go({ kind: "camp" }); });
    await camp();
    const went = await until(() => window.__riddle.lineage.live?.turn > 0 ? window.__riddle.lineage.live : null, "the next run down by itself", 30_000).catch(() => null);
    const s2 = await lanes();
    check(!!went && s2.rows[0]?.state === "live", `the rest out, the next run goes down by itself: the lane turns live (${s2.rows[0]?.text})`);
    if (part("log") || part("density")) {
      await page.evaluate(async () => { const r = window.__riddle; for (let k = 0; k < 400 && r.lineage.live; k++) { await r.engine.advance(20_000); r.lineage = await r.engine.lineage(); } });
      // two hours away: an absence and its report
      await page.evaluate(async () => { const r = window.__riddle; await r.absence(2 * 3600); });
      await until(() => window.__riddle.screen === "report", "the absence's report", 120_000);
      await shot("report");
      const tileLog = await page.evaluate(() => !!document.querySelector(".tiles .to-log"));
      check(tileLog, "the report's runs tile opens the log");
      await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
      await camp();
      const ll = await L();
      await page.locator(".lanes .lanes-log").click();
      await until(() => !!document.querySelector(".runs-sheet"), "the log", 5000);
      await sleep(300);
      await shot("log");
      const g = await page.evaluate(() => ({ folds: [...document.querySelectorAll(".runs-fold")].map((f) => ({ via: f.dataset.via, n: Number(f.dataset.n), on: f.classList.contains("on"), text: f.textContent.replace(/\s+/g, " ").trim() })),
        entries: [...document.querySelectorAll(".runs-sheet .run-entry:not(.sampled)")].map((e) => ({ run: Number(e.dataset.run), text: e.textContent.replace(/\s+/g, " ").trim(), play: !!e.querySelector(".re-play"), verdict: !!e.querySelector("[data-verdict]") })) }));
      const away = g.folds.filter((f) => f.via === "away"), here = g.folds.filter((f) => f.via === "here");
      const awayRuns = ll.runs.filter((r) => r.via === "away");
      check(away.length === 1 && away[0].n >= 2 && away[0].on, `the absence is one fold, \`away · N runs\`, open as the newest (${away.map((f) => f.text).join(" | ")}; ${awayRuns.length} away records)`);
      check(here.length >= 1 && !here[0].on && here[0].n >= 1, `the runs played while open fold as \`here\`, shut (${here.map((f) => f.text).join(" | ")})`);
      check(g.entries.length >= 2 && g.entries.every((e) => /^#\d+/.test(e.text) && /D\d+/.test(e.text) && /\$\d+/.test(e.text)), `entries: #, D, $ (${g.entries.slice(0, 2).map((e) => e.text).join(" | ")})`);
      check(g.entries.filter((e) => e.play).length >= 1, `the absence's runs carry ▶ (${g.entries.filter((e) => e.play).length}/${g.entries.length})`);
      // a shut fold opens on a tap
      if (here.length) { await page.locator(".runs-fold[data-via=here]").first().click(); await sleep(150); }
      const opened = await page.evaluate(() => document.querySelector(".runs-fold[data-via=here]")?.classList.contains("on"));
      check(!!opened, "a shut fold opens on a tap");
      const dv = g.entries.find((e) => e.verdict);
      if (dv) {
        await page.locator(`.run-entry[data-run="${dv.run}"] [data-verdict]`).click();
        await until(() => window.__riddle.screen === "death", "the verdict", 20_000);
        check(true, `a death's entry opens its verdict (run ${dv.run})`);
        await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
      } else out.push(`note no death in the log on this seed (the verdict link is the chronicle's code path)`);
      await page.keyboard.press("Escape"); await sleep(150);
    }
    if (part("density")) {
      await camp();
      const all = await fold();
      check(all.length <= 12, `the camp with its lane: ≤ 12 elements above the fold (${all.length}: ${all.join(" | ")})`);
      const lab = await page.evaluate(() => [...document.querySelectorAll(".lanes .ls-w, .lanes .lane-auto, .lanes .ll-l, .runs-fold .rf-w, .console .tile[data-tile=town] .tl")].map((x) => x.textContent.replace(/[↻⊘\d/]/g, "").trim()));
      check(lab.every((t) => words(t) <= 2), `lane labels ≤ 2 words (${[...new Set(lab)].join(" · ")})`);
      const tips = await page.evaluate(() => [...document.querySelectorAll(".lanes [data-kwh]")].map((x) => x.dataset.kwh));
      check(["scout"].every((t) => tips.includes(t)) && tips.some((t) => t === "live" || t === "lane") && tips.includes("log"), `the lane's terms carry tips (${[...new Set(tips)].join(", ")})`);
    }
  }

  // ---- heroes: N rows from an array (fake)
  if (part("heroes")) {
    await open("engine=fake&seed=4103&fresh=1");
    await camp();
    // (the main hero live: the lineage's run under way is his, as the core's `live` says)
    const set = (heroes) => page.evaluate(async (heroes) => { const r = window.__riddle; const e = JSON.parse(await r.engine.save()); e.lineage.heroes = heroes; e.lineage.live = heroes[0]?.state === "live" ? { run_id: 9, heir: e.lineage.heir, depth: heroes[0].depth, start: 1, hp: heroes[0].hp, max_hp: heroes[0].max_hp, turn: 120 } : null; r.lineage = await r.engine.load(JSON.stringify(e)); await r.refresh(); r.go({ kind: "camp" }); }, heroes);
    const H = (i, state) => ({ id: `h${i}`, name: ["1st heir", "Ysolde", "Brann", "the Fens"][i], state, depth: state === "live" ? 3 + i : undefined, hp: state === "live" ? 20 : undefined, max_hp: state === "live" ? 36 : undefined, rest_s: state === "rests" ? 600 : undefined, auto: true, kind: i === 3 ? "expedition" : "hero" });
    await set([H(0, "live")]); await camp();
    let s = await lanes();
    check(s.n === 1 && s.rows.length === 1 && s.more === null, `one hero: one row (${s.rows.map((r) => r.text).join(" | ")})`);
    await set([H(0, "live"), H(1, "rests"), H(2, "waits")]); await camp();
    s = await lanes();
    await shot("heroes-3");
    check(s.n === 3 && s.rows.length === 3 && s.rows.map((r) => r.state).join() === "live,rests,waits", `three heroes: three rows, each its state (${s.rows.map((r) => r.state).join(", ")})`);
    const all = await fold();
    check(all.length <= 12, `three lanes: ≤ 12 elements above the fold (${all.length}: ${all.join(" | ")})`);
    await set([H(0, "live"), H(1, "rests"), H(2, "waits"), H(3, "live")]); await camp();
    s = await lanes();
    check(s.rows.length === 3 && s.more === "1", `a fourth lane folds into \`+1\` (${s.more}; rows ${s.rows.length})`);
    // desktop: the same lanes in the centre column
    await open("engine=fake&seed=4103", { width: 1440, height: 900 });
    await set([H(0, "live"), H(1, "rests"), H(2, "waits")]); await camp();
    const d = await page.evaluate(() => { const l = document.querySelector(".lanes")?.getBoundingClientRect(); const t = document.querySelector(".town")?.getBoundingClientRect(); return l && t ? { l: [l.left, l.right, l.top], t: [t.left, t.right, t.bottom] } : null; });
    await shot("heroes-desktop");
    check(!!d && d.l[0] >= d.t[0] - 2 && d.l[1] <= d.t[1] + 2 && d.l[2] >= d.t[2] - 4, `desktop: the lanes under the town in the centre column (${JSON.stringify(d)})`);
  }
} catch (e) {
  check(false, `threw: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors.slice(0, 10)) console.log(`note ${e}`);
console.log(failed ? `runsui: ${failed} FAIL` : "runsui: all ok");
process.exit(failed ? 1 : 0);
