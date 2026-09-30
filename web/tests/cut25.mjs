#!/usr/bin/env node
// Cut 25 client gates (docs/CUT25.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §2  an `order` death (the core's `verdict: "order"`, `cause_row`, `order_over`): the headline names both rows (`R5 under R2`), the
//       seal reads ORDER, the lead patch reads `move R5 above R2`; the gem moves the row (nothing added, nothing cut)
//   §3  a drain (the hero's hp or max hp falling with no blow: `hunger`) is no progress and no fight — its word shows once a floor
//       (`starving`), never its `hunger −1 max` bite after bite, and a drain stretch plays ≤ 5 s of wall time in `fights`, `fast` and the
//       plain `1×` (`?fake_drain=1`); the mode picker offers `1×` beside fights/fast (remembered; the send gem says it); the black frame
//       (AM: a new floor's own beat held its load behind the stair's fade — 3 s of black) — real wasm, seed 2501's first run, no dark
//       stretch over 400 ms (the stairs' fade-through) (`--no-wasm` skips it)
//   §4  the slow measures ride lanes of their own: the forge's `kitDeltas` never waits behind the unlock shelf's `unlockDeltas`
//       (`?fake_lag=1`: 3 s each — both land in ~3 s, not 6); the deep-lineage timings are reported by clarity.mjs / the Cut 25 notes
//   §6  the unlock grid keeps its cells under a buy (the bought card's cell stays, `✓`; no other card moves); a sheet opened from a sheet
//       replaces it (the first hidden, a back `‹`; back returns to it, the `×` closes both); the row card at the cap reads `max` and its
//       gate, never the next price; the forge's armed step keeps its place (`ok $N` where the price stood) and the second tap there buys;
//       an offline trace shows every blow (`Trace.blows`); the reel's merged line reads `×n`; a watched run's RUNS tile names its heir
//
//   node web/tests/cut25.mjs [--shots dir] [--no-wasm]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows, openPanel } from "./lib/frame.mjs";
import { measured } from "./lib/load.mjs";

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
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: true }); };
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const camp = async () => { await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250); };
const richSave = (gold, marks) => page.evaluate(async ([gold, marks]) => { const b = JSON.parse(window.__riddle.exportSave()); const e = JSON.parse(b.engine); e.lineage.gold = gold; if (marks !== undefined) e.lineage.marks = marks; e.lineage.best_depth = Math.max(5, e.lineage.best_depth); b.engine = JSON.stringify(e); return window.__riddle.importSave(JSON.stringify(b)); }, [gold, marks]);

