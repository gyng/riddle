// Cut 27 §1 — solved floors fold. The sim still plays every floor (economy, determinism unchanged); a floor the active set clears
// ≥ 95 % on the camp's forecast (paired, per floor) is *folded*: the watch steps the engine through it flat out under one interstitial
// line (`D1–6 · 100% · +$84` and a chip per state-changing beat: `stolen heal`, `took mail`, `hp 4/36`) and opens at the first floor below
// the bar. Nothing that changes the lineage is hidden: every theft, find, cage pick, companion's news, level, max-hp move, bones, fact
// learned, boss down and death-adjacent hp dip of a folded floor is a chip on its line. A tap on the line plays the folded floors
// (`openFoldReplay`, the run log's floors at the travel rate).
//
// Which floors fold is the core's call when it sends one (`Forecast.fold_to`, or per floor `ForecastDepth.fold` / `.clear`); otherwise
// the client reads it off the reach: floor d clears with reach(d + 1) / reach(d) ≥ FOLD_CLEAR, over a reach that is itself read
// (≥ FOLD_MIN_REACH), and never the floor the set's own bank row sends him home from (reach below it falls: it is never ≥ 95 %).
import type { Ev, Forecast, Snapshot } from "../engine/types";

export const FOLD_CLEAR = 0.95;
const FOLD_MIN_REACH = 0.5;

type FoldWire = { fold_to?: number; known_to: number; depths: (Forecast["depths"][number] & { fold?: boolean; folded?: boolean; clear?: number })[] };

/** The floors the watch folds for a send from `start` — every floor whose clear share is ≥ FOLD_CLEAR (the core's own mark when it
 *  sends one). Empty without a forecast, or when `?fold=0` (dev) turns folding off. */
export function foldFloors(f: Forecast | null, start: number): Set<number> {
  const out = new Set<number>();
  if (!f || foldOff()) return out;
  const w = f as unknown as FoldWire;
  // blind 7f7fc2b (A: the Warlord fell inside `D1–8 · 100% · boss down`, never seen): a boss's floor never folds (the core's
  // `fold()` stops before it too) — the fold breaks there and the fall is watched
  const boss = new Set(w.depths.filter((d) => d.boss).map((d) => d.depth));
  if (typeof w.fold_to === "number") { for (let d = Math.max(1, start); d <= w.fold_to; d++) if (!boss.has(d)) out.add(d); return out; }
  const marked = w.depths.some((d) => d.fold !== undefined || d.folded !== undefined || d.clear !== undefined);
  const by = new Map(w.depths.map((d) => [d.depth, d]));
  for (const d of w.depths) {
    if (d.depth < start || boss.has(d.depth)) continue;
    if (marked) {
      if (d.fold || d.folded || (d.clear !== undefined && d.clear >= FOLD_CLEAR)) out.add(d.depth);
      continue;
    }
    const next = by.get(d.depth + 1); if (!next || d.depth + 1 > f.known_to) continue;
    const here = d.depth <= start ? 1 : d.reach;
    if (here < FOLD_MIN_REACH) continue;
    if (next.reach / here >= FOLD_CLEAR) out.add(d.depth);
  }
  return out;
}
/** The share of sends that clear the stretch `a..b` (reach(b + 1) / reach(a)), for the line's `100%`. */
export function stretchShare(f: Forecast | null, a: number, b: number, start: number): number | undefined {
  if (!f) return undefined;
  const r = (d: number): number | undefined => (d <= start ? 1 : f.depths.find((x) => x.depth === d)?.reach);
  const top = r(a), end = r(b + 1);
  return top && end !== undefined ? Math.max(0, Math.min(1, end / top)) : undefined;
}
const foldOff = (): boolean => { try { return new URLSearchParams(location.search).get("fold") === "0"; } catch { return false; } };

