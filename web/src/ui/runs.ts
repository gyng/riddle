import { goldWords } from "./gold-words";
// RUNS_UI (docs/RUNS_UI.md §3) — the runs log: every run the core keeps (`Lineage.runs`, the last 60), newest first, folded by absence
// (`away · 17 runs` is one line until opened; the runs played while the app was open fold as `here`). An entry: the run's number, how
// it ended (the exit's reason, ≤ 3 words) and when, its floor with a `★` for a new best, the gold home, the finds, its length, and `▶`
// — a replay re-simulated by the core from the run's send (`replay(id)`, same rolls), when the core still holds it. A death opens its
// verdict. The chronicle stays the heirs' book (one line an heir); this is the runs'.
import type { App } from "../app";
import type { ExitLine, Lineage, Replay, RunRec } from "../engine/types";
import { h, replace } from "./dom";
import { closeAllSheets, openSheet } from "./sheet";
import { makeViewer, type Viewer } from "./viewer";
import { markRunsSeen, runsSeen } from "./runlane";
import { kwHost } from "./tips";
import { keptDeath } from "./chronicle";
import { clearCard } from "./runclear";   // run-clear: an entry's card
import { bestRarity } from "./items";

/** one fold: an absence's runs, or the runs played while the app was open between two absences */
export type RunGroup = { key: string; via: "away" | "here"; absence?: number; recs: RunRec[]; sampled: number };

/** The log's folds, newest first; each fold's runs newest first. */
export function groupsOf(runs: RunRec[]): RunGroup[] {
  const out: RunGroup[] = [];
  for (const r of runs) {
    const away = r.via === "away";
    const last = out[out.length - 1];
    const same = last && (away ? last.via === "away" && last.absence === r.absence : last.via === "here");
    const g = same ? last : (out.push({ key: away ? `a${r.absence ?? 0}` : `h${r.id}`, via: away ? "away" : "here", absence: r.absence, recs: [], sampled: 0 }), out[out.length - 1]);
    if (r.sampled) g.sampled += r.sampled; else g.recs.push(r);
  }
  for (const g of out) g.recs.reverse();
  return out.reverse();
}

/** `12m` · `3h` · `2d` — how long ago, on the lineage's clock */
export function agoOf(L: Lineage, clock: number): string {
  const s = Math.max(0, (L.clock_s ?? clock) - clock);
  return s < 60 ? /* copy:label */ "now" : s < 3600 ? `${Math.round(s / 60)}m` : s < 86400 ? `${Math.round(s / 3600)}h` : `${Math.round(s / 86400)}d`;
}
/** a run's length (10 ticks a second) */
const lenOf = (turns: number): string => { const s = turns / 10; return s < 60 ? `${Math.round(s)}s` : `${Math.round(s / 60)}m`; };
const TIER_GLYPH: Record<string, string> = { bank: "⌂", return: "↩", death: "☠" };

/** The log sheet: `runs` (the folds) and `heirs` (the chronicle: one line an heir, a kept death's line opens its verdict) — the history's
 *  one place (the chronicle's console tile folded in here). `focus` opens the fold holding that run. */
