#!/usr/bin/env node
// Cut 27 client gates (docs/CUT27.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §1  solved floors fold: a send whose forecast clears D1–2 ≥ 95 % opens the watch on the fold line (`D1–2 · 100% · +$N` and its chips),
//       never on D1's picture; the watch lands on the first floor below the bar; ≤ 5 s watched per folded floor; the line carries every
//       state change the folded floors' events hold (thefts, finds, cage picks, companions, levels, max hp, bones, facts, a boss down, a
//       death-adjacent hp dip) — the events the engine stepped under the line, read here on their own; the line docks under the HUD and a
//       tap on it plays the folded floors (the replay sheet, floor by floor); `?fold=0` folds nothing
//   §2  the edit is a scene: after an edit's refine, the core's `divergence(prev)` plays over the well's foot — the sent branch then the
//       edited one (`sent · R2` → `dies · D7`, `R5 now` → `lives · D9`), 3–5 s, a tap lets it go, its line stays (`R5 now → lives · D9 · vs
//       dies · D7`); an `≈` edit with a divergence reads `≈ ±N · R3 fires 4× more`; reduced motion shows the two end frames, still; the scene
//       starts ≤ 1.5 s after the refine (the fake; the real-wasm D11 number is clarity.mjs's)
//   §4  the stall screen's gem waits for the whole-run measure (`…`), never lights a harming patch, and is the first tablet shown
//   §5  a drive-off whose counter row is already in the set reads `order` (`R7 under R2`) and the gem moves it, not `counter unwritten`
//
//   node web/tests/cut27.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync, readFileSync } from "node:fs";
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
const state = () => page.evaluate(() => { const r = window.__riddle, w = document.querySelector(".watch"); return r ? { screen: r.screen, booted: r.booted, fold: w?.dataset.fold, folded: w?.dataset.folded, over: w?.dataset.over === "1" || w?.dataset.ending === "1" } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(40); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} fold=${s?.fold} folded=${s?.folded})`);
}
const camp = async () => { await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250); };

// the fold's forecast: D1–2 clear 100 %, D3 99 % → D4 50 % (D3 is below the bar) — the camp's own forecast stands in for the core's
const FOLD_FC = { depths: [{ depth: 1, reach: 1, clear: 1 }, { depth: 2, reach: 1, clear: 1 }, { depth: 3, reach: 0.99, clear: 0.5 }, { depth: 4, reach: 0.5 }], causes: [], known_to: 4, start: 1, refined: true, fold_to: 2 };
const GOOD = JSON.parse(readFileSync(resolve(ROOT, "crates/riddle-core/presets/good.json"), "utf8")).rows.slice(0, 4);
// the kinds a fold line must carry, read off the events on their own (the gate's own reading, not the watch's)
const kindsOf = (steps) => {
  const k = new Set();
  for (const r of steps) {
    const hero = r.snapshot.hero.id;
    for (const e of r.events) {
      if (e.k === "steal") k.add("stolen");
      else if (e.k === "pickup" && e.id === hero && !/^gold\b/.test(e.item)) k.add("found");
      else if (e.k === "fact") k.add("learned");
      else if (e.k === "level") k.add("level");
      else if (e.k === "max_hp" && e.id === hero && e.delta) k.add("max_hp");
      else if (e.k === "bones" && e.heir !== r.snapshot.run.heir) k.add("bones");
      else if (e.k === "tame" && e.ok) k.add("pet");
      else if (e.k === "hurt" && e.id === hero && e.hp / Math.max(1, r.snapshot.hero.max_hp) < 0.25) k.add("hp");
    }
    if (r.snapshot.vault_choice?.items?.length) k.add("took");
  }
  return k;
};

try {
  // ---- §1: the fold — the core's `fold()` (the fake's stand-in, `?fake_fold=2`: D1–2 fold), and a core without it (the watch steps the
  // stretch itself, batch by batch, off the camp's forecast)
  let landings = 0;
  const CORE_KIND = { stolen: "theft", found: "find", learned: "fact", hp: "dip", max_hp: "max_hp", level: "level", bones: "bones", pet: "pet" };
  for (const [seed, path] of [[26, "core"], [26, "client"], [157, "client"], [31, "core"], [12, "client"]]) {
    if (seed !== 26 && landings >= 2) break;
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}&fake_fold=2`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.evaluate(async ({ fc, rows, path }) => {
      const r = window.__riddle; window.__steps = []; window.__foldRes = null;
      if (path === "client") { r.forecastOfRules = () => fc; r.engine.fold = undefined; }
      else { const fold0 = r.engine.fold.bind(r.engine); r.engine.fold = async () => { const x = await fold0(); window.__foldRes = x; return x; }; }
      // a set that lives through D1–2 on the fake (the shipped preset's shape), so the watch lands below the stretch
      r.rules.rows = rows; r.rulesChanged(); await r.engine.setRules(r.rules);
      const step0 = r.engine.step.bind(r.engine);
      r.engine.step = async (n) => { const x = await step0(n); window.__steps.push({ fold: document.querySelector(".watch")?.dataset.fold === "1", r: x }); return x; };
    }, { fc: FOLD_FC, rows: GOOD, path });
    const seed_ = `seed ${seed} (${path})`;
    await sleep(300);
    const t0 = Date.now();
    await page.evaluate(() => document.querySelector("button.gem.send")?.click());
    const first = await waitFor((s) => s?.screen === "watch" && (s.fold === "1" || !!s.folded), "the fold line", 10_000);
    const early = await page.evaluate(() => ({ line: !!document.querySelector(".watch .fold-line:not([hidden]):not(.docked)"), head: document.querySelector(".watch .fold-head")?.textContent ?? "" }));
    // (a fold that already landed when the first sample came — a loaded machine — was still the first thing shown: its log says so)
    const log0 = first.fold === "1" ? null : await page.evaluate(() => (window.__foldLog ?? [])[0] ?? null);
    check(first.fold === "1" ? /^D1(–2)? · \d+%/.test(early.head) : !!log0 && log0.from === 1 && /^D1(–2)? · \d+%/.test(log0.head), `${seed_}: the watch opens on the fold line, not on D1 ("${early.head || log0?.head}")`);
    if (seed === 26 && path === "client") await shot("cut27-fold-line");
    const landed = await waitFor((s) => !!s?.folded || s?.screen !== "watch", "the landing", 20_000);
    const ms = Date.now() - t0;
    const r = await page.evaluate(() => {
      const log = (window.__foldLog ?? [])[0], w = document.querySelector(".watch");
      const during = window.__foldRes ? [window.__foldRes.step] : (window.__steps ?? []).filter((x) => x.fold).map((x) => x.r);
      return { core: window.__foldRes ? { chips: window.__foldRes.chips, kinds: [...new Set(window.__foldRes.beats.map((b) => b.kind))], to: window.__foldRes.to } : null, texts: [...document.querySelectorAll(".watch .fold-line .fchip")].map((c) => c.textContent), log, depth: Number(document.querySelector(".watch .depth")?.textContent?.slice(1)), shown: [...document.querySelectorAll(".watch .fold-line .fchip")].map((c) => c.dataset.k), docked: !!document.querySelector(".watch .fold-line.docked:not([hidden])"), during, kinds: w?.dataset.foldPlan };
    });
    const floors = r.log ? r.log.floors : 1;
    check(!!r.log && r.log.from === 1 && r.log.to >= 1 && r.log.to <= 2, `${seed_}: the stretch folded is D${r.log?.from}–${r.log?.to} (plan D1–2)`);
    check((r.log?.perFloor ?? 1e9) <= 5000, `${seed_}: ≤ 5 s watched per folded floor (the line's own ${r.log?.perFloor} ms a floor; ${Math.round(ms / floors)} ms a floor from the send tap, the watch's mount included)`);
    check(landed.screen !== "watch" || r.depth > (r.log?.to ?? 2) || landed.over, `${seed_}: the watch lands on the first floor below the bar (D${r.depth}${landed.over ? ", the run ended inside the fold" : ""})`);
    const want = kindsOf(r.during), shown = new Set(r.core ? r.core.kinds : r.shown);
    const missing = [...want].filter((k) => !shown.has(r.core ? CORE_KIND[k] ?? k : k)).filter((k) => !(r.core && k === "took"));
    if (r.core) check(r.core.chips.every((c) => r.texts.includes(c)), `${seed_}: the line shows the core's chips (${r.core.chips.join(" · ") || "none"})`);
    check(missing.length === 0 && r.during.length > 0, `${seed_}: the line carries every state change of its floors (${[...want].join(", ") || "none"} ⊆ ${[...shown].join(", ") || "none"}${missing.length ? `; missing ${missing.join(", ")}` : ""}; ${r.during.length} batches)`);
    if (landed.screen === "watch" && !landed.over) {
      landings++;
      check(r.docked, `${seed_}: the line docks under the HUD once the watch opens below it`);
      if (seed === 26 && path === "client") await shot("cut27-fold-docked");
      // a tap on the line plays the folded floors (the replay sheet, floor by floor), the watch's picture frozen meanwhile
      await page.locator(".watch .fold-line").click({ timeout: 10_000 });
      await page.waitForSelector(".sheet-wrap .fold-replay", { timeout: 5000 }).catch(() => {});
      await sleep(600);
      const rp = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .fold-replay"); return b ? { floor: b.dataset.floor, at: b.querySelector(".fold-at")?.textContent, paused: document.querySelector(".watch")?.dataset.foldReplay } : null; });
      check(!!rp && rp.floor === "1" && rp.paused === "1", `${seed_}: a tap on the line plays the folded floors (${rp ? `D${rp.floor} · "${rp.at}"` : "no sheet"}; the watch frozen: ${rp?.paused})`);
      if (seed === 26 && path === "client") await shot("cut27-fold-replay");
      await page.keyboard.press("Escape"); await sleep(300);
      const thawed = await page.evaluate(() => ({ r: document.querySelector(".watch")?.dataset.foldReplay, pause: document.querySelector(".watch .gem.on") ? 1 : 0 }));
      check(thawed.r === "0" && thawed.pause === 0, `${seed_}: closing the replay thaws the watch`);
    }
  }
  check(landings >= 1, `a fold landed below the bar on at least one seed (${landings})`);
  // `?fold=0`: nothing folds (dev)
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&fold=0`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.evaluate((fc) => { window.__riddle.forecastOfRules = () => fc; }, FOLD_FC);
    await page.evaluate(() => document.querySelector("button.gem.send")?.click());
    await waitFor((s) => s?.screen === "watch", "the watch");
    await sleep(1200);
    const s = await page.evaluate(() => ({ plan: document.querySelector(".watch")?.dataset.foldPlan, line: !!document.querySelector(".watch .fold-line:not([hidden])") }));
    check(s.plan === "" && !s.line, `\`?fold=0\` folds nothing (plan "${s.plan}")`);
  }

  // ---- §2: the edit is a scene (the fake has no `divergence()`: a stand-in built from a second fake engine's run — the scene's shape)
  const stubDivergence = async (over) => page.evaluate(async (over) => {
    const { createFakeEngine } = await import("/src/engine/fake.ts");
    const branchOf = (seed, row) => { const e = createFakeEngine(); e.newLineage(seed); const s0 = e.send(); const evs = []; let snap = s0; for (let i = 0; i < 6; i++) { const r = e.step(10); evs.push(...r.events); snap = r.snapshot; } return { row, text: `R${row + 1}`, snapshot: s0, events: evs, end_snapshot: snap }; };
    const d = { seed: 3, tick: 120, depth: 4, sent_row: 1, new_row: 4, sent_end: { tier: "death", depth: 7, cause: "ogre", gold: 0 }, new_end: { tier: "bank", depth: 9, gold: 212 },
      sent: branchOf(4242, 1), new: branchOf(4242, 4), moved: 0.12, inside: false, sims: 50, ...over };
    const r = window.__riddle; window.__divAsks = 0;
    r.engine.divergence = async () => { window.__divAsks++; return d; };
  }, over);
  const edit = (k) => page.evaluate((k) => { const r = window.__riddle; const c = r.rules.rows.flatMap((x) => x.conds).find((x) => x.n !== undefined); if (c) c.n = Math.max(5, c.n + k); else r.rules.rows[0].conds.push({ k: "hp<", n: 40 }); r.rulesChanged(); }, k);
  const scene = () => page.evaluate(() => { const e = document.querySelector(".camp .div-scene"), l = document.querySelector(".camp .div-line"); return { shown: !!e && !e.hidden, state: e?.dataset.state ?? "", phase: e?.dataset.phase ?? "", tag: e?.querySelector(".div-tag")?.textContent ?? "", end: e?.querySelector(".div-end.show")?.textContent ?? "", line: l && !l.hidden ? l.textContent.replace(/\s+/g, " ").trim() : "", stills: e?.querySelectorAll(".div-still").length ?? 0, dev: window.__scene ?? {} }; });
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=27`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.waitForFunction(() => window.__riddle.lastForecast?.refined === true, null, { timeout: 15_000 }).catch(() => {});
    await stubDivergence({});
    await edit(5);
    await page.waitForFunction(() => document.querySelector(".camp .div-scene")?.dataset.state === "playing", null, { timeout: 15_000 }).catch(() => {});
    const seen = { sent: null, new: null, ends: [] };
    const t0 = Date.now();
    while (Date.now() - t0 < 7000) {
      const s = await scene();
      if (s.phase === "sent" && !seen.sent) { seen.sent = s.tag; await shot("cut27-scene-sent"); }
      if (s.phase === "new" && !seen.new) seen.new = s.tag;
      if (s.end && !seen.ends.includes(s.end)) { seen.ends.push(s.end); if (s.phase === "new") await shot("cut27-scene-new"); }
      if (!s.shown && seen.new) break;
      await sleep(60);
    }
    const s = await scene();
    const dur = (s.dev.doneAt ?? NaN) - (s.dev.playAt ?? NaN), lat = (s.dev.playAt ?? NaN) - (s.dev.refineAt ?? NaN);
    check(seen.sent === "last run · attack nearest" && seen.new === "a rule now", `the scene plays the sent branch, then the edited one ("${seen.sent}" → "${seen.new}")`);
    const ends = s.dev.ends ?? seen.ends;
    check(ends.join(" | ") === "dies · D7 | lives · D9", `each branch ends on its run's end (${ends.join(" | ")})`);
    check(dur >= 3000 && dur <= 5000, `the scene takes 3–5 s (${Math.round(dur)} ms)`);
    check(lat <= 1500, `the scene starts ≤ 1.5 s after the refine (fake: ${Math.round(lat)} ms)`);
    check(s.line === "a rule now → lives · D9 · was dies · D7" && !s.shown, `its line stays under \`vs sent\` ("${s.line}")`);
    await shot("cut27-scene-line");
    // a tap on the line plays it again; a tap on the scene lets it go (the line stays)
    await page.locator(".camp .div-line").click({ timeout: 10_000 });
    await sleep(500);
    const again = await scene();
    { const b = await page.locator(".camp .div-scene").boundingBox(); if (b) await page.mouse.click(b.x + b.width - 12, b.y + b.height - 12); }   // (a tap anywhere lets it go; the scene takes none)
    await sleep(150);
    const gone = await scene();
    check(again.shown && again.state === "playing" && !gone.shown && gone.line === s.line, `a tap on the line replays it (${again.state}); a tap on the scene lets it go, the line stays`);
    // an `≈` edit with a divergence says what changed
    await stubDivergence({ moved: 0.02, inside: true, fires: [{ sent_row: 2, new_row: 2, text: "R3 drink heal", sent: 0.5, new: 2.1 }] });
    await edit(5);
    await page.waitForFunction(() => { const l = document.querySelector(".camp .div-line"); return l && !l.hidden && /^same/.test(l.textContent.trim()); }, null, { timeout: 15_000 }).catch(() => {});
    const flat = await scene();
    check(/^same · drink heal fires 4× more · a rule now → lives · D9 · was dies · D7$/.test(flat.line) && flat.shown, `an \`≈\` edit plays too and says what changed ("${flat.line}")`);
    // a move under the bar outside its ± is the number's alone (no scene)
    await stubDivergence({ moved: 0.03, inside: false });
    await edit(5);
    await page.waitForFunction(() => window.__divAsks > 0, null, { timeout: 15_000 }).catch(() => {});
    await sleep(300);
    const small = await scene();
    check(!small.shown && !small.line, `a 3-pt move outside its ± plays no scene`);
    // reduced motion: the two end frames, still
    await page.emulateMedia({ reducedMotion: "reduce" });
    await stubDivergence({});
    await edit(5);
    await page.waitForFunction(() => document.querySelector(".camp .div-scene")?.dataset.state === "still", null, { timeout: 15_000 }).catch(() => {});
    await sleep(400);
    const still = await scene();
    check(still.state === "still" && still.stills === 2, `reduced motion shows the two end frames, still (${still.state}, ${still.stills} frames)`);
    await shot("cut27-scene-still");
    await page.emulateMedia({ reducedMotion: "no-preference" });
  }

  // ---- §4: the stall screen's gem — measured before it offers a patch, never a harming one, the first tablet shown
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=41`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.evaluate(() => {
      const r = window.__riddle;
      r.rules.rows = [{ conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack" } }, { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }]; r.rulesChanged();
      const cut = { row: r.rules.rows[0], insert_at: 0, remove: true, survive: 1 / 12, forecast_delta: 0.05 };
      const ret = { row: { conds: [{ k: "depth>=", n: 5 }], verb: { v: "return" } }, insert_at: 0, survive: 1, forecast_delta: 0.4, exits: true };
      const fix = { row: { conds: [{ k: "foes>=", n: 3 }], verb: { v: "retreat" } }, insert_at: 0, survive: 1, forecast_delta: 0.2 };
      const whole = (reach, harms) => ({ reach, reach_pm: 0.03, death: -reach / 2, death_pm: 0.03, harms });
      // the core's landing: ranked on whole runs — the return harms (reach falls), the cut is weak, the retreat leads
      r.engine.deathDeltas = async () => { await new Promise((res) => setTimeout(res, 700)); return [{ ...fix, whole: whole(0.2, false), camp_pending: false }, { ...cut, whole: whole(0.02, false), camp_pending: false }, { ...ret, whole: whole(-0.4, true), camp_pending: false }]; };
      const d = { run_id: 77, depth: 5, cause: "R1 attack ↔ pick up", margin: "", verdict: "stall", baseline: 0, replays: 12, trace: { turns: [] }, patches: [cut, ret, fix], morgue: "" };
      r.go({ kind: "death", death: d });
    });
    await sleep(250);
    const pre = await page.evaluate(() => ({ gem: document.querySelector(".console .patch-gem")?.textContent ?? document.querySelector(".console .gem")?.textContent }));
    await sleep(1200);
    const post = await page.evaluate(() => {
      const tabs = [...document.querySelectorAll(".death .patches button.patch")];
      return { first: tabs[0]?.textContent.replace(/\s+/g, " ").trim(), lit: tabs.findIndex((b) => b.classList.contains("top")), harmsLit: !!document.querySelector(".death .patches button.patch.top.harms"),
        gem: document.querySelector(".console .patch-gem")?.textContent ?? "", gemHarms: !!document.querySelector(".console .patch-gem.harms"), flag: document.querySelector(".death")?.dataset.gemFirst };
    });
    check(/…/.test(pre.gem ?? ""), `the stall gem waits for the whole-run measure ("${pre.gem}")`);
    check(post.lit === 0 && post.flag === "1" && /retreat/.test(post.first ?? ""), `after the measure the gem's tablet is the first shown (${post.lit}: "${post.first}")`);
    check(!post.harmsLit && !post.gemHarms, `the stall gem never lights a harming patch ("${post.gem}")`);
    await shot("cut27-stall-gem");
    // the core names the gem (`Patch.gem`, always its first): the tablets keep its order, the gem lights it; none flagged → `edit`
    for (const none of [false, true]) {
      await page.evaluate((none) => {
        const r = window.__riddle;
        const cut = { row: r.rules.rows[0], insert_at: 0, remove: true, survive: 1 / 12, forecast_delta: 0.05 };
        const fix = { row: { conds: [{ k: "foes>=", n: 3 }], verb: { v: "retreat" } }, insert_at: 0, survive: 1, forecast_delta: 0.2 };
        const whole = (reach, harms) => ({ reach, reach_pm: 0.03, death: -reach / 2, death_pm: 0.03, harms });
        r.engine.deathDeltas = async () => { await new Promise((res) => setTimeout(res, 300)); return [{ ...fix, whole: whole(0.2, none), camp_pending: false, gem: !none }, { ...cut, whole: whole(-0.3, true), camp_pending: false, gem: false }]; };
        r.go({ kind: "death", death: { run_id: 79, depth: 5, cause: "R1 attack ↔ pick up", margin: "", verdict: "stall", baseline: 0, replays: 12, trace: { turns: [] }, patches: [cut, fix], morgue: "" } });
      }, none);
      await sleep(1000);
      const g = await page.evaluate(() => { const tabs = [...document.querySelectorAll(".death .patches button.patch")]; return { first: tabs[0]?.textContent.replace(/\s+/g, " ").trim(), lit: tabs.findIndex((b) => b.classList.contains("top")), gem: document.querySelector(".console .gem")?.textContent }; });
      check(none ? g.lit < 0 && /edit/.test(g.gem ?? "") : g.lit === 0 && /retreat/.test(g.first ?? ""), none ? `no patch flagged: no gem (\`${g.gem}\`)` : `the core's gem is lit and first ("${g.first}")`);
    }
  }
  // ---- §5: a drive-off whose counter the set holds reads `order` (`R3 under R1`), the gem moves it
  {
    await page.evaluate(async () => {
      const r = window.__riddle, { drivenDeath } = await import("/src/ui/death.ts");
      const counter = { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "tag:boss" } };
      r.rules.rows = [{ conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack" } }, { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" } }, counter]; r.rulesChanged();
      const line = { carried: 90, keep_pct: 60, kept: 0, spent: 0, spent_on: [], text: "driven $0 · $90 lost", run_id: 78,
        driven: { boss: "goblin_warlord", title: "Warlord", depth: 8, verdict: "no counter", defence: "shield wall", counter: "attack boss", row: counter, run_id: 78 } };
      const trace = { turns: [0, 0, 0, 1, 0].map((row, i) => ({ t: i * 10, row, verb: { v: "attack" }, hp: 20, foes: 2, telegraphs: [] })) };
      r.go({ kind: "death", death: drivenDeath(line, 78, trace) });
    });
    await sleep(300);
    const dv = await page.evaluate(() => ({ head: document.querySelector(".death .patches.driven .patches-moment")?.textContent, surv: document.querySelector(".death .patches.driven .surv")?.textContent, gem: document.querySelector(".console .patch-gem")?.textContent ?? document.querySelector(".console .gem")?.textContent }));
    check(dv.head === "D8 · order" && dv.surv === "under attack" && /move/.test(dv.gem ?? ""), `a drive-off with its counter held reads order ("${dv.head}" · "${dv.surv}" · gem "${dv.gem}")`);
    await page.evaluate(() => document.querySelector(".console .patch-gem")?.click());
    await sleep(300);
    const moved = await page.evaluate(() => ({ screen: window.__riddle.screen, r1: window.__riddle.rules.rows[0]?.verb.a ?? window.__riddle.rules.rows[0]?.verb.v }));
    check(moved.screen === "camp" && moved.r1 === "tag:boss", `the gem moves the counter above the row that won (${moved.screen}; R1 ${moved.r1})`);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}\n${e.stack}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut27: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut27: ok (${out.length} checks)`);
