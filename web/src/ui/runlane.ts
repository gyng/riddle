// RUNS_UI (docs/RUNS_UI.md) — the run lanes: one row per hero, under the town, saying at a glance what he is doing right now — down
// there (`live D3`, his hp, a quick heartbeat), resting (`rests 18m`, a slow beat, the drain of the rest), or home waiting for a send
// (before the scout: `waits ▸ send`, the gem lit). The `auto` slot says the climb goes on by itself (lit once the scout is hired; before,
// greyed with his count). Built from an array (`lanesOf`), so Cut 31's second hero and the expeditions add rows, not screens: ≤ 3 rows,
// then a `+N` count. A live lane opens the watch on the run in flight (the only lane that taps). The log's stud sits at the end with
// the runs not yet looked at.
import "../runs.css";
import type { App } from "../app";
import type { HeroLane, Lineage } from "../engine/types";
import { h, replace, spanOf } from "./dom";
import { heirOrd } from "./tokens";
import { kwHost } from "./tips";
import { paintFace } from "./frame";

export const LANES_SHOWN = 3;
const SEEN_KEY = (L: Lineage): string => /* copy:none */ `riddle.runs.seen.${L.seed ?? 0}`;
/** the newest run id the log showed this viewer (a per-viewer convenience) */
export function runsSeen(L: Lineage): number { try { return Number(localStorage.getItem(SEEN_KEY(L)) ?? 0) || 0; } catch { return 0; } }
export function markRunsSeen(L: Lineage): void { const last = lastRunId(L); try { localStorage.setItem(SEEN_KEY(L), String(last)); } catch { /* private mode */ } }
const lastRunId = (L: Lineage): number => Math.max(0, ...(L.runs ?? []).map((r) => r.id));
/** runs the log has not shown yet (sampled records count their runs) */
export const unseenRuns = (L: Lineage): number => { const s = runsSeen(L); return (L.runs ?? []).filter((r) => r.id > s).length; };

/** The lanes on the wire (Cut 31: `Lineage.heroes`), else the one hero's, read off the run under way, the rest and the works tree. */
export function lanesOf(L: Lineage, restLeftS: number): HeroLane[] {
  if (L.heroes?.length) return L.heroes;
  const W = L.tree, scout = W?.nodes.find((n) => n.id === "scout");
  const live = L.live && L.live.turn > 0 ? L.live : null;
  const auto = W ? W.auto_send : true;   // an old save: the scout is pre-hired
  const state: HeroLane["state"] = live ? "live" : W?.waits ? "waits" : "rests";
  return [{ id: "heir", name: heirOrd(L.heir), state, depth: live?.depth, hp: live?.hp, max_hp: live?.max_hp, run_id: live?.run_id,
    rest_s: state === "rests" ? Math.max(0, restLeftS) : undefined, auto, need: scout?.need, have: scout?.count, kind: "hero" }];
}

export type LanesUi = { el: HTMLElement; paint(): void; dispose(): void };
export type LaneHooks = { watch(lane: HeroLane): void; log(): void };
/** The lane block: the rows, the `+N`, the log's stud. Paints itself every second (the rest's countdown, the beat). */
export function renderLanes(app: App, hooks: LaneHooks): LanesUi {
  const rows = h("div", { class: "lane-rows" });
  const log = h("button", { class: "lanes-log", "data-tile": "log", "aria-label": /* copy:label */ "runs log", onclick: () => hooks.log() });
  kwHost(log, "log");
  const el = h("div", { class: "lanes", role: "group", "aria-label": /* copy:label */ "runs" }, rows, log);
  let last = "", lastLog = "";
  function paint(): void {
    const L = app.lineage;
    const lanes = lanesOf(L, app.restLeftS());
    const shown = lanes.slice(0, LANES_SHOWN), more = lanes.length - shown.length;
    const runs = (L.runs ?? []).filter((r) => r.id > 0).length;
    const unseen = unseenRuns(L);
    // (the rows rebuild only when what they say changes: a press on a row is never lost to a repaint)
    const key = JSON.stringify([shown.map((x) => [x.id, x.state, x.depth, x.hp, x.max_hp, x.rest_s === undefined ? "" : spanOf(x.rest_s), x.auto, x.have, x.need, x.name, x.kind]), more, runs, unseen, L.class, L.look]);
    if (el.dataset.n !== String(lanes.length)) el.dataset.n = String(lanes.length);
    const states = lanes.map((x) => x.state).join(" ");
    if (el.dataset.states !== states) el.dataset.states = states;
    const hide = runs === 0 && !(L.chronicle?.length) && L.heir < 5;   // day 0: nothing to read (the town keeps its four surfaces); the heirs' book from the 5th heir
    // (the DOM is written only when what it says changes: a repaint a second must not move what a finger or a test is about to press)
    const logKey = `${hide}|${unseen}`;
    if (logKey !== lastLog) {
      lastLog = logKey; log.hidden = hide;
      replace(log, h("span", { class: "ll-ico", "aria-hidden": "true" }, "☰"), h("span", { class: "ll-l" }, /* copy:button */ "log"),
        unseen > 0 ? h("b", { class: "ll-n num", "data-n": unseen }, String(Math.min(unseen, 99))) : "");
    }
    if (key === last) return;
    last = key;
    replace(rows, ...shown.map((x, i) => laneRow(app, x, i, hooks, runs > 0)), more > 0 ? moreRow(lanes.slice(LANES_SHOWN)) : "");
  }
  const timer = window.setInterval(paint, 1000);
  const off = app.onChange(paint), offLive = app.onLive(paint);
  paint();
  return { el, paint, dispose: () => { clearInterval(timer); off(); offLive(); } };
}