export function openRuns(app: App, opts: { focus?: number; tab?: "runs" | "heirs" } = {}): void {
  closeAllSheets();
  openSheet((close) => {
    const L = app.lineage;
    let tab = opts.tab ?? ((L.runs ?? []).some((r) => r.id > 0) ? "runs" : "heirs");
    const seen = runsSeen(L);
    markRunsSeen(L);
    const groups = groupsOf(L.runs ?? []);
    const held = new Set(L.replays ?? []);
    const open = new Set<string>(groups.length ? [groups[0].key] : []);
    if (opts.focus !== undefined) { const g = groups.find((x) => x.recs.some((r) => r.id === opts.focus)); if (g) open.add(g.key); }
    const list = h("div", { class: "runs-list" });
    const count = h("span", { class: "num dim runs-count" });
    const tabRuns = h("button", { class: "log-tab", "data-tab": "runs", onclick: () => { tab = "runs"; paint(); } }, /* copy:button */ "runs", count);
    const tabHeirs = h("button", { class: "log-tab", "data-tab": "heirs", onclick: () => { tab = "heirs"; paint(); } }, /* copy:button */ "heirs");
    const paint = (): void => {
      tabRuns.classList.toggle("on", tab === "runs"); tabHeirs.classList.toggle("on", tab === "heirs");
      tabRuns.setAttribute("aria-pressed", String(tab === "runs")); tabHeirs.setAttribute("aria-pressed", String(tab === "heirs"));
      body.dataset.tab = tab;
      if (tab === "heirs") { replace(list, ...heirLines(app, close)); return; }
      replace(count, `${(L.runs ?? []).filter((r) => r.id > 0).length}`);
      if (!groups.length) { replace(list, h("div", { class: "dim num runs-none" }, "—")); return; }
      replace(list, ...groups.flatMap((g) => {
        const on = open.has(g.key);
        const n = g.recs.length + g.sampled;
        const best = Math.max(0, ...g.recs.map((r) => r.depth)), gold = g.recs.reduce((a, r) => a + r.gold, 0);
        const newBest = g.recs.some((r) => r.best);
        const end = g.recs[0];
        const head = h("button", { class: `runs-fold${on ? " on" : ""}`, "data-fold": g.key, "data-via": g.via, "data-n": n, "aria-expanded": on ? "true" : "false", onclick: () => { if (on) open.delete(g.key); else open.add(g.key); paint(); } },
          h("span", { class: "rf-car", "aria-hidden": "true" }, on ? "▾" : "▸"),
          kwHost(h("b", { class: "rf-w" }, g.via === "away" ? /* copy:label */ "away" : /* copy:label */ "here"), g.via === "away" ? "away" : "live"),
          h("span", { class: "num rf-n" }, /* copy:label */ ` · ${n} runs`),
          h("span", { class: `num rf-d${newBest ? " best" : ""}` }, ` · D${best}${newBest ? "★" : ""}`),
          h("span", { class: "num gold rf-g" }, ` · $${gold}`),
          end ? h("small", { class: "num dim rf-t" }, ` ${agoOf(L, end.clock_s)}`) : "");
        if (!on) return [head];
        return [head, ...g.recs.map((r) => entry(app, r, held.has(r.id), r.id > seen, close)),
          g.sampled ? h("div", { class: "run-entry sampled num dim", "data-sampled": g.sampled }, /* copy:label */ `+${g.sampled} sampled`) : ""];
      }));
    };
    const body = h("div", { class: "sheet-body runs-sheet" },
      h("div", { class: "label row-label log-head" }, kwHost(tabRuns, "log"), tabHeirs), list);
    paint();
    return body;
  });
}

/** the chronicle's lines, newest first (Cut 5 §2; Cut 9 §7: a line whose heir keeps a death opens its verdict) */
function heirLines(app: App, close: () => void): HTMLElement[] {
  const L = app.lineage;
  const lines = [...(L.chronicle ?? [])].reverse();
  return [h("div", { class: "chronicle" }, ...lines.map((line) => {
    const id = keptDeath(L, line);
    if (id === undefined) return h("div", { class: "cline" }, line);
    return h("button", { class: "cline kept", onclick: () => {
      void app.busy(/* copy:label */ "verdict", () => app.engine.death(id)).then((death) => { close(); closeAllSheets(); app.go({ kind: "death", death, kept: true }); })
        .catch((e) => console.warn("kept death", e));
    } }, line, h("small", { class: "dim" }, " ▸"));
  }))];
}

