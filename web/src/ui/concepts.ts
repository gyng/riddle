// Cut 29 (owner, docs/COPY.md §4 blocker 1: "a world concept one screen cannot define" — `from cages`, `keep for heirs`, `bones to
// recover`, `◆ 21`, `★ 1`, `oaths`, `bounty`, the fork's other lane): each world concept has one icon, used wherever it appears, and a
// caption of ≤ 3 words the first time a viewer sees it — once seen (on screen ~2 s) the icon stands alone. Per-viewer memory
// (localStorage); without it the caption shows every time, which only costs words.
// docs/TOOLTIPS.md: the same registry is the keyword list — every glossary term (COPY.md §2) and every concept has a tooltip of
// ≤ 10 words (`TIP`, eval/copy-budgets.json `tooltip`), its aliases (the words that mark it in a line), and optionally a live value
// read from the wire at open (`LIVE`: marks held, the worn stance's runs, the bank's balance). The marking, the plate and the fade
// are ./tips.ts.
import type { App } from "../app";
import { h } from "./dom";
import { icon } from "./skin";
import type { Packages } from "../engine/types";

export type Concept = "marks" | "renown" | "cage" | "vault" | "bones" | "oaths" | "bounty" | "waystone" | "fork" | "grudge";
/** The concept's icon (a packed one, else its glyph) and its first-time caption. */
const DEF: Record<Concept, { ico: string; glyph: string; cap: string }> = /* copy:callout */ {
  marks: { ico: "mark", glyph: "◆", cap: "upgrade tokens" },
  renown: { ico: "renown", glyph: "★", cap: "reputation" },
  cage: { ico: "vault", glyph: "▦", cap: "choose one item" },
  vault: { ico: "vault", glyph: "▣", cap: "stored gear" },
  bones: { ico: "it_bones", glyph: "☠", cap: "dead heir's gear" },
  oaths: { ico: "renown", glyph: "✠", cap: "goal for reward" },
  bounty: { ico: "gold", glyph: "✦", cap: "floor pays double" },
  waystone: { ico: "depth", glyph: "⌂", cap: "start deeper" },
  fork: { ico: "depth", glyph: "⑂", cap: "the other stairs" },
  grudge: { ico: "morgue", glyph: "⚔", cap: "his killer" },
};
const KEY = "riddle.concepts";
const SEEN_MS = 2000;
let seen: Set<string> | null = null;
const load = (): Set<string> => { if (seen) return seen; try { seen = new Set(JSON.parse(localStorage.getItem(KEY) ?? "[]") as string[]); } catch { seen = new Set(); } return seen; };
const mark = (c: Concept): void => { const s = load(); if (s.has(c)) return; s.add(c); try { localStorage.setItem(KEY, JSON.stringify([...s])); } catch { /* per-viewer */ } };
/** The concept's caption was shown to this viewer. */
export const conceptSeen = (c: Concept): boolean => load().has(c);
const io = typeof IntersectionObserver !== "undefined" ? new IntersectionObserver((es) => {
  for (const e of es) if (e.isIntersecting) { const el = e.target as HTMLElement; io!.unobserve(el); setTimeout(() => { if (el.isConnected) mark(el.dataset.concept as Concept); }, SEEN_MS); }
}) : null;
/** The concept's icon. */
export const conceptIcon = (c: Concept): HTMLElement => { const d = DEF[c]; const i = icon(d.ico, d.glyph); i.classList.add("concept-ico"); i.dataset.concept = c; return i; };
/** The first-time caption (`buys unlocks`) — empty once this viewer has seen it on screen. */
export function conceptCap(c: Concept, cap = DEF[c].cap): HTMLElement | "" {
  if (conceptSeen(c)) return "";
  // (the words are drawn from `data-cap` by CSS: the element's text stays the data's — tools and tests read the line it sits in)
  const el = h("small", { class: "concept-cap", "data-concept": c, "data-cap": cap, role: "note", "aria-label": cap });
  io?.observe(el);
  return el;
}
/** Icon and first-time caption together. */
export const conceptTag = (c: Concept): HTMLElement => h("span", { class: "concept", "data-concept": c }, conceptIcon(c), conceptCap(c));

// ---- docs/TOOLTIPS.md: the keyword registry (the concepts above plus the glossary's terms) ----

