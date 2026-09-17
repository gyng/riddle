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
import { verbLabel } from "./tokens";

/** What a chain needs beyond the trace: the rules that ran (verb labels: `verbs` as the engine wrote them, else `rows` —
 *  the editing copy, right after the run), the run (replay), extra links. */
export type ChainCtx = { rows?: Row[]; verbs?: string[]; runId?: number; chain?: Because[];
                         provenance?: boolean };   // §3: also list the trace's provenance log (the exit sheet; the death screen keeps to its chain)

/** Cut 11 §2: the verbs of the rules that ran, off a morgue's `R1 HP<40% → drink heal` lines (core and fake write them), so an
 *  old death from the chronicle labels its chain with its own rules, not the current set's. Undefined without such lines. */
export function morgueVerbs(morgue: string | undefined): string[] | undefined {
  const out: string[] = [];
  for (const m of (morgue ?? "").matchAll(/^\s*R(\d+) .* → (.+?)\s*$/gm)) out[Number(m[1]) - 1] = m[2];
  return out.length ? out : undefined;
}

const sameLink = (a: Because, b: Because): boolean => a.text === b.text && a.t === b.t;

/** The chain of a trace's last turn, or null when nothing on the wire carries a `because`. */
export function chainOf(trace: Trace, ctx: ChainCtx = {}): HTMLElement | null {
  const last = trace.turns[trace.turns.length - 1];
  const rows = last?.rows ?? [];
  const extra = [...(ctx.chain ?? []), ...(ctx.provenance ? trace.provenance ?? [] : [])];
  if (!rows.some((r) => r.because) && !extra.length) return null;
  const shown: Because[] = [];
  const verbOf = (i: number): string | undefined => { const v = ctx.verbs?.[i]; if (v) return v; const r = ctx.rows?.[i]; return r ? verbLabel(r.verb) : undefined; };
  const lines: HTMLElement[] = rows.map((r) => {
    const verb = verbOf(r.row);
    const line = h("div", { class: "chain-row" },
      h("span", { class: "r" }, `R${r.row + 1}`, verb ? h("small", { class: "dim" }, ` ${verb}`) : ""),
      h("span", { class: "why" }, r.why));
    if (r.because) { shown.push(r.because); line.append(...link(r.because, ctx.runId)); }
    return line;
  });
  if (last && last.row >= 0) lines.push(h("div", { class: "chain-row fired" },
    h("span", { class: "r" }, `R${last.row + 1}`, h("small", { class: "dim" }, ` ${verbLabel(last.verb)}`)),
    h("span", { class: "why" }, /* copy:label */ "fired")));
  for (const b of extra) {
    if (shown.some((s) => sameLink(s, b))) continue;
    shown.push(b);
    lines.push(h("div", { class: "chain-row extra" }, ...link(b, ctx.runId)));
  }
  return h("div", { class: "chain num" }, ...lines);
}

/** `← den took the heal, D3` then `[watch]` when the run's replay holds the tick, else `t2140` (`D3 · t2140` when the text
 *  does not name the floor). */
function link(b: Because, runId: number | undefined): (HTMLElement | string)[] {
  const out: (HTMLElement | string)[] = [h("span", { class: "because" }, "← ", b.text)];
  if (/^never /.test(b.text)) return out;   // `never found` / `never met`: there is no moment (the core stamps the death tick)
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
