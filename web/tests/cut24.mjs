#!/usr/bin/env node
// Cut 24 client gates (docs/CUT24.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §1  (fights.mjs: no 15 s of dead watch, the fight frame's included) — here: a `driven` exit (the core's `no counter`: a boss no
//       blow could move drove the hero off) reads `Warlord · no counter · shield wall` with `try: attack boss` on the report, and
//       a tap writes the counter at R1
//   §2  the end-of-run summary leads with what was new (`ExitLine.news`, core): the report's first lines, before the counts; an
//       absence's firsts across its runs, each once
//   §3  the forge's move reads as an edit's — `D9 +7 · death −7`, the depth first, never a bare `death +7`
//   §4  a move inside its own ± reads `≈ ±N` (a dead edit's zero stays `≈`); the refine lane runs the latest ask only (a burst of
//       five refines runs two: the one in flight and the last); the real engine's timing is clarity.mjs's
//   §5  the keep sheet keeps the tapped chip — the report names what went in (`KEPT … → vault`) and the vault holds it; a chip of the
//       row a sheet edits opens its own sheet (the verb chip under an open cond sheet); a released backlog of cues is one burst
//       (≤ 3 cues), the exit's cue still plays
//
//   node web/tests/cut24.mjs [--shots dir]        (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { editRows } from "./lib/frame.mjs";

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
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: true }); };
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const camp = async () => { await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250); };
const section = (label) => page.evaluate((l) => { const s = [...document.querySelectorAll(".report .rsec")].find((x) => x.querySelector(".label")?.textContent === l); return s ? s.innerText.replace(/\s+/g, " ").trim().toLowerCase() : null; }, label);   // (labels and some chips are upper-cased by CSS)

