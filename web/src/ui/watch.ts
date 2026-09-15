// Watch: viewer canvas full-bleed; HUD (hp, depth, alert), speed 1× 4× ▶▶| ⏸, callout ticker, bail.
import type { App, Mounted } from "../app";
import type { Ev, Highlight, InvItem, ReturnReport, Snapshot } from "../engine/types";
import { h, replace } from "./dom";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt, xpToNext } from "../engine/classes";
import { openSheet } from "./sheet";
import { salvageValue } from "./salvage";
import { vaultSlots } from "./unlocks";
import type { InvItem as Item } from "../engine/types";

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
  const cls = app.lineage.class;
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks, level: app.lineage.classes?.[cls]?.level ?? 1, xp: app.lineage.classes?.[cls]?.xp ?? 0, renown: app.lineage.renown ?? 0, rank: app.lineage.rank ?? 0 };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [], tamed: string[] = [], lost: string[] = [];
  let turns = 0;
  const kinds = new Map<number, string>(snap.entities.map((e) => [e.id, e.kind]));
  const lostKind = (id: number): string => kinds.get(id) ?? "?";

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
      else if (ev.k === "tame" && ev.ok) tamed.push(ev.kind);
      else if (ev.k === "ally" && ev.state === "lost") lost.push(lostKind(ev.id));
      else if (ev.k === "spawn" && ev.e.cid !== undefined) kinds.set(ev.e.id, ev.e.kind);
      else if (ev.k === "level") { for (const v of verbsAt(ev.class, ev.level)) learned.push(`verb:${v}`); callout(`${ev.class} ${ev.level}`); }
    }
    return exit;
  }
  let pendingExit: { items: Item[]; tier: string } | undefined;
  function handle(evs: Ev[], s: Snapshot, pending?: { items: Item[]; tier: string }): void {
    snap = s; turns += 1;
    const exit = absorb(evs);
    viewer?.apply(evs);
    if (evs.some((e) => e.k === "descend")) viewer?.load(s);
    for (const e of s.entities) kinds.set(e.id, e.kind);
    paintHud(s);
    if (pending) pendingExit = pending;
    if (exit) finish(exit);
  }
  function tick(): void {
    if (done || disposed) return;
    const r = app.engine.step(1);
    handle(r.events, r.snapshot, r.exit_pending);
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
      if (r.exit_pending) pendingExit = r.exit_pending;
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
  function xpGained(): number {
    const c = app.lineage.classes?.[cls] ?? { level: 1, xp: 0 }; let g = c.xp - before.xp;
    for (let l = before.level; l < c.level; l++) g += xpToNext(l);
    return Math.max(0, g);
  }
  function finish(tier: "bank" | "return" | "death"): void {
    done = true; clearTimeout(timer);
    if (overridden) app.engine.setRules(app.rules);
    if (pendingExit) { const p = pendingExit; pendingExit = undefined; exitSheet(p, () => finish(tier)); return; }
    app.refresh();
    if (tier === "death") { app.go({ kind: "death", death: app.engine.death(runId), lost }); return; }
    const L = app.lineage;
    const bests: string[] = []; for (let d = before.best + 1; d <= L.best_depth; d++) bests.push(`D${d}`);
    const report: ReturnReport = {
      elapsed_s: turns, runs: 1, sampled: false, learned, bests, found, deaths: [], pending: [],
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap, tamed, hatched: [], lost,
      xp: { class: cls, gained: xpGained(), level_ups: (L.classes?.[cls]?.level ?? 1) - before.level },
      salvaged: [], renown: { gained: (L.renown ?? 0) - before.renown, rank: L.rank ?? 0, ranks_up: (L.rank ?? 0) - before.rank },
    };
    app.go({ kind: "report", report });
  }

  // Addendum D: choose what to keep before the run settles
  function exitSheet(p: { items: Item[]; tier: string }, then: () => void): void {
    const free = Math.max(0, vaultSlots(app.lineage.unlocks) - app.lineage.vault.length);
    const keep = new Set<number>();
    openSheet((close) => {
      const chips = h("div", { class: "chips" });
      const count = h("span", { class: "num dim" });
      const paint = (): void => {
        replace(count, `${keep.size}/${free}`);
        replace(chips, ...p.items.map((it) => h("button", { class: `chip item${keep.has(it.id) ? " on" : ""}`, onclick: () => {
          if (keep.has(it.id)) keep.delete(it.id); else if (keep.size < free) keep.add(it.id);
          paint();
        } }, it.label, " ", keep.has(it.id) ? h("b", null, "⌂") : h("b", { class: "num gold" }, `$${salvageValue(it.kind, p.tier)}`))));
      };
      paint();
      return h("div", { class: "sheet-body" },
        h("div", { class: "label row-label" }, /* copy:label */ "vault", " ", count),
        chips,
        h("button", { class: "btn primary wide", onclick: () => { app.lineage = app.engine.keep([...keep]); close(); then(); } }, /* copy:button */ "keep"));
    });
  }

  void makeViewer(canvas).then(({ viewer: v }) => {
    if (disposed) { v.dispose(); return; }
    viewer = v; v.load(snap); v.setSpeed(speed);
  });
  paintHud(snap);
  timer = window.setTimeout(tick, 1000);
  return { el, dispose: () => { disposed = true; clearTimeout(timer); clearTimeout(tickerTimer); viewer?.dispose(); if (overridden && !done) app.engine.setRules(app.rules); } };
}
