#!/usr/bin/env node
// Cut 17 — the frame's gates (docs/CUT17.md, docs/UI.md §5–6), on the fake engine, headless at 400 × 800:
//   · a fresh lineage's camp shows ≤ 8 interactive elements: the bar ($ only), the tablets (compact), the shaft (D1), the gem SEND
//   · each step of the reveal ladder carves its tile on its trigger (and not before), glinting once: first death → edit · first gold
//     → loadout · first mark → unlocks (◆ on the bar); the class picker from the first death · an item kept → vault · a salvage → forge · a companion →
//     party · a 3rd row → the shaft's gems · 5 heirs → ledger, chronicle, set tabs
//   · the unlock panel is a short list (three, `more` for the catalogue)
//   · every sheet and panel carries a close stud, and the stud closes it
//   · every command tile has a pressed state (the tile sinks while held)
//   · every place (camp · watch · death · report) has the bar and the console, the primary action in the gem slot
//
//   node web/tests/ui.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`) }); };

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state();
    if (s?.screen === "exit" && s.vault) { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(100); continue; }
    if (pred(s)) return s;
    await sleep(80);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
async function settle(max = 15_000) {
  const t = Date.now(); let clear = 0;
  while (Date.now() - t < max) { const s = await state(); clear = s && s.booted && !s.busy ? clear + 1 : 0; if (clear >= 2) break; await sleep(120); }
  await sleep(150);
}
/** Patch the fake's saved lineage and reload it through the save (the camp remounts). */
const patchSave = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
  new Function("e", src)(e);
  b.engine = JSON.stringify(e);
  return r.importSave(JSON.stringify(b));
}, `(${fn})(e)`);
/** The interactive elements a player sees above the fold (buttons, inputs, links), open sheets included. */
const interactive = () => page.evaluate(() => [...document.querySelectorAll("button, input, select, textarea, a[href], [role=button]")].filter((b) => {
  if (b.closest("[inert]") || !b.getClientRects().length || getComputedStyle(b).visibility === "hidden") return false;
  const r = b.getBoundingClientRect();
  return r.bottom > 0 && r.top < innerHeight && r.width > 0 && r.height > 0;
}).map((b) => (b.getAttribute("aria-label") || b.textContent || b.className).replace(/\s+/g, " ").trim().slice(0, 24)));
const tiles = () => page.evaluate(() => [...document.querySelectorAll(".console .cmd .tile:not(.empty)")].map((t) => ({ id: t.dataset.tile, reveal: t.classList.contains("reveal") })));
const tileIds = async () => (await tiles()).map((t) => t.id);

