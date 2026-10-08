#!/usr/bin/env node
// QA ad71e72 (blind rater A, seed 73): watching a live delve, the picture showed nothing new for 40–55 s at D8 / D10 and stood
// > 2 min at D18 — a boss standoff (the Warlord's shield wall, the Lich's endless dead) is a dead stretch (no hp change on the boss,
// no kill of note, no descent), and the dead stretch's rate rode the engine's short lead (~16–40×), so a 6 000-tick standoff took
// ~18 s of wall time with one repeated line. Rater B: `−0 hp` lines spamming the combat log.
// Gates, on the fake engine's shrug fight (`fake_shrug`: every blow hits for 0 both ways — a fight that cannot progress; `fake_god`
// keeps the hero up; `fake_depth=5`, the boss floor) with each engine step slowed 15 ms (the real wasm's step cost):
//   · in `fights` and the plain 1× (`one`), no stretch of more than 6 s with the dead flag up and no news (a move, a jump, a floor);
//   · the dead stretch jumps (`data-jumps` ≥ 1) and the whole standoff is crossed well under the old ~18 s;
//   · the clock eases up (blind 1fb7786, B): no single change to > 64× from under half of it;
//   · the combat log never reads `−0 hp` / `+0 hp` (a zero hit reads `blocked`).
//   node web/tests/watch-live-edge.mjs
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
try {
  for (const mode of ["fights", "one"]) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
    page.on("pageerror", (e) => errors.push(`${mode} pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&early=0&fake_god=1&fake_depth=5&fake_shrug=3000`, { waitUntil: "domcontentloaded" });
    await page.waitForFunction(() => window.__riddle?.screen === "watch" && document.querySelector(".watch")?.dataset.tick, null, { timeout: 30_000 });
    if (mode !== "fights") { await page.locator("[data-tile=speed]").click(); await page.locator(`.sheet [data-tile=${mode}]`).click(); }
    await page.waitForFunction((m) => document.querySelector(".watch")?.dataset.mode === m, mode, { timeout: 5000 });
    const r = await page.evaluate(async () => {
      const app = window.__riddle, step = app.engine.step.bind(app.engine);
      app.engine.step = async (n) => { await new Promise((z) => setTimeout(z, 15)); return step(n); };   // the real step's cost
      // blind 1fb7786 (B: "1× to 107× … jumpy"): every clock change, as the watch sets it
      const jumps = []; let was = Number(document.querySelector(".watch")?.dataset.speed ?? 1), top = was;
      const mo = new MutationObserver(() => { const n = Number(document.querySelector(".watch")?.dataset.speed ?? 0); if (n === was) return; if (n > 64 && n > 2.05 * Math.max(was, 16)) jumps.push(`${was}→${n}`); top = Math.max(top, n); was = n; });
      mo.observe(document.querySelector(".watch"), { attributes: true, attributeFilter: ["data-speed"] });
      const t0 = performance.now(); let news = "", newsAt = t0, worst = 0, worstAt = null, deadMs = 0, last = t0, zero = [];
      while (performance.now() - t0 < 60_000) {
        const w = document.querySelector(".watch"); if (!w || app.screen !== "watch") break;
        const d = w.dataset, now = performance.now();
        const key = `${d.progress}|${d.jumps ?? 0}|${document.querySelector(".hud .depth")?.textContent}|${d.ending ?? ""}`;
        if (key !== news || d.dead !== "1" || d.over === "1") { news = key; newsAt = now; }
        if (now - newsAt > worst) { worst = now - newsAt; worstAt = { tick: d.tick, frontier: d.frontier, speed: d.speed, frame: d.frame }; }
        if (d.dead === "1") deadMs += now - last;
        last = now;
        for (const li of document.querySelectorAll(".combat-log li")) if (/[−+-]0 hp/.test(li.textContent)) zero.push(li.textContent);
        if (d.over === "1") break;
        await new Promise((z) => setTimeout(z, 100));
      }
      mo.disconnect();
      const d = document.querySelector(".watch")?.dataset ?? {};
      return { jumps2: jumps.slice(0, 4), top, worst: Math.round(worst), worstAt, deadS: Math.round(deadMs / 100) / 10, jumps: Number(d.jumps ?? 0), wall: Math.round((performance.now() - t0) / 100) / 10, zero: [...new Set(zero)].slice(0, 3), blocked: [...document.querySelectorAll(".combat-log li")].some((li) => /blocked$/.test(li.textContent)) };
    });
    check(r.worst <= 6000, `${mode}: no dead stretch over 6 s without news (worst ${r.worst} ms at ${JSON.stringify(r.worstAt)})`);
    check(r.jumps >= 1, `${mode}: the standoff jumps (${r.jumps} jumps; ${r.deadS} s dead over a ${r.wall} s watch)`);
    check(r.deadS <= 12, `${mode}: the 6 000-tick standoff crossed in ≤ 12 s of dead picture (${r.deadS} s; was ~18 s)`);
    check(r.jumps2.length === 0, `${mode}: the clock eases up — never one step past 64× from under half of it (${r.jumps2.join(" · ") || "none"}; top ${r.top}×)`);
    check(r.zero.length === 0 && r.blocked, `${mode}: no "−0 hp" in the combat log; a zero hit reads "blocked" (${r.zero.join(" / ") || "none"})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`watch-live-edge: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`watch-live-edge: ok (${out.length} checks)`);
