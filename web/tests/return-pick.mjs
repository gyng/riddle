#!/usr/bin/env node
// Cut 113 §2–3, on the real wasm engine, headless at 400 × 800:
//   pick     a 30m absence's report carries one pick of three (chunky tiles, the core's offers); left untaken it waits on the camp;
//            taking one (Legacy) closes it and pays the points
//   forge    the forge's next weapon and armour tiers each offer two steps priced alike (aim · edge, plate · pace)
//   node web/tests/return-pick.mjs
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const browser = await launchBrowser();
try {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  // a lineage with its house (a new town builds it by hand), then a 30m absence on that save
  await page.goto(`${url}?dev=1&fresh=1&seed=113`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
  // the house built and the scout hired (an absence then runs its sends and the return opens on the report)
  await page.evaluate(async () => {
    const r = window.__riddle;
    if (r.lineage.town?.home === false) await r.mutate(() => r.engine.buildTown("house"), "build");
    const e = JSON.parse(await r.engine.save());
    for (const id of ["porter", "scout"]) if (!e.lineage.tree.hired.some((x) => x[0] === id)) e.lineage.tree.hired.push([id, 0]);
    r.lineage = await r.engine.load(JSON.stringify(e)); await r.flush?.();
  });
  await page.goto(`${url}?dev=1&seed=113&absent=30m`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted, null, { timeout: 90_000 });
  check(await page.evaluate(() => window.__riddle.kind) === "wasm", "real wasm engine");
  await page.waitForFunction(() => window.__riddle.screen === "report", null, { timeout: 60_000 }).catch(() => {});
  check(await page.evaluate(() => window.__riddle.screen) === "report", "the absence opens on its report");
  const pick = () => page.evaluate(() => window.__riddle.lineage.return_pick ?? null);
  const p0 = await pick();
  check(p0?.offers?.length === 3 && p0.size === 1, `a 30m return carries a pick of three, size 1 (${p0?.offers?.map((o) => o.id).join(" · ")} · ${p0?.size})`);
  const tiles = (where) => page.evaluate((w) => [...document.querySelectorAll(`.return-pick.${w} .tile[data-offer]`)].filter((t) => t.offsetParent).map((t) => t.dataset.offer), where);
  if (process.env.SHOT) await page.screenshot({ path: `${process.env.SHOT}/report.png` });
  check((await tiles("report")).length === 3, `the report shows the pick as three tiles (${(await tiles("report")).join(" · ")})`);
  // left untaken, the pick waits on the camp
  await page.locator(".report .gem").first().click();
  await page.waitForFunction(() => window.__riddle.screen === "camp", null, { timeout: 30_000 });
  await sleep(500);
  check(JSON.stringify(await pick()) === JSON.stringify(p0), "untaken, the pick waits unchanged");
  // on the camp the waiting pick is one tile (the fold's budget) that opens the three
  check((await tiles("camp")).length === 0 && await page.locator('.return-pick.camp .tile[data-tile="pick"]').isVisible(), "the camp shows the waiting pick as one tile");
  await page.locator('.return-pick.camp .tile[data-tile="pick"]').click();
  await sleep(400);
  const camp = await tiles("sheet");
  if (process.env.SHOT) await page.screenshot({ path: `${process.env.SHOT}/camp.png` });
  check(camp.length === 3, `the camp's pick opens its three (${camp.join(" · ")})`);
  const pts = () => page.evaluate(() => window.__riddle.lineage.bloodline?.points ?? window.__riddle.lineage.legacy?.points ?? null);
  const legacyOffer = p0?.offers?.find((o) => o.id === "legacy");
  const before = await page.evaluate(async () => JSON.parse(await window.__riddle.engine.save()));
  await page.locator('.return-pick.sheet .tile[data-offer="legacy"]').click();
  await page.waitForFunction(() => !window.__riddle.lineage.return_pick, null, { timeout: 15_000 });
  await sleep(300);
  const after = await page.evaluate(async () => JSON.parse(await window.__riddle.engine.save()));
  const bl = (s) => (s.active ?? s).lineage?.bloodline?.points ?? 0;
  check(!!legacyOffer && bl(after) - bl(before) === Number(legacyOffer.line.replace(/\D/g, "")), `Legacy taken: ${legacyOffer?.line} points (${bl(before)} → ${bl(after)})`);
  check(await page.locator('.return-pick.camp .tile').count() === 0 && (await tiles("sheet")).length === 0, "taken, the pick leaves the camp");
  void pts;
  // forge: two steps a tier, priced alike
  const kit = await page.evaluate(() => window.__riddle.lineage.kit ?? []);
  const w = kit.find((k) => k.slot === "weapon"), a = kit.find((k) => k.slot === "armour");
  check(w?.branches?.map((b) => b.id).join() === "aim,edge" && a?.branches?.map((b) => b.id).join() === "plate,pace", `forge tiers offer two steps (${w?.branches?.map((b) => b.label).join(" · ")} | ${a?.branches?.map((b) => b.label).join(" · ")})`);
} catch (e) { errors.push(String(e?.stack ?? e)); } finally { await browser.close(); }
console.log(out.join("\n")); if (errors.length) console.log(errors.join("\n"));
console.log(failed || errors.length ? `return-pick: FAIL (${failed} assertion(s), ${errors.length} error(s))` : `return-pick: ok (${out.length} checks)`);
process.exit(failed || errors.length ? 1 : 0);
