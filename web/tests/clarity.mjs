#!/usr/bin/env node
// Cut 10 §2–§3 client rows, each one assertion on the fake (`?engine=fake&systems=none&dev=1`), on the GPU harness (tools/browser.mjs)
// against the dev server (tools/dev.sh, :5219):
//   §2  the forecast's boss floor reads `D5 0% · goblin warlord · try: attack boss` when the counter fact is known and the row
//       is absent; tapping it inserts the row at the top of the active set (and the bestiary chip inserts at the top too)
//   §3  `rest 20m · send skips` permanently · a greyed supply says why under its price · `▲▼` chips (44 px) move a row ·
//       the `+1 row` card is dimmed `⊘ fill rows` while free rows exist · `3 over` is not on the headline · the report's tiles read
//       `returned · banked · deaths` when returns outnumber banks, its exit lines lead with `returned $61`, a lost companion reads
//       `jackal Ashar fell` · a card's delta reads `reach +N% at R2` (Cut 12: where it goes) · the tiles fade in after an absence
//   Cut 14 §6  paused, the frontier runs and the dot beats while the playhead holds · ▶▶| lands live · a hidden tab runs the world ·
//              a run that ends while paused shows its exit after the replay · ⏸ holds the HUD, the card, the ticker and the tick
//              (under the card too) · the card names the HUD's floor and loot on every 100 ms sample
//   Cut 14     the `slowdowns` toggle: off, the clock keeps the mode's flat rate through a fight; on, a fight in fast runs at 4×
//   Cut 14 §4  a repeated chore callout coalesces (`pick up ×8`) · the `rest 20m` banner sits under the death frame's callout line
//   Cut 15 §4  the lit mode chip carries its rate (`fast 16`, `fights 2`) · the floor card is up ≤ 1.2 s a time, never over a fight
//   Cut 15 §5  a watched vault sheet holds the world (3 s open: the frontier still) with a 6 s bar (Cut 18 §1, was 30); a chip still takes
//              the tap; untouched it closes by 6.5 s and the world goes on
//
//   Cut 20 §3  an edit's first forecast paints ≤ 1.2 s with the slow measures in flight (the fake behind a worker's timing)
//   Cut 24 §4  real wasm: an edit's first pass ≤ 1 s, its refine ≤ 3 s — quiet, after a burst, with the slow measures in flight
//              (`--no-wasm` skips it)
//
//   node web/tests/clarity.mjs [--part=core|watch|hold|card|paint|deep]   (part of `pnpm test` in web/, which runs the parts side by
//   side: the whole walk was one 4-minute test the suite waited on; each part opens its own pages)
import { execFileSync } from "node:child_process";
import { existsSync, readFileSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows, openPanel } from "./lib/frame.mjs";
import { measured } from "./lib/load.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const PART = process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "all";
const part = (p) => PART === "all" || PART.split(",").includes(p);

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
async function settle() { const t = Date.now(); let clear = 0; while (Date.now() - t < 10_000) { const s = await state(); clear = s && s.booted && !s.busy ? clear + 1 : 0; if (clear >= 2) break; await sleep(150); } await sleep(150); }
const rowTexts = () => page.evaluate(() => [...document.querySelectorAll(".editor .row")].map((r) => r.querySelector(".chips").innerText.replace(/\s+/g, " ").trim()));
const engineRows = () => page.evaluate(async () => (await window.__riddle.engine.lineage()).sets[window.__riddle.active].rows.map((r) => `${r.conds.map((c) => c.k + (c.t ?? "") + (c.n ?? "")).join(" ")}→${r.verb.v}${r.verb.a ?? ""}`));

