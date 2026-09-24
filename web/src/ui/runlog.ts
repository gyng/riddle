// Cut 11 §2 — the last watched run's event log, kept in memory so a chain link on the death screen can scrub a replay to
// its tick. The watch wraps its viewer with `recordRun`: every `load(snap)` opens a floor entry (the snapshot the renderer
// rebuilds from on `seek`), every `apply(evs)` appends to the open one. One run is kept (the last watched); offline runs
// have no log, so their chain links show the tick instead of `watch`.
import type { Because, Entity, Ev, FloorItem, Snapshot } from "../engine/types";
import type { Viewer } from "./viewer";

/** A floor as the watch loaded it, its events since, and what later step snapshots added (`preload` / `sync`: entities
 *  and items first seen after the load, keyed by id — the renderer's `seek` rebuilds from the loaded snapshot alone, so
 *  the replay loads them folded in, see `floorSnapshot`). */
export type Floor = { snap: Snapshot; evs: Ev[]; ents: Map<number, Entity>; items: Map<number, FloorItem>; seen?: boolean[] };
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
    // QA e75ec29 (Q: "WATCH opens a sheet ~90 % black with a 200 px map fragment"): the tiles the floor's later snapshots saw — the
    // replay rebuilds from the loaded snapshot, whose map was the floor's first room only
    if (snap.seen.length === f.snap.seen.length) { f.seen ??= [...f.snap.seen]; for (let k = 0; k < snap.seen.length; k++) if (snap.seen[k]) f.seen[k] = true; }
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
  return { ...f.snap, entities: [...f.snap.entities, ...f.ents.values()], items: [...f.snap.items, ...f.items.values()], seen: f.seen ?? f.snap.seen };
}
/** The run's exit tick, once known (chain links past it are never replayable). */
export function markEnd(runId: number, t: number): void { if (last && last.runId === runId) last.endTick = t; }

/** A floor's first tick: its snapshot is the END of the batch that reached it (a skip steps 100–200 ticks at a time), so the
 *  events after the descend can start before `snap.turn`. */
export const floorStart = (f: Floor): number => Math.min(f.snap.turn, f.evs.length ? f.evs[0].t : f.snap.turn);
const floorEnd = (f: Floor): number => (f.evs.length ? f.evs[f.evs.length - 1].t : f.snap.turn);

/** The floor entry a `because` happened on: the one loaded for its depth whose ticks bracket `t` (a depth revisited — the
 *  fake never does — picks the bracketing one), else that depth's floor nearest `t`. Never another depth's floor (QA on 3d71c33:
 *  a D3 link's clip showed a floor a batch off, captioned by an earlier row); a `because` without a depth takes whichever floor
 *  brackets `t`. Undefined when the run does not hold that tick (before the first load, after the exit, or another run). */
export function floorFor(log: RunLog, b: Because): Floor | undefined {
  if (log.endTick !== undefined && b.t > log.endTick) return undefined;
  const brackets = (f: Floor): boolean => b.t >= floorStart(f) && b.t <= floorEnd(f);
  if (b.depth <= 0) return log.floors.find(brackets);
  const same = log.floors.filter((f) => f.snap.depth === b.depth);
  return same.find(brackets) ?? same.filter((f) => b.t >= floorStart(f)).pop();
}
/** Cut 11 §2 (QA on 3d71c33): a clip's window on its floor — from `t − lead` (never before the floor's first tick) to at least
 *  `t + lead`, `window` ticks long at least, so the link's own tick is always inside it. */
export function clipWindow(f: Floor, t: number, lead: number, window: number): { from: number; to: number } {
  const from = Math.max(floorStart(f), Math.min(t, floorEnd(f)) - lead);
  return { from, to: Math.max(from + window, t + lead) };
}
/** Can the death screen scrub this run's replay to `b`? */
export function replayable(runId: number | undefined, b: Because): boolean {
  return !!last && last.runId === runId && !!floorFor(last, b);
}
