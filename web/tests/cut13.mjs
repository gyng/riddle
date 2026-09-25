#!/usr/bin/env node
// Cut 13 gates, client side (docs/CUT13.md §1–§5), on the fake engine (`?engine=fake&dev=1`) through the browser harness
// (tools/browser.mjs; `--shots` runs headed on the GPU and writes scratchpad/cut13/*.png at 400×800×3) against the dev
// server (tools/dev.sh, :5219):
//   §1  the stake reads `stalling` (QA 92eb880: no `keeps $0` before the run ends) while the guard has fired (`Stake.stalling`, the fake's `?fake_stall=N` chore loop);
//       a run that comes home stalled gets the verdict screen: the `stall` pill, the headline, the notes, the trace, the patches;
//       an engine without the record falls back to the report (the exit line reads `returned $0 · … · stalled`); the report's
//       `open` shows a stall verdict too
//   §2  a new heir's trait is chosen: two chips beside `♟N`, the chosen one on, each with its rule under the name; a tap is
//       `setTrait`; the send empties the offer and the chips vanish
//   §3  the report's `spent` section (`heal ×16 · −$640`) and the gold line under the tiles (`+$412 banked · +$96 returned ·
//       +$45 salvage · −$640 spent`); the fake's `auto: restock` fills `spent` across an absence
//   §4  a situation's note cuts the fight frame in for SCENE_MS outside a fight (`fights` and `fast`), the note as the callout;
//       the death screen shows `Death.notes`; a 40-char callout renders whole at 400 px; two callouts on one tick queue
//   §5  an unlock delta within its ± reads `reach ~0`, otherwise `reach +5% ±3`; the first forecast paint's ± trails `…`, the
//       refine's does not; the ends line carries `death 5% ±4`; a `dice` death says `forecast said D4 100%` (the camp's reach, verbatim)
//   QA on 50bb162 (qaF): a card row's chips wrap inside 400 px with its × reachable; the forecast's `try` hint rides the track on
//       the depth's own line; the busy label has its own strip under the header; the ◆ readout repaints as a buy's lineage lands;
//       ▶▶| in `fast` is the run's end; a beat's line clears at a floor change and within 6 s; the gold sheet names its run
//       filter (`died D4` on · `all` off); the report counts stalls apart (`returned 2 · stalled 1`); a death's pile is named once
//   QA on 50bb162 (qaE): the report's TRACE sheet carries the exit line as its header and caps the provenance (the chain's own
//       links, the last 8 by tick, `· N earlier`); an empty forge reads `nothing salvaged`; a long LEARNED chip wraps at 400 px
//       and `alert:rising` reads `alert · rising`; every patch shows its reach (`reach ~0`); the rule-set tabs read `set 2 · 0`;
//       the death headline drops `N hp short`; a verdict screen offers `morgue` and `edit` only; at a run's end the floor
//       stays lit (fade ≤ 0.3), the mode buttons / ▶▶| / bail are dead and ⏸ is gone (the `verdict` label has the corner)
//
//   node web/tests/cut13.mjs [--shots]       (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser, launchGpu } from "../../tools/browser.mjs";
import { editRows, openPanel } from "./lib/frame.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shots = process.argv.includes("--shots") ? resolve(ROOT, "scratchpad/cut13") : null;
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };

const browser = shots ? await launchGpu() : await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: shots ? 3 : 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));