try {
  if (part("core")) {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await editRows(page);   // Cut 17: the tablets carry their chips, ▲▼ and × (the `edit` tile, remembered)
  await settle();
  // Cut 17: the unlock panel lists the whole catalogue from here on (`more`, remembered); the panel closes over the tablets
  await openPanel(page, "unlocks", { all: true }); await page.keyboard.press("Escape"); await sleep(100);

  // §3 ▲▼: two 44 px chips beside the grip; ▼ on R1 swaps R1 and R2 in the editor and in the engine's set; R1's ▲ is off
  {
    const rows = await rowTexts();
    const geom = await page.evaluate(() => { const r = document.querySelector(".editor .row"); const up = r.querySelector(".updown .up"), down = r.querySelector(".updown .down"); const g = r.querySelector(".grip").getBoundingClientRect(); return { up: up.getBoundingClientRect().height, down: down.getBoundingClientRect().height, upDisabled: up.disabled, besideGrip: Math.abs(up.getBoundingClientRect().left - g.right) < 12, lastDown: [...document.querySelectorAll(".editor .row")].pop().querySelector(".updown .down").disabled }; });
    check(geom.up >= 44 && geom.down >= 44 && geom.besideGrip, `▲▼ chips are ≥ 44 px tall beside the grip (${geom.up}/${geom.down} px)`);
    check(geom.upDisabled && geom.lastDown, "R1's ▲ and the last row's ▼ are off");
    await page.locator(".editor .row").first().locator(".updown .down").click({ timeout: 5000 });
    await sleep(400);
    const after = await rowTexts(), eng = await engineRows();
    check(rows.length >= 2 && after[0] === rows[1] && after[1] === rows[0], `▼ on R1 swaps R1 and R2 ("${after[0].slice(0, 30)}" first)`);
    check(eng.length === after.length, `the engine's set has the same ${eng.length} rows`);
    await page.locator(".editor .row").nth(1).locator(".updown .up").click({ timeout: 5000 });
    await sleep(400);
    const back = await rowTexts();
    check(back[0] === rows[0] && back[1] === rows[1], "▲ on R2 puts it back");
  }
  // §3 `+1 row` dimmed `⊘ fill rows` while free rows exist (a fresh fake set has fewer rows than max_rows)
  {
    const info = await page.evaluate(() => { const r = window.__riddle; const card = [...document.querySelectorAll(".unlocks .card")].find((c) => /^\+1 rule/.test(c.textContent)); return { rows: r.rules.rows.length, max: r.vocab.max_rows, has: !!card, gated: card?.classList.contains("gated"), needs: card?.querySelector(".needs")?.textContent.trim() }; });
    check(info.rows < info.max && info.has && info.gated && /fill rules$/.test(info.needs ?? ""), `the +1 row card is dimmed (${info.rows}/${info.max} rows): "${info.needs}"`);
    // fill the set: the card's gate lifts (the fake's own gate then decides)
    await page.evaluate(() => { const r = window.__riddle; while (r.rules.rows.length < r.vocab.max_rows) r.rules.rows.push({ conds: [{ k: "hp<", n: 30 }], verb: { v: "retreat" }, origin: "player" }); r.rulesChanged(); r.go({ kind: "camp" }); });
    await settle();
    const full = await page.evaluate(() => { const card = [...document.querySelectorAll(".unlocks .card")].find((c) => /^\+1 rule/.test(c.textContent)); return card?.querySelector(".needs")?.textContent.trim() ?? ""; });
    check(!/fill rules|rows full/.test(full), `with the set full the card no longer says fill rows ("${full}")`);
  }
  // §3 a card's reach delta says where the card goes — Cut 12 §1: `at R2` (before the engagement row, the catalogue's `insert_at`), else `at end`
  {
    const delta = await page.evaluate(() => [...document.querySelectorAll(".unlocks .card .delta")].map((d) => d.textContent.trim()));
    // docs/COPY.md pass 3–4: where the card joins is the sheet's (`joins above attack boss`); the card's line is its move, or inside
    // its ± only the foes it answers (`vs archers`), else nothing
    check(delta.some((d) => d !== "") && delta.every((d) => /^(reach [+−]\d+( ±\d+)?|vs [a-z ]+|)$/.test(d)), `card deltas read their move or their foes: ${delta.slice(0, 2).join(" · ")}`);
  }
  // §3 a greyed supply says why under its price
  {
    await page.evaluate(async () => { const r = window.__riddle; const save = JSON.parse(await r.engine.save()); save.lineage.gold = 5; r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" }); });
    await settle();
    const why = await page.evaluate(() => [...document.querySelectorAll(".supplies .chip.buy.off")].map((c) => ({ label: c.textContent.replace(/\s+/g, " ").trim(), why: c.querySelector(".why")?.textContent.trim() ?? "" })));
    check(why.length > 0 && why.every((w) => /^\$\d+ short$/.test(w.why)), `greyed supplies say why ($5 in hand): ${why.slice(0, 2).map((w) => w.label).join(" · ")}`);
  }
  // §3 the rest chip reads `rest 20m · send skips` permanently
  {
    await page.evaluate(() => { const r = window.__riddle; r.lineage.rest_left_s = 1200; r.go({ kind: "camp" }); });
    await sleep(300);
    const rest = await page.evaluate(() => { const el = document.querySelector(".rest-line .rest"); return { text: el?.textContent.trim(), hidden: el?.hidden, tag: el?.tagName }; });
    check(rest.text === "heir rests 20m" && !rest.hidden, `the rest chip reads "${rest.text}"`);
  }
  // §2 the try row: the counter fact known, the row absent → `D5 0% · goblin warlord · try: attack boss`; a tap inserts it at the top
  {
    await page.evaluate(async () => {
      const r = window.__riddle;
      const save = JSON.parse(await r.engine.save());
      save.lineage.facts.push("boss:goblin_warlord:counter"); save.lineage.best_depth = 4;
      r.lineage = await r.engine.load(JSON.stringify(save));
      r.rules.rows.splice(0, r.rules.rows.length, { conds: [{ k: "hp<", n: 40 }], verb: { v: "retreat" }, origin: "player" }, { conds: [], verb: { v: "attack", a: "nearest" }, origin: "player" });
      r.rulesChanged(); r.go({ kind: "camp" });
    });
    await settle();
    const bar = await page.evaluate(() => { const b = document.querySelector(".fc-bars .bar.try"); return b ? { tag: b.tagName, text: [...b.children].map((c) => c.textContent.replace(/\s+/g, " ").trim()).filter(Boolean).join(" ") } : null; });
    // (gfx round 18, 83a1e6d: a try row keeps only whose floor it is — the killer line left it; the counter is the gate)
    check(!!bar && bar.tag === "BUTTON" && /^D5 try: attack boss <?\d+%( ±\d+…?)?( · killer: goblin warlord)?$/.test(bar.text), `the boss floor names the counter: "${bar?.text}"`);   // QA on 50bb162: the hint rides the track, before the number
    const before = await rowTexts();
    await openPanel(page, "forecast");   // Cut 17: the bars live in the forecast panel behind the shaft
    await page.locator(".fc-bars .bar.try").click({ timeout: 5000 });
    await waitFor((s) => s?.screen === "camp", "camp"); await settle();
    const after = await rowTexts(), eng = await engineRows();
    check(after.length === before.length + 1 && /^foe: boss \+? ?→ attack boss$/.test(after[0]) && after.slice(1).join("|") === before.join("|"), `the tap inserts the counter row at the top: "${after[0]}"`);
    check(/foe_tagboss→attacktag:boss/.test(eng[0]), `the engine's set leads with it (${eng[0]})`);
    check(!(await page.evaluate(() => !!document.querySelector(".fc-bars .bar.try"))), "with the row held the bar no longer offers it");
    // the bestiary counter chip inserts at the top too (Cut 7 §1 chip, Cut 10 §2 position)
    await page.evaluate(() => { const r = window.__riddle; r.rules.rows.shift(); r.rulesChanged(); r.go({ kind: "camp" }); });
    await settle();
    await page.locator(".cmd .tile[data-tile=ledger]").click({ timeout: 5000 }); await sleep(300);   // Cut 17: the ledger is a console tile
    const chip = page.locator(".sheet-wrap .chip.counter");
    check((await chip.count()) === 1, "the bestiary shows the counter chip");
    await chip.click({ timeout: 5000 });
    await waitFor((s) => s?.screen === "camp", "camp"); await settle();
    const viaChip = await rowTexts();
    check(/^foe: boss \+? ?→ attack boss$/.test(viaChip[0]) && viaChip.length === after.length, `the bestiary chip inserts at the top too: "${viaChip[0]}"`);
  }
  // §3 `3 over` → `3 hp short` in the morgue's reading (`marginText`); the headline drops the hp margin altogether (QA on
  //    50bb162: four readers took `1 hp short` for the hp left); a lost companion reads `jackal Ashar fell`
  {
    await page.evaluate(() => { const r = window.__riddle; r.go({ kind: "death", death: { run_id: 1, depth: 3, cause: "goblin_pack", margin: "3 over", verdict: "gap", baseline: 0.4, trace: { turns: [] }, patches: [], morgue: "" }, lost: ["jackal · Ashar"] }); });
    await sleep(300);
    const d = await page.evaluate(() => ({ line: document.querySelector(".death-line")?.textContent.replace(/\s+/g, " ").trim(), why: document.querySelector(".death .death-why")?.textContent, egg: document.querySelector(".death .eggs-line .egg")?.textContent.replace(/\s+/g, " ").trim() }));
    // death v2: `no rule for it` is the why line under the banner
    check(/^goblin pack · D3 · you died$/.test(d.line ?? "") && d.why === "no rule for it", `the death line reads "${d.line}" · "${d.why}" (no hp margin)`);
    check(d.egg === "◯ jackal Ashar fell", `the lost companion reads "${d.egg}"`);
  }
  // §3 the report: `banked · returned · deaths` always (QA 92eb880 withdrew the larger-first swap); exit lines lead with `returned $61`; lost chips `fell`;
  //    the tiles fade in after an absence
  {
    const base = { elapsed_s: 3600, runs: 14, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "goblin_pack", n: 2 }], pending: [], reel: [], marks_earned: 1, live: null, tamed: [], hatched: [], lost: ["jackal · Ashar"], xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 } };
    await page.evaluate((b) => { const r = window.__riddle; b.live = r.lineage.live ?? null; r.go({ kind: "report", report: { ...b, banked: 0, returned: 12, exits: [{ carried: 102, keep_pct: 60, kept: 61, spent: 0, spent_on: [], text: "$102 carried · return keeps 60% → $61" }, { carried: 84, keep_pct: 100, kept: 84, spent: 0, spent_on: [], text: "$84 carried · bank keeps 100% → $84" }] }, absence: true }); }, base);
    await sleep(300);
    const rep = await page.evaluate(() => ({
      labels: [...document.querySelectorAll(".report .tiles .tile .label")].map((l) => l.textContent.trim()),
      fade: document.querySelector(".report .tiles")?.classList.contains("fade-in"),
      exits: [...document.querySelectorAll(".report .exit-lines .ledger-line")].map((l) => l.textContent.replace(/\s+/g, " ").trim()),
      lost: document.querySelector(".report .chip.egg")?.textContent.replace(/\s+/g, " ").trim(),
    }));
    check(rep.labels.join(" ") === "runs best marks full hauls runs returned deaths", `tiles in one order whatever leads (QA 92eb880): ${rep.labels.join(" · ")}`);
    // QA 23ed91f: the run rows read newest first (the gold sheet's order), so the later bank leads
    check(rep.exits[1]?.startsWith("returned $61 · $102 carried") && rep.exits[0]?.startsWith("collected $84 · "), `exit lines lead with the tier and the sum, newest first: "${rep.exits[0]}" · "${rep.exits[1]}"`);
    check(rep.lost === "◯ jackal Ashar fell", `the report's lost chip reads "${rep.lost}"`);
    check(rep.fade === true, "after an absence the tiles fade in");
    await page.evaluate((b) => { const r = window.__riddle; b.live = r.lineage.live ?? null; r.go({ kind: "report", report: { ...b, banked: 3, returned: 1 } }); }, base);
    await sleep(200);
    const rep2 = await page.evaluate(() => ({ labels: [...document.querySelectorAll(".report .tiles .tile .label")].map((l) => l.textContent.trim()), fade: document.querySelector(".report .tiles")?.classList.contains("fade-in") }));
    check(rep2.labels.join(" ") === "runs best marks full hauls runs returned deaths" && rep2.fade === false, `the same order when banks lead; a watched run's tiles do not fade (${rep2.labels.slice(3).join(" · ")})`);
  }
  // Cut 14 §4: a repeated chore callout coalesces on its line — `pick up ×8` — instead of eight `pick up` reads (the fake emits
  // no chore rows, so every engine batch gets one appended; the ticker is sampled every 40 ms through the run)
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
    await page.evaluate(() => {
      const r = window.__riddle, orig = r.engine.step.bind(r.engine);
      r.engine.step = async (n) => { const res = await orig(n); res.events.push({ t: res.snapshot.turn, k: "rule", row: -2, verb: { v: "pick_up" }, text: "pick up" }); return res; };
    });
    const seen = new Set(); let best = 0; const t0 = Date.now();
    while (Date.now() - t0 < 12_000) {
      const tk = await page.evaluate(() => document.querySelector(".ticker.show")?.textContent ?? "");
      if (tk) seen.add(tk);
      const m = /^pick up ×(\d+)$/.exec(tk); if (m) best = Math.max(best, Number(m[1]));
      if (best >= 4) break;
      if ((await state())?.screen !== "watch") break;
      await sleep(40);
    }
    const plain = [...seen].filter((x) => x === "pick up").length;
    check(best >= 4 && plain <= 1, `a repeated chore reads with a count: pick up ×${best} (lines: ${[...seen].filter((x) => /^pick up/.test(x)).slice(0, 4).join(" · ")})`);
  }
  }
  if (part("watch")) {
  // Cut 14: the `slowdowns` toggle off (settings; `riddle.slowdowns`) — the clock stays at the mode's flat 16× through a fake fight in
  // `fast` (the fight frame still opens); on again, the fight runs at 4×
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "camp");
    await page.locator("button.gear").click({ timeout: 5000 }); await sleep(250);
    const before = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .btn.slowdowns"); return { text: b?.textContent.trim(), on: b?.classList.contains("on"), row: b?.closest(".srow")?.querySelector(".label")?.textContent.trim() }; });
    await page.locator(".sheet-wrap .btn.slowdowns").click({ timeout: 5000 }); await sleep(150);
    const after = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .btn.slowdowns"); return { text: b?.textContent.trim(), on: b?.classList.contains("on"), stored: localStorage.getItem("riddle.slowdowns"), app: window.__riddle.slowdowns }; });
    check(before.row === "slowdowns" && before.text === "on" && before.on && after.text === "off" && !after.on && after.stored === "0" && after.app === false, `the settings row toggles slowdowns: ${before.text} → ${after.text} (stored ${after.stored})`);
    await page.keyboard.press("Escape"); await sleep(150);
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
    const watch = () => page.evaluate(() => { const w = document.querySelector(".watch"); return { frame: w?.dataset.frame, speed: Number(w?.dataset.speed), ending: w?.dataset.ending === "1", slow: window.__riddle.slowdowns, held: !!w?.dataset.hold }; });
    let fightSpeeds = [], seenFight = false; const t0 = Date.now();
    while (Date.now() - t0 < 30_000) {
      const s = await state(); if (s?.screen !== "watch") break;
      const w = await watch(); if (w.ending) break;
      if (w.frame === "fight") { seenFight = true; fightSpeeds.push(w.speed); if (fightSpeeds.length >= 8) break; }
      await sleep(60);
    }
    check(seenFight && fightSpeeds.length > 0 && fightSpeeds.every((x) => x === 32), `slowdowns off: the fight frame opens and the clock stays at the flat 32× (Cut 20 §3; ${[...new Set(fightSpeeds)].join("/") || "no fight"})`);
    // on again — a fresh run (the first may have ended by now), the toggle persisted
    await page.evaluate(() => window.__riddle.setSlowdowns(true));
    // (frames sampled as they come: `measured` takes the run once more on a loaded machine — tests/lib/load.mjs)
    const slow = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch again");
    fightSpeeds = []; const t1 = Date.now();
    // (one round trip a sample, the screen with the frame: two a sample and a 60 ms sleep caught two samples of a short fight
    // on a loaded machine)
    const sample = () => page.evaluate(() => { const w = document.querySelector(".watch"); return { screen: window.__riddle?.screen, frame: w?.dataset.frame, speed: Number(w?.dataset.speed), ending: w?.dataset.ending === "1", held: !!w?.dataset.hold }; });
    while (Date.now() - t1 < 60_000) {   // Cut 18: fast passes cheap fights at travel speed, so a framed one can take longer to come
      const w = await sample(); if (w.screen !== "watch" || w.ending) break;
      // Cut 18 §1: a held beat eases the picture to its stop; that is the beat's rate, not the fight's
      if (w.frame === "fight" && !w.held) { fightSpeeds.push(w.speed); if (fightSpeeds.filter((x) => x === 4).length >= 4) break; }
      await sleep(30);
    }
    // the frame's first/last sample can straddle the cut (the travel rate on either side): the fight's own rate is the median of
    // its samples. A framed fight plays at 4× (≥ 3 samples); the frame's tail may run on at the travel rate — 32× since Cut 20 §3
    // (the `slowdowns off` check above reads it; 16× before) — nothing else is allowed
    return { ok: fightSpeeds.filter((x) => x === 4).length >= 3 && fightSpeeds.every((x) => x === 4 || x === 16 || x === 32), line: `slowdowns on: a fight in fast runs at 4× (${fightSpeeds.join("/") || "no fight"})` };
    });
    check(slow.ok, slow.line);
  }
  // Cut 14 §6: the world runs on the wall clock — paused, the frontier (`data-frontier`) advances and the strip's dot beats
  // (`data-pulses`) while the playhead (`data-tick`, the strip's head) holds; `▶▶|` from behind lands on the frontier; a faked
  // hidden tab for 5 s advances the world ≥ 40 ticks; a run that ends while paused still shows its exit after the replay
  {
    const watch = () => page.evaluate(() => { const w = document.querySelector(".watch"), h = document.querySelector(".scrub .head"); return { screen: window.__riddle.screen, frame: w?.dataset.frame, speed: Number(w?.dataset.speed), tick: Number(w?.dataset.tick), frontier: Number(w?.dataset.frontier), pulses: Number(w?.dataset.pulses), ending: w?.dataset.ending === "1", card: w?.dataset.card, head: h ? parseFloat(h.style.left) : NaN, strip: !!document.querySelector(".scrub:not([hidden])") }; });
    const press = (l) => page.evaluate((l) => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === l) { b.click(); return true; } return false; }, l);
    // paused in a shown fight (`fights`): the picture holds, the world goes on
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&autosend=1&speed=fights&early=0`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
    let w = null; const t0 = Date.now();
    while (Date.now() - t0 < 30_000) { w = await watch(); if (w.screen !== "watch" || (w.frame === "fight" && w.tick > 0)) break; await sleep(50); }
    check(w?.frame === "fight" && w.strip, `a fight is up and the strip shows (frame ${w?.frame}, strip ${w?.strip})`);
    await press("⏸"); await sleep(200);
    const p0 = await watch(); await sleep(3000); const p1 = await watch();
    check(p0.speed === 0 && p1.tick === p0.tick && p1.frontier >= p0.frontier + 40 && p1.pulses > p0.pulses && p1.head < p0.head, `paused: the playhead holds at ${p0.tick} while the frontier runs ${p0.frontier} → ${p1.frontier} (dot beats ${p0.pulses} → ${p1.pulses}, head ${p0.head.toFixed(1)}% → ${p1.head.toFixed(1)}%)`);
    // ▶▶| lands on the frontier (live) — and plays on
    const F = p1.frontier;
    await press("▶▶|"); await sleep(400);
    const l1 = await watch();
    check(l1.tick >= F - 1 && (l1.speed > 0 || l1.card === "1"), `▶▶| from behind lands on the frontier (tick ${l1.tick} ≥ ${F}, speed ${l1.speed}, card ${l1.card})`);
    // a hidden tab: the picture freezes, the world runs on wall time; back, the viewer is live again
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the fast watch");
    await sleep(1500);
    const setHidden = (v) => page.evaluate((v) => { Object.defineProperty(document, "hidden", { value: v, configurable: true }); document.dispatchEvent(new Event("visibilitychange")); }, v);
    await setHidden(true); await sleep(200);
    const h0 = await watch(); await sleep(5000); const h1 = await watch();
    await setHidden(false); await sleep(1200);
    const h2 = await watch();
    check(h1.tick === h0.tick && h1.frontier >= h0.frontier + 40, `hidden 5 s: the picture held at ${h0.tick} while the world ran ${h0.frontier} → ${h1.frontier}`);
    check(h2.speed > 0 && h2.frontier - h2.tick <= 60, `back, the viewer is live (tick ${h2.tick}, frontier ${h2.frontier}, speed ${h2.speed})`);
    // a run that ends while paused (seed 157's default set dies on D1): the world reaches the exit; the exit waits for the replay
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the dying watch");
    const t1 = Date.now(); let e = null;
    while (Date.now() - t1 < 10_000) { e = await watch(); if (e.screen !== "watch" || e.tick >= 30) break; await sleep(20); }
    await press("⏸"); await sleep(100);
    const e0 = await watch();
    const t2 = Date.now(); let e1 = e0;
    while (Date.now() - t2 < 40_000) { e1 = await watch(); if (e1.screen !== "watch" || e1.ending) break; await sleep(100); }
    await sleep(1500); const e2 = await watch();
    check(e0.screen === "watch" && e2.screen === "watch" && e2.tick === e0.tick && e2.ending && e2.frontier > e0.frontier, `the run ended while paused (frontier ${e0.frontier} → ${e2.frontier}, ending ${e2.ending}) and the exit waits (tick ${e2.tick}, ${e2.screen})`);
    // resumed, the replay runs at the viewer's rate (a fight-heavy 375 ticks takes its time); `▶▶|` lands on the ending's start
    // and the last ENDING_TICKS play at 1× before the exit flow
    await press("▶"); await sleep(1500);
    const r1 = await watch();
    check(r1.screen === "watch" && r1.tick > e2.tick && r1.tick < e2.frontier - 20, `resumed, the replay plays on (tick ${e2.tick} → ${r1.tick} of ${e2.frontier})`);
    await press("▶▶|"); await sleep(300);
    const r2 = await watch();
    check(r2.screen === "watch" && r2.tick >= e2.frontier - 22 && r2.speed === 2, `▶▶| lands on the ending's start at 2× (Cut 20 §3: fast walks out at 2×; tick ${r2.tick}, frontier ${e2.frontier}, speed ${r2.speed})`);
    const s2 = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit", "the exit after the walk-out", 30_000);
    check(s2.screen === "death", `then the exit flow (${s2.screen})`);
  }
  }
  if (part("hold")) {
  // Cut 14 §6 (QA on 56f2a1d): `⏸` freezes the picture on the tap — the HUD, the card, the ticker and the viewer's tick stay as
  // they were for 1.5 s whatever the world does underneath (seed 516 in `fights`: the HUD ran `D1 34/36 $23` → `D3 31/36 $52` → a
  // beat after the tap: a card/travel drain and a fight cut kept moving the picture); sampled every 100 ms, three taps a run
  {
    const pic = () => page.evaluate(() => {
      const w = document.querySelector(".watch"), top = document.querySelector(".hud.top"), card = document.querySelector(".interstitial");
      return { screen: window.__riddle.screen, over: w?.dataset.over === "1", key: [top ? [...top.querySelectorAll(".num, .stake")].map((x) => x.textContent).join("|") : "", card && !card.hidden ? card.textContent : "", document.querySelector(".ticker")?.textContent ?? "", w?.dataset.tick, w?.dataset.frame].join(" ¦ "), frontier: Number(w?.dataset.frontier) };
    });
    const press = (l) => page.evaluate((l) => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === l) { b.click(); return true; } return false; }, l);
    for (const seed of [516, 5]) {
      await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}&autosend=1&speed=fights&early=0`, { waitUntil: "domcontentloaded" });
      await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
      for (const wait of [3000, "card", 1200, "card", 700]) {
        // a number: a tap that long after the last; "card": a tap while the floor card is up (the travel runs under it)
        if (wait === "card") { const t = Date.now(); while (Date.now() - t < 8000 && (await page.evaluate(() => document.querySelector(".watch")?.dataset.card)) !== "1") await sleep(20); }
        else await sleep(wait);
        const a = await pic(); if (a.screen !== "watch" || a.over) break;
        await press("⏸");
        const k0 = (await pic()).key; const moved = []; let last = null;
        for (let i = 0; i < 15; i++) { await sleep(100); last = await pic(); if (last.key !== k0) moved.push(last.key); }
        if (last.over) break;   // the run's end unpauses (endControls)
        check(moved.length === 0 && last.frontier > a.frontier, `seed ${seed}${wait === "card" ? " (card)" : ""}: ⏸ holds the picture 1.5 s (${k0}${moved.length ? ` → ${moved[0]}` : ""}; frontier ${a.frontier} → ${last.frontier})`);
        await press("▶");
      }
    }
  }
  // Cut 14 (QA on 56f2a1d): one position on screen — the floor card names the floor the HUD shows (and its loot is the stake's):
  // HUD `17/40 D4` under `D5 · 15 rooms · a shrine`, `$6 · keeps $3` under `D2 · 16 rooms · $13`; sampled every 100 ms through
  // a `fights` run
  {
    for (const seed of [516, 7]) {
      await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}&autosend=1&speed=fights&early=0`, { waitUntil: "domcontentloaded" });
      await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
      let samples = 0, cards = 0; const bad = []; const t0 = Date.now();
      while (Date.now() - t0 < 20_000) {
        const x = await page.evaluate(() => {
          const card = document.querySelector(".interstitial");
          return { screen: window.__riddle.screen, depth: document.querySelector(".watch .depth")?.textContent ?? "", stake: document.querySelector(".watch .stake")?.textContent ?? "", card: card && !card.hidden ? card.textContent : null };
        });
        if (x.screen !== "watch") break;
        samples++;
        if (x.card !== null) {
          cards++;
          const cd = /^D(\d+)/.exec(x.card)?.[1], hd = /^D(\d+)$/.exec(x.depth)?.[1];
          const cl = /· (?:carry )?\$(\d+)$/.exec(x.card)?.[1], sl = /^(?:carry )?\$(\d+)/.exec(x.stake)?.[1];
          if (cd !== hd || (cl !== undefined && sl !== undefined && cl !== sl)) bad.push(`${x.depth} ${x.stake.split(" · ")[0]} vs "${x.card}"`);
        }
        await sleep(100);
      }
      check(cards >= 5 && bad.length === 0, `seed ${seed}: the card names the HUD's floor and loot on every sample (${cards}/${samples} with the card up${bad.length ? `; ${bad.length} off: ${bad.slice(0, 3).join(" · ")}` : ""})`);
    }
  }
  }
  if (part("card")) {
  // Cut 14 §4: the `rest 20m` banner never covers the death frame's callout line — it sits low (`.banner.rest`), under every
  // sprite and name the frame drew (rater S: `rest 20m` over `OGRE WINDS UP`)
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
    await sleep(500);
    await page.evaluate(() => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === "▶▶|") b.click(); });
    let rest = null; const t0 = Date.now();
    while (Date.now() - t0 < 40_000) {
      rest = await page.evaluate(() => {
        const b = document.querySelector(".banner.show.rest"); if (!b) return null;
        // Cut 17: the view is the stage between the bar and the console — the banner's place is read in the canvas's frame
        const v = window.__viewer, r = b.getBoundingClientRect(), c = document.querySelector(".watch .view").getBoundingClientRect();
        const drawn = [...v.debugRects().map((x) => x.y + x.h), ...v.debugLabels().map((l) => l.y)];
        return { text: b.textContent, top: r.top - c.top, frame: document.querySelector(".watch")?.dataset.frame, lowest: Math.max(0, ...drawn), h: c.height };
      });
      if (rest) break;
      const s = await state(); if (s?.screen !== "watch" && s?.screen !== "exit") break;
      await sleep(40);
    }
    // QA 778fa1b (qaU: `♟2 · rest 20m` announced the next heir on the death's last frame): after a death no rest beat — the death screen
    // comes first; a run that came home still shows it, low
    const endScreen = rest ? null : (await state())?.screen;
    if (!rest && endScreen === "death") check(true, "after a death no rest beat: the death screen comes first");
    else check(!!rest && /^rest \S+$/.test(rest.text) && rest.top > rest.lowest && rest.top >= rest.h * 0.75, `the rest banner sits under the frame's sprites and names (${rest ? `"${rest.text}" top ${Math.round(rest.top)} · drawn to ${Math.round(rest.lowest)} · ${rest.frame} frame` : "never seen"})`);
  }
  // Cut 15 §4: the lit mode chip carries its clock as small digits (`data-rate`, drawn by `::after` with a trailing `×` — QA on 3d71c33; the chip's text stays its word):
  // `fast 16` on the travel, `fast 4` in a fight; `fights 2` in a fight; the other chip carries none
  {
    const chip = () => page.evaluate(() => { const w = document.querySelector(".watch"), on = document.querySelector(".cmd .hud-btn.on"), off = [...document.querySelectorAll(".cmd .hud-btn")].filter((b) => b !== on && b.dataset.rate); return { screen: window.__riddle.screen, frame: w?.dataset.frame, speed: Number(w?.dataset.speed), card: w?.dataset.card, text: on?.textContent, rate: on?.dataset.rate ?? "", after: on ? getComputedStyle(on, "::after").content : "", others: off.length }; });
    // (the chip and the clock read in one sample, as frames come: `measured` takes the run once more on a loaded machine)
    for (const mode of ["fast", "fights"]) {
      const lit = await measured(async () => {
      await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=${mode}&early=0`, { waitUntil: "domcontentloaded" });
      await waitFor((s) => s?.booted && s.screen === "watch", `the ${mode} watch`);
      const seen = new Map(); let bad = null; const t0 = Date.now();
      const word = mode === "fights" ? "fights only" : mode;   // (the mode id stays `fights`; its button reads `fights only`)
      while (Date.now() - t0 < 25_000 && !(seen.has("fight") && seen.has("map"))) {
        const c = await chip(); if (c.screen !== "watch") break;
        const want = c.speed > 0 ? (c.speed >= 2 ? String(Math.round(c.speed)) : String(Math.round(c.speed * 10) / 10)) : c.card === "1" ? "16" : "";   // QA 92eb880: the chip's rate rounds (`7`, never `6.666…`)
        if (c.rate !== want || c.text !== word || c.others || (c.rate && c.after !== `"${c.rate}×"`)) bad ??= c;
        if (c.rate && c.frame) seen.set(c.frame, `${c.text} ${c.rate}`);
        await sleep(50);
      }
      return { ok: !bad && seen.has("fight") && [...seen.values()].every((x) => new RegExp(`^${word} \\d+$`).test(x)), line: `${mode}: the lit chip carries its rate (${[...seen].map(([f, x]) => `${f}: ${x}`).join(" · ")}${bad ? `; off: ${JSON.stringify(bad)}` : ""})` };
      });
      check(lit.ok, lit.line);
    }
  }
  // Cut 15 §4: the floor card is short — every span it is up lasts ≤ 1.2 s of wall time, and it is never up over the fight frame
  // (a MutationObserver on the watch's `data-card` / `data-frame`, two `fights` runs of 20 s)
  {
    // (a wall-clock reading: `measured` takes a seed's run once more on a loaded machine, the bar unchanged — tests/lib/load.mjs)
    for (const seed of [516, 7]) {
      const card = await measured(async () => {
      await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}&autosend=1&speed=fights&early=0`, { waitUntil: "domcontentloaded" });
      await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
      await page.evaluate(() => {
        const w = document.querySelector(".watch"); const log = window.__cardLog = { spans: [], over: 0, since: w.dataset.card === "1" ? performance.now() : null };
        new MutationObserver(() => {
          const up = w.dataset.card === "1", now = performance.now();
          if (up && w.dataset.frame === "fight") log.over++;
          if (up && log.since === null) log.since = now;
          else if (!up && log.since !== null) { log.spans.push(now - log.since); log.since = null; }
        }).observe(w, { attributes: true, attributeFilter: ["data-card", "data-frame"] });
      });
      const t0 = Date.now();
      while (Date.now() - t0 < 20_000) { if ((await state())?.screen !== "watch") break; await sleep(200); }
      const log = await page.evaluate(() => window.__cardLog);
      const max = Math.max(0, ...log.spans);
      return { ok: log.spans.length >= 3 && max <= 1200 && log.over === 0, line: `seed ${seed}: the floor card is up ≤ 1.2 s a time (${log.spans.length} cards, longest ${Math.round(max)} ms, over a fight ${log.over})` };
      });
      check(card.ok, card.line);
    }
  }
  // Cut 15 §5 → Cut 19 §1: a watched cage is a beat — `took mail` (the preference's pick, `VaultChoice.pick`) held on the ticker while
  // the world waits; in `fast` a tap on the beat opens the override sheet (its bar runs 10 s, the world waits while it is open) and a
  // chip takes the tap; in `fights`, untouched, no sheet ever opens and the world goes on after the hold (the preference picks). The
  // fake has no vaults: one engine batch after tick 20 carries a `vault_choice`.
  {
    for (const mode of ["fast", "fights"]) {
      await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=${mode}&early=0`, { waitUntil: "domcontentloaded" });
      await waitFor((s) => s?.booted && s.screen === "watch", `the ${mode} watch`);
      await page.evaluate(() => {
        const r = window.__riddle, orig = r.engine.step.bind(r.engine), origChoose = r.engine.choose.bind(r.engine); let done = false;
        r.__chosen = null; window.__sheetSeen = false;
        r.engine.step = async (n) => { const res = await orig(n); if (!done && res.snapshot.turn > 20 && !res.run_over && !res.events.some((e) => e.k === "exit")) { done = true; res.snapshot.vault_choice = { items: [{ id: 9901, kind: "sword", known: true, label: "sword" }, { id: 9902, kind: "mail", known: true, label: "chain mail" }, { id: 9903, kind: "heal", known: true, label: "heal potion" }], left: 50, pick: 9902 }; } return res; };
        r.engine.choose = async (id) => { r.__chosen = id; return origChoose(id); };
        new MutationObserver(() => { if (document.querySelector(".sheet-wrap .vault-choice")) window.__sheetSeen = true; }).observe(document.body, { childList: true, subtree: true });
      });
      const w = () => page.evaluate(() => { const x = document.querySelector(".watch"); const t = document.querySelector(".watch .ticker"); const g = document.querySelector(".sheet-wrap .vault-choice .grace i"); return { screen: window.__riddle.screen, frontier: Number(x?.dataset.frontier), tick: Number(x?.dataset.tick), beat: t?.classList.contains("cage") && t.classList.contains("show") ? t.textContent.trim() : "", sheet: !!document.querySelector(".sheet-wrap .vault-choice .chip"), bar: g ? g.style.transitionDuration : "", cage: x?.dataset.cage ?? "" }; });
      let a = null; const t0 = Date.now();
      while (Date.now() - t0 < 20_000) { a = await w(); if (a.beat || (a.screen !== "watch" && a.screen !== "exit")) break; await sleep(40); }
      if (!a?.beat) { check(false, `${mode}: the cage beat showed (${a?.screen}, cage ${a?.cage})`); continue; }
      check(a.beat === "took chain mail" && !a.sheet, `${mode}: the cage is a beat naming the pick, no sheet (\`${a.beat}\`)`);
      await sleep(250); const f0 = await w();
      await sleep(1500); const f1 = await w();
      check(f1.frontier === f0.frontier, `${mode}: the world waits while the beat holds (frontier ${f0.frontier} → ${f1.frontier})`);
      if (mode === "fast") {
        await page.locator(".watch .ticker.cage.show").click({ timeout: 1500 }).catch(() => {});
        await sleep(300); const s0 = await w();
        check(s0.sheet && s0.bar === "10s", `${mode}: a tap on the beat opens the override (sheet ${s0.sheet}, bar ${s0.bar})`);
        await sleep(3000); const s1 = await w();
        check(s1.sheet && s1.frontier === s0.frontier, `${mode}: the sheet open 3 s past the beat, the world waits (frontier ${s0.frontier} → ${s1.frontier})`);
        await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {});
        await sleep(1500);
        const f2 = await w(); const chosen = await page.evaluate(() => window.__riddle.__chosen);
        check(chosen === 9901 && !f2.sheet && (f2.frontier > s1.frontier || f2.screen !== "watch"), `${mode}: a chip takes the tap (chose ${chosen}), the world goes on (frontier ${s1.frontier} → ${f2.frontier})`);
      } else {
        const opened = Date.now();
        let f2 = f1; while (Date.now() - opened < 8000 && f2.frontier === f1.frontier && f2.screen === "watch") { await sleep(50); f2 = await w(); }
        const chosen = await page.evaluate(() => window.__riddle.__chosen), seen = await page.evaluate(() => window.__sheetSeen);
        check(!seen && chosen === null, `${mode}: untouched, no sheet opens (seen ${seen}), the preference picks (chose ${chosen})`);
        check(f2.frontier > f1.frontier || f2.screen !== "watch", `${mode}: after the hold the world goes on (${Date.now() - opened + 1750} ms after the beat; frontier ${f1.frontier} → ${f2.frontier})`);
      }
    }
  }
  }

  if (part("paint")) {
  // Cut 20 §3: an edit's forecast paints within 1.2 s though the slow measures are in flight — the fake behind a worker's timing
  // (`fake_lag=1`: forecast 0.35 s, refine 2.5 s, unlock deltas 3 s, one call at a time per lane); the refine, the unlock deltas and
  // the cage options ride the second lane (engine/lanes.ts), so the first pass never queues behind them; the refine lands after it
  // (wall-clock readings, here and below: `measured` takes the scene once more on a loaded machine, the bars unchanged)
  {
    const paint = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&fake_lag=1`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "camp (lagged)", 30_000);
    const t = await page.evaluate(async () => {
      const r = window.__riddle, log = [];
      r.onForecast((f) => log.push({ t: performance.now(), refined: f.refined }));
      const until = (pred, ms) => new Promise((res) => { const t0 = performance.now(); const tick = () => { if (pred() || performance.now() - t0 > ms) res(pred()); else setTimeout(tick, 20); }; tick(); });
      await until(() => log.some((x) => x.refined === false), 8000);                       // the camp's first paint
      await new Promise((res) => setTimeout(res, 2300));                                   // the refine (2 s after it) is in flight now
      void r.engine.unlockDeltas(); void r.engine.cageForecast();                          // …and more slow measures queued behind it
      const at = performance.now(); log.length = 0;
      r.insertRow({ conds: [{ k: "hp<", n: 30 }], verb: { v: "rest" } }, 0);               // an edit
      await until(() => log.some((x) => x.refined === true), 12_000);
      const first = log.find((x) => x.refined === false), refine = log.find((x) => x.refined === true);
      return { first: first ? Math.round(first.t - at) : -1, refine: refine ? Math.round(refine.t - at) : -1 };
    });
    return { ok: t.first >= 0 && t.first <= 1200 && t.refine > t.first, line: `an edit's first forecast paints ≤ 1.2 s with the refine and the unlock deltas in flight (${t.first} ms)`, t };
    });
    const t = paint.t, retried = paint.line.includes(" [retried") ? paint.line.slice(paint.line.indexOf(" [retried")) : "";
    check(t.first >= 0 && t.first <= 1200, `an edit's first forecast paints ≤ 1.2 s with the refine and the unlock deltas in flight (${t.first} ms)${retried}`);
    check(t.refine > t.first, `the refine lands after the first paint (${t.refine} ms)`);
  }
  // Cut 24 §4 (AK: "each edit makes you wait 3–7 s for the forecast to settle"): the real engine (wasm in its workers, an 8 h lineage):
  // an edit's first pass paints ≤ 1 s (the median of three quiet edits) and its refine lands ≤ 3 s — quiet, after a burst of four edits
  // 250 ms apart (from the last; the stale refines never queue: the refine lane runs the latest only), and with the forge's, the
  // unlocks' and the cage's measures in flight (they ride the background lane, the refine its own)
  if (!process.argv.includes("--no-wasm")) {
    const wasm = await measured(async () => {
    await page.goto(`${url}?dev=1&fresh=1&seed=2302&absent=8h`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && ["camp", "report"].includes(s.screen), "the real engine's camp", 120_000);
    const kind = await page.evaluate(() => window.__riddle.kind);
    // Cut 30: the editor comes with the pen (the Mother met and 72 h) — the save opens it, the core recompiles on load
    await page.evaluate(async () => {
      const r = window.__riddle;
      if (r.lineage.town?.home === false) await r.mutate(() => r.engine.buildTown("house"), "build");
      const e = JSON.parse(await r.engine.save());
      e.lineage.pkg.pen_open = true;
      for (const id of ["pen", "edit", "dial", "unlocks", "reorder", "vs", "tags", "walls", "divergence", "route"]) if (!e.lineage.systems.includes(id)) e.lineage.systems.push(id);
      await r.importSave(JSON.stringify({ v: 2, engine: JSON.stringify(e), loadout: [], last_seen: Date.now(), runs: 0 }));
    });
    await waitFor((s) => s?.booted && s.screen === "camp", "the pen's camp", 60_000);
    await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
    await sleep(6000);   // the camp's own measures settle
    const t = await page.evaluate(async () => {
      const r = window.__riddle, log = [];
      r.onForecast((f) => log.push({ t: performance.now(), refined: f.refined }));
      const until = (pred, ms) => new Promise((res) => { const t0 = performance.now(); const tick = () => { if (pred() || performance.now() - t0 > ms) res(pred()); else setTimeout(tick, 15); }; tick(); });
      const wait = (ms) => new Promise((res) => setTimeout(res, ms));
      const took = async (at) => { await until(() => log.some((x) => x.refined === true), 15_000); return { first: Math.round((log.find((x) => x.refined === false)?.t ?? NaN) - at), refine: Math.round((log.find((x) => x.refined === true)?.t ?? NaN) - at) }; };
      r.insertRow({ conds: [{ k: "hp<", n: 30 }], verb: { v: "rest" } }, 0);
      await until(() => log.some((x) => x.refined === true), 15_000); await wait(1000);
      const quiet = [];
      for (const n of [40, 50, 60]) { log.length = 0; const at = performance.now(); r.rules.rows[0].conds[0].n = n; r.rulesChanged(); quiet.push(await took(at)); await wait(1000); }
      for (const n of [25, 35, 45, 55]) { r.rules.rows[0].conds[0].n = n; r.rulesChanged(); log.length = 0; await wait(250); }
      const burst = await took(performance.now() - 250); await wait(1000);
      void r.engine.kitDeltas?.(); void r.engine.unlockDeltas(); void r.engine.cageForecast?.(); await wait(100);
      log.length = 0; const at = performance.now(); r.rules.rows[0].conds[0].n = 20; r.rulesChanged();
      const busy = await took(at);
      return { quiet, burst, busy };
    });
    const med = [...t.quiet.map((q) => q.first)].sort((a, b) => a - b)[1];
    const refines = [...t.quiet.map((q) => q.refine), t.burst.refine, t.busy.refine];
    const paint = { ok: med <= 1000 && t.burst.first <= 1200 && t.busy.first <= 1200, line: `real wasm (${kind}): an edit's first pass paints ≤ 1 s (quiet ${t.quiet.map((q) => q.first).join("/")} ms, median ${med}; after a burst ${t.burst.first}; slow measures in flight ${t.busy.first})` };
    const refine = { ok: refines.every((x) => x <= 3000), line: `real wasm: the refine lands ≤ 3 s (quiet ${t.quiet.map((q) => q.refine).join("/")} ms; after a burst ${t.burst.refine}; slow measures in flight ${t.busy.refine})` };
    return { ok: kind === "wasm" && paint.ok && refine.ok, line: `${paint.line} · ${refine.line}`, kind, paint, refine };
    });
    const retried = wasm.line.includes(" [retried") ? wasm.line.slice(wasm.line.indexOf(" [retried")) : "";
    check(wasm.kind === "wasm", `the real engine answers (${wasm.kind})`);
    check(wasm.paint.ok, wasm.paint.line + retried);
    check(wasm.refine.ok, wasm.refine.line);
  }
  }
  if (part("deep")) {
  // Cut 25 §4 (AM: 5–9 s of `…` after an edit, ~8 s for the forge's estimates on a D11 lineage after an absence): the deep fixture
  // (`tests/fixtures/deep.json`, the core's engine save of AM's shape — `crates/riddle-core/examples/deep.rs`), real wasm: the camp's
  // first paint and refine, an edit's (the median of three), and the forge's estimates opened from the camp — reported against the
  // contract's bars (first paint ≤ 1.2 s, refine ≤ 3 s, forge ≤ 3 s); `RIDDLE_DEEP_GATE=1` makes them checks (headed, a quiet machine)
  const deepPath = process.env.RIDDLE_DEEP ?? resolve(ROOT, "web/tests/fixtures/deep.json");   // (`RIDDLE_DEEP=path` another save)
  if (!process.argv.includes("--no-wasm") && existsSync(deepPath)) {
    const save = readFileSync(deepPath, "utf8").trim();
    const engine = save.startsWith("{\"v\"") && JSON.parse(save).engine ? JSON.parse(save).engine : save;   // an engine save, or a client blob
    await page.goto(`${url}?dev=1&fresh=1&seed=2501`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "the real engine's camp", 120_000);
    const ok = await page.evaluate((engine) => window.__riddle.importSave(JSON.stringify({ v: 2, engine, loadout: [], last_seen: Date.now(), runs: 0 })), engine);
    await waitFor((s) => s?.booted && s.screen === "camp", "the deep camp", 60_000);
    const t = await page.evaluate(async () => {
      const r = window.__riddle, log = [];
      r.onForecast((f) => log.push({ t: performance.now(), refined: f.refined }));
      const until = (pred, ms) => new Promise((res) => { const t0 = performance.now(); const tick = () => { if (pred() || performance.now() - t0 > ms) res(pred()); else setTimeout(tick, 15); }; tick(); });
      const wait = (ms) => new Promise((res) => setTimeout(res, ms));
      const took = async (at) => { await until(() => log.some((x) => x.refined === true), 30_000); return { first: Math.round((log.find((x) => x.refined === false)?.t ?? NaN) - at), refine: Math.round((log.find((x) => x.refined === true)?.t ?? NaN) - at) }; };
      log.length = 0; let at = performance.now(); r.go({ kind: "camp" });
      const camp = await took(at); await wait(8000);
      const i = r.rules.rows.findIndex((x) => x.conds.some((c) => c.n !== undefined));
      const c = r.rules.rows[i]?.conds.find((x) => x.n !== undefined);
      const edits = [];
      for (const d of [5, -5, 10]) { if (!c) break; log.length = 0; at = performance.now(); c.n = Math.max(5, c.n + d); r.rulesChanged(); edits.push(await took(at)); await wait(1500); }
      await wait(4000);   // the quiet camp (its prefetch)
      at = performance.now(); document.querySelector(".cmd .tile[data-tile=forge]")?.click();
      await until(() => { const m = [...document.querySelectorAll(".sheet-wrap .forge .kit-move")]; return m.length > 0 && m.every((x) => !/…/.test(x.textContent)); }, 30_000);
      const forge = Math.round(performance.now() - at);
      // Cut 27 §2: the edit as a scene — a big edit (the set's first row cut), its refine, the core's divergence, the scene's first frame
      log.length = 0; r.go({ kind: "camp" }); await took(performance.now()); await wait(1500);
      window.__scene = {}; log.length = 0; at = performance.now();
      r.rules.rows.splice(0, 1); r.rulesChanged();
      const sceneOn = await until(() => !!window.__scene.playAt || window.__scene.last === null || (window.__scene.answerAt && !document.querySelector(".camp .div-scene:not([hidden])")), 30_000);
      await wait(300);
      const sc = window.__scene;
      const scene = { refine: Math.round((sc.refineAt ?? NaN) - at), answer: Math.round((sc.answerAt ?? NaN) - (sc.refineAt ?? NaN)), play: sc.playAt ? Math.round(sc.playAt - sc.refineAt) : null, found: sc.last ? `${sc.last.moved.toFixed(2)}${sc.last.inside ? " inside" : ""}` : sc.last === null ? "none" : "?", ok: sceneOn };
      return { camp, edits, forge, scene, best: r.lineage.best_depth, kit: !!document.querySelector(".sheet-wrap .forge .kit-move") };
    });
    const med = t.edits.map((e) => e.first).sort((a, b) => a - b)[1], medR = t.edits.map((e) => e.refine).sort((a, b) => a - b)[1];
    const line = `deep D${t.best} (real wasm): camp first ${t.camp.first} ms · refine ${t.camp.refine}; an edit's first ${t.edits.map((e) => e.first).join("/")} (median ${med}) · refine ${t.edits.map((e) => e.refine).join("/")} (median ${medR}); forge ${t.kit ? `${t.forge} ms` : "no step"} — bars 1.2 s · 3 s · 3 s`;
    if (process.env.RIDDLE_DEEP_GATE === "1") check(ok && med <= 1200 && medR <= 3000 && (!t.kit || t.forge <= 3000), line);
    else out.push(`note ${line}`);
    // Cut 27 §2: the scene ≤ 1.5 s after the refine (the divergence on the refine's lane, the renderer's context made while it runs)
    const sline = `deep D${t.best} (real wasm): an edit's scene — refine ${t.scene.refine} ms after the edit, the divergence ${t.scene.answer} ms after the refine (move ${t.scene.found}), the scene's first frame ${t.scene.play ?? "—"} ms after the refine — bar 1.5 s`;
    if (process.env.RIDDLE_DEEP_GATE === "1") check(t.scene.play !== null && t.scene.play <= 1500, sline);
    else out.push(`note ${sline}`);
  }
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`clarity${PART === "all" ? "" : ` ${PART}`}: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`clarity${PART === "all" ? "" : ` ${PART}`}: ok (${out.length} checks)`);
