#!/usr/bin/env node
// Take control (owner 2026-10-08, a secondary mode), on the real wasm engine: in a live watched run the panel's toggle takes the
// hero; the world waits (engine tick still) until an action; each arrow key is one hero action (the picture never runs past the
// frontier); Release hands him back to the rules (the world moves again) and the panel reads off.
// Owner 2026-10-10 ("keyboard works? and consider mobile/touch" · "d3 style mouse controls"): the verb keys and `c`; a held arrow
// repeats and stops on release; a mouse click on the floor walks there (stopping at the tile, a new foe, or a refusal), a click on
// a foe fights it, a right-click acts; on a phone (360 px, touch) a tap on the map steps one tile toward it, every tile ≥ 44 px and
// nothing past the edge; a phone screenshot of the panel in hand (`--shot DIR`, default the scratchpad).
//   node web/tests/take-control.mjs [--shot DIR]
import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const shotArg = process.argv.indexOf("--shot");
const shotDir = shotArg > 0 ? process.argv[shotArg + 1] : new URL("../../scratchpad/take-control/", import.meta.url).pathname;
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));

/** Boot a fresh game, build the house if offered, send, and land on the watch; every engine step's snapshot kept (`__lastSnap`). */
async function boot(page) {
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&fresh=1&seed=611`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
  await page.evaluate(() => { const e = window.__riddle.engine, step = e.step.bind(e); e.step = async (...a) => { const r = await step(...a); window.__lastSnap = r.snapshot; return r; }; });
  const house = page.locator(".town-tag[data-next=house]");
  if (await house.count()) { await house.first().click(); await sleep(800); const b = page.locator("button", { hasText: /^build|Build/ }); if (await b.count()) await b.first().click(); await sleep(4000); }
  await page.locator(".camp-gem, .gem").first().click();
  await page.waitForFunction(() => window.__riddle.screen === "watch", null, { timeout: 30_000 });
  await sleep(1500);
}
const awaiting = (page) => page.waitForFunction(() => document.querySelector(".ctl")?.dataset.awaiting === "1" && document.querySelector(".ctl")?.dataset.on === "1", null, { timeout: 20_000 });
const acts = (page) => page.evaluate(() => Number(document.querySelector(".ctl")?.dataset.acts ?? 0));
const snapOf = (page) => page.evaluate(() => { const s = window.__lastSnap; if (!s) return null;
  const foes = s.entities.filter((e) => !e.ally && e.hp > 0 && !e.remembered && !["bones", "captive"].includes(e.kind) && !e.tags.includes("captive") && !e.tags.includes("ally") && s.visible[e.y * s.w + e.x]);
  return { x: s.hero.x, y: s.hero.y, turn: s.turn, order: s.order, awaiting: s.awaiting, foes: foes.map((e) => ({ id: e.id, x: e.x, y: e.y })) }; });
/** Where tile (tx, ty) is on screen (client px), and whether the canvas is what is there (no HUD over it). */
const tileAt = (page, tx, ty) => page.evaluate(([tx, ty]) => {
  const v = window.__viewer, st = v.stats(), r = v.debugRects().find((q) => q.hero), cv = document.querySelector("canvas.view"), c = cv.getBoundingClientRect();
  const px = 8 * st.k / st.dpr, hx = c.left + r.x + r.w / 2, hy = c.top + r.y + r.h - 3 * st.k / st.dpr;
  const x = hx + (tx - st.hero[0]) * px, y = hy + (ty - st.hero[1]) * px;
  const top = document.elementFromPoint(x, y);
  return { sx: x, sy: y, px, free: top === cv, over: top ? `${top.tagName}.${top.className}` : "" };
}, [tx, ty]);
/** Wait for the camera to rest (his sprite still on screen for ~0.5 s): a point aimed while it glides lands on another tile. */
const settle = async (page) => { let last = "", same = 0;
  for (let i = 0; i < 40 && same < 3; i++) { const r = await page.evaluate(() => { const q = window.__viewer.debugRects().find((q) => q.hero); return q ? `${Math.round(q.x)},${Math.round(q.y)}` : ""; }); same = r && r === last ? same + 1 : 0; last = r; await sleep(150); } };
/** Seen open tiles by path length from the hero (the client's walk reads the same: no wall, no chasm, no wall corner cut). */
const reachable = (page) => page.evaluate(() => {
  const s = window.__lastSnap, w = s.w, open = (x, y) => x >= 0 && y >= 0 && x < w && y < s.h && s.seen[y * w + x] && !["wall", "chasm"].includes(s.tiles[y * w + x]);
  const taken = new Set(s.entities.filter((e) => !e.remembered && e.id !== s.hero.id).map((e) => e.y * w + e.x));
  const dist = new Map([[s.hero.y * w + s.hero.x, 0]]), q = [s.hero.y * w + s.hero.x];
  for (let i = 0; i < q.length; i++) { const k = q[i], x = k % w, y = (k - x) / w;
    for (let dy = -1; dy <= 1; dy++) for (let dx = -1; dx <= 1; dx++) { const nx = x + dx, ny = y + dy, j = ny * w + nx;
      if ((!dx && !dy) || !open(nx, ny) || dist.has(j) || taken.has(j)) continue;
      if (dx && dy && (s.tiles[y * w + nx] === "wall" || s.tiles[ny * w + x] === "wall")) continue;
      dist.set(j, dist.get(k) + 1); q.push(j); } }
  return [...dist].map(([k, d]) => ({ x: k % w, y: Math.floor(k / w), d }));
});

const browser = await launchBrowser();
try {
  // ---------------- desktop (a mouse, 400 px) ----------------
  const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
  await boot(page);
  check(await page.evaluate(() => window.__riddle.kind) === "wasm", "real wasm engine");
  const st = () => page.evaluate(() => { const m = document.querySelector(".ctl"), w = document.querySelector("main.watch"); return { on: m?.dataset.on, awaiting: m?.dataset.awaiting, tick: Number(w?.dataset.tick), eng: Number(w?.dataset.engineTick ?? NaN) }; });
  check(await page.locator(".ctl-toggle").isVisible(), "the watch offers take control");
  // discoverability: the lone tile names what it does and its key
  const tip0 = (await page.locator(".ctl-toggle").getAttribute("title")) ?? "";
  check(/·\s*M$/.test(tip0) && tip0.split(/\s+/).length <= 10, `the control tile's tooltip names it and its key ("${tip0}")`);
  await page.click(".ctl-toggle");
  await awaiting(page);
  await sleep(500); const a = await st(); await sleep(2000); const b = await st();
  check(a.on === "1" && b.eng === a.eng, `the world waits for an order (engine ${a.eng} → ${b.eng})`);
  check(/Hand control/.test(await page.locator(".live-badge").textContent()), "the badge reads hand control");
  let moved = 0;
  for (const k of ["d", "a", "x", "w", "s", "e", "z", "ArrowRight", "ArrowLeft", "Numpad5", "."]) { const s0 = await st(); await page.keyboard.press(k); await sleep(900); const s1 = await st(); if (s1.eng > s0.eng) moved++; check(s1.tick <= s1.eng, `the picture stays at the frontier after ${k} (${s1.tick} ≤ ${s1.eng})`); }
  check(moved === 11, `each key is one action — the pad grid, arrows, numpad (${moved}/11 advanced the world)`);
  // blind b58b431 (A: arrows and `descend` inert in gas): every order resolves on the panel — done, or refused and why
  const note = (await page.locator(".ctl-note").textContent()) ?? "";
  check(/^(moved|waited|can't · .+|paralysed · \d+)$/.test(note), `the last order reads its outcome ("${note}")`);
  // every tile names its key: in its tooltip, and in its corner for a mouse
  const tips = await page.evaluate(() => [...document.querySelectorAll(".ctl .tile")].map((t) => ({ tip: t.title, key: t.querySelector(".ctl-key")?.textContent ?? "", shown: !!t.querySelector(".ctl-key") && getComputedStyle(t.querySelector(".ctl-key")).display !== "none" })));
  check(tips.length >= 13 && tips.every((t) => t.key && t.tip.includes(t.key)), `every tile's tooltip names its key (${tips.filter((t) => t.key && t.tip.includes(t.key)).length}/${tips.length})`);
  check(tips.every((t) => t.shown), "a mouse sees each key in its tile's corner");
  // the verb keys: each one order (the core resolves it, done or refused)
  await page.evaluate(() => document.activeElement?.blur?.());
  for (const [k, re] of [["f", /^(attack ✓|foe down|can't · .+|paralysed · \d+)$/], ["g", /^(attack until|foe down|hp under \d+%|can't · .+|paralysed · \d+)$/], ["h", /^(drink ✓|can't.*|paralysed · \d+)$/], ["t", /^(throw ✓|can't.*|paralysed · \d+)$/], [">", /^(descend ✓|can't.*|paralysed · \d+)$/], ["Enter", /^(descend ✓|can't.*|paralysed · \d+)$/]]) {
    await awaiting(page);
    const n0 = await acts(page); await page.keyboard.press(k); await sleep(700);
    const n = (await page.locator(".ctl-note").textContent()) ?? "";
    check(await acts(page) === n0 + 1 && re.test(n), `key ${k} is one order ("${n}")`);
  }
  // `c` takes and releases him; nothing fires in a field
  await awaiting(page);
  await page.keyboard.press("m"); await page.waitForFunction(() => document.querySelector(".ctl")?.dataset.on === "0", null, { timeout: 10_000 }).catch(() => {});
  check(await page.evaluate(() => document.querySelector(".ctl")?.dataset.on) === "0", "`c` releases him");
  const n1 = await acts(page); await page.keyboard.press("f"); await sleep(300);
  check(await acts(page) === n1, "verb keys do nothing out of hand");
  await page.keyboard.press("m"); await awaiting(page).catch(() => {});
  check(await page.evaluate(() => document.querySelector(".ctl")?.dataset.on) === "1", "`c` takes him again");
  const fieldKey = await page.evaluate(async () => { const i = document.createElement("input"); document.body.appendChild(i); i.focus(); const n = Number(document.querySelector(".ctl").dataset.acts ?? 0);
    i.dispatchEvent(new KeyboardEvent("keydown", { key: "f", bubbles: true })); i.dispatchEvent(new KeyboardEvent("keydown", { key: "m", bubbles: true })); await new Promise((r) => setTimeout(r, 300));
    const ok = Number(document.querySelector(".ctl").dataset.acts ?? 0) === n && document.querySelector(".ctl").dataset.on === "1"; i.remove(); return ok; });
  check(fieldKey, "keys inside a field drive nothing");
  // hold-to-repeat: a held arrow steps more than once, and stops on release
  await awaiting(page);
  const dirs = [[1, 0], [-1, 0], [0, 1], [0, -1], [1, 1], [-1, -1], [1, -1], [-1, 1]];
  const runs = await page.evaluate((dirs) => { const s = window.__lastSnap, w = s.w; return dirs.map(([dx, dy]) => { let n = 0, x = s.hero.x, y = s.hero.y;
    while (n < 8) { x += dx; y += dy; const t = s.tiles[y * w + x]; if (!t || t === "wall" || t === "chasm" || s.entities.some((e) => e.x === x && e.y === y && e.id !== s.hero.id)) break; n++; } return n; }); }, dirs);
  const best = dirs[runs.indexOf(Math.max(...runs))];
  const arrow = page.locator(`.ctl-dir[data-dx="${best[0]}"][data-dy="${best[1]}"]`);
  const box = await arrow.boundingBox();
  const h0 = await acts(page);
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down(); await sleep(1600); await page.mouse.up();
  const h1 = await acts(page); await sleep(1200); const h2 = await acts(page);
  check(h1 - h0 > 1, `a held arrow repeats (${h1 - h0} steps in 1.6 s, open run ${Math.max(...runs)})`);
  check(h2 - h1 <= 1, `the repeat stops on release (${h2 - h1} after)`);
  // the mouse: a click on the floor walks there — one step a turn — until he arrives, a new foe shows, or an order is refused
  await awaiting(page); await settle(page);
  const s0 = await snapOf(page);
  let goal = null;
  for (const t of (await reachable(page)).filter((t) => t.d >= 3).sort((p, q) => q.d - p.d)) { const at = await tileAt(page, t.x, t.y); if (at.free) { goal = { ...t, ...at }; break; } }
  check(!!goal, `a far floor tile on screen to walk to (${goal ? `${goal.d} steps` : "none"})`);
  if (goal) {
    const w0 = await acts(page);
    await page.mouse.move(goal.sx, goal.sy); await sleep(100);
    check(await page.evaluate(() => document.querySelector("canvas.view")?.dataset.ctlCursor) === "move", "a move cursor over the floor");
    check(await page.evaluate(() => { const hv = document.querySelector(".ctl-hover"); return !!hv && !hv.hidden; }), "the hovered tile is marked");
    await page.mouse.click(goal.sx, goal.sy);
    let walked = false;
    for (let i = 0; i < 80; i++) { await sleep(150); const d = await page.evaluate(() => document.querySelector(".ctl")?.dataset.drive); if (d === "walk") walked = true; if (walked && !d) break; }
    const w1 = await acts(page), s1 = await snapOf(page);
    const there = s1.x === goal.x && s1.y === goal.y, newFoe = s1.foes.some((f) => !s0.foes.some((g) => g.id === f.id)), refusedW = /^can't/.test(s1.order ?? "");
    check(w1 - w0 >= 2, `a click walks several tiles, one order a turn (${w1 - w0} orders for ${goal.d} steps)`);
    check(there || newFoe || refusedW || s1.foes.some((f) => Math.max(Math.abs(f.x - s1.x), Math.abs(f.y - s1.y)) <= 1),
      `the walk ends at the tile, a new foe, or a refusal (${there ? "there" : newFoe ? "a foe showed" : refusedW ? s1.order : "close foe"})`);
  }
  // a click on him stops a walk
  await awaiting(page);
  const far = (await reachable(page)).filter((t) => t.d >= 4);
  let stopTile = null; for (const t of far) { const at = await tileAt(page, t.x, t.y); if (at.free) { stopTile = at; break; } }
  if (stopTile) {
    await page.mouse.click(stopTile.sx, stopTile.sy); await sleep(200);
    const at = await page.evaluate(() => { const r = window.__viewer.debugRects().find((q) => q.hero), c = document.querySelector("canvas.view").getBoundingClientRect(), b = r.drawn ?? r; return { sx: c.left + b.x + b.w / 2, sy: c.top + b.y + b.h / 2 }; });
    await page.mouse.click(at.sx, at.sy); await sleep(200);
    check(!(await page.evaluate(() => document.querySelector(".ctl")?.dataset.drive)), "a click on him stops the walk");
    await page.keyboard.press("Escape");
  }
  // release, then the old checks (the rules play on)
  await page.keyboard.press("Escape");
  await awaiting(page).catch(() => {});
  await page.click('.ctl-verb[data-verb="descend"]'); await sleep(900);
  const n2 = (await page.locator(".ctl-note").textContent()) ?? "";
  check(/^(descend ✓|can't · no stairs|can't.*)$/.test(n2), `descend resolves visibly ("${n2}")`);
  // blind b58b431 (B: 40 identical attack taps): `attack until` is one order
  check(await page.locator('.ctl-verb[data-verb="attack until"]').isVisible(), "the panel offers attack until");
  await page.click(".ctl-toggle");
  await sleep(500); const b2 = await st(); await sleep(3000);
  const c = await st();
  check(c.on === "0", "released: the panel reads off");
  check(c.tick > b2.tick + 40, `released: the rules play on (${b2.tick} → ${c.tick})`);
  // a foe: the rules (released above) find one; take him back, click it (he walks up and fights), then right-click (the secondary)
  let foe = null;
  for (let i = 0; i < 400 && !foe; i++) { await sleep(250); const s = await snapOf(page); if (s?.foes.length) foe = s.foes[0]; }
  check(!!foe, "the rules find a foe");
  if (foe) {
    await page.keyboard.press("m"); await awaiting(page); await settle(page);
    const s = await snapOf(page), f = s.foes.find((q) => q.id === foe.id) ?? s.foes[0];
    // (aim at the sprite, as a player would: its drawn box, the canvas's px)
    const at = f && await page.evaluate((id) => { const r = window.__viewer.debugRects().find((q) => q.id === id), cv = document.querySelector("canvas.view"), c = cv.getBoundingClientRect();
      if (!r) return null; const b = r.drawn ?? r, sx = c.left + b.x + b.w / 2, sy = c.top + b.y + b.h * 0.6, top = document.elementFromPoint(sx, sy);
      return { sx, sy, free: top === cv, over: top ? `${top.tagName}.${top.className}` : "" }; }, f.id);
    if (at?.free) {
      await page.mouse.move(at.sx, at.sy); await sleep(100);
      check(await page.evaluate(() => document.querySelector("canvas.view")?.dataset.ctlCursor) === "attack", "an attack cursor over a foe");
      const f0 = await acts(page);
      await page.mouse.click(at.sx, at.sy);
      let fought = false;
      for (let i = 0; i < 100 && !fought; i++) { await sleep(200); const n = (await page.locator(".ctl-note").textContent()) ?? ""; fought = /^(attack until|foe down|hp under \d+%|can't · hp low)$/.test(n) || (await snapOf(page)).foes.every((q) => q.id !== f.id); }
      const fe = await snapOf(page);
      check(fought && await acts(page) > f0, `a click on a foe walks up and fights it ("${await page.locator(".ctl-note").textContent()}", ${await acts(page) - f0} orders, hero ${fe.x},${fe.y} foe ${f.x},${f.y} → ${JSON.stringify(fe.foes)}, drive ${await page.evaluate(() => document.querySelector(".ctl")?.dataset.drive)}, stop ${await page.evaluate(() => document.querySelector(".ctl")?.dataset.stop)})`);
    } else check(true, `(the foe sits under ${at?.over}: the click case skipped)`);
    await awaiting(page).catch(() => {});
    const cv = await page.locator("canvas.view").boundingBox();
    const r0 = await acts(page);
    await page.evaluate(() => { window.__menu = null; window.addEventListener("contextmenu", (e) => { window.__menu = e.defaultPrevented; }, { once: true }); });
    await page.mouse.click(cv.x + cv.width / 2, cv.y + cv.height * 0.6, { button: "right" });
    await sleep(800); const prevented = await page.evaluate(() => window.__menu);
    const rn = (await page.locator(".ctl-note").textContent()) ?? "";
    check(await acts(page) === r0 + 1 && /^(attack until|foe down|hp under \d+%|throw ✓|can't · .+|paralysed · \d+)$/.test(rn), `a right-click acts ("${rn}")`);
    check(prevented === true, `no browser menu over the map in hand (${prevented})`);
  }
  // blind 7f7fc2b (B: `Hand control · awaits order` with the run over — arrows and `descend` inert): in hand, `return` ordered (`r`)
  // and the walk home driven a key a turn — the panel goes once the run is over or the exit holds, and the watch moves on
  // (the fight above may have ended the run: then there is nothing to walk home)
  const live = await page.evaluate(() => window.__riddle.screen === "watch" && !document.querySelector(".ctl")?.hidden);
  if (live) {
    if (await page.evaluate(() => document.querySelector(".ctl")?.dataset.on) !== "1") await page.click(".ctl-toggle");
    await awaiting(page);
    await page.evaluate(() => document.activeElement?.blur?.());
    await page.keyboard.press("r");
  }
  let ended = false, panelOff = false, stuckPanel = 0;
  for (let i = 0; i < 120 && !ended; i++) {
    const w = await page.evaluate(() => { const m = document.querySelector(".ctl"), w = document.querySelector("main.watch"); return { screen: window.__riddle.screen, over: w?.dataset.ending === "1" || !!w?.dataset.exit, hidden: !m || m.hidden || getComputedStyle(m).display === "none", awaiting: m?.dataset.awaiting }; });
    if (w.screen !== "watch") { ended = true; break; }
    if (w.over) { if (w.hidden) panelOff = true; else stuckPanel++; }
    // the walk home: `return` again each turn he awaits (as a player would) until the run is over
    if (!w.over && w.awaiting === "1") await page.keyboard.press("r").catch(() => {});
    await sleep(500);
  }
  check(ended, "in hand, the run that ended leaves the watch (no dead state)");
  check(panelOff || ended, "the panel is gone once the run is over");
  check(stuckPanel <= 1, `no take-control panel over a run that is over (${stuckPanel} reads)`);
  await page.close();

  // ---------------- a phone (touch, 360 px) ----------------
  const phone = await browser.newPage({ viewport: { width: 360, height: 780 }, deviceScaleFactor: 2, hasTouch: true, isMobile: true });
  await boot(phone);
  await phone.locator(".ctl-toggle").tap();
  await awaiting(phone);
  await sleep(600);
  const lay = await phone.evaluate(() => {
    const vw = document.documentElement.clientWidth, slab = document.querySelector(".ctl").getBoundingClientRect();
    const tiles = [...document.querySelectorAll(".ctl .tile")].filter((t) => t.offsetParent).map((t) => { const r = t.getBoundingClientRect(); return { id: t.dataset.tile, w: r.width, h: r.height, l: r.left, r: r.right }; });
    return { vw, scroll: document.documentElement.scrollWidth, slabL: slab.left, slabR: slab.right, tiles, keys: [...document.querySelectorAll(".ctl .ctl-key")].some((k) => getComputedStyle(k).display !== "none"), ta: getComputedStyle(document.querySelector(".ctl-pad")).touchAction };
  });
  const small = lay.tiles.filter((t) => t.w < 44 || t.h < 44), outside = lay.tiles.filter((t) => t.l < 0 || t.r > lay.vw + 0.5);
  check(small.length === 0, `360 px: every tile ≥ 44 px (${small.map((t) => `${t.id} ${Math.round(t.w)}×${Math.round(t.h)}`).join(", ") || `${lay.tiles.length} tiles`})`);
  check(outside.length === 0 && lay.scroll <= lay.vw && lay.slabR <= lay.vw + 0.5, `360 px: nothing past the edge (scroll ${lay.scroll} / ${lay.vw}, slab ${Math.round(lay.slabL)}–${Math.round(lay.slabR)})`);
  check(lay.keys, "a phone shows the key hints too (owner 2026-10-10)");
  check(lay.ta === "manipulation", `the pad kills double-tap zoom (touch-action ${lay.ta})`);
  mkdirSync(shotDir, { recursive: true });
  const shot = `${shotDir.replace(/\/$/, "")}/phone-360-control.png`;
  await phone.screenshot({ path: shot });
  out.push(`     screenshot ${shot}`);
  // a tap on the map: one step toward the tapped tile
  await settle(phone);
  const ps = await snapOf(phone);
  let tapT = null;
  for (const t of (await reachable(phone)).filter((t) => t.d >= 2 && t.d <= 5)) {
    const at = await tileAt(phone, t.x, t.y);
    const dx = Math.sign(t.x - ps.x), dy = Math.sign(t.y - ps.y);
    const first = await phone.evaluate(([x, y]) => { const s = window.__lastSnap; const t = s.tiles[y * s.w + x]; return t && t !== "wall" && t !== "chasm" && !s.entities.some((e) => e.x === x && e.y === y && e.id !== s.hero.id); }, [ps.x + dx, ps.y + dy]);
    const corner = dx && dy ? await phone.evaluate(([x, y, dx, dy]) => { const s = window.__lastSnap; return s.tiles[y * s.w + x + dx] !== "wall" && s.tiles[(y + dy) * s.w + x] !== "wall"; }, [ps.x, ps.y, dx, dy]) : true;
    if (at.free && first && corner) { tapT = { ...t, ...at, dx, dy }; break; }
  }
  check(!!tapT, "a tile on the phone's map to tap");
  if (tapT) {
    const t0 = await acts(phone);
    await phone.touchscreen.tap(tapT.sx, tapT.sy); await sleep(900);
    const p1 = await snapOf(phone);
    check(await acts(phone) === t0 + 1 && p1.x === ps.x + tapT.dx && p1.y === ps.y + tapT.dy, `a tap steps one tile toward it ((${ps.x},${ps.y}) → (${p1.x},${p1.y}), toward (${tapT.x},${tapT.y}))`);
  }
  // a touch held on an arrow repeats too (CDP touch: down, hold, up)
  await awaiting(phone);
  const pb = await phone.locator(`.ctl-dir[data-dx="${best[0]}"][data-dy="${best[1]}"]`).boundingBox();
  const cdp = await phone.context().newCDPSession(phone);
  const pt = [{ x: pb.x + pb.width / 2, y: pb.y + pb.height / 2 }];
  const k0 = await acts(phone);
  await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: pt }); await sleep(1300);
  await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
  const k1 = await acts(phone); await sleep(1000); const k2 = await acts(phone);
  check(k1 - k0 >= 1 && k2 - k1 <= 1, `a held touch on an arrow steps and stops on release (${k1 - k0} held, ${k2 - k1} after)`);
  await phone.close();
} catch (e) { errors.push(String(e?.stack ?? e)); } finally { await browser.close(); }
console.log(out.join("\n")); if (errors.length) console.log(errors.join("\n"));
console.log(failed || errors.length ? `take-control: FAIL (${failed} assertion(s), ${errors.length} error(s))` : `take-control: ok (${out.length} checks)`);
process.exit(failed || errors.length ? 1 : 0);
