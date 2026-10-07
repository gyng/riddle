#!/usr/bin/env node
// Cut 30 client gates (docs/CUT30.md), on the fake engine (its Cut 30 stand-ins), headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU).
//
// Panels section (§2, §4, §5, the Reveal):
//   packages  the stance / tactic / temperament slots, chips `<name> L<n>` with a level bar, each alternative priced in one line
//             (`death −8`), a shadowed row greyed `<winner> wins`, equipping free and instant, drills and scars
//   quest     one goal (≤ 5 words, eval/copy-budgets.json `quest_goal`), the reward a picture, a progress bar, the day's one free swap,
//             `QUEST DONE` on the board and as the report's beat; no oath UI in play
//   pen       no editor and no rule tablets before the pen; the editor (and its tile) once it opens
//   death     before the pen: the cause and exactly one lever (no fixes, no trace, no `gap`/`dice`, no replays); with the pen, the fixes
//   report    the report leads with what grew on each track, the packages' beats as plaques; slices merge (`mergeGrew`)
//
//   node web/tests/cut30.mjs [--shots dir] [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync, readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";
import { load } from "./lib/load.mjs";
/** A wall-clock reading retried once on a loaded machine (`undo` between the two), the bar unchanged (tests/lib/load.mjs). */
async function measured(measure, undo) {
  const first = await measure(); if (first.ok) return first;
  const l = load(); if (!l.high) return first;
  await undo(); const again = await measure();
  return { ...again, line: `${again.line} [retried once: load ${l.l1.toFixed(1)} on ${l.cores} cores; first ${first.line}]` };
}

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
const wide = process.argv.includes("--wide");
if (shots) mkdirSync(shots, { recursive: true });
const BUDGET = JSON.parse(readFileSync(resolve(ROOT, "eval/copy-budgets.json"), "utf8")).surfaces;
const words = (s) => s.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const view = wide ? { width: 1440, height: 900 } : { width: 400, height: 800 };
const page = await browser.newPage({ viewport: view, deviceScaleFactor: shots ? 2 : 1 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const shot = async (name) => { if (shots) { await sleep(400); await page.screenshot({ path: resolve(shots, `${name}${wide ? "-desktop" : "-phone"}.png`), fullPage: false }); } };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp", "camp"); await sleep(300); };
/** The fake's whole state edited through a save round-trip (`e.lineage`, `e.sys29` the curriculum, `e.st30` the packages/quest), then the camp. */
const withState = (fn) => page.evaluate(async (src) => {
  const r = window.__riddle; const save = JSON.parse(await r.engine.save());
  new Function("e", src)(save);
  r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" });
}, `(${fn})(e)`);
const boot = async (seed, q = "") => {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}${q}`, { waitUntil: "domcontentloaded" });
  await camp();
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); });
};
const tiles = () => page.evaluate(() => [...document.querySelectorAll(".cmd .tile:not(.empty)")].map((t) => t.dataset.tile + (t.classList.contains("reveal") ? "*" : "")));
const closeSheets = () => page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); });
/** A lineage past the Warlord: the second stance, the tactics, the quest board (the fake: D9). */
const pastWarlord = (extra = "") => withState(new Function("e", `e.lineage.best_depth = 9; e.lineage.gold = 120; e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 6, cause: "goblin" }]; ${extra}`));

try {
  // (tracks: the panel became the works sheet's branches in Cut 30.5 — its gates are tests/cut305.mjs `sheet`)

  // ---- packages: slots, levels, shadowing, the one-line price, equipping free and instant
  if (part("packages")) {
    await boot(3002);
    check(!(await tiles()).includes("packages"), `day 0: no packages tile (${(await tiles()).join(" ")})`);
    const strip0 = await page.evaluate(() => [...document.querySelectorAll(".pkg-strip .pkg-tab")].map((t) => ({ text: t.querySelector(".pkg-name")?.textContent, btn: t.tagName === "BUTTON" })));
    check(strip0.length === 1 && strip0[0].text === "Steady L1" && !strip0[0].btn, `day 0: the worn stance as a plaque on the camp (${JSON.stringify(strip0)})`);
    await pastWarlord();
    await withState((e) => { e.st30.runs = { steady: 45 }; e.rules.rows.push({ ...e.rules.rows[0] }); e.lineage.sets[0].rows.push({ ...e.lineage.sets[0].rows[0] }); });
    const t1 = await tiles();
    check(await page.locator('.cmd .tile[data-tile="packages"] .tl').textContent() === 'tactics', 'town control says tactics');
    await page.click(`.cmd .tile[data-tile="packages"]`);
    await until(() => !!document.querySelector(".pkg-panel .pkg-slot"), "the packages panel");
    check(await page.locator('.pkg-choices').first().isHidden(), 'choices folded on opening');
    await page.click('.pkg-compare');
    if (await page.locator('[data-change-kind="stance"]').getAttribute('aria-expanded') === 'false') await page.click('[data-change-kind="stance"]');
    await page.click('[data-edit-slot="0"]');
    await until(() => !document.querySelector(".pkg-panel .pkg-price.pending"), "the prices", 10_000);
    const P = await page.evaluate(() => {
      const q = (s) => [...document.querySelectorAll(s)];
      return {
        head: document.querySelector(".pkg-headline")?.textContent ?? "",
        stance: document.querySelector('.pkg-sec[data-kind="stance"] .pkg-slot .chip.pkg.on .pkg-name')?.textContent,
        bar: document.querySelector('.pkg-sec[data-kind="stance"] .pkg-slot .lvl-bar .fill')?.style.width,
        alts: q('.pkg-sec[data-kind="stance"] .chip.pkg.alt').map((c) => ({ name: c.querySelector(".pkg-name").textContent, price: [...c.querySelectorAll(".pkg-price")].map((x) => x.textContent), cls: c.querySelector(".pkg-price").className })),
        tPrices: q('.pkg-sec[data-kind="tactic"] .chip.pkg.alt .pkg-price').map((x) => ({ t: x.textContent, cls: x.className })),
        rowSrc: q(".pkg-row .pkg-row-src").map((x) => x.textContent),
        tactic: q('.pkg-sec[data-kind="tactic"] .pkg-slot').length, tAlts: q('.pkg-sec[data-kind="tactic"] .chip.pkg.alt').length,
        temper: !!document.querySelector('.pkg-sec[data-kind="temperament"]'),
        shadow: q(".pkg-row.shadowed .pkg-row-src").map((x) => x.textContent),
        drills: q(".pkg-drill").map((d) => d.textContent),
      };
    });
    check(/^floor \d+ (\d+%|<\d+%|>\d+%)$/.test(P.head), `the panel's head is the forecast's one headline (\`${P.head}\`)`);
    check(P.stance === "Steady L3" && P.bar && P.bar !== "0%", `the worn stance's chip \`Steady L3\` and its level bar (${P.stance}, ${P.bar})`);
    check(P.alts.length >= 2 && P.alts.every((a) => a.price.length === 1 && /^(deaths|deeper|full haul) ≈[+−]\d+$|^$/.test(a.price[0])), `each other stance priced in one line, nothing inside the noise (${P.alts.map((a) => `${a.name} · ${a.price.join("|")}`).join(", ")})`);
    const rank = (c) => (/ up/.test(c) ? 2 : / down/.test(c) ? 0 : 1);
    const ordered = (xs) => xs.every((x, i) => i === 0 || rank(xs[i - 1]) >= rank(x));
    check(ordered(P.alts.map((a) => a.cls)) && ordered(P.tPrices.map((x) => x.cls)) && P.alts.every((a) => !a.price[0]), `rough comparison hides the fake fixture’s small differences (stances ${P.alts.map((a) => a.price[0]).join(" · ")}; tactics ${P.tPrices.map((x) => x.t).join(" · ")})`);
    check(P.tPrices.filter((x) => / down/.test(x.cls)).length <= 1, `the tactics are not a wall of losses (${P.tPrices.map((x) => x.t).join(" · ")})`);
    check(P.rowSrc.length > 0 && !P.rowSrc.some((x) => /the pen/.test(x)), `before the pen every row names its package, never the pen (${P.rowSrc.join(" · ")})`);
    check(P.tactic === 1 && P.tAlts >= 1, `one tactic slot at the Warlord slain, the tactics to wear (${P.tactic} slot, ${P.tAlts} tactics)`);
    check(!P.temper, `no temperament before heir 3 (${P.temper})`);
    check(P.shadow.length >= 1 && P.shadow.every((s) => /^\S.* wins$/.test(s)), `a shadowed row greyed with its winner (${P.shadow.join(", ")})`);
    const warlord = page.locator('.pkg-drill[data-boss="goblin_warlord"]');
    check(await warlord.locator('.counter-name').textContent() === 'Warlord' && await warlord.locator('.counter-action').textContent() === 'attack boss' && await warlord.locator('button').getAttribute('aria-pressed') === 'true', `the Warlord's enabled counter and actual action shown (${P.drills.join(", ")})`);
    await page.click('.pkg-advanced > summary');
    await page.click('.pkg-rows-btn');
    await shot("packages");
    // equip: free and instant
    const g0 = await page.evaluate(() => window.__riddle.lineage.gold);
    // (a wall-clock reading: on a loaded machine it is taken once more, back from Steady — tests/lib/load.mjs)
    const wear = async (id, name) => {
      if (!(await page.locator('.pkg-sec[data-kind="stance"] .pkg-choices').isVisible())) await page.click('[data-change-kind="stance"]');
      const t0 = Date.now();
      await page.click(`.pkg-sec[data-kind="stance"] .chip.pkg.alt[data-pkg="${id}"]`);
      await until((a) => window.__riddle.lineage.packages.stance === a.id && document.querySelector('.pkg-sec[data-kind="stance"] .pkg-slot .chip.pkg.on .pkg-name')?.textContent === a.name, `${name} worn`, 20_000, { id, name });
      return Date.now() - t0;
    };
    const eq = await measured(async () => { const ms = await wear("guarded", "Guarded L1"); return { ok: ms < 1500, ms, line: `${ms} ms` }; }, async () => { await wear("steady", "Steady L3"); });
    const g1 = await page.evaluate(() => window.__riddle.lineage.gold);
    check(g1 === g0 && eq.ok, `equipping is free and instant ($${g0} → $${g1}, ${eq.line})`);
    // a tactic worn in its slot
    await page.click('[data-edit-slot="0"]');
    await page.click('.pkg-sec[data-kind="tactic"] .chip.pkg.alt');
    const tw = await until(() => window.__riddle.lineage.packages.tactics?.length === 1 && document.querySelector('.pkg-sec[data-kind="tactic"] .chip.pkg.on .pkg-name')?.textContent, "a tactic worn");
    check(/ L\d$/.test(tw), `a tactic worn reads \`<name> L<n>\` (${tw})`);
    // revoke the drill: one tap, it stays
    if (!(await page.locator('.pkg-advanced').evaluate(e=>e.open))) await page.click('.pkg-advanced > summary');
    await page.click(".pkg-drill .drill");
    const rv = await until(() => window.__riddle.lineage.packages.drills?.find((d) => d.boss === "goblin_warlord")?.revoked === true && document.querySelector(".pkg-drill.revoked") ? true : null, "the drill revoked");
    check(rv, `a drill revoked with one tap stays revoked`);
    await closeSheets();
    // heir 3: the temperament slot, the wake's three cards (card 1 worn until picked)
    await withState((e) => { e.lineage.heir = 3; });
    await page.click(`.cmd .tile[data-tile="packages"]`);
    await until(() => !!document.querySelector('.pkg-sec[data-kind="temperament"]'), "the temperament slot");
    if (await page.locator('[data-change-kind="temperament"]').count()) await page.click('[data-change-kind="temperament"]');
    const cards = await page.evaluate(() => [...document.querySelectorAll(".chip.pkg.temper")].map((c) => c.dataset.pkg));
    await page.click(`.chip.pkg.temper[data-pkg="${cards[1]}"]`);
    const picked = await until(() => window.__riddle.lineage.packages.temperament, "a temperament picked");
    check(cards.length === 3 && picked === cards[1], `heir 3: three temperament cards, one tap wears one (${cards.join(" · ")} → ${picked})`);
    await closeSheets();
  }

  // ---- quest: one goal, the reward a picture, progress, the day's swap, QUEST DONE
  if (part("quest")) {
    await boot(3003);
    check(!(await tiles()).includes("quest"), `no quest board before the Warlord slain (${(await tiles()).join(" ")})`);
    const oath0 = await page.evaluate(() => !!document.querySelector(".oath-tab:not([hidden])"));
    await pastWarlord();
    const t = await tiles();
    const oath = await page.evaluate(() => !!document.querySelector(".oath-tab:not([hidden]), .shaft-oath-host:not([hidden]) *"));
    check(t.some((x) => x.startsWith("quest")) && !oath && !oath0, `the Warlord slain: the quest tile; no oath UI in play (${t.join(" ")}; oath ${oath0 || oath})`);
    await page.click(`.cmd .tile[data-tile="quest"]`);
    await until(() => !!document.querySelector(".quest-panel .quest-goal"), "the quest board");
    const Q = await page.evaluate(() => ({ goal: document.querySelector(".quest-goal").textContent, img: document.querySelector(".quest-reward img")?.getAttribute("src"), bar: document.querySelector(".quest-bar .fill")?.style.width,
      swap: document.querySelector(".quest-swap")?.textContent, stake: /stake|forswear|swear|oath|price/i.test(document.querySelector(".quest-panel").textContent) }));
    check(words(Q.goal) <= BUDGET.quest_goal.maxWords, `one plain goal ≤ ${BUDGET.quest_goal.maxWords} words (\`${Q.goal}\`)`);
    check(/^\/ui\/quest\/\w+\.webp$/.test(Q.img ?? ""), `the reward is a picture (${Q.img})`);
    check(/^\d+%$/.test(Q.bar ?? "") && Q.bar !== "0%", `a progress bar (${Q.bar})`);
    check(Q.swap === "new quest" && !Q.stake, `one free swap, no stake (${Q.swap}; stake words ${Q.stake})`);
    await shot("quest");
    await page.click(".quest-swap");
    const sw = await until(() => { const b = document.querySelector(".quest-swap"); return b?.disabled ? b.textContent : null; }, "the swap spent");
    const second = await page.evaluate(async () => { try { await window.__riddle.engine.swapQuest(); return "swapped"; } catch (e) { return String(e.message ?? e); } });
    check(sw === "swapped today" && /swapped today/.test(second), `the day's one swap spent (${sw}; a second: ${second})`);
    await closeSheets();
    // kept: the board stamps it, the report's beat says it
    const qd = await page.evaluate(() => Number(/D(\d+)/.exec(window.__riddle.lineage.town.quest.goal)[1]));
    await withState(new Function("e", `e.st30.qbest = ${qd};`));
    await page.click(`.cmd .tile[data-tile="quest"]`);
    const stamp = await until(() => document.querySelector(".quest-panel .quest-stamp")?.textContent, "QUEST DONE on the board");
    check(stamp === "QUEST DONE", `a kept quest reads \`QUEST DONE\` on the board (${stamp})`);
    await shot("quest-done");
    await closeSheets();
    // the beat: an absence that keeps it
    await withState((e) => { e.st30.qd = 1; e.st30.qbest = 0; e.st30.qdone = false; });
    const beat = await page.evaluate(async () => { const r = window.__riddle; const rep = await r.engine.runOffline(1800); return rep.packages ?? []; });
    check(beat.some((b) => /^QUEST DONE · /.test(b)), `the report carries the \`QUEST DONE\` beat (${beat.join(" | ")})`);
  }

  // ---- pen: no editor, no rule tablets before it; the editor once it opens
  if (part("pen")) {
    await boot(3004);
    await withState((e) => { e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 4, cause: "goblin" }]; e.lineage.best_depth = 6; e.lineage.gold = 80; });
    const pre = await page.evaluate(() => ({
      editor: [...document.querySelectorAll(".camp .editor")].some((x) => x.offsetParent !== null),
      tablets: [...document.querySelectorAll(".camp .tablets .row.tablet:not(.pkg-tab)")].filter((x) => x.offsetParent !== null && !x.classList.contains("orders-tab") && !x.classList.contains("start-tab") && !x.classList.contains("cage-tab")).length,
      edit: !!document.querySelector('.cmd .tile[data-tile="edit"]'), prepen: document.querySelector(".camp")?.classList.contains("prepen"),
      notches: [...document.querySelectorAll(".camp .shaft .notch")].filter((n) => n.offsetParent !== null).length,
      pen: window.__riddle.lineage.packages.pen_open }));
    check(!pre.pen && !pre.editor && pre.tablets === 0 && !pre.edit, `before the pen: no editor, no rule tablets, no edit tile (editor ${pre.editor}, ${pre.tablets} tablets, edit ${pre.edit})`);
    check(pre.prepen && pre.notches === 1, `before the pen the forecast is one headline notch (${pre.notches})`);
    await shot("camp-prepen");
    await withState((e) => { e.lineage.best_depth = 14; });
    const post = await page.evaluate(() => ({
      editor: [...document.querySelectorAll(".camp .editor")].some((x) => x.offsetParent !== null),
      tablets: [...document.querySelectorAll(".camp .editor .row.tablet")].filter((x) => x.offsetParent !== null).length,
      edit: !!document.querySelector('.cmd .tile[data-tile="edit"]'), strip: [...document.querySelectorAll(".pkg-strip")].some((x) => x.offsetParent !== null), pen: window.__riddle.lineage.packages.pen_open }));
    check(post.pen && post.editor && post.tablets > 0 && post.edit && !post.strip, `the pen opened: the editor, its tablets and the edit tile (editor ${post.editor}, ${post.tablets} tablets, edit ${post.edit}, strip ${post.strip})`);
    await page.click('.cmd .tile[data-tile="edit"]');
    await sleep(300);
    await shot("pen-open");
    // a row written below the packages is taken into the core's order: the pen's rows above every package, the client's copy the same
    await page.evaluate(() => { const r = window.__riddle; r.insertRow({ conds: [{ k: "hp<", n: 50 }], verb: { v: "rest" } }, r.rules.rows.length); });
    const ord = await until(async () => { const r = window.__riddle; const L = await r.engine.lineage(); const core = L.sets[L.active_set].rows.map((x) => x.verb.v + (x.origin ?? "")); const mine = r.rules.rows.map((x) => x.verb.v + (x.origin ?? ""));
      return core[0]?.startsWith("rest") && JSON.stringify(core) === JSON.stringify(mine) ? { core, mine, tab: document.querySelector('.camp .editor .row.tablet[data-i="0"]')?.textContent ?? "" } : null; }, "the core's order re-adopted", 8000);
    check(ord && /rest/.test(ord.tab), `the pen's row sits above the packages, the editor in the core's order (${ord?.mine.join(" · ")})`);
    const sh = await page.evaluate(() => { const r = window.__riddle; return { rows: r.rules.rows.length, src: r.lineage.packages.rows?.length }; });
    check(sh.rows === sh.src, `the rows and the core's row sources line up (${sh.rows} · ${sh.src})`);
  }

  // ---- death: before the pen, the cause and exactly one lever
  if (part("death")) {
    await boot(3005);
    await withState((e) => { e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 4, cause: "goblin" }]; e.lineage.best_depth = 6; });
    const base = { run_id: 7, depth: 6, cause: "goblin_archer", margin: "3 over", verdict: "gap", baseline: 0.42, replays: 12, lean: "dice", trace: { turns: [] },
      patches: [{ row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "retreat" } }, insert_at: 0, survive: 0.9, forecast_delta: 0.2, forecast_depth: 6 }], morgue: "slain" };
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: { ...d, lever: { kind: "wait", text: "Steady L2" }, package: "Steady · HP<20% → return" } }), base);
    await until(() => window.__riddle.screen === "death" && document.querySelector(".death-lever"), "the pre-pen death");
    const D = await page.evaluate(() => ({ levers: document.querySelectorAll(".death-lever").length, lever: document.querySelector(".death-lever")?.textContent, patches: [...document.querySelectorAll(".death button.patch")].filter((x) => x.offsetParent !== null).length,
      trace: !!document.querySelector(".death .trace-panel"), more: !!document.querySelector(".death .death-more"), seal: document.querySelector(".death .verdict")?.textContent,
      text: document.querySelector(".death .well").textContent, gem: document.querySelector(".gem")?.textContent, why: document.querySelector(".death-why")?.textContent }));
    check(D.levers === 1 && D.patches === 0, `before the pen the death shows exactly one lever (${D.levers} lever: \`${D.lever}\`, ${D.patches} fixes)`);
    check(!D.trace && !D.more && D.seal === "you died" && !/replays|1 in \d|luck|no rule for it/.test(D.text), `no trace, no details, no gap/dice, no replays (trace ${D.trace}, details ${D.more}, seal ${D.seal})`);
    check(/goblin archer · D6/.test(D.text) && D.why === "Steady · HP<20% → return" && D.gem === "send", `the cause, the package row that acted, the gem the lever's act (${D.why}; gem ${D.gem})`);
    await shot("death-prepen");
    // spend · package: each one tablet, its act
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: { ...d, lever: { kind: "spend", text: "sword +2" } } }), base);
    const sp = await until(() => document.querySelector(".death-lever")?.dataset.kind === "spend" ? document.querySelector(".gem")?.textContent : null, "a spend lever");
    check(sp === "forge" && (await page.evaluate(() => document.querySelectorAll(".death-lever").length)) === 1, `a spend lever opens the forge (${sp})`);
    // the pen open: the fixes, no lever
    await withState((e) => { e.lineage.best_depth = 14; });
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: d }), base);
    await until(() => window.__riddle.screen === "death" && document.querySelector(".death button.patch"), "the pen's death");
    const D2 = await page.evaluate(() => ({ levers: document.querySelectorAll(".death-lever").length, patches: document.querySelectorAll(".death button.patch").length, more: !!document.querySelector(".death .death-more") }));
    check(D2.levers === 0 && D2.patches >= 1 && D2.more, `with the pen: the fixes and the details, no lever (${D2.levers} levers, ${D2.patches} fixes)`);
  }

  // ---- report: leads with what grew on each track; the beats as plaques; slices merge
  if (part("report")) {
    await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3006&absent=8h`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.screen === "report" && document.querySelector(".report-sheet"), "the report", 60_000);
    const pend = await page.evaluate(() => [...document.querySelectorAll(".report-sheet .section, .report-sheet section")].filter((x) => /pending/i.test(x.querySelector(".label")?.textContent ?? "") && x.offsetParent !== null).map((x) => x.textContent).join(" | "));
    check(!/patch|fired|\bR\d/.test(pend), `before the pen the report's pending speaks no pen word (${pend || "none"})`);
    const dt = await page.evaluate(() => { const r = window.__riddle; const rep = r.lastReport ?? null; return document.querySelector('.report-sheet .tile.plaque[data-k="deaths"]')?.textContent ?? ""; });
    out.push(`note the deaths tile on Steady: ${dt.replace(/\s+/g, " ").trim()}`);
    const R = await page.evaluate(() => { const s = document.querySelector(".report-sheet"); const first = [...s.children].find((c) => c.offsetParent !== null); return { first: first?.className, lines: [...s.querySelectorAll(".grew-line")].map((l) => l.textContent), beats: [...s.querySelectorAll(".beat-plaque")].map((b) => b.textContent) }; });
    check(R.first === "report-summary" && R.lines.length >= 1, `the report leads with the summary; growth remains in details (${R.first}: ${R.lines.join(" | ")})`);
    await shot("report");
    const M = await page.evaluate(async () => {
      const { mergeGrew } = await import("/src/app.ts");
      return mergeGrew({ grew: [{ track: "items", what: "+$100" }, { track: "scale", what: "best D9" }, { track: "character", what: "xp" }], packages: ["STEADY L2", "+Guarded"] },
        { grew: [{ track: "items", what: "+$50" }, { track: "scale", what: "best D11" }, { track: "character", what: "xp" }], packages: ["STEADY L3", "QUEST DONE · reach D10"] });
    });
    check(JSON.stringify(M.grew) === JSON.stringify([{ track: "items", what: "+$150" }, { track: "scale", what: "best D11" }, { track: "character", what: "xp" }]) && JSON.stringify(M.packages) === JSON.stringify(["STEADY L3", "+Guarded", "QUEST DONE · reach D10"]),
      `slices merge: gold summed, the best and the level the highest (${JSON.stringify(M)})`);
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(e);
const errs = errors.filter((e) => !/favicon|net::ERR|Failed to load resource/.test(e));
console.log(`cut30: ${failed || errs.length ? "FAIL" : "ok"} (${out.length} checks${failed ? `, ${failed} failed` : ""}${errs.length ? `, ${errs.length} errors` : ""})`);
process.exit(failed || errs.length ? 1 : 0);
