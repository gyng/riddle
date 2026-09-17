// Cut 11 §2 — the last watched run's event log, kept in memory so a chain link on the death screen can scrub a replay to
// its tick. The watch wraps its viewer with `recordRun`: every `load(snap)` opens a floor entry (the snapshot the renderer
// rebuilds from on `seek`), every `apply(evs)` appends to the open one. One run is kept (the last watched); offline runs
// have no log, so their chain links show the tick instead of `watch`.
import type { Because, Entity, Ev, FloorItem, Snapshot } from "../engine/types";
import type { Viewer } from "./viewer";

/** A floor as the watch loaded it, its events since, and what later step snapshots added (`preload` / `sync`: entities
 *  and items first seen after the load, keyed by id — the renderer's `seek` rebuilds from the loaded snapshot alone, so
 *  the replay loads them folded in, see `floorSnapshot`). */
export type Floor = { snap: Snapshot; evs: Ev[]; ents: Map<number, Entity>; items: Map<number, FloorItem> };
export type RunLog = { runId: number; startedTurn: number; floors: Floor[]; endTick?: number };

let last: RunLog | null = null;

/** The last watched run's log (null before any run this session). */
export function lastRun(): RunLog | null { return last; }

/** Start a log for `runId` and return a viewer whose `load` / `apply` / `preload` / `sync` record into it (everything else
 *  passes through). */
export function recordRun<V extends Viewer>(viewer: V, runId: number, startedTurn: number): V {
  const log: RunLog = { runId, startedTurn, floors: [] };
  last = log;
  const cur = (): Floor | undefined => log.floors[log.floors.length - 1];
  // a later snapshot of the same floor: entities and items the loaded snapshot did not have (first sighting wins)
  const fold = (snap: Snapshot): void => {
    const f = cur(); if (!f || snap.depth !== f.snap.depth) return;
    const had = new Set(f.snap.entities.map((e) => e.id)); had.add(f.snap.hero.id);
    for (const e of snap.entities) if (!had.has(e.id) && !f.ents.has(e.id)) f.ents.set(e.id, e);
    const items = new Set(f.snap.items.map((i) => i.id));
    for (const i of snap.items) if (!items.has(i.id) && !f.items.has(i.id)) f.items.set(i.id, i);
  };
  const pre = (viewer as Viewer & { preload?: (s: Snapshot) => void }).preload;
  return {
    ...viewer,
    load(snap: Snapshot) { log.floors.push({ snap, evs: [], ents: new Map(), items: new Map() }); viewer.load(snap); },
    apply(evs: Ev[]) { const f = cur(); if (f && evs.length) f.evs.push(...evs); viewer.apply(evs); },
    ...(pre ? { preload(snap: Snapshot) { fold(snap); pre(snap); } } : {}),
    ...(viewer.sync ? { sync(snap: Snapshot) { fold(snap); viewer.sync!(snap); } } : {}),
  };
}
/** The snapshot a replay loads for a floor: the loaded one with every entity and item later snapshots added, so a foe
 *  that walked in after the load exists before its events apply (and survives the renderer's `seek`). */
export function floorSnapshot(f: Floor): Snapshot {
  return { ...f.snap, entities: [...f.snap.entities, ...f.ents.values()], items: [...f.snap.items, ...f.items.values()] };
}
/** The run's exit tick, once known (chain links past it are never replayable). */
export function markEnd(runId: number, t: number): void { if (last && last.runId === runId) last.endTick = t; }

/** The floor entry a `because` happened on: the one loaded for its depth whose ticks bracket `t` (a depth revisited — the
 *  fake never does — picks the bracketing one), else any floor whose ticks bracket `t`. Undefined when the run does not hold
 *  that tick (before the first load, after the exit, or another run). */
export function floorFor(log: RunLog, b: Because): Floor | undefined {
  if (log.endTick !== undefined && b.t > log.endTick) return undefined;
  const lastT = (f: Floor): number => (f.evs.length ? f.evs[f.evs.length - 1].t : f.snap.turn);
  const brackets = (f: Floor): boolean => b.t >= f.snap.turn && b.t <= lastT(f);
  return log.floors.find((f) => f.snap.depth === b.depth && brackets(f))
    ?? log.floors.find((f) => brackets(f))
    ?? (b.depth > 0 ? log.floors.filter((f) => f.snap.depth === b.depth).find((f) => b.t >= f.snap.turn) : undefined);
}
/** Can the death screen scrub this run's replay to `b`? */
export function replayable(runId: number | undefined, b: Because): boolean {
  return !!last && last.runId === runId && !!floorFor(last, b);
}
