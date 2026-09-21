#!/usr/bin/env node
// Cut 10 §1 gates: `fights` is the default mode and the map shows only as the interstitial card (`D3 · 4 rooms · $47`) while
// the engine runs the travel underneath; a fight cuts in at 1× (frame `fight`, speed 1); `▶▶|` under the card reaches the
// next fight in one press; `▶▶|` inside a fight jumps to its end; tapping the card holds the map at 8× until the next fight;
// `fast` is the old auto. Runs on the GPU harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219) with the
// fake engine (`?engine=fake&dev=1`; the fake's fights are frequent and its hero takes hits, so fights are shown).
//
//   node web/tests/fights.mjs        (part of `pnpm test` in web/)
//
// The watch exposes `data-mode`, `data-frame`, `data-card`, `data-speed`, `data-fights` on `.watch` for tooling.
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

const state = () => page.evaluate(() => {
  const r = window.__riddle, w = document.querySelector(".watch");
  return r ? { screen: r.screen, booted: r.booted, mode: w?.dataset.mode, frame: w?.dataset.frame, card: w?.dataset.card, speed: Number(w?.dataset.speed), fights: Number(w?.dataset.fights ?? 0), tick: Number(w?.dataset.tick), cardText: document.querySelector(".interstitial")?.textContent ?? "", cardShown: !!document.querySelector(".interstitial:not([hidden])"), buttons: [...document.querySelectorAll(".hud.bottom .hud-btn")].map((b) => b.textContent), on: [...document.querySelectorAll(".hud.bottom .hud-btn.on")].map((b) => b.textContent), vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null;
});
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state();
    if (s?.screen === "exit" && s.vault) { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(100); continue; }
    if (pred(s)) return s;
    await sleep(40);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} frame=${s?.frame} card=${s?.card} speed=${s?.speed})`);
}
const press = (label) => page.evaluate((l) => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === l) { b.click(); return true; } return false; }, label);
const inRun = (s) => s?.screen === "watch";

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=5&autosend=1`, { waitUntil: "domcontentloaded" });
  let s = await waitFor((x) => x?.booted && inRun(x) && x.mode, "the watch");
  check(s.mode === "fights" && s.on.join() === "fights", `fights is the default mode (on: ${s.on.join(", ")})`);
  check(s.buttons.join(" ") === "fights fast ▶▶| bail", `the buttons read fights · fast · ▶▶| · bail (${s.buttons.join(" · ")})`);
  // the card: the ambient line over the map, the clock held; then the first fight at 1×
  s = await waitFor((x) => !inRun(x) || x.card === "1", "the interstitial", 8000);
  check(s.card === "1" && s.cardShown && /^D\d+( · \d+ rooms)? · \$\d+$/.test(s.cardText), `the interstitial reads the ambient line: "${s.cardText}"`);
  check(s.speed === 0, `the clock holds under the card (speed ${s.speed})`);
  // a tap on the card holds the map at 8× until the next fight (shown whatever it costs)
  await page.locator(".interstitial").click({ timeout: 2000 });
  s = await waitFor((x) => !inRun(x) || (x.card === "0" && x.frame === "map"), "the map after tapping the card", 2000);
  check(s.card === "0" && s.frame === "map" && s.speed === 8, `tapping the card shows the map at 8× (speed ${s.speed}, card ${s.card})`);
  s = await waitFor((x) => !inRun(x) || x.frame === "fight", "the next fight after the hold", 30_000);
  check(inRun(s) && s.frame === "fight" && s.fights === 1 && s.speed === 1 && s.card === "0", `the held map cuts to the next fight at 1× (fights ${s.fights}, speed ${s.speed})`);
  // ▶▶| inside a fight: the fight's end, the card back up within 2 s
  await press("▶▶|");
  s = await waitFor((x) => !inRun(x) || x.frame === "map", "the map after skipping a fight", 3000);
  check(s.frame === "map" && s.card === "1", `▶▶| in a fight jumps to its end (card ${s.card}, frame ${s.frame})`);
  // ▶▶| under the card: the next fight's first frame in one press (the card's minimum waived), or the run's end
  const f1 = s.fights, t = Date.now(), tick0 = s.tick;
  await press("▶▶|");
  s = await waitFor((x) => !inRun(x) || x.frame === "fight", "the next fight after one press", 4000);
  check(!inRun(s) || (s.frame === "fight" && s.fights === f1 + 1 && Date.now() - t < 2500), `one ▶▶| under the card reaches the next fight (${s.frame}, fights ${f1} → ${s.fights}, ${Date.now() - t} ms, tick ${tick0} → ${s.tick})`);
  // `fast`: the old auto — no card, 8× through dead stretches and 1× near
  if (inRun(s)) {
    await press("fast");
    s = await waitFor((x) => !inRun(x) || (x.mode === "fast" && x.card === "0"), "fast mode", 2000);
    check(s.mode === "fast" && s.on.join() === "fast" && s.card === "0" && (s.speed === 8 || s.speed === 1), `fast: the card is gone and the clock runs 8× / 1× (speed ${s.speed})`);
    await press("fights");
    s = await waitFor((x) => !inRun(x) || x.mode === "fights", "fights mode again", 2000);
    check(s.mode === "fights" && s.on.join() === "fights", "fights again");
  }
  // the run ends on its own within the budget
  s = await waitFor((x) => x && x.screen !== "watch", "the run's end", 120_000);
  check(["exit", "death", "report", "camp"].includes(s.screen), `the run reached its end (${s.screen})`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`fights: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`fights: ok (${out.length} checks)`);
