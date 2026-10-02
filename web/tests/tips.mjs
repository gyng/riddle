#!/usr/bin/env node
// docs/TOOLTIPS.md gates — keyword marks and tips, on the fake engine, headless, phone (400 × 800 and 360 × 740, touch) and desktop
// (1440 × 900, mouse):
//   density   on every main screen at several stages (day 0, mid-game, the packages / tracks / quest panels, the pen open, the report,
//             the death screen): a keyword marked once per panel, ≤ 1 marked per line, ≤ 4 on screen, ≤ 8 % of the visible words
//             (1 on a screen under 25 words); every marked keyword has its tip
//   copy      every registry term renders a tip of ≤ 10 words (gloss and live value), no sentence, no forbidden word
//   controls  a tap on a keyword inside a button runs the button and opens no tip; a long-press shows the tip and runs nothing
//   fade      two opens make a keyword plain; its tip still opens (hover / long-press)
//   one       at most one plate, ever
//   layout    opening a tip shifts nothing; the plate stays inside a 360 px viewport
//
//   node web/tests/tips.mjs [--shots dir]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync, readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
if (shots) mkdirSync(shots, { recursive: true });
const BUDGET = JSON.parse(readFileSync(resolve(ROOT, "eval/copy-budgets.json"), "utf8")).surfaces.tooltip;
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = await launchBrowser();
const VIEWS = [
  { name: "phone", viewport: { width: 400, height: 800 }, touch: true },
  { name: "phone360", viewport: { width: 360, height: 740 }, touch: true },
  { name: "desktop", viewport: { width: 1440, height: 900 }, touch: false },
];

/** In the page: the density rules over what is on screen now. */
const DENSITY = () => {
  const SCOPE = ".kw-tip, .sheet, .panel, .report-sheet, .death, .topbar, footer.console, section, aside, .town";
  const vw = document.documentElement.clientWidth, vh = innerHeight;
  const onScreen = (el) => {
    const r = el.getBoundingClientRect();
    if (!(r.width > 0 && r.height > 0 && r.bottom > 0 && r.right > 0 && r.top < vh && r.left < vw)) return false;
    if (el.checkVisibility && !el.checkVisibility({ visibilityProperty: true, opacityProperty: true })) return false;
    const hit = document.elementFromPoint(Math.min(Math.max(r.left + r.width / 2, 0), vw - 1), Math.min(Math.max(r.top + r.height / 2, 0), vh - 1));
    return !hit || el.contains(hit) || hit.contains(el) || !!hit.closest(".kw-tip, .concept-cap");
  };
  let words = 0;
  const w = document.createTreeWalker(document.body, NodeFilter.SHOW_TEXT), memo = new Map();
  for (let t = w.nextNode(); t; t = w.nextNode()) {
    const p = t.parentElement; if (!p || /^(SCRIPT|STYLE)$/.test(p.tagName) || p.closest(".kw-tip, .vh, [aria-hidden=true]")) continue;
    if (!memo.has(p)) memo.set(p, onScreen(p));
    if (memo.get(p)) words += (t.textContent ?? "").split(/\s+/).filter((x) => /\p{L}/u.test(x)).length;
  }
  const marked = [...document.querySelectorAll(".kw.kw-on")].filter(onScreen);
  const bad = [];
  for (const m of marked) {
    const scope = m.closest(SCOPE) ?? document.body;
    const first = [...scope.querySelectorAll(`.kw[data-kw="${m.dataset.kw}"]`)].find((k) => (k.closest(SCOPE) ?? document.body) === scope);
    if (first !== m) bad.push(`${m.dataset.kw} not its panel's first`);
    if (!window.__tips.terms().includes(m.dataset.kw)) bad.push(`${m.dataset.kw} has no tip`);
  }
  for (let i = 0; i < marked.length; i++) for (let j = i + 1; j < marked.length; j++) {
    const a = marked[i].getBoundingClientRect(), b = marked[j].getBoundingClientRect();
    if (Math.min(a.bottom, b.bottom) - Math.max(a.top, b.top) > Math.min(a.height, b.height) / 2) bad.push(`${marked[i].dataset.kw} and ${marked[j].dataset.kw} on one line`);
  }
  const cap = words < 25 ? 1 : Math.min(4, Math.floor(words * 0.08));
  if (marked.length > cap) bad.push(`${marked.length} marked > ${cap} (${words} words)`);
  return { n: marked.length, words, terms: marked.map((m) => m.dataset.kw), bad };
};