/** one hero's row: the face, the name, the state with its beat, hp or the rest's drain, the `auto` slot */
function laneRow(app: App, x: HeroLane, i: number, hooks: LaneHooks, hasRuns: boolean): HTMLElement {
  const face = h("span", { class: "lane-face", "aria-hidden": "true" });
  if (i === 0 && x.kind !== "expedition") paintFace(face, app.lineage.class, 28, app.lineage.look ?? "");
  else face.textContent = x.kind === "expedition" ? "⚑" : "♟";
  const beat = h("i", { class: "lane-beat", "aria-hidden": "true" });
  const state = x.state === "live"
    ? h("span", { class: "lane-state" }, beat, h("b", { class: "ls-w" }, /* copy:label */ "live"), " ", h("span", { class: "num ls-d" }, `D${x.depth ?? 1}`))
    : x.state === "rests"
      // (the blind read: "is 13m time left or spent?" — `13m left`)
      ? (x.rest_s ?? 0) >= 1 ? h("span", { class: "lane-state" }, beat, h("b", { class: "ls-w" }, /* copy:label */ "departs"), " ", h("span", { class: "num ls-t" }, spanOf(x.rest_s ?? 0)))
        : h("span", { class: "lane-state" }, beat, h("b", { class: "ls-w" }, /* copy:label */ "goes down"))   // (the rest out: the next run is due)
      : h("span", { class: "lane-state" }, beat, h("b", { class: "ls-w" }, /* copy:label */ "waits"), " ", h("span", { class: "ls-go" }, /* copy:label */ "▸ send"));
  // the gauge: hp while down there, the rest draining at home (full = just back), nothing while he waits
  const pct = x.state === "live" && x.max_hp ? Math.max(0, Math.min(1, (x.hp ?? 0) / x.max_hp)) : x.state === "rests" ? restShare(x.rest_s ?? 0) : 0;
  const gauge = x.state === "waits" ? "" : h("span", { class: `lane-gauge ${x.state === "live" ? "g-hp" : "g-rest"}`, "data-pct": Math.round(pct * 100) },
    h("span", { class: "fill", style: `width:${Math.round(pct * 100)}%` }), x.state === "live" && x.max_hp ? h("small", { class: "num" }, `${x.hp}/${x.max_hp}`) : "");
  // the climb goes on by itself: `auto` lit (the scout's), else greyed with his count — a preview, never a sentence
  const auto = kwHost(h("span", { class: `lane-auto${x.auto ? " on" : ""}`, "data-auto": x.auto ? "1" : "0" }, h("span", { class: "la-g", "aria-hidden": "true" }, x.auto ? "↻" : "⊘"), /* copy:label */ "auto",
    !x.auto && x.need ? h("small", { class: "num" }, ` ${Math.min(x.have ?? 0, x.need)}/${x.need}`) : ""), "scout");
  const body: (string | HTMLElement)[] = [face, h("span", { class: "lane-name" }, x.name), state, gauge, auto];
  const live = x.state === "live";
  // live: the tap watches it; at home a lane is no surface (its tip on long-press) — the log has its own stud (density: ≤ 12 above the
  // fold with three heroes)
  const tap = live ? () => hooks.watch(x) : null; void hasRuns;
  const row = tap
    ? h("button", { class: "lane", "data-lane": x.id, "data-state": x.state, "data-kind": x.kind ?? "hero", "aria-label": `${x.name} · ${x.state}`, onclick: tap }, ...body,
        h("span", { class: "lane-go", "aria-hidden": "true" }, "▸"))
    : h("div", { class: "lane", "data-lane": x.id, "data-state": x.state, "data-kind": x.kind ?? "hero" }, ...body);
  if (x.rest_s !== undefined) row.dataset.rest = String(Math.round(x.rest_s));
  return kwHost(row, live ? "live" : "lane");
}
/** the rows past the third, as one count (`+2 · 1 live`) */
function moreRow(rest: HeroLane[]): HTMLElement {
  const live = rest.filter((x) => x.state === "live").length;
  return h("div", { class: "lane lane-more", "data-more": rest.length }, h("span", { class: "num" }, `+${rest.length}`), live ? h("small", { class: "dim num" }, /* copy:label */ ` · ${live} live`) : "");
}
/** the rest's share left, against the longest rest (30 min): full just back, empty as he goes */
const REST_FULL_S = 30 * 60;
const restShare = (s: number): number => Math.max(0, Math.min(1, s / REST_FULL_S));

/** RUNS_UI: the watch's `town ↻` carries `he keeps going` over it for this viewer's first few watches (the owner's rule for a new
 *  concept: its mark and a short caption, once learned no more) — the words drawn by CSS from `data-cap`, so the tile's text stays `town` */
const GOES_KEY = "riddle.runs.goes_on", GOES_SHOWN = 3;
export function goesOnCap(): HTMLElement | null {
  let n = 0; try { n = Number(localStorage.getItem(GOES_KEY) ?? 0) || 0; } catch { /* private mode */ }
  if (n >= GOES_SHOWN) return null;
  try { localStorage.setItem(GOES_KEY, String(n + 1)); } catch { /* private mode */ }
  return h("small", { class: "goes-on-cap", "data-cap": /* copy:callout */ "he keeps going", role: "note", "aria-label": "he keeps going" });
}
