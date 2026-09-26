// Cut 11 §2 — the chain under a trace: one line per row of the killing turn that did not fire, the reason, and the
// `because` behind it (engine data, ≤ 8 words): `R1 drink heal · no item ← den took the heal, D3 [watch]`. The `watch` chip
// scrubs the run's replay to the tick (runlog.ts + replay.ts) when this session still holds the run's events; otherwise
// the line ends `D3 · t2140`. The fired row closes the chain (`R3 attack · fired`). Chain entries beyond the rows follow as
// bare links: `Death.chain` on the death screen, the whole provenance log (`Trace.provenance`, §3) on an exit sheet. Without
// any `because` on the wire (an older core) the Cut 6 §3 `R1 why · R2 why` line stands.
import type { Because, Row, Trace } from "../engine/types";
import { h } from "./dom";
import { lastRun, replayable } from "./runlog";
import { openReplay } from "./replay";
import { verbLabel, whyGloss } from "./tokens";

/** Cut 23 §3: a reason (`no use`, `no path`) with a gloss is a tap target — the tap adds the gloss after it (`no use · no effect now`). */
export function whySpan(why: string): HTMLElement {
  const g = whyGloss(why);
  if (!g) return h("span", { class: "why" }, why);
  const b: HTMLButtonElement = h("button", { class: "why has-gloss", onclick: (e: Event) => { e.stopPropagation(); if (b.querySelector(".gloss")) return; b.appendChild(h("small", { class: "gloss" }, ` · ${g}`)); } }, why);
  return b;
}

/** What a chain needs beyond the trace: the rules that ran (verb labels: `verbs` as the engine wrote them, else `rows` —
 *  the editing copy, right after the run), the run (replay), extra links. */
export type ChainCtx = { rows?: Row[]; verbs?: string[]; runId?: number; chain?: Because[];
                         provenance?: boolean;     // §3: also list the trace's provenance log (the exit sheet; the death screen keeps to its chain)
                         home?: boolean;           // Cut 14: the trace of a bank/return — its last row is the way home, not the killing blow (QA on 56f2a1d: painted red)
                         depth?: number;           // QA a946e04: the floor the trace's turns are on (a death's) — a present-state blocker stamped on another floor is re-stamped to its turn here
                         window?: number;          // Cut 21 §3: the turns the table shows (its last N) — their row reasons join the chain, not only the last turn's
                         onRow?: (row: number) => void };   // QA 912e135 (qaW: `R1 unknown item` and the trace rows inert on the death screen): a row's name opens it

/** QA 912e135: a row's name (`R1` and its verb) — a button that opens the row when the screen can (`ChainCtx.onRow`), else text. */
export function rowRef(row: number, verb: string | undefined, onRow?: (row: number) => void): HTMLElement {
  const kids = [`R${row + 1}`, verb ? h("small", { class: "dim" }, ` ${verb}`) : ""];
  return onRow ? h("button", { class: "r row-link", "data-row": row, onclick: (e: Event) => { e.stopPropagation(); onRow(row); } }, ...kids) : h("span", { class: "r" }, ...kids);
}

/** Cut 11 §2: the verbs of the rules that ran, off a morgue's `R1 HP<40% → drink heal` lines (core and fake write them), so an
 *  old death from the chronicle labels its chain with its own rules, not the current set's. Undefined without such lines. */
export function morgueVerbs(morgue: string | undefined): string[] | undefined {
  const out: string[] = [];
  for (const m of (morgue ?? "").matchAll(/^\s*R(\d+) .* → (.+?)\s*$/gm)) out[Number(m[1]) - 1] = m[2];
  return out.length ? out : undefined;
}

/** QA e75ec29 (Q: "`R1 to corridor · no path ← chase given up`; the rule was `to corridor`, nothing chased"): a foe-reach blocker (the
 *  core's `path_blocker`: a foe given up, fleeing, across water, out of reach) answers a row that goes AT a foe — never a move row's
 *  `no path` (`to corridor`, `retreat`, `return`, `descend`): that link is left off (the row's own reason stands). */
const FOE_BLOCKER = /^(chase given up|foe fleeing|foe across water|no way to it)$/;
export const foeBlockerOnMove = (verb: string | undefined, because: string): boolean =>
  FOE_BLOCKER.test(because.trim()) && !!verb && !/^(attack|shoot|throw|tame|zap|hit|strike|pack|kite|boss)\b/.test(verb.trim());
/** QA a946e04 (T: a D4 death's `R2 attack nearest · no target ← chase given up` opened `CHASE GIVEN UP · D2 · t2640`, a jackal 5 000
 *  ticks earlier): a way blocker is the state at that turn (the core's `path_blocker` / `home_blocker`), but the core reused its last
 *  entry of the same words, from another floor. A blocker link stamped on a floor other than the trace's is the turn's own moment. */