for (const V of VIEWS) {
  const ctx = await browser.newContext({ viewport: V.viewport, hasTouch: V.touch, isMobile: false });
  const page = await ctx.newPage();
  page.on("console", (m) => { if (m.type() === "error") errors.push(`${V.name} console.error: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`${V.name} pageerror: ${e.message}`));
  const until = async (pred, label, timeout = 20_000, arg) => { const t = Date.now(); while (Date.now() - t < timeout) { const v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); } throw new Error(`${V.name}: timeout waiting for ${label}`); };
  const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp", "camp"); await sleep(300); };
  const withState = (fn) => page.evaluate(async (src) => { const r = window.__riddle; const save = JSON.parse(await r.engine.save()); new Function("e", src)(save); r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" }); }, `(${fn})(e)`);
  const boot = async (seed, q = "") => {
    await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=${seed}${q}`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.evaluate(() => { for (const k of Object.keys(localStorage)) if (/^riddle\.(reveal|tracks|keywords|concepts)/.test(k)) localStorage.removeItem(k); window.__tips.reset(); });
  };
  const settle = async () => { await sleep(1400); await page.evaluate(() => window.__tips.pass()); };
  const density = async (stage) => {
    await settle();
    const d = await page.evaluate(DENSITY);
    check(!d.bad.length, `${V.name} · ${stage}: density holds (${d.n} marked of ${d.words} words: ${d.terms.join(" ") || "none"}${d.bad.length ? ` — ${d.bad.join("; ")}` : ""})`);
    if (shots) await page.screenshot({ path: resolve(shots, `${stage}-${V.name}.png`) });
    return d;
  };
  const day0 = (e) => { e.lineage.heir = 1; e.lineage.gold = 0; e.lineage.kennel = []; e.lineage.eggs = []; e.lineage.party = []; e.lineage.vault = []; e.lineage.graveyard = []; e.lineage.best_depth = 0; e.lineage.facts = []; };
  const mid = (e) => { e.lineage.best_depth = 9; e.lineage.gold = 120; e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 6, cause: "goblin" }]; e.st30.runs = { steady: 45 }; };
  /** A long-press (touch) or a hover (mouse) on `sel`. */
  const longPress = async (sel) => {
    const r = await page.locator(sel).first().boundingBox();
    const x = r.x + r.width / 2, y = r.y + r.height / 2;
    if (!V.touch) { await page.mouse.move(x, y); await sleep(600); return; }
    const cdp = await ctx.newCDPSession(page);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchStart", touchPoints: [{ x, y }] });
    await sleep(650);
    await cdp.send("Input.dispatchTouchEvent", { type: "touchEnd", touchPoints: [] });
    await sleep(200);
  };
  const tapOrClick = (sel) => (V.touch ? page.locator(sel).first().tap() : page.locator(sel).first().click());
  const closeTip = () => page.evaluate(() => window.__tips.close());
  const tipOpen = () => page.evaluate(() => window.__tips.open());

  try {
    // ---- day 0
    await boot(3001);
    await withState(day0); await camp();
    const d0 = await density("day0");
    check(d0.n >= 1, `${V.name} · day 0 carries a mark (${d0.terms.join(" ")})`);

    // ---- copy: every term's tip within budget (the live values of a mid-game lineage)
    if (V.name === "phone") {
      await withState(mid); await camp();
      const tips = await page.evaluate(() => window.__tips.terms().map((t) => ({ t, g: window.__tips.gloss(t) })));
      const words = (s) => s.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length;
      const over = tips.filter((x) => words(x.g) > BUDGET.maxWords || /\p{L}[.!?]$/u.test(x.g.trim()) || /\b(you|your|click|tap)\b/i.test(x.g));
      check(tips.length >= 30 && !over.length, `every term's tip ≤ ${BUDGET.maxWords} words, a fragment (${tips.length} terms; longest ${Math.max(...tips.map((x) => words(x.g)))}${over.length ? `; over: ${over.map((x) => `${x.t} \`${x.g}\``).join(", ")}` : ""})`);
    }

    // ---- mid-game camp, the panels
    await withState(mid); await camp();
    await density("midgame");
    await page.locator('.cmd .tile[data-tile="packages"]').click();
    await until(() => !!document.querySelector(".pkg-panel .pkg-slot"), "the packages panel");
    await until(() => !document.querySelector(".pkg-panel .pkg-price.pending"), "the prices", 10_000).catch(() => {});
    const dp = await density("packages");
    check(dp.terms.some((t) => ["package", "stance", "tactic", "reach", "drill"].includes(t)), `${V.name} · the packages panel marks its words (${dp.terms.join(" ")})`);

    // one plate; a second tip replaces the first; no layout shift; inside the viewport
    const kws = await page.evaluate(() => [...document.querySelectorAll(".pkg-panel .kw.kw-on")].map((k) => k.dataset.kw));
    if (kws.length >= 2) {
      const rects = () => page.evaluate(() => [...document.querySelectorAll(".pkg-panel *, .topbar *, footer.console *")].map((e) => { const r = e.getBoundingClientRect(); return [r.x, r.y, r.width, r.height]; }));
      const before = await rects();
      await tapOrClick(`.pkg-panel .kw.kw-on[data-kw="${kws[0]}"]`); await sleep(250);
      const a = await tipOpen();
      const after = await rects();
      const shift = before.length === after.length ? Math.max(0, ...before.map((r, i) => Math.max(...r.map((v, k) => Math.abs(v - after[i][k]))))) : Infinity;
      check(a === kws[0] && shift < 1, `${V.name} · a tap on a marked word opens its tip, nothing moves (${a}; shift ${shift} px)`);
      const box = await page.evaluate(() => { const r = document.querySelector(".kw-tip:not([hidden])")?.getBoundingClientRect(); return r ? { l: r.left, r: r.right, t: r.top, b: r.bottom, vw: document.documentElement.clientWidth, vh: innerHeight } : null; });
      check(box && box.l >= 0 && box.r <= box.vw && box.t >= 0 && box.b <= box.vh, `${V.name} · the plate is inside the viewport (${box ? `${Math.round(box.l)}–${Math.round(box.r)} of ${box.vw}` : "none"})`);
      if (shots) await page.screenshot({ path: resolve(shots, `packages-tip-${V.name}.png`) });
      await tapOrClick(`.pkg-panel .kw.kw-on[data-kw="${kws[1]}"]`); await sleep(250);
      const b = await tipOpen(), n = await page.evaluate(() => window.__tips.plates() + document.querySelectorAll(".kw-tip").length - 1);
      check(b === kws[1] && n === 1, `${V.name} · one plate at a time (${b} open, ${n} plate)`);
      // nested: a keyword inside the tip adds its gloss in the same plate
      const nest = await page.evaluate(() => !!document.querySelector(".kw-tip .kw-n"));
      if (nest) { await tapOrClick(".kw-tip .kw-n"); await sleep(V.touch ? 200 : 500); }
      const sub = await page.evaluate(() => ({ subs: document.querySelectorAll(".kw-tip .kw-tip-sub").length, plates: document.querySelectorAll(".kw-tip:not([hidden])").length, nested2: document.querySelectorAll(".kw-tip .kw-tip-sub .kw-n").length }));
      if (nest) check(sub.subs === 1 && sub.plates === 1 && sub.nested2 === 0, `${V.name} · a keyword in a tip adds one line in the same plate, one level (${JSON.stringify(sub)})`);
      // a tap elsewhere closes it
      if (V.touch) { const p = await page.evaluate(() => { const r = document.querySelector('.pkg-sec[data-kind="tactic"] .pkg-head').getBoundingClientRect(); return { x: Math.round(r.right - 12), y: Math.round(r.top + r.height / 2) }; }); await page.touchscreen.tap(p.x, p.y); await sleep(200); check((await tipOpen()) === null, `${V.name} · a tap elsewhere closes the tip`); }
      await closeTip();
    } else check(false, `${V.name} · two marked words on the packages panel to test with (${kws.join(" ")})`);
    await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); });
    await sleep(300);

    // ---- a keyword inside a button: the tap is the button's; a long-press (hover) the tip's
    await page.evaluate(() => window.__tips.reset());
    await camp();
    const inBtn = await page.evaluate(() => !!document.querySelector("button .kw"));
    if (inBtn) {
      await tapOrClick("button.pkg-tab .kw");
      const opened = await until(() => !!document.querySelector(".pkg-panel"), "the panel from a keyword in its button", 4000).catch(() => false);
      check(opened && (await tipOpen()) === null, `${V.name} · a tap on a keyword inside a button runs the button, no tip (panel ${!!opened})`);
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); }); await sleep(300);
    } else check(false, `${V.name} · a keyword inside a button on the mid-game camp`);
    await closeTip();
    await longPress('.cmd .tile[data-tile="packages"]');
    const lp = { tip: await tipOpen(), panel: await page.evaluate(() => !!document.querySelector(".pkg-panel")) };
    check(lp.tip === "package" && !lp.panel, `${V.name} · a ${V.touch ? "long-press" : "hover"} on a tile shows its tip and runs nothing (${JSON.stringify(lp)})`);
    await closeTip(); await sleep(150);

    // ---- fade: two opens make a word plain; its tip stays a hover / long-press away
    if (V.name !== "desktop") {
      await page.locator('.cmd .tile[data-tile="packages"]').click();
      await until(() => !!document.querySelector(".pkg-panel .kw"), "the packages panel");
      await settle();
      const t = await page.evaluate(() => document.querySelector(".pkg-panel .kw.kw-on")?.dataset.kw);
      const sel = `.pkg-panel .kw[data-kw="${t}"]`;
      for (let i = 0; i < 2; i++) { await tapOrClick(sel); await sleep(200); await closeTip(); await sleep(150); }
      await settle();
      const plain = await page.evaluate((s) => !document.querySelector(s)?.classList.contains("kw-on"), sel);
      await longPress(sel);
      const still = await tipOpen();
      check(t && plain && still === t, `${V.name} · two opens fade \`${t}\` to plain text; a long-press still opens it (plain ${plain}, ${still})`);
      await closeTip();
      await tapOrClick(sel); await sleep(250);
      check((await tipOpen()) === null, `${V.name} · a plain word does not take a tap (${await tipOpen()})`);
      await closeTip();
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); }); await sleep(300);
      await page.evaluate(() => window.__tips.reset());
    }

    // ---- the works sheet (Cut 30.5: the tracks panel's successor, opened from the `next` pill) and the quest panel
    if (await page.locator(".next-pill:not([hidden])").count()) {
      await page.locator(".next-pill").click();
      await until(() => !!document.querySelector(".works-sheet"), "the works sheet");
      await density("works");
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); }); await sleep(300);
    }
    if (await page.locator('.cmd .tile[data-tile="quest"]').count()) {
      await page.locator('.cmd .tile[data-tile="quest"]').click();
      await until(() => !!document.querySelector(".quest-panel"), "the quest panel");
      await density("quest");
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); }); await sleep(300);
    }

    // ---- the pen open: the editor and the forecast panel
    await withState((e) => { e.lineage.best_depth = 14; }); await camp();
    if (await page.locator('.cmd .tile[data-tile="edit"]').count()) { await page.locator('.cmd .tile[data-tile="edit"]').click(); await sleep(300); }
    await density("pen");
    const prio = await page.evaluate(() => !!document.querySelector('.kw[data-kw="priority"]'));
    check(prio, `${V.name} · the pen's priority head is a keyword`);
    const cond = page.locator(".editor .row .chip.cond").first();
    if (await cond.count()) {
      await cond.click(); await sleep(300);
      await density("pen-condition");
      await page.evaluate(() => { for (const s of document.querySelectorAll(".sheet .close-stud, .sheet .sheet-x")) s.click(); }); await sleep(300);
    }

    // ---- the death screen (its seal and lever carry tips, nothing marked at rest)
    const base = { run_id: 7, depth: 6, cause: "goblin_archer", margin: "3 over", verdict: "gap", baseline: 0.1, replays: 12, trace: { turns: [] },
      patches: [{ row: { conds: [{ k: "hp<", n: 40 }], verb: { v: "retreat" } }, insert_at: 0, survive: 0.9, forecast_delta: 0.2, forecast_depth: 6 }], morgue: "slain" };
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: { ...d, verdict: "order", cause_row: 0, order_over: 0 } }), base);
    await until(() => window.__riddle.screen === "death" && document.querySelector(".death .verdict"), "the death");
    await density("death");
    await longPress(".death .verdict");
    check((await tipOpen()) === "v_order", `${V.name} · the seal's verdict is a tip on ${V.touch ? "long-press" : "hover"} (${await tipOpen()})`);
    await closeTip();
    await withState((e) => { e.lineage.best_depth = 6; });
    await page.evaluate((d) => window.__riddle.go({ kind: "death", death: { ...d, lever: { kind: "wait", text: "Steady L2" } } }), base);
    await until(() => document.querySelector(".death-lever"), "the pre-pen death");
    await density("death-prepen");
    await longPress(".death-lever");
    const lv = { tip: await tipOpen(), screen: await page.evaluate(() => window.__riddle.screen) };
    check(lv.tip === "lever" && lv.screen === "death", `${V.name} · the lever's tip on ${V.touch ? "long-press" : "hover"}, the lever not pulled (${JSON.stringify(lv)})`);
    await closeTip();

    // ---- the report
    await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3006&absent=8h`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.screen === "report" && document.querySelector(".report-sheet"), "the report", 60_000);
    await page.evaluate(() => { localStorage.removeItem("riddle.keywords"); window.__tips.reset(); });
    await sleep(1500);
    const dr = await density("report");
    check(dr.n >= 1, `${V.name} · the report marks its words (${dr.terms.join(" ")})`);
    if (shots) {
      const k = page.locator(".report-sheet .kw.kw-on").first();
      if (await k.count()) { await (V.touch ? k.tap() : k.hover()); await sleep(600); await page.screenshot({ path: resolve(shots, `report-tip-${V.name}.png`) }); await closeTip(); }
    }
  } catch (e) { errors.push(`${V.name}: ${e.message}`); }
  await ctx.close();
}
await browser.close();
for (const l of out) console.log(l);
for (const e of errors) console.log(`FAIL ${e}`);
const bad = failed + errors.length;
console.log(`tips: ${out.length - failed}/${out.length} pass${errors.length ? `, ${errors.length} errors` : ""}`);
process.exit(bad ? 1 : 0);
