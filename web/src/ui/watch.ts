// Watch: viewer canvas full-bleed; HUD (hp, depth, alert), speed 1× 4× ▶▶| ⏸, callout ticker, bail.
import type { App, Mounted } from "../app";
import type { Ev, Highlight, InvItem, ReturnReport, Snapshot } from "../engine/types";
import { h, replace } from "./dom";
import { makeViewer, type Viewer } from "./viewer";

const INTERESTING = new Set(["hurt", "die", "telegraph", "pickup", "use", "fact", "steal", "ally", "descend", "exit", "spawn"]);

export function renderWatch(app: App): Mounted {
  const canvas = h("canvas", { class: "view" });
  const hpFill = h("span", { class: "fill" });
  const hpText = h("span", { class: "num" });
  const depth = h("span", { class: "num depth" });
  const alert = h("span", { class: "alert num" });
  const ticker = h("div", { class: "ticker" });
  const pause = h("button", { class: "hud-btn", onclick: () => setSpeed(speed ? 0 : 1) }, "⏸");
  const s1 = h("button", { class: "hud-btn on", onclick: () => setSpeed(1) }, "1×");
  const s4 = h("button", { class: "hud-btn", onclick: () => setSpeed(4) }, "4×");
  const skip = h("button", { class: "hud-btn", onclick: () => skipToEvent() }, "▶▶|");
  const bail = h("button", { class: "hud-btn bail", onclick: () => doBail() }, /* copy:button */ "bail");
  const el = h("main", { class: "watch" }, canvas,
    h("div", { class: "hud top" }, h("div", { class: "hp" }, h("span", { class: "track" }, hpFill), hpText), depth, alert, pause),
    ticker,
    h("div", { class: "hud bottom" }, s1, s4, skip, bail));

  let viewer: Viewer | null = null;
  let speed = 1, timer = 0, done = false, disposed = false, overridden = false, tickerTimer = 0;
  let snap: Snapshot = app.engine.send();
  const runId = snap.run.id;
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [];
  let turns = 0;

  function paintHud(s: Snapshot): void {
    const p = s.hero.max_hp ? s.hero.hp / s.hero.max_hp : 0;
    hpFill.style.width = `${Math.round(p * 100)}%`;
    hpFill.classList.toggle("low", p < 0.3);
    replace(hpText, `${s.hero.hp}/${s.hero.max_hp}`);
    replace(depth, `D${s.depth}`);
    replace(alert, "!".repeat(s.alert));
  }
  function callout(text: string): void {
    replace(ticker, text); ticker.classList.add("show");
    clearTimeout(tickerTimer); tickerTimer = window.setTimeout(() => ticker.classList.remove("show"), 1800 / Math.max(1, speed));
  }
  function absorb(evs: Ev[]): "bank" | "return" | "death" | null {
    let exit: "bank" | "return" | "death" | null = null;
    for (const ev of evs) {
      if (ev.k === "callout") callout(ev.text);
      else if (ev.k === "fact") learned.push(ev.fact);
      else if (ev.k === "pickup") found.push({ id: ev.id, kind: ev.item, known: true, label: ev.item });
      else if (ev.k === "note") notes.push({ pattern: "note", score: 0, t: ev.t, run_id: runId, text: ev.text });
      else if (ev.k === "exit") exit = ev.tier;
    }
    return exit;
  }
  function handle(evs: Ev[], s: Snapshot): void {
    snap = s; turns += 1;
    const exit = absorb(evs);
    viewer?.apply(evs);
    if (evs.some((e) => e.k === "descend")) viewer?.load(s);
    paintHud(s);
    if (exit) finish(exit);
  }
  function tick(): void {
    if (done || disposed) return;
    const r = app.engine.step(1);
    handle(r.events, r.snapshot);
    if (!done && speed > 0) timer = window.setTimeout(tick, 1000 / speed);
  }
  function setSpeed(n: number): void {
    speed = n;
    for (const [b, v] of [[s1, 1], [s4, 4]] as const) b.classList.toggle("on", n === v);
    pause.classList.toggle("on", n === 0);
    replace(pause, n === 0 ? "▶" : "⏸");
    clearTimeout(timer);
    if (n > 0) { viewer?.setSpeed(n); if (!done) timer = window.setTimeout(tick, 1000 / n); }
  }
  function skipToEvent(): void {
    if (done) return;
    clearTimeout(timer);
    const all: Ev[] = []; let s = snap; let hit = false;
    for (let i = 0; i < 300 && !hit; i++) {
      const r = app.engine.step(1); all.push(...r.events); s = r.snapshot; turns += 1;
      hit = r.run_over || r.events.some((e) => INTERESTING.has(e.k));
    }
    turns -= 1;
    handle(all, s);
    viewer?.skipToEvent();
    if (!done && speed > 0) timer = window.setTimeout(tick, 1000 / speed);
  }
  function doBail(): void {
    if (done || overridden) return;
    overridden = true; bail.classList.add("on");
    app.engine.setRules({ rows: [{ conds: [], verb: { v: "return" } }, ...app.rules.rows] });
    if (speed === 0) setSpeed(1);
  }
  function finish(tier: "bank" | "return" | "death"): void {
    done = true; clearTimeout(timer);
    if (overridden) app.engine.setRules(app.rules);
    app.refresh();
    if (tier === "death") { app.go({ kind: "death", death: app.engine.death(runId) }); return; }
    const L = app.lineage;
    const bests: string[] = []; for (let d = before.best + 1; d <= L.best_depth; d++) bests.push(`D${d}`);
    const report: ReturnReport = {
      elapsed_s: turns, runs: 1, sampled: false, learned, bests, found, deaths: [], pending: [],
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap,
    };
    app.go({ kind: "report", report });
  }

  void makeViewer(canvas).then(({ viewer: v }) => {
    if (disposed) { v.dispose(); return; }
    viewer = v; v.load(snap); v.setSpeed(speed);
  });
  paintHud(snap);
  timer = window.setTimeout(tick, 1000);
  return { el, dispose: () => { disposed = true; clearTimeout(timer); clearTimeout(tickerTimer); viewer?.dispose(); if (overridden && !done) app.engine.setRules(app.rules); } };
}