const t0 = Date.now();
try {
  // ---- a fresh lineage: heir 1, nothing earned (the fake seeds a chronicle and a free unlock; strip them)
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); localStorage.removeItem("riddle.unlocks.all"); });
  check(await patchSave((e) => {
    const L = e.lineage;
    // as the core's new_lineage: `tame` owned and a free leash on the shelf (neither earned)
    Object.assign(L, { heir: 1, chronicle: [], unlocks: ["tame"], marks: 0, gold: 0, gold_ledger: [], graveyard: [], vault: [], forge: {}, party: [], kennel: [], eggs: [], rank: 0, renown: 0, best_depth: 0, trophies: [],
      supplies: [{ id: 100000, kind: "leash", known: true, label: "leash", free: true }] });
  }), "a fresh lineage (heir 1, nothing earned)");
  await waitFor((s) => s?.screen === "camp", "the fresh camp"); await settle();
  await shot("ui-fresh-camp");
  {
    const els = await interactive();
    check(els.length <= 8, `a fresh camp shows ≤ 8 interactive elements at 400 × 800 (${els.length}: ${els.join(" | ")})`);
    const f = await page.evaluate(() => ({
      stats: [...document.querySelectorAll(".topbar .stats > *")].map((e) => e.className),
      tablets: document.querySelectorAll(".editor .row.tablet.compact").length,
      notches: [...document.querySelectorAll(".shaft .notch")].map((n) => n.querySelector(".dl")?.textContent),
      ends: !document.querySelector(".shaft .shaft-ends")?.hidden,
      gem: document.querySelector(".console .gem")?.textContent.trim(),
      tiles: document.querySelectorAll(".console .cmd .tile:not(.empty)").length,
      tabs: !!document.querySelector(".tabs:not([hidden]) .tab"),
      cls: !!document.querySelector("button.cls"),
    }));
    check(f.stats.length === 1 && /\bgold\b/.test(f.stats[0]), `the bar shows $ only (${f.stats.join(", ")})`);
    check(f.tablets === 2 && f.notches.join(",") === "D1" && !f.ends, `two compact tablets, the shaft at D1 alone, no gems (${f.tablets} tablets, notches ${f.notches.join(",")}, ends ${f.ends})`);
    check(/^send$/i.test(f.gem ?? "") && f.tiles === 0 && !f.tabs && !f.cls, `the gem SEND and nothing else: no tile, no set tab, no class picker (gem "${f.gem}", ${f.tiles} tiles)`);
    // a tablet is one tap: it opens the tablets for editing at that row
    await page.locator(".editor .row.tablet.compact").first().click({ timeout: 5000 }); await sleep(200);
    check(await page.locator(".editor .row .chip.cond").count() > 0, "a tap on a compact tablet opens it for editing (its chips)");
  }

  // ---- the ladder: each tile on its trigger, not before; the first appearance glints
  const steps = [
    ["edit", "first death", (e) => { e.lineage.graveyard.push({ heir: 1, depth: 1, cause: "rat", deeds: [] }); }],
    ["loadout", "first gold home", (e) => { e.lineage.gold = 40; e.lineage.gold_ledger = [{ t: 10, delta: 40, why: "returned D1" }]; }],
    ["unlocks", "first mark", (e) => { e.lineage.marks = 1; }],
    ["vault", "an item worth keeping", (e) => { e.lineage.vault = [{ id: 901, kind: "sword", label: "sword +1" }]; }],
    ["forge", "first salvage", (e) => { e.lineage.forge = { sword: { salvaged: 1, craftable: false, tier: 0 } }; }],
    ["party", "a companion", (e) => { e.lineage.eggs = [{ id: 902, kind: "jackal", tags: [], gen: 1, hatch_in: 3, from_loss: false }]; }],
  ];
  const seen = [];
  for (const [id, why, fn] of steps) {
    const before = await tileIds();
    check(!before.includes(id), `before ${why}: no \`${id}\` tile (${before.join(" · ") || "none"})`);
    check(await patchSave(fn), `the lineage took ${why}`);
    await waitFor((s) => s?.screen === "camp", "camp"); await settle();
    const after = await tiles();
    const t = after.find((x) => x.id === id);
    check(!!t && t.reveal && seen.every((s) => after.some((x) => x.id === s)), `${why} carves the \`${id}\` tile, glinting (${after.map((x) => `${x.id}${x.reveal ? "*" : ""}`).join(" · ")})`);
    seen.push(id);
    if (id === "edit") check(await page.locator("button.cls").count() === 1, "the first death puts the class picker on the portrait");
    if (id === "unlocks") {
      const u = await page.evaluate(() => document.querySelector(".strip .marks")?.textContent);
      check(u === "◆1", `the first mark puts ◆ on the bar (${u})`);
    }
  }
  // a later repaint is no reveal: the tiles stand, none glints (a glint lasts its 1.5 s)
  await sleep(1700); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await settle();
  check((await tiles()).every((t) => !t.reveal) && (await tileIds()).length === seen.length, `a repaint keeps the carved tiles without a glint (${(await tileIds()).join(" · ")})`);
  // a 3rd row: the shaft's gems
  check(await page.evaluate(() => document.querySelector(".shaft .shaft-ends")?.hidden !== false), "before a 3rd row: the shaft has no gems");
  await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [{ k: "hp<", n: 50 }], verb: { v: "attack", a: "nearest" } }, 2); r.go({ kind: "camp" }); });
  await page.waitForFunction(() => document.querySelector(".shaft .shaft-ends")?.hidden === false, null, { timeout: 15_000 }).catch(() => {});
  const ends = await page.evaluate(() => [...document.querySelectorAll(".shaft .shaft-ends .end")].map((e) => e.textContent.replace(/\s+/g, " ").trim()));
  check(ends.length === 4 && /^bank \d+%$/.test(ends[0]) && /^return \d+%$/.test(ends[1]) && /^death \d+%$/.test(ends[2]) && /^~\$\d+$/.test(ends[3]), `a 3rd row lights the shaft's gems: ${ends.join(" · ")}`);
  // 5 heirs: ledger, chronicle, the set tabs
  check(!(await tileIds()).includes("ledger") && !(await page.locator(".tabs:not([hidden]) .tab").count()), "before the 5th heir: no ledger, no chronicle, no set tabs");
  check(await patchSave((e) => { e.lineage.heir = 5; }), "the lineage took its 5th heir");
  await waitFor((s) => s?.screen === "camp", "camp"); await settle();
  const ids = await tileIds();
  check(ids.includes("ledger") && ids.includes("chronicle") && (await page.locator(".tabs:not([hidden]) .tab").count()) >= 3, `the 5th heir carves ledger and chronicle and the set tabs (${ids.join(" · ")})`);
  check(ids.length === 8, `the command card is full at the ladder's top (${ids.length} tiles)`);
  await shot("ui-ladder-camp");

  // ---- the unlock panel: the next three, `more` for the catalogue
  await patchSave((e) => { e.lineage.marks = 30; });
  await waitFor((s) => s?.screen === "camp", "camp"); await settle();
  await page.locator(".cmd .tile[data-tile=unlocks]").click({ timeout: 5000 });
  await page.waitForFunction(() => document.querySelectorAll(".panel .unlocks .card").length > 0, null, { timeout: 15_000 });
  const cards0 = await page.locator(".panel .unlocks .card").count();
  const more = await page.locator(".panel .unlocks button.more").count();
  check(cards0 === 3 && more === 1, `the unlock panel lists the next three, \`more\` under them (${cards0} cards, more ${more})`);
  await page.locator(".panel .unlocks button.more").click({ timeout: 5000 }); await sleep(200);
  const cards1 = await page.locator(".panel .unlocks .card").count();
  check(cards1 > 3 && !(await page.locator(".panel .unlocks button.more").count()), `\`more\` opens the catalogue (${cards1} cards)`);
  await shot("ui-unlocks-panel");

  // ---- every panel and sheet carries a close stud that closes it
  for (const p of ["unlocks", "loadout", "vault", "party"]) {
    const open = await page.locator(".panel-host .panel").count();
    if (!open || (await page.locator(`.panel[data-panel=${p}]`).count()) === 0) await page.locator(`.cmd .tile[data-tile=${p}]`).click({ timeout: 5000 });
    await sleep(150);
    const has = await page.locator(`.panel[data-panel=${p}] .close-stud`).count();
    await page.locator(`.panel[data-panel=${p}] .close-stud`).click({ timeout: 5000 }).catch(() => {}); await sleep(150);
    check(has === 1 && !(await page.locator(".panel-host .panel").count()), `the ${p} panel has a close stud, and it closes the panel`);
  }
  await page.locator(".shaft").click({ timeout: 5000 }); await sleep(150);
  check(await page.locator(".panel[data-panel=forecast] .close-stud").count() === 1, "the shaft opens the forecast panel, with its stud");
  await page.keyboard.press("Escape"); await sleep(150);
  check(!(await page.locator(".panel-host .panel").count()), "Escape closes the open panel");
  const sheets = [
    ["settings", "button.gear"], ["gold", ".strip button.gold"], ["forge", ".cmd .tile[data-tile=forge]"], ["ledger", ".cmd .tile[data-tile=ledger]"],
    ["chronicle", ".cmd .tile[data-tile=chronicle]"], ["class", "button.cls"], ["cond picker", ".editor .row .chip.cond"], ["verb picker", ".editor .row .chip.verb"], ["rename", ".tabs .tab.edit"],
  ];
  for (const [name, sel] of sheets) {
    await page.locator(sel).first().click({ timeout: 5000 }); await sleep(200);
    const s = await page.evaluate(() => { const w = [...document.querySelectorAll(".sheet-wrap")].pop(); return w ? { stud: w.querySelectorAll(".sheet > .close-stud, .sheet .sheet-x").length } : null; });
    if (s?.stud) { await page.locator(".sheet-wrap .sheet > .close-stud, .sheet-wrap .sheet .sheet-x").last().click({ timeout: 5000 }); await sleep(150); }
    const left = await page.locator(".sheet-wrap").count();
    check(!!s && s.stud === 1 && left === 0, `the ${name} sheet has one close stud, and it closes the sheet`);
    if (left) for (let i = 0; i < 4; i++) await page.keyboard.press("Escape");
  }
  await page.locator(".cmd .tile[data-tile=unlocks]").click({ timeout: 5000 }); await sleep(200);
  await page.locator(".panel .unlocks .card").first().click({ timeout: 5000 }); await sleep(200);
  check(await page.locator(".sheet-wrap .sheet > .close-stud").count() === 1, "an unlock card's sheet has its close stud");
  await page.keyboard.press("Escape"); await page.keyboard.press("Escape"); await sleep(150);

  // ---- every tile has a pressed state: held, it sinks (and darkens)
  const pressed = [];
  for (const id of await tileIds()) {
    const loc = page.locator(`.cmd .tile[data-tile=${id}]`);
    const box = await loc.boundingBox(); if (!box) { pressed.push(`${id}: no box`); continue; }
    const rest = await loc.evaluate((b) => getComputedStyle(b).transform), on = await loc.evaluate((b) => b.classList.contains("on"));   // an `on` tile rests pressed
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2); await page.mouse.down(); await sleep(120);
    const down = await loc.evaluate((b) => ({ t: getComputedStyle(b).transform, img: getComputedStyle(b).borderImageSource }));
    await page.mouse.move(200, 400); await page.mouse.up(); await sleep(60);
    if (!/matrix\(1, 0, 0, 1, 0, 2\)/.test(down.t) || (!on && down.t === rest)) pressed.push(`${id}: ${rest} → ${down.t}`);
    else if (/tile\.png/.test(down.img)) pressed.push(`${id}: the raised frame while held`);
  }
  for (let i = 0; i < 3; i++) { await page.keyboard.press("Escape"); await sleep(60); }
  check(pressed.length === 0, `every tile sinks while held (${pressed.join("; ") || (await tileIds()).length + " tiles"})`);

  // ---- every place: the bar, the console, the primary in the gem
  const frame = () => page.evaluate(() => {
    const m = document.querySelector("#app > main");
    const g = m?.querySelector(":scope > footer.console .gem-slot > .gem");
    return { main: m?.className ?? "", bar: !!m?.querySelector(":scope > header.topbar"), console: !!m?.querySelector(":scope > footer.console"), gem: g ? { text: g.textContent.trim(), cls: g.className, visible: !!g.getClientRects().length && getComputedStyle(g).visibility !== "hidden" } : null };
  });
  let f = await frame();
  check(f.bar && f.console && f.gem?.visible && /\bsend\b/.test(f.gem.cls) && f.gem.text === "send", `camp: bar + console, \`send\` in the gem (${JSON.stringify(f.gem)})`);
  await page.locator("button.send").click({ timeout: 5000 });
  await waitFor((s) => s?.screen === "watch", "the watch"); await sleep(1500);
  f = await frame();
  check(f.bar && f.console && f.gem?.visible && ["⏸", "▶"].includes(f.gem.text), `watch: bar + console, ⏸ in the gem (${JSON.stringify(f.gem)})`);
  const cmd = await page.evaluate(() => [...document.querySelectorAll(".console .cmd .tile.hud-btn")].map((b) => b.textContent.trim()));
  check(cmd.join(" · ") === "fights · fast · ▶▶| · bail", `watch: the command card is fights · fast · ▶▶| · bail (${cmd.join(" · ")})`);
  await shot("ui-watch");
  let s = await state(); const tw = Date.now();
  while (s?.screen === "watch" && Date.now() - tw < 90_000) { await page.evaluate(() => { for (const b of document.querySelectorAll("button.hud-btn")) if (b.textContent === "▶▶|") b.click(); }); await sleep(250); s = await state(); }
  if (s?.screen === "exit") { await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 }); }
  s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after the run", 30_000); await settle();
  if (s.screen !== "death") {   // a report: the gem is `camp`; then a fabricated death for the death's frame
    f = await frame();
    check(f.bar && f.console && f.gem?.visible && f.gem.text === "camp", `report: bar + console, \`camp\` in the gem (${JSON.stringify(f.gem)})`);
    const r = await page.evaluate(async () => { const r = window.__riddle; const L = await r.engine.lineage(); const g = L.graveyard.at(-1); return g?.death_id; });
    if (r !== undefined) { await page.evaluate(async (id) => { const r = window.__riddle; r.go({ kind: "death", death: await r.engine.death(id), kept: true }); }, r); await waitFor((x) => x?.screen === "death", "a death"); await settle(); }
  }
  if ((await state())?.screen === "death") {
    f = await frame();
    const patch = await page.locator("button.patch.top").count();
    check(f.bar && f.console && f.gem?.visible && (patch ? /\bpatch-gem\b/.test(f.gem.cls) : f.gem.text === "edit"), `death: bar + console, the top patch in the gem (${JSON.stringify(f.gem)}, ${patch} lit patch)`);
    const d = await page.evaluate(() => ({ banner: !!document.querySelector(".banner-cloth .death-line"), seal: getComputedStyle(document.querySelector(".death-line .verdict")).position, tiles: [...document.querySelectorAll(".console .cmd .tile:not(.empty)")].map((t) => t.textContent.trim()) }));
    check(d.banner && d.seal === "absolute" && d.tiles.includes("morgue") && d.tiles.includes("camp"), `death: the line on the banner, the verdict in the seal, morgue · camp on the card (${d.tiles.join(" · ")})`);
    await shot("ui-death");
    // the morgue's sheet carries its stud too
    await page.locator(".cmd .tile[data-tile=morgue]").click({ timeout: 5000 }); await sleep(200);
    check(await page.locator(".sheet-wrap .sheet > .close-stud").count() === 1, "the morgue sheet has its close stud");
    await page.keyboard.press("Escape"); await sleep(100);
  } else check(false, "no death to check the death's frame on");
  await page.goto(`${url}?dev=1&engine=fake&absent=2h`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && (x.screen === "report" || x.screen === "ending"), "the report", 120_000); await settle();
  f = await frame();
  const plaques = await page.locator(".report .parchment .tile.plaque").count();
  check(f.bar && f.console && f.gem?.visible && f.gem.text === "camp" && plaques >= 4, `report: bar + console, \`camp\` in the gem, the counts as plaques on parchment (${plaques} plaques)`);
  await shot("ui-report");
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
const secs = ((Date.now() - t0) / 1000).toFixed(1);
if (failed || errors.length) { console.error(`ui: FAIL (${failed} assertion(s), ${errors.length} error(s), ${secs}s)`); process.exit(1); }
console.log(`ui: ok (${out.length} checks · ${secs}s)`);