try {
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=24`, { waitUntil: "domcontentloaded" });
  await camp();

  // ---- §3 / §4: the forge's move and `≈ ±N`, from the modules themselves (the same functions the sheets paint with)
  const moves = await page.evaluate(async () => {
    const forge = await import("/src/ui/forge.ts"), fc = await import("/src/ui/forecast.ts");
    const k = (n) => forge.kitMove({ label: "leather +1", price: 1400, affordable: true, ...n });
    return {
      armour: k({ depth: 9, delta: 0.07, pm: 0.03, death: -0.07, bank: 0.01 }),
      flat: k({ depth: 11, delta: 0.01, pm: 0.04, death: 0.07 }),
      bank: k({ depth: 8, delta: -0.02, pm: 0.03, bank: 0.06 }),
      none: k({ depth: 8 }),
      inside: fc.moveOf({ delta: 0.02, pm: 0.04 })?.text, zero: fc.moveOf({ delta: 0, pm: 0 })?.text, bare: fc.moveOf({ delta: 0 })?.text, out: fc.moveOf({ delta: 0.09, pm: 0.04 })?.text,
    };
  });
  check(moves.armour === "reach D9 +7 · death −7", `the forge's move reads as an edit's, the depth first ("${moves.armour}"; AL read \`leather +1 · death +7\`)`);
  check(moves.flat === "reach D11 same · death +7" && moves.bank === "reach D8 same · bank +6" && moves.none === null, `a flat depth still leads, the ends that clear their ± follow ("${moves.flat}" · "${moves.bank}" · unmeasured ${moves.none})`);
  check(moves.inside === "same" && moves.zero === "same" && moves.bare === "same" && moves.out === "+9", `a move inside its ± reads \`same\` (${moves.inside}); a dead edit's zero \`${moves.zero}\`; outside it the move (${moves.out})`);
  // the forge sheet paints the same form (the fake's kit, gold to show every step)
  {
    const r = await page.evaluate(async () => { const b = JSON.parse(window.__riddle.exportSave()); const e = JSON.parse(b.engine); e.lineage.gold = 5000; e.lineage.best_depth = Math.max(5, e.lineage.best_depth); b.engine = JSON.stringify(e); await window.__riddle.importSave(JSON.stringify(b)); return true; });
    await camp();
    const tile = page.locator(".cmd .tile[data-tile=forge]");
    if (r && await tile.count()) {
      await tile.click({ timeout: 5000 });
      await page.locator(".forge-details summary").click(); await page.locator(".forge-details button", {hasText:"Forecast"}).click();
      const t0 = Date.now(); let kit = [];
      while (Date.now() - t0 < 10_000) { kit = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .forge .forge-forecasts > div")].map((m) => m.textContent.replace(/\s+/g, " ").trim())); if (kit.length && kit.every((m) => !/…/.test(m))) break; await sleep(150); }
      check(kit.length > 0 && kit.every((m) => /^(Weapon|Armour|Pack) · (reach D\d+ ([+−]\d+|same)( · (bank|death) [+−]\d+)*|No change|Complete)$/i.test(m)), `the forge sheet's steps read \`reach D9 +7 · death −7\` (${kit.join(" | ")})`);
      await shot("cut24-forge");
      await page.keyboard.press("Escape"); await sleep(150);
    } else check(false, "the forge tile is carved with gold for a step");
  }

  // ---- §2: an absence's news — the newest runs' firsts, each once, no `differ` / `learned`; a watched run's own lines, its `differ` too
  {
    const n = await page.evaluate(async () => {
      const { newsLines } = await import("/src/ui/report.ts");
      const x = (news) => ({ carried: 0, keep_pct: 60, kept: 0, spent: 0, spent_on: [], text: "returned $0", news });
      const night = newsLines({ runs: 3, exits: [x([{ k: "record", text: "record: D9" }, { k: "learned", text: "learned 2" }]), x([{ k: "differ", text: "banked, last returned" }]), x([{ k: "first", text: "first: Warlord slain" }, { k: "record", text: "record: D9" }])] });
      const one = newsLines({ runs: 1, exits: [x([{ k: "differ", text: "deeper: D9, last D8" }])] });
      const old = newsLines({ runs: 1, exits: [x(undefined)] });
      const finds = newsLines({ runs: 1, exits: [x([{ k: "find", text: "new find: bow" }, { k: "find", text: "new find: amber potion?" }, { k: "situation", text: "first: the cage" }])] });
      const many = newsLines({ runs: 2, exits: [x(["a", "b", "c"].map((i) => ({ k: "find", text: `new find: ${i}` }))), x(["d", "e"].map((i) => ({ k: "find", text: `new find: ${i}` })))] });
      return { night: night.map((y) => y.text), one: one.map((y) => y.text), old: old.length, finds: finds.map((y) => y.text), many: many.map((y) => y.text) };
    });
    check(n.night.join(" | ") === "first: Warlord slain | record: D9" && n.one.join() === "deeper: D9, last D8" && n.old === 0, `the night's news: its firsts newest first, once each (${n.night.join(" | ")}); a watched run with nothing new: its one difference (${n.one.join()}); an older core: none`);
    check(n.finds.join(" | ") === "new find: bow, amber potion? | first: the cage" && n.many.join() === "new find: d, e, a +2", `finds read as one line, a night's capped at three (${n.finds.join(" | ")} · ${n.many.join()})`);
  }

  // ---- §4: the refine lane runs the latest ask only
  {
    const lane = await page.evaluate(async () => {
      const { latestOnly } = await import("/src/engine/lanes.ts");
      const ran = [];
      const run = (m, a) => new Promise((res) => { ran.push(`${m}:${a[0]}`); setTimeout(() => res(`${m}:${a[0]}`), 120); });
      const ask = latestOnly(run);
      const ps = [1, 2, 3, 4, 5].map((i) => ask("forecastRefine", [i]));
      const vs = ask("forecastVs", ["v"], "forecastVsRefined");
      const got = await Promise.all([...ps, vs]);
      return { ran, got };
    });
    check(lane.ran.join(" ") === "forecastRefine:1 forecastRefine:5 forecastVs:v" && lane.got.slice(1, 5).every((g) => g === "forecastRefine:5") && lane.got[0] === "forecastRefine:1",
      `a burst of five refines runs the one in flight and the latest (${lane.ran.join(" → ")}); the stale asks answer with the latest's`);
  }

  // ---- §5: a chip of the row a sheet edits opens its own sheet
  {
    await editRows(page); await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await camp();
    await page.locator(`.editor .row[data-i="0"] .chip.cond:not(.add)`).first().click({ timeout: 5000 }); await sleep(250);
    const first = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap")].map((w) => w.querySelector(".label")?.textContent));
    const v = await page.evaluate(() => { const r = document.querySelector(`.editor .row[data-i="0"] .chip.verb`).getBoundingClientRect(); return { x: r.x + r.width / 2, y: r.y + r.height / 2 }; });
    await page.mouse.click(v.x, v.y); await sleep(300);
    const then = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap")].map((w) => w.querySelector(".label")?.textContent));
    check(first.join() === "condition" && then.join() === "action", `the verb chip under the row's open cond sheet opens the verb sheet (${first.join()} → ${then.join() || "none"}; AK: "the chip tap didn't open the sheet")`);
    await page.keyboard.press("Escape"); await sleep(150);
  }

  // ---- §5: a released backlog of cues is one burst
  {
    const a = await page.evaluate(async () => {
      const au = window.__audio; au.arm?.(); document.body.click();
      await new Promise((r) => setTimeout(r, 200));
      const n0 = au.log.length; for (let i = 0; i < 12; i++) au.cue(i % 2 ? "hit" : "rule", { dmg: 3 });
      const burst = au.log.length - n0; au.cue("exit_bank"); const exit = au.log.length - n0 - burst;
      await new Promise((r) => setTimeout(r, 200)); const n1 = au.log.length; au.cue("slay");
      return { burst, exit, after: au.log.length - n1, muted: au.muted };
    });
    check(a.muted || (a.burst <= 3 && a.burst >= 1 && a.exit === 1 && a.after === 1), `twelve cues at once play as a burst of ${a.burst} (≤ 3); the exit's cue plays (${a.exit}); the next one later plays (${a.after})`);
  }

  // ---- §1: a `driven` exit on the report
  {
    await page.evaluate(() => {
      const r = window.__riddle, L = r.lineage;
      const row = { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "boss" } };
      const driven = { boss: "goblin_warlord", title: "Warlord", depth: 8, verdict: "no counter", defence: "shield wall", counter: "attack boss", row };
      const news = [{ k: "first", text: "first: the captive" }, { k: "driven", text: "driven off: Warlord" }];
      const exits = [{ carried: 80, keep_pct: 60, kept: 48, spent: 0, spent_on: [], text: "returned $48 · $80 carried · keeps 60% · no counter", driven, news, run_id: 1 }];
      r.go({ kind: "report", report: { elapsed_s: 600, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 1, exits } });
    });
    await sleep(300);
    await page.locator(".report .details-fold").click();
    const d = await section("counter");
    // (Cut 28 §2: the tablet sits above the report's fold and carries the drive-off's `verdict` chip always)
    check(/^counter\s*warlord repelled him · shield wall\s*try: attack boss(\s*verdict)?$/.test(d), `a boss that drove him off reads its counter ("${d}")`);
    const lead = await page.evaluate(() => { const n = document.querySelector(".report .news"), t = document.querySelector(".report .tiles"); return { lines: [...(n?.querySelectorAll(".news-line") ?? [])].map((x) => x.textContent), before: !!n && !!t && !!(n.compareDocumentPosition(t) & Node.DOCUMENT_POSITION_FOLLOWING), lead: n?.querySelector(".news-line.lead")?.textContent ?? null }; });
    check(!lead.before && lead.lines.join(" | ") === "first: the captive | repelled by Warlord", `news inside Details keeps first discoveries before drive-offs (${lead.lines.join(" | ") || "none"})`);
    await shot("cut24-driven");
    await page.locator(".report .driven-line").first().click({ timeout: 5000 });
    await camp();
    const r1 = await page.evaluate(() => { const r = window.__riddle.rules.rows[0]; return `${r.conds.map((c) => c.k + (c.t ?? "")).join(" ")} → ${r.verb.v} ${r.verb.a ?? ""}`; });
    check(r1 === "foe_tagboss → attack boss", `a tap writes the counter at R1 (${r1})`);
  }

  // ---- §1 / §2: a night's report — the drive-offs counted apart (`driven`), each exit line led by its run's news
  {
    await page.evaluate(() => {
      const r = window.__riddle, L = r.lineage;
      const x = (kept, news, extra = {}) => ({ carried: kept * 2, keep_pct: 60, kept, spent: 0, spent_on: [], text: `returned $${kept} · $${kept * 2} carried · keeps 60%`, news, ...extra });
      const exits = [x(20, [{ k: "record", text: "record: D9" }]), x(30, [{ k: "differ", text: "banked, last returned" }]), x(40, [{ k: "driven", text: "driven off: Warlord" }], { text: "returned $40 · $80 carried · keeps 60% · no counter" })];
      r.go({ kind: "report", report: { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 3, driven: 1, exits } });
    });
    await sleep(300);
    await page.locator(".report .details-fold").click();
    const t = await page.evaluate(() => ({ tiles: [...document.querySelectorAll(".report .tiles .tile")].map((x) => `${x.querySelector(".label")?.textContent} ${x.querySelector("b")?.textContent}`), leads: [...document.querySelectorAll(".report .exit-lines .ledger-btn")].map((b) => b.querySelector(".news-lead")?.textContent ?? "-") }));
    check(t.tiles.includes("runs returned 2/3") && t.tiles.includes("repelled 1/3"), `a drive-off is counted apart from the returns (${t.tiles.join(" · ")})`);
    check(t.leads.join(" | ") === "repelled by Warlord | - | record: D9", `each exit line leads with its run's news, newest first (${t.leads.join(" | ")})`);
  }

  // ---- §5 (core: `ForecastDepth.boss`, `ForecastTry.met`): the boss is named on the floor he is met on, his counter read there
  {
    await page.evaluate(async () => { const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine); e.lineage.best_depth = 8; b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b)); });
    await camp(); await sleep(1500);   // the camp's own first pass and refine land first
    const f = await page.evaluate(() => {
      const r = window.__riddle, b = r.lastForecast;
      const row = { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "boss" } };
      const depths = Array.from({ length: 9 }, (_, i) => ({ depth: i + 1, reach: i < 7 ? 0.9 - i * 0.05 : i === 7 ? 0.5 : 0, pm: 0.03 }));
      depths[7].boss = "goblin_warlord"; depths[8].wall = "goblin_warlord"; depths[8].try = { row, text: "attack boss", met: 8 };
      const x = { ...b, depths, known_to: 9, refined: true, start: 1 }; r.lastForecast = x; for (const fn of r.fcListeners) fn(x);
      return true;
    });
    await sleep(150);
    const notch = await page.evaluate(() => ({ d8: document.querySelector('.shaft .notch[data-d="8"] .dl')?.textContent ?? null, d9: document.querySelector('.shaft .notch[data-d="9"] .dl')?.textContent ?? null }));
    await page.locator(".shaft").click({ timeout: 5000 }); await sleep(250);
    const bars = await page.evaluate(() => Object.fromEntries([...document.querySelectorAll(".panel .fc-bars .bar")].map((b) => [b.querySelector(".d")?.textContent, { text: b.textContent.replace(/\s+/g, " ").trim(), try: !!b.querySelector(".try") }])));
    check(f && notch.d8 === "D8 · warlord" && notch.d9 === "D9 · behind warlord", `the shaft names him on his floor and his wall below (QA 0c6e126, qaY: the wall under his named floor reads \`· behind warlord\`, never a second warlord) (${notch.d8} | ${notch.d9})`);
    check(bars.D8?.try && /try: attack boss/.test(bars.D8.text) && /warlord/.test(bars.D8.text) && !bars.D9?.try && /behind warlord/.test(bars.D9?.text ?? ""), `the panel reads his counter on D8, where he is met (D8: "${bars.D8?.text}" · D9: "${bars.D9?.text}")`);
    await shot("cut24-boss-floor");
    await page.keyboard.press("Escape"); await sleep(150);
  }

  // ---- §5: the keep sheet keeps the tapped chip (a fake run that returns with items: seed 13, return at D3)
  {
    const rules = encodeURIComponent("depth>=3 → return\nfoes>=1 → attack nearest");
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=13&rules=${rules}&autosend=1&early=0&speed=fast`, { waitUntil: "domcontentloaded" });
    await waitFor((s) => s?.booted && s.screen === "watch", "the watch");
    const t0 = Date.now(); let s = await state();
    while (s?.screen === "watch" && Date.now() - t0 < 120_000) { await page.locator(".cmd .hud-btn", { hasText: "▶▶|" }).click({ timeout: 1000 }).catch(() => {}); await sleep(300); s = await state(); }
    s = await waitFor((x) => x && x.screen !== "watch", "the keep sheet", 30_000);
    const chips = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chips .chip.item")].map((c) => c.firstChild?.textContent?.trim() ?? ""));
    check(s.screen === "exit" && chips.length >= 2, `the run came home on the keep sheet (${s.screen}, ${chips.length} chips)`);
    if (s.screen === "exit" && chips.length) {
      const i = chips.length - 1, tapped = chips[i];
      const vault0 = await page.evaluate(() => window.__riddle.lineage.vault.map((v) => v.id));
      await page.locator(".sheet-wrap .chips .chip.item").nth(i).click({ timeout: 5000 }); await sleep(150);
      const on = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .chips .chip.item")].map((c) => c.classList.contains("on")));
      await shot("cut24-keep-sheet");
      await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 });
      await waitFor((x) => x && x.screen === "report" && !x.busy, "the report", 30_000); await sleep(300);
      const kept = await section("kept"), salv = await section("salvaged");
      const gained = await page.evaluate((v0) => window.__riddle.lineage.vault.filter((v) => !v0.includes(v.id)).map((v) => v.label), vault0);
      const twin = chips.filter((c) => c === tapped).length > 1;
      check(on[i] && on.filter(Boolean).length === 1 && gained.length === 1, `the tapped chip is the one pick and the vault took it (${tapped} → ${gained.join(", ") || "nothing"})`);
      check(!!kept && gained.length === 1 && kept.includes(gained[0].replace(/_/g, " ").toLowerCase()) && / → storage/.test(kept), `the report names what went in ("${kept}")`);
      check(twin || !salv || !salv.includes(tapped.toLowerCase()), `SALVAGED lists the tapped chip only for a twin of it ("${salv ?? "none"}", twin ${twin})`);
      await shot("cut24-kept");
    }
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut24: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut24: ok (${out.length} checks)`);
