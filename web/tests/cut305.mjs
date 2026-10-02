#!/usr/bin/env node
// Cut 30.5 client gates — the works tree (docs/CUT30_5.md, docs/AUTOMATION_TREE.md §3–§4), on the fake engine (its 30.5 stand-ins),
// headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU). The tracks panel's coverage folds in here (`branches`):
//   day0      a heir-1 camp: the chest drawn (closed, no surface), the mouth, the gem, the pill (the wire's `next`: `send` before the
//             scout); ≤ 5 interactive surfaces in the well and the console; ≤ 12 elements above the fold
//   chest     a haul waits in the chest (`!`, its sum); a tap opens it: the purse takes it all, the porter's count moves (`1/3`), the chest
//             never caps (a second haul adds up)
//   porter    3 chests opened → the porter lit and free (the pill gold, his greyed figure in the town) → hired from the pill's works sheet
//             (`PORTER HIRED`, ≤ 2 words; he stands at his post) → he opens chests: the next haul lands in the purse, no chest
//   scout     the manual sends fill the gem's counter toward the scout (`1/3` …); before him an absence yields ≤ 1 run (the run in
//             flight); hired, the gem reads `auto` and an absence sends again and again
//   sheet     the works sheet: the done nodes dimmed, the lit node with its hire and price, two silhouettes, never more; ≤ 1 hire; copy
//             caps (node names ≤ 2 words, state lines ≤ 3 words + a number); mid-game (the trunk done) the four branches (the tracks'
//             stage, the next and its trigger, the hero's face on the character branch) and ≤ 7 nodes on a phone
//   toggles   settings: each hired worker a switch (`setWorker`): the porter off brings the chest back, on again retires it
//   report    an absence's report lists the workers' acts compactly (`porter · hauled $N`) within the lead; slices merge
//   rank      week 2: a rank on offer (no hire lit) — promote on the works, `PORTER II`, the numeral on the done row, a pip in the town
//   walk      the scripted first 10 minutes (§4 table, step by step; the pill as the contract orders it)
//
//   node web/tests/cut305.mjs [--shots dir] [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const words = (s) => String(s ?? "").split(/\s+/).filter((w) => /\p{L}/u.test(w)).length;
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: shots ? 2 : 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name, ms = 500) => { if (shots) { await page.mouse.move(1, 300); await sleep(ms); await page.screenshot({ path: resolve(shots, `${name}.png`) }); } };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp" && window.__town, "camp"); await sleep(350); };
/** a heir-1 lineage with nothing earned (cut30town's stage 0) and a fresh works tree; `fn` edits the save (`e.lineage`, `e.st305`) */
const fresh = (fn = "") => page.evaluate(async (src) => {
  const r = window.__riddle; const e = JSON.parse(await r.engine.save());
  Object.assign(e.lineage, { heir: 1, chronicle: [], unlocks: ["tame"], marks: 0, gold: 0, gold_ledger: [], graveyard: [], vault: [], forge: {}, party: [], kennel: [], eggs: [], rank: 0, renown: 0, best_depth: 0, trophies: [], trait_offer: [], class_offer: [],
    supplies: [{ id: 100000, kind: "leash", known: true, label: "leash", free: true }] });
  delete e.st305;
  new Function("e", src)(e);
  r.lineage = await r.engine.load(JSON.stringify(e)); await r.refresh(); r.go({ kind: "camp" });
}, fn);
/** the save edited in place (the works' state `e.st305`), then the camp */
const withState = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const e = JSON.parse(await r.engine.save());
  e.st305 ??= { hired: ["quartermaster"], counts: {}, chest: 0, sent: false, paused: [], acted: [] };
  new Function("e", `(${src})(e)`)(e);
  r.lineage = await r.engine.load(JSON.stringify(e)); await r.refresh(); r.go({ kind: "camp" });
}, String(fn));
const boot = async (seed) => {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}&speed=fast`, { waitUntil: "domcontentloaded" });
  await until(() => window.__riddle?.booted && ["camp", "report"].includes(window.__riddle.screen), "boot", 60_000);
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); for (const k of Object.keys(localStorage)) if (k.startsWith("riddle.town.")) localStorage.removeItem(k); });
};
const S = () => page.evaluate(() => {
  const r = window.__riddle, L = r.lineage, W = L.tree, t = window.__town?.stats();
  const node = (id) => W?.nodes.find((n) => n.id === id);
  return { screen: r.screen, gold: L.gold, chest: W?.chest ?? 0, next: W?.next, waits: W?.waits, auto: W?.auto_send, lit: W?.lit, porter: node("porter"), scout: node("scout"),
    tchest: t?.chest, workers: t?.workers ?? [], targets: window.__town?.targets().map((x) => x.id) ?? [], markers: t?.markers ?? [],
    pill: document.querySelector(".next-pill:not([hidden])")?.textContent ?? null, pillKind: document.querySelector(".next-pill")?.dataset.kind,
    gem: document.querySelector(".console .gem.send")?.textContent ?? "" };
});
/** one watched send from the camp (the gem): the run plays (fast), any exit sheet kept, the report read, home */
async function sendRun(via = "gem") {
  if (via === "mouth") await page.locator('.town-hit[data-building="mouth"]').click(); else await page.locator(".console .gem.send").click();
  await until(() => window.__riddle.screen === "watch", "the watch", 10_000);
  const t = Date.now();
  while (Date.now() - t < 90_000) {
    const s = await page.evaluate(() => window.__riddle.screen);
    if (s === "report" || s === "death" || s === "camp") break;
    await page.evaluate(() => { for (const b of document.querySelectorAll(".sheet-wrap .btn.primary")) b.click(); });
    await sleep(250);
  }
  const s = await page.evaluate(() => window.__riddle.screen);
  if (s === "report") await page.locator(".console .gem").click();
  else if (s === "death") await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
  await camp();
  return s;
}
/** the interactive elements a player sees above the fold, one per function (cut30town's reading; the pill and the lit worker both open
 *  the works on the lit node — one function) */
const SAME = { forge: "blacksmith", vault: "storehouse", party: "kennel", bank: "bank", loadout: "crate" };
const fold = (scope = "") => page.evaluate(([SAME, scope]) => {
  const shown = (b) => {
    let r = b.getBoundingClientRect(); let top = Math.max(0, r.top), bottom = Math.min(innerHeight, r.bottom);
    for (let p = b.parentElement; p && bottom > top; p = p.parentElement) {
      const o = getComputedStyle(p).overflowY; if (o === "visible") continue;
      const q = p.getBoundingClientRect(); top = Math.max(top, q.top); bottom = Math.min(bottom, q.bottom);
    }
    return bottom - top > 1 && r.width > 0;
  };
  const els = [...document.querySelectorAll(`${scope} :is(button, input, select, textarea, a[href], [role=button])`)].filter((b) => {
    if (b.closest("[inert]") || !b.getClientRects().length || getComputedStyle(b).visibility === "hidden" || b.hidden) return false;
    return shown(b);
  });
  const key = (b) => b.classList.contains("next-pill") || b.dataset.building === "worker" ? "works" : b.dataset.building ?? (b.dataset.tile ? SAME[b.dataset.tile] ?? `tile:${b.dataset.tile}` : (b.getAttribute("aria-label") || b.textContent || b.className).replace(/\s+/g, " ").trim().slice(0, 24));
  return [...new Set(els.map(key))];
}, [SAME, scope]);
const openSheetFromPill = async () => { await page.locator(".next-pill").click(); await until(() => !!document.querySelector(".works-sheet"), "the works sheet", 5000); await sleep(200); };
const sheetRead = () => page.evaluate(() => {
  const q = (s) => [...document.querySelectorAll(s)];
  return { lit: q(".works-sheet .wnode.lit").map((n) => n.dataset.node), cur: q(".works-sheet .wnode.cur").map((n) => n.dataset.node), sil: q(".works-sheet .wnode.sil").map((n) => n.dataset.node),
    done: q(".works-sheet .works-done .wd").map((n) => n.dataset.node), hires: q(".works-sheet .hire-btn").map((b) => b.textContent), focus: document.querySelector(".works-sheet .wnode.focus")?.dataset.node,
    names: q(".works-sheet .wn-name").map((n) => n.textContent), states: q(".works-sheet .wnode .wn-state").map((n) => n.textContent), branches: q(".works-sheet .wbranch").map((b) => ({ id: b.dataset.branch, now: b.querySelector(".wb-now")?.textContent, next: b.querySelector(".wb-next")?.textContent, face: !!b.querySelector("img.track-hero") })),
    nodesOnScreen: q(".works-sheet .wnode, .works-sheet .works-done .wd, .works-sheet .wbranch").filter((n) => { const r = n.getBoundingClientRect(); return r.bottom > 0 && r.top < innerHeight; }).length };
});
const closeSheets = () => page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); });

try {
  await boot(Number(process.env.CUT305_SEED ?? 30504));

  // ---- day 0
  if (part("day0") || part("walk")) {
    await fresh(); await camp();
    const s = await S();
    await shot("day0");
    const surf = await fold(":is(.well-wrap, .console)");
    check(s.tchest === "closed" && !s.targets.includes("chest"), `day 0: the chest drawn by the mouth, closed and empty — no surface (${s.tchest}; targets ${s.targets.join(",")})`);
    check(s.targets.includes("mouth") && /send/i.test(s.gem), `day 0: the mouth and the gem send (${s.targets.join(",")} · gem "${s.gem.trim()}")`);
    check(!!s.pill && s.pill === s.next?.text && ["send", "count"].includes(s.next?.kind), `day 0: the pill reads the wire's next goal (${s.pill} · ${s.next?.kind}; the contract's order: \`send\` while the hero waits before the scout, else \`porter · n/3\`)`);
    check(surf.length <= 5 && ["mouth", "tent", "crate", "works"].every((x) => surf.includes(x)) && surf.some((x) => /send/.test(x)), `day 0: ≤ 5 interactive surfaces — mouth, tent, crate, gem, pill (${surf.length}: ${surf.join(" | ")})`);
    const all = await fold();
    check(all.length <= 12, `day 0: ≤ 12 elements above the fold (${all.length}: ${all.join(" | ")})`);
    check(/0\/3/.test(s.gem), `day 0: the gem counts the sends toward the scout (${s.gem.trim()})`);
  }

  // ---- the scripted first 10 minutes (§4), step by step — on real (fake) sends
  if (part("walk")) {
    const log = [];
    const step = (t, ok, what) => { log.push(`${t} ${ok ? "✓" : "✗"} ${what}`); check(ok, `walk ${t}: ${what}`); };
    let s = await S();
    step("0:00", s.pill === "send" && s.gold === 0 && JSON.stringify(s.workers) === JSON.stringify(["quartermaster"]), `camp, $${s.gold}, pill \`${s.pill}\`; on the scene only the given quartermaster by the crate (${s.workers.join(",")})`);
    let sends = 0, chests = 0, purse = s.gold, guard = 0;
    while ((await S()).porter?.state !== "lit" && guard++ < 10) {
      const via = sends === 0 ? "mouth" : "gem";
      const end = await sendRun(via); sends++;
      s = await S();
      if (sends === 1) step("0:05", true, `SEND (${via}) → the watch → home (${end})`);
      if (s.chest > 0) {
        const t = ["2:30", "5:10", "7:40"][Math.min(chests, 2)];
        step(t, s.tchest === "full" && s.markers.includes("chest@chest") && s.pill === "open chest" && s.waits, `home: the chest full by the mouth ($${s.chest}, \`!\`), pill \`${s.pill}\`, the hero waits`);
        if (chests === 0) await shot("chest-full");
        const before = s.gold, haul = s.chest;
        await page.locator('.town-hit[data-building="chest"]').click();
        if (chests === 0) await shot("chest-open", 240);   // (the coins in the air)
        await until(() => (window.__riddle.lineage.tree?.chest ?? 1) === 0, "the chest emptied", 5000);
        await sleep(200); s = await S(); chests++;
        step(`${t}+`, s.gold === before + haul && s.porter.count === chests && (s.tchest === "open" || s.porter.state === "lit"), `chest tapped: $${before} → $${s.gold} (+$${haul}), porter ${s.porter.count}/${s.porter.need}, the pill moves (\`${s.pill}\`)`);
        purse = s.gold;
      } else step(`run ${sends}`, s.chest === 0, `a run home with no haul (died or empty): nothing in the chest, the hero waits (pill \`${s.pill}\`)`);
    }
    s = await S();
    step("7:40", s.porter.state === "lit" && s.pillKind === "buy" && /porter · free/.test(s.pill) && s.workers.includes("porter(lit)"), `3 chests: the pill turns gold (\`${s.pill}\`); a greyed porter by the mouth (${s.workers.join(",")})`);
    await shot("pill-porter");
    const all = await fold();
    check(all.length <= 12, `porter lit: ≤ 12 elements above the fold (${all.length}: ${all.join(" | ")})`);
    await openSheetFromPill();
    let w = await sheetRead();
    step("7:45", w.lit.length === 1 && w.lit[0] === "porter" && w.focus === "porter" && w.sil.length === 2 && w.hires.length === 1 && /free/.test(w.hires[0]), `pill → the works on PORTER (lit, \`${w.hires[0]}\`); silhouettes ${w.sil.join(", ")}; done ${w.done.join(", ")}`);
    await shot("works-early");
    await page.locator(".works-sheet .hire-btn").click();
    const beat = await until(() => document.querySelector(".works-beat")?.textContent, "the beat", 3000).catch(() => "");
    await sleep(300); s = await S();
    step("7:47", beat === "PORTER HIRED" && s.porter.state === "done" && s.workers.includes("porter") && !s.tchest, `hire → \`${beat}\` (${words(beat)} words); the porter at his post (${s.workers.join(",")}); the chest retired; pill \`${s.pill}\``);
    await sleep(1600); await shot("porter-hired");
    // the scout is lit next (3 sends by hand); the trigger met, his price in the pill
    // (the contract's pill order: the lit node affordable · a chest · the hero waiting before the scout · the lit node short)
    step("7:50", s.lit === "scout" && (s.pillKind === "buy" ? /^scout · \$/.test(s.pill) : s.pill === "send") && s.workers.includes("scout(lit)"), `the scout lights after ${sends} sends by hand (≤ 5: ${sends <= 5}), greyed by the fire: pill \`${s.pill}\` (${s.pillKind})`);
    check(sends <= 5 + 2, `the porter and the scout lit within the walk's sends (${sends}; the fake's runs that die bring no haul)`);
    if (!s.scout.affordable) await withState((e) => { e.lineage.gold += 500; });   // (the fake's purse short of his unit: topped up)
    await openSheetFromPill();
    await page.locator(".works-sheet .hire-btn").click();
    await until(() => window.__riddle.lineage.tree?.auto_send, "the scout hired", 5000);
    await sleep(300); s = await S();
    step("8:00", s.auto && /auto/.test(s.gem) && s.workers.includes("scout"), `the scout hired: the gem reads auto (${s.gem.trim()}), he stands at his post`);
    const g0 = s.gold;
    let end = "", tries = 0, s2;
    do { end = await sendRun(); s2 = await S(); tries++; } while (s2.gold === g0 && tries < 4);
    step("10:20", s2.chest === 0 && !s2.tchest && s2.gold > g0, `a run ends: the porter wheels the haul home — $${g0} → $${s2.gold}, no chest (${end}, ${tries} runs)`);
    console.log(log.join("\n"));
  }

  // ---- the chest: a tap empties it into the purse; it never caps
  if (part("chest")) {
    await fresh(); await camp();
    await withState((e) => { e.lineage.gold += 34; e.st305.chest = 34; });
    await camp();
    let s = await S();
    check(s.tchest === "full" && s.targets.includes("chest") && s.markers.includes("chest@chest") && s.pill === "open chest", `a haul waits: the chest full, a surface, its \`!\` and sum (${s.markers.join(",")}); pill \`${s.pill}\``);
    const lab = await page.evaluate(() => ({ m: document.querySelector('.town-hit[data-building="chest"] .town-marker')?.textContent, c: document.querySelector('.town-hit[data-building="chest"] .town-count')?.textContent }));
    check(lab.m === "$34" && lab.c === "0/3", `the chest carries its sum and the porter's count (${lab.m} · ${lab.c})`);
    // it never caps: a second haul adds up
    await withState((e) => { e.lineage.gold += 9000; e.st305.chest += 9000; });
    await camp();
    s = await S();
    check(s.chest === 9034 && s.gold === 0, `the chest never caps: $34 + $9000 = $${s.chest}, the purse untouched ($${s.gold})`);
    await page.locator('.town-hit[data-building="chest"]').click();
    await until(() => window.__riddle.lineage.tree?.chest === 0, "the chest emptied", 5000);
    await sleep(150); s = await S();
    check(s.gold === 9034 && s.porter.count === 1 && s.tchest === "open" && !s.targets.includes("chest"), `one tap: the purse takes it all ($${s.gold}), porter 1/3, the lid open, no surface while empty (${s.targets.join(",")})`);
    await sleep(1700); s = await S();
    check(s.tchest === "closed", `the lid shuts again (${s.tchest})`);
  }

  // ---- the porter: lit, free, hired, then he opens the chests
  if (part("porter")) {
    await fresh(); await camp();
    await withState((e) => { e.st305.counts = { porter: 3 }; e.lineage.gold += 20; e.st305.chest = 20; e.lineage.best_depth = 2; });
    await camp();
    let s = await S();
    check(s.porter.state === "lit" && s.porter.price === 0 && s.pillKind === "buy" && /porter · free/.test(s.pill) && s.workers.includes("porter(lit)") && s.targets.includes("worker"),
      `3 chests opened: the porter lit and free (pill \`${s.pill}\`), greyed at his post (${s.workers.join(",")}), his figure a surface`);
    const price = await page.evaluate(() => document.querySelector('.town-hit[data-building="worker"] .town-marker')?.textContent);
    check(price === "free", `his marker reads his price (${price})`);
    await page.locator('.town-hit[data-building="worker"]').click();
    await until(() => !!document.querySelector(".works-sheet .wnode.lit"), "the works from the figure", 5000);
    const w = await sheetRead();
    check(w.focus === "porter" && w.lit[0] === "porter", `the greyed figure opens the works on him (${w.focus})`);
    await page.locator(".works-sheet .hire-btn").click();
    await until(() => window.__riddle.lineage.tree?.nodes.find((n) => n.id === "porter")?.state === "done", "the porter hired", 5000);
    await sleep(300); s = await S();
    check(s.chest === 0 && s.gold === 20 && !s.tchest && s.workers.includes("porter") && !s.workers.includes("porter(lit)"), `hired: the waiting haul carried in ($${s.gold}), the chest gone, the porter at his post (${s.workers.join(",")})`);
    await withState((e) => { e.st305.hired.push("scout"); });   // (the scout too: an absence sends)
    const r = await page.evaluate(async () => { const r = window.__riddle; const g0 = r.lineage.gold; const rep = await r.runOfflineChunked(3 * 3600); await r.refresh(); return { runs: rep.runs, workers: rep.workers, chest: rep.chest ?? 0, g0, L: { chest: r.lineage.tree.chest, gold: r.lineage.gold } }; });
    const hauled = (r.workers ?? []).find((x) => x.id === "porter");
    check(r.runs > 0 && r.L.chest === 0 && r.chest === 0 && (!!hauled ? r.L.gold >= r.g0 : true), `he opens the chests: ${r.runs} runs, the hauls in the purse ($${r.g0} → $${r.L.gold}), the chest $${r.L.chest}; his act \`${hauled ? `porter · ${hauled.what}` : "none (no haul)"}\``);
  }

  // ---- the scout: the manual sends fill his counter; before him an absence is one run; after, auto
  if (part("scout")) {
    await fresh(); await camp();
    let s = await S();
    const g = [];
    g.push(s.gem.match(/\d\/\d/)?.[0]);
    for (let i = 0; i < 3; i++) { await sendRun(); s = await S(); g.push(s.gem.match(/\d\/\d/)?.[0] ?? (s.lit === "scout" ? "lit" : "?")); }
    check(JSON.stringify(g.slice(0, 3)) === JSON.stringify(["0/3", "1/3", "2/3"]) && s.scout.count >= 3, `each manual send fills the scout's counter on the gem (${g.join(" → ")}; count ${s.scout.count})`);
    // before the scout: an absence yields at most the run in flight (none here: the hero waits at home)
    const before = await page.evaluate(async () => { const r = window.__riddle; const rep = await r.runOfflineChunked(8 * 3600); await r.refresh(); return { runs: rep.runs, waits: r.lineage.tree.waits }; });
    check(before.runs <= 1 && before.waits, `before the scout an 8 h absence yields ≤ 1 run (${before.runs}); the hero waits`);
    await withState((e) => { e.st305.counts.porter = 0; e.lineage.gold += 500; });   // (the porter not yet: the scout lit; the fake's purse covers his unit)
    s = await S();
    check(s.lit === "scout", `3 sends: the scout lit (${s.lit} · \`${s.pill}\`)`);
    await openSheetFromPill();
    await page.locator(".works-sheet .hire-btn").click();
    await until(() => window.__riddle.lineage.tree?.auto_send, "the scout hired", 5000);
    await sleep(400); s = await S();
    check(s.auto && /auto/.test(s.gem) && !/\d\/\d/.test(s.gem) && s.workers.includes("scout"), `hired: the gem shows auto-send on (${s.gem.trim()}); the scout at his post`);
    const rest = await page.evaluate(() => document.querySelector(".rest-line .rest")?.textContent ?? "");
    check(!/waits/.test(rest), `the hero no longer waits (rest line "${rest}")`);
    const after = await page.evaluate(async () => { const r = window.__riddle; const rep = await r.runOfflineChunked(8 * 3600); await r.refresh(); return { runs: rep.runs }; });
    check(after.runs > 1, `after the scout an 8 h absence sends again and again (${after.runs} runs)`);
  }

  // ---- the works sheet: done + lit + 2 silhouettes, never the whole tree; mid-game the branches
  if (part("sheet")) {
    await fresh(); await camp();
    await withState((e) => { e.st305.counts = { porter: 3, scout: 1 }; e.lineage.best_depth = 2; });
    await openSheetFromPill();
    let w = await sheetRead();
    check(w.lit.length === 1 && w.sil.length === 2 && w.done.length === 1 && w.cur.length === 1 && w.hires.length <= 1, `early: done ${w.done.join(",")} · lit ${w.lit.join(",")} · silhouettes ${w.sil.join(",")} — never the whole tree; ≤ 1 hire (${w.hires.length})`);
    check(w.names.every((n) => words(n) <= 2) && w.states.every((x) => words(x.replace(/\d+\/\d+|\$\d+/g, "")) <= 3), `copy: names ≤ 2 words (${w.names.join(", ")}), state lines ≤ 3 words + a number (${w.states.join(" | ")})`);
    const tip = await page.evaluate(() => document.querySelector(".works-sheet .wnode.lit .wn-tip")?.textContent ?? "");
    const tips = await page.evaluate(() => [...document.querySelectorAll(".works-sheet .wn-tip, .kw-tip")].map((t) => t.textContent));
    check(!!tip && tips.every((t) => words(t) <= 4 && !/\byou\b|[.!?]\s*$/i.test(t)), `the lit node says what it retires in a fragment of ≤ 4 words, no \`you\`, no stop ("${tip}"; ${tips.join(" | ")})`);
    await shot("works-early-sheet");
    await closeSheets();
    // a counting node with nothing lit: it is the current node, no hire
    await withState((e) => { e.st305.counts = { porter: 1 }; });
    await openSheetFromPill();
    w = await sheetRead();
    check(w.lit.length === 0 && w.cur.length === 1 && w.hires.length === 0 && w.sil.length === 2, `nothing lit: the counting node is current (${w.cur.join(",")}), no hire, two silhouettes (${w.sil.join(",")})`);
    await closeSheets();
    // mid-game: the trunk done → the four branches (the tracks' stages, the next and its trigger)
    await withState((e) => { e.st305.hired = ["quartermaster", "porter", "scout", "armourer", "apprentice"]; e.lineage.best_depth = 9; e.lineage.gold = 900; e.lineage.vault = [{ id: 1, kind: "sword", label: "sword", known: true }]; e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 6, cause: "goblin" }]; });
    await openSheetFromPill();
    w = await sheetRead();
    const wire = await page.evaluate(() => window.__riddle.lineage.tree.nodes.filter((n) => n.kind === "stage"));
    check(JSON.stringify(w.branches.map((b) => b.id)) === JSON.stringify(["character", "items", "scale", "town"].filter((t) => wire.some((n) => n.branch === t))), `mid: the trunk done, the branches open in order (${w.branches.map((b) => b.id).join(" · ")})`);
    check(w.branches.every((b) => { const nx = wire.find((n) => n.branch === b.id && n.state === "next"); return nx ? b.next === `next · ${nx.name}${nx.trigger ? ` · ${nx.trigger}` : ""}` : b.next === "✓"; }), `each branch: its stage and \`next · <stage> · <trigger>\` (${w.branches.map((b) => `${b.now} / ${b.next}`).join(" | ")})`);
    check(w.branches.find((b) => b.id === "character")?.face === true, `the character branch wears the hero's face`);
    check(w.done.length === 5 && w.sil.length <= 2 && w.nodesOnScreen <= 7 + 5, `mid: the trunk's five done in one dim row, the next ≤ 2 silhouettes; ${w.nodesOnScreen} items on screen (≤ 7 nodes + the done row)`);
    await shot("works-mid");
    await closeSheets();
  }

  // ---- ranks (week 2): one rank on offer when no hire is lit — its card and promote on the works; the numeral on the done row; a quiet
  // pip on the worker in the town; the `PORTER II` beat; the rank's tooltip term
  if (part("rank")) {
    await fresh(); await camp();
    await withState((e) => { e.st305.hired = ["quartermaster", "porter", "scout"]; e.lineage.best_depth = 3; e.lineage.gold += 2000; });
    let s = await S();
    const offer = await page.evaluate(() => { const W = window.__riddle.lineage.tree; const n = W.nodes.find((x) => x.id === W.lit_rank); return { lit: W.lit ?? null, rank: W.lit_rank ?? null, price: n?.rank_price, at: n?.rank }; });
    check(offer.rank === "porter" && !offer.lit && offer.at === 1 && offer.price > 0, `a rank on offer while no hire is lit (${offer.rank} I → II · $${offer.price})`);
    await page.evaluate(async () => { const { openWorks } = await import("/src/ui/works.ts"); openWorks(window.__riddle, window.__riddle.lineage.tree.lit_rank); });
    await until(() => !!document.querySelector(".works-sheet .wnode.rank"), "the rank card", 5000);
    const card = await page.evaluate(() => { const c = document.querySelector(".works-sheet .wnode.rank"); return { node: c?.dataset.node, name: c?.querySelector(".wn-name")?.textContent, btn: c?.querySelector(".promote-btn")?.textContent, buys: document.querySelectorAll(".works-sheet :is(.hire-btn, .promote-btn):not([disabled])").length, kw: !!c?.querySelector('[data-kw="rank"]'), focus: document.querySelector(".works-sheet .wnode.focus")?.dataset.node, numerals: [...document.querySelectorAll(".works-sheet .wd .wd-rank")].map((x) => x.textContent.trim()) }; });
    check(card.node === "porter" && card.name === "porter II" && card.btn === `promote · $${offer.price}` && card.buys === 1 && card.focus === "porter", `the works: the rank card \`${card.name}\` with \`${card.btn}\`, ≤ 1 buy (${card.buys}), focused`);
    check(card.kw && card.numerals.length === 0, `the card marks the \`rank\` term; no numeral on a rank-1 done node (${card.numerals.join(",") || "none"})`);
    const tip = await page.evaluate(async () => { const { TIP } = await import("/src/ui/concepts.ts"); return TIP.rank; });
    check(!!tip && words(tip) <= 4, `the rank's tooltip entry ("${tip}")`);
    await shot("works-rank");
    await page.locator(".works-sheet .promote-btn").click();
    const beat = await until(() => document.querySelector(".works-beat")?.textContent, "the beat", 3000).catch(() => "");
    await sleep(300);
    const after = await page.evaluate(() => ({ rank: window.__riddle.lineage.tree.nodes.find((n) => n.id === "porter").rank, ranks: window.__town.stats().ranks }));
    check(beat === "PORTER II" && after.rank === 2 && after.ranks.porter === 2, `promote → \`${beat}\`; porter rank ${after.rank}; his pip in the town (${JSON.stringify(after.ranks)})`);
    await page.evaluate(async () => { const r = window.__riddle; await r.mutate(() => r.engine.promote(r.lineage.tree.lit_rank)); await r.mutate(() => r.engine.promote(r.lineage.tree.lit_rank)); });
    await page.evaluate(async () => { const { openWorks } = await import("/src/ui/works.ts"); openWorks(window.__riddle); });
    await until(() => !!document.querySelector(".works-sheet"), "the works", 5000);
    const nums = await page.evaluate(() => [...document.querySelectorAll(".works-sheet .wd")].map((x) => `${x.dataset.node}${x.querySelector(".wd-rank")?.textContent ?? ""}`));
    check(nums.some((x) => / (II|III)$/.test(x)), `the done row shows the ranks as numerals (${nums.join(", ")})`);
    await closeSheets(); await sleep(300);
    s = await S();
    const r2 = await page.evaluate(() => window.__town.stats().ranks);
    check(Object.keys(r2).length >= 2 && s.workers.includes("porter"), `the town: ranked workers carry their pips (${JSON.stringify(r2)})`);
    await page.evaluate(() => window.__town.setHour(12));
    await shot("town-ranks", 800);
  }

  // ---- toggles: each hired worker a switch in settings
  if (part("toggles")) {
    await fresh(); await camp();
    await withState((e) => { e.st305.hired = ["quartermaster", "porter", "scout"]; });
    await page.locator(".topbar .gear").click();
    await until(() => document.querySelectorAll(".settings .worker-switch").length > 0, "the workers' switches", 5000);
    const sw = await page.evaluate(() => [...document.querySelectorAll(".settings .worker-switch")].map((b) => `${b.dataset.worker}:${b.getAttribute("aria-checked")}`));
    check(JSON.stringify(sw) === JSON.stringify(["porter:true", "scout:true"]), `settings: a switch per hired worker with a chore (${sw.join(" ")})`);
    await page.locator('.settings .worker-switch[data-worker="porter"]').click();
    await until(() => window.__riddle.lineage.tree.nodes.find((n) => n.id === "porter").paused, "the porter off", 5000);
    await page.locator('.settings .worker-switch[data-worker="scout"]').click();
    await until(() => window.__riddle.lineage.tree.auto_send === false, "the scout off", 5000);
    await closeSheets(); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
    let s = await S();
    check(s.porter.paused && s.tchest === "closed" && s.workers.includes("porter(off)") && /\d\/\d/.test(s.gem) === false && !s.auto, `off: the chest is back by the mouth (${s.tchest}), the porter dimmed (${s.workers.join(",")}); the scout off — sends by hand (gem "${s.gem.trim()}")`);
    await page.locator(".topbar .gear").click();
    await until(() => document.querySelectorAll(".settings .worker-switch").length > 0, "the switches again", 5000);
    const sw2 = await page.evaluate(() => document.querySelector('.settings .worker-switch[data-worker="porter"]').getAttribute("aria-checked"));
    await page.locator('.settings .worker-switch[data-worker="porter"]').click();
    await page.locator('.settings .worker-switch[data-worker="scout"]').click();
    await until(() => !window.__riddle.lineage.tree.nodes.find((n) => n.id === "porter").paused && window.__riddle.lineage.tree.auto_send, "both on", 5000);
    await closeSheets(); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
    s = await S();
    check(sw2 === "false" && !s.tchest && s.auto, `on again: the switch read off before (${sw2}), the chest retired, auto-send back`);
  }

  // ---- the report: the workers' acts, compact, within the lead; slices merge
  if (part("report")) {
    await fresh(); await camp();
    await withState((e) => { e.st305.hired = ["quartermaster", "porter", "scout"]; e.st305.acted = []; e.lineage.best_depth = 3; });
    const rep = await page.evaluate(async () => {
      const r = window.__riddle;
      const report = await r.runOfflineChunked(3 * 3600); await r.refresh(); r.go({ kind: "report", report, absence: true });
      return { workers: report.workers, runs: report.runs, grew: report.grew };
    });
    await until(() => window.__riddle.screen === "report", "the report");
    await sleep(400);
    const lead = await page.evaluate(() => {
      const acts = [...document.querySelectorAll(".report-sheet .works-acts .work-act")].map((a) => a.textContent);
      const fold = document.querySelector(".report-sheet .details-fold"), wa = document.querySelector(".report-sheet .works-acts");
      const above = wa && fold ? !!(wa.compareDocumentPosition(fold) & Node.DOCUMENT_POSITION_FOLLOWING) : false;
      // the lead's words: what grew, the workers' line and the news (the tiles are numbers; the ledger folds under `details`)
      const text = [...document.querySelectorAll(".report-sheet :is(.grew, .works-acts, .news)")].map((x) => x.innerText).join(" ");
      return { acts, above, lead: text };
    });
    const leadWords = words(lead.lead);
    check(lead.acts.length >= 1 && lead.acts.every((a) => /^porter · hauled \$\d+$/.test(a.trim()) || words(a) <= 5) && lead.above, `the report lists the workers' acts in its lead, compact (${lead.acts.join(" | ")}; ${rep.runs} runs)`);
    check(leadWords <= 25, `the lead stays ≤ 25 words with the workers' line (${leadWords} words: ${lead.lead.replace(/\s+/g, " ").trim().slice(0, 160)})`);
    await shot("report-workers");
    const merged = await page.evaluate(async () => {
      const { mergeWorkers } = await import("/src/ui/works.ts");
      return mergeWorkers([{ id: "apprentice", what: "+1 step", n: 1, first: true }, { id: "porter", what: "hauled $40", n: 40, first: false }], [{ id: "apprentice", what: "+2 steps", n: 2, first: false }, { id: "clerk", what: "+$100 banked", n: 100, first: true }]);
    });
    check(JSON.stringify(merged) === JSON.stringify([{ id: "apprentice", what: "+3 steps", n: 3, first: true }, { id: "porter", what: "hauled $40", n: 40, first: false }, { id: "clerk", what: "+$100 banked", n: 100, first: true }]), `slices merge: each worker's acts summed (${merged.map((x) => `${x.id} ${x.what}`).join(" · ")})`);
    const grew = await page.evaluate(() => { const g = document.querySelector(".report-sheet .grew"), w = document.querySelector(".report-sheet .works-acts"); return { g: !!g, order: !!g && !!w && !!(g.compareDocumentPosition(w) & Node.DOCUMENT_POSITION_FOLLOWING) }; });
    check(!rep.grew?.length || (grew.g && grew.order), `the report keeps what grew (\`ReturnReport.grew\`, ${rep.grew?.length ?? 0} lines) above the workers' line (${JSON.stringify(grew)})`);
  }
} catch (e) {
  check(false, `threw: ${e.message}`);
}
const errs = errors.filter((e) => !/favicon|404|net::ERR/.test(e));
check(errs.length === 0, `no page errors (${errs.slice(0, 3).join(" | ")})`);
await browser.close();
console.log(out.join("\n"));
console.log(failed ? `cut305: ${failed} failed` : "cut305: all passed");
process.exit(failed ? 1 : 0);
