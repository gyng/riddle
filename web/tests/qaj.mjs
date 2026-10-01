#!/usr/bin/env node
// QA on 3d71c33 (qaJ, seed 816) — the client-side lapses, each reproduced on the fake engine (`?engine=fake&systems=none&dev=1`) through the
// headless harness (tools/browser.mjs) against the dev server (tools/dev.sh; RIDDLE_PORT picks the port):
//   1  the exit beat (`BANKED $N`) shows when the PLAYHEAD reaches the exit, never over an earlier fight the viewer replays behind
//      the frontier (`fast`, the picture paused while the world runs to the bank)
//   2  a chain link's clip is on the link's floor and spans its tick (`data-depth` · `data-from` · `data-to` on the sheet), and its
//      first caption comes from the events in that window (the seek sets no stale row caption)
//   3  a floor's card shows once per floor entry (a card between fights on the same floor is a ghost: `data-ghost`, the card
//      element hidden), and never over a fight (no hostile adjacent to the hero when a card goes up)
//   4  the rule-set tab counts own rows (`fighter · 6` beside `6/6 · 3 cards`), repainted when a card lands
//   5  the unlock sheet: a disabled `◆ buy` looks as disabled as a disabled `$ buy` (no accent fill), and a short `$ buy` reads
//      `$N short`
//   6  the cage sheet is titled `cage`; `⏸` and `▶▶|` stay tappable while it waits (⏸ keeps the sheet, ▶▶| closes it choosing
//      nothing); a full vault says `vault full`
//   7  the lit chip's digits carry `×` (`16×`)
//   8  the drop sheet has a `×` close; tied least-fired rows mark none
//
//   node web/tests/qaj.mjs        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows, openPanel, deathDetails } from "./lib/frame.mjs";
import { measured } from "./lib/load.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
await deathDetails(page);   // death v2: this suite reads the trace, the ledger and the tablets under `details`
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => {
  const r = window.__riddle, w = document.querySelector(".watch");
  return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, frame: w?.dataset.frame, tick: Number(w?.dataset.tick ?? NaN), card: w?.dataset.card, sheets: document.querySelectorAll(".sheet-wrap").length } : null;
});
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(60); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const inRun = (s) => s?.screen === "watch";
/** Record every exit event the engine hands the client (tick, depth). */
const recordExit = () => page.evaluate(() => {
  const r = window.__riddle, orig = r.engine.step.bind(r.engine);
  r.__exit = null;
  r.engine.step = async (n) => { const res = await orig(n); const x = res.events.find((e) => e.k === "exit"); if (x) r.__exit = { t: x.t, depth: res.snapshot.depth, tier: x.tier }; return res; };
});

