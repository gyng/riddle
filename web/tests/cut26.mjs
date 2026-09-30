#!/usr/bin/env node
// Cut 26 client gates (docs/CUT26.md), on the fake engine, headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU):
//   §2  the route chip line above the rows appears only after the fact `fork D5` (`⑂ D5 burrows`); its sheet is the fork tablet — both
//       stairs priced for the set (`routeForecast()`: `burrows D8 61%` · `fens D8 34%`), the set's stair lit; a tap writes the route (a set
//       edit: the engine gets `route: [5]`, the forecast reprices, the chips read `⑂ D5 fens`); a set imported with a route keeps it
//       (export round-trip); the `in: <biome>` cond in the picker (open ones selectable, locked ones `⊘` with their gate); the shaft names
//       each band's lane (`D5 fens`) and hangs the untaken lane as a frontier (`burrows · D5 · untried`); the start sheet lists (lane, depth)
//       pairs (`D9 crypt`); the report names each band's lane (`D5–8 · the Fens`); the renderer shows a fork floor's two stairs, the
//       route's stair lit (`Snapshot.stairs`); the watch's `TWO STAIRS` on the fork fact
//   §6  a `gap` whose unpatched replays mostly survive reads its count beside the stamp (`10/12 live unpatched`) (`Death.lean`); a drive-off opens its verdict
//       (the seal `driven`, the counter tablet, the gem writes it; the report's drive-off line has `verdict`); a patch's reach is from→to
//       (`reach D5 80→4%`), never a signed delta; a locked cond in a row is marked (`⊘`, `locked · ◆2`); the `repeat` plate covers no
//       button (the forge's second tap in place: cut25.mjs)
//   control rater AR: the ogre's sprite is cut to size by area (no checkerboard speckle)
//
//   node web/tests/cut26.mjs [--shots dir]        (part of `pnpm test` in web/)
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
const shot = async (name) => { if (shots) await page.screenshot({ path: resolve(shots, `${name}.png`), fullPage: false }); };
const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(80); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const camp = async () => { await waitFor((s) => s?.booted && s.screen === "camp", "camp"); await sleep(250); };
const txt = (sel) => page.evaluate((sel) => document.querySelector(sel)?.textContent.replace(/\s+/g, " ").trim() ?? null, sel);
// the lineage as the engine would send it, with facts added, and the camp repainted on it
const withLineage = (patch) => page.evaluate((patch) => { const r = window.__riddle; r.lineage = { ...r.lineage, ...patch, facts: [...(r.lineage.facts ?? []), ...(patch.facts ?? [])] }; r.go({ kind: "camp" }); }, patch);

