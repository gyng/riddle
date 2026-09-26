// Cut 27 §2 — the edit is a scene. After an edit's refine lands, the camp asks the core where the edited set first acts differently
// from the set sent (`divergence(prev)`: a paired panel seed, the tick, both branches' next seconds and each whole run's end) and plays
// it over the foot of the well on the renderer, before then after: the sent branch (`sent · R2`) ending on its run's end (`dies · D7`),
// then the edited one (`R5 now`) ending on its own (`lives · D9`) — 3–5 s in all; a tap lets it go. What stays is one line under the
// `vs sent` line: `R5 now → lives · D9 · vs dies · D7` (a tap plays it again). A move inside its ± plays too when the sets act
// differently somewhere, and its line says what changed (`≈ ±6 · R3 fires 4× more`). Under `prefers-reduced-motion` the scene is the
// two branches' end frames side by side, still. Shown for a move ≥ SCENE_MOVE outside its ±, or any `≈` move with a divergence.
import type { App } from "../app";
import type { Divergence, DivergenceBranch, DivergenceEnd, RowFires, RuleSet } from "../engine/types";
import { h, replace } from "./dom";
import { makeViewer, type Viewer } from "./viewer";

const SCENE_MOVE = 0.05;
const PHASE_MS = 1500, END_MS = 500, LINGER_MS = 600;   // each branch plays PHASE_MS, its end holds END_MS; the scene lingers, then folds to its line
type SceneViewer = Viewer & { seek?(t: number): void; setFrame?(frame: "map" | "fight", focus?: { x: number; y: number; radius: number }): void };
/** The fight frame on the hero where the branch starts (a few tiles around him: the moment, not the floor). */
const FOCUS_R = 5;

/** `lives · D9` · `dies · D7` · `stalls · D6` — how a branch's whole run ended, in the player's words. */
export function endText(e: DivergenceEnd): string {
  const w = e.tier === "death" ? /* copy:callout */ "dies" : e.tier === "stall" ? /* copy:callout */ "stalls" : /* copy:callout */ "lives";
  // Cut 28 §1: with an oath sworn, whether the branch's whole run kept it
  return `${w} · D${e.depth}${e.oath === undefined ? "" : e.oath ? /* copy:callout */ " · oath ✓" : /* copy:callout */ " · oath ✗"}`;
}
/** The row a branch fired at the divergence (`R5`), or `—` when none did (a chore, a step). */
const rowTag = (row: number | undefined): string => (row === undefined ? "—" : `R${row + 1}`);
/** `R3 fires 4× more` / `R3 fires 2× less` / `R3 fires new` — the row whose fires moved most (the `≈` edit's what-changed). */
export function firesText(f: RowFires | undefined): string | null {
  if (!f) return null;
  const r = f.new_row ?? f.sent_row; if (r === undefined) return null;
  const tag = `R${r + 1}`;
  if (f.new_row === undefined) return /* copy:callout */ `${tag} gone`;
  if (f.sent <= 0.05 && f.new > 0) return /* copy:callout */ `${tag} fires new`;
  if (f.new <= 0.05 && f.sent > 0) return /* copy:callout */ `${tag} never fires`;
  const k = f.new / Math.max(1e-6, f.sent);
  if (k >= 1.5) return /* copy:callout */ `${tag} fires ${Math.round(k)}× more`;
  if (k <= 1 / 1.5) return /* copy:callout */ `${tag} fires ${Math.round(1 / k)}× less`;
  return null;
}
const reducedMotion = (): boolean => { try { return matchMedia("(prefers-reduced-motion: reduce)").matches; } catch { return false; } };

