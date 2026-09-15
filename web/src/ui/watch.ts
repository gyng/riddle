// Watch: viewer canvas full-bleed; HUD (hp, depth, alert), speed 1× 4× ▶▶| ⏸, callout ticker, bail.
//
// Pacing (Addendum E): the viewer owns the clock (10 ticks/s × speed). The engine worker is pumped in
// 10-tick batches whenever it is fewer than LEAD ticks ahead of the viewer, so events always arrive
// before the viewer needs them and the engine never runs far ahead (≤ 22 ticks, under the viewer's
// dead-air threshold). HUD hp/depth and the ticker are queued by tick and released at the viewer's
// clock, so what the numbers say matches what the sprites do.
import type { App, Mounted } from "../app";
import type { Ev, Highlight, InvItem, ReturnReport, Snapshot, StepResult } from "../engine/types";
import { h, replace } from "./dom";
import { makeViewer, type Viewer } from "./viewer";
import { verbsAt, xpToNext } from "../engine/classes";
import { openSheet } from "./sheet";
import { salvageValue } from "./salvage";
import { vaultSlots } from "./unlocks";
import { verbLabel } from "./tokens";

type Tier = "bank" | "return" | "death";
const INTERESTING = new Set(["hurt", "die", "telegraph", "pickup", "use", "fact", "steal", "ally", "descend", "exit", "spawn", "tame"]);
const LEAD = 12, BATCH = 10;        // ticks: pump when the engine is < LEAD ahead; step BATCH at a time
const PUMP_MS = 100;
const EXIT_GRACE_MS = 4000;         // wait for the viewer to drain after an exit, at most this long
const PERSIST_MS = 5000;
const CHORE_CALLOUT: Record<string, string> = { descend: /* copy:callout */ "descend", pick_up: /* copy:callout */ "pick up" };

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
  let speed = 1, done = false, disposed = false, overridden = false, tickerTimer = 0, pumpTimer = 0;
  let snap: Snapshot | null = null;
  let runId = -1, engineTick = 0, startTick = 0, inflight = false, lastPersist = performance.now();
  let pendingLoad: { snap: Snapshot; rest: Ev[] } | null = null;
  let exitTier: Tier | null = null, exitAt = 0;
  let pendingExit: { items: InvItem[]; tier: string } | undefined;
  const cls = app.lineage.class;
  const before = { best: app.lineage.best_depth, marks: app.lineage.marks, level: app.lineage.classes?.[cls]?.level ?? 1, xp: app.lineage.classes?.[cls]?.xp ?? 0, renown: app.lineage.renown ?? 0, rank: app.lineage.rank ?? 0 };
  const learned: string[] = [], found: InvItem[] = [], notes: Highlight[] = [], tamed: string[] = [], lost: string[] = [];
  const kinds = new Map<number, string>(), names = new Map<number, string>();
  const partyAtStart = (app.lineage.party ?? []).map((c) => `${c.kind} · ${c.name}`);
  const tamedIds: number[] = [], lostIds: number[] = [];
  const compLabel = (id: number): string => { const n = names.get(id); return `${kinds.get(id) ?? "?"}${n ? ` · ${n}` : ""}`; };
  const note_ = (e: { id: number; kind: string; name?: string }): void => { kinds.set(e.id, e.kind); if (e.name) names.set(e.id, e.name); };
  // HUD updates released at the viewer's clock
  const hud = { hp: 0, maxHp: 1, depth: 1 };
  const timed: { t: number; f: () => void }[] = [];
  let lastRuleText = "", lastRuleAt = 0;
  // placeholder viewer (no clock): a wall clock at 10 ticks/s × speed stands in
  let fbTick = 0, fbAt = performance.now();
  function viewerTick(): number {
    if (viewer?.tick) return viewer.tick();
    const now = performance.now(); fbTick += ((now - fbAt) / 1000) * 10 * speed; fbAt = now;
    return Math.min(fbTick, engineTick);
  }
  const viewerIdle = (): boolean => viewer?.idle ? viewer.idle() : true;

  function paintHud(): void {
    const p = hud.maxHp ? hud.hp / hud.maxHp : 0;
    hpFill.style.width = `${Math.round(Math.max(0, p) * 100)}%`;
    hpFill.classList.toggle("low", p < 0.3);
    replace(hpText, `${Math.max(0, hud.hp)}/${hud.maxHp}`);
    replace(depth, `D${hud.depth}`);
    if (snap) replace(alert, "!".repeat(Math.max(0, Math.min(5, snap.alert))));
  }
  function hudFrom(s: Snapshot): void { hud.hp = s.hero.hp; hud.maxHp = s.hero.max_hp; hud.depth = s.depth; paintHud(); }
  function callout(text: string): void {
    replace(ticker, text); ticker.classList.add("show");
    clearTimeout(tickerTimer); tickerTimer = window.setTimeout(() => ticker.classList.remove("show"), 1800 / Math.max(1, speed));
  }
  function ruleCallout(ev: Extract<Ev, { k: "rule" }>): string | null {
    if (ev.row >= 0) return `R${ev.row + 1} · ${verbLabel(ev.verb)}`;
    if (ev.row === -1) return ev.text;                       // trait deviation, e.g. "cowardly → retreat"
    return CHORE_CALLOUT[ev.verb.v] ?? null;                  // chores are silent (pillar 2)
  }
  function at(t: number, f: () => void): void { timed.push({ t, f }); }
  function release(upTo: number): void {
    if (!timed.length) return;
    const keep: typeof timed = [];
    for (const x of timed) { if (x.t <= upTo) x.f(); else keep.push(x); }
    timed.length = 0; timed.push(...keep);
  }
  function absorb(evs: Ev[], s: Snapshot): Tier | null {
    let exit: Tier | null = null;
    const heroId = s.hero.id;
    for (const ev of evs) {
      switch (ev.k) {
        case "callout": at(ev.t, () => callout(ev.text)); break;
        case "rule": {
          const text = ruleCallout(ev);
          if (text) at(ev.t, () => { const now = performance.now(); if (text !== lastRuleText || now - lastRuleAt > 4000) callout(text); lastRuleText = text; lastRuleAt = now; });
          break;
        }
        case "hurt": if (ev.id === heroId) at(ev.t, () => { hud.hp = ev.hp; paintHud(); }); break;
        case "descend": at(ev.t, () => { hud.depth = ev.depth; paintHud(); }); break;
        case "fact": learned.push(ev.fact); break;
        case "pickup": if (ev.id === heroId) found.push({ id: ev.id, kind: ev.item, known: true, label: ev.item }); break;
        case "note": notes.push({ pattern: "note", score: 0, t: ev.t, run_id: runId, text: ev.text }); break;
        case "exit": exit = ev.tier; break;
        case "tame": if (ev.ok) { tamedIds.push(ev.id); kinds.set(ev.id, ev.kind); } break;
        case "ally": if (ev.state === "lost") lostIds.push(ev.id); break;
        case "spawn": note_(ev.e); break;
        case "level": for (const v of verbsAt(ev.class, ev.level)) learned.push(`verb:${v}`); at(ev.t, () => callout(`${ev.class} L${ev.level}`)); break;
        case "rank": at(ev.t, () => callout(`★${ev.rank}`)); break;
        default: break;
      }
    }
    return exit;
  }
  function handle(r: StepResult): void {
    const s = r.snapshot;
    engineTick = s.turn;
    for (const e of s.entities) note_(e);
    const exit = absorb(r.events, s);
    const di = r.events.findIndex((e) => e.k === "descend");
    if (viewer && di >= 0) { viewer.apply(r.events.slice(0, di + 1)); pendingLoad = { snap: s, rest: r.events.slice(di + 1) }; }
    else viewer?.apply(r.events);
    snap = s;
    hud.maxHp = s.hero.max_hp; paintHud();
    if (r.exit_pending) pendingExit = r.exit_pending;
    if (exit) { exitTier = exit; exitAt = performance.now() + EXIT_GRACE_MS; }
    if (performance.now() - lastPersist > PERSIST_MS) { lastPersist = performance.now(); app.persist(); }
  }
  function pump(): void {
    if (done || disposed || !viewer || !snap) return;
    const now = viewerTick();
    release(now);
    if (pendingLoad) {
      if (viewerIdle()) { const p = pendingLoad; pendingLoad = null; viewer.load(p.snap); hudFrom(p.snap); viewer.apply(p.rest); }
      return;
    }
    if (exitTier) { if (viewerIdle() || performance.now() > exitAt) { release(Infinity); void finish(exitTier); } return; }
    if (speed <= 0 || inflight || engineTick - now >= LEAD) return;
    inflight = true;
    app.engine.step(BATCH).then((r) => { inflight = false; if (!disposed && !done) handle(r); })
      .catch((e) => { inflight = false; console.warn("step failed", e); exitTier = "return"; exitAt = 0; });
  }
  function setSpeed(n: number): void {
    speed = n;
    for (const [b, v] of [[s1, 1], [s4, 4]] as const) b.classList.toggle("on", n === v);
    pause.classList.toggle("on", n === 0);
    replace(pause, n === 0 ? "▶" : "⏸");
    viewer?.setSpeed(n);
  }
  async function skipToEvent(): Promise<void> {
    if (done || inflight || !viewer || exitTier || pendingLoad) return;
    inflight = true;
    try {
      let hit = false;
      for (let i = 0; i < 30 && !hit && !disposed; i++) {
        const r = await app.engine.step(BATCH);
        hit = r.run_over || r.events.some((e) => INTERESTING.has(e.k));
        handle(r);
        if (pendingLoad) break;
      }
    } catch (e) { console.warn("skip failed", e); }
    inflight = false;
    if (!pendingLoad) { viewer.skipToEvent(); fbTick = engineTick; release(viewerTick()); }
  }
  function doBail(): void {
    if (done || overridden) return;
    overridden = true; bail.classList.add("on");
    void app.engine.setRules({ rows: [{ conds: [], verb: { v: "return" } }, ...app.rules.rows] }).catch((e) => console.warn("bail", e));
    if (speed === 0) setSpeed(1);
  }
  function xpGained(): number {
    const c = app.lineage.classes?.[cls] ?? { level: 1, xp: 0 }; let g = c.xp - before.xp;
    for (let l = before.level; l < c.level; l++) g += xpToNext(l);
    return Math.max(0, g);
  }
  async function finish(tier: Tier): Promise<void> {
    if (done) return;
    done = true; clearInterval(pumpTimer);
    if (overridden) await app.engine.setRules(app.rules).catch(() => { /* rules restored on next camp edit */ });
    if (pendingExit && pendingExit.items.length) { const p = pendingExit; pendingExit = undefined; exitSheet(p, () => { done = false; void finish(tier); }); return; }
    pendingExit = undefined;
    await app.refresh();
    app.runsSeen += 1;
    if (disposed) return;
    tamed.push(...tamedIds.map(compLabel));
    lost.push(...lostIds.map(compLabel));
    if (tier === "death") {
      for (const c of partyAtStart) if (!lost.some((l) => l === c || l.endsWith(c.slice(c.indexOf(" · "))))) lost.push(c);
      try {
        const death = await app.busy(/* copy:label */ "verdict", () => app.engine.death(runId));
        if (!disposed) app.go({ kind: "death", death, lost });
      } catch (e) { console.warn("no death record", e); app.go({ kind: "camp" }); }
      return;
    }
    const L = app.lineage;
    const bests: string[] = []; for (let d = before.best + 1; d <= L.best_depth; d++) bests.push(`D${d}`);
    const report: ReturnReport = {
      elapsed_s: Math.round((engineTick - startTick) / 10), runs: 1, sampled: false, learned, bests, found, deaths: [], pending: [],
      reel: notes.slice(-5), marks_earned: L.marks - before.marks, live: snap!, tamed, hatched: [], lost,
      xp: { class: cls, gained: xpGained(), level_ups: (L.classes?.[cls]?.level ?? 1) - before.level },
      salvaged: [], renown: { gained: (L.renown ?? 0) - before.renown, rank: L.rank ?? 0, ranks_up: (L.rank ?? 0) - before.rank },
    };
    app.go({ kind: "report", report });
  }

  // Addendum D: choose what to keep before the run settles
  function exitSheet(p: { items: InvItem[]; tier: string }, then: () => void): void {
    const free = Math.max(0, vaultSlots(app.lineage.unlocks) - app.lineage.vault.length);
    const keep = new Set<number>();
    let sent = false;
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
        h("button", { class: "btn primary wide", onclick: () => {
          if (sent) return; sent = true;
          app.engine.keep([...keep]).then((L) => { app.lineage = L; }).catch((e) => console.warn("keep", e)).finally(() => { close(); then(); });
        } }, /* copy:button */ "keep"));
    });
  }

  async function init(): Promise<void> {
    let s: Snapshot;
    try { s = await app.engine.send(); } catch (e) { console.warn("send failed", e); if (!disposed) app.go({ kind: "camp" }); return; }
    if (disposed) return;
    snap = s; runId = s.run.id; engineTick = startTick = s.turn;
    for (const e of s.entities) note_(e);
    hudFrom(s);
    const { viewer: v } = await makeViewer(canvas);
    if (disposed) { v.dispose(); return; }
    viewer = v; v.resize?.(); v.load(s); v.setSpeed(speed); fbTick = s.turn; fbAt = performance.now();
    pumpTimer = window.setInterval(pump, PUMP_MS);
  }
  void init();
  const onResize = (): void => viewer?.resize?.();
  window.addEventListener("resize", onResize);
  return { el, dispose: () => {
    disposed = true; window.removeEventListener("resize", onResize); clearInterval(pumpTimer); clearTimeout(tickerTimer); viewer?.dispose();
    if (overridden && !done) void app.engine.setRules(app.rules);
  } };
}
