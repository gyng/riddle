#!/usr/bin/env node
// Graphics / fx / animation / UI eval harness (docs/JUICE.md §10). Headed Chromium on the GPU (tools/browser.mjs launchGpu), the same
// walk every round, against a dev server:
//
//   node tools/gfx-eval.mjs --out scratchpad/gfx-eval/round0 [--port 5461] [--only camp,boss] [--no-desktop] [--no-phone]
//
// Per moment it writes `<name>.png` (the screen), `<name>-strip.png` (4 frames 180 ms apart, half size: the motion) and records
//   - frame times: rAF interval p50 / p95 / p99 over ~3 s (the viewer's own cpuMs too where a run is mounted) → frames.json
//   - animation timings: every CSS animation / transition running right after the moment's trigger (target, name, duration,
//     delay, easing) → anims.json
//   - layout: the console, the portrait and the primary gem on screen and uncovered (elementFromPoint at the gem's centre and
//     corners), nothing wider than the viewport → layout.json (the owner's IA check (a))
// and a `rater/` folder: every shot and strip under a neutral name (m01.png, m01-strip.png …) with `moments.md` (what each is, for
// the scorer — never shown to a rater) and `prompt.txt` (the blind rater's brief). Audio: tools/gfx-audio.mjs.
import { mkdirSync, writeFileSync, readFileSync, copyFileSync } from "node:fs";
import { join, resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { execFileSync } from "node:child_process";
import { launchGpu } from "./browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "..");
const arg = (n, d) => { const i = process.argv.indexOf(n); return i > 0 ? process.argv[i + 1] : d; };
const out = resolve(arg("--out", "scratchpad/gfx-eval/round0"));
const port = arg("--port", process.env.RIDDLE_PORT ?? "5219");
const only = arg("--only", "")?.split(",").filter(Boolean) ?? [];
const want = (n) => !only.length || only.some((o) => n.startsWith(o));
const BASE = `http://localhost:${port}/`;
// gfx round 7: extra dev query params for every app page (`--q "sprite=0.5&texels=66"`) and for render-demo.html (`--qdemo`)
const XQ = arg("--q", ""), XQD = arg("--qdemo", "");
mkdirSync(out, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const frames = [], anims = {}, layout = {}, shots = [];
const log = (...a) => console.log(...a);

const browser = await launchGpu();

// ---- helpers --------------------------------------------------------------------------------------------------------------------
async function newPage(w, h, dpr) {
  const page = await browser.newPage({ viewport: { width: w, height: h }, deviceScaleFactor: dpr });
  page.__errs = [];
  page.on("pageerror", (e) => page.__errs.push(e.message));
  page.on("console", (m) => { if (m.type() === "error") page.__errs.push(m.text()); });
  return page;
}
const state = (page) => page.evaluate(() => { const r = window.__riddle, w = document.querySelector(".watch"); return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, frame: w?.dataset.frame, boss: w?.dataset.boss ?? "", mode: w?.dataset.mode, ending: w?.dataset.ending === "1" } : null; }).catch(() => null);
async function waitFor(page, pred, label, timeout = 60000) {
  const t = Date.now(); let s;
  while (Date.now() - t < timeout) { s = await state(page); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout ${label} (${s?.screen})`);
}
async function settle(page, max = 15000) { const t = Date.now(); let c = 0; while (Date.now() - t < max) { const s = await state(page); c = s && s.booted && !s.busy ? c + 1 : 0; if (c >= 3) break; await sleep(200); } await sleep(500); }
const goto = (page, q) => page.goto(`${BASE}?dev=1&${q}${XQ ? `&${XQ}` : ""}`, { waitUntil: "domcontentloaded" });
const pct = (a, p) => { const s = [...a].filter(Number.isFinite).sort((x, y) => x - y); return s.length ? +s[Math.min(s.length - 1, Math.floor(p * s.length))].toFixed(2) : null; };

/** rAF intervals (and the viewer's cpuMs) for `ms`. */
async function sampleFrames(page, name, ms = 3000) {
  const r = await page.evaluate(async (ms) => {
    const v = window.__viewer; const it = [], cpu = [];
    let last = performance.now(); const t0 = last;
    await new Promise((res) => { const f = (now) => { it.push(now - last); last = now; const s = v?.stats?.(); if (s && Number.isFinite(s.cpuMs)) cpu.push(s.cpuMs); if (now - t0 < ms) requestAnimationFrame(f); else res(); }; requestAnimationFrame(f); });
    const s = v?.stats?.() ?? {};
    return { it: it.slice(2), cpu, fx: s.fx ?? "-", calls: s.calls ?? null, k: s.k ?? null, dpr: devicePixelRatio, glLost: s.glLost ?? false };
  }, ms);
  const row = { name, p50: pct(r.it, 0.5), p95: pct(r.it, 0.95), p99: pct(r.it, 0.99), cpuP50: pct(r.cpu, 0.5), cpuP95: pct(r.cpu, 0.95), n: r.it.length, fx: r.fx, calls: r.calls, k: r.k, dpr: r.dpr };
  frames.push(row); log("frames", JSON.stringify(row));
  return row;
}
/** CSS animations/transitions alive now (their timing, not their state). */
async function animsNow(page, name) {
  const a = await page.evaluate(() => document.getAnimations().map((x) => {
    const t = x.effect?.getTiming?.() ?? {}, el = x.effect?.target;
    const cls = el ? `${el.tagName?.toLowerCase() ?? ""}.${String(el.className?.baseVal ?? el.className ?? "").split(/\s+/).filter(Boolean).slice(0, 3).join(".")}` : "?";
    return { target: cls + (x.effect?.pseudoElement ?? ""), name: x.animationName ?? x.transitionProperty ?? x.constructor.name, duration: typeof t.duration === "number" ? Math.round(t.duration) : t.duration, delay: Math.round(t.delay ?? 0), easing: t.easing, iterations: t.iterations };
  }).filter((x) => x.duration !== 0).slice(0, 40)).catch(() => []);
  anims[name] = [...(anims[name] ?? []), ...a];
}
/** The owner's IA check: the console, portrait and gem on screen; the gem's centre and corners hit the gem (nothing covers it). */
async function checkLayout(page, name) {
  const r = await page.evaluate(() => {
    const vis = (el) => { if (!el) return null; const b = el.getBoundingClientRect(); return { x: Math.round(b.x), y: Math.round(b.y), w: Math.round(b.width), h: Math.round(b.height), on: b.width > 4 && b.height > 4 && b.bottom <= innerHeight + 1 && b.top >= -1 && b.right <= innerWidth + 1 && b.left >= -1 }; };
    const con = [...document.querySelectorAll("footer.console")].find((e) => e.offsetParent !== null) ?? null;
    const gem = con?.querySelector(".gem-slot .gem, .gem-slot button") ?? null;
    const por = con?.querySelector(".well-slot > *") ?? null;
    let covered = [];
    if (gem) { const b = gem.getBoundingClientRect(); for (const [fx, fy] of [[0.5, 0.5], [0.3, 0.3], [0.7, 0.3], [0.3, 0.7], [0.7, 0.7]]) { const x = b.left + b.width * fx, y = b.top + b.height * fy; const hit = document.elementFromPoint(x, y); if (hit && !gem.contains(hit) && hit !== gem) covered.push(`${hit.tagName.toLowerCase()}.${String(hit.className?.baseVal ?? hit.className ?? "").split(/\s+/).slice(0, 2).join(".")}`); } }
    return { console: vis(con), gem: vis(gem), portrait: vis(por), gemCoveredBy: [...new Set(covered)], scrollW: document.documentElement.scrollWidth, innerW: innerWidth };
  }).catch((e) => ({ error: String(e) }));
  r.ok = !!(r.console?.on && r.gem?.on && r.portrait?.on && !r.gemCoveredBy?.length && r.scrollW <= r.innerW + 1);
  layout[name] = r; if (!r.ok) log("layout!", name, JSON.stringify(r));
}
/** A shot, a 4-frame strip, and the layout check. */
/** The strip: 4 frames 180 ms apart, taken from the compositor's screencast (CDP `Page.startScreencast`: every presented frame with its
 *  timestamp), so the spacing is true however slow a screenshot is on a loaded machine. `trigger` (a tap) runs once the cast is live:
 *  frame 0 is the tap's moment, frames 1–3 its opening. Without a trigger the strip is the screen's own motion from now. */
async function castStrip(page, name, trigger = null) {
  const cdp = await page.context().newCDPSession(page);
  const got = [];
  cdp.on("Page.screencastFrame", (f) => { got.push({ t: f.metadata.timestamp * 1000, data: f.data }); cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {}); });
  // (half-size JPEG frames: a full-size PNG took ~100 ms to encode on a loaded machine and the frames arrived after the window closed)
  const vp = page.viewportSize();
  await cdp.send("Page.startScreencast", { format: "jpeg", quality: 85, maxWidth: vp.width, maxHeight: vp.height, everyNthFrame: 1 });
  await sleep(150);
  const start = Date.now(); if (trigger) await trigger();
  await sleep(1400);   // the window is 540 ms; the rest lets the last frames arrive
  await cdp.send("Page.stopScreencast").catch(() => {});
  await cdp.detach().catch(() => {});
  const base = join(out, name);
  for (let i = 0; i < 4; i++) {
    const want = start + i * 180; let best = got[0];
    for (const f of got) if (Math.abs(f.t - want) < Math.abs((best?.t ?? 0) - want)) best = f;
    if (best) writeFileSync(`${base}-f${i}.jpg`, Buffer.from(best.data, "base64"));
    else await page.screenshot({ path: `${base}-f${i}.png` });
  }
  return got.length;
}
const stripNow = (page, name) => castStrip(page, name);
async function shoot(page, name, what, { strip = true, full = false, lay = true, pre = false } = {}) {
  const base = join(out, name);
  if (strip && !pre) await castStrip(page, name);
  await page.screenshot({ path: `${base}.png`, fullPage: full });
  if (lay) await checkLayout(page, name);
  shots.push({ name, what, strip });
  log("shot", name);
}

// ---- the phone walk: the real engine ---------------------------------------------------------------------------------------------
const RICH = JSON.stringify({ rows: [
  { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } },
  { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" } },
  { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "boss" } },
  { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" } } ] });
const press = (page, id) => page.evaluate((id) => { const b = document.querySelector(`button[data-tile="${id}"]`); if (b) { b.click(); return true; } return false; }, id);
const tileClick = async (page, id) => { const l = page.locator(`button[data-tile="${id}"]`).first(); if (await l.count()) { await l.click({ timeout: 4000 }).catch(() => {}); return true; } return false; };
const closeSheets = async (page) => { for (let i = 0; i < 3; i++) { const x = page.locator(".sheet-x:visible, .close-stud:visible").first(); if (await x.count()) await x.click({ timeout: 2000 }).catch(() => {}); else await page.keyboard.press("Escape"); await sleep(250); } };
const step = async (label, fn) => { try { await fn(); } catch (e) { log(`skip ${label}: ${e.message.split("\n")[0]}`); } };
async function toEnd(page, timeout = 150000) {
  const t = Date.now(); let s;
  while (Date.now() - t < timeout) { s = await state(page); if (s?.screen !== "watch") break; await press(page, "skip"); await sleep(120); }
  s = await state(page);
  if (s?.screen === "exit") { await settle(page); await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 1500 }).catch(() => {}); await page.locator(".sheet-wrap .btn.primary").first().click({ timeout: 3000 }).catch(() => {}); await waitFor(page, (x) => x && x.screen !== "watch" && x.screen !== "exit", "after exit"); }
  await settle(page); return state(page);
}
async function measuredDeath(page) { const t = Date.now(); while (Date.now() - t < 25000) { const g = await page.locator(".patch-gem").first().textContent().catch(() => ""); if (!/…|measuring/i.test(g ?? "")) break; await sleep(300); } await sleep(400); }

async function phoneReal() {
  const page = await newPage(400, 800, 2);
  await goto(page, `fresh=1&seed=4242`);
  await waitFor(page, (s) => s?.booted && s.screen === "camp", "camp"); await settle(page);
  // one real run, watched at normal (1×) in the Warrens
  await step("watch-warrens", async () => {
    await page.locator("button.send, .gem-slot .gem").first().click();
    await waitFor(page, (s) => s?.screen === "watch", "watch");
    await sleep(800); await tileClick(page, "one"); await sleep(2500);
    if (want("watch-warrens")) { await sampleFrames(page, "watch-warrens", 3000); await shoot(page, "watch-warrens", "the watch at normal speed, D1 the Warrens (real engine)"); }
    await toEnd(page);
  });
  // the absence → the report
  await goto(page, `absent=8h`);
  await waitFor(page, (s) => s?.booted && (s.screen === "report" || s.screen === "ending"), "report", 240000);
  if (want("report")) { await animsNow(page, "report"); await stripNow(page, "report"); }
  await settle(page);
  if (want("report")) { await sampleFrames(page, "report", 2000); await shoot(page, "report", "the offline report after 8 h away (strip: its arrival)", { pre: true }); }
  await step("death", async () => {
    if (!(await page.locator(`button[data-tile="open"]`).count())) throw new Error("no worst death");
    await tileClick(page, "open");
    await waitFor(page, (s) => s?.screen === "death", "death"); await animsNow(page, "death");
    if (want("death")) await stripNow(page, "death");
    await settle(page); await measuredDeath(page);
    if (want("death")) { await sampleFrames(page, "death", 2000); await shoot(page, "death", "the death screen, the worst death of the absence (strip: its arrival)", { pre: true }); }
    await tileClick(page, "camp"); await waitFor(page, (s) => s?.screen === "camp", "camp");
  });
  await settle(page);
  if ((await state(page))?.screen === "report") { await page.locator("main.report .gem").first().click().catch(() => {}); await waitFor(page, (s) => s?.screen === "camp", "camp"); await settle(page); }
  await sleep(2000); await settle(page);
  if (want("camp")) { await sampleFrames(page, "camp", 2000); await shoot(page, "camp", "the camp: rules, the depth shaft, the console"); }
  // an edit: the rules change, the forecast re-prices, the divergence scene plays over the well
  await step("scene", async () => {
    await page.evaluate((t) => window.__riddle.setRulesText(t), RICH);
    const t = Date.now(); let got = false;
    while (Date.now() - t < 30000) { if (await page.locator(".div-scene:not([hidden])").count()) { got = true; break; } await sleep(100); }
    if (got && want("scene")) { await sleep(700); await animsNow(page, "scene"); await shoot(page, "scene", "a rule edited: the before/after scene over the camp's well"); }
    else if (want("scene")) { await settle(page); await shoot(page, "scene", "a rule edited: the camp after the edit (no scene played)"); }
    await sleep(4000); await settle(page);
  });
  await step("edit", async () => {
    const r = page.locator("main.camp .row.tablet:not(.oath-tab)").first();
    // (the tap as a DOM click: a Playwright click's actionability waits cost ~0.3 s under load and pushed the opening out of the window)
    const tap = () => r.evaluate((e) => e.click()).catch(() => {});
    if (want("edit")) await castStrip(page, "edit", tap); else await tap();
    await animsNow(page, "edit"); await settle(page);
    if (want("edit")) await shoot(page, "edit", "a rule tablet tapped: the rule editor (strip: its opening)", { pre: true });
    await closeSheets(page);
  });
  await step("forecast", async () => {
    await closeSheets(page); const sh = page.locator(".shaft").first();
    const tap = () => sh.evaluate((e) => e.click()).catch(() => {});
    if (want("forecast")) await castStrip(page, "forecast", tap); else await tap();
    await animsNow(page, "forecast"); await settle(page);
    if (want("forecast")) await shoot(page, "forecast", "the forecast, the depth shaft opened (strip: its opening)", { pre: true });
    await closeSheets(page);
  });
  await step("oaths", async () => {
    await closeSheets(page); const t = page.locator(".oath-tab").first();
    const tap = () => t.evaluate((e) => e.click()).catch(() => {});
    if (want("oaths")) await castStrip(page, "oaths", tap); else await tap();
    await animsNow(page, "oaths"); await settle(page);
    if (want("oaths")) await shoot(page, "oaths", "the oath board (strip: its opening)", { pre: true });
    await closeSheets(page);
  });
  if (page.__errs.length) log("page errors (real):", page.__errs.slice(0, 5));
  await page.close();
}

// ---- the fake engine: a deeper biome, a fight, a boss -------------------------------------------------------------------------------
async function phoneFake(name, depth, what, boss = false) {
  const page = await newPage(400, 800, 2);
  await goto(page, `engine=fake&fake_depth=${depth}&fake_god=1&fresh=1&seed=77&autosend=1`);
  await waitFor(page, (s) => s?.screen === "watch", "watch");
  await sleep(600);
  if (!boss) {
    await tileClick(page, "one"); await sleep(depth > 3 ? 1500 : 2500);
    if (depth > 3 && want(name) && (await state(page))?.screen === "watch") { await shoot(page, name, what); await sampleFrames(page, name, 600); }
    // a fight at 1×: the fight frame
    if (want("fight") && depth <= 3) {
      const t = Date.now(); while (Date.now() - t < 40000 && (await state(page))?.frame !== "fight") { await press(page, "skip"); await sleep(400); }
      if ((await state(page))?.frame === "fight") { await sleep(700); await sampleFrames(page, "fight", 2500); await shoot(page, "fight", "a fight (the fight frame, highlights mode)"); }
    }
  } else {
    // the boss: entrance (in view), break, fall — highlights plays fights at 1.5×
    const beats = () => page.evaluate(() => (window.__beatLog ?? []).map((b) => b.text).join("|"));
    let t = Date.now();
    while (Date.now() - t < 60000 && !(await state(page))?.boss) { await press(page, "skip"); await sleep(500); }
    if ((await state(page))?.boss) { await sleep(250); await sampleFrames(page, "boss-in", 2000); await shoot(page, "boss-in", "a boss comes into view (the entrance)"); }
    t = Date.now(); let broke = false;
    while (Date.now() - t < 60000) { const b = await beats(); if (/break/i.test(b)) { broke = true; break; } const s = await state(page); if (s?.screen !== "watch" || s.ending) break; await sleep(60); }
    if (broke) { await shoot(page, "boss-break", "the boss breaks (his guard shattered)"); }
    t = Date.now(); let fell = false;
    while (Date.now() - t < 60000) { const b = await beats(); if (/down|slain/i.test(b)) { fell = true; break; } const s = await state(page); if (s?.screen !== "watch") break; await sleep(60); }
    if (fell) { await sampleFrames(page, "boss-fall", 1500); await shoot(page, "boss-fall", "the boss falls (the killing blow)"); }
  }
  if (page.__errs.length) log(`page errors (${name}):`, page.__errs.slice(0, 5));
  await page.close();
}

// ---- a real save (gfx round 7): web/tests/fixtures/deep.json (a D11 lineage, waystones D5 burrows · D9 fens) — the real engine at depth,
// where the fake engine's crowds piled on the hero. `start` the waystone the send starts from; `fight`: wait for the fight frame (highlights)
async function phoneDeep(name, start, what, fight = false) {
  const page = await newPage(400, 800, 2);
  await goto(page, `fresh=1&seed=2501`);
  await waitFor(page, (s) => s?.booted && s.screen === "camp", "camp");
  const engine = readFileSync(join(ROOT, "web/tests/fixtures/deep.json"), "utf8").trim();
  await page.evaluate((engine) => window.__riddle.importSave(JSON.stringify({ v: 2, engine, loadout: [], last_seen: Date.now(), runs: 0 })), engine);
  await waitFor(page, (s) => s?.booted && s.screen === "camp", "deep camp");
  await page.evaluate(async (d) => { const r = window.__riddle; await r.mutate(() => r.engine.setStart(d)); }, start);
  await settle(page);
  await page.locator("button.send, .gem-slot .gem").first().click();
  await waitFor(page, (s) => s?.screen === "watch", "watch");
  await sleep(600);
  if (!fight) {
    await tileClick(page, "one"); await sleep(3500);
    if ((await state(page))?.screen === "watch") { await sampleFrames(page, name, 3000); await shoot(page, name, what); }
  } else {
    // (round 7's strip caught the floor's title card over the fight: wait for the fight frame with no card up)
    const card = () => page.evaluate(() => document.querySelector(".watch")?.dataset.card === "1" || !!document.querySelector(".interstitial:not([hidden])"));
    const foe = () => page.evaluate(() => (window.__viewer?.debugRects?.() ?? []).some((r) => !r.hero)).catch(() => false);
    const inFight = async () => (await state(page))?.frame === "fight" && !(await card()) && (await foe());
    const t = Date.now(); let ok = false;
    while (Date.now() - t < 60000) { if (await inFight()) { await sleep(450); if (await inFight()) { ok = true; break; } } await sleep(80); }
    if (ok) { await shoot(page, name, what, { pre: true }); await castStrip(page, name); await sampleFrames(page, name, 1500); }   // (the still first: a fight can end in the strip's 1.5 s)
  }
  if (page.__errs.length) log(`page errors (${name}):`, page.__errs.slice(0, 5));
  await page.close();
}

// ---- the boss moments (gfx round 7: at the phone watch's 100 env texels, was the demo's 160): the renderer's own boss script (render-demo.html?boss=1): entrance t≈12, break t=90, fall t=152 (10 ticks/s)
async function bossDemo() {
  const page = await newPage(400, 800, 2);
  await page.goto(`${BASE}render-demo.html?boss=1&biome=burrows&depth=8&speed=1&hud=0&texels=100${XQD ? `&${XQD}` : ""}`);
  await page.waitForFunction(() => !!window.viewer, null, { timeout: 30000 });
  await page.evaluate(() => { window.__viewer = window.viewer; });
  const at = async (tick) => { while ((await page.evaluate(() => window.viewer.stats().tick)) < tick) await sleep(30); };
  await at(10); await castStrip(page, "boss-in"); await shoot(page, "boss-in", "a boss comes into view (the entrance), dungeon view only", { lay: false, pre: true });
  await sampleFrames(page, "boss-fight", 2000);
  // (gfx round 10: the break's shatter and stamp play just after its beat, the fall's stamp and death pose after his: shot a few ticks on)
  // (the still first, then the strip: the strip's 1.5 s outlived the shatter and the stamp and the still missed them)
  await at(91); await shoot(page, "boss-break", "the boss breaks (his guard shattered), dungeon view only", { lay: false, pre: true }); await castStrip(page, "boss-break");
  await at(153); await shoot(page, "boss-fall", "the boss falls (the killing blow), dungeon view only", { lay: false, pre: true }); await castStrip(page, "boss-fall");
  await page.close();
}

// ---- desktop 1440 × 900 --------------------------------------------------------------------------------------------------------------
async function desktop() {
  const page = await newPage(1440, 900, 1);
  await goto(page, `fresh=1&seed=4243`);
  await waitFor(page, (s) => s?.booted && s.screen === "camp", "camp"); await settle(page);
  await step("d-watch", async () => {
    await page.locator("button.send, .gem-slot .gem").first().click();
    await waitFor(page, (s) => s?.screen === "watch", "watch"); await sleep(800); await tileClick(page, "one"); await sleep(3000);
    if (want("d-watch")) { await sampleFrames(page, "d-watch", 3000); await shoot(page, "d-watch", "desktop 1440×900: the watch"); }
    await toEnd(page);
  });
  await goto(page, `absent=8h`);
  await waitFor(page, (s) => s?.booted && (s.screen === "report" || s.screen === "ending"), "report", 240000); await settle(page);
  if (want("d-report")) await shoot(page, "d-report", "desktop 1440×900: the report");
  await step("d-death", async () => {
    if (!(await tileClick(page, "open"))) throw new Error("no worst death");
    await waitFor(page, (s) => s?.screen === "death", "death"); await settle(page); await measuredDeath(page);
    if (want("d-death")) await shoot(page, "d-death", "desktop 1440×900: the death screen");
    await tileClick(page, "camp"); await waitFor(page, (s) => s?.screen === "camp", "camp");
  });
  await settle(page);
  if ((await state(page))?.screen === "report") { await page.locator("main.report .gem").first().click().catch(() => {}); await waitFor(page, (s) => s?.screen === "camp", "camp"); }
  await page.evaluate((t) => window.__riddle.setRulesText(t), RICH); await sleep(6000); await settle(page);
  if (want("d-camp")) await shoot(page, "d-camp", "desktop 1440×900: the camp");
  await step("d-edit", async () => {
    // gfx round 7: the strip is the tap's opening, like the phone's edit (it was four frames of the open sheet: raters read "no opening")
    const r = page.locator("main.camp .row.tablet:not(.oath-tab)").first();
    const tap = () => r.evaluate((e) => e.click()).catch(() => {});
    if (want("d-edit")) await castStrip(page, "d-edit", tap); else await tap();
    await settle(page);
    if (want("d-edit")) await shoot(page, "d-edit", "desktop 1440×900: a rule tapped, the editor (strip: its opening)", { pre: true });
    await closeSheets(page);
  });
  await page.close();
}

const phone = !process.argv.includes("--no-phone"), desk = !process.argv.includes("--no-desktop");
try {
  if (phone) {
    await step("real", phoneReal);
    // gfx round 7: the Fens and the fight from a real save (the fake engine: `phoneFake`, kept for a fixture-less tree)
    if (want("watch-fens")) await step("fens", () => phoneDeep("watch-fens", 9, "the watch at normal speed, D9 the Fens (real engine)"));
    if (want("fight")) await step("fight", () => phoneDeep("fight", 5, "a fight (the fight frame, highlights mode)", true));
    if (want("boss")) await step("boss", bossDemo);
  }
  if (desk) await step("desktop", desktop);
} finally { await browser.close(); }

// strips: 4 frames side by side at half size
const py = `
import sys, json, os
from PIL import Image
out, names = sys.argv[1], json.loads(sys.argv[2])
for n in names:
    try:
        fr = [Image.open(f"{out}/{n}-f{i}.jpg" if os.path.exists(f"{out}/{n}-f{i}.jpg") else f"{out}/{n}-f{i}.png").convert("RGB") for i in range(4)]
    except FileNotFoundError:
        continue
    w, h = fr[0].size; s = 400 / w if w > 400 else 1
    W, H = int(w * s), int(h * s)
    im = Image.new("RGB", (W * 4 + 30, H), (40, 40, 40))
    for i, f in enumerate(fr): im.paste(f.resize((W, H), Image.LANCZOS), (i * (W + 10), 0))
    im.save(f"{out}/{n}-strip.png")
    import os
    for i in range(4):
        for ext in ("jpg", "png"):
            if os.path.exists(f"{out}/{n}-f{i}.{ext}"): os.remove(f"{out}/{n}-f{i}.{ext}")
`;
execFileSync("python3", ["-c", py, out, JSON.stringify(shots.filter((s) => s.strip).map((s) => s.name))]);

// the rater folder: neutral names, the moments key (for the scorer only) and the brief
const rd = join(out, "rater"); mkdirSync(rd, { recursive: true });
// --only: merge into the round's earlier capture (moments keep their order; re-shot ones replace theirs)
const prev = (f, d) => { try { return JSON.parse(readFileSync(join(out, f), "utf8")); } catch { return d; } };
if (only.length || !phone || !desk) {
  const old = prev("shots.json", []);
  for (const o of old) if (!shots.some((s) => s.name === o.name)) shots.push(o);
  const order = old.map((o) => o.name); shots.sort((a, b) => (order.indexOf(a.name) + 1 || 999) - (order.indexOf(b.name) + 1 || 999));
  for (const f of prev("frames.json", [])) if (!frames.some((x) => x.name === f.name)) frames.push(f);
  for (const [k, v] of Object.entries(prev("anims.json", {}))) anims[k] ??= v;
  for (const [k, v] of Object.entries(prev("layout.json", {}))) layout[k] ??= v;
}
writeFileSync(join(out, "shots.json"), JSON.stringify(shots, null, 1));
const key = [];
shots.forEach((s, i) => {
  const m = `m${String(i + 1).padStart(2, "0")}`;
  copyFileSync(join(out, `${s.name}.png`), join(rd, `${m}.png`));
  if (s.strip) try { copyFileSync(join(out, `${s.name}-strip.png`), join(rd, `${m}-strip.png`)); } catch {}
  key.push({ m, name: s.name, what: s.what });
});
writeFileSync(join(out, "moments.json"), JSON.stringify(key, null, 1));
writeFileSync(join(out, "frames.json"), JSON.stringify(frames, null, 1));
writeFileSync(join(out, "anims.json"), JSON.stringify(anims, null, 1));
writeFileSync(join(out, "layout.json"), JSON.stringify(layout, null, 1));
const tpl = readFileSync(join(ROOT, "tools/gfx-rater-prompt.txt"), "utf8");
const list = key.map((k) => `- ${rd}/${k.m}.png${shots.find((s) => s.name === k.name)?.strip ? ` and ${rd}/${k.m}-strip.png` : ""} — ${k.what}`).join("\n");
writeFileSync(join(rd, "prompt.txt"), tpl.replaceAll("{{LIST}}", list).replaceAll("{{TARGETS}}", join(ROOT, "art/ui/targets")));
log(`\n${shots.length} moments → ${out}`);
const bad = Object.entries(layout).filter(([, r]) => !r.ok).map(([n]) => n);
log(`layout: ${bad.length ? `FAIL ${bad.join(", ")}` : "ok"}`);
