#!/usr/bin/env node
// c30-legible gates (the owner, a new player: "I didn't understand why there were new buildings or why the run ended early at like D3"),
// headless at 400 × 800:
//   ends    real engine, a fresh lineage, three watched runs (▶▶| as a player would): every run's end shows a reason beat under its sum
//           (the core's `ExitLine.reason`, ≤ 3 words — the first run's `hurt · banked`), and the report's end tile carries it
//   build   real engine diagnostic milestone fixtures, each trigger in turn: every building's arrival names its cause (`first gold home → blacksmith`, ≤ 4 words + the
//           arrow) and draws the eye (its target glows); the staked plot's tag is always visible — the next building and its trigger
//           (`storehouse · first find kept`), from day 0 until the v1 set stands
//
//   node web/tests/legible.mjs [--shots dir] [--part=ends,build]      (part of `pnpm test` in web/)
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { mkdirSync } from "node:fs";
import { launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const shotsArg = process.argv.indexOf("--shots"), shots = shotsArg > 0 ? process.argv[shotsArg + 1] : null;
const partArg = process.argv.find((a) => a.startsWith("--part=")); const parts = partArg ? partArg.slice(7).split(",") : null;
const part = (p) => !parts || parts.includes(p);
if (shots) mkdirSync(shots, { recursive: true });
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const words = (s) => s.split(/\s+/).filter((w) => w && w !== "·" && w !== "→").length;

const browser = await launchBrowser();
let page;
async function open() {
  page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 1 });
  page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
}
const shot = async (name) => { if (shots) { await page.waitForFunction(() => !document.querySelector(".sheet-ghost"), null, {timeout:5000}); await page.screenshot({ path: resolve(shots, `${name}.png`) }); } };
async function until(pred, label, timeout = 20_000, arg) {
  const t = Date.now(); let v;
  while (Date.now() - t < timeout) { v = await page.evaluate(pred, arg).catch(() => null); if (v) return v; await sleep(80); }
  throw new Error(`timeout waiting for ${label}`);
}
const camp = async () => { await until(() => window.__riddle?.booted && window.__riddle.screen === "camp" && window.__town, "camp", 60_000); await sleep(300); };
const screen = () => page.evaluate(() => window.__riddle?.screen);

