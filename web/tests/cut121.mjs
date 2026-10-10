#!/usr/bin/env node
// Cut 121 §2–§4 — a watch you can read, words, the gold headline; client half (headless, tools/browser.mjs; the fake engine):
//   why    — every automatic speed shows its reason chip (`fight · 2×`, `travel · 16×`) while it holds; a rise ramps (never ×2.5 in one
//            sample past 4×); Normal never runs over 4× and its over-1× stretches carry `quiet`.
//   strip  — a swift send is a one-line strip along the map's top (`D1–4 swift · …%`), never a card: no fight frame under a card, the
//            HUD stepped below it.
//   rest   — a run of `rest +5 hp` heals is one counting line (`rest ×6 · +30 hp`).
//   gold   — the report's gold tile leads with earned (`+$2659`, Earned) and names the workers' spending apart (`−$6750 workers`).
//   words  — the credit in plain words, `drink fire` named by what it does beside `throw fire`, `also matches: siren` on a boss row.
//   node web/tests/cut121.mjs [--part=why,strip,rest,gold,words]
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "why,strip,rest,gold,words").split(",");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
const page0 = async (tag, q = "") => {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
  page.on("pageerror", (e) => errors.push(`${tag} pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5${q}`, { waitUntil: "domcontentloaded" });
  return page;
};
const toWatch = async (page) => page.waitForFunction(() => window.__riddle?.screen === "watch" && document.querySelector(".watch")?.dataset.tick, null, { timeout: 30_000 });
/** Samples the watch's clock and its chip every ~30 ms for `ms`. */
const sample = (page, ms) => page.evaluate(async (ms) => {
  const t0 = performance.now(), rows = [];
  while (performance.now() - t0 < ms) {
    const w = document.querySelector(".watch"); if (!w || window.__riddle.screen !== "watch" || w.dataset.over === "1") break;
    const chip = w.querySelector(".speed-why");
    rows.push({ at: Math.round(performance.now() - t0), speed: Number(w.dataset.speed ?? 0), mode: w.dataset.mode, card: w.dataset.card, fold: w.dataset.fold, frame: w.dataset.frame,
      chip: chip && !chip.hidden ? chip.textContent : "", why: chip?.dataset.why ?? "" });
    await new Promise((z) => setTimeout(z, 30));
  }
  return rows;
}, ms);
try {
  if (parts.includes("why")) {
    const page = await page0("why", "&autosend=1&fake_god=1");
    await toWatch(page);
    const rows = await sample(page, 16_000);
    const live = rows.filter((r) => r.speed > 0 && r.card !== "1" && r.fold !== "1" && r.mode === "fights");
    // a reason must hold 180 ms to show (no flicker): count the live samples whose speed has held ≥ 400 ms with no chip
    let since = 0, bare = 0, held = 0;
    for (let i = 0; i < rows.length; i++) {
      if (i && Math.abs(rows[i].speed - rows[i - 1].speed) > 0.5) since = rows[i].at;
      const r = rows[i];
      if (r.speed > 0 && r.card !== "1" && r.fold !== "1" && r.at - since >= 400) { held++; if (!r.chip) bare++; }
    }
    const texts = [...new Set(live.map((r) => r.chip).filter(Boolean))];
    check(texts.length > 0 && texts.every((t) => /^(fight|chores|foe near|travel|quiet|catch up|stall|scene|exit|cage) · [\d.]+×$/.test(t)), `Fights only: the chip names its reason and rate (${texts.slice(0, 6).join(" | ")})`);
    check(held > 20 && bare / held <= 0.05, `every held automatic rate shows its chip (${bare}/${held} bare)`);
    // (a new reason settles 180 ms before it shows, an ended one lingers 500 ms: those samples read the last reason's rate)
    const withChip = live.filter((r) => r.chip), off = withChip.filter((r) => { const m = /· ([\d.]+)×$/.exec(r.chip); return !m || Math.abs(Number(m[1]) - r.speed) > Math.max(1, r.speed * 0.5); });
    check(withChip.length > 20 && off.length / withChip.length <= 0.25, `the chip's rate is the clock's (${off.length}/${withChip.length} off: ${off.slice(0, 3).map((r) => `${r.chip}@${r.speed}`).join(", ")})`);
    // ramps: from RISE_FREE (4×) up the clock at most doubles every 150 ms — never ×2.5 inside one ~30 ms sample
    const jumps = rows.filter((r, i) => i && rows[i - 1].speed >= 4 && r.speed / rows[i - 1].speed > 2.5 && r.at - rows[i - 1].at < 60);
    check(jumps.length === 0, `rises ramp, never a jump (${jumps.slice(0, 3).map((r) => `${r.speed}@${r.at}`).join(", ") || "none"})`);
    // Normal: never over 4× (its dead stretch's ONE_MAX), and a stretch over 1× says why
    await page.close();
    const p1 = await page0("why-one", "&fake_god=1");
    await p1.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    await p1.evaluate(() => { const a = window.__riddle; a.watchMode = "one"; a.go({ kind: "watch" }); });
    await toWatch(p1);
    const one = await sample(p1, 14_000);
    const over = one.filter((r) => r.speed > 4.01);
    check(one.length > 50 && over.length === 0, `Normal never runs over 4× (${over.length} of ${one.length} over; max ${Math.max(0, ...one.map((r) => r.speed))})`);
    let s1 = 0, fast = 0, said = 0;
    for (let i = 0; i < one.length; i++) {
      if (i && Math.abs(one[i].speed - one[i - 1].speed) > 0.5) s1 = one[i].at;
      if (one[i].speed > 1 && one[i].at - s1 >= 400) { fast++; if (one[i].chip) said++; }
    }
    check(fast === 0 || said / fast >= 0.95, `Normal: a stretch over 1× carries its chip (${said}/${fast}; ${[...new Set(one.map((r) => r.chip).filter(Boolean))].slice(0, 3).join(" | ")})`);
    check(one.filter((r) => r.mode === "one").length > 50 && one.filter((r) => r.speed <= 1 && r.chip).length <= 10, "Normal at 1× shows no chip (the chosen rate needs no reason)");
    await p1.close();
  }
  if (parts.includes("strip")) {
    const page = await page0("strip", "&fake_god=1");
    await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    await page.evaluate(() => {
      const a = window.__riddle, send = a.engine.send.bind(a.engine);
      a.lineage.feats = { ...(a.lineage.feats ?? {}), swift_to: 4 };
      const lin = a.engine.lineage.bind(a.engine); a.engine.lineage = async () => { const L = await lin(); L.feats = { ...(L.feats ?? {}), swift_to: 4 }; return L; };
      a.engine.send = async () => { const s = await send(); s.swift = true; return s; };
      a.engine.fold = async () => null;   // (the fake's fold reads the forecast alone: the client steps the swift stretch itself)
      window.__stripLog = [];
      new MutationObserver(() => {
        const w = document.querySelector(".watch"), f = w?.querySelector(".fold-line");
        if (f && !f.hidden) window.__stripLog.push({ docked: f.classList.contains("docked"), frame: w.dataset.frame, h: f.getBoundingClientRect().height });
      }).observe(document, { subtree: true, attributes: true, childList: true });
      a.watchMode = "fights"; a.go({ kind: "watch" });
    });
    await toWatch(page);
    await page.waitForFunction(() => document.querySelector(".watch .fold-line.docked:not([hidden])"), null, { timeout: 20_000 }).catch(() => {});
    await page.waitForTimeout(1500);
    const r = await page.evaluate(() => {
      const w = document.querySelector(".watch"), f = w.querySelector(".fold-line"), st = w.querySelector(".stage").getBoundingClientRect(), fr = f.getBoundingClientRect();
      const hud = w.querySelector(".hud.top").getBoundingClientRect();
      const chips = getComputedStyle(w.querySelector(".fold-chips")).display;
      return { shown: !f.hidden, docked: f.classList.contains("docked"), head: w.querySelector(".fold-head").textContent, top: Math.round(fr.top - st.top), h: Math.round(fr.height), hudTop: Math.round(hud.top - fr.bottom), chips, strip: w.dataset.strip, log: window.__stripLog };
    });
    check(r.shown && r.docked && r.strip === "1", `a swift send docks as a strip (${JSON.stringify({ shown: r.shown, docked: r.docked, strip: r.strip })})`);
    check(/^D1(–\d+)? swift( · \d+%)?/.test(r.head), `the strip reads \`D1–N swift · %\` (${r.head})`);
    check(r.top <= 1 && r.h <= 24 && r.chips === "none", `one line along the map's top edge (top ${r.top} px, ${r.h} px tall, chips ${r.chips})`);
    check(r.hudTop >= -1, `the HUD steps below the strip (gap ${r.hudTop} px)`);
    check(r.log.length > 0 && r.log.every((x) => x.docked && x.h <= 24), `never a card: every frame the line showed it was the strip (${r.log.filter((x) => !x.docked).length} card frames of ${r.log.length})`);
    // and it never covers a fight: through the rest of the run, the fight frame never sits under a card-sized line
    const later = await page.evaluate(async () => {
      const t0 = performance.now(); let bad = 0, fights = 0;
      while (performance.now() - t0 < 6000) {
        const w = document.querySelector(".watch"); if (!w) break;
        const f = w.querySelector(".fold-line");
        if (w.dataset.frame === "fight") { fights++; if (f && !f.hidden && f.getBoundingClientRect().height > 24) bad++; }
        await new Promise((z) => setTimeout(z, 50));
      }
      return { bad, fights };
    });
    check(later.bad === 0, `no fight frame under a fold card (${later.bad} of ${later.fights} fight samples)`);
    await page.close();
  }
  if (parts.includes("rest")) {
    const page = await page0("rest", "&fake_god=1");
    await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    await page.evaluate(() => {
      const a = window.__riddle, step = a.engine.step.bind(a.engine);
      a.engine.step = async (n) => {
        const res = await step(n), s = res.snapshot, hero = s?.hero;
        if (hero && !res.run_over) {
          const t = s.turn;
          res.events = [...(res.events ?? []), { t, k: "rule", row: 2, text: "R3 · rest", verb: { v: "rest" } }, { t, k: "heal", id: hero.id, amount: 5, src: "rest" },
            { t, k: "telegraph", id: 777777, what: "shifts" }, { t, k: "heal", id: hero.id, amount: 5, src: "rest" }];
        }
        return res;
      };
      a.watchMode = "one"; a.go({ kind: "watch" });
    });
    await toWatch(page);
    await page.waitForTimeout(6000);
    const r = await page.evaluate(() => {
      const rows = [...document.querySelectorAll(".watch .combat-lines li")];
      const rest = rows.filter((x) => x.dataset.rest === "rest");
      return { all: rows.length, rest: rest.length, max: Math.max(0, ...rest.map((x) => Number(x.dataset.n ?? 1))), text: rest.map((x) => x.textContent).slice(-2), warn: rows.filter((x) => x.dataset.warn).length, rules: rows.filter((x) => /R3 · rest/.test(x.textContent)).length };
    });
    check(r.max >= 4, `a run of rests is one counting line (the longest ×${r.max}; ${r.rest} rest lines of ${r.all})`);
    check(r.text.length > 0 && r.text.every((t) => /rest ×\d+ · \+\d+ hp$/.test(t) || /rest \+5 hp$/.test(t)), `it reads \`rest ×N · +M hp\` (${r.text.join(" | ")})`);
    check(r.rules <= r.rest + 1, `the rest row's own lines fold into the run (${r.rules} rule lines)`);
    await page.close();
  }
  if (parts.includes("gold")) {
    const page = await page0("gold");
    await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    const rep = (gold, workers) => ({ elapsed_s: 1200, runs: 1, sampled: false, learned: [], bests: [], found: [], deaths: [], pending: [], marks_earned: 0, tamed: [], hatched: [], lost: [], reel: [], deepest: 13, gold, workers });
    const read = (r) => page.evaluate(async (r) => {
      window.__riddle.go({ kind: "report", report: r, absence: true });
      await new Promise((z) => setTimeout(z, 700));
      const t = document.querySelector(".report-gold");
      return { head: t?.querySelector("b")?.textContent, label: t?.querySelector(".label")?.textContent, workers: t?.querySelector(".report-workers-spent")?.textContent ?? null, net: t?.dataset.net };
    }, r);
    // B's 20-minute return: the apprentice spent $2880 of a $73 income — the head is the income, the spending its own term
    const terms = [{ label: "carried", amount: 73 }, { label: "apprentice", amount: -2880 }];
    const a = await read(rep({ home: 73, salvage: 0, wake: 0, spent: 0, net: -2807, ledger: { earned: 73, spent: 2880, net: -2807, terms } }, [{ id: "apprentice", n: 1, what: "+1 step", spent: 2880 }]));
    check(a.head === "+$73" && a.label === "Gold earned", `the tile leads with earned, never a negative headline (${a.head} ${a.label})`);
    check(a.workers === "−$2880 workers" && a.net === "-2807", `the workers' spending is its own term (${a.workers}; purse ${a.net} in the tip)`);
    // the core's own fields win when sent
    const b = await read(rep({ home: 73, salvage: 0, wake: 0, spent: 0, net: -2807, earned: 80, spent_by_workers: 2887, ledger: { earned: 73, spent: 2880, net: -2807, terms } }, []));
    check(b.head === "+$80" && b.workers === "−$2887 workers", `the core's \`earned\` / \`spent_by_workers\` when present (${b.head} · ${b.workers})`);
    // nothing spent: no workers term
    const c = await read(rep({ home: 500, salvage: 20, wake: 0, spent: 0, net: 520 }, []));
    check(c.head === "+$520" && c.workers === null, `an income with no workers' spending reads earned alone (${c.head} · ${c.workers})`);
    await page.close();
  }
  if (parts.includes("words")) {
    const page = await page0("words");
    await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
    const r = await page.evaluate(async () => {
      const { creditLine } = await import("/src/ui/report.ts");
      const { verbLabel } = await import("/src/ui/tokens.ts");
      const { alsoMatches } = await import("/src/ui/editor.ts");
      const credit = [{ credit: "picked", fires: 29, share: 0.29 }, { credit: "taught", fires: 2, share: 0.02 }, { credit: "chores", fires: 69, share: 0.69 }];
      const boss = { conds: [{ k: "foe_tag", t: "boss" }], verb: { v: "read", a: "silence" } };
      const why = { sends: 4, actions: 900, fired: 12, matched: 12, text: "12/900", fired_on: [{ kind: "goblin_warlord", boss: true, n: 3 }, { kind: "siren", boss: false, n: 5 }] };
      return { line: creditLine({ credit }), drink: verbLabel({ v: "drink", a: "fire" }), heal: verbLabel({ v: "drink", a: "heal" }), throw_: verbLabel({ v: "throw", a: "fire" }),
        also: alsoMatches(boss, why), none: alsoMatches(boss, { ...why, fired_on: undefined }), other: alsoMatches({ conds: [{ k: "hp<", n: 50 }], verb: { v: "rest" } }, why) };
    });
    check(r.line === "own rules 29% · lessons 2% · chores 69%", `who decided, in plain words (${r.line})`);
    check(r.drink === "fire at feet" && r.throw_ === "throw fire" && r.heal === "drink heal", `one name per action: \`throw fire\` at a foe, the drink by what it does (${r.drink} · ${r.throw_} · ${r.heal})`);
    check(r.also.join() === "siren" && r.none.length === 0 && r.other.length === 0, `a boss row that fired on a siren warns \`also matches: siren\` (${r.also.join()})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut121: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut121: ok (${out.length} checks)`);
