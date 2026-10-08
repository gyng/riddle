#!/usr/bin/env node
// Blind 1fb7786 (B: "tactics 'change' toggles a chooser closed"), after the Cut 111 slot target: on the fake engine, two tactic slots,
// a worn slot's `change` opens the choices for that slot and a tap on another tactic switches it (never closes without a switch); the
// combat style's toggle reads `close` while its choices are open and `change` when folded.
//   node web/tests/tactic-change.mjs
import { execFileSync } from "node:child_process";
import { launchBrowser } from "../../tools/browser.mjs";

const url = execFileSync("bash", ["tools/dev.sh"], { cwd: new URL("../../", import.meta.url), encoding: "utf8" }).trim();
const out = [], errors = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const browser = await launchBrowser();
try {
  const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=3002`, { waitUntil: "domcontentloaded" });
  await page.waitForFunction(() => window.__riddle?.booted && window.__riddle.screen === "camp", null, { timeout: 30_000 });
  await page.evaluate(async () => { const r = window.__riddle; const e = JSON.parse(await r.engine.save()); e.lineage.best_depth = 22; e.lineage.gold = 120; e.lineage.heir = 2; e.lineage.graveyard = [{ heir: 1, depth: 6, cause: "goblin" }]; e.st30.runs = { steady: 45 }; r.lineage = await r.engine.load(JSON.stringify(e)); r.go({ kind: "camp" }); });
  await page.waitForSelector('.cmd .tile[data-tile="packages"]');
  await page.click('.cmd .tile[data-tile="packages"]');
  await page.waitForSelector(".pkg-panel .pkg-slot");
  const worn = () => page.evaluate(() => window.__riddle.lineage.packages.tactics ?? []);
  const open = () => page.evaluate(() => !!document.querySelector('.pkg-sec[data-kind="tactic"] .pkg-choices:not([hidden])'));
  const slots = await page.evaluate(() => window.__riddle.lineage.packages.tactic_slots);
  check(slots === 2, `two tactic slots (${slots})`);
  await page.click('[data-edit-slot="0"]');
  await page.click('.pkg-sec[data-kind="tactic"] .chip.pkg.alt');
  await page.waitForFunction(() => window.__riddle.lineage.packages.tactics?.length === 1);
  const first = (await worn())[0];
  await page.click('[data-edit-slot="0"]');
  check(await open(), "a worn slot's `change` opens the choices");
  const other = await page.evaluate(() => document.querySelector('.pkg-sec[data-kind="tactic"] .chip.pkg.alt')?.dataset.pkg);
  await page.click(`.pkg-sec[data-kind="tactic"] .chip.pkg.alt[data-pkg="${other}"]`);
  await page.waitForFunction((o) => window.__riddle.lineage.packages.tactics?.[0] === o, other, { timeout: 10_000 }).catch(() => {});
  const now = await worn();
  check(now.length === 1 && now[0] === other && other !== first, `the tap switches slot 1 (${first} → ${now.join(",")})`);
  const stance = page.locator('[data-change-kind="stance"]');
  if (await stance.count()) {
    const a = await stance.textContent(); await stance.click();
    const b = await page.locator('[data-change-kind="stance"]').textContent(); await page.locator('[data-change-kind="stance"]').click();
    const c = await page.locator('[data-change-kind="stance"]').textContent();
    check(a === "change" && b === "close" && c === "change", `the style's toggle says what it does (${a} → ${b} → ${c})`);
  } else check(false, "the style's toggle is shown");
  await page.close();
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}
for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`tactic-change: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`tactic-change: ok (${out.length} checks)`);