try {
  await open();
  // ---- the run's end: a reason beat, and the report carries it
  if (part("ends")) {
    await page.goto(`${url}?dev=1&fresh=1&seed=5101&speed=8`, { waitUntil: "domcontentloaded" });
    await camp();
    check(await page.evaluate(() => !window.__riddle.lineage.town.home && window.__riddle.lineage.hero_slots.length === 0), "fresh town is empty before construction");
    await page.locator('.town-tag[data-next="house"]').click();
    await until(() => window.__riddle.lineage.town.home && window.__riddle.lineage.hero_slots.length === 1 && !document.querySelector('main.camp')?.classList.contains('awaiting-home'), "built house and resident");
    await page.evaluate(() => {
      const e=window.__riddle.engine;
      for(const method of ['step','fold']) if(e[method]) {
        const original=e[method].bind(e);
        e[method]=async(...args)=>{const result=await original(...args);for(const ev of (result.step??result).events??[]) if(ev.k==='exit'&&ev.line) (window.__actualExits??=[]).push(ev.line);return result;};
      }
    });
    for (let k = 1; k <= 3; k++) {
      await page.evaluate(() => { window.__beatLog = []; window.__whyLog = []; window.__actualExits=[]; });
      await page.locator("button.send:visible").first().click();
      await until(() => window.__riddle.screen === "watch", "the watch");
      let whyShown = null, endShot = false;
      const t0 = Date.now();
      for (;;) {
        const s = await screen();
        if (s === "exit") { await page.locator(".sheet-wrap .vault-choice .chip").first().click().catch(() => {}); await page.locator(".sheet-wrap .btn.primary").first().click().catch(() => {}); await sleep(200); continue; }
        if (s !== "watch") break;
        whyShown ??= await page.evaluate(() => document.querySelector(".watch .beat-why.show")?.textContent ?? null);
        if (!whyShown) {
          await page.locator('.console [data-tile="speed"]').click();
          const skip=page.locator('.sheet-wrap .sheet [data-tile="skip"]');
          if(await skip.isVisible() && await skip.isEnabled()) await skip.click({timeout:5000}).catch(async e=>{
            if(await screen()==='watch' && await skip.isVisible() && await skip.isEnabled()) throw e;
          });
          if(await page.locator('.sheet-wrap').count()) await page.keyboard.press('Escape');
        }
        else if (k === 1 && !endShot) { await shot("end-beat"); endShot=true; }
        if (Date.now() - t0 > 180_000) throw new Error(`run ${k} still going`);
        await sleep(60);
      }
      const log = await page.evaluate(() => (window.__beatLog ?? []).filter((b) => b.why));
      const wl = await page.evaluate(() => window.__whyLog ?? []);
      const why = whyShown ?? wl[wl.length - 1] ?? "";
      check(!!why && words(why) <= 3, `run ${k}: its end shows a reason beat ≤ 3 words ("${log[log.length - 1]?.text ?? "death"}" · "${why}"${whyShown ? "" : ", logged, not seen"})`);
      const engineWhy = await page.evaluate(() => window.__actualExits.at(-1)?.reason);
      check(why === engineWhy?.replace(/\bbanked\b/g,'collected'), `run ${k}: displayed reason equals actual Rust exit ("${engineWhy}")`);
      // (Cut 30.5, c305-core 2ec1cb0: a new record is a beat, never an end — runs end hurt, out of heals, at a wall or dead)
      if (k === 1) check(/^(hurt · collected|hurt · went home|no heals · collected|slain\b.*|starved)$/.test(why), `run 1: a fresh lineage's first end reads as the hero being sensible ("${why}")`);
      const s = await until(() => ["report", "death"].includes(window.__riddle.screen) && window.__riddle.screen, "the screen after", 30_000);
      if (s === "report") {
        const details=page.locator('.details-fold[aria-expanded="false"]');
        if(await details.count()) await details.click();
        const tw = await until(() => document.querySelector(".report .tile-why")?.textContent ?? null, "the report's reason", 8000).catch(() => null);
        check(tw === why, `run ${k}: the report's end tile carries the same reason ("${tw}")`);
        if (k === 1) await shot("report");
        await page.locator("main.report .gem").first().click();
      } else {
        check(/^slain|^starved/.test(why), `run ${k}: a death's beat names the killer ("${why}")`);
        await page.locator(".cmd [data-tile=camp]").click();
      }
      await camp();
    }
  }

  // ---- Explicit diagnostic milestones on the real Rust wire; construction stays manual.
  if (part("build")) {
    await page.goto(`${url}?dev=1&fresh=1&seed=3111`, { waitUntil: "domcontentloaded" });
    await camp();
    await page.locator('.town-tag[data-next="house"]').click();
    await until(() => window.__riddle.lineage.town.home && !document.querySelector('main.camp')?.classList.contains('awaiting-home'), "house");
    const order = ["blacksmith", "storehouse", "kennel", "bank"];
    const labels = ["forge", "stored gear", "companions", "savings"];
    const triggers = ["first gold home", "first find kept", "first tame", "a night's purse"];
    for (let n = 0; n < order.length; n++) {
      const id = order[n];
      const locked = await page.evaluate(() => {
        const t=document.querySelector('.town-tag'),r=t?.getBoundingClientRect();
        return {text:t?.textContent,visible:!t?.hidden&&r.width>0&&r.top>=0&&r.bottom<=innerHeight,ready:window.__riddle.lineage.town.next_ready,next:window.__riddle.lineage.town.next,trigger:window.__riddle.lineage.town.next_trigger};
      });
      if(n===0) check(!locked.visible && !locked.ready && locked.next===id && locked.trigger===triggers[n],
        'before first gold, the home prioritises hero and Send; Rust retains the locked forge milestone');
      else check(locked.visible && !locked.ready && locked.text.includes(labels[n]) && locked.text.includes(triggers[n]) && words(locked.text)<=5,
        `stage ${n}: locked plot visibly names ${id} and its trigger ("${locked.text}")`);
      await page.evaluate(async n => {
        const a=window.__riddle,s=JSON.parse(await a.engine.save()),L=s.lineage;
        if(n===0) L.banked_depths=[3];
        if(n===1) L.facts.push('vault');
        if(n===2) L.facts.push('tamed:jackal');
        if(n===3) {L.gold=900;L.last_night_net=500;}
        a.lineage=await a.engine.load(JSON.stringify(s));a.go({kind:'camp'});
      }, n);
      await camp();
      check(await page.evaluate(id => window.__riddle.lineage.town.next === id && window.__riddle.lineage.town.next_ready && !window.__riddle.lineage.town.buildings.some(b=>b.id===id), id),
        `stage ${n}: milestone offers ${id} without automatically building it`);
      const marker=page.locator(`.town-tag[data-next="${id}"]`);
      check(/Build.*Free/.test(await marker.textContent()), `stage ${n}: ready marker explicitly offers free construction`);
      await marker.click();
      await until(id => window.__riddle.lineage.town.buildings.some(b=>b.id===id), `manual ${id}`, 20_000, id);
      const st=await page.evaluate(() => {
        const a=document.querySelector('.town-arrival');
        return {why:a?.classList.contains('show')?a.dataset.why:'',glow:[...document.querySelectorAll('.town-hit.arrived')].map(b=>b.dataset.building)};
      });
      check(st.why===`${triggers[n]} → ${id}` && words(st.why.split('→')[0])<=4,
        `stage ${n}: ${id} arrival names exact Rust trigger ("${st.why}")`);
      check(st.glow.includes(id), `stage ${n}: the newly built ${id} glows`);
      if(n===0) await shot('arrival');
    }
    check(await page.locator('.town-tag').evaluate(e=>e.hidden), 'no next tag once all four buildings stand');
  }

} catch (e) {
  check(false, `threw: ${e.message}`);
}
const errs = errors.filter((e) => !/favicon|404|net::ERR/.test(e));
check(errs.length === 0, `no page errors (${errs.slice(0, 3).join(" | ")})`);
await browser.close();
console.log(out.join("\n"));
console.log(failed ? `legible: ${failed} failed` : "legible: all passed");
process.exit(failed ? 1 : 0);