try {
  // ---- 1: `BANKED $N` at the exit, not over an earlier fight replayed behind the frontier (seed 7 fights on D1, banks on D2)
  // (a wall-clock reading — the exit's grace lets the beat go when the picture lags on a loaded machine: `measured` retries it once
  // then, the bar unchanged; tests/lib/load.mjs)
  const banked = await measured(async () => {
    const rules = encodeURIComponent("foes>=1 → attack nearest\ndepth>=2 → bank");
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=fast&rules=${rules}`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && inRun(s) && Number.isFinite(s.tick), "the fast watch");
    await recordExit();
    await page.locator(".gem.hud-btn").first().click({ timeout: 2000 });   // ⏸: the world runs on to the bank
    const t0 = Date.now();
    while (Date.now() - t0 < 15_000 && !(await page.evaluate(() => window.__riddle.__exit))) await sleep(100);
    const exit = await page.evaluate(() => window.__riddle.__exit);
    await page.locator(".gem.hud-btn").first().click({ timeout: 2000 });   // ▶: the picture replays from behind
    const seen = []; const t1 = Date.now();
    while (Date.now() - t1 < 40_000) {
      const s = await page.evaluate(() => ({ screen: window.__riddle.screen, tk: document.querySelector(".ticker.show")?.textContent ?? "", tick: Number(document.querySelector(".watch")?.dataset.tick), depth: document.querySelector(".watch .depth")?.textContent ?? "" }));
      if (s.screen !== "watch") break;
      if (/^BANKED/.test(s.tk)) seen.push(s);
      await sleep(40);
    }
    const early = seen.filter((s) => s.tick < exit.t - 2 || s.depth !== `D${exit.depth}`);
    return { ok: !!exit && exit.tier === "bank" && seen.length > 0 && early.length === 0, line: `BANKED shows at the exit only (exit t${exit?.t} D${exit?.depth}; seen at ${[...new Set(seen.map((s) => `t${s.tick} ${s.depth}`))].slice(0, 4).join(", ") || "never"})` };
  });
  check(banked.ok, banked.line);

  // ---- 2: a chain link's clip — its floor, its tick inside the window, no caption from before the window
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=fast&fake_depth=3`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && inRun(s) && Number.isFinite(s.tick), "the fast watch for the clip");
    // ▶▶| in `fast` is the run's end: 100-tick engine batches, so a floor's snapshot lands after its first events
    const t0 = Date.now();
    while (Date.now() - t0 < 60_000) {
      const s = await state(); if (!inRun(s)) break;
      await page.locator(".cmd .hud-btn", { hasText: "▶▶|" }).click({ timeout: 1000 }).catch(() => {});
      await sleep(400);
    }
    let s = await waitFor((x) => x && !inRun(x), "the run's end", 30_000);
    if (s.screen === "exit") {
      await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 }).catch(() => {});
      s = await waitFor((x) => x && !inRun(x) && x.screen !== "exit" && !x.busy, "after keep", 30_000);
    }
    await waitFor((x) => x && !x.busy, "idle", 30_000);
    // a link per floor at a row's tick with another row (another caption) fired 20–60 ticks before it — the caption a stale seek showed
    const links = await page.evaluate(() => {
      const l = window.__riddle.runLog(); const res = [];
      for (const f of l.floors) {
        const rules = f.evs.filter((e) => e.k === "rule" && e.row >= 0);
        // a tick before the floor's snapshot (the batch reached the floor before its end), and a tick with a row fired just before it
        const early = rules.find((e) => e.t < f.snap.turn);
        const stale = rules.find((e) => { const win = rules.filter((q) => q.t >= e.t - 20 && q.t <= e.t + 20); return rules.some((p) => p.t < e.t - 20 && p.t > e.t - 60 && !win.some((q) => q.text === p.text)); });
        const mid = rules[Math.floor(rules.length / 2)];
        for (const pick of new Set([early, stale ?? mid])) if (pick) res.push({ t: pick.t, depth: f.snap.depth, snap: f.snap.turn, first: f.evs[0]?.t });
      }
      return { runId: l.runId, links: res, floors: l.floors.map((f) => `D${f.snap.depth} snap t${f.snap.turn} first t${f.evs[0]?.t}`) };
    });
    check(links.links.length >= 1, `the run log has links to test (${links.floors.join(" · ")})`);
    // the floor lookup on a log whose second floor's snapshot came after its first events (a 100-tick batch across the descend)
    const pick = await page.evaluate(async () => {
      const { floorFor } = await import("/src/ui/runlog.ts");
      const ev = (t) => ({ t, k: "move", id: 0, x: 1, y: 1 });
      const snap = (depth, turn) => ({ depth, turn, entities: [], items: [], hero: { id: 0 } });
      const log = { runId: 1, startedTurn: 0, floors: [
        { snap: snap(1, 0), evs: [ev(10), ev(500)], ents: new Map(), items: new Map() },
        { snap: snap(2, 600), evs: [ev(510), ev(700)], ents: new Map(), items: new Map() },
      ] };
      const d = (b) => floorFor(log, b)?.snap.depth ?? null;
      return { early: d({ text: "", t: 520, depth: 2 }), other: d({ text: "", t: 520, depth: 3 }), late: d({ text: "", t: 650, depth: 2 }) };
    });
    // the seek itself: a row fired before the landing tick leaves no caption (the clip's first frame was `R5 ATTACK GOBLIN` from a
    // row before its window); one fired at the landing tick keeps its caption
    {
      const k = links.links[0];
      await page.evaluate(({ runId, k }) => { window.__riddle.go({ kind: "death", death: { run_id: runId, depth: k.depth, cause: "goblin", margin: "1 over", verdict: "gap", baseline: 0.25, morgue: "", patches: [], trace: { turns: [] }, chain: [{ text: "the seek", t: k.t, depth: k.depth }] } }); }, { runId: links.runId, k });
      await sleep(250);
      await page.locator(".chain button.link").first().click({ timeout: 5000 }); await sleep(600);
      const cap = await page.evaluate(async () => {
        const v = window.__replay, t = v.tick(), frame = () => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r)));
        v.setSpeed(0);
        v.apply([{ t: t - 5, k: "rule", row: 7, verb: { v: "attack", a: "nearest" }, text: "foes>=1 → zzz before" }]); v.seek(t); await frame();
        const before = v.stats().caption;
        v.apply([{ t: t + 1, k: "rule", row: 6, verb: { v: "attack", a: "nearest" }, text: "foes>=1 → yyy at" }]); v.seek(t + 1); await frame();
        return { before, at: v.stats().caption };
      });
      check(cap.before === null && cap.at === "yyy at", `a seek drops a caption from before its tick (${cap.before ?? "none"}) and keeps one at it (${cap.at})`);
      await page.keyboard.press("Escape"); await sleep(200);
    }
    check(pick.early === 2 && pick.other === null && pick.late === 2, `a link before its floor's snapshot finds its floor, never another depth's (t520 D2 → D${pick.early}, t520 D3 → ${pick.other === null ? "none" : `D${pick.other}`}, t650 D2 → D${pick.late})`);
    for (const k of links.links) {
      await page.evaluate(({ runId, k }) => {
        window.__riddle.go({ kind: "death", death: { run_id: runId, depth: k.depth, cause: "goblin", margin: "1 over", verdict: "gap", baseline: 0.25, morgue: "", patches: [],
          trace: { turns: [{ t: k.t + 50, row: 0, verb: { v: "attack", a: "nearest" }, hp: 3, foes: 1, telegraphs: [] }] }, chain: [{ text: `the link D${k.depth}`, t: k.t, depth: k.depth }] } });
      }, { runId: links.runId, k });
      await sleep(250);
      const link = page.locator(".chain button.link").first();
      if (!(await link.count())) { check(false, `D${k.depth} t${k.t}: the link is watchable`); continue; }
      await link.click({ timeout: 5000 });
      // the first frames after the seek: the caption must be a row firing inside the window (or none)
      const caps = []; let clip = null; const tc = Date.now();
      while (Date.now() - tc < 1200) {
        const c = await page.evaluate(() => { const b = document.querySelector(".sheet-wrap .replay"), v = window.__replay; return b && v ? { depth: Number(b.dataset.depth), from: Number(b.dataset.from), to: Number(b.dataset.to), cap: v.stats?.().caption ?? null, tick: v.tick?.() } : null; });
        if (c) { clip ??= c; if (c.cap) caps.push(c.cap); }
        await sleep(30);
      }
      const allowed = await page.evaluate(({ from, to, depth }) => {
        const l = window.__riddle.runLog(); const f = l.floors.find((x) => x.snap.depth === depth);
        return (f?.evs ?? []).filter((e) => e.k === "rule" && e.row >= -1 && e.t >= from && e.t <= to).map((e) => { const tail = e.text.includes("→") ? e.text.slice(e.text.lastIndexOf("→") + 1).trim() : e.text; return (e.row >= 0 ? tail.trim().split(/\s+/).slice(0, 3).join(" ") : e.text.replace(/→/g, ">")).slice(0, 24); });   // render/state.ts: ≤ 3 words of the verb, no row id (docs/COPY.md)
      }, clip ?? { from: 0, to: 0, depth: 0 });
      const stale = caps.filter((c) => !allowed.includes(c));
      check(!!clip && clip.depth === k.depth && clip.from <= k.t && k.t <= clip.to, `D${k.depth} t${k.t} (floor snap t${k.snap}, first t${k.first}): the clip is on D${clip?.depth}, t${clip?.from}–t${clip?.to}`);
      check(!!clip && stale.length === 0, `D${k.depth} t${k.t}: captions from the window only (${[...new Set(caps)].join(" · ") || "none"}${stale.length ? `; stale: ${[...new Set(stale)].join(" · ")}` : ""})`);
      await page.keyboard.press("Escape"); await sleep(200);
    }
  }

  // ---- 3: one card per floor entry, never over a fight (two `fights` runs, 25 s each). "Over a fight": a hero blow / a hit on the
  // hero / a telegraph in the 10 ticks before the playhead, or a visible hostile adjacent in an engine snapshot of those ticks
  // (read off the frames as they come — a card's tick is the frame's: `measured` takes the run once more on a loaded machine)
  for (const seed of [516, 7]) {
    const cards = await measured(async () => {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=${seed}&autosend=1&speed=fights&early=0`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && inRun(s), "the fights watch");
    await page.evaluate(() => {
      const r = window.__riddle, orig = r.engine.step.bind(r.engine);
      const fightTicks = window.__fightTicks = [];
      r.engine.step = async (n) => {
        const res = await orig(n), s = res.snapshot, hero = s.hero;
        for (const e of res.events) if (e.k === "attack" || e.k === "telegraph" || (e.k === "hurt" && e.id === hero.id)) fightTicks.push(e.t);
        const adj = s.entities.some((e) => !e.ally && !e.remembered && !/bones|captive/.test(e.kind) && !(e.tags ?? []).some((t) => t === "captive" || t === "ally") && s.visible[e.y * s.w + e.x] && Math.max(Math.abs(e.x - hero.x), Math.abs(e.y - hero.y)) <= 1);
        if (adj) fightTicks.push(s.turn);
        return res;
      };
      const w = document.querySelector(".watch"), card = document.querySelector(".interstitial");
      const log = window.__cards = { shown: [], ups: [] };
      let was = false, wasUp = false;
      new MutationObserver(() => {
        const up = w.dataset.card === "1" && !card.hidden, anyUp = w.dataset.card === "1";
        if (anyUp && !wasUp) log.ups.push({ tick: Number(w.dataset.tick), text: card.hidden ? "ghost" : card.textContent });
        if (up && !was) log.shown.push(document.querySelector(".watch .depth")?.textContent ?? "?");
        was = up; wasUp = anyUp;
      }).observe(w, { attributes: true, attributeFilter: ["data-card", "data-frame", "data-ghost"] });
    });
    const t0 = Date.now();
    while (Date.now() - t0 < 25_000) { if (!inRun(await state())) break; await sleep(200); }
    const { log, ticks } = await page.evaluate(() => ({ log: window.__cards, ticks: window.__fightTicks }));
    const per = {}; for (const d of log.shown) per[d] = (per[d] ?? 0) + 1;
    const one = { ok: log.shown.length >= 1 && Object.values(per).every((n) => n === 1), line: `seed ${seed}: one card per floor (${Object.entries(per).map(([d, n]) => `${d} ×${n}`).join(" · ")})` };
    const over = log.ups.filter((u) => ticks.some((t) => t <= u.tick && t > u.tick - 10));
    const clear = { ok: log.ups.length >= 2 && over.length === 0, line: `seed ${seed}: no card over a fight (${log.ups.length} cards${over.length ? `; over: ${over.slice(0, 3).map((u) => `${u.text} t${u.tick}`).join(" | ")}` : ""})` };
    return { ok: one.ok && clear.ok, line: `${one.line} · ${clear.line}`, one, clear };
    });
    const retried = cards.line.includes(" [retried") ? cards.line.slice(cards.line.indexOf(" [retried")) : "";
    check(cards.one.ok, cards.one.line);
    check(cards.clear.ok, cards.clear.line + retried);
  }

  // ---- 4: the tab counts own rows; the counter the cards beside. Three cards bought on a full set (the fake's own buy; cards sit
  // outside the cap, Cut 12 §1): no drop is asked, the set stays full, the tab and the counter follow what is listed
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=21`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "the camp");
    await editRows(page);   // Cut 17: the tablets carry their chips, ▲▼ and × (the `edit` tile, remembered)
    await page.evaluate(async () => {
      const r = window.__riddle; r.sets[r.active].name = "fighter";
      while (r.ownRows() < r.vocab.max_rows) r.insertRow({ conds: [{ k: "hp<", n: 30 + r.rules.rows.length }], verb: { v: "retreat" } }, r.rules.rows.length);
      const save = JSON.parse(await r.engine.save()); save.lineage.marks = 40; save.lineage.facts.push("foe:jackal:pack", "foe:archer:ranged", "foe:thief:thief"); r.lineage = await r.engine.load(JSON.stringify(save)); await r.engine.setRules(r.rules);
      r.unlockCat = await r.engine.unlocks(); r.go({ kind: "camp" });
    });
    await sleep(300);
    const read = () => page.evaluate(() => { const r = window.__riddle; return { tab: document.querySelector(".tabs .tab.on")?.textContent.replace(/\s+/g, " ").trim(), count: document.querySelector(".rows-foot .num")?.textContent.replace(/\s+/g, " ").trim(), rows: document.querySelectorAll(".editor .row").length, own: r.ownRows(), max: r.vocab.max_rows }; });
    const bought = [];
    for (let i = 0; i < 3; i++) {
      const id = await page.evaluate(async (have) => { const r = window.__riddle; r.unlockCat = await r.engine.unlocks(); const u = r.unlockCat.find((x) => !x.owned && x.available && /^card:|^(pack_break|boss_focus|corridor_fighting|stair_dance|gas_step|last_stand|kite_archers|thief_guard)$/.test(x.id) && !have.includes(x.id)); return u && (await r.buy(u.id)) ? u.id : null; }, bought);
      if (id) bought.push(id);
      await sleep(300);
    }
    const c = await read();
    check(bought.length === 3, `three cards bought on a full set (${bought.join(", ")})`);
    check(c.own === c.max && c.tab === `fighter · ${c.own} rules` && c.count.startsWith(`${c.own}/${c.max} rules + 3 tactics`) && c.rows === c.own + 3 && (await state()).sheets === 0, `the tab counts own rows: "${c.tab}" beside "${c.count}" (${c.rows} rows listed, no sheet)`);
  }

  // ---- 5: the unlock sheet's disabled buys look disabled; a short `$ buy` says `$N short` (a fresh lineage: ◆0, $120)
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=21`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "the camp for the unlock sheet");
    await openPanel(page, "unlocks", { all: true });   // Cut 17: the unlock shelf is a panel (its whole catalogue behind `more`)
    await page.locator(".unlocks .card", { hasText: "+1 vault" }).first().click({ timeout: 5000 }); await sleep(200);
    const b = await page.evaluate(() => {
      const look = (el) => { if (!el) return null; const cs = getComputedStyle(el); return { disabled: el.disabled, bg: cs.backgroundColor, color: cs.color, opacity: Number(cs.opacity) }; };
      return { marks: look(document.querySelector(".sheet-wrap button.buy.marks")), goldBtn: look(document.querySelector(".sheet-wrap button.buy.gold")), short: document.querySelector(".sheet-wrap .gold-short")?.textContent.trim() ?? "", price: Number(/\$(\d+)/.exec(document.querySelector(".sheet-wrap .gold-price")?.textContent ?? "")?.[1] ?? NaN), gold: window.__riddle.lineage.gold,
        acc: getComputedStyle(document.documentElement).getPropertyValue("--acc").trim() };
    });
    const hex2rgb = (hx) => `rgb(${parseInt(hx.slice(1, 3), 16)}, ${parseInt(hx.slice(3, 5), 16)}, ${parseInt(hx.slice(5, 7), 16)})`;
    check(b.marks?.disabled && b.marks.bg !== hex2rgb(b.acc) && b.marks.opacity <= 0.5, `a disabled ◆ buy is not the accent fill (bg ${b.marks?.bg}, opacity ${b.marks?.opacity})`);
    check(b.goldBtn?.disabled && b.marks?.bg === b.goldBtn.bg && b.marks.opacity === b.goldBtn.opacity && b.marks.color === b.goldBtn.color, `both disabled buys look alike (◆ ${b.marks?.bg} ${b.marks?.color} · $ ${b.goldBtn?.bg} ${b.goldBtn?.color})`);
    check(b.short === `$${b.price - b.gold} short`, `a short $ buy says so: "${b.short}" ($${b.gold} of $${b.price})`);
    await page.keyboard.press("Escape"); await sleep(150);
  }

  // ---- 6: the cage sheet — its title, ⏸ and ▶▶| live under it, `vault full` when the pick will be salvaged. Cut 19 §1: the sheet is the
  //      override — opened by a tap on the cage beat (`took sword`) within its hold
  for (const mode of ["fast", "fights"]) {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=7&autosend=1&speed=${mode}&early=0`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && inRun(s), `the ${mode} watch for the cage`);
    const full = mode === "fights";
    await page.evaluate((full) => {
      const r = window.__riddle, orig = r.engine.step.bind(r.engine), origChoose = r.engine.choose.bind(r.engine); let done = false;
      r.__chosen = null;
      if (full) r.lineage.vault = [{ id: 7001, kind: "axe", known: true, label: "axe" }];   // 1 slot, taken
      // QA 23ed91f: after the cage, the pack holds the sword (the pref's pick), so a pick the player did not make can be named
      r.engine.step = async (n) => { const res = await orig(n); if (done && !res.snapshot.vault_choice) res.snapshot.hero.inv = [...res.snapshot.hero.inv, { id: 9901, kind: "sword", known: true, label: "sword" }]; if (!done && res.snapshot.turn > 20 && !res.run_over && !res.events.some((e) => e.k === "exit")) { done = true; res.snapshot.vault_choice = { items: [{ id: 9901, kind: "sword", known: true, label: "sword" }, { id: 9902, kind: "mail", known: true, label: "mail" }], left: 50, pick: 9901 }; } return res; };
      window.__took = []; new MutationObserver(() => { const t = document.querySelector(".ticker")?.textContent ?? ""; if (/^took /.test(t) && !window.__took.includes(t)) window.__took.push(t); }).observe(document.body, { subtree: true, childList: true, characterData: true });
      r.engine.choose = async (id) => { r.__chosen = id; return origChoose(id); };
    }, full);
    const sheet = () => page.evaluate(() => { const b = document.querySelector(".sheet-wrap .vault-choice"); return b ? { title: b.querySelector(".row-label")?.textContent.replace(/\s+/g, " ").trim(), full: b.querySelector(".vault-full")?.textContent.trim() ?? "" } : null; });
    let v = null; const t0 = Date.now();
    while (Date.now() - t0 < 20_000) {
      v = await sheet(); if (v || !["watch", "exit"].includes((await state())?.screen)) break;
      if (await page.locator(".watch .ticker.cage.show").count()) await page.locator(".watch .ticker.cage.show").click({ timeout: 1000 }).catch(() => {});
      await sleep(50);
    }
    if (!v) { check(false, `${mode}: the cage sheet opened on a tap on the beat`); continue; }
    const on = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .vault-choice .chip.item.on")].map((c) => c.textContent.trim()));
    check(on.length === 1 && /sword/.test(on[0]), `${mode}: the override marks the preference's pick (${on.join(",")})`);
    check(/^cage\b/.test(v.title) && (full ? v.full === "vault full → sold" : v.full === ""), `${mode}: the sheet reads "${v.title}"${full ? `, "${v.full}"` : ""}`);
    const pause = await page.locator(".gem.hud-btn").first().click({ timeout: 2000 }).then(() => true, () => false);
    await sleep(300);
    const p = await page.evaluate(() => ({ paused: document.querySelector(".gem.hud-btn")?.textContent, sheet: !!document.querySelector(".sheet-wrap .vault-choice") }));
    check(pause && p.paused === "▶" && p.sheet, `${mode}: ⏸ takes the tap under the cage and keeps the sheet (${pause ? p.paused : "click intercepted"}, sheet ${p.sheet})`);
    await page.locator(".gem.hud-btn").first().click({ timeout: 2000 }).catch(() => {});
    const skip = await page.locator(".cmd .hud-btn", { hasText: "▶▶|" }).click({ timeout: 2000 }).then(() => true, () => false);
    await sleep(400);
    const k = await page.evaluate(() => ({ sheet: !!document.querySelector(".sheet-wrap .vault-choice"), chosen: window.__riddle.__chosen }));
    check(skip && !k.sheet && k.chosen === null, `${mode}: ▶▶| takes the tap, closes the sheet and picks nothing (${skip ? `sheet ${k.sheet}, chose ${k.chosen}` : "click intercepted"})`);
    // QA 23ed91f (K: "the cage sheet closed by itself … nothing said what had been taken"): the pick the pref made is named
    await page.waitForFunction(() => window.__took.length > 0 || !["watch", "exit"].includes(window.__riddle.screen), null, { timeout: 10_000 }).catch(() => {});
    const took = await page.evaluate(() => window.__took);
    check(took.includes("took sword"), `${mode}: a pick the player did not make is named on the ticker (${took.join(" · ") || "nothing"})`);
  }

  // ---- 7: the lit chip's rate reads `16×`
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && inRun(s), "the watch for the chip");
    let c = null; const t0 = Date.now();
    while (Date.now() - t0 < 10_000) { c = await page.evaluate(() => { const on = document.querySelector(".cmd .hud-btn.on"); return on?.dataset.rate ? { rate: on.dataset.rate, after: getComputedStyle(on, "::after").content } : null; }); if (c) break; await sleep(50); }
    check(!!c && c.after === `"${c.rate}×"`, `the lit chip reads its rate with ×: ${c ? c.after : "no rate"}`);
  }

  // ---- 8: the drop sheet — a × close; tied least-fired rows mark none, a unique minimum marks one
  {
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=21`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "camp", "the camp for the drop sheet");
    const max = await page.evaluate(() => { const r = window.__riddle; while (r.ownRows() < r.vocab.max_rows) r.insertRow({ conds: [{ k: "hp<", n: 30 + r.rules.rows.length }], verb: { v: "retreat" } }, r.rules.rows.length); r.go({ kind: "camp" }); return r.vocab.max_rows; });
    const death = { run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns: [] }, morgue: "",
      patches: [{ row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 0.75, forecast_delta: 0.05 }] };
    const open = async (fires) => { await page.evaluate(({ d, fires }) => { const r = window.__riddle; r.rowFires = fires; r.go({ kind: "death", death: d }); }, { d: death, fires }); await sleep(250); await page.locator("button.patch").first().click({ timeout: 5000 }); await page.locator(".patch-gem").click({ timeout: 5000 }); await sleep(200); };
    const marks = () => page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .drop-sheet .drop-row")].filter((b) => b.classList.contains("least")).length);
    await open(Array(max).fill(0));
    check((await marks()) === 0, "all rows at 0 fires: none marked");
    const x = await page.locator(".sheet-wrap .drop-sheet button.x").click({ timeout: 2000 }).then(() => true, () => false);
    await sleep(200);
    const after = await state();
    check(x && after.sheets === 0 && after.screen === "death", `the × closes the drop sheet, the death screen stays (${x ? `${after.sheets} sheets, ${after.screen}` : "no ×"})`);
    const tied = Array(max).fill(5); tied[0] = 1; tied[1] = 1;
    await open(tied);
    check((await marks()) === 0, "two rows tied at the minimum: none marked");
    await page.keyboard.press("Escape"); await sleep(150);
    const uniq = Array(max).fill(5); uniq[1] = 0;
    await open(uniq);
    const one = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .drop-sheet .drop-row.least")].map((b) => Number(b.dataset.row)));
    check(one.length === 1 && one[0] === 1, `a unique minimum is marked (${one.join(",")})`);
    await page.keyboard.press("Escape"); await sleep(150);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`qaj: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`qaj: ok (${out.length} checks)`);