/** Every keyword: a concept (its icon and caption above) or a glossary term. */
export type Term = Concept | "bloodline" | "heir" | "gold" | "best" | "reach" | "package" | "stance" | "tactic" | "temperament" | "drill" | "scar" | "quest" | "track"
  | "pen" | "lever" | "bank" | "banked" | "returned" | "death" | "plateau" | "ends" | "priority" | "condition" | "action" | "forge" | "kennel" | "pack"
  | "price" | "v_gap" | "v_luck" | "v_rule" | "v_order" | "v_stall" | "v_route" | "v_repelled"
  | "works" | "worker" | "chest" | "scout" | "next" | "rank"   // Cut 30.5: the works tree
  | "rarity"   // run-clear: an item's rim colour
  | "lane" | "live" | "log" | "replay" | "away";   // RUNS_UI: the run lanes, the runs log
/** Cut 30.5: the works tree's terms — fragments of ≤ 4 words (eval/copy-budgets.json `node_tip`). */
const WORKS_TIP = /* copy:node_tip */ { works: "workers take chores over", worker: "hand for one chore", chest: "the haul waits here", scout: "sends him each rest", next: "the one next goal", rank: "worker's grade · from service" };
/** The tooltip's gloss: a fragment, ≤ 10 words with its live value (`tips.mjs` renders every one). A word that is another keyword is
 *  marked inside the plate (one level). */
export const TIP: Record<Term, string> = /* copy:tooltip */ {
  marks: "tokens from progression · buy unlocks and tactic levels", renown: "reputation from deeds · ranks award upgrade tokens", cage: "choose one item · loot preference selects automatically", vault: "stored gear · survives hero replacement",
  bones: "a dead heir's gear on his floor", oaths: "a goal for a reward", bounty: "this floor pays double gold", waystone: "start a run deeper",
  fork: "the other stairs, another place", grudge: "the foe that killed him",
  heir: "the family's hero now · the next takes over", gold: "earned on runs · spent in town", best: "deepest floor any heir reached",
  reach: "chance a run gets that deep", package: "fighting, healing and returning rules", stance: "when to fight, heal and return",
  tactic: "extra rules for specific fights · alongside the combat style", temperament: "personality · changes the hero's behavior", drill: "learned boss counter · enabled automatically · can be disabled",
  scar: "boss weaker each meeting", quest: "one goal · a reward when done", track: "one way the family grows", pen: "hero rules written by hand",
  lever: "the cheapest move against this death", bank: "deposits earn interest each night", banked: "home with all the loot",
  returned: "turned back early · keeps most loot", death: "gear left on the floor as bones", plateau: "every run home · none deeper",
  ends: "full haul, early return or death", priority: "the top rule that fits acts", condition: "when a rule may act", action: "what he does then",
  price: "change in chance, in percentage points · uncertain comparisons hidden", forge: "gold buys starting gear upgrades", kennel: "pets and tamed allies", pack: "supplies carried on the next run",
  v_gap: "no rule answered it", v_luck: "rules were fine · a bad roll", v_rule: "his own rule backfired", v_order: "right rule, ranked too low",
  v_stall: "stuck in a loop", v_route: "took the wrong stairs", v_repelled: "a boss drove him out",
  rarity: "how fine a find is · common up to legendary",
  ...WORKS_TIP,
  // RUNS_UI (docs/RUNS_UI.md)
  bloodline: "a persistent hero slot · keeps Legacy and upgrades",
  lane: "a hero's runs · live, resting or waiting", live: "the run going on now · watch or not",
  log: "every run · by absence · replays", replay: "the run again · same rolls", away: "runs while the game was shut",

};
/** The words that mark a term in a line (whole words, any case; the longest first). A term without aliases is marked only where a
 *  caller names it (`kw("v_luck", "luck")`, a host). */