/** one run's entry: `#41 · D9★ · $212 · ✦2 · 4m` over `hurt · banked · 12m`, `▶` at its end; a death opens its verdict */
function entry(app: App, r: RunRec, replay: boolean, fresh: boolean, close: () => void): HTMLElement {
  const L = app.lineage;
  const why = r.reason ?? (r.tier === "death" ? /* copy:label */ "died" : r.tier === "bank" ? /* copy:label */ "banked" : /* copy:label */ "returned");
  const top = h("span", { class: "re-top num" },
    h("b", { class: "re-id" }, `#${r.id}`),
    h("span", { class: `re-tier t-${r.tier}`, "aria-hidden": "true" }, TIER_GLYPH[r.tier] ?? ""),
    h("span", { class: `re-d${r.best ? " best" : ""}` }, `D${r.depth}`, r.best ? h("b", { class: "re-best", title: "new best" }, "★") : ""),
    h("span", { class: "gold re-g" }, `$${r.gold}`),
    (r.finds ?? []).length ? h("span", { class: "re-f", "data-rarity": bestRarity(r.finds ?? []), "aria-label": /* copy:label */ "finds" }, ...(r.finds ?? []).slice(0, 4).map((f) => h("span", { class: `re-gem r-${f.rarity ?? "common"}` }, "◆"))) : "",
    h("span", { class: "dim re-len" }, lenOf(r.turns)),
    r.via === "watched" ? h("span", { class: "dim re-w", "aria-hidden": "true" }, "◉") : "");
  // (Cut 30.5: the checkpoints' gold, kept whole whatever the end — `$80 secured`)
  const sub = h("small", { class: "re-sub dim" }, goldWords(why), (r.secured ?? 0) > 0 ? h("span", { class: "num re-sec" }, /* copy:label */ ` · $${r.secured} secured`) : "", " · ", h("span", { class: "num" }, agoOf(L, r.clock_s)));
  const verdict = r.tier === "death" && r.death_id !== undefined && (L.graveyard ?? []).some((g) => g.death_id === r.death_id);
  const body = verdict
    ? h("button", { class: "re-body", "data-verdict": r.death_id, onclick: () => {
        void app.busy(/* copy:label */ "verdict", () => app.engine.death(r.death_id!)).then((death) => { close(); closeAllSheets(); app.go({ kind: "death", death, kept: true }); }).catch((e) => console.warn("run verdict", e));
      } }, top, sub)
    : h("button", { class: "re-body", "data-card": r.id, onclick: () => openRunCard(app, r) }, top, sub);   // run-clear: the run's card
  const play = replay ? kwHost(h("button", { class: "re-play", "data-run": r.id, "aria-label": /* copy:label */ "replay", onclick: () => void openRunReplay(app, r) }, "▶"), "replay") : "";
  return h("div", { class: `run-entry${fresh ? " fresh" : ""}`, "data-run": r.id, "data-tier": r.tier, "data-via": r.via }, body, play);
}

/** run-clear × RUNS_UI: a past run's card (the end's seal, its reason, the floor and a new best, the gold home, the finds by rarity) — the
 *  watched run's clear screen, as the log keeps it; its ▶ under it when the core holds the run. */
function openRunCard(app: App, r: RunRec): void {
  const x: ExitLine = { carried: r.gold, keep_pct: 100, kept: r.gold, spent: 0, spent_on: [], text: `${r.tier === "bank" ? "banked" : r.tier === "death" ? "died" : "returned"} D${r.depth}`,
    end: r.tier, reached: r.depth, new_best: !!r.best, finds: r.finds ?? [], ...(r.reason ? { reason: r.reason } : {}), ...(r.secured ? { secured: r.secured } : {}), run_id: r.id };
  const held = (app.lineage.replays ?? []).includes(r.id);
  openSheet(() => h("div", { class: "sheet-body run-card-sheet", "data-run": r.id },
    h("div", { class: "label row-label" }, h("b", { class: "num" }, `#${r.id}`), " ", h("span", { class: "num dim" }, agoOf(app.lineage, r.clock_s))),
    clearCard(app, x),
    held ? h("button", { class: "btn wide re-play-wide", "data-run": r.id, onclick: () => void openRunReplay(app, r) }, "▶ ", /* copy:button */ "replay") : ""));
}

