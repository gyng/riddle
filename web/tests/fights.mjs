#!/usr/bin/env node
// Cut 10 §1 gates: `fights` is the default mode and the map shows only as the interstitial card (`D3 · 4 rooms · $47`) while
// the engine runs the travel underneath; a fight cuts in at 1× (frame `fight`, speed 1); `▶▶|` under the card reaches the
// next fight in one press; `▶▶|` inside a fight jumps to its end; tapping the card holds the map at 8× until the next fight;
// `fast` is the old auto. Runs on the GPU harness (tools/browser.mjs) against the dev server (tools/dev.sh, :5219) with the
// fake engine (`?engine=fake&systems=none&dev=1`; the fake's fights are frequent and its hero takes hits, so fights are shown).
// Cut 12 §4: from D3 the card names the floor's situation (`D4 · 9 rooms · a nest`). Cut 12 §6: a second run in `fast` — travel
// at 16×, a fight at 2×. QA on 50bb162: `▶▶|` in `fast` reaches the run's END in one press (the engine steps to `run_over`, the
// ending plays at 1×) — five players read the old "next fight" press as "plays faster"; in `fights` it stays the next fight.
// Cut 14: the fight frame runs at 2× in `fights` and 4× in `fast` (the slowdowns were "way too slow").
// Cut 14 §3: a stack (two foes on one tile) fans sideways in the map frame with a name per row, a foe there is ≥ 24 CSS px tall
// (`__viewer.debugRects` / `debugLabels`); a bank exit opens the fight frame with `BANKED $N` as its callout before the sheet.
// Cut 15 §4: a boss's kill opens (holds) the fight frame with `GOBLIN WARLORD DOWN` in `fast` and `fights`; two name tags whose
// boxes would intersect draw on two rows (no two tags drawn in a frame intersect).
// Cut 28 §3: at 1× no stretch > 5 s without a fight, a beat, a pickup of note or a descent.
// Cut 24 §1: no 15 s of dead watch — a stretch with no hp change, kill, pickup or descent plays at the travel rate (fights included).
// Cut 16 §4: while a boss is in view the HUD carries his bar under the hero's (`warlord` + a thin track at his hp); `warlord
// breaks` (the core's callout + note) is a beat — the fight frame holds on `WARLORD BREAKS` like the kill. §3: the run's first
// floor of a biome names it on the card (`D1 · the Warrens · $0`).
//
//   node web/tests/fights.mjs        (part of `pnpm test` in web/)
//
// The watch exposes `data-mode`, `data-frame`, `data-card`, `data-speed`, `data-fights` on `.watch` for tooling.
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { measured } from "./lib/load.mjs";

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
  return r ? { screen: r.screen, booted: r.booted, mode: w?.dataset.mode, frame: w?.dataset.frame, card: w?.dataset.card, speed: Number(w?.dataset.speed), fights: Number(w?.dataset.fights ?? 0), tick: Number(w?.dataset.tick), ending: w?.dataset.ending === "1", held: w?.dataset.held === "1", cardText: document.querySelector(".interstitial")?.textContent ?? "", cardShown: !!document.querySelector(".interstitial:not([hidden])"), buttons: [...document.querySelectorAll(".cmd .hud-btn")].map((b) => b.textContent), on: [...document.querySelectorAll(".cmd .hud-btn.on")].map((b) => b.textContent), vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null;
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
  // Cut 20 §3: on D1–D3 `fights` shows no card — the travel at 8×, every fight at 1.5× (early runs were card-skipped travel "too short
  // to follow"); the card's own checks below turn that off (`early=0`)
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1`, { waitUntil: "domcontentloaded" });
  {
    const early = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(), seen = { card: 0, map: new Set(), fight: new Set(), depth: "" };
      const poll = () => {
        const w = document.querySelector(".watch");
        if (w && window.__riddle?.screen === "watch" && w.dataset.frame) {
          seen.depth = document.querySelector(".hud .depth")?.textContent ?? "";
          if (w.dataset.card === "1") seen.card++;
          if (w.dataset.held !== "1" && w.dataset.speed !== "0") (w.dataset.frame === "fight" ? seen.fight : seen.map).add(Number(w.dataset.speed));
        }
        if (performance.now() - t0 > 8000 || window.__riddle?.screen !== "watch" && performance.now() - t0 > 3000) { res({ card: seen.card, map: [...seen.map], fight: [...seen.fight], depth: seen.depth }); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    // QA e75ec29 (R: 28–40 s on D1 at a flat 8×): the travel starts at 8× and ramps as `fast`'s dead stretch does (never under 8×)
    check(early.card === 0 && early.map.includes(8) && early.map.every((r) => r >= 8 || r === 1) && early.fight.includes(1.5), `fights on ${early.depth || "D1"}: no card, the map from 8× up, a fight at 1.5× (card frames ${early.card}; map ${early.map.join("/")}; fight ${early.fight.map((r) => Math.round(r * 100) / 100).join("/")})`);
  }
  // (the fake's D4 kills a hero in his first costly fight: the card's gates run on its gentle D1 with the first floors' mode off, `early=0`)
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&autosend=1&early=0`, { waitUntil: "domcontentloaded" });
  let s = await waitFor((x) => x?.booted && inRun(x) && x.mode, "the watch");
  check(s.mode === "fights" && s.on.join() === "fights only", `fights is the default mode (on: ${s.on.join(", ")})`);
  check(s.buttons.join(" ") === "fights only fast normal ▶▶| bail town", `the buttons read fights only · fast · normal · ▶▶| · bail · town (Cut 25 §3: the plain 1×, docs/COPY.md: its word \`normal\`; RUNS_UI: back to the town, the run goes on) (${s.buttons.join(" · ")})`);
  // the card: the ambient line over the map, the clock held; then the first fight at 1×
  // Cut 15 §4: the card is short (≤ 1.2 s; 0.5 s before a beat — seed 5 opens on a situation), so the tap is made in the page the
  // frame the card is seen
  s = await page.evaluate(() => new Promise((res) => {
    const t0 = performance.now();
    const poll = () => {
      const w = document.querySelector(".watch"), c = document.querySelector(".interstitial");
      if (w?.dataset.card === "1" && c && !c.hidden) { const seen = { card: "1", cardShown: true, cardText: c.textContent, speed: Number(w.dataset.speed) }; c.click(); res(seen); return; }
      if (performance.now() - t0 > 8000) { res({ card: w?.dataset.card, cardShown: false, cardText: "", speed: NaN }); return; }
      requestAnimationFrame(poll);
    };
    poll();
  }));
  check(s.card === "1" && s.cardShown && /^D\d+( · \d+ rooms| · the [A-Z][a-z]+)? · (carry \$\d+|an? [a-z]+)$/.test(s.cardText), `the interstitial reads the ambient line: "${s.cardText}"`);
  check(s.speed === 0, `the clock holds under the card (speed ${s.speed})`);
  // a tap on the card holds the map at 8× until the next fight (shown whatever it costs) — tapped above
  s = await waitFor((x) => !inRun(x) || (x.card === "0" && x.frame === "map"), "the map after tapping the card", 2000);
  check(s.card === "0" && s.frame === "map" && s.speed === 8, `tapping the card shows the map at 8× (speed ${s.speed}, card ${s.card})`);
  s = await waitFor((x) => !inRun(x) || x.frame === "fight", "the next fight after the hold", 30_000);
  // Cut 18 §1: a beat held on the cut (seed 5's cage) eases the clock under its hold (`data-held`)
  check(inRun(s) && s.frame === "fight" && s.fights === 1 && (s.speed === 2 || (s.held && s.speed <= 2)) && s.card === "0", `the held map cuts to the next fight at 2× (fights ${s.fights}, speed ${s.speed}${s.held ? ", a beat held" : ""})`);   // Cut 14: 2×, was 1×
  // ▶▶| inside a fight: the fight's end, the card back up within 2 s
  await press("▶▶|");
  s = await waitFor((x) => !inRun(x) || x.frame === "map", "the map after skipping a fight", 3000);
  check(s.frame === "map" && s.card === "1", `▶▶| in a fight jumps to its end (card ${s.card}, frame ${s.frame})`);
  // ▶▶| under the card: the next fight's first frame in one press (the card's minimum waived), or the run's end
  const f1 = s.fights, t = Date.now(), tick0 = s.tick;
  await press("▶▶|");
  s = await waitFor((x) => !inRun(x) || x.frame === "fight", "the next fight after one press", 4000);
  check(!inRun(s) || (s.frame === "fight" && s.fights === f1 + 1 && Date.now() - t < 2500), `one ▶▶| under the card reaches the next fight (${s.frame}, fights ${f1} → ${s.fights}, ${Date.now() - t} ms, tick ${tick0} → ${s.tick})`);
  // `fast`: no card, 16× through dead stretches and 2× near (Cut 12 §6)
  if (inRun(s)) {
    await press("fast");
    s = await waitFor((x) => !inRun(x) || (x.mode === "fast" && x.card === "0"), "fast mode", 2000);
    check(s.mode === "fast" && s.on.join() === "fast" && s.card === "0" && (s.speed >= 16 || s.speed === 4 || s.speed === 1 || (s.held && s.speed <= 4)), `fast: the card is gone and the clock runs 16× / 4× (speed ${s.speed}${s.held ? ", a beat held" : ""})`);
    await press("fights only");
    s = await waitFor((x) => !inRun(x) || x.mode === "fights", "fights mode again", 2000);
    check(s.mode === "fights" && s.on.join() === "fights only", "fights again");
  }
  // the run ends on its own within the budget
  s = await waitFor((x) => x && x.screen !== "watch", "the run's end", 120_000);
  check(["exit", "death", "report", "camp"].includes(s.screen), `the run reached its end (${s.screen})`);

  // Cut 12 §6: a second run in `fast` (seed 157, rater P's): travel 16×, fights 2×; QA on 50bb162: ▶▶| reaches the END
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && inRun(x) && x.mode === "fast", "the fast run");
  check(s.mode === "fast" && s.card === "0", `fast from boot (card ${s.card})`);
  s = await waitFor((x) => !inRun(x) || x.speed >= 32, "32× travel", 8000);
  check(s.speed >= 32, `fast travels at 32× from the first tick (speed ${s.speed})`);   // Cut 20 §3: 32×, was 16× ramping
  // Cut 18 §1: `fast` frames what `fights` frames, a chore stretch inside the frame at its flat rate — the blows at 4×
  s = await waitFor((x) => !inRun(x) || (x.frame === "fight" && x.speed === 4), "a fight in fast", 30_000);
  check(inRun(s) && s.frame === "fight" && s.speed === 4, `fast watches a fight at 4× (speed ${s.speed}, frame ${s.frame})`);   // Cut 14: 4×, was 2×
  // ▶▶| once, inside the fight: the run's end — the viewer lands at the ending (its last 30 ticks play at 1×, `ending`), then
  // the exit flow; the press used to reach the fight's end / the next fight (read as "plays faster" by five players)
  if (inRun(s)) {
    const before = s.tick, t = Date.now();
    await press("▶▶|");
    s = await waitFor((x) => !inRun(x) || x.ending, "the ending after ▶▶| in fast", 20_000).catch(() => s);
    check(!inRun(s) || s.ending, `one ▶▶| in fast reaches the run's end (tick ${before} → ${s.tick}, ${s.ending ? "ending" : s.screen}, ${Date.now() - t} ms)`);
    check(!inRun(s) || s.tick > before, `the clock jumped to the ending (${before} → ${s.tick})`);
    s = await waitFor((x) => x && x.screen !== "watch", "the end of the fast run", 60_000);
    check(["exit", "death", "report", "camp"].includes(s.screen), `the fast run left the watch (${s.screen})`);
  }
  // in `fights` the press stays the next fight (checked above on seed 5)

  // Cut 14 §3: a stack fans in the map frame and its names take two rows — two foes put on the hero's tile in the viewer (the
  // fake keeps its monsters apart), the clock paused, the viewer sought so the spawns apply; then the labels the frame drew
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  // the map frame, paused (a fight can open between the sample and the press: unpause and wait for the map again)
  for (let tries = 0; tries < 6; tries++) {
    s = await waitFor((x) => x?.booted && inRun(x) && x.frame === "map" && x.tick > 30, "the map frame in fast", 20_000);
    await press("⏸"); await sleep(150);
    if ((await state())?.frame === "map") break;
    await press("▶"); await sleep(300);
  }
  const stack = await page.evaluate(async () => {
    const v = window.__viewer, hero = v.debugPos().find((e) => e.hero), t = v.tick();
    const mk = (id, kind, name) => ({ t: t - 5, k: "spawn", e: { id, kind, name, x: hero.x, y: hero.y, hp: 5, max_hp: 5, tags: [] } });
    v.apply([mk(90001, "goblin"), mk(90002, "monkey", "Tain")]); v.seek(t);
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    const w = document.querySelector(".watch"), st = v.stats();
    return { frame: w.dataset.frame, texel: st.k / st.dpr, rects: v.debugRects().filter((r) => r.id >= 90000), labels: v.debugLabels().filter((l) => l.id >= 90000) };
  });
  // the hero shares the tile (a 3-stack: −⅓ · 0 · +⅓ tile), so two neighbours in the fan sit ≥ 2 texels apart
  const [ga, gb] = stack.rects;
  check(stack.frame === "map" && stack.rects.length === 2 && ga.stack >= 3 && Math.abs((ga.x + ga.w / 2) - (gb.x + gb.w / 2)) >= 2 * stack.texel, `two foes on one tile fan sideways in the map frame (${stack.rects.map((r) => `${r.kind} @${Math.round(r.x + r.w / 2)}`).join(" · ")}, texel ${stack.texel} px)`);
  const rows = stack.labels.map((l) => Math.round(l.y));
  check(stack.labels.length === 2 && new Set(rows).size === 2 && Math.abs(rows[0] - rows[1]) >= 10, `their names sit on two rows (${stack.labels.map((l) => `${l.text} y${Math.round(l.y)}`).join(" · ")})`);
  check(stack.rects.every((r) => r.h >= 24), `a foe in the map frame is ≥ 24 CSS px tall (${stack.rects.map((r) => `${r.kind} ${Math.round(r.h)} px`).join(" · ")})`);

  // Cut 14 §3: a bank is a beat — the fight frame opens on the stairs with `BANKED $N` as the callout before the exit sheet (the
  // hero starts on the up stairs, so `depth>=1 → bank` banks on its first action), in `fights`
  const bankRules = encodeURIComponent("foes>=1 → attack nearest\ndepth>=1 → bank");
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&rules=${bankRules}`, { waitUntil: "domcontentloaded" });
  const ticker = () => page.evaluate(() => document.querySelector(".ticker.show")?.textContent ?? "");
  let beat = null;
  const tb = Date.now();
  while (Date.now() - tb < 20_000) {
    const st = await state(); const tk = await ticker();
    if (st?.screen !== "watch" && st?.screen !== "exit") break;
    if (/^BANKED \$\d+$/.test(tk) && st.frame === "fight") { beat = { text: tk, frame: st.frame, at: Date.now() }; break; }
    await sleep(40);
  }
  check(!!beat, `a bank opens the fight frame with the sum as its callout (${beat ? `"${beat.text}"` : "never seen"})`);
  if (beat) {
    await sleep(2500);
    const st = await state(); const tk = await ticker();
    check(st?.screen === "watch" && st.frame === "fight" && tk === beat.text, `the beat holds the frame for its scene (${tk || "gone"} after 2.5 s, ${st?.screen})`);
    s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit", "the report after the bank", 20_000);
    check(s.screen === "report", `then the exit flow (${s.screen})`);
  }

  // Cut 15 §4: a boss's kill is a beat — the fight frame opens (or holds) with `GOBLIN WARLORD DOWN` as its callout and holds
  // ~SCENE_MS, in `fast` and in `fights` (under the card: the beat never waits on it). The fake has no boss kill on D1, so the
  // engine's next batch after tick 20 carries a warlord's spawn beside the hero and its `die`.
  for (const mode of ["fast", "fights"]) {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=${mode}`, { waitUntil: "domcontentloaded" });
    await waitFor((x) => x?.booted && inRun(x) && x.mode === mode, `the ${mode} run for the boss kill`);
    await page.evaluate(() => {
      const r = window.__riddle, orig = r.engine.step.bind(r.engine); let done = false;
      r.engine.step = async (n) => {
        const res = await orig(n);
        if (!done && res.snapshot.turn > 20 && !res.run_over && !res.events.some((e) => e.k === "exit")) {
          done = true; const hh = res.snapshot.hero, t = res.snapshot.turn;
          res.events.push({ t: t - 4, k: "spawn", e: { id: 95001, kind: "goblin_warlord", x: hh.x + 1, y: hh.y, hp: 1, max_hp: 30, tags: ["boss"] } }, { t: t - 1, k: "die", id: 95001, cause: "hero" });
          res.events.sort((a, b) => a.t - b.t);
        }
        return res;
      };
    });
    const both = () => page.evaluate(() => ({ screen: window.__riddle.screen, frame: document.querySelector(".watch")?.dataset.frame, speed: Number(document.querySelector(".watch")?.dataset.speed), tk: document.querySelector(".ticker.show")?.textContent ?? "" }));
    let seen = null; const t0 = Date.now();
    while (Date.now() - t0 < 20_000) {
      const st = await both(), tk = st.tk;
      if (st?.screen !== "watch" && st?.screen !== "exit") break;
      if (tk === "GOBLIN WARLORD DOWN") { seen = { frame: st.frame, speed: st.speed, at: Date.now() }; break; }
      await sleep(40);
    }
    check(!!seen && seen.frame === "fight", `${mode}: a boss kill opens the fight frame with "GOBLIN WARLORD DOWN" (${seen ? `${seen.frame} frame, speed ${seen.speed}` : "never seen"})`);
    if (seen) {
      await sleep(2000);
      const st = await both(), tk = st.tk;
      check(st?.frame === "fight" && tk === "GOBLIN WARLORD DOWN", `${mode}: the beat holds the frame and the line 2 s on (${st?.frame}, "${tk}", speed ${st?.speed})`);
    }
  }

  // Cut 18 §1: peaks hold in wall time — a boss killed three ticks before the stairs (rater Y: "`GOBLIN WARLORD DOWN` went by in about
  // a second": the descend cut the line and faded the frame) keeps the fight frame and its line ≥ 2.5 s of wall time, in both modes;
  // the next floor waits for it. Measured in the page, per animation frame.
  for (const mode of ["fast", "fights"]) {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=${mode}`, { waitUntil: "domcontentloaded" });
    await waitFor((x) => x?.booted && inRun(x) && x.mode === mode, `the ${mode} run for the kill before the stairs`);
    const held = await page.evaluate(() => new Promise((res) => {
      const r = window.__riddle, orig = r.engine.step.bind(r.engine); let done = false;
      r.engine.step = async (n) => {
        const res = await orig(n);
        if (!done && res.snapshot.turn > 30 && !res.run_over && !res.events.some((e) => e.k === "exit" || e.k === "descend")) {
          done = true; const s = res.snapshot, hh = s.hero, t = s.turn;
          res.events.push({ t: t - 8, k: "spawn", e: { id: 95011, kind: "goblin_warlord", x: hh.x + 1, y: hh.y, hp: 1, max_hp: 30, tags: ["boss"] } },
            { t: t - 6, k: "die", id: 95011, cause: "hero" }, { t: t - 3, k: "descend", depth: s.depth + 1, biome: s.biome });
          res.events.sort((a, b) => a.t - b.t);
        }
        return res;
      };
      const t0 = performance.now(); let from = 0, until = 0;
      const poll = () => {
        const w = document.querySelector(".watch"), tk = document.querySelector(".ticker.show")?.textContent ?? "";
        const on = tk === "GOBLIN WARLORD DOWN" && w?.dataset.frame === "fight" && w?.dataset.card !== "1";
        const depth = document.querySelector(".hud .depth")?.textContent;
        if (on && !from) { from = performance.now(); window.__d0 = depth; }
        if (on && depth !== window.__d0) { res({ ms: Math.round(performance.now() - from), after: `the floor changed under it (${window.__d0} → ${depth})`, frame: w?.dataset.frame }); return; }
        if (from && !on) { until = performance.now(); res({ ms: Math.round(until - from), after: tk, frame: w?.dataset.frame }); return; }
        if (performance.now() - t0 > 25_000) { res({ ms: from ? Math.round(performance.now() - from) : 0, after: "timeout", frame: w?.dataset.frame }); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    check(held.ms >= 2500, `${mode}: a boss killed before the stairs holds its frame and line ≥ 2.5 s of wall time (${held.ms} ms, then "${held.after}" · ${held.frame})`);
  }

  // Cut 18 §1: `fast` is never slower than `fights` — the same fake run (seed 157, rater P's) played through in each mode, untouched;
  // `fast` shows the fights `fights` shows (costed ahead of the picture) at 4×, and its dead stretches ramp past 16×
  {
    // (two wall-clock readings: `measured` plays both runs once more on a loaded machine, the bar unchanged — tests/lib/load.mjs)
    const ratio = await measured(async () => {
    const wall = {};
    for (const mode of ["fights", "fast"]) {
      await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=${mode}`, { waitUntil: "domcontentloaded" });
      await waitFor((x) => x?.booted && inRun(x) && x.mode === mode && x.frame, `the ${mode} run for the wall time`);
      const t0 = Date.now();
      const s2 = await waitFor((x) => x && x.screen !== "watch", `the end of the ${mode} run`, 120_000);
      wall[mode] = { ms: Date.now() - t0, screen: s2.screen };
    }
    // Cut 20 §3: `fast` takes ≤ 40 % of `fights`'s wall time over the run (32× dead stretches, 4× fights, a situation's beat 1 s)
    return { ok: wall.fast.ms <= 0.4 * wall.fights.ms, line: `\`fast\` ≤ 0.4 × \`fights\` on one world (fast ${(wall.fast.ms / 1000).toFixed(1)} s · fights ${(wall.fights.ms / 1000).toFixed(1)} s = ${(wall.fast.ms / wall.fights.ms).toFixed(2)}, both ${wall.fast.screen}/${wall.fights.screen})` };
    });
    check(ratio.ok, ratio.line);
  }

  // Cut 16 §4: the boss bar and the break beat. From tick 20 the engine's snapshots carry a warlord beside the hero (in view) for
  // 80 ticks; the first such batch has his spawn at 30/30, a blow taking him to 14, and the core's `warlord breaks` + note.
  for (const mode of ["fights", "fast"]) {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=${mode}`, { waitUntil: "domcontentloaded" });
    await waitFor((x) => x?.booted && inRun(x) && x.mode === mode, `the ${mode} run for the break`);
    await page.evaluate(() => {
      const r = window.__riddle, orig = r.engine.step.bind(r.engine); let from = -1;
      r.engine.step = async (n) => {
        const res = await orig(n); const s = res.snapshot, t = s.turn;
        if (res.run_over || res.events.some((e) => e.k === "exit" || e.k === "descend")) return res;
        if (from < 0 && t > 20) {
          from = t; const hh = s.hero;
          res.events.push({ t: t - 6, k: "spawn", e: { id: 95002, kind: "goblin_warlord", x: hh.x + 1, y: hh.y, hp: 30, max_hp: 30, tags: ["boss"] } },
            { t: t - 4, k: "hurt", id: 95002, dmg: 16, hp: 14, cause: "hero" }, { t: t - 4, k: "callout", text: "warlord breaks" }, { t: t - 4, k: "note", text: "The Warlord breaks." });
          res.events.sort((a, b) => a.t - b.t);
        }
        if (from >= 0 && t < from + 80) {
          const hh = s.hero, x = Math.min(s.w - 1, hh.x + 1), i = hh.y * s.w + x;
          s.entities.push({ id: 95002, kind: "goblin_warlord", x, y: hh.y, hp: 14, max_hp: 30, tags: ["boss"] }); s.visible[i] = true; s.seen[i] = true;
        }
        return res;
      };
    });
    const look = () => page.evaluate(() => { const w = document.querySelector(".watch"), b = document.querySelector(".boss-hp");
      return { screen: window.__riddle.screen, frame: w?.dataset.frame, tk: document.querySelector(".ticker.show")?.textContent ?? "", bar: b && !b.hidden ? { name: b.querySelector(".name")?.textContent ?? "", width: b.querySelector(".fill")?.style.width ?? "", data: w?.dataset.boss ?? "" } : null }; });
    let bar = null, brk = null; const t0 = Date.now();
    while (Date.now() - t0 < 25_000 && !(bar && brk)) {
      const st = await look();
      if (st.screen !== "watch" && st.screen !== "exit") break;
      if (st.bar && !bar) bar = st.bar;
      if (st.tk === "WARLORD BREAKS" && !brk) brk = { frame: st.frame, at: Date.now() };
      await sleep(40);
    }
    check(!!bar && bar.name === "warlord" && /^(47|100)%$/.test(bar.width), `${mode}: the boss bar under the hero's while he is in view (${bar ? `"${bar.name}" ${bar.width} · ${bar.data}` : "never seen"})`);
    check(!!brk && brk.frame === "fight", `${mode}: the break is a beat — the fight frame with "WARLORD BREAKS" (${brk ? `${brk.frame} frame` : "never seen"})`);
    if (brk) {
      await sleep(Math.max(0, 2000 - (Date.now() - brk.at)));
      const st = await look();
      check(st.frame === "fight" && st.tk === "WARLORD BREAKS", `${mode}: the break beat holds the frame and the line 2 s on (${st.frame}, "${st.tk}")`);
    }
  }

  // Cut 15 §4: name tags never overlap — two hostiles with 12-letter names on adjacent tiles of one row in the fight frame (their
  // tags would share a row and intersect) draw on two rows, their boxes apart
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  for (let tries = 0; tries < 6; tries++) {
    s = await waitFor((x) => x?.booted && inRun(x) && x.frame === "fight", "a fight frame for the tags", 30_000);
    await press("⏸"); await sleep(150);
    if ((await state())?.frame === "fight") break;
    await press("▶"); await sleep(300);
  }
  const tags = await page.evaluate(async () => {
    const v = window.__viewer, hero = v.debugPos().find((e) => e.hero), t = v.tick();
    const mk = (id, x, name) => ({ t: t - 5, k: "spawn", e: { id, kind: "goblin", name, x, y: hero.y, hp: 5, max_hp: 5, tags: [] } });
    v.apply([mk(90011, hero.x - 1, "Captain Tain"), mk(90012, hero.x - 2, "Ashar Monkey")]); v.seek(t);
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    return { frame: document.querySelector(".watch").dataset.frame, labels: v.debugLabels() };
  });
  const [ta, tb2] = tags.labels.filter((l) => l.id >= 90011);
  const boxes = (l) => [l.x - l.w / 2, l.y - l.h, l.x + l.w / 2, l.y];
  const inter = (a, b) => a[0] < b[2] && a[2] > b[0] && a[1] < b[3] && a[3] > b[1];
  const hOverlap = ta && tb2 && Math.abs(ta.x - tb2.x) < (ta.w + tb2.w) / 2;
  check(tags.frame === "fight" && !!ta && !!tb2 && hOverlap && Math.abs(ta.y - tb2.y) >= ta.h, `two tags on intersecting spans draw on two rows (${[ta, tb2].filter(Boolean).map((l) => `${l.text} x${Math.round(l.x)} y${Math.round(l.y)}`).join(" · ")})`);
  const all = tags.labels.map(boxes);
  check(all.every((a, i) => all.every((b, j) => i === j || !inter(a, b))), `no two tags drawn this frame intersect (${tags.labels.length} tags)`);

  // Cut 18 §2: the hero is never covered — a boss beside him and a foe on his own tile in the fight frame: he draws in front of every
  // sprite (`debugRects` z), and no sprite's rect covers more than 30 % of his (≥ 70 % of him unoccluded, whatever draws behind)
  // (the rects are read two frames after the seek, the sprites still easing to their tiles on a loaded machine: `measured` takes
  // the scene once more then — tests/lib/load.mjs)
  const cover = await measured(async () => {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  for (let tries = 0; tries < 6; tries++) {
    s = await waitFor((x) => x?.booted && inRun(x) && x.frame === "fight", "a fight frame for the hero's rect", 30_000);
    await press("⏸"); await sleep(150);
    if ((await state())?.frame === "fight") break;
    await press("▶"); await sleep(300);
  }
  const occl = await page.evaluate(async () => {
    const v = window.__viewer, hero = v.debugPos().find((e) => e.hero), t = v.tick();
    const mk = (id, kind, x, tags = []) => ({ t: t - 5, k: "spawn", e: { id, kind, x, y: hero.y, hp: 30, max_hp: 30, tags } });
    v.apply([mk(90021, "goblin_warlord", hero.x + 1, ["boss"]), mk(90022, "goblin", hero.x), mk(90023, "ogre", hero.x - 1)]); v.seek(t);
    await new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
    return { frame: document.querySelector(".watch").dataset.frame, rects: v.debugRects() };
  });
  {
    const hr = occl.rects.find((r) => r.hero), others = occl.rects.filter((r) => !r.hero);
    const inter = (a, b) => Math.max(0, Math.min(a.x + a.w, b.x + b.w) - Math.max(a.x, b.x)) * Math.max(0, Math.min(a.y + a.h, b.y + b.h) - Math.max(a.y, b.y));
    const area = hr ? hr.w * hr.h : 1;
    const covers = others.map((r) => ({ kind: r.kind, share: inter(hr, r) / area, front: r.z >= hr.z }));
    // the union of what could hide him (a grid of his rect's points under any sprite drawn in front) and of everything drawn at all
    const grid = (pred) => { let n = 0, hit = 0; for (let i = 0; i < 20; i++) for (let j = 0; j < 20; j++) { const x = hr.x + ((i + 0.5) / 20) * hr.w, y = hr.y + ((j + 0.5) / 20) * hr.h; n++; if (others.some((r) => pred(r) && x >= r.x && x < r.x + r.w && y >= r.y && y < r.y + r.h)) hit++; } return hit / n; };
    const hidden = hr ? grid((r) => r.z >= hr.z) : 1;
    const front = { ok: occl.frame === "fight" && !!hr && others.some((r) => r.kind === "goblin_warlord") && covers.every((c) => !c.front), line: `the hero draws in front of every sprite beside him (${covers.map((c) => `${c.kind} ${c.front ? "front" : "behind"}`).join(" · ")})` };
    const clear = { ok: !!hr && 1 - hidden >= 0.7 && covers.every((c) => c.share <= 0.31), line: `with a boss adjacent the hero's rect is ≥ 70 % unoccluded (${Math.round((1 - hidden) * 100)} %; each sprite covers ${covers.map((c) => `${c.kind} ${Math.round(c.share * 100)} %`).join(" · ")})` };
    return { ok: front.ok && clear.ok, line: `${front.line} · ${clear.line}`, front, clear };
  }
  });
  const coverRetried = cover.line.includes(" [retried") ? cover.line.slice(cover.line.indexOf(" [retried")) : "";
  check(cover.front.ok, cover.front.line);
  check(cover.clear.ok, cover.clear.line + coverRetried);

  // Cut 18 §2: one line of callout over a fight — a row and a telegraph on one tick: the renderer draws the telegraph alone over the
  // hero (`debugText`), the row reads on the ticker (`rule`, visible in the fight frame). A goblin stands beside the hero in the
  // engine's snapshots from tick 20 (the fight frame), its `attack` and the two callouts on one tick.
  // (read off frames as they come: `measured` takes the scene once more on a loaded machine)
  const callout = await measured(async () => {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fights`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && inRun(x) && x.mode === "fights", "the fights run for the callout line");
  const lines = await page.evaluate(() => new Promise((res) => {
    const r = window.__riddle, orig = r.engine.step.bind(r.engine); let from = -1;
    r.engine.step = async (n) => {
      const out = await orig(n); const sn = out.snapshot, t = sn.turn;
      if (out.run_over || out.events.some((e) => e.k === "exit" || e.k === "descend")) return out;
      if (from < 0 && t > 20) {
        from = t; const hh = sn.hero;
        out.events.push({ t: t - 8, k: "spawn", e: { id: 95031, kind: "goblin_archer", x: hh.x + 1, y: hh.y, hp: 9, max_hp: 9, tags: ["ranged"] } },
          { t: t - 7, k: "attack", src: 95031, dst: hh.id, dmg: 5, hit: true }, { t: t - 7, k: "hurt", id: hh.id, dmg: 5, hp: Math.max(1, hh.hp - 5), cause: "goblin_archer" },
          { t: t - 3, k: "rule", row: 0, verb: { v: "attack", a: "nearest" }, text: "R1 foes>=1 → attack nearest" }, { t: t - 3, k: "callout", text: "archer draws" });
        out.events.sort((a, b) => a.t - b.t);
      }
      if (from >= 0 && t < from + 60) { const hh = sn.hero, x = Math.min(sn.w - 1, hh.x + 1), i = hh.y * sn.w + x; sn.entities.push({ id: 95031, kind: "goblin_archer", x, y: hh.y, hp: 9, max_hp: 9, tags: ["ranged"] }); sn.visible[i] = true; sn.seen[i] = true; }
      return out;
    };
    const t0 = performance.now(); let maxLines = 0, both = null, rule = null;
    const poll = () => {
      const w = document.querySelector(".watch"), v = window.__viewer, tk = document.querySelector(".ticker.show");
      if (w?.dataset.frame === "fight" && v?.debugText) {
        const tx = v.debugText(); maxLines = Math.max(maxLines, tx.length);
        if (tx.some((x) => /archer draws/i.test(x.text))) both ??= tx.map((x) => `${x.kind}:${x.text}`);
        // the row's line once the injected fight's telegraph is up (the fake's own fights before it fire their own rows)
        if (both && tk && tk.classList.contains("rule") && getComputedStyle(tk).opacity === "1") rule ??= tk.textContent;
      }
      if ((both && rule) || performance.now() - t0 > 20_000) { res({ maxLines, both, rule }); return; }
      requestAnimationFrame(poll);
    };
    poll();
  }));
  const one = { ok: !!lines.both && lines.both.length === 1 && lines.maxLines <= 1, line: `a row and a telegraph on one tick draw one line over the fight (${lines.both ? lines.both.join(" · ") : "telegraph never drawn"}; at most ${lines.maxLines} a frame)` };
  const row = { ok: lines.rule === "attack nearest", line: `the row reads on the ticker meanwhile ("${lines.rule ?? "never"}")` };
  return { ok: one.ok && row.ok, line: `${one.line} · ${row.line}`, one, row };
  });
  const calloutRetried = callout.line.includes(" [retried") ? callout.line.slice(callout.line.indexOf(" [retried")) : "";
  check(callout.one.ok, callout.one.line + calloutRetried);
  check(callout.row.ok, callout.row.line);
  // Cut 24 §1 (AL: a Warlord fight > 4 min at 1×, the boss bar full; AK: ~100 s of retreat ↔ pack break): the watch never shows more
  // than 15 s of wall time without an hp change, a kill, a pickup or a descent — a stretch that cannot progress (`fake_shrug=100`: the
  // run's first 400 ticks beside a foe shrug every blow, the guard asleep) plays at the travel rate, the fight frame's included; in
  // `fights` on its first floors (no card: every fight watched) and in `fast`
  // (wall-clock readings: on a loaded machine `measured` takes the scene once more, the bar unchanged — tests/lib/load.mjs)
  for (const m of ["fights", "fast"]) {
    const dead = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&autosend=1&fake_shrug=100`, { waitUntil: "domcontentloaded" });
    await waitFor((x) => x?.booted && inRun(x) && x.mode, `the watch (${m}, shrug)`);
    if (m === "fast") await press("fast");
    const d = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(); let cur = null, since = 0, sinceTick = 0, maxMs = 0, maxTicks = 0, deadFight = 0, fastest = 0, at = "";
      const poll = () => {
        const w = document.querySelector(".watch"), now = performance.now();
        const live = window.__riddle?.screen === "watch" && w && w.dataset.ending !== "1" && !document.querySelector(".sheet-wrap");
        if (live && w.dataset.progress !== undefined && w.dataset.tick !== undefined) {
          const p = w.dataset.progress, tick = Number(w.dataset.tick);
          if (p !== cur) { cur = p; since = now; sinceTick = tick; }
          else { const ms = now - since; if (ms > maxMs) { maxMs = ms; at = `D${document.querySelector(".hud .depth")?.textContent ?? "?"} t${sinceTick}`; } maxTicks = Math.max(maxTicks, tick - sinceTick); }
          if (w.dataset.dead === "1" && w.dataset.frame === "fight") { deadFight++; fastest = Math.max(fastest, Number(w.dataset.speed)); }
        } else cur = null;
        if (now - t0 > 60_000 || (window.__riddle?.screen !== "watch" && now - t0 > 3000)) { res({ maxMs: Math.round(maxMs), maxTicks, deadFight, fastest, at }); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    return { ok: d.deadFight > 0 && d.maxTicks >= 300 && d.maxMs <= 15_000, line: `${m}: a fight that cannot progress plays as travel — the longest stretch without a move ${(d.maxMs / 1000).toFixed(1)} s wall over ${d.maxTicks} ticks (≤ 15 s; ≥ 300 ticks is 15 s at the fight's 2×), the fight frame dead at up to ${d.fastest}× (${d.at})` };
    });
    check(dead.ok, dead.line);
  }
  // Cut 28 §3 (AU: at 1× ~50 s of `pick up ×N` and 9–12 s gaps with nothing new): at the plain 1× no stretch passes 5 s of wall time
  // without a fight, a beat, a pickup of note or a descent (`data-progress`: a coin or a repeat pickup is no move; a held beat is news) —
  // the rest plays at the travel rate; a run read up to 60 s
  {
    const one = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&autosend=1`, { waitUntil: "domcontentloaded" });
    await waitFor((x) => x?.booted && inRun(x) && x.mode, "the watch (1×)");
    await press("normal");
    const d = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(); let cur = null, since = 0, sinceTick = 0, maxMs = 0, maxTicks = 0, dead = 0, at = "", depth = "";
      const poll = () => {
        const w = document.querySelector(".watch"), now = performance.now();
        const live = window.__riddle?.screen === "watch" && w && w.dataset.mode === "one" && w.dataset.ending !== "1" && w.dataset.speed !== "0" && !document.querySelector(".sheet-wrap");
        if (live && w.dataset.progress !== undefined && w.dataset.tick !== undefined) {
          depth = document.querySelector(".hud .depth")?.textContent ?? depth;
          const key = `${w.dataset.progress}|${depth}|${w.dataset.hold ?? ""}`, tick = Number(w.dataset.tick);
          if (key !== cur || w.dataset.hold) { cur = key; since = now; sinceTick = tick; }
          else { const ms = now - since; if (ms > maxMs) { maxMs = ms; at = `${depth} t${sinceTick}`; } maxTicks = Math.max(maxTicks, tick - sinceTick); }
          if (w.dataset.dead === "1") dead++;
        } else { cur = null; }
        if (now - t0 > 60_000 || (window.__riddle?.screen !== "watch" && now - t0 > 3000)) { res({ maxMs: Math.round(maxMs), maxTicks, dead, at, wall: Math.round(now - t0) }); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    return { ok: d.maxMs <= 5000 && d.dead > 0, line: `1×: no stretch > 5 s without a fight, a beat, a pickup of note or a descent — the longest ${(d.maxMs / 1000).toFixed(1)} s over ${d.maxTicks} ticks (${d.at}; ${d.dead} dead frames at the travel rate; ${(d.wall / 1000).toFixed(0)} s read)` };
    });
    check(one.ok, one.line);
  }
  // Cut 27 §1: solved floors fold — a send whose forecast clears D1 ≥ 95 % (`fake_fold`, the fake's stand-in for the core's `fold_to`) opens
  // on the fold line and lands below it; the screen time on the folded floors is ≤ 5 s a floor, in `fights` and `fast` alike
  for (const m of ["fights", "fast"]) {
    const fold = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=26&fake_fold=2`, { waitUntil: "domcontentloaded" });
    await waitFor((x) => x?.booted && x.screen === "camp", "the camp (fold)");
    await page.evaluate((m) => { window.__riddle.watchMode = m; }, m);
    await page.waitForFunction(() => window.__riddle.lastForecast?.fold_to !== undefined, null, { timeout: 15_000 }).catch(() => {});
    const t0 = Date.now();
    await page.evaluate(() => document.querySelector("button.gem.send")?.click());
    const s = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(), poll = () => {
        const w = document.querySelector(".watch"), log = (window.__foldLog ?? [])[0];
        if (log && w?.dataset.fold === "0") { res({ log, depth: Number(document.querySelector(".watch .depth")?.textContent?.slice(1)), ending: w.dataset.ending === "1" }); return; }
        if (performance.now() - t0 > 20_000) { res(null); return; }
        requestAnimationFrame(poll);
      };
      poll();
    }));
    const ms = Date.now() - t0, floors = s?.log.floors ?? 1;
    return { ok: !!s && s.log.perFloor <= 5000 && (s.depth > s.log.to || s.ending), line: `${m}: folded floors D${s?.log.from}–${s?.log.to} take ${s?.log.perFloor} ms of screen a floor (≤ 5 s; ${Math.round(ms / floors)} ms a floor from the send tap), the watch opens on D${s?.depth}${s?.ending ? " (the run ended inside the fold)" : ""}` };
    });
    check(fold.ok, fold.line);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`fights: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`fights: ok (${out.length} checks)`);