const state = () => page.evaluate(() => {
  const r = window.__riddle, w = document.querySelector(".watch");
  return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy, frame: w?.dataset.frame, mode: w?.dataset.mode, beats: Number(w?.dataset.beats ?? 0), tick: Number(w?.dataset.tick ?? 0), stake: document.querySelector(".stake")?.textContent ?? "", vault: !!document.querySelector(".sheet-wrap .vault-choice .chip") } : null;
});
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) {
    s = await state();
    if (s?.screen === "exit" && s.vault) { await page.locator(".sheet-wrap .vault-choice .chip").first().click({ timeout: 2000 }).catch(() => {}); await sleep(100); continue; }
    if (pred(s)) return s;
    await sleep(60);
  }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted} frame=${s?.frame})`);
}
const shot = async (name) => { if (shots) await page.screenshot({ path: `${shots}/${name}.png`, fullPage: !/beat|stall-hud|death-frame/.test(name) }); };
/** The death screen as text: the headline, the pill, the notes, the forecast line, the patches. */
const deathScreen = () => page.evaluate(() => ({
  line: document.querySelector(".death-line")?.textContent.replace(/\s+/g, " ").trim() ?? "",
  pill: document.querySelector(".death-line .verdict")?.textContent ?? "", pillClass: document.querySelector(".death-line .verdict")?.className ?? "",
  notes: [...document.querySelectorAll(".death-notes .note")].map((n) => n.textContent),
  said: document.querySelector(".forecast-said")?.textContent ?? null,
  trace: document.querySelectorAll(".death .trace tr, .death .trace-row, .death .trace li").length, patches: document.querySelectorAll("button.patch").length,
  ledger: document.querySelector(".death .ledger-line")?.textContent ?? "",
}));
const emptyReport = (L, extra = {}) => ({ elapsed_s: 3600, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, ...extra });
const fakeDeath = (extra) => page.evaluate((extra) => {
  window.__riddle.go({ kind: "death", death: { run_id: 0, depth: 3, cause: "goblin_archer", margin: "3 hp short", verdict: "gap", baseline: 0.25, trace: { turns: [] }, patches: [], morgue: "t10 a line", ...extra } });
}, extra);

try {
  // ---- §1: the stake while stalling, then the stall verdict screen (the fake's chore loop after 20 turns on the floor)
  const stallRules = encodeURIComponent("depth>=9 → return\nfoes>=1 → attack nearest");
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=21&fake_stall=20&rules=${stallRules}&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  let s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fast", "the stalling run");
  s = await waitFor((x) => x?.screen !== "watch" || /stalling/.test(x.stake), "the stalling stake", 30_000);
  // QA 92eb880 (N: `keeps $0 · stalling` for 10 s on a run that returned keeping 60 %): `stalling` alone while the run may still come home
  check(s.screen === "watch" && /(^|· )stalling\b/.test(s.stake) && !/keeps \$/.test(s.stake), `the stake reads \`stalling\` alone while the guard has fired: "${s.stake}"`);
  await shot("01-stall-hud");
  s = await waitFor((x) => x && x.screen !== "watch", "the stalled run's end", 60_000);
  if (s.screen === "exit") {   // the keep sheet still runs (a stall is a return with items in hand)
    check(!!(await page.locator(".sheet-wrap .label.row-label").first().textContent().catch(() => "")).startsWith("keep"), "the keep sheet ran before the verdict");
    await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 });
    s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after keep", 30_000);
  } else s = await waitFor((x) => x && !x.busy, "the verdict", 30_000);
  await sleep(200);
  let d = await deathScreen();
  check(s.screen === "death" && d.pill === "stall" && /verdict stall/.test(d.pillClass), `a stalled run opens the verdict screen with the stall pill (${s.screen}, pill "${d.pill}")`);
  check(/^stalled · D\d+ · no path · stall$/.test(d.line), `the headline is the stall's cause: "${d.line}"`);
  check(d.notes.length >= 1 && d.notes.at(-1) === "Stalled. Came home empty-handed." && d.notes.length <= 2, `the run's last notes sit under the headline (verbatim): ${JSON.stringify(d.notes)}`);
  check(/returned \$0 · .* · keeps 0% · stalled/.test(d.ledger), `the stall's ledger line pays nothing: "${d.ledger}"`);
  check(d.patches >= 1, `patches are measured like a death's (${d.patches})`);
  await shot("02-stall-verdict");
  // the fallback: an engine whose `death(id)` has no record → today's report, its exit line `returned $0 · … · stalled`
  // Cut 18 §4: the core's line goes on after `stalled` (`… · stalled · 1 supply back`): the report still tallies the stall
  await page.evaluate(() => {
    const r = window.__riddle, orig = r.engine.step.bind(r.engine);
    r.engine.step = async (n) => { const res = await orig(n); for (const e of res.events) if (e.k === "exit" && e.line && /· stalled$/.test(e.line.text)) e.line = { ...e.line, text: `${e.line.text} · 1 supply back` }; return res; };
    r.engine.death = () => Promise.reject(new Error("no record")); r.go({ kind: "watch" });
  });
  s = await waitFor((x) => x?.screen === "watch", "the second stalling run");
  s = await waitFor((x) => x && x.screen !== "watch", "its end", 60_000);
  if (s.screen === "exit") { await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 }); s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after keep", 30_000); }
  await sleep(200);
  const exitLine = await page.evaluate(() => document.querySelector(".report .exit-lines .ledger-btn")?.textContent ?? "");
  check(s.screen === "report" && /^returned \$0 · .* · stalled/.test(exitLine), `without a stall record the report shows the run (${s.screen}: "${exitLine}")`);
  const plaques = await page.evaluate(() => [...document.querySelectorAll(".report .tile.plaque")].map((t) => `${t.querySelector("b")?.textContent} ${t.querySelector(".label")?.textContent}`.toUpperCase()));
  check(plaques.includes("1 RUNS") && plaques.includes("1 STALLED"), `the report after one stall reads RUNS 1 · STALLED 1 (${plaques.join(" · ")})`);
  // the report's `open` shows a stall verdict too
  await page.evaluate(() => {
    const r = window.__riddle; const L = r.lineage;
    const worst = { run_id: 0, depth: 2, cause: "stalled", margin: "archer, no path", verdict: "stall", baseline: 0.1, trace: { turns: [] }, patches: [], morgue: "", notes: ["A den. Something sleeps.", "Stalled. Came home empty-handed."] };
    r.go({ kind: "report", report: { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 3, worst_death: worst } });
  });
  await sleep(200);
  await page.locator("main.report button", { hasText: /^deepest$/ }).first().click({ timeout: 5000 });
  await waitFor((x) => x?.screen === "death", "the worst stall from the report");
  d = await deathScreen();
  check(d.pill === "stall" && d.line === "stalled · D2 · archer, no path · stall" && d.notes.length === 2, `the report's open shows the stall verdict: "${d.line}"`);

  // ---- §2: the heir's trait is chosen
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && x.screen === "camp", "camp");
  await editRows(page);   // Cut 17: the tablets carry their chips, ▲▼ and × (the `edit` tile, remembered)
  await sleep(300);
  const traits = () => page.evaluate(() => ({
    chips: [...document.querySelectorAll(".strip .chip.trait")].map((c) => ({ name: c.querySelector("span")?.textContent ?? "", rule: c.querySelector(".rule")?.textContent ?? "", on: c.classList.contains("on"), disabled: c.disabled })),
    trait: window.__riddle.lineage.trait, offer: window.__riddle.lineage.trait_offer ?? null, plain: [...document.querySelectorAll(".strip > span:not(.num):not(.chips)")].map((e) => e.textContent),
  }));
  let tr = await traits();
  check(tr.chips.length === 2 && tr.offer?.length === 2 && tr.chips.map((c) => c.name).join() === tr.offer.join(), `two trait chips beside ♟N: ${tr.chips.map((c) => c.name).join(" | ")}`);
  check(tr.chips.filter((c) => c.on).length === 1 && tr.chips.find((c) => c.on)?.name === tr.trait && tr.chips.find((c) => c.on)?.disabled === true, `the chosen one is on (${tr.trait}) and inert`);
  check(tr.chips.every((c) => c.rule && c.rule.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length <= 3), `each chip carries its rule under the name (≤ 3 words): ${tr.chips.map((c) => `"${c.rule}"`).join(" · ")}`);
  await shot("03-trait-offer");
  const other = tr.chips.find((c) => !c.on).name;
  await page.locator(".strip .chip.trait", { hasText: other }).first().click({ timeout: 5000 });
  await sleep(400);
  tr = await traits();
  check(tr.trait === other && tr.chips.find((c) => c.on)?.name === other, `a tap picks the trait (${tr.trait})`);
  // the send takes the offer: the chips vanish, the plain trait shows
  await page.evaluate(() => window.__riddle.go({ kind: "watch" }));
  await waitFor((x) => x?.screen === "watch", "the watch");
  await sleep(800);
  await page.evaluate(async () => { await window.__riddle.refresh(); window.__riddle.go({ kind: "camp" }); });
  await waitFor((x) => x?.screen === "camp", "camp again");
  await sleep(200);
  tr = await traits();
  check(tr.chips.length === 0 && tr.offer === null && tr.plain.includes(other), `after the send the chips are gone and the strip reads the chosen trait (${tr.plain.join(", ")})`);

  // ---- §3: the night's ledger
  await page.evaluate(() => {
    const r = window.__riddle; const L = r.lineage;
    const exits = [{ carried: 412, keep_pct: 100, kept: 412, spent: 0, spent_on: [], text: "banked $412 · $412 carried · keeps 100%" }, { carried: 160, keep_pct: 60, kept: 96, spent: 0, spent_on: [], text: "returned $96 · $160 carried · keeps 60%" }];
    r.go({ kind: "report", report: { elapsed_s: 3600, runs: 2, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [{ kind: "sword", n: 2, gold: 45 }], spent: [{ kind: "heal", n: 16, gold: 640 }], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 1, returned: 1, exits } });
  });
  await sleep(200);
  const rep = () => page.evaluate(() => ({
    spent: [...document.querySelectorAll(".report .rsec")].filter((s) => s.querySelector(".label")?.textContent === "spent").flatMap((s) => [...s.querySelectorAll("li")].map((l) => l.textContent.replace(/\s+/g, " ").trim())),
    gold: document.querySelector(".report .gold-line")?.textContent ?? null,
  }));
  let rp = await rep();
  check(rp.spent.length === 1 && rp.spent[0] === "heal ×16 · −$640", `the spent section: ${JSON.stringify(rp.spent)}`);
  check(rp.gold === "+$412 banked · +$96 returned · +$45 salvage · −$640 spent", `the gold line reconciles the tiles: "${rp.gold}"`);
  await shot("04-report-ledger");
  await page.evaluate(() => { const r = window.__riddle; const L = r.lineage; r.go({ kind: "report", report: { elapsed_s: 60, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 } } }); });
  await sleep(150);
  rp = await rep();
  check(rp.spent.length === 0 && rp.gold === null, "nothing bought, nothing salvaged: no spent section, no gold line");
  // the fake's `auto: restock` across an absence: the leash packed is rebought at each return, the report's SPENT says so
  const restocked = await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    e.lineage.gold = 900; e.lineage.unlocks.push("auto_supply"); e.lineage.trait_offer = undefined;
    b.engine = JSON.stringify(e);
    if (!(await r.importSave(JSON.stringify(b)))) return false;
    await r.mutate(() => r.engine.buySupply("leash"));
    await r.flush();   // the buy is debounced into the save; the reload below must find it
    return r.lineage.supplies.some((s) => s.kind === "leash" && !s.free);
  });
  check(restocked, "the lineage packed a bought leash under auto: restock");
  await page.goto(`${url}?dev=1&engine=fake&absent=2h`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "report" && !x.busy, "the 2 h report", 120_000);
  await sleep(300);
  rp = await rep();
  const spentLeash = rp.spent.find((l) => /^leash ×\d+ · −\$\d+$/.test(l));
  check(!!spentLeash && /spent/.test(rp.gold ?? ""), `the absence's report carries the restock: ${JSON.stringify(rp.spent)} · "${rp.gold}"`);
  await shot("05-report-absence");

  // ---- §4: the beats are on screen (fast, then fights), the death notes, the ticker
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=5&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fast", "the fast run");
  // a den note injected on the next quiet batch (no hostile adjacent, the frame on the map): the frame cuts in for its beat
  const inject = (note) => page.evaluate((note) => {
    const e = window.__riddle.engine; const real = e.step.bind(e); let done = false;
    e.step = async (n) => {
      const r = await real(n);
      const w = document.querySelector(".watch");
      const hx = r.snapshot.hero.x, hy = r.snapshot.hero.y;
      const adjacent = r.snapshot.entities.some((x) => !x.ally && x.id !== r.snapshot.hero.id && Math.max(Math.abs(x.x - hx), Math.abs(x.y - hy)) <= 1);
      if (!done && !adjacent && w?.dataset.frame === "map" && !r.run_over) { done = true; r.events.push({ t: r.snapshot.turn, k: "note", text: note }); window.__beatTick = r.snapshot.turn; }
      return r;
    };
  }, note);
  await inject("A den. Something sleeps.");
  s = await waitFor((x) => x?.screen !== "watch" || x.beats >= 1, "the beat", 20_000);
  check(s.beats >= 1, `a den note outside a fight is a beat (beats ${s.beats})`);
  s = await waitFor((x) => x?.screen !== "watch" || x.frame === "fight", "the frame cut", 8000);
  const beatText = await page.evaluate(() => ({ text: document.querySelector(".ticker")?.textContent ?? "", cls: document.querySelector(".ticker")?.className ?? "" }));
  await sleep(250);   // the ticker's fade-in
  const beatVis = await page.evaluate(() => { const t = document.querySelector(".ticker"); return t.classList.contains("beat") ? getComputedStyle(t).opacity : "gone"; });
  check(s.frame === "fight", `the fight frame opens on the beat in fast (frame ${s.frame})`);
  // the fake writes its own situation notes too (a vault or a shrine in view), any of which may be the first beat
  const BEAT = /^(A den\. Something sleeps\.|A cage: three inside, one to take\.|A shrine\. Pray, at a price\.)$/;
  check(BEAT.test(beatText.text) && /\bbeat\b/.test(beatText.cls) && beatVis !== "0", `the note is the callout, verbatim, visible in the frame: "${beatText.text}" (${beatText.cls}, opacity ${beatVis})`);
  await shot("06-beat-fast");
  // QA on 50bb162: the beat's line is gone within 6 s of showing (`A den.` sat 15 s over three floors)
  {
    const t0 = Date.now(); let gone = null;
    while (Date.now() - t0 < 8000) { const up = await page.evaluate(() => { const el = document.querySelector(".ticker"); return !!el && el.classList.contains("beat") && el.classList.contains("show"); }); if (!up) { gone = Date.now() - t0; break; } await sleep(50); }
    check(gone !== null && gone <= 6500, `the beat's line is gone within 6 s (${gone === null ? "still up at 8 s" : `${gone} ms`})`);
  }
  // in `fights`: the beat cuts in from under the card
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=5&autosend=1&speed=fights`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fights", "the fights run");
  await inject("A cage: three inside, one to take.");
  s = await waitFor((x) => x?.screen !== "watch" || (x.beats >= 1 && x.frame === "fight"), "the beat's cut in fights", 20_000);
  const beat2 = await page.evaluate(() => ({ text: document.querySelector(".ticker")?.textContent ?? "", card: document.querySelector(".watch")?.dataset.card, tick: Number(document.querySelector(".watch")?.dataset.tick), at: window.__beatTick }));
  check(s.frame === "fight" && beat2.card === "0" && BEAT.test(beat2.text), `in fights the beat cuts in from under the card (frame ${s.frame}, card ${beat2.card}): "${beat2.text}"`);
  check(beat2.text !== "A cage: three inside, one to take." || Math.abs(beat2.tick - beat2.at) <= 12, `the cut lands on the beat's tick (${beat2.tick} vs ${beat2.at})`);
  await shot("07-beat-fights");
  // the beat lets go after its ~4 s (the card is back) — unless a real fight took over
  s = await waitFor((x) => x?.screen !== "watch" || x.frame === "map" || x.tick - beat2.at > 60, "the beat's release", 12_000).catch(() => null);
  check(!!s, "the frame moved on after the beat");
  // the ticker: a 40-char callout renders whole at 400 px; two callouts on one tick both show, one after the other — injected
  // in `fast` on the map (no card to swallow them) on a batch whose ticker is idle (an ambient line holds it for its 1.5 s)
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=5&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fast", "a fast run for the ticker");
  const long = "the goblin conjurer summons three blades"; // 40 chars
  // the two land at the viewer's own tick and the clock is paused in the same task, so nothing else competes for the ticker
  // (at 16× on the map a callout a turn evicts a queue — that regime is a flicker either way); Cut 14 §6: ⏸ freezes the ticker
  // with the picture, so the pair is released on the thaw (▶), in one pass — the same tick — on a dead stretch (16×), and the
  // batches after it are stripped of their callouts so nothing evicts the queue
  await page.evaluate((long) => {
    const e = window.__riddle.engine; const real = e.step.bind(e); let done = 0;
    e.step = async (n) => {
      const r = await real(n); const w = document.querySelector(".watch"), t = document.querySelector(".ticker");
      // QA 92eb880: near a cage the watch now steps in short batches, so the cage's sheet (and its beat) opens in `fast` too — this
      // test is the ticker's: the cage is taken out of its world
      delete r.snapshot.vault_choice; r.events = r.events.filter((x) => !(x.k === "note" && /cage/i.test(x.text)));
      // after the pair, the batches the world steps meanwhile carry no line of their own (they would evict the queue at 16×)
      if (done >= 1) r.events = r.events.filter((x) => !["callout", "rule", "hurt", "die", "note", "pickup", "level", "rank", "steal", "telegraph", "bones"].includes(x.k));
      // Cut 18 §1: `fast` keeps its engine LEAD_PROBE ahead (a fight is costed before the picture meets it) and ramps past 16× on a
      // dead stretch — the pair lands on this batch's last tick (every line queued before it is earlier), on the map at ≥ 16×
      if (done < 1 && w?.dataset.frame === "map" && Number(w.dataset.speed) >= 16 && !t?.classList.contains("show") && !r.run_over && !r.events.some((x) => x.k === "exit")) {
        done++; const at = r.snapshot.turn;
        r.events.push({ t: at, k: "callout", text: long }, { t: at, k: "callout", text: "second of two on one tick" });
        for (const b of document.querySelectorAll("main.watch .hud-btn")) if (b.textContent === "⏸") b.click();
      }
      return r;
    };
  }, long);
  const seen = new Set(); let clipped = false, joined = false;
  await page.waitForFunction(() => document.querySelector("main.watch .gem.hud-btn.on")?.textContent === "▶" || window.__riddle.screen !== "watch", null, { timeout: 20_000 });
  await sleep(200);
  await page.evaluate(() => { for (const b of document.querySelectorAll("main.watch .hud-btn")) if (b.textContent === "▶") b.click(); });
  for (let i = 0; i < 160; i++) {
    const t = await page.evaluate(() => { const el = document.querySelector(".ticker"); if (!el || !el.classList.contains("show")) return null; const r = el.getBoundingClientRect(); return { text: el.textContent, right: r.right, left: r.left, scrollW: el.scrollWidth, clientW: el.clientWidth }; });
    if (t) { seen.add(t.text); if (t.right > 400.5 || t.left < -0.5 || t.scrollW > t.clientW + 1) clipped = true; if (t.text.includes(long) && t.text !== long) joined = true; }
    if (seen.has(long) && seen.has("second of two on one tick")) break;
    await sleep(50);
  }
  check(seen.has(long) && !clipped, `a 40-char callout renders whole at 400 px (seen: ${[...seen].map((x) => `"${x}"`).join(", ")})`);
  check(seen.has("second of two on one tick") && !joined, "two callouts on one tick queue: both show, never on one line");
  // the death screen shows the notes
  await fakeDeath({ notes: ["The green one: fire.", "Gambled: fire potion."] });
  await waitFor((x) => x?.screen === "death", "a death with notes");
  d = await deathScreen();
  // QA 778fa1b (qaU: `Goblin Captain: telegraph.` read as a sentence): a fact note (`Name: tag.`) drops its stop
  check(d.notes.join(" | ") === "The green one: fire | Gambled: fire potion" && d.said === null, `the death screen shows the run's last two notes (their stops dropped) (a gap death says no forecast): ${JSON.stringify(d.notes)}`);
  await shot("08-death-notes");

  // ---- §5: the forecast's noise shown as noise
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && x.screen === "camp", "camp");
  // QA e75ec29 (Q: `D1 100% ±1`): a share that reads 0 % or 100 % carries no ± — the ± is asked of the rows strictly between
  const fc = () => page.evaluate(() => { const L = window.__riddle.lastForecast; const inside = (x) => { const r = Math.round(x * 100); return r > 0 && r < 100; };
    return { refined: document.querySelector(".forecast")?.dataset.refined, pms: [...document.querySelectorAll(".fc-bars .pm")].map((e) => e.textContent), want: (L?.depths ?? []).filter((d) => d.pm !== undefined && inside(d.reach)).length, deathInside: !!L?.ends && inside(L.ends.death), ends: document.querySelector(".fc-ends:not([hidden])")?.textContent ?? "", stale: document.querySelector(".forecast")?.classList.contains("stale") }; });
  const endsOk = (f, tail) => f.deathInside ? new RegExp(` · death \\d+% ±\\d+${tail} · ~\\$\\d+$`).test(f.ends) : / · death (0|100|[<>]\d+)% · ~\$\d+$/.test(f.ends);
  await page.waitForFunction(() => document.querySelector(".forecast")?.dataset.refined === "0", null, { timeout: 15_000 });
  let f = await fc();
  check(f.refined === "0" && f.pms.length === f.want && f.pms.every((p) => /^ ±\d+…$/.test(p)), `the first paint's ± trail …, one per share strictly inside 0–100 %: ${f.pms.join(",")} (${f.want} wanted)`);
  check(endsOk(f, "…"), `the ends line carries its own ± (first paint; none at 0 / 100 %): "${f.ends}"`);
  await page.waitForFunction(() => document.querySelector(".forecast")?.dataset.refined === "1", null, { timeout: 15_000 });
  f = await fc();
  check(f.refined === "1" && f.pms.length === f.want && f.pms.every((p) => /^ ±\d+$/.test(p)), `after the refine the … is gone: ${f.pms.join(",")} (${f.want} wanted)`);
  check(endsOk(f, ""), `the ends line after the refine: "${f.ends}"`);
  // the unlock deltas: within their ± → `reach ~0`; otherwise with the ±
  // Cut 17: the unlock shelf is a panel, carved with the lineage's first mark (docs/UI.md §5): the lineage takes one, the panel opens
  // on its whole catalogue (`more`)
  await page.evaluate(async () => { const r = window.__riddle; const save = JSON.parse(await r.engine.save()); save.lineage.marks = Math.max(1, save.lineage.marks); r.lineage = await r.engine.load(JSON.stringify(save)); r.go({ kind: "camp" }); });
  await openPanel(page, "unlocks", { all: true });
  await page.waitForFunction(() => document.querySelectorAll(".unlocks .card .delta").length > 0, null, { timeout: 15_000 });
  const deltas = await page.evaluate(() => [...document.querySelectorAll(".unlocks .card .delta")].map((e) => ({ text: e.textContent, cls: e.className })));
  check(deltas.length > 0 && deltas.every((x) => /^reach (≈|[+−]\d+ ±\d+) at (R\d+|end)( · vs [a-z ]+)?$/.test(x.text)), `card deltas carry their ± or read ~0: ${deltas.map((x) => x.text).join(" · ")}`);
  check(deltas.some((x) => /≈/.test(x.text) && /flat/.test(x.cls)) && deltas.some((x) => /±/.test(x.text)), "both forms occur on the fake's catalogue (a ~0 is flat, not up or down)");
  await page.locator(".unlocks .card").filter({ hasText: "reach ≈" }).first().click({ timeout: 5000 }); await sleep(200);
  const sheetDelta = await page.evaluate(() => document.querySelector(".sheet-wrap .delta")?.textContent ?? "");
  check(/^reach ≈ at (R\d+|end)$/.test(sheetDelta), `the unlock sheet reads the same: "${sheetDelta}"`);
  await shot("09-forecast-noise");
  await page.keyboard.press("Escape"); await sleep(100);
  // a dice death says what the forecast said for its depth (reach[d] − reach[d+1]); a gap death does not
  await page.evaluate(() => { window.__riddle.lastForecast = { depths: [{ depth: 1, reach: 1 }, { depth: 2, reach: 0.8 }, { depth: 3, reach: 0.3 }], causes: [], known_to: 3 }; });
  await fakeDeath({ verdict: "dice", depth: 2 });
  await waitFor((x) => x?.screen === "death", "a dice death");
  d = await deathScreen();
  check(d.said === "forecast said D2 80%" && d.pill === "dice", `a dice death names the reach the forecast showed for its floor, verbatim: "${d.said}"`);
  await shot("10-dice-forecast-said");
  await fakeDeath({ verdict: "dice", depth: 3 });
  await sleep(100); d = await deathScreen();
  check(d.said === "forecast said D3 30%", `at the frontier the same: "${d.said}"`);
  await fakeDeath({ verdict: "dice", depth: 5 });
  await sleep(100); d = await deathScreen();
  check(d.said === null, "past the forecast's floors nothing is claimed");
  await page.evaluate(() => { window.__riddle.lastForecast = { depths: [{ depth: 1, reach: 1 }, { depth: 2, reach: 0.8 }, { depth: 3, reach: 0.3 }], causes: [], known_to: 2 }; });
  await fakeDeath({ verdict: "dice", depth: 3 });
  await sleep(100); d = await deathScreen();
  check(d.said === null, "a floor past known_to is not claimed either, even with a bar");

  // ---- QA on 50bb162 (qaE)
  // the death headline: no `N hp short` (read as the hp left by four players); a stall's reason stays; morgue and edit only
  await fakeDeath({ margin: "1 hp short" });
  await sleep(100); d = await deathScreen();
  // Cut 17: the verdict screen's buttons are its console — the command card (morgue · camp, docs/CUT17.md §1) and the gem (edit,
  // with no patch to apply)
  const btns = await page.evaluate(() => [...document.querySelectorAll("main.death .console button")].map((b) => b.textContent.trim()));
  check(d.line === "goblin archer · D3 · gap", `the headline drops the hp margin: "${d.line}"`);
  check(btns.join() === "morgue,camp,edit", `the verdict screen's buttons are morgue · camp and the edit gem only: [${btns.join(", ")}]`);
  await fakeDeath({ margin: "3 over" });
  await sleep(100); d = await deathScreen();
  check(d.line === "goblin archer · D3 · gap", `the core's \`3 over\` is dropped too: "${d.line}"`);
  await fakeDeath({ cause: "stalled", margin: "archer, no path", verdict: "stall" });
  await sleep(100); d = await deathScreen();
  check(d.line === "stalled · D3 · archer, no path · stall", `a stall keeps the guard's reason: "${d.line}"`);
  // every patch shows its reach: `reach ~0` when the delta rounds to 0
  const row = { conds: [{ k: "hp<", n: 20 }], verb: { v: "rest" } };
  await fakeDeath({ patches: [{ row, insert_at: 0, survive: 0.5, forecast_delta: 0.002 }, { row: { ...row, verb: { v: "attack" } }, insert_at: 0, survive: 0.6, forecast_delta: 0.25 }] });
  await sleep(100);
  const reaches = await page.evaluate(() => [...document.querySelectorAll("button.patch")].map((p) => ({ delta: p.querySelector(".delta")?.textContent ?? null, cls: p.querySelector(".delta")?.className ?? "" })));
  check(reaches.length === 2 && reaches[0].delta === "reach ≈" && /flat/.test(reaches[0].cls) && reaches[1].delta === "reach +25", `every patch carries a reach line: ${reaches.map((r) => r.delta).join(" · ")}`);
  // the report: the LEARNED chips wrap and `alert:rising` reads `alert · rising`; the TRACE sheet from a ledger line
  await page.evaluate(() => {
    const r = window.__riddle; const L = r.lineage;
    const link = (i) => ({ text: `found item ${i} on D1`, t: i * 10, depth: 1 });
    const because = { text: "den took the heal, D3", t: 5, depth: 3 };
    const trace = { turns: [{ t: 120, row: 1, verb: { v: "attack" }, hp: 4, foes: 2, telegraphs: [], rows: [{ row: 0, why: "no item", because }] }], provenance: [because, ...Array.from({ length: 12 }, (_, i) => link(i + 1))] };
    const exits = [{ carried: 117, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $117 carried · keeps 0%", run_id: 0, trace }];
    r.go({ kind: "report", report: { elapsed_s: 3600, runs: 1, sampled: false, learned: ["foe:goblin_warlord:boss", "foe:goblin_warlord:buffer", "foe:goblin_warlord:summoner", "foe:goblin_warlord:telegraph", "alert:rising", "biome:warrens"], bests: [], found: [], deaths: [{ cause: "goblin_warlord", n: 1 }], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 0, exits } });
  });
  await sleep(200);
  const chipsQ = await page.evaluate(() => [...document.querySelectorAll(".report .chip.fact")].map((c) => { const b = c.getBoundingClientRect(); return { text: c.textContent.replace(/\s+/g, " ").trim(), right: b.right, over: c.scrollWidth > c.clientWidth + 1 }; }));
  const warlord = chipsQ.find((c) => /goblin warlord/.test(c.text));
  check(!!warlord && /boss · buffer · summoner · telegraph$/.test(warlord.text) && warlord.right <= 400.5 && !warlord.over, `a long LEARNED chip wraps inside the viewport: "${warlord?.text}" right ${warlord?.right}`);
  check(chipsQ.some((c) => c.text === "alert rises · alert ≥ open") && !chipsQ.some((c) => /:/.test(c.text)), `alert:rising reads with a dot: ${chipsQ.map((c) => `"${c.text}"`).join(", ")}`);
  await shot("11-report-chips");
  await page.locator(".report .exit-lines .chip.mini", { hasText: /\btrace$/ }).first().click({ timeout: 5000 }); await sleep(200);   // Cut 14 §4: the chip reads `D5 · died · trace`
  const traceSheet = await page.evaluate(() => {
    const w = document.querySelector(".sheet-wrap"); if (!w) return null;
    return { head: w.querySelector(".trace-head")?.textContent ?? null, rows: [...w.querySelectorAll(".chain-row")].map((r) => r.textContent.replace(/\s+/g, " ").trim()) };
  });
  check(traceSheet?.head === "died $0 · $117 carried · keeps 0%", `the TRACE sheet's header is the exit line, verbatim: "${traceSheet?.head}"`);
  const extras = (traceSheet?.rows ?? []).filter((r) => /^← found item/.test(r));
  check(extras.length === 8 && /item 5 /.test(extras[0]) && /item 12 /.test(extras[7]), `the provenance under the chain is the last 8 by tick: ${extras.length} (${extras[0]} … ${extras[7]})`);
  check((traceSheet?.rows ?? []).at(-1) === "· 4 earlier" && (traceSheet?.rows ?? []).some((r) => /^R1 .*no item.*← den took the heal/.test(r)), `the rest is one dim line, the row's own link stays: ${JSON.stringify(traceSheet?.rows)}`);
  await shot("12-trace-sheet");
  await page.keyboard.press("Escape"); await sleep(100);
  // the camp: the rule-set tabs, the empty forge
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && x.screen === "camp", "camp");
  await sleep(300);
  const tabsT = await page.evaluate(() => [...document.querySelectorAll(".tabs .tab:not(.edit)")].map((t) => t.textContent.replace(/\s+/g, " ").trim()));
  check(tabsT.length >= 2 && tabsT.slice(1).every((t) => /^set \d+ · \d+$/.test(t)), `an unnamed set's tab reads \`set 2 · 0\`: [${tabsT.join(" | ")}]`);
  await page.evaluate(() => window.__riddle.renameSet(1, "tank"));
  await sleep(200);
  const tabsN = await page.evaluate(() => [...document.querySelectorAll(".tabs .tab:not(.edit)")].map((t) => t.textContent.replace(/\s+/g, " ").trim()));
  check(/^tank · \d+$/.test(tabsN[1] ?? ""), `a named set reads \`name · N\`: [${tabsN.join(" | ")}]`);
  await page.locator(".tabs .tab", { hasText: /^set 3/ }).first().click({ timeout: 5000 }); await sleep(200);
  check((await page.evaluate(() => window.__riddle.active)) === 2, "tapping `set 3` selects the third set");
  await page.evaluate(() => window.__riddle.selectSet(0)); await sleep(100);
  // a lineage that has salvaged nothing yet (the fake seeds a rung or two)
  check(await page.evaluate(async () => { const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine); e.lineage.forge = {}; b.engine = JSON.stringify(e); return r.importSave(JSON.stringify(b)); }), "the lineage took an empty forge");
  await waitFor((x) => x?.booted && x.screen === "camp", "camp with an empty forge"); await sleep(300);
  await page.locator(".cmd .tile[data-tile=forge]").first().click({ timeout: 5000 }); await sleep(200);
  const forgeT = await page.evaluate(() => { const w = document.querySelector(".sheet-wrap"); return w ? { text: w.innerText.replace(/\s+/g, " ").trim(), heads: w.querySelectorAll(".lrow.head").length, empty: w.querySelector(".forge .empty-line")?.textContent ?? null } : null; });
  check(forgeT?.empty === "nothing salvaged" && forgeT.heads === 0 && /^forge( .*)? nothing salvaged$/i.test(forgeT.text), `an empty forge says so under its label (Cut 23: under the kit ladders): "${forgeT?.text}"`);
  await shot("13-forge-empty");
  await page.keyboard.press("Escape"); await sleep(100);
  // the death frame: at the run's end the floor stays lit, the run controls are dead, ⏸ is gone while `verdict` runs
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=5&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fast", "a fast run for the death frame");
  await page.evaluate(() => {
    const e = window.__riddle.engine; const real = e.step.bind(e); let done = false;
    // the next quiet map batch ends the run in a death; the verdict then takes 4 s (the busy label's window)
    e.step = async (n) => {
      const r = await real(n);
      if (!done && document.querySelector(".watch")?.dataset.frame === "map" && !r.run_over) {
        done = true; const t = r.snapshot.turn, hero = r.snapshot.hero;
        r.events.push({ t, k: "die", id: hero.id, cause: "jackal" }, { t, k: "exit", tier: "death", loot_kept: 0, line: { carried: 12, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $12 carried · keeps 0%" } });
        r.run_over = true;
        e.death = () => new Promise((res) => setTimeout(() => res({ run_id: r.snapshot.run.id, depth: r.snapshot.depth, cause: "jackal", margin: "1 hp short", verdict: "gap", baseline: 0.5, trace: { turns: [] }, patches: [], morgue: "" }), 4000));
      }
      return r;
    };
  });
  const frameState = () => page.evaluate(() => {
    const w = document.querySelector(".watch"); const v = window.__viewer;
    const btn = (t) => [...document.querySelectorAll("main.watch .hud-btn")].find((b) => b.textContent === t);
    return { screen: window.__riddle.screen, busy: window.__riddle.engineBusy, over: w?.dataset.over ?? "0", fade: v?.stats?.().fade ?? null,
      dead: ["fights", "fast", "▶▶|", "bail"].map((t) => btn(t)?.disabled ?? null), pause: btn("⏸")?.hidden ?? btn("▶")?.hidden ?? "gone", label: document.querySelector(".busy-label")?.textContent ?? "",
      gem: document.querySelector("main.watch .gem-slot > .gem")?.textContent ?? "", heir: document.querySelector("main.watch .topbar .heir")?.textContent ?? "" };
  });
  s = await waitFor((x) => x?.screen !== "watch" || x.busy, "the verdict's busy window", 40_000);
  await sleep(600);   // the walk-out drained: the fade has settled at its target
  const fs = await frameState();
  check(fs.screen === "watch" && fs.busy && fs.label === "verdict", `the verdict runs over the final frame (screen ${fs.screen}, busy ${fs.busy}, "${fs.label}")`);
  check(fs.over === "1" && fs.dead.every((d) => d === true), `fights · fast · ▶▶| · bail are dead on a dead hero: [${fs.dead.join(", ")}]`);
  // QA 23ed91f (K): the gem slot keeps a gem at the end — `verdict`, what comes next — where ⏸ was (the corner label no longer shows)
  check(fs.pause === "gone" && fs.gem === "verdict", `⏸ gives the gem slot to \`verdict\` at the end (pause ${fs.pause}, gem "${fs.gem}")`);
  check(fs.fade !== null && fs.fade <= 0.3 + 1e-6, `the floor stays lit at the end (fade ${fs.fade})`);
  await shot("death-frame");
  await waitFor((x) => x?.screen === "death", "the death screen after the verdict", 30_000);

  // ---- QA on 50bb162 (qaF): the camp at 400 px
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((x) => x?.booted && x.screen === "camp", "camp for qaF");
  // marks, the card gates met, a boss counter known past the best (the forecast's `try` row)
  await page.evaluate(async () => {
    const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine);
    e.lineage.marks = 20; e.lineage.gold = 300; e.lineage.best_depth = 5;
    e.lineage.facts.push("foe:goblin_archer:ranged", "boss:goblin_warlord:counter");
    b.engine = JSON.stringify(e);
    return r.importSave(JSON.stringify(b));
  });
  await page.waitForFunction(() => window.__riddle.unlockCat.length > 0, null, { timeout: 10_000 });
  await sleep(600);
  // a card row fits the phone: the chips wrap inside the row, the × stays inside the viewport (it sat at x=500 on a 523 px row)
  await page.evaluate(() => window.__riddle.buy("kite_archers"));
  await sleep(800);
  const cardRow = await page.evaluate(() => {
    const r = document.querySelector(".editor .row.locked"); if (!r) return null;
    const b = r.getBoundingClientRect(), x = r.querySelector(".x").getBoundingClientRect();
    return { right: b.right, xRight: x.right, xLeft: x.left, chipsRight: Math.max(...[...r.querySelectorAll(".chip")].map((c) => c.getBoundingClientRect().right)), scrollW: document.documentElement.scrollWidth, text: r.querySelector(".chips").innerText.replace(/\s+/g, " ").trim() };
  });
  check(!!cardRow && cardRow.right <= 400 && cardRow.xRight <= 400 && cardRow.chipsRight <= cardRow.xLeft && cardRow.scrollW <= 400, `a card row fits 400 px, its × inside (row right ${cardRow?.right}, × ${cardRow?.xLeft}–${cardRow?.xRight}, chips to ${cardRow?.chipsRight}, page ${cardRow?.scrollW}): "${cardRow?.text}"`);
  await shot("qaF-card-row");
  // the forecast's `try` row: one line per depth — the hint rides the track, the track keeps the row's width
  await page.waitForFunction(() => !!document.querySelector(".forecast .bar.try .try"), null, { timeout: 15_000 });
  await openPanel(page, "forecast");   // Cut 17: the bars are read where the player sees them, in the forecast panel
  const tryRow = await page.evaluate(() => {
    const bar = document.querySelector(".forecast .bar.try"); const plain = document.querySelector(".forecast .bar:not(.try):not(.unknown)");
    const b = bar.getBoundingClientRect(), t = bar.querySelector(".try").getBoundingClientRect(), n = bar.querySelector(".n").getBoundingClientRect(), tr = bar.querySelector(".track").getBoundingClientRect();
    return { h: b.height, plainH: plain.getBoundingClientRect().height, sameLine: t.top < n.bottom && n.top < t.bottom, hintInTrack: t.left >= tr.left - 1 && t.right <= tr.right + 1, hintRight: t.right, trackW: tr.width, text: bar.innerText.replace(/\s+/g, " ").trim() };
  });
  check(tryRow.sameLine && tryRow.hintInTrack && tryRow.h <= tryRow.plainH + 6 && tryRow.hintRight <= 400, `the try hint sits on the depth's own line, on the track (row ${tryRow.h} px vs ${tryRow.plainH}, track ${Math.round(tryRow.trackW)} px): "${tryRow.text}"`);
  await shot("qaF-forecast-try");
  // the engine's busy label in its own strip: under the header, above the tabs, never over `D4 ★0` or a bar
  await page.evaluate(() => { const r = window.__riddle; const orig = r.engine.forecast.bind(r.engine); window.__origForecast = orig; r.engine.forecast = () => new Promise((res) => setTimeout(() => orig().then(res), 2500)); r.insertRow({ conds: [{ k: "hp<", n: 30 }], verb: { v: "retreat" } }, 0); r.go({ kind: "camp" }); });
  await sleep(600);
  const busyStrip = await page.evaluate(() => { const st = document.querySelector(".busy-strip"), hd = document.querySelector("header.strip"), tabs = document.querySelector(".tabs"), lb = document.querySelector(".busy-label"); return { text: st?.textContent ?? null, top: st?.getBoundingClientRect().top, bottom: st?.getBoundingClientRect().bottom, header: hd?.getBoundingClientRect().bottom, tabs: tabs?.getBoundingClientRect().top, labelHidden: lb ? lb.hidden : null, bar: !!document.querySelector(".busy") }; });
  check(busyStrip.bar && busyStrip.text === "forecast" && busyStrip.top >= busyStrip.header && busyStrip.bottom <= busyStrip.tabs && busyStrip.labelHidden === true, `the busy label has its own strip between the header and the tabs ("${busyStrip.text}", ${busyStrip.header} ≤ ${busyStrip.top}..${busyStrip.bottom} ≤ ${busyStrip.tabs}, corner label hidden ${busyStrip.labelHidden})`);
  await shot("qaF-busy-strip");
  await page.waitForFunction(() => !window.__riddle.engineBusy, null, { timeout: 10_000 });
  await page.evaluate(() => { window.__riddle.engine.forecast = window.__origForecast; });
  // the ◆ readout repaints from the returned lineage at once, not after the vocabulary round-trip (1.5 s behind a forecast)
  const marksRead = await page.evaluate(async () => {
    const r = window.__riddle; const orig = r.engine.vocabulary.bind(r.engine);
    r.engine.vocabulary = () => new Promise((res) => setTimeout(() => orig().then(res), 1500));
    const before = document.querySelector(".strip .marks").textContent; const t0 = performance.now();
    const p = r.buy("row5");
    let after = before; while (performance.now() - t0 < 3000) { after = document.querySelector(".strip .marks").textContent; if (after !== before) break; await new Promise((x) => setTimeout(x, 20)); }
    const dt = Math.round(performance.now() - t0); await p; r.engine.vocabulary = orig;
    return { before, after, dt, marks: r.lineage.marks };
  });
  check(marksRead.after === `◆${marksRead.marks}` && marksRead.after !== marksRead.before && marksRead.dt < 600, `the header's ◆ repaints as the buy's lineage lands (${marksRead.before} → ${marksRead.after} in ${marksRead.dt} ms, vocabulary held 1.5 s)`);

  // ---- QA on 50bb162 (qaF): the watch — ▶▶| in `fast` is the run's end; a beat's line clears at a floor change and within 6 s
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=5&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fast", "a fast run for the beat's clearing");
  // a den note on a quiet map batch, then a floor change (the same floor's snapshot) on the batch after: the beat's line goes
  await page.evaluate(() => {
    const e = window.__riddle.engine; const real = e.step.bind(e); let phase = 0;
    e.step = async (n) => {
      const r = await real(n); const w = document.querySelector(".watch");
      const hx = r.snapshot.hero.x, hy = r.snapshot.hero.y;
      const adjacent = r.snapshot.entities.some((x) => !x.ally && x.id !== r.snapshot.hero.id && Math.max(Math.abs(x.x - hx), Math.abs(x.y - hy)) <= 1);
      if (phase === 0 && !adjacent && w?.dataset.frame === "map" && !r.run_over) { phase = 1; r.events.push({ t: r.snapshot.turn, k: "note", text: "A den. Something stirs." }); window.__beatTick = r.snapshot.turn; }
      else if (phase === 1 && !r.run_over) { phase = 2; r.events.push({ t: r.snapshot.turn, k: "descend", depth: r.snapshot.depth }); window.__descendTick = r.snapshot.turn; window.__descendAt = performance.now(); }
      return r;
    };
  });
  s = await waitFor((x) => x?.screen !== "watch" || x.beats >= 1, "the beat for the clearing test", 20_000);
  // the fake writes its own situation notes too (a shrine, a cage, its own den): the injected line's wording is its own
  let beatSeen = false, beatGoneBy = null, beatAfterDescend = null, beatLongest = 0, firstSeen = 0;
  for (let i = 0; i < 200 && s.screen === "watch"; i++) {
    const t = await page.evaluate(() => { const el = document.querySelector(".ticker"), w = document.querySelector(".watch"); return { beat: !!el && el.classList.contains("beat") && el.classList.contains("show"), text: el?.textContent ?? "", tick: Number(w?.dataset.tick), descendTick: window.__descendTick ?? null, descendAt: window.__descendAt ?? null, now: performance.now() }; });
    const same = t.beat && t.text === "A den. Something stirs.";
    if (same && !beatSeen) { beatSeen = true; firstSeen = t.now; }
    if (same) beatLongest = Math.max(beatLongest, t.now - firstSeen);
    if (beatSeen && !same && beatGoneBy === null) beatGoneBy = t.now - firstSeen;
    // after the descend has both reached the client (its batch answered ≥ 200 ms ago) and been released at the viewer's clock
    if (t.descendAt !== null && t.now - t.descendAt >= 200 && t.tick >= t.descendTick + 3 && beatAfterDescend === null) beatAfterDescend = same;
    if (beatGoneBy !== null && beatAfterDescend !== null) break;
    await sleep(40);
    s = await state();
  }
  check(beatSeen && beatGoneBy !== null && beatGoneBy <= 6500, `the beat's line is gone within 6 s (shown ${Math.round(beatLongest)} ms, gone by ${beatGoneBy === null ? "never" : Math.round(beatGoneBy)} ms)`);
  check(beatAfterDescend === false, `the beat's line clears at the floor change (the den's beat shown after descend: ${beatAfterDescend})`);
  // ▶▶| in `fast`: one press reaches the run's end (the ending, or the screen after it) — not the next fight
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=157&autosend=1&speed=fast`, { waitUntil: "domcontentloaded" });
  s = await waitFor((x) => x?.booted && x.screen === "watch" && x.mode === "fast", "a fast run for ▶▶|");
  await sleep(300);   // Cut 20's fast can reach the fake run's last tick inside 1.5 s: press early
  const t0 = Date.now();
  const endState = () => page.evaluate(() => ({ screen: window.__riddle.screen, ending: document.querySelector(".watch")?.dataset.ending === "1", tick: Number(document.querySelector(".watch")?.dataset.tick) }));
  let es = await endState(); const tick0 = es.tick, wasEnding = es.ending;
  await page.evaluate(() => { for (const b of document.querySelectorAll("main.watch button.hud-btn")) if (b.textContent === "▶▶|") b.click(); });
  while (Date.now() - t0 < 20_000 && es.screen === "watch" && !es.ending) { await sleep(60); es = await endState(); }
  // reaching the end is the point; a press that lands on the run's last tick advances no tick
  check(!wasEnding && (es.ending || es.screen !== "watch") && es.tick >= tick0, `one ▶▶| in fast reaches the run's end (${es.screen}${es.ending ? ", ending" : ""}, tick ${tick0} → ${es.tick}, after ${Date.now() - t0} ms)`);
  check(Date.now() - t0 < 15_000, `the end comes within seconds (${Date.now() - t0} ms)`);
  // the ending plays out, then the exit flow: the keep sheet or the verdict
  s = await waitFor((x) => x && x.screen !== "watch", "the screen after the skip", 60_000);
  if (s.screen === "exit") { await page.locator(".sheet-wrap button.btn.primary.wide").first().click({ timeout: 5000 }); s = await waitFor((x) => x && x.screen !== "watch" && x.screen !== "exit" && !x.busy, "the screen after keep", 30_000); }
  else s = await waitFor((x) => x && !x.busy, "the verdict after the skip", 30_000);
  check(["death", "report", "camp"].includes(s.screen), `the run ended (${s.screen})`);
  // the gold sheet from an exit line names its filter: the run's ledger word on, `all` off; a tap swaps them
  if (s.screen === "death") {
    await sleep(200);
    const hasLine = await page.evaluate(() => !!document.querySelector(".death .ledger-btn"));
    if (hasLine) {
      await page.locator(".death .ledger-btn").first().click({ timeout: 5000 });
      await sleep(200);
      const goldChips = () => page.evaluate(() => ({ filter: document.querySelector(".gold-sheet")?.dataset.filter ?? null, chips: [...document.querySelectorAll(".gold-sheet .row-label .chip.mini")].map((c) => ({ text: c.textContent, on: c.classList.contains("on") })), lines: document.querySelectorAll(".gold-sheet .lrow:not(.bal)").length }));
      let gc = await goldChips();
      if (gc.filter) {
        check(gc.chips.length === 2 && /^(died|returned|banked|stalled|lost thread) D\d+$/.test(gc.chips[0].text) && gc.chips[0].on && gc.chips[1].text === "all" && !gc.chips[1].on, `the run-filtered gold sheet names its filter: ${gc.chips.map((c) => `${c.text}${c.on ? " (on)" : ""}`).join(" · ")}`);
        await shot("qaF-gold-run");
        await page.locator(".gold-sheet .row-label .chip.mini", { hasText: /^all$/ }).first().click({ timeout: 5000 });
        await sleep(150);
        const all = await goldChips();
        check(all.filter === "" && all.chips[1].on && !all.chips[0].on && all.lines >= gc.lines, `all: the whole ledger, the all chip on (${all.lines} lines ≥ ${gc.lines})`);
      } else out.push("     (the exit's line has left the ledger: no run filter to name)");
      await page.keyboard.press("Escape"); await sleep(100);
    }
  }

  // ---- QA on 50bb162 (qaF): the report's stall count, the death screen's bones line
  const tilesOf = () => page.evaluate(() => [...document.querySelectorAll(".report .tiles .tile")].map((t) => `${t.querySelector(".label")?.textContent} ${t.querySelector("b")?.textContent}`));
  await page.evaluate(() => { const r = window.__riddle; const L = r.lineage; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 4, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "jackal", n: 1 }], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 3, stalled: 1 } }); });
  await sleep(150);
  let tiles = await tilesOf();
  check(tiles.slice(3).join(" · ") === "stalled 1 · returned 2 · deaths 1", `no banks: the stall tile takes banked's place, returned counts the rest (${tiles.slice(3).join(" · ")})`);
  await shot("qaF-report-stalled");
  await page.evaluate(() => { const r = window.__riddle; const L = r.lineage; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 6, sampled: false, learned: [], bests: [], found: [], deaths: [{ cause: "jackal", n: 1 }], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 2, returned: 3, stalled: 1 } }); });
  await sleep(150);
  tiles = await tilesOf();
  check(tiles.slice(3).join(" · ") === "returned 2 · banked 2 · stalled 1 · deaths 1" || tiles.slice(3).join(" · ") === "banked 2 · returned 2 · stalled 1 · deaths 1", `with banks nothing is dropped (${tiles.slice(3).join(" · ")})`);
  await page.evaluate(() => { const r = window.__riddle; const L = r.lineage; r.go({ kind: "report", report: { elapsed_s: 3600, runs: 3, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], reel: [], marks_earned: 0, live: null, tamed: [], hatched: [], lost: [], xp: { class: L.class, gained: 0, level_ups: 0 }, salvaged: [], renown: { gained: 0, rank: 0, ranks_up: 0 }, banked: 0, returned: 3 } }); });
  await sleep(150);
  tiles = await tilesOf();
  check(tiles.slice(3).join(" · ") === "banked 0 · returned 3 · deaths 0", `no stalls: the row as before (${tiles.slice(3).join(" · ")})`);
  const fakeStalled = await page.evaluate(async () => { const rep = await window.__riddle.engine.runOfflineQuick(600); return typeof rep.stalled; });
  check(fakeStalled === "number", `the fake's report carries stalled (${fakeStalled})`);
  // the pile once: the exit line's `bones: 8 items on D4` stands alone; the client's `bones left` line only without it
  await page.evaluate(() => { const L = window.__riddle.lineage; L.graveyard = [...(L.graveyard ?? []), { heir: 9, depth: 3, cause: "goblin_archer", t: 0 }]; L.bones = [{ heir: 9, depth: 3, items: 8 }]; });
  await fakeDeath({ line: { carried: 37, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $37 carried · keeps 0% · bones: 8 items on D3" } });
  await waitFor((x) => x?.screen === "death", "a death whose line carries the bones");
  const bonesLines = () => page.evaluate(() => ({ ledger: document.querySelector(".death .ledger-line")?.textContent ?? "", bones: document.querySelector(".death .bones-line")?.textContent ?? null }));
  let bl = await bonesLines();
  check(/bones: 8 items on D3/.test(bl.ledger) && bl.bones === null, `the pile is named once, on the exit line ("${bl.ledger}" · bones line ${bl.bones === null ? "gone" : `"${bl.bones}"`})`);
  await fakeDeath({ line: { carried: 37, keep_pct: 0, kept: 0, spent: 0, spent_on: [], text: "died $0 · $37 carried · keeps 0%" } });
  await waitFor((x) => x?.screen === "death", "a death whose line has no bones");
  bl = await bonesLines();
  check(bl.bones === "bones left · 8 items", `without it on the line the pile's own line stands ("${bl.bones}")`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut13: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut13: ok (${out.length} checks)`);
