#!/usr/bin/env node
// Blind 77030eb (B): at Normal the watch stood ~2.5 min at the start of D7 with no new line (the badge flicking 1×/4×), and an
// iron-golem stalemate ran ~3 min with the hero's hp see-sawing. Gates, on the fake engine's shrug fight (`fake_shrug`: blows hit for
// 0 both ways; `fake_god`; `fake_depth=5`) with each engine step slowed 15 ms and every batch carrying noise the player cannot see — a
// blow on a foe out of sight, and the hero hurt again and again to the same hp (a see-saw, no new low):
//   · at Normal (`one`) no stretch of more than 10 s with no news (a move, a jump, a floor);
//   · the noise never counts as a move: the stretch is jumped (`data-jumps` ≥ 1) and every jump says `skipped ahead`.
//   · blind b58b431: the same with a boss in view whose hp see-saws (the Bloat Mother healing in her gas) — no move either.
//   node web/tests/watch-normal-news.mjs
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
try {
  // blind b58b431 (A ~170 s, B ~90 s at D13 toggling 1×/4×): the Bloat Mother heals in her gas — a boss in view whose hp see-saws
  // (hit, healed back, hit again to the same hp) is a stalemate too, never news
  for (const boss of [false, true]) {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&early=0&fake_god=1&fake_depth=5&fake_shrug=3000`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.screen === "watch" && document.querySelector(".watch")?.dataset.tick, null, { timeout: 30_000 });
  await page.locator("[data-tile=speed]").click(); await page.locator(".sheet [data-tile=one]").click();
  await page.waitForFunction(() => document.querySelector(".watch")?.dataset.mode === "one", null, { timeout: 5000 });
  const r = await page.evaluate(async (boss) => {
    const app = window.__riddle, step = app.engine.step.bind(app.engine);
    app.engine.step = async (n) => {
      await new Promise((z) => setTimeout(z, 15));
      const res = await step(n);
      const t = res.snapshot?.turn ?? 0, hero = res.snapshot?.hero;
      if (hero && !res.run_over) res.events = [...(res.events ?? []),
        { t, k: "hurt", id: 987654, dmg: 2, hp: 50 - (t % 7), cause: "hero" },   // a foe never in sight
        { t, k: "hurt", id: hero.id, dmg: 1, hp: Math.max(1, hero.max_hp - 3), cause: "rat" }];   // the hero's see-saw: the same hp
      if (boss && hero && !res.run_over) {
        const s = res.snapshot, x = hero.x, y = hero.y;
        // a boss beside him, in sight; each batch a blow takes him to the same hp (he heals back between)
        s.entities = [...s.entities.filter((e) => e.id !== 424242), { id: 424242, kind: "bloat_mother", x, y, hp: 40, max_hp: 60, tags: ["boss"] }];
        if (s.visible) s.visible[y * s.w + x] = true;
        res.events.push({ t, k: "hurt", id: 424242, dmg: 3, hp: 40, cause: "hero" });
      }
      return res;
    };
    const t0 = performance.now(); let news = "", newsAt = t0, worst = 0, worstAt = null;
    while (performance.now() - t0 < 60_000) {
      const w = document.querySelector(".watch"); if (!w || app.screen !== "watch") break;
      const d = w.dataset, now = performance.now();
      // (the shrug fight has no real move: news is a jump, a floor, the ending — never the injected noise)
      const key = `${d.jumps ?? 0}|${document.querySelector(".hud .depth")?.textContent}|${d.ending ?? ""}`;
      if (key !== news || d.over === "1") { news = key; newsAt = now; }
      if (now - newsAt > worst) { worst = now - newsAt; worstAt = { tick: d.tick, frontier: d.frontier, speed: d.speed, dead: d.dead }; }
      if (d.over === "1") break;
      await new Promise((z) => setTimeout(z, 100));
    }
    const d = document.querySelector(".watch")?.dataset ?? {};
    return { worst: Math.round(worst), worstAt, jumps: Number(d.jumps ?? 0), said: Number(d.jumpsSaid ?? 0), wall: Math.round((performance.now() - t0) / 100) / 10 };
  }, boss);
  const tag = boss ? "a boss see-saw" : "unseen blows";
  check(r.worst <= 10_000, `Normal (${tag}): no stretch over 10 s without news (worst ${r.worst} ms at ${JSON.stringify(r.worstAt)}; ${r.wall} s watched)`);
  check(r.jumps >= 1, `Normal (${tag}): unseen blows and a see-saw are no moves — the stalemate is jumped (${r.jumps} jumps)`);
  check(r.said >= 1, `Normal (${tag}): a jump says "skipped ahead" (${r.said} said)`);
  await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`watch-normal-news: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`watch-normal-news: ok (${out.length} checks)`);