const PRESENT = /^(chase given up|foe fleeing|foe across water|no way to it|captive chained the way|gas cloud, this room|bloats seal the stair|water in the way|foes on every side|foes hold the way|ally in the way|.+ holds the way)$/;
export function restamp(b: Because, t: number, depth: number | undefined): Because {
  return depth !== undefined && b.depth > 0 && b.depth !== depth && PRESENT.test(b.text.trim()) ? { ...b, t, depth } : b;
}
const sameLink = (a: Because, b: Because): boolean => a.text === b.text && a.t === b.t;
/** The provenance log under an exit sheet's chain is capped: the links the chain's rows carry, then the last this many by
 *  tick, then `· N earlier` (QA on 50bb162: "R4 fired followed by ~70 `← found X on Dn` lines"). */
export const PROVENANCE_SHOW = 8;
/** Cut 21 §3: the earlier ticks' reason lines shown at most (the newest), the rest behind `· N earlier`. */
export const TICKS_SHOW = 8;

/** Cut 21 §3 (AF: "why R3 bank (hp<30%, 9/42) didn't fire earlier isn't explained — only the last tick's per-row reasons"): the row
 *  reasons of the turns before the last, one line per row per stretch of turns with the same reason and `because`
 *  (`t812–830 R3 bank · no way ← foes held the way`), oldest first; a reason the last turn repeats for its row is left to the last
 *  turn's line. Engine data: each `TraceTurn.rows[].why` and its `because`. */
export type TickLine = { from: number; to: number; row: number; why: string; because?: Because };
export function tickLines(turns: Trace["turns"]): TickLine[] {
  const last = turns[turns.length - 1];
  const lastOf = new Map((last?.rows ?? []).map((r) => [r.row, `${r.why}|${r.because?.text ?? ""}`]));
  const open = new Map<number, TickLine & { key: string; at: number }>(), out: (TickLine & { key: string; at: number })[] = [];
  turns.slice(0, -1).forEach((t, i) => {
    const seen = new Set<number>();
    for (const r of t.rows ?? []) {
      seen.add(r.row);
      const key = `${r.why}|${r.because?.text ?? ""}`, cur = open.get(r.row);
      if (cur && cur.key === key && cur.at === i - 1) { cur.to = t.t; cur.at = i; continue; }
      const line = { from: t.t, to: t.t, row: r.row, why: r.why, because: r.because, key, at: i };
      open.set(r.row, line); out.push(line);
    }
    for (const [row, l] of open) if (!seen.has(row) && l.at !== i) open.delete(row);
  });
  // a stretch that runs into the last turn with the reason the last turn states again is the last turn's line
  return out.filter((l) => !(l.at === turns.length - 2 && lastOf.get(l.row) === l.key)).map(({ from, to, row, why, because }) => ({ from, to, row, why, because }));
}

