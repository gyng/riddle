#!/usr/bin/env node
// Cut 10 §2–§3 client rows, each one assertion on the fake (`?engine=fake&dev=1`), on the GPU harness (tools/browser.mjs)
// against the dev server (tools/dev.sh, :5219):
//   §2  the forecast's boss floor reads `D5 0% · goblin warlord · try: attack boss` when the counter fact is known and the row
//       is absent; tapping it inserts the row at the top of the active set (and the bestiary chip inserts at the top too)
//   §3  `rest 20m · send skips` permanently · a greyed supply says why under its price · `▲▼` chips (44 px) move a row ·
//       the `+1 row` card is dimmed `⊘ fill rows` while free rows exist · `3 over` reads `3 hp short` · the report's tiles read
//       `returned · banked · deaths` when returns outnumber banks, its exit lines lead with `returned $61`, a lost companion reads
//       `jackal Ashar fell` · a card's delta reads `reach +N% at R2` (Cut 12: where it goes) · the tiles fade in after an absence
//
//   node web/tests/clarity.mjs        (part of `pnpm test` in web/)
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
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await settle();

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
    const info = await page.evaluate(() => { const r = window.__riddle; const card = [...document.querySelectorAll(".unlocks .card")].find((c) => /^\+1 row/.test(c.textContent)); return { rows: r.rules.rows.length, max: r.vocab.max_rows, has: !!card, gated: card?.classList.contains("gated"), needs: card?.querySelector(".needs")?.textContent.trim() }; });
    check(info.rows < info.max && info.has && info.gated && /fill rows$/.test(info.needs ?? ""), `the +1 row card is dimmed (${info.rows}/${info.max} rows): "${info.needs}"`);
    // fill the set: the card's gate lifts (the fake's own gate then decides)
    await page.evaluate(() => { const r = window.__riddle; while (r.rules.rows.length < r.vocab.max_rows) r.rules.rows.push({ conds: [{ k: "hp<", n: 30 }], verb: { v: "retreat" }, origin: "player" }); r.rulesChanged(); r.go({ kind: "camp" }); });
    await settle();
    const full = await page.evaluate(() => { const card = [...document.querySelectorAll(".unlocks .card")].find((c) => /^\+1 row/.test(c.textContent)); return card?.querySelector(".needs")?.textContent.trim() ?? ""; });
    check(!/fill rows|rows full/.test(full), `with the set full the card no longer says fill rows ("${full}")`);
  }
  // §3 a card's reach delta says where the card goes — Cut 12 §1: `at R2` (before the engagement row, the catalogue's `insert_at`), else `at end`
  {
    const delta = await page.evaluate(() => [...document.querySelectorAll(".unlocks .card .delta")].map((d) => d.textContent.trim()));
    check(delta.length > 0 && delta.every((d) => /^reach [+−]\d+% at (R\d+|end)$/.test(d)), `card deltas say where the card goes: ${delta.slice(0, 2).join(" · ")}`);
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
    const rest = await page.evaluate(() => { const el = document.querySelector(".send-bar .rest"); return { text: el?.textContent.trim(), hidden: el?.hidden, tag: el?.tagName }; });
    check(rest.text === "rest 20m · send skips" && !rest.hidden, `the rest chip reads "${rest.text}"`);
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
    check(!!bar && bar.tag === "BUTTON" && /^D5 \d+%( ±\d+)? · goblin warlord · try: attack boss$/.test(bar.text), `the boss floor names the counter: "${bar?.text}"`);
    const before = await rowTexts();
    await page.locator(".fc-bars .bar.try").click({ timeout: 5000 });
    await waitFor((s) => s?.screen === "camp", "camp"); await settle();
    const after = await rowTexts(), eng = await engineRows();
    check(after.length === before.length + 1 && /^foe: boss \+? ?→ attack boss$/.test(after[0]) && after.slice(1).join("|") === before.join("|"), `the tap inserts the counter row at the top: "${after[0]}"`);
    check(/foe_tagboss→attacktag:boss/.test(eng[0]), `the engine's set leads with it (${eng[0]})`);
    check(!(await page.evaluate(() => !!document.querySelector(".fc-bars .bar.try"))), "with the row held the bar no longer offers it");
    // the bestiary counter chip inserts at the top too (Cut 7 §1 chip, Cut 10 §2 position)
    await page.evaluate(() => { const r = window.__riddle; r.rules.rows.shift(); r.rulesChanged(); r.go({ kind: "camp" }); });
    await settle();
    await page.locator(".party .mini").filter({ hasText: /^ledger$/ }).click({ timeout: 5000 }); await sleep(300);
    const chip = page.locator(".sheet-wrap .chip.counter");
    check((await chip.count()) === 1, "the bestiary shows the counter chip");
    await chip.click({ timeout: 5000 });
    await waitFor((s) => s?.screen === "camp", "camp"); await settle();
    const viaChip = await rowTexts();
    check(/^foe: boss \+? ?→ attack boss$/.test(viaChip[0]) && viaChip.length === after.length, `the bestiary chip inserts at the top too: "${viaChip[0]}"`);
  }
  // §3 `3 over` → `3 hp short` on the death line; a lost companion reads `jackal Ashar fell`
  {
    await page.evaluate(() => { const r = window.__riddle; r.go({ kind: "death", death: { run_id: 1, depth: 3, cause: "goblin_pack", margin: "3 over", verdict: "gap", baseline: 0.4, trace: { turns: [] }, patches: [], morgue: "" }, lost: ["jackal · Ashar"] }); });
    await sleep(300);
    const d = await page.evaluate(() => ({ line: document.querySelector(".death-line")?.textContent.replace(/\s+/g, " ").trim(), egg: document.querySelector(".death .chip.egg")?.textContent.replace(/\s+/g, " ").trim() }));
    check(/goblin pack · D3 · 3 hp short · gap/.test(d.line ?? ""), `the death line reads "${d.line}"`);
    check(d.egg === "◯ jackal Ashar fell", `the lost companion reads "${d.egg}"`);
  }
  // §3 the report: `returned · banked · deaths` when returns outnumber banks; exit lines lead with `returned $61`; lost chips `fell`;
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
    check(rep.labels.join(" ") === "runs best marks returned banked deaths", `tiles: ${rep.labels.join(" · ")}`);
    check(rep.exits[0]?.startsWith("returned $61 · $102 carried") && rep.exits[1]?.startsWith("banked $84 · "), `exit lines lead with the tier and the sum: "${rep.exits[0]}"`);
    check(rep.lost === "◯ jackal Ashar fell", `the report's lost chip reads "${rep.lost}"`);
    check(rep.fade === true, "after an absence the tiles fade in");
    await page.evaluate((b) => { const r = window.__riddle; b.live = r.lineage.live ?? null; r.go({ kind: "report", report: { ...b, banked: 3, returned: 1 } }); }, base);
    await sleep(200);
    const rep2 = await page.evaluate(() => ({ labels: [...document.querySelectorAll(".report .tiles .tile .label")].map((l) => l.textContent.trim()), fade: document.querySelector(".report .tiles")?.classList.contains("fade-in") }));
    check(rep2.labels.join(" ") === "runs best marks banked returned deaths" && rep2.fade === false, `banked leads when it is the larger; a watched run's tiles do not fade (${rep2.labels.slice(3).join(" · ")})`);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`clarity: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`clarity: ok (${out.length} checks)`);
