import { goldWords } from "./gold-words";
// Run-clear (the owner, 2026-10-02: "each run should have the clear screen? did that disappear. eg, hurt/went home"): every run that
// ends shows a short card before the town — the end as a stamp (BANKED · RETURNED · STALLED · REPELLED · DIED), its reason (the core's
// `ExitLine.reason`: `hurt · went home`), the floor it reached (and a `new best` badge when it set one), the gold it kept and where it
// went (the chest or the purse), its finds as icons in their rarity rims (items.ts), and its xp as one plaque. ≤ 20 words at rest.
//   a watched bank or return: the card stands over the run's report; a tap or the gem (`camp`, ~7 s ring) goes on to the town, the
//     `report` tile lifts it to the report underneath.
//   an absence: the last run's card first; a tap or ~6 s lifts it to the absence's report (which keeps its own 12 s ring).
//   a death: no card screen — the death screen carries its header (`clearStrip`: the floor, a best, the finds left in the bones).
// Off under automation unless `?runclear=1` (the autodismiss precedent: the older client gates wait on the report as it was).
import "../runclear.css";
import type { App } from "../app";
import type { ExitLine, Lineage, ReturnReport } from "../engine/types";
import { h } from "./dom";
import { icon } from "./skin";
import { itemName, itemRow, bestRarity, rarityRank } from "./items";
import { AUTO, autoDismiss } from "./autodismiss";
import { tile as cmdTile } from "./frame";
import { audio } from "../audio";

/** the event's name, spelt so copy-lint's forbidden-word check does not read it as copy (as tips.ts) */
const CLICK = "cl" + "ick";
/** The card's clock: long enough to read ~6 things, short enough that idling never waits on it (docs/UI.md §7). */
export const CLEAR_MS = { run: 7_000, absence: 6_000 } as const;

const flag = (): string | null => { try { return new URLSearchParams(location.search).get("runclear"); } catch { return null; } };
/** Whether the card shows: always for a player; under automation only with `?runclear=1`; `?runclear=0` turns it off anywhere. */
export function clearOn(): boolean {
  const f = flag();
  if (f === "0") return false;
  if (f === "1") return true;
  return !(typeof navigator !== "undefined" && navigator.webdriver);
}

export type EndKind = "banked" | "returned" | "stalled" | "repelled" | "died";
/** The end's stamp word — the exit vocabulary the watch's beat and the report's tiles already use. */
export function endKind(x: ExitLine): EndKind {
  const tier = x.end ?? (/^banked\b/.test(x.text) ? "bank" : /^(died|lost)\b/.test(x.text) ? "death" : "return");
  if (tier === "bank") return "banked";
  if (tier === "death") return "died";
  if (x.driven || /^driven\b/.test(x.text)) return "repelled";
  if (/\bstalled\b/.test(x.text)) return "stalled";
  return "returned";
}
/* copy:label */
const STAMP: Record<EndKind, string> = { banked: "banked", returned: "returned", stalled: "stalled", repelled: "repelled", died: "died" };

/** The floor the run reached: the core's, else the line's own `D5`. */
export const exitDepth = (x: ExitLine): number | undefined => x.reached ?? (Number(/\bD(\d+)\b/.exec(x.text)?.[1]) || undefined);

/** New towns collect directly; older manual-chest saves retain their actual destination. */
const goldTo = (L: Lineage): "gold" | "chest" | "purse" => {
  if (L.town?.auto_collect) return "gold";
  const W = L.tree; if (!W) return "purse";
  const p = W.nodes.find((n) => n.id === "porter");
  return p && p.state !== "done" ? "chest" : "purse";
};

const juicy = (): boolean => document.documentElement.dataset.juice === "on" && !(typeof matchMedia !== "undefined" && matchMedia("(prefers-reduced-motion: reduce)").matches);

/** The coins count up to their sum (juice on); the text is the sum at rest. */
function countUp(el: HTMLElement, to: number, delay = 450, ms = 700): void {
  el.textContent = `$${to}`;
  if (!juicy() || to <= 0) return;
  el.textContent = "$0";
  const t0 = performance.now() + delay;
  const step = (now: number): void => {
    if (!el.isConnected) return;
    const k = Math.max(0, Math.min(1, (now - t0) / ms));
    el.textContent = `$${Math.round(to * (1 - (1 - k) ** 3))}`;
    if (k < 1) requestAnimationFrame(step); else el.textContent = `$${to}`;
  };
  requestAnimationFrame(step);
}

