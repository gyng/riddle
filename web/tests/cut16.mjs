#!/usr/bin/env node
// Cut 16 gates, client side (docs/CUT16.md §1–§3; §4's boss bar and break beat are in fights.mjs), on the fake engine
// (`?engine=fake&dev=1`) through the browser harness (tools/browser.mjs; `--shots` runs headed on the GPU and writes
// scratchpad/cut16/*.png at 400×800×3) against the dev server (tools/dev.sh, :5219; RIDDLE_PORT overrides):
//   §1  the report's thinned depths as one dim line under the tiles (`D3–4 · D6 · picked clean`); the camp's forecast carries the
//       same line (small, dim) while `Lineage.picked` holds depths, and none without
//   §2  at the wake (a trait offer standing) with ≥ 2 classes owned, a class chip per owned class beside the trait chips
//       (`fighter L1 · shield bash`, `rogue L1 · vanish`, a signature not yet open `mark L7`), the current one on and inert; a tap
//       is `setClass` and the pick sticks through the send (the run's class, the camp after it); the row is gone once the offer is
//       spent, and absent with one class owned
//   §3  a Burrows floor (the fake's D4–5; `fake_depth=5`) names its biome on the card (`D5 · the Burrows · …`) and loads the
//       `burrows` biome; `--shots` saves the floor on the GPU (05-burrows.png)
//
//   node web/tests/cut16.mjs [--shots]       (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser, launchGpu } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shots = process.argv.includes("--shots") ? resolve(ROOT, "scratchpad/cut16") : null;
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = shots ? await launchGpu() : await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: shots ? 3 : 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => {
  const r = window.__riddle, w = document.querySelector(".watch");
  return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, frame: w?.dataset.frame, vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null;
});
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state();
    if (s?.screen === "exit" && s.vault) { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(100); continue; }
    if (pred(s)) return s;
    await sleep(60);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`) }); };
/** Patch the fake's saved state (the lineage) and reload it through the save. */
const patchSave = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  new Function("e", src)(e);
  b.engine = JSON.stringify(e);
  return r.importSave(JSON.stringify(b));
}, `(${fn})(e)`);
const emptyReport = (extra) => page.evaluate((x) => { const r = window.__riddle; const L = r.lineage; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 4, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 2, returned: 2, ...x } }); }, extra);

try {
  // ---- §2: the wake's class chips
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && x.screen === "camp", "camp");
  await sleep(300);
  const classes = () => page.evaluate(() => ({
    chips: [...document.querySelectorAll(".strip .chip.cls-offer")].map((c) => ({ cls: c.dataset.class, top: c.querySelector("span")?.textContent ?? "", sig: c.querySelector(".rule")?.textContent ?? "", on: c.classList.contains("on"), disabled: c.disabled })),
    traits: document.querySelectorAll(".strip .chip.trait").length, clsBtn: !!document.querySelector("button.cls"),
    cls: window.__riddle.lineage.class, offer: window.__riddle.lineage.class_offer ?? null,
  }));
  let c = await classes();
  check(c.offer === null && c.chips.length === 0 && c.clsBtn && c.traits === 2, `one class owned: no class row, the class button stands (${c.chips.length} chips, ${c.traits} trait chips)`);
  check(await patchSave((e) => { e.lineage.unlocks.push("rogue", "ranger"); }), "the save takes rogue and ranger");
  await waitFor((x) => x?.booted && x.screen === "camp", "camp with three classes");
  await sleep(300);
  c = await classes();
  check(c.chips.length === 3 && c.traits === 2 && !c.clsBtn, `beside the trait chips, a chip per owned class: ${c.chips.map((x) => `${x.top} · ${x.sig}`).join(" | ")}`);
  check(c.chips[0]?.cls === c.cls && c.chips[0].on && c.chips[0].disabled && c.chips.filter((x) => x.on).length === 1, `the current class first, on and inert (${c.cls})`);
  const rogue = c.chips.find((x) => x.cls === "rogue"), ranger = c.chips.find((x) => x.cls === "ranger");
  check(rogue?.sig === "vanish" && /^mark L7$/.test(ranger?.sig ?? ""), `each carries its signature; one not yet open reads its level (rogue "${rogue?.sig}", ranger "${ranger?.sig}")`);
  check(c.chips.every((x) => x.sig.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length <= 3), "each under-label ≤ 3 words");
  await shot("01-class-offer");
  await page.locator(".strip .chip.cls-offer[data-class=rogue]").click({ timeout: 5000 });
  await sleep(400);
  c = await classes();
  check(c.cls === "rogue" && c.chips.find((x) => x.on)?.cls === "rogue" && c.chips.length === 3, `a tap picks the class (${c.cls}, the rogue chip lit)`);
  // the pick sticks: the run is the rogue's, the camp after it too; the send spends the offer and the row goes
  await page.evaluate(() => window.__riddle.go({ kind: "watch" }));
  await waitFor((x) => x?.screen === "watch", "the watch");
  await sleep(800);
  const heroKind = await page.evaluate(() => window.__viewer?.debugPos?.().find((e) => e.hero)?.kind ?? "");
  await page.evaluate(async () => { await window.__riddle.refresh(); window.__riddle.go({ kind: "camp" }); });
  await waitFor((x) => x?.screen === "camp", "camp again");
  await sleep(300);
  c = await classes();
  check(c.cls === "rogue" && (!heroKind || /rogue/.test(heroKind)), `the pick sticks through the send (lineage ${c.cls}, the run's hero ${heroKind || "—"})`);
  check(c.chips.length === 0 && c.offer === null && c.clsBtn, `after the send the class row is gone and the class button reads it (${c.chips.length} chips)`);

  // ---- §1: picked clean
  await emptyReport({ picked: [3, 4, 6] });
  await waitFor((x) => x?.screen === "report", "the report");
  await sleep(200);
  let line = await page.evaluate(() => { const e = document.querySelector(".report .picked-line"); return e ? { text: e.textContent, dim: e.classList.contains("dim") } : null; });
  check(line?.text === "D3–4 · D6 · picked clean" && line.dim, `the report's thinned depths in one dim line ("${line?.text}")`);
  await shot("02-report-picked");
  await emptyReport({});
  await sleep(150);
  line = await page.evaluate(() => document.querySelector(".report .picked-line")?.textContent ?? null);
  check(line === null, "none picked: no line");
  check(await patchSave((e) => { e.lineage.picked = [2, 3]; }), "the save takes picked D2–3");
  await waitFor((x) => x?.booted && x.screen === "camp", "camp with picked depths");
  const fcPicked = async () => { const t = Date.now(); let v = null; while (Date.now() - t < 8000) { v = await page.evaluate(() => { const e = document.querySelector(".forecast .fc-picked"); return e && !e.hidden ? e.textContent : null; }); if (v) break; await sleep(100); } return v; };
  const fp = await fcPicked();
  check(fp === "D2–3 · picked clean", `the camp's forecast says it ("${fp}")`);
  await shot("03-forecast-picked");
  check(await patchSave((e) => { delete e.lineage.picked; }), "the save drops picked");
  await waitFor((x) => x?.booted && x.screen === "camp", "camp without picked depths");
  await sleep(500);
  check((await page.evaluate(() => { const e = document.querySelector(".forecast .fc-picked"); return !!e && !e.hidden; })) === false, "none picked: the forecast has no line");

  // ---- §3: the Burrows
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7&autosend=1&fake_depth=5`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && x.screen === "watch", "the watch on D5");
  let card = null, biome = null; const t0 = Date.now();
  while (Date.now() - t0 < 10_000 && !card) {
    const x = await page.evaluate(() => { const c = document.querySelector(".interstitial"); return { card: c && !c.hidden ? c.textContent : null, biome: window.__viewer?.debugBiome?.() ?? null }; });
    if (x.card) card = x.card; biome = x.biome ?? biome;
    await sleep(40);
  }
  check(/^D5 · the Burrows · /.test(card ?? ""), `a Burrows floor's card names it ("${card}")`);
  check(biome === "burrows", `the floor draws in the burrows biome (${biome})`);
  if (shots) {
    // the map frame (the card over it is tapped away: the map held at 8×) — the palette on the GPU
    await page.locator(".interstitial").click({ timeout: 3000 }).catch(() => {});
    await sleep(700);
    await shot("05-burrows");
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut16: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut16: ok (${out.length} checks)`);
