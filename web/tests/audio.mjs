#!/usr/bin/env node
// Cut 10 §4 gate: every cue schedules ≤ 200 ms from at most two oscillators and one gain envelope (the death note is the
// one exception: a single low note fading over 1 s); nothing schedules while muted; the mute persists; the camp drone is up
// while the camp is open and gone in a run; cues fire on their events in a watched fake run. Runs on the GPU harness
// (tools/browser.mjs) against the dev server (tools/dev.sh, :5219) with the fake engine (`?engine=fake&dev=1`).
//
//   node web/tests/audio.mjs        (part of `pnpm test` in web/)
//
// The spy: an init script wraps AudioContext.prototype.createOscillator / createGain so every node's start/stop and
// every gain automation lands in window.__audioLog before the page's own code runs. The page exposes the audio layer as
// window.__audio (dev); a click unlocks the context (the harness's click is a user gesture).
import { execFileSync } from "node:child_process";
import { resolve, dirname } from "node:path";
import { fileURLToPath } from "node:url";
import { HEADLESS_ARGS, launchBrowser } from "../../tools/browser.mjs";

const ROOT = resolve(dirname(fileURLToPath(import.meta.url)), "../..");
const url = execFileSync("bash", [resolve(ROOT, "tools/dev.sh")], { encoding: "utf8" }).trim();
const sleep = (ms) => new Promise((r) => setTimeout(r, ms));
const errors = [], out = [];
let failed = 0;
const check = (ok, what) => { out.push(`${ok ? "ok  " : "FAIL"} ${what}`); if (!ok) failed++; };
const CUE_MAX = 0.2, DEATH_MAX = 1.0;

const browser = await launchBrowser({ args: [...HEADLESS_ARGS, "--autoplay-policy=no-user-gesture-required"] });
const page = await browser.newPage({ viewport: { width: 400, height: 800 }, deviceScaleFactor: 2 });
page.on("console", (m) => { if (m.type() === "error") errors.push(`console.error: ${m.text()}`); });
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
await page.addInitScript(() => {
  const log = (window.__audioLog = []);
  let seq = 0;
  const P = AudioContext.prototype;
  const wrapOsc = P.createOscillator, wrapGain = P.createGain;
  P.createOscillator = function () {
    const o = wrapOsc.call(this); const rec = { kind: "osc", id: seq++, start: null, stop: null, freqs: [] };
    log.push(rec);
    const s0 = o.start.bind(o), s1 = o.stop.bind(o), f = o.frequency.setValueAtTime.bind(o.frequency);
    o.start = (t) => { rec.start = t ?? this.currentTime; return s0(t); };
    o.stop = (t) => { rec.stop = t ?? this.currentTime; return s1(t); };
    o.frequency.setValueAtTime = (v, t) => { rec.freqs.push([v, t]); return f(v, t); };
    return o;
  };
  P.createGain = function () {
    const g = wrapGain.call(this); const rec = { kind: "gain", id: seq++, ramps: [] };
    log.push(rec);
    for (const m of ["setValueAtTime", "linearRampToValueAtTime", "exponentialRampToValueAtTime", "setTargetAtTime"]) {
      const orig = g.gain[m].bind(g.gain);
      g.gain[m] = (...a) => { rec.ramps.push([m, ...a]); return orig(...a); };
    }
    return g;
  };
});

const state = () => page.evaluate(() => { const r = window.__riddle; return r ? { screen: r.screen, booted: r.booted, busy: r.engineBusy } : null; });
async function waitFor(pred, label, timeout = 20_000) {
  const t = Date.now(); let s = null;
  while (Date.now() - t < timeout) { s = await state(); if (pred(s)) return s; await sleep(100); }
  throw new Error(`timeout waiting for ${label} (screen=${s?.screen} booted=${s?.booted})`);
}
const logLen = () => page.evaluate(() => window.__audioLog.length);
const logFrom = (i) => page.evaluate((i) => window.__audioLog.slice(i), i);
/** Fire one cue and return the nodes it scheduled. */
async function cue(name, opts) {
  const i = await logLen();
  const played = await page.evaluate(([n, o]) => window.__audio.cue(n, o), [name, opts ?? {}]);
  return { played, nodes: await logFrom(i) };
}
const span = (nodes) => Math.max(0, ...nodes.filter((n) => n.kind === "osc").map((n) => n.stop - n.start));

