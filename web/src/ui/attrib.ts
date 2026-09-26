// Cut 28 §2 — the answer names its cause: a forecast move that no row edit made (a pet died, the kit, the purse, a fact learned, a new
// heir) is the STATE's, and reads as its own line under the shaft (`party −2 jackals · death +24`), apart from the rows' `vs sent`.
// The camp's last refined forecast is kept per rules with the lineage it was painted under (`StateSnap`, per seed in localStorage, so an
// absence reads too); after a run, the first refined forecast of the same rules on the same number of sims is the state's move — the
// sent set now less the sent set then. When the rows changed too, the sent set's share now is the `vs` move's `base` (the core pairs it on
// today's lineage): the state's part is `base − then`, the rows' part is the `vs` move itself, and the two sum to `shown − then`.
import type { Forecast, ForecastVs, Lineage, RuleSet, VsMove } from "../engine/types";

/** What a forecast's sims start from, in the words a state change is named by. */
export type LinSum = { heir: number; party: string[]; facts: number; kit: string; trait: string; cls: string; maxhp?: number };
export type StateSnap = { seed: number; rules: string; lin: LinSum; shares: Record<string, number>; pm: Record<string, number>; sims: number; ran: boolean; low?: number };
/** One part of the state's move (`death +24`): the key (`death`, `bank`, `D9`), the move in points, its noise in points. */
export type StateTerm = { key: string; pts: number; pm: number; worse: boolean };
export type StateMove = { label: string; rules: string; terms: StateTerm[] };

const KEY = "riddle.statesnap";
export const rulesKey = (s: RuleSet): string => JSON.stringify([s.rows.map((r) => [r.conds, r.verb]), s.route ?? []]);

export function linSum(L: Lineage): LinSum {
  return { heir: L.heir, party: (L.party ?? []).map((c) => c.kind).sort(), facts: L.facts?.length ?? 0,
    kit: JSON.stringify((L.kit ?? []).map((k) => k.owned)), trait: L.trait, cls: L.class };
}
/** The shares a forecast paints, by key (`D9`, `bank`, `death`), and their ± (a 0..1 fraction). */
export function sharesOf(f: Forecast): { shares: Record<string, number>; pm: Record<string, number> } {
  const shares: Record<string, number> = {}, pm: Record<string, number> = {};
  const n = f.sims ?? 0, bin = (p: number): number => n > 0 ? 1.96 * Math.sqrt(Math.max(0, p * (1 - p)) / n) : 0;
  for (const d of f.depths) { shares[`D${d.depth}`] = d.reach; pm[`D${d.depth}`] = d.pm ?? bin(d.reach); }
  if (f.ends) {
    shares.bank = f.ends.bank; pm.bank = bin(f.ends.bank);
    shares.death = f.ends.death; pm.death = f.ends.pm ?? bin(f.ends.death);
    shares.return = f.ends.return; pm.return = bin(f.ends.return);
  }
  return { shares, pm };
}
export function readSnap(seed: number): StateSnap | null {
  try { const s = JSON.parse(localStorage.getItem(KEY) ?? "null") as StateSnap | null; return s && s.seed === seed ? s : null; } catch { return null; }
}
export function writeSnap(s: StateSnap | null): void {
  try { if (s) localStorage.setItem(KEY, JSON.stringify(s)); else localStorage.removeItem(KEY); } catch { /* private mode: this session only */ }
}

const plural = (k: string, n: number): string => (n === 1 ? k : /(s|sh|ch|x)$/.test(k) ? `${k}es` : `${k}s`);
/** What changed in the state, ≤ 2 parts of ≤ 3 words: `party −2 jackals`, `learned 3`, `new heir`, `kit`; empty when nothing did. */
export function stateLabel(a: LinSum, b: LinSum): string {
  const parts: string[] = [];
  const count = (xs: string[]): Map<string, number> => { const m = new Map<string, number>(); for (const x of xs) m.set(x, (m.get(x) ?? 0) + 1); return m; };
  const pa = count(a.party), pb = count(b.party);
  const lost: string[] = [], got: string[] = [];
  for (const [k, n] of pa) for (let i = (pb.get(k) ?? 0); i < n; i++) lost.push(k);
  for (const [k, n] of pb) for (let i = (pa.get(k) ?? 0); i < n; i++) got.push(k);
  const kinds = (xs: string[]): string => { const ks = [...new Set(xs)]; return ks.length === 1 ? ` ${plural(ks[0].replace(/_/g, " "), xs.length)}` : ""; };
  if (lost.length && !got.length) parts.push(/* copy:callout */ `party −${lost.length}${kinds(lost)}`);
  else if (got.length && !lost.length) parts.push(/* copy:callout */ `party +${got.length}${kinds(got)}`);
  else if (lost.length || got.length) parts.push(/* copy:callout */ `party ${b.party.length - a.party.length >= 0 ? "+" : "−"}${Math.abs(b.party.length - a.party.length)}`);
  if (a.kit !== b.kit) parts.push(/* copy:callout */ "kit");
  if (a.heir !== b.heir && (a.trait !== b.trait || a.cls !== b.cls)) parts.push(/* copy:callout */ `heir ${b.cls === a.cls ? b.trait : b.cls}`);
  else if (a.heir !== b.heir) parts.push(/* copy:callout */ "new heir");
  if (b.facts > a.facts) parts.push(/* copy:callout */ `learned ${b.facts - a.facts}`);
  return parts.slice(0, 2).join(" · ");
}
/** The state's terms that clear their noise (the two panels are not paired: the ± of each, summed in quadrature): death first, then
 *  the bank, then the depth that moved most. */
export function stateTerms(then: StateSnap, now: Record<string, number>, nowPm: Record<string, number>): StateTerm[] {
  const term = (key: string, worse: boolean): StateTerm | null => {
    const a = then.shares[key], b = now[key];
    if (typeof a !== "number" || typeof b !== "number") return null;
    const pts = Math.round(b * 100) - Math.round(a * 100), pm = Math.max(1, Math.round(Math.hypot(then.pm[key] ?? 0, nowPm[key] ?? 0) * 100));
    return { key, pts, pm, worse };
  };
  const out: StateTerm[] = [];
  const death = term("death", true), bank = term("bank", false);
  if (death && Math.abs(death.pts) > death.pm) out.push(death);
  if (bank && Math.abs(bank.pts) > bank.pm) out.push(bank);
  if (!out.length) {
    const ds = Object.keys(then.shares).filter((k) => /^D\d+$/.test(k)).map((k) => term(k, false)).filter((t): t is StateTerm => !!t && Math.abs(t.pts) > t.pm);
    ds.sort((x, y) => Math.abs(y.pts) - Math.abs(x.pts));
    if (ds[0]) out.push(ds[0]);
  }
  return out;
}
/** The sent set's shares now, off a `vs` move's bases (the core pairs the sent set on today's lineage). */
export function basesOf(vs: ForecastVs): { shares: Record<string, number>; pm: Record<string, number> } {
  const shares: Record<string, number> = {}, pm: Record<string, number> = {};
  const add = (k: string, m: number | VsMove | undefined): void => { if (m && typeof m === "object" && typeof m.base === "number") { shares[k] = m.base; pm[k] = m.pm ?? 0; } };
  for (const d of vs.depths) add(`D${d.depth}`, d);
  add("bank", vs.bank); add("death", vs.death); add("return", vs.return);
  return { shares, pm };
}
