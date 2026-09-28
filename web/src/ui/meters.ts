// Cut 29 §3: the diagnostics meters — one module for every surface that shows them (the watch's toggle, the death
// screen's fight, the report's night, the camp's two-run comparison, the desktop's right column). The core meters the
// event stream (`meters.rs`); this file only merges and draws. Units always (`12 dps`, `3 hp/s`, `$40/min`); a rule is
// named by its words (`tokens.ruleName`), never its index.
import type { MeterSides, MeterWire, Row } from "../engine/types";

const r3 = (x: number): number => Math.round(x * 1000) / 1000;
const addSides = (a: MeterSides, b: MeterSides): MeterSides => ({ hero: a.hero + b.hero, pets: a.pets + b.pets, foes: a.foes + b.foes });
const rate = (x: MeterSides, s: number): MeterSides => ({ hero: r3(x.hero / s), pets: r3(x.pets / s), foes: r3(x.foes / s) });

/** Two meters as one (slices of one absence): totals add, rates are recomputed from the summed totals and seconds the way
 *  the core's `meters::wire` derives them (per game second, floor 0.1 s; a row's share of the summed actions). */
export function mergeMeters(a?: MeterWire, b?: MeterWire): MeterWire | undefined {
  if (!a) return b;
  if (!b) return a;
  const seconds = r3(a.seconds + b.seconds), s = Math.max(0.1, seconds);
  const dealt = addSides(a.dealt, b.dealt), taken = addSides(a.taken, b.taken);
  const heal = new Map<string, number>();
  for (const h of [...a.healed, ...b.healed]) heal.set(h.src, (heal.get(h.src) ?? 0) + h.total);
  const healed = [...heal].map(([src, total]) => ({ src, total, per_s: r3(total / s) }));
  const rows = new Map<number, number>();
  for (const r of [...a.rows, ...b.rows]) rows.set(r.row, (rows.get(r.row) ?? 0) + r.fires);
  const actions = a.actions + b.actions;
  const supplies: { [k: string]: number } = { ...a.supplies };
  for (const [k, n] of Object.entries(b.supplies)) supplies[k] = (supplies[k] ?? 0) + n;
  const gold = a.gold + b.gold;
  const T = (k: "fight" | "travel" | "chores" | "rest"): number => a.time[k] + b.time[k];
  const TS = (k: "fight" | "travel" | "chores" | "rest"): number => r3(a.time_s[k] + b.time_s[k]);
  return {
    seconds, dealt, taken, dps_dealt: rate(dealt, s), dps_taken: rate(taken, s),
    healed, hps: r3(healed.reduce((x, h) => x + h.total, 0) / s),
    time: { fight: T("fight"), travel: T("travel"), chores: T("chores"), rest: T("rest") },
    time_s: { fight: TS("fight"), travel: TS("travel"), chores: TS("chores"), rest: TS("rest") },
    rows: [...rows].sort((x, y) => x[0] - y[0]).map(([row, fires]) => ({ row, fires, share: r3(fires / Math.max(1, actions)) })),
    actions, supplies, gold, gold_per_min: r3(gold / (s / 60)),
    hits_hero: a.hits_hero + b.hits_hero, hits_pets: a.hits_pets + b.hits_pets, fights: a.fights + b.fights,
  };
}

export type RuleNamer = (row: Row, rows: Row[]) => string;
