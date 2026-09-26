// Cut 17 §3 — the reveal ladder (docs/UI.md §5). The frame grows: a console tile is carved in when it first means something,
// read off the lineage (facts, the gold ledger, heirs), so an existing save starts fully revealed and nothing is ever taken
// away. A step once shown stays shown (per lineage seed, localStorage), and its first appearance glints once (`fresh`).
//
//   step 0  fresh lineage          the bar ($ only), the tablets (compact), the shaft (D1), the gem SEND
//   edit    first death            the `edit` tile (the tablets' reorder / delete / add); the class picker on the portrait
//   loadout first gold home        the `loadout` tile (supplies)
//   unlocks first mark             `◆` on the bar, the `unlocks` tile (the next three)
//   vault   an item kept, a slot   the `vault` tile (an item kept, or a `+1 vault` bought)
//   forge   first salvage          the `forge` tile
//   kit     a kit step affordable  the `forge` tile too (Cut 23 §1: the heir's kit, bought with gold) — it glints then
//   party   a companion            the `party` tile
//   gems    a 3rd row              the shaft's bank / return / death gems
//   heirs   5 heirs                the `chronicle` and `ledger` tiles; set tabs 2–3
//   cage    a cage seen (`vault`)  the cage tablet under the rules (`cage → armour`, Cut 19 §1)
//   start   a waystone lit         the start tablet beside them (`start → D9`, Cut 21 §1)
//   oaths   an oath affordable     the oath tablet beside them (`oath → D10 no drink 34%`, Cut 28 §1); the sworn oath on the shaft
//   (a second class owned: the wake's class chips — the core offers them only then)
import type { App } from "../app";
import { isFreeSupply, ownRowCount } from "./tokens";
import { oathsEarned } from "./oaths";

export type Step = "edit" | "loadout" | "unlocks" | "vault" | "forge" | "party" | "gems" | "heirs" | "rank" | "depth" | "cage" | "start" | "kit" | "oaths";
const KEY = "riddle.reveal";
const FRESH_UNLOCKS = new Set(["tame"]);
/** When each step was first seen this session (wall ms): a step glints on every paint for GLINT_MS after it was earned, so the
 *  several readers of one paint (the bar, the console) agree. */
const firstSeen = new Map<string, number>();
const GLINT_MS = 1500;

/** What the lineage has earned, read fresh (no memory). */
export function earned(app: App): Set<Step> {
  const L = app.lineage, out = new Set<Step>();
  if (!L) return out;
  if ((L.graveyard?.length ?? 0) > 0 || L.heir > 1 || (L.chronicle?.length ?? 0) > 0) out.add("edit");
  // a fresh lineage owns `tame` and a free leash on the shelf (the core's new_lineage): neither is earned
  if (L.gold > 0 || (L.gold_ledger ?? []).some((g) => g.delta > 0) || (L.supplies ?? []).some((s) => !isFreeSupply(L, s))) out.add("loadout");
  if (L.marks > 0 || (L.unlocks ?? []).some((u) => !FRESH_UNLOCKS.has(u)) || (L.rank ?? 0) > 0) out.add("unlocks");
  // QA 23ed91f (K: "bought `+1 vault` … no vault tile anywhere"): a vault slot bought is a reason to see the vault (and its prefs)
  if ((L.vault?.length ?? 0) > 0 || app.loadout.length > 0 || (L.unlocks ?? []).some((u) => /^vault\d+$/.test(u))) out.add("vault");
  if (Object.keys(L.forge ?? {}).length > 0) out.add("forge");
  // Cut 23 §1: the first kit step the purse can buy (or one owned) carves the forge
  if ((L.kit ?? []).some((k) => k.owned > 0 || k.next?.affordable)) out.add("kit");
  if ((L.party?.length ?? 0) + (L.kennel?.length ?? 0) + (L.eggs?.length ?? 0) > 0) out.add("party");
  if (app.sets.some((s) => ownRowCount(s.rows) >= 3) || (L.unlocks ?? []).some((u) => /^row\d+$/.test(u))) out.add("gems");
  if (L.heir >= 5) out.add("heirs");
  if ((L.rank ?? 0) > 0 || (L.renown ?? 0) > 0) out.add("rank");
  if (L.best_depth > 0) out.add("depth");
  // Cut 19 §1: a cage seen (the core's `vault` fact, learned when one opens) carves the cage tablet beside the rules
  if ((L.facts ?? []).includes("vault")) out.add("cage");
  // Cut 21 §1: the first waystone lit carves the start tablet
  if ((L.waystones?.length ?? 0) > 0) out.add("start");
  // Cut 28 §1: the oath board is carved when the purse first affords an oath
  if (oathsEarned(L)) out.add("oaths");
  return out;
}

function stored(seed: number): Set<Step> {
  try { const s = JSON.parse(localStorage.getItem(KEY) ?? "null") as { seed: number; steps: Step[] } | null; return new Set(s && s.seed === seed ? s.steps : []); }
  catch { return new Set(); }
}
function store(seed: number, steps: Set<Step>): void {
  try { localStorage.setItem(KEY, JSON.stringify({ seed, steps: [...steps] })); } catch { /* private mode: the ladder is read from the lineage anyway */ }
}

/** The revealed steps (earned ∪ once shown for this lineage) and the ones appearing for the first time now (they glint). */
export function revealed(app: App): { has: (s: Step) => boolean; fresh: (s: Step) => boolean; all: Set<Step> } {
  const seed = app.lineage?.seed ?? 0;
  const was = stored(seed), now = earned(app);
  const all = new Set<Step>([...was, ...now]);
  // the first read of a lineage (nothing stored) is not a reveal: an old save starts revealed, without a glint on every tile
  let first = false; try { first = !was.size && localStorage.getItem(KEY) === null; } catch { /* private mode */ }
  const t = performance.now();
  if (!first) for (const s of now) if (!was.has(s)) firstSeen.set(`${seed}:${s}`, t);
  if (first || all.size !== was.size) store(seed, all);
  const fresh = (s: Step): boolean => t - (firstSeen.get(`${seed}:${s}`) ?? -Infinity) < GLINT_MS;
  return { has: (s) => all.has(s), fresh, all };
}