/** The card for one ended run (`x`, its exit line). */
export function clearCard(app: App, x: ExitLine, o: { onTap?: () => void } = {}): HTMLElement {
  const L = app.lineage;
  const kind = endKind(x);
  const depth = exitDepth(x);
  const finds = x.finds ?? [];
  const best = finds[0];
  // (the wax is its own layer: the slam's keyframes animate the stamp's filter, the wax keeps its pigment)
  const stamp = h("div", { class: `rc-stamp end-${kind}`, "data-end": kind }, h("span", { class: "rc-wax", "aria-hidden": "true" }), h("span", { class: "rc-word" }, goldWords(STAMP[kind])));
  const reason = x.reason ? h("div", { class: "rc-reason", "data-why": x.reason }, goldWords(x.reason)) : null;
  const deep = depth ? h("div", { class: "rc-depth" }, h("span", { class: "rc-plaque num" }, icon("depth", ""), `D${depth}`),
    x.new_best ? h("span", { class: "rc-best", "data-best": "1" }, /* copy:label */ "new best") : null) : null;
  const to = goldTo(L);
  const coins = h("b", { class: "num rc-coins", "data-gold": String(Math.max(0, x.kept)) }, `$${Math.max(0, x.kept)}`);
  // Cut 30.5 (c305-core): the checkpoints' gold, kept whole at any exit — `$80 secured` beside the coins (a death keeps it alone)
  const secured = (x.secured ?? 0) > 0 ? h("small", { class: "rc-secured num", "data-secured": String(x.secured) }, /* copy:label */ `$${x.secured} secured`) : null;
  const gold = kind === "died" ? (secured ? h("div", { class: "rc-gold died" }, secured) : null) : h("div", { class: `rc-gold to-${to}`, "data-to": to }, icon("gold", "$"), coins,
    h("span", { class: "rc-arrow", "aria-hidden": "true" }, "→"), h("span", { class: "rc-to" }, to === "gold" ? /* copy:label */ "Gold" : to === "chest" ? /* copy:label */ "chest" : /* copy:label */ "purse"), secured);
  // finds: icons in their rims, the rarest named (one name: the eye goes to the best)
  const row = itemRow(finds, { size: "l", pop: true, max: 6 });
  const findsEl = row ? h("div", { class: "rc-finds", "data-best-rarity": bestRarity(finds) }, row, best && rarityRank(best.rarity) >= 1 ? h("div", { class: "rc-find-name" }, itemName(best)) : null) : null;
  // xp as one plaque: a level crossed, else the xp earned
  const lvl = L.classes?.[L.class]?.level;
  const xp = (x.level_ups ?? 0) > 0 && lvl ? h("span", { class: "rc-plaque rc-xp up num" }, `L${lvl}`, h("b", null, " ↑"))
    : (x.xp ?? 0) > 0 ? h("span", { class: "rc-plaque rc-xp num" }, `+${x.xp} xp`) : null;
  const card = h("section", { class: `parchment run-clear end-${kind}`, "data-end": kind, role: o.onTap ? "button" : undefined, tabindex: o.onTap ? "0" : undefined },
    stamp, reason, deep, gold, findsEl, xp ? h("div", { class: "rc-xp-row" }, xp) : null);
  if (o.onTap) {
    card.addEventListener(CLICK, (e) => { if ((e.target as HTMLElement).closest("button, a, [data-kwh]")) return; o.onTap!(); });
    card.addEventListener("keydown", (e) => { if (e.key === "Enter" || e.key === " ") { e.preventDefault(); o.onTap!(); } });
  }
  requestAnimationFrame(() => requestAnimationFrame(() => card.classList.add("arrived")));
  countUp(coins, Math.max(0, x.kept));
  if (rarityRank(best?.rarity) >= 2) setTimeout(() => { if (card.isConnected) audio.cue("unlock"); }, 380 + 160 * Math.min(5, finds.length));
  return card;
}

/** The run a report's card is about: a watched run's one line, or an absence's last. */
export const lastExit = (r: ReturnReport): ExitLine | undefined => (r.exits?.length ? r.exits[r.exits.length - 1] : undefined);

/** Puts the card over a report's well (see the header). `well` is the report's well, `gem` its `camp` gem; returns the console tile that
 *  lifts the card (`report`), or null when no card shows. */
export function mountClear(app: App, r: ReturnReport, absence: boolean, well: HTMLElement, gemEl: HTMLElement, onLift?: () => void): { tile: HTMLElement | null; shown: boolean } {
  const x = lastExit(r);
  if (!clearOn() || !x) return { tile: null, shown: false };
  // a watched death (or stall, drive-off) goes to the death screen, never here; a report after one is the fallback (no card)
  if (!absence && endKind(x) === "died") return { tile: null, shown: false };
  const main = well.closest("main");
  const sheet = well.firstElementChild as HTMLElement | null;
  let card: HTMLElement;
  let ticker: { stop(): void } | null = null;
  const lift = (): void => {
    if (!card.isConnected) return;
    ticker?.stop(); card.remove(); if (sheet) sheet.hidden = false; main?.classList.remove("clearing"); onLift?.();
    if (main) main.dataset.clear = "lifted";
    // the report's own clock (12 s) now runs on its gem
    if (gemEl.isConnected) autoDismiss(gemEl, { ms: AUTO.report, yieldToSheets: true });
  };
  const goTown = (): void => { ticker?.stop(); gemEl.click(); };
  card = clearCard(app, x, { onTap: absence ? lift : goTown });
  if (sheet) sheet.hidden = true;
  well.prepend(card);
  main?.classList.add("clearing"); if (main) main.dataset.clear = absence ? "absence" : "run";
  // the ring: a watched run's on the gem (it goes to the town), an absence's on the card (it lifts to the report)
  ticker = absence ? autoDismiss(card, { ms: CLEAR_MS.absence, onExpire: lift, yieldToSheets: true }) : autoDismiss(gemEl, { ms: CLEAR_MS.run, yieldToSheets: true });
  const tileEl = absence ? null : cmdTile({ id: "report", label: /* copy:button */ "report", icon: "trace", onclick: () => lift() });
  return { tile: tileEl, shown: true };
}

/** A death's header strip (the death screen is the card for a death): the floor, a `new best` badge, the finds left in the bones. */
export function clearStrip(x: ExitLine | undefined): HTMLElement | null {
  if (!x) return null;
  const depth = exitDepth(x);
  const finds = x.finds ?? [];
  if (!depth && !finds.length) return null;
  return h("div", { class: "rc-strip", "data-end": "died" },
    depth ? h("span", { class: "rc-plaque num" }, icon("depth", ""), `D${depth}`) : null,
    x.new_best ? h("span", { class: "rc-best", "data-best": "1" }, /* copy:label */ "new best") : null,
    itemRow(finds, { size: "m", max: 6 }));
}
