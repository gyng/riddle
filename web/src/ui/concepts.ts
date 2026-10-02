// Cut 29 (owner, docs/COPY.md §4 blocker 1: "a world concept one screen cannot define" — `from cages`, `keep for heirs`, `bones to
// recover`, `◆ 21`, `★ 1`, `oaths`, `bounty`, the fork's other lane): each world concept has one icon, used wherever it appears, and a
// caption of ≤ 3 words the first time a viewer sees it — once seen (on screen ~2 s) the icon stands alone. Per-viewer memory
// (localStorage); without it the caption shows every time, which only costs words.
import { h } from "./dom";
import { icon } from "./skin";

export type Concept = "marks" | "renown" | "cage" | "vault" | "bones" | "oaths" | "bounty" | "waystone" | "fork" | "grudge";
/** The concept's icon (a packed one, else its glyph) and its first-time caption. */
const DEF: Record<Concept, { ico: string; glyph: string; cap: string }> = /* copy:callout */ {
  marks: { ico: "mark", glyph: "◆", cap: "buys unlocks" },
  renown: { ico: "renown", glyph: "★", cap: "rank from deeds" },
  cage: { ico: "vault", glyph: "▦", cap: "pick one loot" },
  vault: { ico: "vault", glyph: "▣", cap: "kept between heirs" },
  bones: { ico: "morgue", glyph: "☠", cap: "dead heir's gear" },
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