/** The kinds of state change a fold line must carry (the gate's list: each that happened on a folded floor is a chip). */
export type FoldKind = "stolen" | "found" | "took" | "pet" | "level" | "max_hp" | "bones" | "learned" | "boss" | "hp" | "situation";
export type FoldChip = { k: FoldKind; text: string; t: number; depth: number };

const SITUATION: [RegExp, (m: RegExpExecArray) => string][] = [
  [/^Freed the captive/i, () => /* copy:callout */ "freed captive"],
  [/^Cut the captive/i, () => /* copy:callout */ "cut captive"],
  [/^Lit the shrine/i, () => /* copy:callout */ "lit shrine"],
  [/^([A-Z][a-z]+)(?: the [a-z ]+)?, gone wild\b/, (m) => /* copy:callout */ `${m[1]} gone wild`],
  [/^([A-Z][a-z]+) is avenged/i, (m) => /* copy:callout */ `avenged ${m[1]}`],
  [/^([A-Z][a-z]+) joins\.$/, (m) => /* copy:callout */ `${m[1]} joins`],
  [/^([A-Z][a-z]+): level (\d+)\.$/, (m) => /* copy:callout */ `${m[1]} L${m[2]}`],
];
const oneWord = (s: string): string => s.replace(/_/g, " ").trim().split(/\s+/).pop() ?? s;
const short = (s: string): string => s.replace(/_/g, " ").trim().split(/\s+/).slice(-2).join(" ");

/** Collects a folded stretch's state changes from the batches the engine stepped under the line, and its carry. */
export class FoldTally {
  readonly chips: FoldChip[] = [];
  private finds = new Map<string, number>();
  private learned = 0;
  private maxHp = new Map<string, number>();   // max-hp moves per cause, summed (a drain bites every few turns)
  private minHp: { hp: number; max: number; t: number; depth: number } | null = null;
  private names = new Map<number, string>();
  private allies = new Set<number>();
  private bosses = new Set<number>();
  private cages = new Set<string>();
  loot0: number | undefined;
  loot = 0;
  depth = 0;
  readonly from: number;
  readonly share: number | undefined;
  constructor(from: number, share: number | undefined) { this.from = from; this.share = share; }

  absorb(evs: Ev[], s: Snapshot): void {
    const heroId = s.hero.id;
    for (const e of s.entities) { if (e.ally) this.allies.add(e.id); this.names.set(e.id, (e.name ?? e.kind)); if (e.tags.includes("boss")) this.bosses.add(e.id); }
    let depth = this.depth || s.depth;
    const add = (k: FoldKind, text: string, t: number): void => { this.chips.push({ k, text, t, depth }); };
    for (const ev of evs) {
      switch (ev.k) {
        case "descend": depth = ev.depth; break;
        case "steal": add("stolen", ev.amount ? /* copy:callout */ `stolen $${ev.amount}` : /* copy:callout */ `stolen ${short(ev.item)}`, ev.t); break;
        case "pickup": if (ev.id === heroId && !/^gold\b/.test(ev.item)) { const k = short(ev.item); this.finds.set(k, (this.finds.get(k) ?? 0) + 1); } break;
        case "fact": this.learned++; break;
        case "level": add("level", `${ev.class} L${ev.level}`, ev.t); break;
        case "max_hp": if (ev.id === heroId && ev.delta !== 0) { const c = oneWord(ev.cause); this.maxHp.set(c, (this.maxHp.get(c) ?? 0) + ev.delta); } break;
        case "bones": if (ev.heir !== s.run.heir) add("bones", /* copy:callout */ `heir ${ev.heir} bones`, ev.t); break;
        case "tame": if (ev.ok) { this.allies.add(ev.id); add("pet", /* copy:callout */ `tamed ${oneWord(ev.kind)}`, ev.t); } break;
        case "hatch": add("pet", /* copy:callout */ `hatched ${oneWord(ev.kind)}`, ev.t); break;
        case "ally": if (ev.state === "lost") add("pet", /* copy:callout */ `${oneWord(this.names.get(ev.id) ?? "ally")} lost`, ev.t); break;
        case "die":
          if (ev.id !== heroId && this.allies.has(ev.id)) add("pet", /* copy:callout */ `${oneWord(this.names.get(ev.id) ?? "ally")} fell`, ev.t);
          else if (this.bosses.has(ev.id)) add("boss", /* copy:callout */ `${oneWord(this.names.get(ev.id) ?? "boss")} down`, ev.t);
          break;
        case "hurt": if (ev.id === heroId) { const max = s.hero.max_hp || 1; if (!this.minHp || ev.hp < this.minHp.hp) this.minHp = { hp: ev.hp, max, t: ev.t, depth }; } break;
        case "note": { for (const [re, f] of SITUATION) { const m = re.exec(ev.text); if (m) { add("situation", f(m), ev.t); break; } } break; }
        default: break;
      }
    }
    this.depth = s.depth;
    // Cut 19 §1: a cage met under the fold — the preference's pick is the beat (`took mail`)
    const vc = s.vault_choice;
    if (vc?.items.length) {
      const key = vc.items.map((i) => i.id).join(",");
      if (!this.cages.has(key)) { this.cages.add(key); const pick = vc.items.find((i) => i.id === vc.pick) ?? vc.items[0]; add("took", /* copy:callout */ `took ${short(pick.label)}`, s.turn); }
    }
    const loot = carriedOf(s);
    if (this.loot0 === undefined) this.loot0 = loot;   // (the watch sets it from the snapshot before the fold)
    this.loot = loot;
  }