try {
  await page.goto(`${url}?dev=1&engine=fake&fresh=1&seed=7`, { waitUntil: "domcontentloaded" });
  await waitFor((s) => s?.booted && s.screen === "camp", "camp");
  check(await page.evaluate(() => !!window.__audio), "window.__audio is exposed in dev");
  check(!(await page.evaluate(() => window.__audio.unlocked)), "the context is not created before a gesture");
  const before = await cue("hit", { dmg: 5 });
  check(!before.played && before.nodes.length === 0, "a cue before the first gesture schedules nothing");
  // the first gesture unlocks the context; the camp drone comes up
  await page.keyboard.press("Shift"); await sleep(300);   // a key is a gesture too; a click could land on a chip
  check(await page.evaluate(() => window.__audio.unlocked), "the first click creates the AudioContext");
  const drone = await page.evaluate(() => window.__audio.droneState());
  check(drone === "warrens", `the camp drone is up for the next floor's biome (${drone})`);
  const droneNodes = await logFrom(0);
  check(droneNodes.filter((n) => n.kind === "osc").length === 2 && droneNodes.some((n) => n.kind === "gain" && n.ramps.some((r) => r[0] === "linearRampToValueAtTime" && r[1] <= 0.03)), "the drone is two oscillators under a very quiet gain (≤ 0.03)");

  // every cue: ≤ 2 oscillators, exactly 1 gain envelope, every oscillator stops within 200 ms of its start
  // juice pass 2: the family timbres, the boss moments, the chrome and the Cut 27 surfaces keep the same instrument bar
  const cues = [["hit", { dmg: 1 }], ["hit", { dmg: 20 }], ["slay"], ["rule"], ["telegraph"], ["exit_bank"], ["exit_return"], ["level"], ["unlock"],
    ["strike", { dmg: 5, kind: "skeleton" }], ["strike", { dmg: 5, kind: "iron_golem" }], ["slay", { kind: "bloat" }], ["hit", { dmg: 6, kind: "wraith" }],
    ["boss_in"], ["click"], ["boss_break"], ["edit"], ["boss_down"], ["buy"], ["verdict"], ["fold"], ["scene"], ["scene_end", { up: true }]];
  const pitch = [];
  for (const [name, opts] of cues) {
    const { played, nodes } = await cue(name, opts);
    const oscs = nodes.filter((n) => n.kind === "osc"), gains = nodes.filter((n) => n.kind === "gain");
    const len = span(nodes);
    check(played && oscs.length >= 1 && oscs.length <= 2 && gains.length === 1 && oscs.every((o) => o.start !== null && o.stop !== null), `${name}: ${oscs.length} oscillator(s) + ${gains.length} gain`);
    check(len <= CUE_MAX + 1e-6, `${name}: schedules ${Math.round(len * 1000)} ms ≤ 200 ms`);
    if (name === "hit") pitch.push(oscs[0].freqs[0][0]);
  }
  check(pitch.length >= 2 && pitch[0] > pitch[1], `hit pitch falls with damage (${pitch.slice(0, 2).map((p) => Math.round(p)).join(" → ")} Hz)`);
  {  // juice pass 2: no two consecutive blows are the same sound (pitch, filter, noise slice and length are jittered per cue)
    const sigs = [];
    for (let i = 0; i < 8; i++) { await page.evaluate(() => window.__audio.cue("strike", { dmg: 4, kind: "goblin" })); sigs.push(await page.evaluate(() => window.__audio.log.at(-1)?.v)); await sleep(20); }
    check(sigs.every((v, i) => i === 0 || (v && v !== sigs[i - 1])), `8 strikes in a row: no two consecutive identical (${new Set(sigs).size} distinct)`);
  }
  {
    const { nodes } = await cue("exit_death");
    const oscs = nodes.filter((n) => n.kind === "osc"), gains = nodes.filter((n) => n.kind === "gain");
    const len = span(nodes);
    check(oscs.length === 1 && gains.length === 1 && len <= DEATH_MAX + 1e-6 && len > CUE_MAX, `exit_death: one low note fading over ${Math.round(len * 1000)} ms (≤ 1 s)`);
    check(oscs[0].freqs[0][0] < 100 && gains[0].ramps.some((r) => r[0] === "exponentialRampToValueAtTime"), "exit_death: low pitch, exponential fade");
  }
  // mute: nothing schedules, the drone stops, the flag persists across a reload
  await page.evaluate(() => window.__audio.setMuted(true));
  await sleep(50);
  const muted = await cue("slay");
  check(!muted.played && muted.nodes.filter((n) => n.kind === "osc").length === 0, "muted: a cue schedules no oscillator");
  check((await page.evaluate(() => window.__audio.droneState())) === null, "muted: the drone is gone");
  check((await page.evaluate(() => localStorage.getItem("riddle.mute"))) === "1", "mute is persisted");
  // the settings sheet's toggle reads the state and flips it back
  await page.locator("button.gear").click(); await sleep(200);
  const mute = page.locator(".sheet-wrap button.mute");
  check((await mute.count()) === 1 && (await mute.getAttribute("aria-pressed")) === "true", "settings: the mute toggle shows on");
  await mute.click(); await sleep(200);
  check(!(await page.evaluate(() => window.__audio.muted)) && (await page.evaluate(() => localStorage.getItem("riddle.mute"))) === "0", "settings: tapping mute unmutes and persists");
  check((await page.evaluate(() => window.__audio.droneState())) === "warrens", "unmuted: the drone returns in the camp");
  await page.keyboard.press("Escape"); await sleep(100);

  // a watched run: the drone stops, and cues fire on their events (hit / slay / rule / telegraph, then the exit)
  const i0 = await logLen();
  await page.evaluate(() => window.__riddle.go({ kind: "watch" }));
  await waitFor((s) => s?.screen === "watch", "watch");
  await sleep(500);
  check((await page.evaluate(() => window.__audio.droneState())) === null, "in a run the drone is gone");
  const bed = await page.evaluate(() => window.__audio.bedState());
  check(bed === "warrens", `in a run the biome's ambience bed is up (${bed})`);
  const t = Date.now();
  while (Date.now() - t < 90_000) {
    const s = await state();
    if (s?.screen === "exit" && await page.locator(".sheet-wrap .vault-choice .chip").count()) { await page.locator(".sheet-wrap .vault-choice .chip").first().click().catch(() => {}); await sleep(200); continue; }
    if (s?.screen !== "watch") break;
    await sleep(200);
  }
  await sleep(300);
  check((await page.evaluate(() => window.__audio.bedState())) === null, "out of the run the bed is gone");
  const fired = await page.evaluate(() => window.__audio.log.map((x) => x.cue));
  const kinds = new Set(fired);
  check(kinds.has("hit") || kinds.has("slay"), `the run fired combat cues: ${[...kinds].join(", ")}`);
  check(kinds.has("rule"), "a player row firing ticked");
  check([...kinds].some((k) => k.startsWith("exit_")), "the exit played its cue");
  const runNodes = await logFrom(i0);
  const oscs = runNodes.filter((n) => n.kind === "osc" && n.stop !== null);
  check(oscs.length > 0 && oscs.every((o) => o.stop - o.start <= DEATH_MAX + 1e-6), `every oscillator of the run (${oscs.length}) stopped within its bar`);
} catch (e) {
  errors.push(`walk aborted: ${e.message}`);
} finally {
  await browser.close().catch(() => {});
}

for (const l of out) console.log(l);
for (const e of errors) console.error(e);
if (failed || errors.length) { console.error(`audio: FAIL (${failed} assertion(s), ${errors.length} error(s))`); process.exit(1); }
console.log(`audio: ok (${out.length} checks)`);
