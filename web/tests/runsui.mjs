#!/usr/bin/env node
// Current run UI gates (docs/RUNS_UI.md §8, superseded roster: docs/UX_BLOODLINES.md).
// Real WASM at 400×800; responsive earned bloodline checks at 320/400/1440×900.
// RIDDLE_BROWSER=headed selects the GPU. runs=1 enables the actual open-app clock;
// runs=0 freezes the earned roster fixture while public purchases/selection are tested.
//   manual    empty town → manual first house → Ready resident; no premature log/auto;
//             original ≤5 main functions and ≤12 above-fold functions.
//   live      Town preserves and advances the run; hero jump resumes the same ID;
//             manual send finishes one run, reachable through Details → Run log.
//   replay    normal exit/completion, exact watched event hash, full log replay.
//   rests     scout earned through public sends/hiring; actual rest then automatic send.
//   log       actual 2h absence, away/here folds, entries, verdict and house Details.
//   density   original ≤12 above-fold functions, brief labels and bloodline/status tips.
//   heroes    earned town: paid 1→3 bloodlines, cap refusal, persistent Legacy, mobile
//             selection and desktop left roster. Display-only live/rest/ready refresh
//             changes no Rust save; exit-animation ghosts are excluded from selectors.
//
//   node web/tests/runsui.mjs [--shots dir] [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync, readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { pressWatchControl } from "../../tools/watch-control.mjs";

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
  await page.goto(`${url}?dev=1&${q.includes("runs=")?"":"runs=1&"}${q}`);
  await until(() => window.__riddle?.booted, "boot", 60_000);
}
const shot = async (name, ms = 400) => { if (shots) { await page.mouse.move(1, 300); await sleep(ms); await page.screenshot({ path: resolve(shots, `${name}.png`) }); } };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(100); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp" && !!document.querySelector(".town"), "camp", 30_000); await sleep(300); };
/** Current visible active-hero roster; wire state is rendered by the Hero row. */
const lanes = () => page.evaluate(() => {
  const root=document.querySelector(innerWidth>=1024?'.hero-desktop':'.hero-mobile');
  const rows=[...root?.querySelectorAll('.hero-row')??[]];
  return {n:window.__riddle.lineage.hero_slots.length, rows:rows.map(row=>{const title=row.querySelector('.hero-action')?.title??'',hp=/^(\d+)\/(\d+) hp/.exec(title);return {id:Number(row.dataset.slot),state:row.dataset.state,tag:row.tagName,text:row.textContent.replace(/\s+/g,' ').trim(),auto:window.__riddle.lineage.tree?.auto_send?'1':'0',gauge:hp?100*Number(hp[1])/Number(hp[2]):null};}),more:null,
    log:document.querySelector('.hero-runs')?.textContent??null,
    gem:document.querySelector('.console .gem')?.textContent.replace(/\s+/g,' ').trim()??'',pulse:!!document.querySelector('.console .gem.pulse')};
});
async function openRunLog(){
  if(!await page.locator('.runs-sheet:visible').count()){
    if(!await page.locator('.hero-sheet:visible').count())await page.locator('.hero-details:visible').first().click();
    await page.locator('.hero-runs:visible').click();
  }
  await until(()=>!!document.querySelector('.runs-sheet'),'Run log',5000);
}
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
  const send = document.querySelector("main.camp .console .gem");
  const key = (b) => send && (b === send || b.dataset.building === "mouth") ? (live ? "watch" : "send") : live && (b === live || b.dataset.building === "mouth" || (b.matches(".lane[data-state=live]") && b === document.querySelector(".lanes .lane"))) ? "watch"
    : b.classList.contains("hero-details") || b.dataset.building === "tent" ? "hero-details"   // The house and Details open the same hero menu.
    : b.classList.contains("next-pill") || b.dataset.building === "worker" ? "works" : b.dataset.building ?? (b.dataset.tile ? SAME[b.dataset.tile] ?? `tile:${b.dataset.tile}` : (b.getAttribute("aria-label") || b.textContent || b.className).replace(/\s+/g, " ").trim().slice(0, 24));
  return [...new Set(els.map(key))];
}, [SAME, scope]);
/** send by the gem; resolves on the watch with the run's id (the engine's `send` answer, tapped) */
async function sendWatch() {
  await page.evaluate(() => { const r = window.__riddle; if (r.__sendTap) return; const o = r.engine.send.bind(r.engine); r.engine.send = async (...args) => { const s = await o(...args); window.__sentRun = s.run.id; return s; }; r.__sendTap = true; });
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
  // ---- current manual phase: an empty town, then a house and a waiting resident.
  if (part("manual") || part("live") || part("replay")) {
    await open(`seed=${Number(process.env.RUNSUI_SEED ?? 4101)}&fresh=1`);
    await camp();
    const empty=await page.evaluate(()=>({home:window.__riddle.lineage.town.home,heroes:window.__riddle.lineage.hero_slots.length,send:document.querySelector('.console .gem')?.disabled,build:!!document.querySelector('.town-tag.build-marker[role="button"]:not([hidden])')}));
    check(empty.home===false&&empty.heroes===0&&empty.send===true&&empty.build, `day 0 starts empty, requires construction, and cannot send: ${JSON.stringify(empty)}`);
    const emptySurfaces=await fold(":is(.well-wrap, .console)"),emptyAll=await fold();
    check(emptySurfaces.length<=5&&emptyAll.length<=12, `empty town retains ≤5 main surfaces and ≤12 above-fold controls (${emptySurfaces.length}/${emptyAll.length})`);
    await page.locator('.town-tag[data-next="house"]').click();
    await until(()=>window.__riddle.lineage.town.home===true&&window.__riddle.lineage.hero_slots.length===1&&!document.querySelector('.console .gem')?.disabled,"the first resident",15000);
    const s=await page.evaluate(()=>{const root=document.querySelector('.hero-mobile'),row=root.querySelector('.hero-row');return {state:row?.dataset.state,action:row?.querySelector('.hero-action')?.textContent,xp:row?.querySelector('.hero-xp')?.textContent,details:!!row?.querySelector('.hero-details'),tip:row?.querySelector('[data-kwh]')?.dataset.kwh,gem:document.querySelector('.console .gem')?.textContent,pulse:!!document.querySelector('.console .gem.pulse'),log:!!document.querySelector('.hero-runs')};});
    const l=await L();
    await shot("manual-day0");
    check(s.state==='waits'&&s.action==='Ready'&&s.details&&/XP/.test(s.xp), `the first resident is Ready with XP and Details (${JSON.stringify(s)})`);
    check(!l.auto&&l.waits, `before the scout sending remains manual (auto ${l.auto}, waits ${l.waits})`);
    check(s.pulse&&/send/i.test(s.gem), `the first Send is lit (${s.gem})`);
    check(!s.log&&l.runs.length===0, "an unplayed hero has no Run log");
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
    check(townTile === "Town menu", `the watch carries \`town\` (${townTile})`);
    await page.locator(".console .tile[data-tile=town]").click();
    await camp();
    const a = await until(() => { const l = window.__riddle.lineage.live; return l && l.turn > 0 ? l : null; }, "the run live in the town", 15_000);
    const s = await lanes();
    await shot("live-town");
    check(a.run_id === run && s.rows[0]?.state === "live" && /D\d+ · (Exploring|In combat|Heading home|Delving)/.test(s.rows[0].text) && Number(s.rows[0].gauge) > 0, `left mid-run: the lane reads live with his floor and hp (${s.rows[0]?.text}; run ${a.run_id} = ${run})`);
    check(/watch/i.test(s.gem), `a run under way: the gem reads \`watch\` (${s.gem})`);
    const b = await until((t0) => { const l = window.__riddle.lineage.live; return l && l.turn > t0 + 10 ? l : (window.__riddle.lineage.runs ?? []).some((r) => r.id === window.__riddle.lineage.live?.run_id) ? l : null; }, "the run's tick moving in the town", 20_000, a.turn);
    check(b.turn > a.turn, `leaving the watch keeps the run going: tick ${a.turn} → ${b.turn} in the town`);
    // back into the same run from the lane
    await page.evaluate(() => { window.__sentRun = undefined; });
    await page.locator(".hero-row[data-state=live]:visible .hero-jump").first().click();
    await until(() => window.__riddle.screen === "watch" && window.__sentRun !== undefined, "the watch from the lane", 20_000);
    const again = await page.evaluate(() => window.__sentRun);
    check(again === run, `the lane opens the watch on the run in flight (run ${again} = ${run})`);
    await until(() => !!document.querySelector(".console .tile[data-tile=town]:not([disabled])"), "the town tile again", 15_000);
    await page.locator(".console .tile[data-tile=town]").click();
    await camp();
    // (the town plays it on at the clock's pace; a run that no longer ends at its record can take many minutes — the clock jumps ahead:
    // the same unwatched `advance` the town's runner makes, in long steps)
    await sleep(3000);
    await page.evaluate(async (id) => { const r = window.__riddle; for (let k = 0; k < 900; k++) { const a = await r.engine.advance(20_000); if (a.ended.includes(id) || !a.live) break; } await r.refresh(); }, run);
    const rec = await until((id) => (window.__riddle.lineage.runs ?? []).find((r) => r.id === id) ?? null, "the run's end in the log", 60_000, run);
    await sleep(1200);
    await page.locator(".hero-details:visible").first().click();
    const s2 = await lanes(), l2 = await L();
    await shot("live-ended");
    check(rec.via === "town", `the run left mid-watch ended in the town: the log holds it as \`town\` (${rec.via}, D${rec.depth}, ${rec.tier})`);
    check(s2.rows[0]?.state === "waits" && !l2.auto, `before the scout one send is one run: the lane waits again (${s2.rows[0]?.text})`);
    check(s2.log !== null && /log/i.test(s2.log) && l2.runs.filter((r) => r.id > 0).length === 1, `the log from run 1: one entry, the stud shown (${s2.log})`);
    await openRunLog();
    check(await page.locator(`.runs-sheet .run-entry[data-run="${run}"]`).count()===1,"the completed town run is reachable through Hero Details → Run log");
    await page.keyboard.press("Escape");await page.keyboard.press("Escape");
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
    check(await pressWatchControl(page,"fast"),"Fast is selected through the visible Speed menu");
    // the run to its end (▶▶| as a player would), the exit sheet kept, the report, the camp
    for (let i = 0; i < 600; i++) {
      const sc = await page.evaluate(() => window.__riddle.screen);
      if (sc === "camp") break;
      if(sc==='watch'){
        const next=page.locator('.watch .next-gem:visible');
        if(await next.count())await next.click();else await pressWatchControl(page,'▶▶|');
      }else if(sc==='exit'){
        await page.locator('.sheet-wrap button.btn.primary.wide').first().click();
      }else if(sc==='death')await page.locator('.console [data-tile="camp"]').click();
      else if(sc==='report')await page.locator('.console .camp-gem').click();
      await sleep(400);
    }
    await page.evaluate(() => { window.__tapOn = false; });
    check(await page.evaluate(()=>window.__riddle.screen==="camp"&&!window.__riddle.lineage.live),"the watched run finishes through the normal exit flow before replay inspection");
    await camp();
    const live = await page.evaluate(new Function(`return (${HASH})(window.__evs)`));
    const rep = await page.evaluate(async (id) => { const x = await window.__riddle.engine.replay(id); return x && { hash: x.hash, floors: x.floors.length, n: x.floors.reduce((a, f) => a + f.events.length, 0), evs: x.floors.flatMap((f) => f.events) }; }, run);
    const replayHash = rep ? await page.evaluate(new Function("evs", `return (${HASH})(evs)`), rep.evs) : null;
    const nLive = await page.evaluate(() => { const i = window.__evs.findIndex((e) => e.k === "exit"); return i < 0 ? window.__evs.length : i + 1; });
    check(!!rep && replayHash === live, `the watched run (${nLive} events) replays identically: ${live} = ${replayHash} (core's ${rep?.hash}, ${rep?.floors} floors)`);
    // the log's ▶ on it
    await openRunLog();
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
    await page.locator('.town-tag[data-next="house"]').click();
    await until(()=>window.__riddle.lineage.town.home===true&&window.__riddle.lineage.hero_slots.length===1&&!document.querySelector('.console .gem')?.disabled,"the first resident before hiring",15000);
    // Earn scout access through actual manual sends and paid public hire actions.
    await page.evaluate(async () => { const r = window.__riddle; for (let i = 0; i < 3 && !r.lineage.tree.auto_send; i++) { try { r.lineage = await r.engine.hire("scout"); } catch { /* not lit: send by hand */ break; } } });
    let l = await L();
    for (let i = 0; i < 8 && !l.auto; i++) {
      // the sends by hand (the trunk's), each played out unwatched by the open app's clock; the chest opened, the lit worker hired
      await page.evaluate(async () => { const r = window.__riddle; await r.engine.send(); for (let k = 0; k < 600; k++) { const a = await r.engine.advance(20_000); if (a.ended.length) break; } try { await r.engine.openChest(); } catch { /* none */ } r.lineage = await r.engine.lineage(); for (let j = 0; j < 2; j++) { try { r.lineage = await r.engine.hire(r.lineage.tree.lit ?? ""); } catch { /* not yet */ } } r.go({ kind: "camp" }); });
      l = await L();
    }
    check(l.auto, `the scout hired by the trunk's sends (auto ${l.auto})`);
    await camp();
    // a run ends; home, he rests
    await page.evaluate(async () => { const r = window.__riddle; await r.engine.send(); for (let k = 0; k < 400; k++) { const a = await r.engine.advance(20_000); if (a.ended.length) break; } r.lineage = await r.engine.lineage(); r.go({ kind: "camp" }); });
    await camp();
    const s = await lanes();
    await shot("rests");
    check(s.rows[0]?.state === "rests" && /Resting \d+[sm]/.test(s.rows[0].text) && s.rows[0].auto === "1", `home after a run: \`rests Nm\` with \`↻ auto\` lit (${s.rows[0]?.text})`);
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
      await openRunLog();
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
      // the hero's tent keeps his log too
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); });
      await page.locator('.town-hit[data-building="tent"]').click();
      const heroFromTent=await until(()=>!!document.querySelector('.hero-sheet'),'Hero Details from the house',5000).catch(()=>false);
      if(heroFromTent)await page.locator('.hero-runs:visible').click();
      const viaTent = await until(() => !!document.querySelector(".runs-sheet"), "Run log from house Details", 5000).catch(() => false);
      check(heroFromTent&&viaTent, "the hero's house opens Details and its Run log");
      // an entry opens the run's card (run-clear), its ▶ under it
      const anyRun = g.entries.find((e) => !e.verdict);
      if (viaTent && anyRun) {
        await page.locator(`.run-entry[data-run="${anyRun.run}"] .re-body`).click();
        const card = await until(() => document.querySelector(".run-card-sheet .run-clear")?.dataset.end ?? null, "the run's card", 5000).catch(() => null);
        await shot("run-card");
        check(!!card, `an entry opens the run's card (run ${anyRun.run}: ${card})`);
      }
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); });
    }
    if (part("density")) {
      await camp();
      const all = await fold();
      check(all.length <= 12, `the camp with its lane: ≤ 12 elements above the fold (${all.length}: ${all.join(" | ")})`);
      const lab = await page.evaluate(() => [...document.querySelectorAll('.hero-mobile .hero-action, .hero-mobile .hero-details, .hero-mobile .hero-expand')].map(x=>x.textContent));
      check(lab.length>=3&&lab.every(t=>words(t)<=2), `active hero action labels ≤2 words (${[...new Set(lab)].join(' · ')})`);
      const tips = await page.evaluate(() => ({terms:[...document.querySelectorAll('.hero-mobile [data-kwh]')].map(x=>x.dataset.kwh),health:document.querySelector('.hero-mobile .hero-action')?.title}));
      check(tips.terms.includes('bloodline')&&!!tips.health, `the bloodline tooltip and current status detail remain available (${JSON.stringify(tips)})`);
    }
  }

  // ---- heroes: one and three actual paid bloodlines, mobile selection and desktop roster.
  if (part("heroes")) {
    const engine=readFileSync(resolve(ROOT,'web/tests/fixtures/earned-gunner-home.json'),'utf8');
    for(const width of [320,400,1440]){
      await open('seed=4103&fresh=1&runs=0',{width,height:900});
      check(await page.evaluate(async engine=>window.__riddle.importSave(JSON.stringify({v:2,engine,loadout:[],last_seen:Date.now(),runs:0})),engine),'actual earned save imported without grants');
      await camp();
      let s=await lanes();
      check(s.n===1&&s.rows.length===1,`${width}: one active bloodline renders one hero`);
      if(width<1024)await page.locator('.hero-mobile .hero-expand').click();
      const root=width<1024?'.sheet-wrap:not([hidden]) .heroes-sheet':'.hero-desktop';
      for(const expected of [2,3]){
        const before=await page.evaluate(()=>({gold:window.__riddle.lineage.gold,price:window.__riddle.lineage.bloodline_price}));
        await page.locator(`${root} .hero-add`).click();
        await until(n=>window.__riddle.lineage.hero_slots.length===n,'paid bloodline founded',15000,expected);
        const after=await page.evaluate(()=>window.__riddle.lineage.gold);
        check(after===before.gold-before.price,`${width}: bloodline ${expected} pays its actual $${before.price} price`);
        if(width<1024)check(await page.locator('.sheet-wrap:not([hidden]) .heroes-sheet .row-label').evaluateAll(nodes=>nodes[0]?.textContent??null)==='Active heroes',`${width}: paid founding preserves the Active heroes heading`);
      }
      const rows=await page.locator(`${root} .hero-row`).evaluateAll(rows=>rows.map(row=>({id:Number(row.dataset.slot),state:row.dataset.state,name:row.querySelector('.hero-info b')?.textContent,xp:row.querySelector('.hero-xp')?.textContent,icon:!!row.querySelector('.class-icon'),face:!!row.querySelector('.hero-thumb'),details:!!row.querySelector('.hero-details')})));
      const slots=await page.evaluate(()=>window.__riddle.lineage.hero_slots);
      check(rows.length===3&&rows.every(row=>slots.some(slot=>slot.id===row.id&&slot.state===row.state)&&row.name&&row.xp&&row.icon&&row.face&&row.details),`${width}: all three actual active slots retain identity, state, XP, icon, portrait and Details`);
      check(await page.locator(`${root} .hero-add`).count()===0,`${width}: full roster offers no fourth bloodline`);
      const refused=await page.evaluate(async()=>{const a=window.__riddle,before=await a.engine.save();let failed=false;try{await a.engine.addBloodline();}catch{failed=true;}return failed&&(await a.engine.save())===before;});
      check(refused,`${width}: fourth bloodline is refused with exact save preservation`);
      if(width===400){
        const observed=await page.evaluate(async()=>{
          const a=window.__riddle,before=await a.engine.save();
          const slots=a.lineage.hero_slots;
          // Controlled presentation only: Rust state/save is never edited.
          slots[0].state='live';slots[0].live={run_id:7,heir:slots[0].heir,depth:13,start:1,hp:20,max_hp:36,turn:100,activity:'combat'};
          slots[1].state='rests';slots[1].rest_s=600;slots[2].state='waits';
          a.emitLive();
          const rowStates=[...document.querySelectorAll('.sheet-wrap:not([hidden]) .heroes-sheet .hero-row')].map(row=>row.dataset.state);
          const text=[...document.querySelectorAll('.sheet-wrap:not([hidden]) .heroes-sheet .hero-action')].map(row=>row.textContent);
          const unchanged=(await a.engine.save())===before;
          await a.refresh();
          return {rowStates,text,unchanged};
        });
        check(observed.rowStates.join()==='live,rests,waits'&&observed.text.join()==='D13 · In combat,Resting 10m,Ready'&&observed.unchanged,`open Heroes tracks live/rest/ready wire changes without gameplay mutation: ${JSON.stringify(observed)}`);
      }
      await shot(`heroes-${width}`);
      if(width<1024){await page.locator('.sheet-wrap:has(.heroes-sheet) .close-stud:visible').click();await page.locator('.sheet-wrap:has(.heroes-sheet)').waitFor({state:'detached'});await page.locator('.hero-mobile .hero-expand').click();await page.locator('.sheet-wrap:not([hidden]) .heroes-sheet [data-slot="3"] .hero-jump').click();}
      else await page.locator('.hero-desktop [data-slot="3"] .hero-jump').click();
      await until(()=>window.__riddle.lineage.selected_bloodline===3,'hero row selects its bloodline',15000);
      const selected=await page.evaluate(()=>window.__riddle.lineage.hero_slots.find(slot=>slot.id===3));
      check((await lanes()).rows.some(row=>row.id===3&&row.state===selected.state&&row.text.includes('Ready')),`${width}: jump selects bloodline 3 with its actual ${selected.state} state and Ready action`);
      if(width<1024)await page.locator('.hero-mobile .hero-expand').click();
      await page.locator(`${root} [data-slot="2"] .hero-details`).click();
      await until(()=>!!document.querySelector('.hero-sheet')&&window.__riddle.lineage.selected_bloodline===2,'Details selects the requested bloodline',15000);
      check(await page.locator('.sheet-wrap:not([hidden]) .hero-sheet .row-label').textContent()==='Bloodline 2',`${width}: Details opens the correct bloodline menu`);
      await page.locator('.sheet-wrap:has(.hero-sheet) .close-stud:visible').click();
      await page.locator('.sheet-wrap:has(.hero-sheet)').waitFor({state:'detached'});
      check(await page.evaluate(expected=>JSON.stringify(window.__riddle.lineage.hero_slots.map(({id,legacy})=>({id,legacy})))===expected,JSON.stringify(slots.map(({id,legacy})=>({id,legacy})))),`${width}: selecting heroes preserves each bloodline's Legacy`);
      const geometry=await page.evaluate(()=>{const roster=document.querySelector('.hero-desktop').getBoundingClientRect(),town=document.querySelector('.town').getBoundingClientRect();return {left:roster.right<=town.left+1,overflow:document.documentElement.scrollWidth>innerWidth};});
      check(!geometry.overflow&&(width<1024||geometry.left),`${width}: mobile stays compact and desktop heroes occupy the left panel (${JSON.stringify(geometry)})`);
      if(width<1024){const all=await fold();check(all.length<=12,`${width}: three bloodlines retain ≤12 above-fold functions (${all.length}: ${all.join(' | ')})`);}
    }
  }
} catch (e) {
  check(false, `threw: ${e.message}`);
}
await browser.close();
if(errors.length)check(false,`browser emitted ${errors.length} errors`);
for (const l of out) console.log(l);
for (const e of errors.slice(0, 10)) console.log(`note ${e}`);
console.log(failed ? `runsui: ${failed} FAIL` : "runsui: all ok");
process.exit(failed ? 1 : 0);