  /** The chips in order: the beats as they came, then the finds, what was learned, and the hp dip (a death-adjacent one only). */
  list(): FoldChip[] {
    const out = [...this.chips];
    if (this.finds.size) {
      const n = [...this.finds.values()].reduce((a, b) => a + b, 0);
      const [k] = [...this.finds.keys()];
      out.push({ k: "found", text: this.finds.size === 1 ? /* copy:callout */ `found ${k}${n > 1 ? ` ×${n}` : ""}` : /* copy:callout */ `found ${n}`, t: 0, depth: 0 });
    }
    for (const [c, d] of this.maxHp) if (d !== 0) out.push({ k: "max_hp", text: /* copy:callout */ `${c} ${d > 0 ? "+" : "−"}${Math.abs(d)} max`, t: 0, depth: 0 });
    if (this.learned) out.push({ k: "learned", text: /* copy:callout */ `learned ${this.learned}`, t: 0, depth: 0 });
    const m = this.minHp;
    if (m && m.hp / m.max < 0.25) out.push({ k: "hp", text: `hp ${Math.max(0, m.hp)}/${m.max}`, t: m.t, depth: m.depth });
    return out;
  }
  /** The kinds seen (the gate compares these with the kinds the line shows). */
  kinds(): Set<FoldKind> { return new Set(this.list().map((c) => c.k)); }
  /** The line's head: `D1–6 · 100% · +$84` (the stretch, its clear share on the forecast, the carry it added). */
  head(to: number): string {
    const span = to > this.from ? `D${this.from}–${to}` : `D${this.from}`;
    const share = this.share !== undefined ? ` · ${Math.round(this.share * 100)}%` : "";
    const gain = this.loot - (this.loot0 ?? this.loot);
    return `${span}${share}${gain !== 0 ? ` · ${gain > 0 ? "+" : "−"}$${Math.abs(gain)}` : ""}`;
  }
}

/** blind 1fb7786 (A): the run's carried gold as the core counts it (`Run::carried`: the checkpoints' secured gold with the carry at risk
 *  since) — the strip's `Carried`, the card's `carry`, a fold's gain; a checkpoint moves gold between its parts, never out of it. */
export function carriedOf(s: { loot: number; stake?: { loot: number; death_keep?: number } }): number {
  return s.stake ? Math.max(0, s.stake.loot) + (s.stake.death_keep ?? 0) : s.loot;
}