try {
  // ---- §2: the route chip line — only after the fork fact (a fresh lineage has seen none)
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=26`, { waitUntil: "domcontentloaded" });
  await camp();
  {
    await withLineage({ best_depth: 9, gold: 400 }); await sleep(300);
    const before = await page.evaluate(() => { const e = document.querySelector(".camp .route-line"); return { shown: e ? !e.hidden : false, forks: (window.__riddle.lineage.forks ?? []).length }; });
    check(!before.shown && before.forks === 0, `no route line before a fork is seen (${before.shown ? "shown" : "hidden"}; forks ${before.forks})`);
  }
  // the fake's descent forks at D4 (`?fake_fork=1`: the D4 fork seen at boot, both lanes entered)
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=26&fake_fork=1`, { waitUntil: "domcontentloaded" });
  await camp();
  {
    await page.evaluate(async () => { const r = window.__riddle, b = JSON.parse(r.exportSave()), e = JSON.parse(b.engine); Object.assign(e.lineage, { best_depth: 7, gold: 400 }); b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b)); });
    await camp(); await sleep(300);
    const line = await txt(".camp .route-line");
    const above = await page.evaluate(() => { const l = document.querySelector(".camp .route-line"), r = document.querySelector(".camp .editor .row"); return !!l && !!r && l.getBoundingClientRect().bottom <= r.getBoundingClientRect().top + 1; });
    check(line === "route D4 burrows" && above, `after \`fork:4\` the chip line sits above the rows ("${line}"; above the first rule: ${above})`);
    await shot("cut26-route-line");

    // the fork tablet: both stairs priced for this set (the core's `forkForecast`), the set's lit
    await page.evaluate(() => { const r = window.__riddle; r.__setRules = []; const set0 = r.engine.setRules.bind(r.engine); r.engine.setRules = async (s) => { r.__setRules.push(JSON.parse(JSON.stringify(s))); return set0(s); }; });
    await page.locator(".camp .route-line").click({ timeout: 5000 });
    await page.waitForFunction(() => [...document.querySelectorAll(".sheet-wrap .route-opt")].every((b) => /%/.test(b.textContent)), null, { timeout: 15_000 }).catch(() => {});
    const sheet = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .route-opt")].map((b) => ({ t: b.textContent.replace(/\s+/g, " ").trim(), on: b.classList.contains("on") })));
    check(sheet.length === 2 && /^burrows D5 <?\d+%…?$/.test(sheet[0].t) && sheet[0].on && /^fens D5 <?\d+%…?$/.test(sheet[1].t) && !sheet[1].on,   // (`…`: a first-pass number, QA 308f045)
      `the fork tablet prices both stairs for this set, the set's lit (${sheet.map((s) => `${s.t}${s.on ? " ●" : ""}`).join(" · ")})`);
    await shot("cut26-fork-tablet");
    const seq0 = await page.evaluate(() => window.__riddle.forecastSeq);
    await page.locator(".sheet-wrap .route-opt[data-far='1']").click({ timeout: 5000 });
    await page.waitForFunction((s0) => window.__riddle.forecastSeq > s0, seq0, { timeout: 15_000 }).catch(() => {});
    await sleep(400);
    const after = await page.evaluate(() => ({ route: window.__riddle.rules.route, sent: window.__riddle.__setRules.at(-1)?.route, seq: window.__riddle.forecastSeq, line: document.querySelector(".camp .route-line")?.textContent.replace(/\s+/g, " ").trim() }));
    check(JSON.stringify(after.route) === "[4]" && JSON.stringify(after.sent) === "[4]" && after.seq > seq0 && after.line === "route D4 fens",
      `a tap writes the route — the engine gets route [${after.sent}], the forecast reprices (${seq0} → ${after.seq}), the chips read "${after.line}"`);
    const vs = await page.evaluate(() => { const r = window.__riddle; return { pending: r.vsPending(), shown: !!r.vsShown(), line: document.querySelector(".shaft-vs-host")?.textContent.replace(/\s+/g, " ").trim() ?? "" }; });
    check(vs.pending || vs.shown, `the route change reads as an edit against the sent set (${vs.shown ? "move shown" : "vs sent …"}: "${vs.line}")`);
    // the export carries the route; importing it restores it
    const rt = await page.evaluate(async () => {
      const r = window.__riddle, n0 = r.rules.rows.length, text = await r.engine.exportRules();
      r.rules.route = undefined; delete r.rules.route; r.rulesChanged();
      await r.setRulesText(text);
      return { text, route: r.rules.route, rows: r.rules.rows.length === n0 && n0 > 0 };
    });
    check(JSON.stringify(rt.route) === "[4]" && rt.rows, `export round-trips the route ("${rt.text.split("\n")[0]}" → route ${JSON.stringify(rt.route)})`);
  }

  // ---- §2: the shaft names each band's lane; the untaken lane hangs as a frontier
  {
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await sleep(700);
    await page.waitForFunction(() => (window.__riddle.lastForecast?.depths ?? []).some((d) => d.biome), null, { timeout: 10_000 }).catch(() => {});
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })); await sleep(400);
    const sh = await page.evaluate(() => ({ dbg: (window.__riddle.lastForecast?.depths ?? []).map((d) => `${d.depth}${d.biome ?? ""}`).join(","), lane4: document.querySelector(".shaft .notch[data-d='4'] .lane")?.textContent.trim(), lane6: document.querySelector(".shaft .notch[data-d='6'] .lane")?.textContent.trim(),
      front: document.querySelector(".shaft .notch[data-d='4'] .frontier")?.textContent.trim() }));
    check(sh.lane4 === "fens" && sh.lane6 === "burrows" && sh.front === "or burrows", `the shaft on route [4]: D4 ${sh.lane4}, D6 ${sh.lane6}; the lane not taken "${sh.front}" (entered: no \`?\`) [${sh.dbg}]`);
    await shot("cut26-shaft");
    await page.evaluate(() => { const r = window.__riddle; delete r.rules.route; r.rulesChanged(); r.lineage = { ...r.lineage, facts: r.lineage.facts.filter((f) => f !== "biome:fens") }; r.go({ kind: "camp" }); }); await sleep(700);
    const f2 = await txt(".shaft .notch[data-d='4'] .frontier");
    check(f2 === "or fens · untried", `on the near stair the far lane, never entered, is the frontier ("${f2}")`);
  }

  // ---- §2: the start sheet lists lit (lane, depth) pairs — this route's selectable, another route's dim
  {
    await withLineage({ waystones: [4, 6], start: 1, lanes: [{ depth: 4, lane: "burrows", current: true }, { depth: 6, lane: "fens", current: true }, { depth: 6, lane: "burrows", route: [4], current: false }] }); await sleep(300);
    const tab = page.locator(".camp .start-tab");
    if (await tab.count() && await tab.isVisible()) {
      await tab.click({ timeout: 5000 }); await sleep(400);
      const opts = await page.evaluate(() => [...document.querySelectorAll(".sheet-wrap .start-opt")].map((b) => `${b.querySelector(".num")?.textContent}${b.querySelector(".lane")?.textContent ?? ""}${b.classList.contains("other-lane") ? " (other)" : ""}`));
      check(opts.join(",") === "D1,D4 burrows,D6 fens,D6 burrows (other)", `the start sheet lists (lane, depth) pairs (${opts.join(" · ")})`);
      await shot("cut26-start");
      await page.keyboard.press("Escape"); await sleep(150);
    } else check(false, "the start tablet is carved once a waystone is lit");
  }

  // ---- §2: the `in: <biome>` cond in the picker — open ones selectable, locked ones marked with their gate
  {
    // (the fake opens `in:` for each biome entered — burrows, fens — and a lane a seen fork offers but no hero entered is locked)
    await page.evaluate(() => { const r = window.__riddle; if (!(r.vocab.locked ?? []).some((l) => l.cond.k === "in")) r.vocab = { ...r.vocab, locked: [...(r.vocab.locked ?? []), { cond: { k: "in", t: "crypt" }, needs: "enter crypt" }] }; r.go({ kind: "camp" }); });
    await camp(); await editRows(page); await sleep(200);
    if (!(await page.locator(".editor .row .chip.cond").count())) { await shot("cut26-debug-editor"); console.log(await page.evaluate(() => document.querySelector(".editor")?.outerHTML.slice(0, 600))); }
    await page.locator(".editor .row").first().locator(".chip.cond").first().click({ timeout: 5000 }); await sleep(250);
    const p = await page.evaluate(() => ({ open: [...document.querySelectorAll(".sheet-wrap button.chip.cond")].map((b) => b.textContent.trim()).filter((t) => t.startsWith("in:")),
      locked: [...document.querySelectorAll(".sheet-wrap span.chip.cond.locked")].map((b) => b.textContent.replace(/\s+/g, " ").trim()).filter((t) => t.includes("in:")) }));
    check(p.open.includes("in: burrows") && p.locked.length >= 1 && /^⊘ in: [a-z]+ ?enter [a-z]+$/.test(p.locked[0]), `the cond picker offers \`${p.open.join(" · ")}\` and shows \`${p.locked.join(" · ")}\` locked, not a button`);
    await shot("cut26-in-cond");
    await page.keyboard.press("Escape"); await sleep(150);
  }

  // ---- §6: a locked cond in a row is marked (a text import, an old set) — on its chip and on the tablet
  {
    const m = await page.evaluate(async () => {
      const r = window.__riddle;
      const lk = (r.vocab.locked ?? []).find((l) => l.cond.k === "on_see") ?? { cond: { k: "on_see", t: "hunger" }, needs: "◆2" };
      if (!(r.vocab.locked ?? []).some((l) => l.cond.k === lk.cond.k)) r.vocab = { ...r.vocab, locked: [...(r.vocab.locked ?? []), lk] };
      r.sets[r.active] = { ...r.sets[r.active], rows: [{ conds: [{ ...lk.cond }], verb: { v: "descend" }, origin: "player" }, ...r.rules.rows] };
      r.go({ kind: "camp" }); await new Promise((f) => setTimeout(f, 400));
      const chip = document.querySelector(".editor .row .chip.cond.locked-in")?.textContent.replace(/\s+/g, " ").trim();
      const mark = document.querySelector(".editor .row .inert-mark")?.textContent.trim();
      return { chip, mark, needs: lk.needs };
    });
    check(!!m.chip && m.chip.startsWith("⊘") && m.mark === `⊘ locked · ${m.needs}`, `a row holding a locked cond is marked: chip "${m.chip}", tablet "${m.mark}"`);
    await shot("cut26-locked-row");
    await page.evaluate(() => { const r = window.__riddle; r.sets[r.active].rows.shift(); r.go({ kind: "camp" }); });
    await camp();
  }

  // ---- §6: the stamp agrees with its counts; reach moves from→to
  {
    const rows = [{ conds: [{ k: "hp<", n: 30 }], verb: { v: "drink", a: "heal" }, origin: "player" }, { conds: [{ k: "foes>=", n: 1 }], verb: { v: "attack", a: "nearest" }, origin: "player" }];
    const death = (baseline) => ({ run_id: 0, depth: 4, cause: "goblin", margin: "", verdict: "gap", baseline, replays: 12, trace: { turns: [] }, morgue: "m", rules: { rows },
      patches: [{ row: { conds: [{ k: "hp<", n: 50 }], verb: { v: "drink", a: "heal" } }, insert_at: 0, survive: 11 / 12, forecast_delta: -0.76, forecast_depth: 5, forecast_pm: 0.03 }] });
    await page.evaluate(() => { const r = window.__riddle; r.lastForecast = { ...(r.lastForecast ?? { causes: [] }), known_to: 9, depths: Array.from({ length: 9 }, (_, i) => ({ depth: i + 1, reach: i + 1 === 5 ? 0.8 : 0.5 })) }; });
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d }), death(10 / 12));
    await waitFor((x) => x?.screen === "death", "the lucky gap"); await sleep(300);
    const a = await page.evaluate(() => ({ seal: document.querySelector(".death-line .verdict")?.textContent, lean: document.querySelector(".death-line .lean")?.textContent, surv: document.querySelector("button.patch .surv")?.textContent, reach: document.querySelector("button.patch .delta")?.textContent.replace(/\s+/g, " ").trim() }));
    check(a.seal === "you died" && a.lean === "10/12 replays survive" && /was 10\/12/.test(a.surv ?? ""), `a gap 10 of 12 unpatched replays survive reads \`${a.seal} · ${a.lean}\` beside "${a.surv}" (the stamp and its counts agree)`);
    check(a.reach === "reach D5 80→4%", `a patch's reach reads from→to ("${a.reach}"; AP: \`reach D5 −76\`)`);
    await shot("cut26-dice-lean");
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d }), death(2 / 12)); await sleep(300);
    const b = await txt(".death-line .verdict"), bl = await txt(".death-line .lean");
    check(b === "you died" && bl === null, `a gap 2 of 12 unpatched replays survive reads \`${b}\`, no lean`);
    // the core's own `lean` stands whatever the counts
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: { ...d, lean: "dice" } }), death(5 / 12)); await sleep(300);
    const c = await txt(".death-line .lean");
    check(/replays survive$/.test(c ?? ""), `the core's \`lean: dice\` reads beside the stamp ("${c}")`);
  }

  // ---- Cut 26 (core, risks): a `route` death names the stair (`D5 fens`), its lead tablet takes the other (`take burrows`)
  {
    await page.evaluate(() => { const r = window.__riddle; r.rules.route = [5]; });
    await page.evaluate(() => window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 7, cause: "bloat", margin: "", verdict: "route", baseline: 3 / 12, replays: 12, trace: { turns: [] }, morgue: "m",
      route_cause: { fork: 5, taken: "fens", other: "burrows", route: [], survive: 9 / 12 }, patches: [] } }));
    await waitFor((x) => x?.screen === "death", "the route death"); await sleep(300);
    const d = await page.evaluate(() => ({ cause: document.querySelector(".death-line .cause")?.textContent, seal: document.querySelector(".death-line .verdict")?.textContent,
      lead: document.querySelector("button.patch.route-fix")?.textContent.replace(/\s+/g, " ").trim(), gem: document.querySelector(".gem.patch-gem .gem-n")?.textContent }));
    check(d.cause === "bloat · D7 · D5 fens" && d.seal === "route" && /take burrows/.test(d.lead ?? "") && /survives 9\/12 · was 3\/12/.test(d.lead ?? "") && d.gem === "9/12",
      `a route death: "${d.cause}" · ${d.seal}; lead "${d.lead}", gem ${d.gem}`);
    await shot("cut26-route-death");
    await page.locator(".patch-gem").click({ timeout: 5000 }); await sleep(400);
    const w = await page.evaluate(() => ({ screen: window.__riddle.screen, route: window.__riddle.rules.route }));
    check(w.screen === "camp" && !w.route?.length, `the gem takes the other stair (route ${JSON.stringify(w.route ?? [])}, ${w.screen})`);
  }

  // ---- §6: a drive-off opens its verdict
  {
    const driven = { boss: "goblin_warlord", title: "Warlord", depth: 8, verdict: "no counter", defence: "shield wall", counter: "attack boss", row: { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "attack", a: "tag:boss" } } };
    const line = { carried: 297, keep_pct: 60, kept: 0, spent: 0, spent_on: [], text: "driven $0 · $297 lost · by Warlord", run_id: 7, driven };
    await page.evaluate((line) => window.__riddle.go({ kind: "report", report: { elapsed_s: 0, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [],
      xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, returned: 1, driven: 1, exits: [line], reel: [] } }), line);
    await waitFor((x) => x?.screen === "report", "the report"); await sleep(300);
    const chip = page.locator(".report .verdict-chip");
    const has = await chip.count();
    if (has) { await chip.first().click({ timeout: 5000 }); await waitFor((x) => x?.screen === "death", "the drive-off's verdict"); await sleep(300); }
    else await page.evaluate(async (line) => { const { drivenDeath } = await import("/src/ui/death.ts"); window.__riddle.go({ kind: "death", death: drivenDeath(line, 7) }); }, line);
    await sleep(300);
    const d = await page.evaluate(() => ({ cause: document.querySelector(".death-line .cause")?.textContent, seal: document.querySelector(".death-line .verdict")?.textContent,
      tab: document.querySelector("button.patch.driven-line")?.textContent.replace(/\s+/g, " ").trim(), gem: document.querySelector(".gem.patch-gem .gem-n")?.textContent, ledger: document.querySelector(".death .ledger-line")?.textContent }));
    check(has > 0 && d.seal === "repelled" && d.cause === "Warlord · D8 · shield wall" && /try: attack boss/.test(d.tab ?? "") && d.gem === "write",
      `a drive-off opens its verdict from the report's line (chip ${has}): "${d.cause}" · ${d.seal}, tablet "${d.tab}", gem \`${d.gem}\`, "${(d.ledger ?? "").slice(0, 30)}"`);
    await shot("cut26-driven");
    // (a kept verdict; the gem writes the counter at the top)
    const rows0 = await page.evaluate(() => window.__riddle.rules.rows.length);
    await page.locator(".patch-gem").click({ timeout: 5000 }); await sleep(400);
    const w = await page.evaluate(() => ({ screen: window.__riddle.screen, r1: window.__riddle.rules.rows[0]?.verb.a, n: window.__riddle.rules.rows.length }));
    check(w.screen === "camp" && w.r1 === "tag:boss" && w.n === rows0 + 1, `the gem writes the counter as R1 (${w.screen}, R1 ${w.r1}, ${rows0} → ${w.n} rows)`);
    await page.evaluate(() => { const r = window.__riddle; r.sets[r.active].rows.shift(); r.go({ kind: "camp" }); }); await camp();
  }

  // ---- §6: the repeat plate covers no button
  {
    await page.evaluate(() => { const r = window.__riddle; r.engine.setRestock = async (on) => ({ ...r.lineage, repeat: on }); r.lineage = { ...r.lineage, gold: 300, marks: 3, repeat: true, repeat_kinds: ["heal"], repeat_gold: 120 }; r.go({ kind: "camp" }); });
    await sleep(400);
    const o = await page.evaluate(() => {
      const b = document.querySelector(".cmd .repeat-badge"); if (!b) return null;
      const r = b.getBoundingClientRect();
      // a button's rect as the player sees it: clipped by every scrolling/clipping box it sits in (a tablet scrolled under the well's edge
      // is not on screen there)
      const seen = (x) => { let q = x.getBoundingClientRect(), L = q.left, T = q.top, R = q.right, B = q.bottom;
        for (let p = x.parentElement; p; p = p.parentElement) { const cs = getComputedStyle(p); if (cs.overflowX !== "visible" || cs.overflowY !== "visible") { const c = p.getBoundingClientRect(); L = Math.max(L, c.left); T = Math.max(T, c.top); R = Math.min(R, c.right); B = Math.min(B, c.bottom); } }
        return { left: L, top: T, right: R, bottom: B }; };
      const hit = [...document.querySelectorAll("button, .tile, [role=button]")].filter((x) => x !== b && !x.contains(b) && !b.contains(x) && x.getClientRects().length).filter((x) => {
        const q = seen(x); return q.right - q.left > 0.5 && q.bottom - q.top > 0.5 && r.left < q.right - 0.5 && r.right > q.left + 0.5 && r.top < q.bottom - 0.5 && r.bottom > q.top + 0.5;
      }).map((x) => x.dataset.tile ?? x.className);
      const own = b.closest(".tile").getBoundingClientRect();
      return { hit, clear: r.bottom <= own.top + 0.5, text: b.textContent };
    });
    check(!!o && o.hit.length === 0 && o.clear, `the repeat plate ("${o?.text}") covers no button${o?.hit.length ? `: ${o.hit.join(", ")}` : ""}, its own tile's face clear`);
    await shot("cut26-repeat");
  }

  // ---- §2: the report names each band's lane
  {
    await page.evaluate(() => { const r = window.__riddle; r.rules.route = [5]; r.lineage = { ...r.lineage, forks: [] , facts: [...r.lineage.facts.filter((f) => !/^fork/.test(f)), "fork:5"] }; });
    await page.evaluate(() => window.__riddle.go({ kind: "report", report: { elapsed_s: 28800, runs: 16, sampled: false, deepest: 11, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [],
      xp: { class: "fighter", gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 16, reel: [] } }));
    await waitFor((x) => x?.screen === "report", "the night's report"); await sleep(300);
    const lanes = await page.evaluate(() => [...document.querySelectorAll(".report .lanes .lane-line")].map((l) => l.textContent.replace(/\s+/g, " ").trim()));
    check(lanes.join(" | ") === "D5–8 · the Fens | D9–13 · the Burrows", `the report names each band's lane (${lanes.join(" | ")})`);
    await shot("cut26-report-lanes");
    await page.evaluate(() => { const r = window.__riddle; delete r.rules.route; r.go({ kind: "camp" }); }); await camp();
  }

  // ---- §2: the watch — `TWO STAIRS` on the fork fact; the renderer's two stairs, the route's lit
  {
    const w = await page.evaluate(async () => {
      const { createViewer } = await import("/src/render/index.ts");
      const snap = await window.__riddle.engine.send();
      const host = document.createElement("div"); Object.assign(host.style, { position: "fixed", left: "0", top: "0", width: "400px", height: "600px", zIndex: 99, background: "#000" });
      const canvas = document.createElement("canvas"); Object.assign(canvas.style, { width: "400px", height: "600px", display: "block" }); host.appendChild(canvas); document.body.appendChild(host);
      const s = JSON.parse(JSON.stringify(snap)), hx = s.hero.x, hy = s.hero.y;
      const at = [[hx + 2, hy], [hx - 2, hy]].map(([x, y]) => [Math.max(1, Math.min(s.w - 2, x)), y]);
      s.tiles = s.tiles.map((t) => t === "stairs_down" ? "floor" : t);
      for (const [x, y] of at) { const i = y * s.w + x; s.tiles[i] = "stairs_down"; s.seen[i] = true; s.visible[i] = true; }
      // the floor's own down stairs are the route's (`taken`); the other is drawn at (x, y) beside it (`Snapshot.fork`)
      const [x1, y1] = at[1]; s.tiles[y1 * s.w + x1] = "floor";
      s.fork = { depth: 5, taken: "fens", other: "burrows", x: x1, y: y1 };
      const v = createViewer(canvas); v.load(s); v.setSpeed(0);
      await new Promise((f) => setTimeout(f, 1500));
      const drawn = v.debugStairs?.() ?? [];
      const plates = [...document.querySelectorAll(".rtag.stair")].filter((e) => e.style.display !== "none").map((e) => `${e.textContent}${e.classList.contains("taken") ? "*" : ""}`);
      window.__c26 = { v, host };
      return { drawn: drawn.length, taken: drawn.filter((d) => d.taken).map((d) => d.text), plates };
    });
    check(w.drawn === 2 && w.taken.join() === "fens" && w.plates.sort().join(",") === "burrows,fens*", `a fork floor draws two stairs, each with its lane, the route's lit (${w.drawn} drawn; plates ${w.plates.join(", ")})`);
    await shot("cut26-two-stairs");
    await page.evaluate(() => { window.__c26.v.dispose(); window.__c26.host.remove(); });
    // the watch's callout on the fork fact (the core's events: the fact, then the stairs)
    const c = await page.evaluate(async () => {
      const r = window.__riddle; r.watchMode = "one"; r.go({ kind: "watch" });
      await new Promise((f) => setTimeout(f, 800));
      return null;
    });
    void c;
    const seen = await page.evaluate(() => new Promise((res) => {
      const t0 = performance.now(); const words = new Set();
      const mo = new MutationObserver(() => { for (const el of document.querySelectorAll(".watch .ticker, .watch .callout")) { const t = el.textContent.trim().toLowerCase(); if (t) words.add(t); } });
      mo.observe(document.body, { subtree: true, childList: true, characterData: true });
      // feed the fact through the engine's next step (the fake's step result carries it)
      const E = window.__riddle.engine, step0 = E.step.bind(E); let fed = false;
      E.step = async (n) => { const res = await step0(n); if (!fed && res.events.length) { fed = true; const t = res.events[0].t; res.events.unshift({ t, k: "fact", fact: "fork D5" }); } return res; };
      const poll = () => { if (words.has("two stairs") || performance.now() - t0 > 12_000) { mo.disconnect(); E.step = step0; res([...words]); } else setTimeout(poll, 100); };
      poll();
    }));
    check(seen.includes("two stairs"), `the watch calls \`TWO STAIRS\` on the fork fact (${seen.slice(0, 6).join(" · ")})`);
    await page.evaluate(() => window.__riddle.go({ kind: "camp" })).catch(() => {});
  }
  // ---- control rater AR ("the ogre drew as a checkerboard blob"): a painted master is cut to its runtime size by area, not by every
  //      other texel — the ogre's hide reads as a surface (speckle: the share of neighbouring texels whose luminance jumps > 40)
  {
    const sp = await page.evaluate(async () => {
      const { Atlas } = await import("/src/render/atlas.ts");
      const a = new Atlas(); if (!(await a.load("/art/atlas.json"))) return null;
      const lum = (d, i) => 0.299 * d[i] + 0.587 * d[i + 1] + 0.114 * d[i + 2];
      const speckle = (d, w, h) => { let n = 0, j = 0; for (let y = 0; y < h; y++) for (let x = 0; x + 1 < w; x++) { const i = (y * w + x) * 4; if (d[i + 3] && d[i + 7]) { n++; if (Math.abs(lum(d, i) - lum(d, i + 4)) > 40) j++; } } return n ? j / n : 0; };
      const s = a.entity("ogre"), box = a.sprite.ctx.getImageData(s.x, s.y, s.w, s.h).data;
      const res = await fetch("/art/atlas.json").then((r) => r.json()), f = res.frames.ogre;
      const img = await new Promise((ok) => { const i = new Image(); i.onload = () => ok(i); i.src = "/art/atlas.png"; });
      const c = document.createElement("canvas"); c.width = s.w; c.height = s.h; const x = c.getContext("2d"); x.imageSmoothingEnabled = false; x.drawImage(img, f.x, f.y, f.w, f.h, 0, 0, s.w, s.h);
      return { box: speckle(box, s.w, s.h), nearest: speckle(x.getImageData(0, 0, s.w, s.h).data, s.w, s.h), w: s.w, h: s.h };
    });
    check(!!sp && sp.box < sp.nearest * 0.85, `the ogre is cut down by area: speckle ${sp ? Math.round(sp.box * 100) : "?"} % vs nearest ${sp ? Math.round(sp.nearest * 100) : "?"} % (${sp?.w}×${sp?.h})`);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut26: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut26: ok (${out.length} checks)`);