export const ALIASES: Partial<Record<Term, string[]>> = /* copy:none */ {
  heir: ["heir"], marks: ["upgrade tokens", "marks"], renown: ["reputation", "renown"], package: ["packages", "package"], stance: ["combat style", "stance"], tactic: ["extra tactics", "tactics", "tactic"],
  temperament: ["personality", "temperament"], drill: ["boss counters", "drills", "drill", "drilled"], scar: ["scarred", "scars"], quest: ["quests", "quest"], track: ["tracks", "track"],
  pen: ["custom rules", "the pen"], lever: ["lever"], bank: ["bank"], banked: ["full haul", "banked"], returned: ["returned"], death: ["deaths"], plateau: ["plateau"], reach: ["floor chance", "reach"],
  ends: ["run outcomes", "ends"], priority: ["priority"], condition: ["condition"], action: ["action"], vault: ["stored gear", "vault", "storehouse"], bones: ["bones"], bounty: ["bounty"],
  waystone: ["waystone", "waystones"], grudge: ["grudge"], kennel: ["kennel"], forge: ["forge", "blacksmith"],
  works: ["works"], worker: ["workers", "worker"], chest: ["chest"], scout: ["scout"], rank: ["rank"], rarity: ["rarity"],
};
/** The plate's title: the term as the screen says it. */
export const TITLE: Partial<Record<Term, string>> = /* copy:label */ {
  price: "if equipped", gold: "gold", best: "best depth", death: "death",
  v_gap: "no rule", v_luck: "luck", v_rule: "rule", v_order: "order", v_stall: "stall", v_route: "route", v_repelled: "repelled",
  next: "next goal", log: "runs log",
  package: "tactics", stance: "combat style", tactic: "extra tactics", temperament: "personality", drill: "boss counters",
  marks: "upgrade tokens", renown: "reputation", cage: "loot choice", vault: "stored gear", oaths: "challenges", pen: "custom rules", ends: "run outcomes", reach: "floor chance", pack: "supplies",
  works: "workers", kennel: "companions",
};
export const termTitle = (t: Term): string => TITLE[t] ?? t;
/** The term's icon (a concept's, else the packed one named here). */
const ICO: Partial<Record<Term, [string, string]>> = /* copy:none */ { rarity: ["", "◈"], works: ["node_porter", ""], worker: ["node_porter", ""], chest: ["gold", "$"], scout: ["node_scout", ""], next: ["", "▸"], gold: ["gold", "$"], best: ["depth", ""], reach: ["depth", ""], bank: ["gold", "$"], banked: ["gold", ""], returned: ["bail", ""], death: ["morgue", "☠"], forge: ["forge", "⚒"], kennel: ["party", ""], pack: ["loadout", ""], stance: ["pkg_steady", ""], heir: ["", ""] };
export const termIcon = (t: Term): HTMLElement | null => { const c = (DEF as Record<string, { ico: string; glyph: string }>)[t]; const [ic, gl] = c ? [c.ico, c.glyph] : ICO[t] ?? ["", ""]; return ic || gl ? icon(ic, gl) : null; };
const pctOf = (x: number): string => `${Math.round(Math.max(0, Math.min(1, x)) * 100)}%`;
/** The live value at open, from the wire (none when the wire has none). All of it is the core's. */
export const LIVE: Partial<Record<Term, (app: App) => string | null>> = {
  heir: (a) => { const n = a.lineage.graveyard?.length ?? 0; return n ? /* copy:tooltip */ `${n} fallen` : null; },
  gold: (a) => `$${a.lineage.gold}`,
  // Cut 30.5: the works tree's — what waits in the chest, the hands hired, the sends toward the scout
  chest: (a) => { const W = a.lineage.tree; return W ? /* copy:tooltip */ `$${W.chest} waits` : null; },
  worker: (a) => { const n = (a.lineage.tree?.nodes ?? []).filter((x) => x.kind === "worker" && x.state === "done").length; return n ? /* copy:tooltip */ `${n} hired` : null; },
  works: (a) => { const W = a.lineage.tree; const l = W?.nodes.find((x) => x.id === W.lit); return l ? /* copy:tooltip */ `lit: ${l.name}` : null; },
  rank: (a) => { const W = a.lineage.tree, n = W?.nodes.find((x) => x.id === W.lit_rank); return n ? /* copy:tooltip */ `on offer: ${n.name} ${["", "I", "II", "III", "IV"][(n.rank ?? 1) + 1] ?? ""}` : null; },
  scout: (a) => { const W = a.lineage.tree, s = W?.nodes.find((x) => x.id === "scout"); return !s ? null : W!.auto_send ? /* copy:tooltip */ "hired · auto" : s.need ? /* copy:tooltip */ `${s.count ?? 0}/${s.need} sends` : null; },
  marks: (a) => `◆${a.lineage.marks}`,
  // RUNS_UI: the run under way, the runs held
  live: (a) => { const r = a.lineage.live; return r ? /* copy:tooltip */ `D${r.depth} · ${r.hp}/${r.max_hp} hp` : null; },
  lane: (a) => { const n = (a.lineage.runs ?? []).filter((r) => r.id > 0).length; return n ? /* copy:tooltip */ `${n} runs logged` : null; },
  log: (a) => { const n = (a.lineage.runs ?? []).filter((r) => r.id > 0).length; return n ? /* copy:tooltip */ `${n} runs` : null; },
  renown: (a) => /* copy:tooltip */ `★ rank ${a.lineage.rank ?? 0}`,
  best: (a) => `D${a.lineage.best_depth}`,
  reach: (a) => {
    const f = a.lastForecast; if (!f) return null;
    const next = Math.max(f.start ?? a.lineage.start ?? 1, a.lineage.best_depth + 1);
    const d = f.depths.find((x) => x.depth === next) ?? f.depths[f.depths.length - 1];
    return d ? `D${d.depth} ${pctOf(d.reach)}` : null;
  },
  package: (a) => { const P = a.lineage.packages; const p = P?.all.find((x) => x.id === P.stance); return p ? /* copy:tooltip */ `equipped: ${p.name} L${p.level}` : null; },
  stance: (a) => { const P = a.lineage.packages; const p = P?.all.find((x) => x.id === P.stance); return p ? `${p.name} L${p.level}${p.next_at ? /* copy:tooltip */ ` · ${p.runs}/${p.next_at} runs` : ""}` : null; },
  tactic: (a) => { const P = a.lineage.packages; return P && (P.tactic_slots ?? 0) > 0 ? /* copy:tooltip */ `${(P.tactics ?? []).length}/${P.tactic_slots} equipped` : null; },
  temperament: (a) => { const P = a.lineage.packages; return P?.temperament ? P.all.find((x) => x.id === P.temperament)?.name ?? null : null; },
  scar: (a) => { const s = (a.lineage.packages?.scars ?? []).reduce((m, [, v]) => Math.max(m, v), 0); return s ? `−${s}% hp` : null; },
  quest: (a) => { const q = a.lineage.town?.quest; return q ? (q.done ? /* copy:tooltip */ "done" : pctOf(q.progress)) : null; },
  track: (a) => { const ts = a.lineage.tracks ?? []; return ts.length ? /* copy:tooltip */ `${ts.reduce((n, t) => n + t.stages, 0)} stages` : null; },
  bank: (a) => { const T = a.lineage.town; return T ? /* copy:tooltip */ `$${T.bank} in · cap $${T.bank_cap}` : null; },
  vault: (a) => { const n = a.lineage.vault?.length ?? 0; return n ? /* copy:tooltip */ `${n} kept` : null; },
  // blind 1fb7786 (B: `custom rules · write hero rules · unlocks later` with five rules owned): the lock is live, never the gloss — the
  // chip's own word while locked, the rules written once open
  pen: (a) => {
    const P = a.lineage.packages; if (!P) return null;
    return P.pen_open || P.literal ? null : P.pen_needs?.length ? penWaits(P) : /* copy:tooltip */ "locked";
  },
};
/** The term a word marks (`packages` → package), or null. */
const ALIAS_OF = new Map<string, Term>(Object.entries(ALIASES).flatMap(([t, ws]) => (ws ?? []).map((w) => [w.toLowerCase(), t as Term] as [string, Term])));
export const termOf = (word: string): Term | null => ALIAS_OF.get(word.toLowerCase()) ?? null;
/** One regex for every alias, whole words, longest first. */
export const ALIAS_RE = new RegExp(/* copy:none */ `(?<![\\p{L}\\d])(${[...ALIAS_OF.keys()].sort((a, b) => b.length - a.length).map((w) => w.replace(/[.*+?^${}()|[\]\\]/g, "\\$&")).join("|")})(?![\\p{L}\\d])`, "giu");
export const TERMS = Object.keys(TIP) as Term[];

/** The lock chip's short word: `next report` when only the reveal waits, `after Bloat Mother` while the boss gate stands, else `locked`. */
export function penWaits(P: Pick<Packages, "pen_needs">): string {
  const first = P.pen_needs?.[0] ?? "";
  if (/^upcoming reports?$/i.test(first)) return /* copy:label */ "next report";
  const boss = /^meet (.+)$/i.exec(first);
  return boss ? /* copy:callout */ `after ${boss[1]}` : /* copy:label */ "locked";
}
