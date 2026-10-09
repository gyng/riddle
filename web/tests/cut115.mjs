#!/usr/bin/env node
// Cut 115 — builds from tactics, client half (headless, tools/browser.mjs; real wasm on the 307dbed fixture):
//   panel  — Guarded + corridor fighting wears the `Bulwark` build (the core's `packages.build`): the tactics panel names it and its
//            effect; the worn tactic's two variants stand from L1; `compare outcomes` prices the other variant (the core's `variant`
//            move) beside its chip.
//   words  — the report's credit line (`picked 61% · taught 9% · chores 30%`) and build head, a death's deciding row with its credit,
//            and a move's walls (`better at D8 Warlord`) read as the core sends them.
//   node web/tests/cut115.mjs [--part=panel,words]
import { execFileSync } from "node:child_process";
import { readFileSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const root = new URL("../../", import.meta.url);
const url = execFileSync("bash", ["tools/dev.sh"], { cwd: root, encoding: "utf8" }).trim();
const parts = (process.argv.find((a) => a.startsWith("--part="))?.slice(7) ?? "panel,words").split(",");
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
const camp = (page) => page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 60_000 });
try {
  if (parts.includes("panel")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
    page.on("pageerror", (e) => errors.push(`panel pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&seed=307&fresh=1&runs=0`, { waitUntil: "domcontentloaded" });
    await camp(page);
    const fixture = readFileSync(new URL("crates/riddle-core/src/fixtures/save_307dbed.json", root), "utf8");
    await page.evaluate(async (e) => { const r = window.__riddle; const b = JSON.parse(r.exportSave()); b.engine = e; await r.importSave(JSON.stringify(b)); }, fixture);
    await camp(page); await page.waitForTimeout(500);
    await page.evaluate(async () => {
      const r = window.__riddle; const b = JSON.parse(r.exportSave()); const e = JSON.parse(b.engine); const p = e.lineage.pkg;
      p.owned = [...new Set([...(p.owned ?? []), "corridor_fighting", "guarded"])]; p.tactics = ["corridor_fighting"]; p.stance = "guarded";
      p.runs = { ...(p.runs ?? {}), corridor_fighting: 0 };
      b.engine = JSON.stringify(e); await r.importSave(JSON.stringify(b));
    });
    await camp(page); await page.waitForTimeout(500);
    const P = await page.evaluate(() => { const P = window.__riddle.lineage.packages; return { build: P?.build, cf: P?.all.find((p) => p.id === "corridor_fighting") }; });
    check(P.build?.name === "Bulwark" && P.build?.synergy === "bulwark" && !!P.build?.effect, `the core names the build (${JSON.stringify(P.build)})`);
    check(P.cf?.level === 1 && P.cf?.variants?.length === 2 && P.cf?.variant === 0, `the tactic's two variants stand from L1 (${JSON.stringify({ l: P.cf?.level, v: P.cf?.variants, at: P.cf?.variant })})`);
    await page.locator(".cmd .tile[data-tile=packages]").click();
    await page.waitForSelector(".pkg-panel", { timeout: 5000 });
    const head = await page.evaluate(() => ({ name: document.querySelector(".pkg-build")?.dataset.build ?? null, text: document.querySelector(".pkg-build")?.textContent ?? "" }));
    check(head.name === "Bulwark" && /−1 melee/.test(head.text), `the panel names the build and its effect (${head.text})`);
    const chips = await page.evaluate(() => [...document.querySelectorAll('.pkg-sec[data-kind=tactic] .pkg-variant')].map((b) => b.textContent));
    check(chips.length === 2, `the variants are chips at L1 (${chips})`);
    await page.locator(".pkg-compare").click();
    const priced = await page.waitForFunction(() => {
      const c = document.querySelector('.pkg-sec[data-kind=tactic] .pkg-variant[data-variant="1"] .pkg-price');
      return c ? c.textContent : document.querySelector(".pkg-estimate") ? "" : null;
    }, null, { timeout: 120_000 }).then((h) => h.jsonValue()).catch(() => null);
    check(!!priced && priced.trim().length > 0, `compare prices the other variant beside its chip (${JSON.stringify(priced)})`);
    await page.close();
  }
  if (parts.includes("words")) {
    const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
    page.on("pageerror", (e) => errors.push(`words pageerror: ${e.message}`));
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=5&runs=0`, { waitUntil: "domcontentloaded" });
    await camp(page);
    const r = await page.evaluate(async () => {
      const { creditLine, buildHead } = await import("/src/ui/report.ts");
      const { deathAction } = await import("/src/ui/death.ts");
      const { wallLine } = await import("/src/ui/packages.ts");
      const credit = [{ credit: "picked", fires: 61, share: 0.61 }, { credit: "taught", fires: 9, share: 0.09 }, { credit: "chores", fires: 30, share: 0.3 }];
      const L = { packages: { build: { name: "Bulwark", synergy: "bulwark", effect: "−1 melee taken in corridors", picks: ["Guarded", "corridor fighting · at three"] } } };
      const head = buildHead(L, 23, true, { credit });
      const d = { package: "corridor fighting · R4: 2+ foes → to corridor", credit: "picked", cause_row: 0, rules: { rows: [{ conds: [{ k: "foes>=", n: 2 }], verb: { v: "back_corridor" } }] }, trace: { turns: [] } };
      return {
        line: creditLine({ credit }), empty: creditLine(undefined), head: head?.textContent ?? null, none: buildHead({ packages: {} }, 9, true, undefined),
        action: deathAction(d),
        walls: wallLine({ walls: [{ depth: 8, boss: "Warlord", n: 8, better: 7, worse: 0 }, { depth: 28, boss: "Queen", n: 8, better: 0, worse: 7 }, { depth: 18, boss: "Lich", n: 8, better: 3, worse: 2 }] }),
      };
    });
    check(r.line === "picked 61% · taught 9% · chores 30%" && r.empty === "", `the report's credit line (${r.line})`);
    check(r.head === "Bulwark held D23 · picked 61% · taught 9% · chores 30%" && r.none === null, `the report's head names the build (${r.head})`);
    check(/· picked$/.test(r.action), `a death's deciding row says whose it was (${r.action})`);
    check(r.walls === "better at D8 Warlord · worse at D28 Queen", `a move names the walls it is for, the noise left out (${r.walls})`);
    await page.close();
  }
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`cut115: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`cut115: ok (${out.length} checks)`);
