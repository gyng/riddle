#!/usr/bin/env node
// Run-clear gates (the owner, 2026-10-02: "include item rarity colours + icons, each run should have the clear screen? … eg,
// hurt/went home"), headless at 400 × 800. The card is off under automation unless `?runclear=1` (the older gates' meaning).
//   run       real wasm: a watched run's end shows the card — the exit stamp in the exit vocabulary, the core's reason, the floor,
//             ≤ 20 words at rest; its finds in their rarity rims (a non-common rim among them); a tap goes on to the town
//   lift      the `report` tile lifts the card to the run's report underneath (the report keeps its own clock)
//   auto      the card continues by itself (`?autodismiss=0.1`: its 7 s ring on the gem) to the town
//   absence   fake engine, 8 h away: the last run's card first (its ring on the card), a tap lifts it to the absence's report; by itself too
//   death     fake engine, a run that dies: the watch goes to the death screen and never through a card or report (one screen), and
//             the death screen's header carries the card's strip (the floor)
//   rarity    the five tiers have five distinct rarity pigments; the core reads a vault mace +2 as epic, a dagger common
//   node web/tests/runclear.mjs [--part=a,b]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const STAMPS = ["collected", "returned", "stalled", "repelled", "died"];
const MAX_WORDS = 20;

const browser = await launchBrowser();
const page = await browser.newPage({ viewport: { width: 400, height: 800 } });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
async function until(pred, label, timeout = 30_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const screen = () => page.evaluate(() => window.__riddle?.screen);
/** The card as a reader sees it at rest: its words (a token with a letter or a digit; `→`, `·`, `$` alone are marks), stamp, reason. */
const card = () => page.evaluate(() => {
  const c = document.querySelector(".run-clear"); if (!c) return null;
  const words = c.innerText.split(/\s+/).filter((w) => /[\p{L}\p{N}]/u.test(w));
  const rims = [...c.querySelectorAll(".item-ico")].map((i) => ({ r: i.dataset.rarity, rim: getComputedStyle(i).boxShadow, bg: getComputedStyle(i).backgroundImage }));
  return { words: words.length, text: words.join(" "), stamp: c.querySelector(".rc-stamp")?.textContent?.trim().toLowerCase(), why: c.querySelector(".rc-reason")?.textContent?.trim(),
    rawWhy: c.querySelector(".rc-reason")?.dataset.why, depth: c.querySelector(".rc-depth")?.textContent?.trim(), gold: c.querySelector(".rc-coins")?.dataset.gold, rims, clear: document.querySelector("main.report")?.dataset.clear };
});
/** A send by hand from the town (the gem), then the keep sheet answered as it comes, until a screen after the watch. */
async function sendAndEnd(timeout = 150_000) {
  // New towns follow the owner's manual house → resident → Send opening.
  if (await page.evaluate(() => window.__riddle?.lineage.town?.home === false)) {
    await page.locator('.town-tag[data-next="house"]').click();
    await until(() => window.__riddle.lineage.town.home === true, "the manually built house");
  }
  await until(() => window.__riddle?.screen === "camp" && document.querySelector(".gem.send:not([disabled])"), "the send gem");
  await page.click(".gem.send");
  await until(() => window.__riddle.screen === "watch", "the watch");
  const t = Date.now();
  while (Date.now() - t < timeout) {
    const s = await screen();
    if (s !== "watch" && s !== "exit") return s;
    const keep = page.locator(".sheet button", { hasText: /^keep$/ });
    if (await keep.count()) await keep.first().click().catch(() => {});
    await sleep(250);
  }
  throw new Error("the run did not end");
}

try {
  if (part("run") || part("lift") || part("auto")) {
    // real wasm: the core's rarity and reason. Seed 1's first send banks a new best with an uncommon axe among its finds
    await page.goto(`${url}?seed=1&fresh=1&speed=8&runclear=1`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.booted, "boot", 60_000);
    if (part("run")) {
      const s = await sendAndEnd();
      const c = await until(() => document.querySelector(".run-clear") && window.__riddle.screen, "the card");
      const k = await card();
      check(s === "report" && c === "report" && !!k, `a watched run's end shows the card (${s})`);
      check(STAMPS.includes(k.stamp), `the stamp is an exit word (${k.stamp})`);
      const line = await page.evaluate(() => { const r = window.__riddle.view.report; const x = r?.exits?.[r.exits.length - 1]; return x ? { reason: x.reason, end: x.end, reached: x.reached, finds: (x.finds ?? []).length } : null; });
      check(!!line?.reason && k.rawWhy === line.reason && k.why === line.reason.replace(/\bbanked\b/g, "collected"), `the core reason retains current gold wording (${k.why} | ${line?.reason})`);
      check(!!line?.end && !!line.reached && k.depth?.startsWith(`D${line.reached}`), `the end and the floor are the core's (${line?.end} · D${line?.reached} | ${k.depth})`);
      check(k.words <= MAX_WORDS, `≤ ${MAX_WORDS} words at rest (${k.words}: ${k.text})`);
      const rimmed = k.rims.filter((x) => x.r && x.rim === "none" && x.bg === "none");
      check(k.rims.length === Math.min(6, line.finds) && rimmed.length === k.rims.length, `every find has an unframed silhouette, ≤ 6 shown (${rimmed.length}/${k.rims.length} of ${line.finds})`);
      check(k.rims.some((x) => x.r !== "common"), `a find above common shows its tier (${k.rims.map((x) => x.r).join(", ")})`);
      check(k.clear === "run", `the card stands over the run's report (${k.clear})`);
      await page.waitForTimeout(400);
      await page.click(".run-clear .rc-reason");
      await until(() => window.__riddle.screen === "camp", "the town after a tap", 10_000);
      check((await screen()) === "camp", "a tap on the card goes on to the town");
    }
    if (part("lift")) {
      // Keep this card fixture independent: a later run may legitimately die.
      await page.goto(`${url}?seed=1&fresh=1&speed=8&runclear=1`, { waitUntil: "domcontentloaded" });
      await until(() => window.__riddle?.booted, "boot", 60_000);
      await sendAndEnd();
      await until(() => document.querySelector(".run-clear"), "the card");
      await page.locator(".report .console [data-tile=\"report\"]").click();
      const lifted = await until(() => !document.querySelector(".run-clear") && document.querySelector(".report-sheet") && !document.querySelector(".report-sheet").hidden && window.__riddle.screen, "the report under the card", 5000);
      check(lifted === "report", "the `report` tile lifts the card to the run's report");
    }
    if (part("auto")) {
      await page.goto(`${url}?seed=1&fresh=1&speed=8&runclear=1&autodismiss=0.1`, { waitUntil: "domcontentloaded" });
      await until(() => window.__riddle?.booted, "boot", 60_000);
      await sendAndEnd();
      const L = await until(() => document.querySelector(".run-clear") && window.__autodismiss.live(), "the card's clock");
      check(L.length === 1 && /gem/.test(L[0].cls) && L[0].ms === 700, `the card's ring is on the gem, 7 s (${L.map((x) => `${x.cls.split(" ")[0]} ${x.ms}`).join(", ")})`);
      const t = Date.now();
      await until(() => window.__riddle.screen === "camp", "the town after the card's time", 8000);
      check(Date.now() - t < 5000, `the card continued to the town by itself (${Date.now() - t} ms)`);
    }
  }

  if (part("absence")) {
    for (const auto of [false, true]) {
      await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=4101&absent=8h&runclear=1${auto ? "&autodismiss=0.25" : ""}`, { waitUntil: "domcontentloaded" });
      await until(() => window.__riddle?.screen === "report" && document.querySelector(".run-clear"), "the absence's card", 60_000);
      const k = await card();
      check(k.clear === "absence" && STAMPS.includes(k.stamp) && !!k.why && k.words <= MAX_WORDS, `${auto ? "auto: " : ""}an absence opens on its last run's card (${k.stamp} · ${k.why} · ${k.words} words)`);
      const hidden = await page.evaluate(() => document.querySelector(".report-sheet")?.hidden);
      check(hidden === true, "the absence's report waits under it");
      if (auto) {
        const L = await page.evaluate(() => window.__autodismiss.live());
        check(L.length === 1 && /run-clear/.test(L[0].cls) && L[0].ms === 1500, `its ring is on the card, 6 s (${L.map((x) => `${x.cls.split(" ")[1] ?? x.cls} ${x.ms}`).join(", ")})`);
        await until(() => !document.querySelector(".run-clear") && !document.querySelector(".report-sheet").hidden, "the report after the card's time", 6000);
        const L2 = await page.evaluate(() => window.__autodismiss.live());
        check(L2.length === 1 && /gem/.test(L2[0].cls) && L2[0].ms === 3000, `then the report's own clock on its gem (${L2.map((x) => `${x.cls.split(" ")[0]} ${x.ms}`).join(", ")})`);
      } else {
        await page.click(".run-clear .rc-stamp");
        const ok = await until(() => !document.querySelector(".run-clear") && !document.querySelector(".report-sheet").hidden && window.__riddle.screen, "the report after a tap", 5000);
        check(ok === "report", "a tap lifts the card to the absence's report");
      }
    }
  }

  if (part("death")) {
    // a set that only attacks walks into a death (the fake's systems off: the rows are the set)
    const rules = encodeURIComponent("foes>=1 → attack nearest");
    await page.goto(`${url}?dev=1&engine=fake&systems=none&fresh=1&seed=13&rules=${rules}&speed=fast&runclear=1`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.booted, "boot", 60_000);
    let died = false;
    for (let i = 0; i < 6 && !died; i++) {
      const seen = new Set();
      const poll = setInterval(() => { void page.evaluate(() => ({ s: window.__riddle?.screen, c: !!document.querySelector(".run-clear") })).then((x) => { if (x) { seen.add(x.s); if (x.c) seen.add("card"); } }).catch(() => {}); }, 60);
      const s = await sendAndEnd();
      await sleep(600); clearInterval(poll);
      if (s === "death") {
        died = true;
        check(!seen.has("report") && !seen.has("card"), `a death goes from the watch to the death screen alone (${[...seen].join(" → ")})`);
        const strip = await page.evaluate(() => document.querySelector(".death .rc-strip")?.textContent?.trim() ?? null);
        check(!!strip && /^D\d+/.test(strip), `the death screen's header carries the card's strip (${strip})`);
      } else {
        await page.evaluate(() => window.__riddle.go({ kind: "camp" }));
      }
    }
    check(died, "a run died within six sends");
  }

  if (part("rarity")) {
    await page.goto(`${url}?seed=7&fresh=1&runclear=1`, { waitUntil: "domcontentloaded" });
    await until(() => window.__riddle?.booted, "boot", 60_000);
    // the five tiers' rims are five distinct pigments
    const rims = await page.evaluate(() => {
      const host = document.createElement("div"); document.body.appendChild(host);
      const out = ["common", "uncommon", "rare", "epic", "legendary"].map((r) => { const e = document.createElement("span"); e.className = `item-ico r-${r}`; host.appendChild(e); return getComputedStyle(e).getPropertyValue("--r").trim(); });
      host.remove(); return out;
    });
    check(new Set(rims).size === 5 && rims.every((x) => /^#[0-9a-f]{6}$/i.test(x)), `five distinct rarity pigments (${rims.join(" ")})`);
    // the cage and the vault: real items from the core in their rims (a vault item put there through the engine's save)
    const shown = await page.evaluate(async () => {
      const r = window.__riddle; const save = JSON.parse(await r.engine.save());
      save.lineage.vault = [{ id: 7001, kind: "mace", enchant: 2, known: true }, { id: 7002, kind: "dagger", known: true }];
      r.lineage = await r.engine.load(JSON.stringify(save));
      return r.lineage.vault.map((v) => `${v.label}:${v.rarity ?? "common"}`);
    });
    check(shown.includes("mace +2:epic") && shown.includes("dagger:common"), `the core reads a mace +2 epic, a dagger common (${shown.join(", ")})`);
  }
} catch (e) {
  failed++; out.push(`FAIL ${e.message}`);
}
const fatal = errors.filter((e) => !/favicon|ResizeObserver|net::ERR/.test(e));
check(!fatal.length, `no console errors (${fatal.slice(0, 3).join(" | ")})`);
await browser.close();
console.log(out.join("\n"));
console.log(failed ? `runclear: ${failed} failed` : "runclear: all passed");
process.exit(failed ? 1 : 0);
