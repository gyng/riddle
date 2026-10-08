import type { Counter, Lineage, Row } from "../engine/types";

/** Blind 5331f40 (both raters: the floor chart's `counter: cadence` and the boss bar's `COUNTER: CADENCE` never named the tactic
 *  `Mirror rhythm`, found only by browsing): a counter that is a tactic package reads as the package's own name, the name the
 *  tactics panel lists (`cadence` → `Mirror rhythm`, `reflect read` → `mirror read`); any other counter as the core wrote it. */
export function counterName(L: Pick<Lineage, "packages"> | undefined, text: string, row?: Row | string): string {
  const id = row && typeof row !== "string" && row.verb.v === "tactic" ? row.verb.a ?? "" : text.trim().replace(/ /g, "_");
  const p = (L?.packages?.all ?? []).find((x) => x.kind === "tactic" && (x.id === id || x.name.toLowerCase() === text.trim().toLowerCase()));
  return p?.name ?? text;
}

/** The lineage's known counters (`Lineage.counters`), each by the name the player finds it under (`counterName`). */
export const namedCounters = (L: Pick<Lineage, "packages" | "counters">): Counter[] => (L.counters ?? []).map((c) => ({ ...c, text: counterName(L, c.text, c.row) }));
