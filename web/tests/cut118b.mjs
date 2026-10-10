#!/usr/bin/env node
// Cut 118 (b) — the client shows and uses the Cut 118 core (docs/CUT118_IDLE_LESSONS.md, both owner amendments); headless,
// tools/browser.mjs, the fake engine and the ui modules direct. No added required tap: every chip has its default.
//   death   — the memorial leads (`+3 Legacy · Mother try 4 · +8%`), then the cause; the epitaph; `other heirs` a closed fold whose card
//             swaps by `setTrait(chip)` — never a prompt.
//   yard    — the town's graveyard: a stone per fallen heir (the newest few), the epitaph on hover, a tap opens every stone and the titles.
//   siege   — `Mother · 4 tries · best 22%` on the wall preview and the report; the grave on the chart (`Ada's pack · $1240`); the ledger
//             tip glosses `recovered` and `sinks`.
//   seek    — `seek Queen ×2` on the camp, the sheet priced by seekForecast, `in order` the default; a tap sets seekBoss.
//   trial   — one quiet camp card: the rule, its answers, `opt in` (setTrial(week)), a second tap opts out (setTrial(-1)).
//   finds   — one reveal in the report (best first; the sealed count its data and its tip, never the head), the set log in a fold.
//   sinks   — the run setup: the apprentice's sink order and one sink by hand (two taps), the heir order chip; the wish chip (two taps).
//   core    — the core's fields replace the client's estimates: king_eta_h / king_pct, News.day and the feats in the diary, the roster
//             (affix · guard), the pick's default, the heirs rung lit_by, the swift floors folded; the retired wall order shows nowhere.
//   taps    — a routine return with every Cut 118 field present is ≤ 2 taps from the report to the watch, no Cut 118 call made.
//   node web/tests/cut118b.mjs [--part=death,yard,siege,seek,trial,finds,sinks,core,taps]
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const ALL = "death,yard,siege,seek,trial,finds,sinks,core,taps";
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? ALL).split(",");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
const fresh = async (tag) => {
  const page = await browser.newPage({ viewport: { width: 400, height: 860 } });
  page.on("pageerror", (e) => errors.push(`${tag} pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&engine=fake&systems=all&fresh=1&seed=5&runs=0`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
  await page.waitForTimeout(300);
  // the fixtures below are lineages set by hand: the engine's lineage reads them back (no repaint from the fake's own state)
  await page.evaluate(() => {
    const a = window.__riddle; a.engine.lineage = async () => a.lineage;
    // shared fixtures (window.__fx): a feats wire and a report
    const stone = (heir, name, depth, epitaph, extra = {}) => ({ heir, name, depth, cause: "bloat_mother", epitaph, day: heir, run: heir, ...extra });
    window.__fx = {
      feats: (x = {}) => ({ tokens: [], trials: [], trial_marks: 0, sealed: 0, sets: [], swift_to: 0, sinks: [], sink_order: "both", feats: 0, siege: [], graves: [], graveyard: [], worn: {}, heir_order: "answer", titles: [], deed_legacy: 0, ...x }),
      stones: [stone(1, "Ada", 13, "fell to the Mother, D13", { boss: "bloat_mother", try_n: 1 }), stone(2, "Bram", 13, "fell to the Mother, D13", { boss: "bloat_mother", try_n: 2 }),
        stone(3, "Cora", 9, "fell to goblin archer, D9"), stone(4, "Dell", 13, "fell to the Mother, D13", { boss: "bloat_mother", try_n: 3 }), stone(5, "Edda", 13, "fell to the Mother, D13", { boss: "bloat_mother", try_n: 4 })],
      siege: { boss: "bloat_mother", title: "Mother", depth: 13, tries: 4, best_pct: 22, edge_pct: 8, heirs: ["Ada", "Bram", "Dell", "Edda"] },
      report: { elapsed_s: 28800, runs: 12, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 14, gold: { home: 900, salvage: 40, wake: 0, spent: 0 } },
      home: (L) => ({ ...L, live: null, ended: false, town: { ...(L.town ?? { buildings: [], bank: 0, bank_cap: 0, interest: 0 }), home: true } }),
    };
  });
  return page;
};
const wait = (page, ms) => page.waitForTimeout(ms);
try {
  if (parts.includes("death")) {
    const page = await fresh("death");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const card = (chip, source, gift = "guard") => ({ chip, head: chip.split(" · ")[0], formula: `[x] → ${gift} +1`, when: "boss", gift, tier: "common", source, learned: true });
      const offer = [card("stout · slow", "fresh"), card("guarded · bosses", "answer"), card("swift · frail", "fresh", "quick")];
      a.lineage = { ...a.lineage, heir: 5, heir_traits: { born: { ...offer[1], source: "born" }, offer, blood_open: false, bloodline_open: false }, feats: fx.feats({ siege: [fx.siege] }) };
      const swaps = [];
      a.engine.setTrait = async (chip) => { swaps.push(chip); a.lineage = { ...a.lineage, heir_traits: { ...a.lineage.heir_traits, born: { ...offer.find((c) => c.chip === chip), source: "born" } } }; return a.lineage; };
      const memorial = { lead: "+3 Legacy · Mother try 4 · +8%", epitaph: "fell to the Mother, D13", name: "Edda", siege: fx.siege, grave_gold: 1240, legacy: 3 };
      const d = { run_id: 7, depth: 13, cause: "bloat_mother", margin: "", verdict: "gap", baseline: 0, replays: 12, patches: [], morgue: "", trace: { turns: [] }, boss: "bloat_mother", memorial, lever: { kind: "wait", text: "drill next" } };
      a.go({ kind: "death", death: d, lost: [] });
      await new Promise((res) => setTimeout(res, 400));
      const well = document.querySelector(".death-well");
      const kids = [...well.children].map((c) => c.className.split(" ")[0]);
      const lead = document.querySelector(".memorial-lead"), epi = document.querySelector(".memorial-epitaph");
      const fold = document.querySelector(".death .heirs-fold");
      const closed = fold && !fold.open;
      const cards = [...(fold?.querySelectorAll(".heir-card") ?? [])].map((b) => `${b.dataset.chip}${b.classList.contains("on") ? "*" : ""}${b.classList.contains("answers") ? "!" : ""}`);
      fold?.querySelector('.heir-card[data-chip="swift · frail"]')?.click();
      await new Promise((res) => setTimeout(res, 200));
      const after = [...(document.querySelectorAll(".death .heirs-fold .heir-card.on") ?? [])].map((b) => b.dataset.chip);
      // a stall carries no memorial
      a.go({ kind: "death", death: { ...d, verdict: "stall" }, lost: [] });
      await new Promise((res) => setTimeout(res, 200));
      const stall = !!document.querySelector(".memorial-lead");
      // a kept death from the chronicle: no swap offered
      a.go({ kind: "death", death: d, lost: [], kept: true });
      await new Promise((res) => setTimeout(res, 200));
      const keptFold = !!document.querySelector(".heirs-fold");
      return { kids, lead: lead?.textContent, leadTip: lead?.title, epi: epi?.textContent, closed, cards, swaps, after, stall, keptFold, cause: document.querySelector(".death .cause")?.textContent };
    });
    const li = r.kids.indexOf("memorial-lead"), di = r.kids.indexOf("defeat"), ei = r.kids.indexOf("memorial-epitaph");
    check(r.lead === "+3 Legacy · Mother try 4 · +8%" && li >= 0 && li < di && ei === di + 1, `the memorial leads, then the cause; the epitaph under the banner (${r.kids.join(" ")})`);
    check(r.leadTip === "Mother · 4 tries · best 22%" && /Edda · fell to the Mother, D13 · pack \$1240 on D13/.test(r.epi ?? ""), `the siege on the lead, the stone and its pack (${r.leadTip} · ${r.epi})`);
    check(r.closed && r.cards.join("|") === "stout · slow|guarded · bosses*!|swift · frail", `other heirs: a closed fold, the order's pick lit, the answer named (${r.cards.join(" | ")})`);
    check(r.swaps.join() === "swift · frail" && r.after.join() === "swift · frail", `a card swaps by setTrait(chip) (${r.swaps} → ${r.after})`);
    check(!r.stall && !r.keptFold, `no memorial on a stall; no swap on a kept death (${r.stall} · ${r.keptFold})`);
    await page.close();
  }
  if (parts.includes("yard")) {
    const page = await fresh("yard");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      a.lineage = fx.home({ ...a.lineage, feats: fx.feats({ graveyard: fx.stones, titles: ["Warlordbane"] }) });
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 500));
      const y = document.querySelector(".town-graveyard");
      const shown = { visible: !!y && !y.hidden && y.getBoundingClientRect().width >= 44 && y.getBoundingClientRect().height >= 44, n: y?.dataset.stones, stones: y?.querySelectorAll(".stone").length,
        tips: [...(y?.querySelectorAll(".stone") ?? [])].map((s) => s.title), more: y?.querySelector(".stone-more")?.textContent, title: y?.querySelector(".stone-title")?.textContent, svg: !!y?.querySelector(".stone svg") };
      y?.click();
      await new Promise((res) => setTimeout(res, 300));
      const sheet = document.querySelector(".graveyard-sheet");
      const lines = [...(sheet?.querySelectorAll(".stone-line") ?? [])].map((x) => x.textContent);
      const titles = sheet?.querySelector(".graveyard-titles")?.textContent;
      const { closeAllSheets } = await import("/src/ui/sheet.ts"); closeAllSheets();
      a.lineage = fx.home({ ...a.lineage, feats: fx.feats() });
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 300));
      return { shown, lines, titles, empty: document.querySelector(".town-graveyard")?.hidden };
    });
    check(r.shown.visible && r.shown.n === "5" && r.shown.stones === 4 && r.shown.more === "+1" && r.shown.svg, `a stone per heir, the newest four drawn, chunky (${JSON.stringify(r.shown)})`);
    check(r.shown.tips[3] === "Edda · fell to the Mother, D13" && r.shown.title === "Warlordbane", `each stone's epitaph on hover; the title under them (${r.shown.tips.at(-1)} · ${r.shown.title})`);
    check(r.lines.length === 5 && /^Edda · fell to the Mother, D13 · try 4 · day 5$/.test(r.lines[0]) && r.titles === "Warlordbane", `a tap opens every stone, newest first, and the titles (${r.lines[0]} · ${r.titles})`);
    check(r.empty === true, `no stones, no graveyard (${r.empty})`);
    await page.close();
  }
  if (parts.includes("siege")) {
    const page = await fresh("siege");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const { wallPreview } = await import("/src/ui/wall-preview.ts");
      const { graveMark } = await import("/src/ui/feats.ts");
      const walls = [{ boss: "goblin_warlord", title: "Warlord", depth: 8, slain: true, known: true, fact: "" }, { boss: "bloat_mother", title: "Mother", depth: 13, slain: false, known: true, fact: "mother: fire", counter: "fire",
        affix: "brood", affix_counter: "burn", guard: "bloat", guard_counter: "attack" }];
      const feats = fx.feats({ siege: [fx.siege], graves: [{ depth: 1, heir: 1, name: "Ada", gold: 1240, day: 1 }] });
      const wp = wallPreview({ ...a.lineage, walls, feats });
      const mark = graveMark({ feats }, 1)?.textContent, none = graveMark({ feats }, 2);
      // the camp's chart: the grave on its floor's notch
      a.lineage = fx.home({ ...a.lineage, walls, feats });
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 800));
      const notch = document.querySelector(".camp .grave-mark")?.textContent ?? null;
      // the report: the siege line from its tries (one line a boss)
      const rep = { ...fx.report, deaths: [{ cause: "bloat_mother", depth: 13, n: 2 }], feats: [{ k: "siege", text: "Mother · try 3 · +6%", day: 2 }, { k: "siege", text: "Mother · try 4 · +8%", day: 2 }, { k: "recovered", text: "Ada's pack · $1240 · D1", day: 2 }],
        gold: { home: 900, salvage: 40, wake: 0, spent: 0, net: 1100, ledger: { earned: 2180, spent: 1080, net: 1100, terms: [{ label: "carried", amount: 940 }, { label: "recovered", amount: 1240 }, { label: "sinks", amount: -900 }, { label: "supplies", amount: -180 }] } } };
      a.go({ kind: "report", report: rep, absence: true });
      await new Promise((res) => setTimeout(res, 500));
      // owner IA pass 2026-10-10: the card carries the one death-that-made-progress line; the pack recovered waits in the fold's way down
      const feat = [...document.querySelectorAll(".report-sheet .report-feats .feat-line")].map((x) => `${x.dataset.k}:${x.textContent}`);
      const card = [...document.querySelectorAll(".report-sheet > .report-feats .feat-line")].map((x) => x.dataset.k);
      const folded = [...document.querySelectorAll(".report-details [data-group=way] .report-feats .feat-line")].map((x) => x.dataset.k);
      return { wp: wp?.querySelector(".wall-siege")?.textContent, more: [...(wp?.querySelectorAll(".wall-more") ?? [])].map((x) => x.textContent), mark, none, notch, feat, card, folded };
    });
    check(r.wp === "Mother · 4 tries · best 22%" && r.more.join("|") === "affix brood → burn|guard bloat → attack", `the wall preview: the roster and the siege (${r.more.join(" | ")} · ${r.wp})`);
    check(r.mark === "✝ Ada's pack · $1240" && r.none === null && r.notch === "✝ Ada's pack · $1240", `the grave on its floor, on the chart too (${r.mark} · chart ${r.notch})`);
    check(r.feat[0] === "siege:Mother · 4 tries · best 22%" && r.feat.filter((x) => x.startsWith("siege")).length === 1 && r.feat.some((x) => /^recovered:Ada's pack/.test(x)), `the report: one siege line a boss, the pack recovered (${r.feat.join(" | ")})`);
    check(r.card.join() === "siege" && r.folded.includes("recovered"), `the siege a highlight, the pack under details (card ${r.card} · fold ${r.folded})`);
    await page.hover(".report-gold");
    await page.waitForFunction(() => document.querySelectorAll("#kw-tip .ledger-term").length > 0, null, { timeout: 5000 }).catch(() => {});
    const tip = await page.evaluate(() => [...document.querySelectorAll("#kw-tip .ledger-term")].map((x) => x.textContent));
    check(tip.includes("recovered · +$1240 · graves brought home") && tip.includes("sinks · −$900 · tithe · rations · survey"), `the ledger tip names recovered and sinks (${tip.join(" | ")})`);
    await page.close();
  }
  if (parts.includes("seek")) {
    const page = await fresh("seek");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const tokens = [{ boss: "bloat_mother", title: "Mother", depth: 13, n: 1, cap: 3, stone: 10 }, { boss: "mirror_queen", title: "Queen", depth: 28, n: 2, cap: 3, stone: 24 }];
      a.lineage = fx.home({ ...a.lineage, feats: fx.feats({ tokens }) });
      const calls = [];
      a.engine.seekForecast = async () => { calls.push("forecast"); return tokens.map((t, i) => ({ boss: t.boss, title: t.title, depth: t.depth, stone: t.stone, tokens: t.n, reach: 0.8 - 0.3 * i, past: 0.5 - 0.3 * i, death: 0.2, sims: 24, current: false })); };
      a.engine.seekBoss = async (b) => { calls.push(`seek:${b}`); a.lineage = { ...a.lineage, feats: { ...a.lineage.feats, seek: b || undefined } }; return a.lineage; };
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 400));
      const chip = () => document.querySelector(".camp .seek-chip");
      const before = { text: chip()?.textContent, on: chip()?.classList.contains("on") };
      chip().click();
      await new Promise((res) => setTimeout(res, 300));
      const opts = [...document.querySelectorAll(".seek-sheet .seek-opt")].map((b) => `${b.dataset.seek}${b.classList.contains("on") ? "*" : ""}:${b.textContent}`);
      document.querySelector('.seek-sheet .seek-opt[data-seek="mirror_queen"]').click();
      await new Promise((res) => setTimeout(res, 300));
      const after = { text: chip()?.textContent, on: chip()?.classList.contains("on") };
      return { before, opts, after, calls };
    });
    check(r.before.text === "seek Queen ×2" && !r.before.on, `the camp's seek chip, dim while in order (${r.before.text})`);
    check(r.opts[0] === "*:in order · default" && /^mirror_queen:Queen D28 ×2 · from D24 · reach 50% · past 20%$/.test(r.opts[2]) && r.calls[0] === "forecast", `the tokens priced by seekForecast; in order the default (${r.opts.join(" | ")})`);
    check(r.calls.includes("seek:mirror_queen") && r.after.text === "seeking Queen ×2" && r.after.on, `a tap seeks him (${r.calls} · ${r.after.text})`);
    await page.close();
  }
  if (parts.includes("trial")) {
    const page = await fresh("trial");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const trial = (week, opted = false) => ({ week, boss: "bloat_mother", title: "Mother", depth: 13, rule: "barred", label: "no boss focus", barred: "boss_focus", answers: ["guarded", "hunter"], owned: ["guarded"], open: true, cleared: false, opted, weeks_left: 3, legacy: 10 });
      const shut = { ...trial(39), open: false, needs: "slay Lich" };
      a.lineage = fx.home({ ...a.lineage, feats: fx.feats({ trials: [trial(41), trial(40), shut] }) });
      const calls = [];
      a.engine.setTrial = async (w) => { calls.push(w); a.lineage = { ...a.lineage, feats: { ...a.lineage.feats, trials: a.lineage.feats.trials.map((t) => ({ ...t, opted: t.week === w })) } }; return a.lineage; };
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 400));
      const card = () => document.querySelector(".camp .trial-card");
      const c0 = card();
      const face = { n: document.querySelectorAll(".camp .trial-card").length, week: c0?.dataset.week ?? c0?.dataset.trial, head: c0?.querySelector(".trial-head")?.textContent, rule: c0?.querySelector(".trial-rule")?.textContent,
        answers: c0?.querySelector(".trial-answers")?.textContent, owned: [...(c0?.querySelectorAll(".trial-answer.owned") ?? [])].map((x) => x.dataset.answer), opt: c0?.querySelector(".trial-opt")?.textContent, weeks: [...(c0?.querySelectorAll(".trial-week") ?? [])].map((x) => x.dataset.week) };
      c0.querySelector(".trial-opt").click();
      await new Promise((res) => setTimeout(res, 300));
      const on = { opted: card()?.dataset.opted, opt: card()?.querySelector(".trial-opt")?.textContent };
      card().querySelector(".trial-opt").click();
      await new Promise((res) => setTimeout(res, 300));
      return { face, on, calls, words: (face.head ?? "").split(/\s+/).filter((w) => /\p{L}/u.test(w)).length };
    });
    check(r.face.n === 1 && r.face.week === "41" && r.face.head === "trial Mother D13 · +10 Legacy" && r.face.rule === "no boss focus", `one quiet card, the newest open week (${r.face.head} · ${r.face.rule})`);
    check(r.face.answers === "answers · guarded, hunter" && r.face.owned.join() === "guarded" && r.face.weeks.join() === "40" && r.face.opt === "opt in", `its answers (owned lit), the other open week a chip, opt in (${r.face.answers} · ${r.face.weeks})`);
    check(r.calls.join() === "41,-1" && r.on.opted === "1" && r.on.opt === "opted", `opt in → setTrial(41), again → setTrial(-1) (${r.calls})`);
    await page.close();
  }
  if (parts.includes("finds")) {
    const page = await fresh("finds");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const finds = [{ id: "quill", kind: "piece", name: "scholar's quill", rank: 4, set: "scholar", table: 2 }, { id: "quilt", kind: "piece", name: "sleeper's quilt", rank: 3, set: "sleeper", table: 1 }, { id: "lamp", kind: "cosmetic", name: "brass lamp", rank: 1, table: 1 }];
      const sets = [{ id: "sleeper", title: "Sleeper's rest", pieces: ["quilt", "pillow", "candle"], have: [true, true, true], done: true, bonus: "rest −2%" }, { id: "scholar", title: "Scholar's", pieces: ["quill", "ink", "lens"], have: [true, false, false], done: false, bonus: "class xp +2%" }];
      a.lineage = { ...a.lineage, tree: { ...(a.lineage.tree ?? { nodes: [], waits: false, sent: false, ledger: 0 }), auto_send: true }, feats: fx.feats({ sets }) };
      const rep = { ...fx.report, finds: { sealed: 7, best: finds[0], finds, legacy: 2, sets: ["Sleeper's rest"] } };
      a.go({ kind: "report", report: rep, absence: true });
      await new Promise((res) => setTimeout(res, 500));
      const sheet = document.querySelector(".report-sheet");
      const kids = [...sheet.children].map((c) => c.className.split(" ")[0]);
      const rv = document.querySelector(".report-reveal");
      const log = document.querySelector(".report-details .set-log");
      return { kids, reveal: rv?.textContent, best: rv?.dataset.best, n: document.querySelectorAll(".report-reveal").length, sealed: rv?.dataset.sealed, head: document.querySelector(".report-summary h2")?.textContent,
        log: log ? { open: log.open, summary: log.querySelector("summary")?.textContent, lines: [...log.querySelectorAll(".set-line")].map((x) => `${x.dataset.set}:${x.querySelectorAll(".set-piece.have").length}`) } : null };
    });
    check(r.n === 1 && r.best === "quill" && r.reveal === "✦ scholar's quill · set piece +2 · +2 Legacy · set Sleeper's rest", `one reveal, best first, never a click per find (${r.reveal})`);
    check(r.sealed === "7" && !/sealed/.test(r.head ?? ""), `the sealed count rides on the reveal, never the away head (${r.sealed} · ${r.head})`);
    check(r.kids.indexOf("report-reveal") < r.kids.indexOf("collect-send") && r.kids.indexOf("report-reveal") > r.kids.indexOf("report-summary"), `the reveal is a highlight, before collect & send (${r.kids.join(" ")})`);
    check(r.log && !r.log.open && r.log.summary === "set log 1/2" && r.log.lines.join() === "sleeper:3,scholar:1", `the set log, a closed fold under details (${JSON.stringify(r.log)})`);
    await page.close();
  }
  if (parts.includes("sinks")) {
    const page = await fresh("sinks");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const sinks = [{ id: "tithe", price: 900, line: "1 Legacy / $900", available: true }, { id: "ration", price: 90, line: "+10% hp · 1 run", available: true }];
      const nodes = [...(a.lineage.tree?.nodes ?? []).filter((n) => n.id !== "apprentice"), { id: "apprentice", kind: "worker", name: "apprentice", state: "done", branch: "trunk" }];
      const orders = { ...(a.lineage.orders ?? { keep: "best_weapon", cage: "weapon", start: 1, repeat: true, insure: true }), forge: "half", sink: "both", heir: "answer", wall: "bank" };
      a.lineage = fx.home({ ...a.lineage, gold: 5000, orders, tree: { ...(a.lineage.tree ?? { waits: false, sent: false, ledger: 0 }), nodes }, systems: [...(a.lineage.systems ?? []), { id: "heirs", open: true, trigger: "first death", lit_by: "first death" }],
        feats: fx.feats({ sinks, wish: { id: "lantern", text: "a lantern", price: 45, legacy: 2, available: true } }) });
      const calls = [];
      a.engine.setOrders = async (o) => { calls.push(`orders:${o.heir}/${o.sink}`); a.lineage = { ...a.lineage, orders: o }; return a.lineage; };
      a.engine.tithe = async (n) => { calls.push(`tithe:${n}`); return a.lineage; };
      a.engine.grantWish = async () => { calls.push("wish"); a.lineage = { ...a.lineage, feats: { ...a.lineage.feats, wish: undefined } }; return a.lineage; };
      a.go({ kind: "camp" });
      await new Promise((res) => setTimeout(res, 400));
      const tab = document.querySelector(".camp .orders-tab");
      const sum0 = tab?.textContent;
      // the wish: one chip, two taps
      const wish = () => document.querySelector(".camp .wish-chip");
      const w0 = wish()?.textContent; wish().click();
      const wArmed = wish()?.textContent, wCalls = calls.length; wish().click();
      await new Promise((res) => setTimeout(res, 300));
      const wGone = !wish();
      tab.click();
      await new Promise((res) => setTimeout(res, 400));
      const rows = [...document.querySelectorAll(".orders-sheet .order-row")].map((x) => x.textContent);
      const heir = [...document.querySelectorAll(".orders-sheet .order-row")].find((x) => /next heir/.test(x.textContent));
      heir.querySelector("button:nth-of-type(2)").click();
      await new Promise((res) => setTimeout(res, 300));
      const hand = document.querySelector(".orders-sheet .sink-hand");
      const h0 = hand?.textContent; hand.click(); const hArmed = document.querySelector(".orders-sheet .sink-hand")?.textContent; document.querySelector(".orders-sheet .sink-hand").click();
      await new Promise((res) => setTimeout(res, 300));
      const { closeAllSheets } = await import("/src/ui/sheet.ts"); closeAllSheets();
      await new Promise((res) => setTimeout(res, 200));
      return { sum0, w0, wArmed, wCalls, wGone, rows, h0, hArmed, calls, sum1: document.querySelector(".camp .orders-tab")?.textContent };
    });
    check(r.w0 === "✧ a lantern · +2" && r.wArmed === "ok $45" && r.wCalls === 0 && r.calls.includes("wish") && r.wGone, `the wish: one chip, armed then granted, gone (${r.w0} → ${r.wArmed})`);
    check(r.rows.some((x) => x.replace(/\s+/g, "") === "nextheiranswerstrongestsurprise") && r.rows.some((x) => /^apprentice sinks/.test(x)) && !r.rows.some((x) => /at walls|bank before/.test(x)), `run setup: the heir order and the sinks; no wall order (${r.rows.join(" | ")})`);
    check(r.calls.includes("orders:strongest/both") && /heirs strongest/.test(r.sum1 ?? "") && !/heirs/.test(r.sum0 ?? "x"), `the heir order chip sets it; the summary names it off its default (${r.sum1})`);
    check(r.h0 === "tithe · 1 Legacy / $900" && r.hArmed === "ok $900" && r.calls.includes("tithe:1"), `one sink by hand, its rate shown, two taps (${r.h0} → ${r.hArmed})`);
    await page.close();
  }
  if (parts.includes("core")) {
    const page = await fresh("core");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const { kingEta, kingLine } = await import("/src/ui/king-eta.ts");
      const { cries, writeDiary } = await import("/src/ui/crier.ts");
      const { defaultOffer } = await import("/src/ui/return-pick.ts");
      const { ladderRungs } = await import("/src/ui/ladder.ts");
      const { ordersLine } = await import("/src/ui/collect-send.ts");
      // king: the core's hours and share, not the client's curve
      const L = (x) => ({ age_h: 30, best_depth: 13, ended: false, walls: [], runs: [], trophies: [], ...x });
      const core = kingEta(L({ king_eta_h: 100 })), crude = kingEta(L({}));
      const line = kingLine(L({ king_eta_h: 100, king_pct: 40 }));
      const slain = kingLine(L({ king_eta_h: 0, king_pct: 100 }))?.textContent;
      // the diary: the core's days and its feats
      try { localStorage.removeItem(`riddle.diary.${a.lineage.seed ?? 0}`); } catch {}
      const rep = { ...fx.report, exits: [{ carried: 0, keep_pct: 100, kept: 0, spent: 0, spent_on: [], text: "banked $0", news: [{ k: "named", text: "avenged Zelul", day: 3 }] }],
        feats: [{ k: "siege_won", text: "Mother fell · 5 tries", day: 4 }, { k: "title", text: "the line is Motherbane", day: 4 }, { k: "heir", text: "Bram · guarded · bosses · answers the Mother", day: 4 }, { k: "trial", text: "trial cleared · Mother · +10 Legacy", day: 2 }] };
      const cs = cries(rep, { walls: [] });
      const diary = writeDiary({ seed: a.lineage.seed, age_h: 200 }, cs).map((d) => `${d.day}:${d.k}`);
      // the pick's default is the core's
      const offers = [{ id: "drill", title: "Steady", line: "+3", available: true }, { id: "legacy", title: "Legacy", line: "+4", available: true }];
      const pick = { core: defaultOffer(offers, "drill")?.id, old: defaultOffer(offers)?.id, gone: defaultOffer(offers, "forge")?.id };
      // the ladder: the heirs rung, lit by the core's lit_by; other rungs read lit_by too
      const sys = [{ id: "stances", open: true, trigger: "first bank", lit_by: "feat: trial" }, { id: "loadout", open: false, trigger: "first salvage" }, { id: "pen", open: false, trigger: "meet Mother" }, { id: "heirs", open: true, trigger: "first death", lit_by: "first death" }];
      const base = { ...a.lineage, best_depth: 4, systems: sys, tree: undefined, packages: { ...(a.lineage.packages ?? {}), literal: false, pen_open: false, all: [{ id: "steady", kind: "stance", owned: true }, { id: "guarded", kind: "stance", owned: true }] } };
      const rungs = ladderRungs(base, () => false).map((x) => `${x.id}${x.lit ? "+" : "-"}${x.litBy ? `(${x.litBy})` : ""}`);
      const heirs = ladderRungs(base, () => false).find((x) => x.id === "heirs");
      const noHeirs = ladderRungs({ ...base, systems: sys.filter((s) => s.id !== "heirs") }, () => false).map((x) => x.id).join(",");
      // the retired wall order is never named
      const orders = ordersLine({ keep: "best_weapon", cage: "weapon", start: 1, repeat: true, insure: true, wall: "bank", heir: "surprise", sink: "off" });
      return { core: core && { day: core.day }, crude: crude && { day: crude.day }, line: line?.textContent, src: line?.dataset.src, pct: line?.dataset.pct, slain,
        cries: cs.map((c) => `${c.k}${c.day ?? ""}`), diary, pick, rungs, retires: heirs?.retires, noHeirs, orders };
    });
    check(r.core?.day === 6 && r.crude?.day !== r.core?.day && r.src === "core" && r.pct === "40" && /^King · ~day 640% · next: harder dungeon$/.test(r.line ?? "") && r.slain === "King · slain100% · next: harder dungeon", `the King's line from king_eta_h / king_pct (${r.line} · crude day ${r.crude?.day} · ${r.slain})`);
    check(r.cries.join() === "siege_won4,title4,trial2,avenged3" && r.diary.join() === "4:siege_won,4:title,2:trial,3:avenged", `the crier: the core's feats in his voice, each dated by its own day (${r.cries} · ${r.diary})`);
    check(r.pick.core === "drill" && r.pick.old === "legacy" && r.pick.gone === "legacy", `the pick's default is the core's (${JSON.stringify(r.pick)})`);
    check(r.rungs.join(" ") === "weights+(feat: trial) orders- heirs+(first death) workers- pen-" && r.retires === "picking heirs" && r.noHeirs === "weights,orders,workers,pen", `the heirs rung lit by lit_by; the core's lit_by on every rung (${r.rungs.join(" ")})`);
    check(r.orders.join("|") === "keep weapon|restock on|insure on|heirs surprise|sinks off", `the carried orders: heir and sink named, the retired wall order not (${r.orders.join(" | ")})`);
    await page.close();
    // the swift floors: a send whose snapshot is swift folds them under one line marked `swift`
    const p2 = await fresh("swift");
    const s = await p2.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      a.lineage = fx.home({ ...a.lineage, feats: fx.feats({ swift_to: 3 }) });
      const send = a.engine.send;
      a.engine.send = async () => ({ ...(await send()), swift: true });
      a.go({ kind: "watch" });
      for (let i = 0; i < 40 && !document.querySelector(".watch")?.dataset.swift; i++) await new Promise((res) => setTimeout(res, 100));
      const w = document.querySelector(".watch");
      return { swift: w?.dataset.swift, plan: w?.dataset.foldPlan };
    });
    check(s.swift === "3" && /\b1\b/.test(s.plan ?? "") && /\b3\b/.test(s.plan ?? ""), `the swift floors fold (swift to D${s.swift} · plan ${s.plan})`);
    await p2.close();
  }
  if (parts.includes("taps")) {
    const page = await fresh("taps");
    const r = await page.evaluate(async () => {
      const a = window.__riddle, fx = window.__fx;
      const step = (label, price) => ({ label, price, affordable: true });
      const kit = [{ slot: "weapon", owned: 1, steps: [], next: step("sword +2", 300) }];
      const tokens = [{ boss: "mirror_queen", title: "Queen", depth: 28, n: 2, cap: 3, stone: 24 }];
      const trials = [{ week: 41, boss: "bloat_mother", title: "Mother", depth: 13, rule: "barred", label: "no boss focus", answers: ["guarded"], owned: ["guarded"], open: true, cleared: false, opted: false, weeks_left: 3, legacy: 10 }];
      const feats = fx.feats({ tokens, trials, wish: { id: "lantern", text: "a lantern", price: 45, legacy: 2, available: true }, sinks: [{ id: "tithe", price: 900, line: "1 Legacy / $900", available: true }], graveyard: window.__fx.stones });
      const pick = { minutes: 480, size: 3, default: "drill", offers: [{ id: "drill", title: "Steady", line: "+3 runs", available: true }, { id: "legacy", title: "Legacy", line: "+4", available: true }, { id: "forge", title: "sword +2", line: "−20%", price: 240, available: true }] };
      a.lineage = fx.home({ ...a.lineage, kit, gold: 5000, return_pick: pick, feats, heir_traits: undefined,
        tree: { ...(a.lineage.tree ?? { nodes: [], waits: false, sent: false, ledger: 0 }), chest: 120, auto_send: true }, orders: { keep: "best_weapon", cage: "weapon", start: 1, repeat: true, insure: false, heir: "answer", sink: "both" } });
      const calls = [];
      a.engine.openChest = async () => { calls.push("chest"); a.lineage = { ...a.lineage, gold: a.lineage.gold + 120, tree: { ...a.lineage.tree, chest: 0 } }; return a.lineage; };
      a.engine.buyKit = async (slot) => { calls.push(slot); a.lineage = { ...a.lineage, kit: a.lineage.kit.map((k) => ({ ...k, next: undefined })) }; return a.lineage; };
      a.engine.takeReturnPick = async (id) => { calls.push(`pick:${id}`); a.lineage = { ...a.lineage, return_pick: undefined }; return a.lineage; };
      for (const m of ["seekBoss", "setTrial", "grantWish", "tithe", "buySurvey", "setTrait", "setOrders"]) { const f = a.engine[m]; a.engine[m] = async (...x) => { calls.push(m); return f ? f(...x) : a.lineage; }; }
      const rep = { ...fx.report, finds: { sealed: 3, best: { id: "lamp", kind: "cosmetic", name: "brass lamp", rank: 1, table: 1 }, finds: [{ id: "lamp", kind: "cosmetic", name: "brass lamp", rank: 1, table: 1 }] },
        feats: [{ k: "heir", text: "Bram · guarded · bosses · answers the Mother", day: 2 }, { k: "siege", text: "Mother · try 4 · +8%", day: 2 }] };
      a.go({ kind: "report", report: rep, absence: true });
      await new Promise((res) => setTimeout(res, 500));
      const btn = () => document.querySelector(".report .collect-go");
      const label = btn()?.textContent;
      const prompts = [".return-pick", ".report-choices", ".report-upgrade-host"].filter((s) => { const el = document.querySelector(`.report-sheet ${s}`); return el && !el.hidden && !el.closest(".report-details"); });
      let taps = 0;
      btn().click(); taps++;
      const armed = btn()?.textContent;
      btn().click(); taps++;
      for (let i = 0; i < 60 && a.screen !== "watch"; i++) await new Promise((res) => setTimeout(res, 100));
      return { label, armed, taps, calls, screen: a.screen, prompts };
    });
    check(r.label === "collect & send" && r.armed === "ok $300", `one action with every Cut 118 field present (${r.label} → ${r.armed})`);
    check(r.taps <= 2 && r.screen === "watch" && r.calls.join() === "chest,pick:drill,weapon", `a routine return: ≤ 2 taps to the watch, the pick's core default taken, no Cut 118 call (${r.calls} · ${r.taps} taps · ${r.screen})`);
    check(r.prompts.length <= 1, `at most one decision prompt in the card (${r.prompts})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut118b: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut118b: ok (${out.length} checks)`);
