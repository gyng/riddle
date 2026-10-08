#!/usr/bin/env node
// The pen's editor on packages (blind rater A on c4705f9: "▲▼ on pre-written rows reverted on screen", "APPLY never changed the list",
// "`5/4 · drop one` blocked SEND after a row I did not add", "× needed a double tap with no cue"), on the fake engine, headless.
//   move     a package row's ▲ takes it into the pen as the player's, and it stays there once the core's order is adopted; the pen's
//            last row has no ▼ (it cannot sit under a package); package rows carry no ×
//   cap      on a full set a package row's ▲ is off; a move patch that takes a row asks first (`patchTakesRow`) and, over a drop, lands
//            without crossing the cap
//   fold     the edit tile brings the editor's first row into view (blind 5331f40 A: the rows sat below the fold)
//   delete   the first × tap arms it visibly (`drop`, the tablet ringed) and keeps the row; the second deletes it
//
//   node web/tests/pen-edit.mjs [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
async function until(pred, label, timeout = 15_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const withState = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const save = JSON.parse(await r.engine.save());
  new Function("e", src)(save);
  r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
}, `(${fn})(e)`);
/** A lineage with the pen open (the fake: Bloat Mother met at D13), the editor up. */
async function penOpen(seed) {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}`, { waitUntil: "domcontentloaded" });
  await until(() => window.__riddle?.booted && window.__riddle.screen === "camp", "camp");
  await withState((e) => { e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 12, cause: "goblin" }]; e.lineage.best_depth = 14; e.lineage.gold = 80; });
  await until(() => window.__riddle.lineage.packages?.pen_open && document.querySelector('.cmd .tile[data-tile="edit"]'), "the pen");
  await page.click('.cmd .tile[data-tile="edit"]');
  await until(() => document.querySelector(".camp .editor .row.tablet .updown"), "the editor's moves");
}
const state = () => page.evaluate(async () => {
  const r = window.__riddle, L = await r.engine.lineage();
  const key = (x) => `${x.verb.v}${x.verb.a ? " " + x.verb.a : ""}|${x.origin ?? ""}`;
  return { mine: r.rules.rows.map(key), core: L.sets[L.active_set].rows.map(key), end: r.penEnd(), max: r.vocab.max_rows, own: r.ownRows() };
});
const tablet = (i) => `.camp .editor .rows > .row.tablet[data-i="${i}"]`;

try {
  if (part("move")) {
    await penOpen(4101);
    const s0 = await state();
    check(s0.end === 0 && s0.own === 0, `the pen starts empty above the packages (end ${s0.end}, own ${s0.own})`);
    const xs = await page.evaluate(() => [...document.querySelectorAll(".camp .editor .rows > .row.tablet")].map((t) => !!t.querySelector(":scope > button.x")));
    check(xs.length > 0 && xs.every((x) => !x), `package rows carry no × (${xs.join(",")})`);
    const downOff = await page.evaluate((sel) => document.querySelector(sel)?.disabled, `${tablet(1)} .step.down`);
    check(downOff === true, "a package row has no ▼ (its order is the package's)");
    const moved = s0.mine[2].split("|")[0];
    await page.click(`${tablet(2)} .step.up`);
    const s1 = await until((m) => { const r = window.__riddle; return r.rules.rows[0] && `${r.rules.rows[0].verb.v}${r.rules.rows[0].verb.a ? " " + r.rules.rows[0].verb.a : ""}` === m ? true : null; }, "the taken row at the top", 4000, moved).then(state);
    check(s1.mine[0] === `${moved}|player` && s1.end === 1 && s1.own === 1, `▲ on a package row takes it into the pen as the player's (${s1.mine.join(" · ")})`);
    await sleep(1500);   // the core's order adopted (App.adoptCompiled)
    const s2 = await state();
    check(s2.mine[0] === `${moved}|player` && JSON.stringify(s2.mine) === JSON.stringify(s2.core), `the move persists in the core and on screen (${s2.core.join(" · ")})`);
    check(s2.mine.filter((k) => k.startsWith(moved + "|")).length === 1, `the package's copy yields to the pen's (${s2.mine.join(" · ")})`);
    const penDown = await page.evaluate((sel) => document.querySelector(sel)?.disabled, `${tablet(0)} .step.down`);
    check(penDown === true, "the pen's last row has no ▼ (it never sits under a package)");
    // blind 1fb7786 (B: "pinned at the top, step-down disabled"): the dead ▼ says why, and the row carries the line
    const penEnd = await page.evaluate((sel) => ({ title: document.querySelector(`${sel} .step.down`)?.title, note: document.querySelector(`${sel} .pen-end-note`)?.textContent, others: document.querySelectorAll(".camp .editor .pen-end-note").length }), tablet(0));
    check(penEnd.title === "packages below" && penEnd.note === "▼ packages below" && penEnd.others === 1, `the pen's end reads \`packages below\` on its ▼ and its row, once (${JSON.stringify(penEnd)})`);
  }

  if (part("fold")) {
    // blind 5331f40 (A: "tapping `edit` only showed a tooltip; the rows sat under the town and needed a scroll"): the edit tile brings
    // the editor's rows into view
    await penOpen(4104);
    await sleep(300);
    const seen = await page.evaluate(() => {
      const r = document.querySelector(".camp .editor .rows > .row.tablet")?.getBoundingClientRect(), w = document.querySelector(".camp .well")?.getBoundingClientRect();
      if (!r || !w) return { r: !!r, w: !!w };
      const cx = r.left + r.width / 2, cy = r.top + Math.min(r.height / 2, 12), hit = document.elementFromPoint(cx, cy);
      return { top: Math.round(r.top), bottom: Math.round(r.bottom), wellTop: Math.round(w.top), wellBottom: Math.round(w.bottom), vh: innerHeight, onRow: !!hit?.closest(".editor") };
    });
    check(seen.top >= seen.wellTop - 1 && seen.top < Math.min(seen.wellBottom, seen.vh) - 20 && seen.onRow, `after \`edit\` the editor's first row is in view, uncovered (${JSON.stringify(seen)})`);
  }

  if (part("cap")) {
    await penOpen(4102);
    const s0 = await state();
    // fill the pen with `+` to the cap
    for (let i = 0; i < s0.max; i++) { await page.click(".camp .editor .rows-foot .btn.ghost"); await sleep(120); }
    const s1 = await until(() => { const r = window.__riddle; return r.ownRows() >= r.vocab.max_rows ? true : null; }, "a full pen").then(state);
    check(s1.own === s1.max && s1.end === s1.max, `\`+\` fills the pen above the packages (${s1.own}/${s1.max}, end ${s1.end})`);
    const upOff = await page.evaluate((sel) => document.querySelector(sel)?.disabled, `${tablet(s1.end + 1)} .step.up`);
    check(upOff === true, "on a full set a package row's ▲ is off");
    const p = await page.evaluate((end) => { const r = window.__riddle; const p = { row: r.rules.rows[end + 1], insert_at: 0, moves_from: end + 1, survive: 0, forecast_delta: 0 }; return { takes: r.patchTakesRow(p), full: r.rowsFull }; }, s1.end);
    check(p.takes && p.full, `a move patch on a package row takes a row — on a full set it asks first (${JSON.stringify(p)})`);
    const after = await page.evaluate((end) => { const r = window.__riddle; const row = r.rules.rows[end + 1]; const at = r.applyPatchOver({ row, insert_at: 0, moves_from: end + 1, survive: 0, forecast_delta: 0 }, end - 1); return { at, own: r.ownRows(), max: r.vocab.max_rows, over: r.overBudget, origin: r.rules.rows[at]?.origin, same: JSON.stringify([r.rules.rows[at]?.conds, r.rules.rows[at]?.verb]) === JSON.stringify([row.conds, row.verb]) }; }, s1.end);
    check(after.same && after.origin === "patch" && after.own === after.max && !after.over, `over a drop the move lands in the pen, never over the cap (${JSON.stringify(after)})`);
    await sleep(800);
    const send = await page.evaluate(() => document.querySelector(".camp .send, .camp [data-tile='send']")?.textContent ?? "");
    check(!/drop one/.test(send), `send is not blocked by the cap (${send.trim()})`);
  }

  if (part("delete")) {
    await penOpen(4103);
    await page.click(".camp .editor .rows-foot .btn.ghost");
    await until(() => document.querySelector('.camp .editor .rows > .row.tablet[data-i="0"] > button.x'), "the written row's ×");
    const n0 = await page.evaluate(() => window.__riddle.rules.rows.length);
    await page.click(`${tablet(0)} > button.x`);
    const armed = await page.evaluate((sel) => { const b = document.querySelector(`${sel} > button.x`); const cs = b ? getComputedStyle(b) : null; const t = document.querySelector(sel); return { cls: b?.classList.contains("armed"), text: b?.textContent, color: cs?.color, ring: t ? getComputedStyle(t).outlineStyle : "", rows: window.__riddle.rules.rows.length }; }, tablet(0));
    check(armed.cls && armed.text === "drop" && armed.ring !== "none" && armed.rows === n0, `the first × tap arms it in sight — \`drop\`, the tablet ringed, the row kept (${JSON.stringify(armed)})`);
    await page.click(`${tablet(0)} > button.x`);
    const n1 = await until((n) => window.__riddle.rules.rows.length === n - 1 ? window.__riddle.rules.rows.length : null, "the delete", 4000, n0);
    check(n1 === n0 - 1, `the second tap deletes it (${n0} → ${n1})`);
  }
} catch (e) {
  check(false, `threw: ${e.message}`);
} finally {
  await browser.close();
}
check(errors.length === 0, `no console errors (${errors.length})`);
for (const l of out) console.log(l);
for (const e of errors) console.log(`  ${e}`);
console.log(failed ? `pen-edit: ${failed} FAILED` : "pen-edit: all ok");
process.exit(failed ? 1 : 0);
