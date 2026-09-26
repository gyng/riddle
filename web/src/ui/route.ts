// Cut 26 §2 — the route is policy: the set's route (`RuleSet.route`, the fork depths whose far stair the hero takes; empty = every
// near stair, the base order) shown above the rows as a chip line (`⑂ D5 fens · D14 crypt`) once the lineage has seen a fork
// (`Lineage.forks`, the facts `fork:<d>`), and its sheet — the fork tablet: both stairs priced for the current set (`forkForecast`:
// `fens D8 61% · burrows D8 34%`), the tap writes the route (a set edit: `vs sent` reads it like any other). Also the lanes a report
// names (`D5–8 · the Fens`), the shaft's untaken lane (`crypt · D9 · ?`) and the start sheet's (lane, depth) pairs.
//
// The core's wire names every lane (`Lineage.forks`, `ForkOption.route`, `ForecastDepth.biome`, `Lineage.lanes`, `ReturnReport.lanes`).
// The tables below mirror crates/riddle-core/src/descent.rs and only stand in between an edit and the core's next word (a chip's
// stair right after a tap) or on an older wire.
import type { ForkChip, Lineage, RuleSet } from "../engine/types";

export const FORKS = [5, 9, 14, 19, 24] as const;
export const BANDS: readonly (readonly [number, number])[] = [[5, 8], [9, 13], [14, 18], [19, 23], [24, 28], [29, 33]];
export const BASE_ORDER = ["burrows", "fens", "crypt", "foundry", "deep", "sanctum"] as const;
/* copy:callout */
const TITLE: Record<string, string> = { warrens: "the Warrens", burrows: "the Burrows", fens: "the Fens", crypt: "the Crypt", foundry: "the Foundry", deep: "the Deep", sanctum: "the Sanctum" };
export const laneTitle = (b: string): string => TITLE[b] ?? b.replace(/_/g, " ");

/** The set's far stairs, ascending. */
export const routeForks = (s: RuleSet | undefined): number[] => [...new Set(s?.route ?? [])].sort((a, b) => a - b);
const sameRoute = (a: number[], b: number[]): boolean => a.length === b.length && a.every((x, i) => x === b[i]);

/** The biomes of the six bands on `route`, in order (a far stair swaps its band with the next) — the core's `Route::order`. */
export function order(route: number[]): string[] {
  const o: string[] = [...BASE_ORDER];
  for (const f of [...route].sort((a, b) => a - b)) { const i = (FORKS as readonly number[]).indexOf(f); if (i >= 0 && i + 1 < o.length) [o[i], o[i + 1]] = [o[i + 1], o[i]]; }
  return o;
}
export const bandOf = (depth: number): number => { if (depth < BANDS[0][0]) return -1; const i = BANDS.findIndex(([a, b]) => depth >= a && depth <= b); return i < 0 ? BANDS.length - 1 : i; };
export const biomeAt = (route: number[], depth: number): string => { const i = bandOf(depth); return i < 0 ? "warrens" : order(route)[i]; };

type ForkL = Pick<Lineage, "facts" | "forks">;
/** The forks the lineage has seen: the core's chips (`Lineage.forks`), else the facts `fork:<d>` / `fork D5` on the base order's table. */
function seenChips(L: ForkL | undefined): ForkChip[] {
  if (L?.forks?.length) return [...L.forks].sort((a, b) => a.depth - b.depth);
  const out: ForkChip[] = [];
  for (const f of L?.facts ?? []) {
    const m = /^fork[ :]?\s*D?(\d+)$/i.exec(f), d = m ? Number(m[1]) : NaN, i = (FORKS as readonly number[]).indexOf(d);
    if (i >= 0 && !out.some((c) => c.depth === d)) out.push({ depth: d, near: BASE_ORDER[i], far: BASE_ORDER[i + 1], taken: BASE_ORDER[i] });
  }
  return out.sort((a, b) => a.depth - b.depth);
}
export const seenForks = (L: ForkL | undefined): number[] => seenChips(L).map((c) => c.depth);
/** The fork table adjacency is read on: the core's forks when every seen one is among them, else the seen forks themselves. */
const tableOf = (chips: ForkChip[]): number[] => chips.every((c) => (FORKS as readonly number[]).includes(c.depth)) ? [...FORKS] : chips.map((c) => c.depth);

export type RouteChip = ForkChip & { open: boolean; isFar: boolean };
/** The chip line for `set`: each seen fork, the lane the set's route takes there and whether it is a choice (`open`). The core's chips
 *  speak for the set it holds; a set edited since (a stair just tapped) reads its own route on the same forks. */
export function routeChips(set: RuleSet, L: (ForkL & Partial<Pick<Lineage, "sets" | "active_set">>) | undefined): RouteChip[] {
  const chips = seenChips(L), r = routeForks(set);
  const held = L?.sets?.[L.active_set ?? 0], fresh = !!L?.forks?.length && !!held && sameRoute(routeForks(held), r);
  const table = tableOf(chips);
  return chips.map((c) => {
    if (fresh) { const open = c.open !== false; return { ...c, open, isFar: open && c.taken === c.far && c.far !== c.near }; }
    const prev = table[table.indexOf(c.depth) - 1], open = prev === undefined || !r.includes(prev);
    const prevChip = chips.find((x) => x.depth === prev);
    const taken = open ? (r.includes(c.depth) ? c.far : c.near) : prevChip?.near ?? c.taken;
    return { ...c, taken, open, isFar: open && r.includes(c.depth) };
  });
}
/** `route` with the stair at `fork` set — a far stair drops the neighbouring far stairs it overlaps (the core's `Route::with`). The
 *  fork tablet's options carry the core's own (`ForkOption.route`); this stands in when they have not landed. */
export function withFork(route: number[], fork: number, far: boolean, L?: ForkL): number[] {
  const table = tableOf(seenChips(L)), i = table.indexOf(fork);
  let out = route.filter((f) => f !== fork);
  if (far) { out = out.filter((f) => i < 0 || (f !== table[i - 1] && f !== table[i + 1])); out.push(fork); }
  return out.sort((a, b) => a - b);
}
/** The lane the route does not take at each open fork (`crypt · D9 · ?` on the shaft) — a frontier unless the lineage has entered it. */
export function frontiers(set: RuleSet, L: (ForkL & Partial<Pick<Lineage, "sets" | "active_set">>) | undefined): { fork: number; biome: string; entered: boolean }[] {
  const facts = new Set(L?.facts ?? []);
  return routeChips(set, L).filter((c) => c.open).map((c) => {
    const other = c.taken === c.near ? c.far : c.near;
    return { fork: c.depth, biome: other, entered: facts.has(`biome:${other}`) };
  });
}
/** The report's lanes on an older wire: each band the route passes, `D5–8 · the Fens`, down to `deepest`. */
export function lanes(route: number[], deepest = Infinity): { from: number; to: number; biome: string }[] {
  const o = order(route);
  return BANDS.map(([a, b], i) => ({ from: a, to: b, biome: o[i] })).filter((x, i) => i === 0 || x.from <= deepest);
}
