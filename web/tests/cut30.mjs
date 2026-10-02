#!/usr/bin/env node
// Cut 30 client gates (docs/CUT30.md), on the fake engine (its Cut 30 stand-ins), headless at 400 × 800 (RIDDLE_BROWSER=headed for the GPU).
//
// Panels section (§2, §4, §5, the Reveal):
//   tracks    four tracks on the panel (opened from the portrait-mini), a next stage on every track until its v1 stages are done, a bar
//             when the trigger is numeric; the mini is no control on day 0 and glints when a stage opens
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
  await page.evaluate(() => { localStorage.removeItem("riddle.reveal"); localStorage.removeItem("riddle.tracks.seen"); });
};
const tiles = () => page.evaluate(() => [...document.querySelectorAll(".cmd .tile:not(.empty)")].map((t) => t.dataset.tile + (t.classList.contains("reveal") ? "*" : "")));
const closeSheets = () => page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); });
/** A lineage past the Warlord: the second stance, the tactics, the quest board (the fake: D9). */
const pastWarlord = (extra = "") => withState(new Function("e", `e.lineage.best_depth = 9; e.lineage.gold = 120; e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 6, cause: "goblin" }]; ${extra}`));

try {
  // ---- tracks: four rows, a next stage on every track until its stages are done; the mini opens the panel
  if (part("tracks")) {
    await boot(3001);
    // (the fake's fresh lineage carries a chronicle, a purse and a kennel: day 0 is set by hand)
    await withState((e) => { e.lineage.heir = 1; e.lineage.gold = 0; e.lineage._k = e.lineage.kennel; e.lineage.kennel = []; e.lineage.eggs = []; e.lineage.party = []; e.lineage.vault = []; e.lineage.graveyard = []; e.lineage.best_depth = 0; e.lineage.facts = []; });
    const day0 = await page.evaluate(() => ({ btn: !!document.querySelector(".topbar button.mini-portrait"), n: window.__riddle.lineage.tracks?.length }));
    check(day0.n === 4 && !day0.btn, `day 0: four tracks on the wire, the portrait-mini is no control yet (${day0.n} tracks, button ${day0.btn})`);
    await withState((e) => { e.lineage.best_depth = 5; e.lineage.gold = 260; });
    const mini = await until(() => !!document.querySelector(".topbar button.mini-portrait.tracks-btn"), "the mini as a button");
    const glint = await page.evaluate(() => document.querySelector(".topbar button.mini-portrait")?.classList.contains("reveal"));
    check(mini && glint, `a stage opened: the portrait-mini opens the tracks and glints once (${glint})`);
    await page.click(".topbar button.mini-portrait");
    await until(() => document.querySelectorAll(".tracks-panel .track-row").length === 4, "the tracks panel");
    const rows = await page.evaluate(() => [...document.querySelectorAll(".tracks-panel .track-row")].map((r) => ({ id: r.dataset.track, stage: r.querySelector(".track-stage")?.textContent ?? "", next: r.querySelector(".track-next:not(.done)")?.textContent ?? "", bar: !!r.querySelector(".track-bar"), ico: !!r.querySelector(".track-ico .ico") })));
    const wire = await page.evaluate(() => window.__riddle.lineage.tracks);
    check(JSON.stringify(rows.map((r) => r.id)) === JSON.stringify(["character", "items", "scale", "town"]), `four tracks in order: ${rows.map((r) => r.id).join(" · ")}`);
    check(rows.every((r) => r.ico && r.stage && words(r.stage) <= 3), `each row an icon and its stage (${rows.map((r) => r.stage).join(" · ")})`);
    check(rows.every((r) => { const w = wire.find((t) => t.id === r.id); return w.next ? r.next === `next · ${w.next}${w.trigger ? ` · ${w.trigger}` : ""}` : !r.next; }), `every track not done shows \`next · <stage> · <trigger>\` (${rows.map((r) => r.next || "✓").join(" | ")})`);
    const town = rows.find((r) => r.id === "town"), townW = wire.find((t) => t.id === "town");
    check(townW.next === "bank" ? town.bar : true, `a numeric trigger draws its bar (town → bank: ${town.bar})`);
    await shot("tracks");
    await closeSheets();
    // every v1 stage done on a track: no next there; the others keep theirs
    await withState((e) => { e.lineage.gold = 900; e.lineage.vault = [{ id: 1, kind: "sword", label: "sword", known: true }]; e.lineage.kennel = e.lineage._k; });
    await page.click(".topbar button.mini-portrait");
    await until(() => document.querySelectorAll(".tracks-panel .track-row").length === 4, "the tracks panel again");
    const r2 = await page.evaluate(() => [...document.querySelectorAll(".tracks-panel .track-row")].map((r) => ({ id: r.dataset.track, next: !!r.querySelector(".track-next:not(.done)"), done: !!r.querySelector(".track-next.done") })));
    const w2 = await page.evaluate(() => window.__riddle.lineage.tracks);
    check(r2.every((r) => r.next === !!w2.find((t) => t.id === r.id).next) && r2.find((r) => r.id === "town").done, `a track whose v1 stages are done shows none; the rest keep a next stage (${r2.map((r) => `${r.id}:${r.next ? "next" : "done"}`).join(" ")})`);
    await closeSheets();
  }

  // ---- packages: slots, levels, shadowing, the one-line price, equipping free and instant
  if (part("packages")) {
    await boot(3002);
    check(!(await tiles()).includes("packages"), `day 0: no packages tile (${(await tiles()).join(" ")})`);
    const strip0 = await page.evaluate(() => [...document.querySelectorAll(".pkg-strip .pkg-tab")].map((t) => ({ text: t.querySelector(".pkg-name")?.textContent, btn: t.tagName === "BUTTON" })));
    check(strip0.length === 1 && strip0[0].text === "Steady L1" && !strip0[0].btn, `day 0: the worn stance as a plaque on the camp (${JSON.stringify(strip0)})`);
    await pastWarlord();
    await withState((e) => { e.st30.runs = { steady: 45 }; e.rules.rows.push({ ...e.rules.rows[0] }); e.lineage.sets[0].rows.push({ ...e.lineage.sets[0].rows[0] }); });
    const t1 = await tiles();
    check(t1.some((t) => t.startsWith("packages")), `the second stance arrived: the packages tile (${t1.join(" ")})`);
    await page.click(`.cmd .tile[data-tile="packages"]`);
    await until(() => !!document.querySelector(".pkg-panel .pkg-slot"), "the packages panel");
    await until(() => !document.querySelector(".pkg-panel .pkg-price.pending"), "the prices", 10_000);
    const P = await page.evaluate(() => {
      const q = (s) => [...document.querySelectorAll(s)];
      return {
        head: document.querySelector(".pkg-headline")?.textContent ?? "",
        stance: document.querySelector('.pkg-sec[data-kind="stance"] .pkg-slot .chip.pkg.on .pkg-name')?.textContent,
        bar: document.querySelector('.pkg-sec[data-kind="stance"] .pkg-slot .lvl-bar .fill')?.style.width,
        alts: q('.pkg-sec[data-kind="stance"] .chip.pkg.alt').map((c) => ({ name: c.querySelector(".pkg-name").textContent, price: [...c.querySelectorAll(".pkg-price")].map((x) => x.textContent) })),
        tactic: q('.pkg-sec[data-kind="tactic"] .pkg-slot').length, tAlts: q('.pkg-sec[data-kind="tactic"] .chip.pkg.alt').length,
        temper: !!document.querySelector('.pkg-sec[data-kind="temperament"]'),
        shadow: q(".pkg-row.shadowed .pkg-row-src").map((x) => x.textContent),
        drills: q(".pkg-drill").map((d) => d.textContent),
      };
    });
    check(/^reach D\d+ (\d+%|<\d+%|>\d+%)$/.test(P.head), `the panel's head is the forecast's one headline (\`${P.head}\`)`);
    check(P.stance === "Steady L3" && P.bar && P.bar !== "0%", `the worn stance's chip \`Steady L3\` and its level bar (${P.stance}, ${P.bar})`);
    check(P.alts.length >= 2 && P.alts.every((a) => a.price.length === 1 && /^(death|past|bank) [+−]\d+$|^same$/.test(a.price[0])), `each other stance priced in one line (${P.alts.map((a) => `${a.name} · ${a.price.join("|")}`).join(", ")})`);
    check(P.tactic === 1 && P.tAlts >= 1, `one tactic slot at the Warlord slain, the tactics to wear (${P.tactic} slot, ${P.tAlts} tactics)`);
    check(!P.temper, `no temperament before heir 3 (${P.temper})`);
    check(P.shadow.length >= 1 && P.shadow.every((s) => /^\S.* wins$/.test(s)), `a shadowed row greyed with its winner (${P.shadow.join(", ")})`);
    check(P.drills.some((d) => /drill · warlord/.test(d)), `the Warlord's drill shown (${P.drills.join(", ")})`);
    await page.click(".pkg-sec[data-kind=\"stance\"] .pkg-rows-btn, .pkg-rows-btn").catch(() => undefined);
    await shot("packages");
    // equip: free and instant
    const g0 = await page.evaluate(() => window.__riddle.lineage.gold);
    const t0 = Date.now();
    await page.click('.pkg-sec[data-kind="stance"] .chip.pkg.alt[data-pkg="guarded"]');
    await until(() => window.__riddle.lineage.packages.stance === "guarded" && document.querySelector('.pkg-sec[data-kind="stance"] .pkg-slot .chip.pkg.on .pkg-name')?.textContent === "Guarded L1", "Guarded worn");
    const ms = Date.now() - t0, g1 = await page.evaluate(() => window.__riddle.lineage.gold);
    check(g1 === g0 && ms < 1500, `equipping is free and instant ($${g0} → $${g1}, ${ms} ms)`);
    // a tactic worn in its slot
    await page.click('.pkg-sec[data-kind="tactic"] .chip.pkg.alt');
    const tw = await until(() => window.__riddle.lineage.packages.tactics?.length === 1 && document.querySelector('.pkg-sec[data-kind="tactic"] .chip.pkg.on .pkg-name')?.textContent, "a tactic worn");
    check(/ L\d$/.test(tw), `a tactic worn reads \`<name> L<n>\` (${tw})`);
    // revoke the drill: one tap, it stays
    await page.click(".pkg-drill .drill");
    const rv = await until(() => window.__riddle.lineage.packages.drills?.find((d) => d.boss === "goblin_warlord")?.revoked === true && document.querySelector(".pkg-drill.revoked") ? true : null, "the drill revoked");
    check(rv, `a drill revoked with one tap stays revoked`);
    await closeSheets();
    // heir 3: the temperament slot, the wake's three cards (card 1 worn until picked)
    await withState((e) => { e.lineage.heir = 3; });
    await page.click(`.cmd .tile[data-tile="packages"]`);
    await until(() => !!document.querySelector('.pkg-sec[data-kind="temperament"]'), "the temperament slot");
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
    const R = await page.evaluate(() => { const s = document.querySelector(".report-sheet"); const first = [...s.children].find((c) => c.offsetParent !== null); return { first: first?.className, lines: [...s.querySelectorAll(".grew-line")].map((l) => l.textContent), beats: [...s.querySelectorAll(".beat-plaque")].map((b) => b.textContent) }; });
    check(R.first === "grew" && R.lines.length >= 1, `the report leads with what grew (${R.first}: ${R.lines.join(" | ")})`);
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
