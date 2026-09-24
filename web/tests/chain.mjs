#!/usr/bin/env node
// Cut 11 §2 / §5 gates on the client: the death screen's chain, its replay links, root / below-bar / unlock patches, and the
// gold sheet filtered to a run. Runs on the GPU harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219)
// with the fake engine (`?engine=fake&dev=1`).
//
//   node web/tests/chain.mjs        (part of `pnpm test` in web/)
//
// Walk: boot fresh with autosend → drive the watched run to its end (▶▶| pressed through it) so the client holds the run's
// event log → a death is fabricated on that run id (the screen and its taps are the code under test; the fake's own death
// is checked when the run happened to end in one): its trace's last turn carries `because`s at a tick inside the run and at
// one past its end → the chain shows one line per row with the reason and `← because`, a `watch` chip only on the in-range
// tick, `D3 · tN` on the other, and the fired row last → tapping `watch` opens the replay sheet, whose viewer ticks from
// `t − 20` and pauses by `t + 20` → the root patch shows `← root` in the accent colour, the below-bar patch is dimmed and
// reads `below bar`, the unlock pseudo-patch (`◆2 cond: alert · buy`) calls `app.buy("cond_alert")` and then inserts the row
// at the top of the camp → the death's ledger line and a report exit line open the gold sheet filtered to that run's
// ledger lines (`data-filter` set, every shown line inside the run's slice, `all` lifts the filter). Exit 1 on any failed
// assertion, console error or page error.
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const shots = process.env.CHAIN_SHOTS ? resolve(process.env.CHAIN_SHOTS) : null;

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, vault: !!document.querySelector(".sheet-wrap .vault-choice .chip"), keep: !!document.querySelector(".sheet-wrap button.btn.primary.wide") } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const text = (sel) => page.evaluate((sel) => [...document.querySelectorAll(sel)].map((el) => el.innerText.replace(/\s+/g, " ").trim()), sel);
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, name) }); };

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7&autosend=1&fake_depth=4`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
  // drive the run to its end: ▶▶| every 400 ms (under the card it waives the minimum, in a fight it jumps to its end)
  const t0 = Date.now();
  while (Date.now() - t0 < 150_000) {
    const s = await state();
    if (!s || s.screen !== "watch") break;
    await page.locator(".cmd .hud-btn", { hasText: "▶▶|" }).click({ timeout: 1000 }).catch(() => {});
    await sleep(400);
  }
  let s = await waitFor((x) => x && x.screen !== "watch", "the run's end", 30_000);
  // the keep sheet (a return with items) settles the run
  if (s.screen === "exit") {
    if (s.vault) await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {});
    await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 }).catch(() => {});
    s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after keep", 30_000);
  }
  await waitFor((x) => x && !x.busy, "the engine idle", 30_000);
  check(["death", "report"].includes(s.screen), `the watched run ended (${s.screen})`);
  const log = await page.evaluate(() => { const l = window.__riddle.runLog(); return l ? { runId: l.runId, floors: l.floors.map((f) => ({ depth: f.snap.depth, from: f.snap.turn, to: f.evs.length ? f.evs[f.evs.length - 1].t : f.snap.turn, n: f.evs.length })), endTick: l.endTick } : null; });
  check(!!log && log.floors.length >= 1 && log.floors.some((f) => f.n > 0), `the run log holds the run: ${JSON.stringify(log)}`);
  check(log?.endTick !== undefined, `the run log knows the exit tick (${log?.endTick})`);

  // the fake's own death, when the run ended in one: every state reason with a provenance carries a because
  if (s.screen === "death") {
    const rows = await text(".chain .chain-row");
    const line = await text(".death-line");
    out.push(`info the fake's death: ${line[0]} · chain rows: ${rows.length ? rows.join(" | ") : "(none: no state reason had a provenance)"}`);
    await shot("chain-fake-death.png");
  }

  // a fabricated death on the watched run: a because inside the run's ticks and one past its end
  const fl = log.floors.find((f) => f.n > 20) ?? log.floors[0];
  const inT = Math.min(fl.to, fl.from + 30), outT = (log.endTick ?? fl.to) + 5000;
  const death = await page.evaluate(({ runId, inT, outT, depth }) => {
    const r = window.__riddle;
    const rows = r.rules.rows;
    const d = {
      run_id: runId, depth, cause: "goblin_archer", margin: "1 over", verdict: "gap", baseline: 0.25, morgue: "fabricated",
      line: { carried: 30, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "$30 carried · death keeps 0% → $0" },
      trace: { turns: [
        { t: inT - 20, row: 1, verb: { v: "attack", a: "nearest" }, hp: 9, foes: 1, telegraphs: [] },
        { t: inT + 30, row: Math.min(2, rows.length - 1), verb: { v: "attack", a: "nearest" }, hp: 3, foes: 2, telegraphs: ["archer draws"], rows: [
          { row: 0, why: "no item", because: { text: "den took the heal, D3", t: inT, depth } },
          { row: 1, why: "no path", because: { text: "gas cloud, this room", t: outT, depth: depth + 1 } },
        ] },
      ] },
      chain: [{ text: "den took the heal, D3", t: inT, depth }, { text: "never found a scroll", t: inT + 10, depth }],
      patches: [
        { row: { conds: [{ k: "foe_tag", t: "thief" }], verb: { v: "attack", a: "tag:thief" } }, insert_at: 0, survive: 0.8, forecast_delta: 0.55, root: { text: "den took the heal" } },
        { row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "drink", a: "unknown" } }, insert_at: 0, survive: 0.4, forecast_delta: 0.15, below_bar: true },
        { row: { conds: [{ k: "alert>=", n: 3 }], verb: { v: "return" } }, insert_at: -1, survive: 0.7, forecast_delta: 0.45, root: { text: "◆2 cond: alert" } },
      ],
    };
    r.__death = d;
    r.go({ kind: "death", death: d });
    return { rowsN: rows.length };
  }, { runId: log.runId, inT, outT, depth: fl.depth });
  await sleep(300);
  await shot("chain-death.png");
  const chain = await page.evaluate(() => [...document.querySelectorAll(".chain .chain-row")].map((el) => ({
    text: el.innerText.replace(/\s+/g, " ").trim(), watch: !!el.querySelector("button.link"), at: el.querySelector(".at")?.textContent ?? "", cls: el.className })));
  check(chain.length === 4, `chain: ${chain.length} lines (2 rows + fired + 1 extra chain link): ${chain.map((c) => c.text).join(" | ")}`);
  check(/^R1 .*no item ← den took the heal, D3/.test(chain[0]?.text ?? ""), `R1 reads reason ← because: "${chain[0]?.text}"`);
  check(chain[0]?.watch === true, "R1's because (inside the run) carries a watch chip");
  check(/^R2 .*no path ← gas cloud, this room/.test(chain[1]?.text ?? "") && chain[1].watch === false, `R2's because (past the run) has no watch chip: "${chain[1]?.text}"`);
  check(new RegExp(`D${fl.depth + 1} · t${outT}$`).test(chain[1]?.at ?? ""), `R2's line ends with the depth and tick instead: "${chain[1]?.at}"`);
  check(/fired$/.test(chain[2]?.text ?? "") && chain[2].cls.includes("fired"), `the fired row closes the rows: "${chain[2]?.text}"`);
  check(/never found a scroll/.test(chain[3]?.text ?? "") && chain[3].cls.includes("extra"), `Death.chain entries beyond the rows follow: "${chain[3]?.text}"`);
  check(chain[3]?.watch === false && !chain[3]?.at, `a \`never\` entry has no watch chip and no tick: "${chain[3]?.text}"`);
  check(death.rowsN >= 2, `the active set has rows for verb labels (${death.rowsN})`);

  // patches: root marker in the accent colour, below-bar dimming, the unlock pseudo-patch
  const patches = await page.evaluate(() => [...document.querySelectorAll(".patches .patch")].map((el) => ({
    text: el.innerText.replace(/\s+/g, " ").trim(), cls: el.className, opacity: Number(getComputedStyle(el).opacity),
    root: el.querySelector(".root")?.textContent ?? "", rootColor: el.querySelector(".root") ? getComputedStyle(el.querySelector(".root")).color : "",
  })));
  const acc = await page.evaluate(() => getComputedStyle(document.documentElement).getPropertyValue("--acc").trim());
  const hex2rgb = (hx) => `rgb(${parseInt(hx.slice(1, 3), 16)}, ${parseInt(hx.slice(3, 5), 16)}, ${parseInt(hx.slice(5, 7), 16)})`;
  check(patches.length === 3, `3 patches shown (${patches.length})`);
  check(patches[0]?.root === "← den took the heal" && patches[0].rootColor === hex2rgb(acc), `root patch marks its root in the accent: "${patches[0]?.root}" ${patches[0]?.rootColor}`);
  check(patches[1]?.cls.includes("below") && patches[1].opacity < 0.7 && /survives 40% · below bar/.test(patches[1].text), `below-bar patch dimmed (${patches[1]?.opacity}): "${patches[1]?.text}"`);
  check(patches[2]?.cls.includes("unlock") && /◆2 cond: alert · buy/.test(patches[2].text), `unlock pseudo-patch reads ◆2 cond: alert · buy: "${patches[2]?.text}"`);
  // the tap buys the cond's unlock, then inserts the row at the top (app.buy stubbed: the fake's marks are not the point)
  const before = await page.evaluate(() => { const r = window.__riddle; r.__buys = []; r.__buy0 = r.buy; r.buy = async (id) => { r.__buys.push(id); return true; }; return r.rules.rows.length; });
  await page.locator(".patches .patch.unlock").click({ timeout: 5000 });
  await waitFor((x) => x?.screen === "camp", "the camp after the unlock tap");
  const after = await page.evaluate(() => { const r = window.__riddle; const rows = r.rules.rows; const top = rows[0]; r.buy = r.__buy0; return { buys: r.__buys, n: rows.length, top: `${top.conds.map((c) => `${c.k}${c.n ?? ""}`).join(" ")} → ${top.verb.v}`, origin: top.origin }; });
  check(after.buys.length === 1 && after.buys[0] === "cond_alert", `the tap bought cond_alert (${JSON.stringify(after.buys)})`);
  check(after.n === before + 1 && after.top === "alert>=3 → return" && after.origin === "patch", `then inserted the row at the top: ${after.top} (${before} → ${after.n} rows, origin ${after.origin})`);
  // a refused buy inserts nothing
  await page.evaluate(() => { const r = window.__riddle; r.go({ kind: "death", death: r.__death }); r.__buy0 = r.buy; r.buy = async () => false; });
  await sleep(200);
  await page.locator(".patches .patch.unlock").click({ timeout: 5000 });
  await sleep(300);
  const refused = await page.evaluate(() => { const r = window.__riddle; r.buy = r.__buy0; return { screen: r.screen, n: r.rules.rows.length }; });
  check(refused.screen === "death" && refused.n === after.n, `a refused buy inserts nothing and stays (${refused.screen}, ${refused.n} rows)`);

  // the replay: tap watch → a sheet with a viewer that plays t − 20 … t + 20 and pauses
  await page.locator(".chain button.link").first().click({ timeout: 5000 });
  await sleep(600);
  const rep0 = await page.evaluate(() => { const v = window.__replay; return { sheet: !!document.querySelector(".sheet-wrap .replay canvas"), head: document.querySelector(".sheet-wrap .replay .row-label")?.textContent.replace(/\s+/g, " ").trim(), tick: v?.tick?.() ?? null, frame: v?.frame?.() ?? null }; });
  check(rep0.sheet, "watch opens the replay sheet with a canvas");
  check(/← den took the heal, D3/.test(rep0.head ?? ""), `the sheet names the link: "${rep0.head}"`);
  check(rep0.tick !== null && rep0.tick >= inT - 20 && rep0.tick <= inT + 25, `the viewer's clock started at t − 20 (${rep0.tick}, t = ${inT})`);
  check(rep0.frame === "fight", `the replay is in the fight frame (${rep0.frame})`);
  await sleep(5000);
  const rep1 = await page.evaluate(() => { const v = window.__replay; return v?.tick?.() ?? null; });
  await sleep(600);
  const rep2 = await page.evaluate(() => { const v = window.__replay; return v?.tick?.() ?? null; });
  check(rep1 !== null && rep1 >= inT + 20 && rep1 <= inT + 26 && rep2 === rep1, `the replay played 40 ticks then paused (${rep1} → ${rep2})`);
  await shot("chain-replay.png");
  await page.keyboard.press("Escape"); await sleep(300);
  const gone = await page.evaluate(() => ({ sheet: !!document.querySelector(".sheet-wrap .replay"), viewer: "__replay" in window }));
  check(!gone.sheet && !gone.viewer, "escape closes the replay and disposes its viewer");

  // QA on e0f87e7 (D): a clip whose window reaches the run's end "played ~3 s then the canvas went fully black and stayed
  // black" — the floor log's `exit` / `descend` faded the renderer to dark. The clip holds its last frame: the canvas is lit
  // 1 s and 5 s after the link opens (read inside a rAF, after the renderer's own, so the WebGL buffer is this frame's).
  const endT = log.endTick ?? fl.to;
  const lastFl = log.floors[log.floors.length - 1];
  const lit = () => page.evaluate(() => new Promise((res) => requestAnimationFrame(() => {
    const c = document.querySelector(".sheet-wrap .replay canvas"); if (!c) return res(-1);
    const o = document.createElement("canvas"); o.width = c.width; o.height = c.height;
    const g = o.getContext("2d"); g.drawImage(c, 0, 0);
    const d = g.getImageData(0, 0, o.width, o.height).data; let n = 0;
    for (let i = 0; i < d.length; i += 4) if (d[i] + d[i + 1] + d[i + 2] > 60) n++;
    res(n);
  })));
  await page.evaluate(({ t, depth }) => { const r = window.__riddle; const d = r.__death; d.chain = [{ text: "the walk-out", t, depth }]; d.trace.turns[1].rows[0].because = { text: "the walk-out", t, depth }; r.go({ kind: "death", death: d }); }, { t: Math.max(lastFl.from, endT - 5), depth: lastFl.depth });
  await sleep(300);
  await page.locator(".chain button.link").first().click({ timeout: 5000 });
  await sleep(1000);
  const lit1 = await lit();
  await sleep(4000);
  const lit5 = await lit();
  const endTick = await page.evaluate(() => window.__replay?.tick?.() ?? null);
  check(lit1 > 0 && lit5 > 0, `a clip that reaches the run's end holds its last frame: ${lit1} lit px at 1 s, ${lit5} at 5 s (paused at t${endTick}, exit t${endT})`);
  await shot("chain-replay-end.png");
  await page.keyboard.press("Escape"); await sleep(300);

  // §5: the death's ledger line opens the gold sheet filtered to the run
  const ledger = await page.evaluate(() => (window.__riddle.lineage.gold_ledger ?? []).map((g) => ({ t: g.t, delta: g.delta, why: g.why })));
  const exits = ledger.map((g, i) => ({ ...g, i })).filter((g) => /^(returned|banked|died|lost)\b/.test(g.why));
  check(exits.length >= 1, `the ledger holds ${exits.length} exit line(s) of ${ledger.length}`);
  // the fabricated death's line must match a ledger line to filter: use the real last exit's tier and kept sum
  const lastExit = exits[exits.length - 1];
  await page.evaluate(({ kept, tier }) => { const r = window.__riddle; r.__death.line = { carried: 30, keep_pct: tier === "banked" ? 100 : tier === "died" ? 0 : 60, kept, spent: 0, spent_on: [], text: `$30 carried · ${tier} → $${kept}` }; r.go({ kind: "death", death: r.__death }); }, { kept: lastExit.delta, tier: lastExit.why.split(" ")[0] === "lost" ? "returned" : lastExit.why.split(" ")[0] });
  await sleep(200);
  await page.locator(".death .ledger-line button").click({ timeout: 5000 });
  await sleep(300);
  const gs = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .gold-sheet"); return b ? { filter: b.dataset.filter, lines: [...b.querySelectorAll(".lrow[data-t]")].map((l) => ({ t: Number(l.dataset.t), text: l.textContent.replace(/\s+/g, " ").trim() })), all: !!b.querySelector("button.chip.mini") } : null; });
  check(!!gs && gs.filter !== "", `the gold sheet opened filtered (${gs?.filter})`);
  const prevExit = exits[exits.length - 2];
  const from = prevExit ? prevExit.i + 1 : 0;
  // QA 23ed91f: the newest exit's run runs on to now (the camp's charges since: a restock), so the sheet explains the bar
  const expect = ledger.slice(from);
  check(!!gs && gs.lines.length === expect.length && gs.lines.length >= 1, `the filtered sheet shows the run's ${expect.length} line(s) (${gs?.lines.length}): ${gs?.lines.map((l) => l.text).join(" | ")}`);
  check(!!gs && gs.lines.every((l) => l.text.includes(lastExit.why) || l.t >= (prevExit?.t ?? 0)), "every shown line is inside the run's slice");
  await shot("chain-gold.png");
  await page.locator(".sheet-wrap .gold-sheet button.chip.mini", { hasText: /^all$/ }).click({ timeout: 5000 });   // QA on 50bb162: the run's own chip sits beside `all`
  await sleep(200);
  const gsAll = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .gold-sheet"); return b ? { filter: b.dataset.filter, n: b.querySelectorAll(".lrow[data-t]").length } : null; });
  check(!!gsAll && gsAll.filter === "" && gsAll.n === ledger.length, `all lifts the filter to the whole ledger (${gsAll?.n} of ${ledger.length})`);
  await page.keyboard.press("Escape"); await sleep(200);

  // §5: a report exit line is tappable to the same sheet
  await page.evaluate(({ kept, tier }) => {
    const r = window.__riddle; const L = r.lineage;
    const line = { carried: 30, keep_pct: tier === "banked" ? 100 : tier === "died" ? 0 : 60, kept, spent: 0, spent_on: [], text: `$30 carried · ${tier} → $${kept}` };
    r.go({ kind: "report", report: { elapsed_s: 60, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 1, exits: [line] } });
  }, { kept: lastExit.delta, tier: lastExit.why.split(" ")[0] === "lost" ? "returned" : lastExit.why.split(" ")[0] });
  await sleep(200);
  await page.locator(".report .exit-lines .ledger-line button.ledger-btn").first().click({ timeout: 5000 });
  await sleep(300);
  const gsR = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .gold-sheet"); return b ? { filter: b.dataset.filter, n: b.querySelectorAll(".lrow[data-t]").length } : null; });
  check(!!gsR && gsR.filter === gs?.filter, `the report's exit line opens the same filtered sheet (${gsR?.filter})`);
  await page.keyboard.press("Escape"); await sleep(200);

  // QA on e0f87e7 (D): "because-links in a report's TRACE have no WATCH button; the return sheet's had them" — an exit line
  // that carries its `run_id` (the core sends it) gives its trace chip the run, so the chain's in-range links get `watch`;
  // the stall's trace, matched to the exit whose trace it is, too
  const watchN = await page.evaluate(({ runId, inT, depth }) => {
    const r = window.__riddle; const L = r.lineage;
    const trace = { turns: [{ t: inT + 30, row: 0, verb: { v: "attack", a: "nearest" }, hp: 3, foes: 2, telegraphs: [], rows: [{ row: 0, why: "no item", because: { text: "den took the heal, D3", t: inT, depth } }] }] };
    const line = { carried: 30, keep_pct: 60, kept: 18, spent: 0, spent_on: [], text: "returned $18 · $30 carried · keeps 60%", trace, run_id: runId };
    const stall = { row: 0, fired: 3, text: "R1 return ended 3 runs at D4", patches: [], trace: JSON.parse(JSON.stringify(trace)) };
    r.go({ kind: "report", report: { elapsed_s: 60, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 1, exits: [line], stall } });
    return { chips: document.querySelectorAll(".report .exit-lines .chip.mini").length + document.querySelectorAll(".report .stall .chip.mini").length };
  }, { runId: log.runId, inT, depth: fl.depth });
  check(watchN.chips === 2, `the report's exit line and the stall carry trace chips (${watchN.chips})`);
  await page.locator(".report .exit-lines .chip.mini").first().click({ timeout: 5000 }); await sleep(300);
  const exitLinks = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chain .chain-row")].map((el) => ({ text: el.innerText.replace(/\s+/g, " ").trim(), watch: !!el.querySelector("button.link") })));
  check(exitLinks.length >= 1 && exitLinks[0].watch, `a report exit line's trace has watch on its in-range link: ${JSON.stringify(exitLinks[0])}`);
  await page.keyboard.press("Escape"); await sleep(200);
  await page.locator(".report .stall .chip.mini").first().click({ timeout: 5000 }); await sleep(300);
  const stallLinks = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chain .chain-row")].map((el) => ({ text: el.innerText.replace(/\s+/g, " ").trim(), watch: !!el.querySelector("button.link") })));
  check(stallLinks.length >= 1 && stallLinks[0].watch, `the stall's trace (the same run's) has watch too: ${JSON.stringify(stallLinks[0])}`);
  await page.keyboard.press("Escape");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`chain: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`chain: ok (${out.length} checks)`);