/** The chain of a trace's last turn, or null when nothing on the wire carries a `because`. */
export function chainOf(trace: Trace, ctx: ChainCtx = {}): HTMLElement | null {
  const last = trace.turns[trace.turns.length - 1];
  const rows = (last?.rows ?? []).map((r) => r.because ? { ...r, because: restamp(r.because, last.t, ctx.depth) } : r);
  // Cut 21 §3: the earlier turns' reasons (the table's window) — each tick's `because`, not only the last's
  const ticks = tickLines(ctx.window !== undefined ? trace.turns.slice(-ctx.window) : trace.turns).map((l) => l.because ? { ...l, because: restamp(l.because, l.from, ctx.depth) } : l);
  const rowLinks = [...rows.flatMap((r) => r.because ? [r.because] : []), ...ticks.flatMap((l) => l.because ? [l.because] : [])];
  // QA 0c6e126 (qaZ: under `R2 return · fired` the chain listed `swapped for the summon ally · found … · chase given up` — none of them
  // made hp < 40%): a way home's trace keeps to its rows' own reasons; the run's provenance log is no cause of the row that fired
  let prov = ctx.provenance && !ctx.home ? (trace.provenance ?? []).filter((b) => !rowLinks.some((s) => sameLink(s, b))) : [];
  let earlier = 0, older: Because[] = [];
  if (prov.length > PROVENANCE_SHOW) { prov = [...prov].sort((a, b) => a.t - b.t); earlier = prov.length - PROVENANCE_SHOW; older = prov.slice(0, earlier); prov = prov.slice(-PROVENANCE_SHOW); }
  const extra = [...(ctx.chain ?? []).map((b) => restamp(b, last?.t ?? b.t, ctx.depth)), ...prov];
  if (!rows.some((r) => r.because) && !ticks.some((l) => l.because) && !extra.length) return null;
  const shown: Because[] = [];
  const verbOf = (i: number): string | undefined => { const v = ctx.verbs?.[i]; if (v) return v; const r = ctx.rows?.[i]; return r ? verbLabel(r.verb) : undefined; };
  const tickLine = (l: TickLine): HTMLElement => {
    const verb = verbOf(l.row);
    const line = h("div", { class: "chain-row tick", "data-row": l.row },
      h("span", { class: "at-t dim" }, l.to > l.from ? `t${l.from}–${l.to}` : `t${l.from}`),
      rowRef(l.row, verb, ctx.onRow),
      whySpan(l.why));
    if (l.because) { const dup = shown.some((s) => s.text === l.because!.text); if (!dup) shown.push(l.because); if (!dup && !foeBlockerOnMove(verb, l.because.text)) line.append(...link(l.because, ctx.runId)); }
    return line;
  };
  const tickOlder = ticks.length > TICKS_SHOW ? ticks.slice(0, ticks.length - TICKS_SHOW) : [];
  const tickEls = ticks.slice(tickOlder.length).map(tickLine);
  const lines: HTMLElement[] = rows.map((r) => {
    const verb = verbOf(r.row);
    const line = h("div", { class: "chain-row" },
      rowRef(r.row, verb, ctx.onRow),
      whySpan(r.why));
    // QA 778fa1b (qaV: `← found heal on D1 · watch` first and last): a reason an earlier tick's line already linked is not linked again
    if (r.because) { const dup = shown.some((s) => s.text === r.because!.text); shown.push(r.because); if (!dup && !foeBlockerOnMove(verb, r.because.text)) line.append(...link(r.because, ctx.runId)); }
    return line;
  });
  // QA 524827b (qaAA: `R4 attack nearest · fired · ← cowardly ran first` read as why R4 fired): the chain's own links are the earlier
  // moments — they go before the last turn's lines, and the fired row closes the chain, never followed by a bare `←`
  const extraEls: HTMLElement[] = [];
  for (const b of extra) {
    if (shown.some((s) => sameLink(s, b) || s.text === b.text)) continue;
    shown.push(b);
    extraEls.push(h("div", { class: "chain-row extra" }, ...link(b, ctx.runId)));
  }
  lines.unshift(...extraEls);
  if (last && last.row >= 0) lines.push(h("div", { class: "chain-row fired" },
    rowRef(last.row, verbLabel(last.verb), ctx.onRow),
    h("span", { class: "why" }, /* copy:label */ "fired")));
  // QA e75ec29 (R: "`· 20 earlier` … does nothing when tapped"): the older links unfold in place
  const tickMore: HTMLElement | "" = tickOlder.length ? h("button", { class: "chain-row tick earlier dim", onclick: (e: Event) => { (e.currentTarget as HTMLElement).replaceWith(...tickOlder.map(tickLine)); } }, /* copy:button */ `· ${tickOlder.length} earlier`) : "";
  const box = h("div", { class: "chain num" }, tickMore, ...tickEls, ...lines);
  if (earlier > 0) {
    const more: HTMLButtonElement = h("button", { class: "chain-row extra earlier dim", onclick: () => {
      more.replaceWith(...older.filter((b) => !shown.some((x) => sameLink(x, b))).map((b) => h("div", { class: "chain-row extra" }, ...link(b, ctx.runId))));
    } }, /* copy:button */ `· ${earlier} earlier`);
    box.appendChild(more);
  }
  return box;
}

/** `← den took the heal, D3` then `[watch]` when the run's replay holds the tick, else `t2140` (`D3 · t2140` when the text
 *  does not name the floor). */
function link(b: Because, runId: number | undefined): (HTMLElement | string)[] {
  const out: (HTMLElement | string)[] = [h("span", { class: "because" }, "← ", b.text)];
  // `never found` / `never met`: there is no moment (the core stamps the death tick); QA 912e135 (qaX: `← repeat short · watch` opened the
  // killing blow): nor for a camp event (`repeat short`, the send's re-pack the purse could not pay)
  if (/^(never |repeat short\b)/.test(b.text)) return out;
  if (replayable(runId, b)) {
    const log = lastRun()!;
    out.push(h("button", { class: "chip mini link", "data-t": b.t, onclick: () => openReplay(log, b) }, /* copy:button */ "watch"));
  } else {
    // the depth, unless the text already names it (`den took the heal, D3`), then the tick
    const named = new RegExp(/* copy:none */ `\\bD${b.depth}\\b`).test(b.text);
    out.push(h("small", { class: "at dim" }, `${b.depth > 0 && !named ? `D${b.depth} · ` : ""}t${b.t}`));
  }
  return out;
}