try {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=25`, { waitUntil: "domcontentloaded" });
  await camp();

  // ---- §2: the `order` verdict
  {
    const rows = [
      { conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" }, origin: "player" }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" },
      { conds: [{ k: "foe_tag", t: "ranged" }], verb: { v: "attack", a: "tag:ranged" }, origin: "player" }, { conds: [{ k: "depth>=", n: 8 }], verb: { v: "bank" }, origin: "player" },
      { conds: [{ k: "hp<", n: 20 }], verb: { v: "return" }, origin: "player" },
    ];
    await page.evaluate((rows) => { const r = window.__riddle; r.sets[r.active] = { ...r.sets[r.active], rows }; r.vocab = { ...r.vocab, max_rows: 6 }; }, rows);
    await page.evaluate((rows) => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 8, cause: "ogre", margin: "", verdict: "order", cause_row: 4, order_over: 1, baseline: 0, replays: 12,
      trace: { turns: [] }, morgue: "t1", rules: { rows },
      patches: [{ row: rows[4], insert_at: 1, moves_from: 4, survive: 11 / 12, forecast_delta: 0.12 },
                { row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "return" } }, insert_at: 0, survive: 1, forecast_delta: -0.3, exits: true }] } }), rows);
    await waitFor((x) => x?.screen === "death", "the order death"); await sleep(300);
    const d = await page.evaluate(() => ({ cause: document.querySelector(".death-line .cause")?.textContent, seal: document.querySelector(".death-line .verdict")?.textContent,
      sealUp: getComputedStyle(document.querySelector(".death-line .verdict")).textTransform, lead: document.querySelector("button.patch .target")?.textContent.trim(),
      lit: document.querySelector("button.patch.top .target")?.textContent.trim(), surv: document.querySelector("button.patch .surv")?.textContent }));
    check(d.cause === "ogre · D8 · return at 20% under attack nearest" && d.seal === "order" && d.sealUp === "uppercase", `an order death names both rows, the seal one word ("${d.cause}" · ${d.seal})`);
    check(d.lead === "move above attack nearest" && d.lit === d.lead && /survives 11\/12 · was 0\/12/.test(d.surv ?? ""), `its lead patch is the move, lit for the gem ("${d.lead}" · ${d.surv})`);
    await shot("cut25-order");
    await page.locator(".patch-gem").click({ timeout: 5000 }); await sleep(250);
    const after = await page.evaluate(() => ({ screen: window.__riddle.screen, rows: window.__riddle.rules.rows.map((r) => r.verb.v).join(",") }));
    check(after.screen === "camp" && after.rows === "drink,return,attack,attack,bank", `the gem moves R5 above R2 — nothing added or cut (${after.rows})`);
  }

  // ---- §6: an offline trace shows every blow (`Trace.blows`: 14 → 0 is several rows); the report's reel prints a merged line's `×n`
  {
    const b = await page.evaluate(async () => {
      const { traceTable } = await import("/src/ui/trace.ts");
      const blows = [{ t: 101, by: "goblin", dmg: 5, hp: 9 }, { t: 102, by: "goblin", dmg: 5, hp: 4 }, { t: 103, by: "goblin_archer", dmg: 4, hp: 0 }];
      const els = traceTable({ turns: [{ t: 100, row: 1, verb: { v: "attack", a: "nearest" }, hp: 14, foes: 3, telegraphs: [] }], blow: blows[2], blows }, { rows: window.__riddle.rules.rows });
      const host = document.createElement("div"); host.append(...els);
      return [...host.querySelectorAll("tr.blow")].map((r) => [...r.querySelectorAll("td")].slice(0, 3).map((d) => d.textContent.trim()).join(" "));
    });
    check(b.length === 3 && b[2] === "103 goblin archer −4 0" && b[0] === "101 goblin −5 9", `an offline trace shows every blow, hp after each (${b.join(" | ")})`);
    await page.evaluate(() => window.__riddle.go({ kind: "report", report: { elapsed_s: 0, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [],
      xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 1, returned: 0, heirs: [4, 4],
      reel: [{ pattern: "x", score: 1, t: 1, run_id: 1, text: "Lock bloats took him to 7 HP; R1 drank; reached D8.", n: 5 }] } }));
    await sleep(300);
    const rep = await page.evaluate(() => ({ reel: [...document.querySelectorAll(".report .rsec")].find((x) => x.querySelector(".label")?.textContent === "reel")?.innerText.replace(/\s+/g, " ") ?? "", runs: document.querySelector(".report .tiles .tile")?.innerText.replace(/\s+/g, " ") ?? "" }));
    check(/×5/.test(rep.reel) && /heir 4/.test(rep.runs), `the reel's merged line reads ×5 ("${rep.reel.slice(-24)}"); a watched run's tile names its heir ("${rep.runs}")`);
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
  }

  // ---- §3: drains — the word, not the numbers
  {
    const w = await page.evaluate(async () => {
      const { drainOf } = await import("/src/ui/watch.ts");
      return [drainOf({ t: 1, k: "hurt", id: 0, dmg: 0, hp: 9, cause: "hunger" }), drainOf({ t: 1, k: "max_hp", id: 0, max: 30, delta: -1, cause: "hunger" }),
        drainOf({ t: 1, k: "hurt", id: 0, dmg: 2, hp: 9, cause: "poison" }), drainOf({ t: 1, k: "hurt", id: 0, dmg: 3, hp: 9, cause: "goblin" }),
        drainOf({ t: 1, k: "hurt", id: 0, dmg: 1, hp: 9, cause: "wraith", drain: "drained" }), drainOf({ t: 1, k: "hurt", id: 0, dmg: 3, hp: 9, cause: "gas" })];
    });
    check(w.join(",") === "starving,starving,,,drained,", `a drain reads its word (hunger → starving, the core's \`drain\` word); a blow, gas or a bare poison is none (${w.map((x) => x ?? "·").join(",")})`);
  }
  // (a wall-clock reading: `measured` takes the mode's run once more on a loaded machine, the bar unchanged — tests/lib/load.mjs)
  for (const m of ["fights", "fast", "one"]) {
    const drain = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=9&fake_drain=1`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.evaluate((m) => { window.__riddle.watchMode = m; window.__riddle.go({ kind: "watch" }); }, m);
    const d = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(); let cur = null, maxMs = 0, at = "", dead = 0, drain0 = null, drained = false, acc = 0, last = t0;
      const words = [], seen = new Set();
      const obs = new MutationObserver(() => { for (const el of document.querySelectorAll(".watch .callout, .watch .ticker, .watch [class*=callout]")) { const t = el.textContent.trim(); if (t && !seen.has(t + "@" + (document.querySelector(".hud .depth")?.textContent ?? ""))) { seen.add(t + "@" + (document.querySelector(".hud .depth")?.textContent ?? "")); words.push(t); } } });
      obs.observe(document.body, { subtree: true, childList: true, characterData: true });
      const poll = () => {
        const w = document.querySelector(".watch"), now = performance.now();
        const live = window.__riddle?.screen === "watch" && w && w.dataset.ending !== "1" && !document.querySelector(".sheet-wrap") && w.dataset.card !== "1";
        if (live && w.dataset.progress !== undefined) {
          const p = w.dataset.progress;
          // a drain stretch: no move since `since`, and a drain reached in it
          // (from its first bite to the next move, a situation's held beat aside: the beat's own pace is Cut 18's)
          if (p !== cur) { cur = p; drain0 = w.dataset.drain; drained = false; acc = 0; }
          else { if (w.dataset.drain !== drain0) drained = true; if (drained && w.dataset.held !== "1") acc += now - last; if (acc > maxMs) { maxMs = acc; at = `${document.querySelector(".hud .depth")?.textContent ?? "?"} p${p}`; } }
          if (w.dataset.dead === "1") dead++;
        } else cur = null;
        last = now;
        if (now - t0 > 60_000 || (window.__riddle?.screen !== "watch" && now - t0 > 3000)) { obs.disconnect(); res({ maxMs: Math.round(maxMs), at, dead, words }); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    return { ok: d.dead > 0 && d.maxMs > 0 && d.maxMs <= 5000, line: `${m}: a drain stretch plays ≤ 5 s (the longest wall time from a bite to the next move, held beats aside, ${(d.maxMs / 1000).toFixed(1)} s at ${d.at}; dead frames ${d.dead})`, d };
    });
    const d = drain.d;
    const starving = d.words.filter((x) => /starving/i.test(x)).length, bites = d.words.filter((x) => /hunger −1 max/i.test(x)).length;
    check(drain.ok, drain.line);
    check(starving >= 1 && bites === 0, `${m}: the drain shows its word (\`starving\` ×${starving} across floors), never a bite's \`hunger −1 max\` (${bites})`);
    await page.evaluate(() => document.querySelectorAll(".sheet-wrap").forEach((x) => x.remove()));
  }
  // the plain 1×: a tile beside fights/fast; the chip's clock reads 1×; the mode is remembered and the send gem says it
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=9`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.evaluate(() => { window.__riddle.watchMode = "fights"; window.__riddle.go({ kind: "watch" }); });
    await sleep(600);
    const tiles = await page.evaluate(() => [...document.querySelectorAll(".watch .cmd .tile:not(.empty)")].map((t) => t.dataset.tile));
    await page.locator(".cmd .tile[data-tile=one]").click({ timeout: 5000 });
    const rates = new Set(); const t0 = Date.now();
    while (Date.now() - t0 < 4000) { const r = await page.evaluate(() => { const w = document.querySelector(".watch"); return w?.dataset.dead === "1" ? "dead" : document.querySelector(".cmd .tile[data-tile=one]")?.dataset.rate ?? ""; }); if (r) rates.add(r); await sleep(100); }
    const mode = await page.evaluate(() => ({ el: document.querySelector(".watch")?.dataset.mode, app: window.__riddle.watchMode }));
    check(tiles.slice(0, 3).join(",") === "fights,fast,one" && mode.el === "one" && mode.app === "one", `the mode picker offers \`1×\` beside fights/fast (${tiles.join(",")}; now ${mode.el})`);
    check([...rates].every((r) => /^1×?$/.test(r) || r === "dead") && [...rates].some((r) => /^1×?$/.test(r)), `the plain 1× runs its live frames at 1× (${[...rates].join(" ")})`);
    await page.evaluate(() => { document.querySelectorAll(".sheet-wrap").forEach((x) => x.remove()); window.__riddle.go({ kind: "camp" }); }); await camp();
    const gemTxt = await page.evaluate(() => document.querySelector(".gem[data-mode]")?.textContent ?? document.querySelector("[data-mode]")?.textContent);
    check(/send\s*▸?\s*normal/i.test(gemTxt ?? ""), `the send gem says the remembered 1× (normal) ("${gemTxt}")`);
  }

  // ---- §4: the forge's measure never waits behind the unlock shelf's
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=25&fake_lag=1`, { waitUntil: "domcontentloaded" });
    await camp(); await sleep(8000);   // the camp's own measures settle
    const t = await page.evaluate(async () => {
      const E = window.__riddle.engine, t0 = performance.now(), at = {};
      await Promise.all([E.unlockDeltas().then(() => { at.unlock = performance.now() - t0; }), E.kitDeltas().then(() => { at.kit = performance.now() - t0; })]);
      return { unlock: Math.round(at.unlock), kit: Math.round(at.kit), cores: navigator.hardwareConcurrency };
    });
    check(t.cores < 6 || (t.kit < 4500 && t.unlock < 4500), `the forge's measure rides its own lane: unlock ${t.unlock} ms, kit ${t.kit} ms with both asked at once (3 s each; one lane would take 6; ${t.cores} cores)`);
  }

  // ---- §6: sheets — one at a time, with a back
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=25`, { waitUntil: "domcontentloaded" });
    await camp();
    const s = await page.evaluate(async () => {
      const { openSheet } = await import("/src/ui/sheet.ts"); const { h } = await import("/src/ui/dom.ts");
      const vis = () => [...document.querySelectorAll(".sheet-wrap")].filter((w) => !w.hidden && getComputedStyle(w).display !== "none").map((w) => w.querySelector(".label")?.textContent);
      openSheet(() => h("div", { class: "sheet-body" }, h("div", { class: "label" }, "first")));
      openSheet(() => h("div", { class: "sheet-body" }, h("div", { class: "label" }, "second")));
      const two = vis(), back = !!document.querySelector(".sheet-wrap:not([hidden]) .sheet-back");
      document.querySelector(".sheet-wrap:not([hidden]) .sheet-back").click();
      const afterBack = vis();
      openSheet(() => h("div", { class: "sheet-body" }, h("div", { class: "label" }, "third")));
      document.querySelector(".sheet-wrap:not([hidden]) .close-stud, .sheet-wrap:not([hidden]) .sheet-x").click();
      const afterX = document.querySelectorAll(".sheet-wrap").length;
      return { two, back, afterBack, afterX };
    });
    check(s.two.join() === "second" && s.back && s.afterBack.join() === "first" && s.afterX === 0, `a sheet opened from a sheet replaces it (${s.two.join()} shown, back ${s.back}); back returns (${s.afterBack.join()}); × closes both (${s.afterX} left)`);
  }
  // the row card at the cap: `max` and its gate, not the next price
  {
    const r = await page.evaluate(async () => {
      const { openUnlockSheet } = await import("/src/ui/unlocks.ts"); const app = window.__riddle;
      const was = app.unlockCat;
      app.unlockCat = [{ id: "row8", cost: 11, owned: false, available: true }, { id: "row9", cost: 8, owned: false, available: false, needs: "3 boss kinds", gold: 6050 }];
      openUnlockSheet(app, { id: "row8", cost: 11, owned: false, available: true, label: "+1 row", gated: false, gold: 4400, next: { id: "row9", cost: 8, gold: 6050, gold_after_gold: 7000 } });
      await new Promise((res) => setTimeout(res, 100));
      const t = document.querySelector(".sheet-wrap .unlock-sheet")?.innerText.replace(/\s+/g, " ") ?? "";
      document.querySelectorAll(".sheet-wrap").forEach((w) => w.remove());
      openUnlockSheet(app, { id: "row10", cost: 12, owned: false, available: true, label: "+1 row", gated: false });
      await new Promise((res) => setTimeout(res, 100));
      const t2 = document.querySelector(".sheet-wrap .unlock-sheet .next-price")?.textContent ?? "";
      document.querySelectorAll(".sheet-wrap").forEach((w) => w.remove());
      app.unlockCat = was;
      return { t, t2 };
    });
    check(/max · ⊘ 3 boss kinds/.test(r.t) && !/next ◆/.test(r.t) && r.t2 === "max", `the row card at the cap reads \`max\` and its gate, never the next price ("${r.t.slice(-40)}" · last step "${r.t2}")`);
  }
  // the unlock grid keeps its cells under a buy
  {
    await richSave(0, 60); await camp();
    await openPanel(page, "unlocks", { all: true }); await sleep(400);
    const before = await page.evaluate(() => [...document.querySelectorAll(".panel .unlocks .cards > .card")].map((c) => { const r = c.getBoundingClientRect(); return { id: c.dataset.id, x: Math.round(r.x), y: Math.round(r.y), cls: c.className }; }));
    const target = before.find((c) => /buyable/.test(c.cls) && !/^row/.test(c.id ?? "")) ?? before.find((c) => /buyable/.test(c.cls));
    if (target) {
      await page.locator(`.panel .unlocks .cards > .card[data-id="${target.id}"]`).click({ timeout: 5000 }); await sleep(200);
      await page.locator(".sheet-wrap .unlock-sheet button.buy.marks").click({ timeout: 5000 }); await sleep(1200);
      const after = await page.evaluate(() => [...document.querySelectorAll(".panel .unlocks .cards > .card")].map((c) => { const r = c.getBoundingClientRect(); return { id: c.dataset.id, x: Math.round(r.x), y: Math.round(r.y), cls: c.className }; }));
      const cell = after.find((c) => Math.abs(c.x - target.x) <= 1 && Math.abs(c.y - target.y) <= 1);
      const moved = before.filter((b) => b.id !== target.id).filter((b) => { const a = after.find((x) => x.id === b.id); return a && (Math.abs(a.x - b.x) > 1 || Math.abs(a.y - b.y) > 1); });
      check(!!cell && (cell.id === target.id ? /done/.test(cell.cls) : true) && moved.length === 0 && before.length > 3, `a buy leaves the grid's cells where they were: ${target.id}'s cell holds ${cell?.id} (${cell?.cls.replace("card ", "")}); ${moved.length} other cards moved (${moved.map((m) => m.id).join(",")})`);
      await shot("cut25-unlocks");
    } else check(false, `a buyable card on the grid (${before.map((b) => `${b.id}:${b.cls}`).join(" ")})`);
    await page.keyboard.press("Escape"); await page.keyboard.press("Escape"); await sleep(150);
  }
  // the forge's armed step keeps its place; the second tap there buys
  {
    await richSave(5000); await camp();
    await page.locator(".cmd .tile[data-tile=forge]").click({ timeout: 5000 }); await sleep(400);
    const btn = page.locator(".sheet-wrap .forge button.kit-next.buyable").first();
    const r0 = await btn.boundingBox();
    const lab0 = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .forge button.kit-next.buyable"); const l = b?.querySelector(".kit-label"); return { slot: b?.dataset.slot, x: Math.round(l?.getBoundingClientRect().x ?? -1), pips: b?.closest(".kit-slot")?.querySelectorAll(".pip.on").length }; });
    await page.mouse.click(r0.x + r0.width * 0.8, r0.y + r0.height / 2); await sleep(150);
    const armed = await page.evaluate((slot) => { const b = document.querySelector(`.sheet-wrap .forge button.kit-next[data-slot=${slot}]`); const r = b.getBoundingClientRect(); const l = b.querySelector(".kit-label"); return { armed: b.classList.contains("armed"), ok: b.querySelector(".kit-ok")?.textContent ?? "", x: Math.round(r.x), y: Math.round(r.y), w: Math.round(r.width), lx: Math.round(l?.getBoundingClientRect().x ?? -1) }; }, lab0.slot);
    check(armed.armed && /ok \$\d+/.test(armed.ok) && armed.x === Math.round(r0.x) && armed.y === Math.round(r0.y) && armed.w === Math.round(r0.width) && armed.lx === lab0.x, `the armed forge step keeps its place (\`${armed.ok.trim()}\` where the price stood; the line did not move)`);
    await shot("cut25-forge-armed");
    await page.mouse.click(r0.x + r0.width * 0.8, r0.y + r0.height / 2); await sleep(700);
    const pips = await page.evaluate((slot) => document.querySelector(`.sheet-wrap .forge .kit-slot[data-slot=${slot}]`)?.querySelectorAll(".pip.on").length, lab0.slot);
    check(pips === lab0.pips + 1, `the second tap where the first was buys the step (${lab0.pips} → ${pips} steps)`);
    await page.keyboard.press("Escape"); await sleep(150);
  }

  // ---- §3: the black frame (real wasm): seed 2501's first run, fights — no dark stretch over 6 frames
  if (!process.argv.includes("--no-wasm")) {
    const black = await measured(async () => {
    await page.goto(`${url}?dev=1&fresh=1&seed=2501`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "the real engine's camp", 120_000);
    const kind = await page.evaluate(() => window.__riddle.kind);
    await page.evaluate(() => { window.__riddle.watchMode = "fights"; window.__riddle.go({ kind: "watch" }); });
    const r = await page.evaluate(() => new Promise((res) => {
      let from = -1, frames = 0, worst = { frames: 0, ms: 0, text: "" }; const t0 = performance.now();
      const f = () => {
        const v = window.__viewer, st = v?.stats?.();
        if (window.__riddle.screen !== "watch" || performance.now() - t0 > 150_000) { res(worst); return; }
        if (st) {
          const dark = st.fade >= 0.95 || st.drawn === 0, now = performance.now();
          if (dark) { frames++; if (from < 0) from = now; }
          else if (from >= 0) { if (frames > worst.frames) worst = { frames, ms: Math.round(now - from), text: document.querySelector(".watch .beat-line, .watch .ticker")?.textContent?.slice(0, 40) ?? "" }; from = -1; frames = 0; }
        }
        requestAnimationFrame(f);
      };
      requestAnimationFrame(f);
    }));
    // (the stairs' own fade-through and a frame cut are dark by design: ≤ 400 ms; the purse's held load was 3 s)
    return { ok: kind === "wasm" && r.ms <= 400, line: `real wasm (${kind}): no empty board — the longest dark stretch ${r.frames} frames / ${r.ms} ms ("${r.text}"; AM's purse beat held a new floor's load 3 s)` };
    });
    check(black.ok, black.line);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut25: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut25: ok (${out.length} checks)`);