export function renderScene(app: App): { el: HTMLElement; line: HTMLElement; dispose(): void } {
  const view = h("canvas", { class: "div-view" });
  const tag = h("div", { class: "div-tag num" });
  const end = h("div", { class: "div-end num" });
  // (the scene takes no taps: a tap anywhere lets it go and still reaches what lies under it — the next edit is never a tap away)
  const el = h("div", { class: "div-scene", hidden: true }, view, tag, end);
  const onTap = (): void => skip();
  window.addEventListener("pointerdown", onTap, { capture: true });
  const line = h("button", { class: "div-line num", hidden: true, onclick: () => { if (cur) void play(cur, true); } });
  let cur: Divergence | null = null, viewer: SceneViewer | null = null, viewerP: Promise<SceneViewer | null> | null = null;
  let timers: number[] = [], seq = 0, gone = false, asked = "", stills: SceneViewer[] = [], rulesGen = 0;
  const later = (ms: number, f: () => void): void => { timers.push(window.setTimeout(f, ms)); };
  const clearTimers = (): void => { for (const t of timers) clearTimeout(t); timers = []; };
  const dev = (k: string, v: unknown): void => { if ("__riddle" in window) ((window as unknown as { __scene?: Record<string, unknown> }).__scene ??= {})[k] = v; };
  /** The viewer, made once the scene may play (a WebGL context only while the camp has one to show). */
  const ensureViewer = (): Promise<SceneViewer | null> => viewerP ??= makeViewer(view).then(({ viewer: v }) => { if (gone) { v.dispose(); return null; } viewer = v as SceneViewer; return viewer; }).catch(() => null);

  function stop(): void {
    clearTimers(); seq++; viewer?.setSpeed(0);
    el.hidden = true; el.dataset.state = ""; delete el.dataset.phase;
    for (const s of stills) s.dispose(); stills = [];
    el.querySelector(".div-stills")?.remove();
  }
  /** A tap on the scene: it goes, the line stays. */
  function skip(): void { if (el.hidden) return; dev("skipped", performance.now()); stop(); }
  /** One branch on the viewer: its seconds from the snapshot, at the rate that plays them in PHASE_MS; `done` at its last tick. */
  function branch(v: SceneViewer, b: DivergenceBranch): number {
    v.load(b.snapshot); v.apply(b.events.filter((e) => e.k !== "descend" && e.k !== "exit"));
    v.setFrame?.("fight", { x: b.snapshot.hero.x, y: b.snapshot.hero.y, radius: FOCUS_R });
    const t0 = Math.min(b.snapshot.turn, b.events[0]?.t ?? b.snapshot.turn), t1 = b.events.length ? b.events[b.events.length - 1].t : t0;
    v.seek?.(t0);
    const rate = Math.max(1, Math.min(16, ((t1 - t0) / 10) / (PHASE_MS / 1000)));
    v.setSpeed(rate);
    return rate;
  }
  async function play(d: Divergence, replayed = false): Promise<void> {
    stop();
    const my = ++seq;
    el.hidden = false; el.dataset.state = "loading"; replace(tag, ""); replace(end, ""); end.classList.remove("show");
    if (reducedMotion()) { await stillsOf(d, my); return; }
    const v = await ensureViewer();
    if (my !== seq || gone) return;
    if (!v) { stop(); return; }
    v.resize?.();
    // before: the sent set's branch, ending on its run's end
    el.dataset.state = "playing"; el.dataset.phase = "sent";
    dev("playAt", performance.now()); dev("replayed", replayed);
    replace(tag, h("span", { class: "was" }, /* copy:callout */ `sent · ${rowTag(d.sent_row)}`));
    branch(v, d.sent);
    dev("ends", []);
    later(PHASE_MS, () => { if (my !== seq) return; replace(end, endText(d.sent_end)); ((window as unknown as { __scene?: { ends?: string[] } }).__scene?.ends)?.push(endText(d.sent_end)); end.className = `div-end num show ${d.sent_end.tier === "death" ? "down" : "up"}`; });
    // after: the edited set's branch, ending on its own
    later(PHASE_MS + END_MS, () => {
      if (my !== seq || !viewer) return;
      el.dataset.phase = "new"; end.classList.remove("show");
      replace(tag, h("span", { class: "now" }, /* copy:callout */ `${rowTag(d.new_row)} now`));
      branch(viewer, d.new);
    });
    later(2 * PHASE_MS + END_MS, () => { if (my !== seq) return; replace(end, endText(d.new_end)); ((window as unknown as { __scene?: { ends?: string[] } }).__scene?.ends)?.push(endText(d.new_end)); end.className = `div-end num show ${d.new_end.tier === "death" ? "down" : "up"}`; });
    later(2 * (PHASE_MS + END_MS) + LINGER_MS, () => { if (my !== seq) return; dev("doneAt", performance.now()); stop(); });
  }
  /** Reduced motion: the two branches' end frames side by side, each with its run's end, still. */
  async function stillsOf(d: Divergence, my: number): Promise<void> {
    const box = h("div", { class: "div-stills" });
    const pane = (e: DivergenceEnd, label: string): { c: HTMLCanvasElement; el: HTMLElement } => {
      const c = h("canvas", { class: "div-still" });
      return { c, el: h("div", { class: "div-pane" }, c, h("div", { class: `div-cap num ${e.tier === "death" ? "down" : "up"}` }, label, " → ", endText(e))) };
    };
    const a = pane(d.sent_end, /* copy:callout */ `sent · ${rowTag(d.sent_row)}`), b = pane(d.new_end, /* copy:callout */ `${rowTag(d.new_row)} now`);
    box.append(a.el, b.el); el.append(box);
    el.dataset.state = "still"; dev("playAt", performance.now());
    for (const [p, br] of [[a, d.sent], [b, d.new]] as const) {
      const { viewer: v } = await makeViewer(p.c);
      if (my !== seq || gone) { v.dispose(); return; }
      const sv = v as SceneViewer; stills.push(sv); sv.resize?.();
      sv.load(br.end_snapshot); sv.setFrame?.("fight", { x: br.end_snapshot.hero.x, y: br.end_snapshot.hero.y, radius: FOCUS_R }); sv.setSpeed(0);
    }
  }
  /** The line under `vs sent`: `R5 now → lives · D9 · vs dies · D7`; inside the ±, `≈ ±6 · R3 fires 4× more` before it. */
  function paintLine(d: Divergence): void {
    const vs = app.vsShown(), pm = vs ? Math.max(0, ...vs.depths.map((x) => x.pm ?? 0)) : 0;
    const fires = firesText(d.fires?.[0]);
    line.hidden = false;
    replace(line,
      d.inside ? h("span", { class: "div-flat" }, `≈${pm ? ` ±${Math.max(1, Math.round(pm * 100))}` : ""}${fires ? ` · ${fires}` : ""}`, h("i", { class: "sep" }, " · ")) : "",
      h("b", { class: "now" }, /* copy:callout */ `${rowTag(d.new_row)} now`), " → ",
      h("span", { class: d.new_end.tier === "death" ? "down" : "up" }, endText(d.new_end)),
      h("i", { class: "sep" }, " · "), /* copy:callout */ "vs ", h("span", { class: d.sent_end.tier === "death" ? "down" : "up" }, endText(d.sent_end)));
    line.dataset.inside = d.inside ? "1" : "0";
  }
  /** After an edit's refine: ask where the edited set first acts differently from the set sent, then play it (the lane is the refine's). */
  async function ask(refinedAt: number, gen: number): Promise<void> {
    // (a refine that landed just before an edit is the old set's: its ask would measure the new rules on the main thread's tick after
    // the edit — the fake's divergence is synchronous, ~1 s, and on wasm it queued ahead of the edit's own refine on that lane)
    if (gen !== rulesGen) return;
    const f = app.lastForecast, prev: RuleSet | null = app.sentSet();
    if (!f || f.refined !== true || !prev || !app.engine.divergence) return;
    // Cut 28 §2 (AV: the scene said `R2 now → dies` when the pets had died): the scene is a row edit's — a route or any state change
    // is not a row that fires differently (the state's part is its own line, `party −2 jackals · death +24`)
    if (JSON.stringify(prev.rows.map((r) => [r.conds, r.verb])) === JSON.stringify(app.rules.rows.map((r) => [r.conds, r.verb]))) return;
    const key = JSON.stringify([prev.rows, app.rules.rows, (app.rules as RuleSet).route ?? []]);
    if (key === asked) return;
    asked = key;
    dev("askAt", performance.now()); dev("refineAt", refinedAt);
    void ensureViewer();   // the context is made while the core looks (the scene's first frame is then ≤ the answer's)
    let d: Divergence | null = null;
    try { d = await app.engine.divergence(prev); } catch (e) { console.warn("divergence", e); return; }
    if (gone || key !== asked) return;
    dev("answerAt", performance.now()); dev("last", d);
    cur = d;
    el.dataset.found = d ? "1" : "0";
    if (!d) { line.hidden = true; return; }
    if (!d.inside && d.moved < SCENE_MOVE) { line.hidden = true; return; }   // a small move outside its ± is the number's to say
    paintLine(d);
    void play(d);
  }
  // (a tick later: the refine's own `vs` is asked first on the same lane — the divergence then reads the panels it paired, cached)
  const offFc = app.onForecast((f) => { if (f.refined === true) { const at = performance.now(), gen = rulesGen; setTimeout(() => void ask(at, gen), 0); } });
  const offRules = app.onRules(() => { rulesGen++; stop(); cur = null; line.hidden = true; asked = ""; });
  return { el, line, dispose: () => { gone = true; window.removeEventListener("pointerdown", onTap, { capture: true }); stop(); offFc(); offRules(); viewer?.dispose(); viewer = null; } };
}