/** RUNS_UI §3: a past run played again — the core re-simulates it from its send (`replay(id)`); the floors one after another in the map
 *  frame, the floor chips to jump, a tap on the picture goes on. `window.__runReplay` carries the run, its hash and where it is (tests). */
const REPLAY_RATE = 16, POLL_MS = 200;
type SeekViewer = Viewer & { seek?(t: number): void; setFrame?(frame: "map" | "fight"): void };
export async function openRunReplay(app: App, r: RunRec): Promise<void> {
  if (!app.engine.replay) return;
  let rep: Replay | null = null;
  try { rep = await app.busy(/* copy:label */ "replay", () => app.engine.replay!(r.id)); } catch (e) { console.warn("replay", e); }
  if (!rep || !rep.floors.length) return;
  const R = rep;
  const dev = window as unknown as { __runReplay?: { run: number; hash: string; floors: number; floor: number; done: boolean; events: number } };
  dev.__runReplay = { run: R.run_id, hash: R.hash, floors: R.floors.length, floor: -1, done: false, events: R.floors.reduce((a, f) => a + f.events.length, 0) };
  openSheet(() => {
    const canvas = h("canvas", { class: "replay-view run-replay-view" });
    const at = h("span", { class: "num dim rr-at" });
    const chips = h("div", { class: "chips rr-floors" });
    const body = h("div", { class: "sheet-body replay run-replay", "data-run": R.run_id, "data-hash": R.hash },
      h("div", { class: "label row-label" }, kwHost(h("span", null, /* copy:label */ "replay"), "replay"), " ", h("b", { class: "num" }, `#${R.run_id}`), " ", at), canvas, chips);
    let viewer: SeekViewer | null = null, timer = 0, k = -1, end = 0;
    const stop = (): void => { clearInterval(timer); viewer?.dispose(); viewer = null; };
    const show = (i: number): void => {
      if (!viewer) return;
      k = i;
      if (k >= R.floors.length) { viewer.setSpeed(0); body.dataset.done = "1"; if (dev.__runReplay) dev.__runReplay.done = true; return; }
      const f = R.floors[k];
      viewer.load(f.snapshot); viewer.apply(f.events.filter((e) => e.k !== "descend" && e.k !== "exit"));
      viewer.setFrame?.("map");
      const t0 = Math.min(f.snapshot.turn, f.events[0]?.t ?? f.snapshot.turn); end = f.events.length ? f.events[f.events.length - 1].t : f.snapshot.turn;
      viewer.seek?.(t0); viewer.setSpeed(REPLAY_RATE);
      at.textContent = `D${f.snapshot.depth}`;
      body.dataset.floor = String(f.snapshot.depth);
      if (dev.__runReplay) dev.__runReplay.floor = k;
      for (const c of chips.children) c.classList.toggle("on", (c as HTMLElement).dataset.k === String(k));
    };
    replace(chips, ...R.floors.map((f, i) => h("button", { class: "chip mini num", "data-k": i, onclick: () => show(i) }, `D${f.snapshot.depth}`)));
    requestAnimationFrame(() => {
      if (!document.contains(canvas)) return;
      void makeViewer(canvas).then(({ viewer: v }) => {
        if (!document.contains(canvas)) { v.dispose(); return; }
        viewer = v; v.resize?.(); show(0);
        timer = window.setInterval(() => {
          if (!document.contains(canvas)) { stop(); return; }
          if (k >= R.floors.length) return;
          if (!viewer?.tick || viewer.tick() >= end) show(k + 1);   // (the placeholder has no clock: each floor as it ends)
        }, POLL_MS);
      });
    });
    canvas.onclick = () => show(Math.min(k + 1, R.floors.length));
    return body;
  }, { stay: true });   // (a replay plays on with no input: no auto-close — RUNS_UI, docs/UI.md §7's exception like the watch)
}
