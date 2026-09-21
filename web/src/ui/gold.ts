// Cut 6 §1: the `gold` sheet — the last 80 gold movements (the core's GOLD_LEDGER_CAP), newest first, one per line: `+$50 returned D5` · `−$40 heal`
// (engine data, no prose). Cut 11 §5: opened from an exit line (the report, the death screen's ledger line) it shows that
// run's movements only — `−$40 heal · −$30 leash · +$36 returned D5 · +$8 salvage` — with an `all` chip for the whole ledger.
//
// Which lines are a run's: the ledger is oldest first and every exit lands one line whose `why` starts with its tier word
// (`returned D5`, `banked D8`, `died D3`, `lost thread D3`, `stalled D2`) at the exit's lineage tick; camp purchases sit between it and
// the previous exit's line (the core charges supplies at `start_run`, the fake at the buy, both after the previous exit),
// its salvage and a death's wake pay follow it at the same tick. So a run is the slice after the previous exit's tail up
// to its own tail — an index range, which also holds when `GoldLine.t` is a coarser clock than the run's ticks. The exit line is matched to the ledger by its tier
// and kept sum (`ExitLine.kept` = the line's delta), the newest match first; report exits (oldest first) claim matches
// from the newest backwards so two identical exits map to two different lines. An exit whose line has left the 80-line
// ledger opens the sheet unfiltered.
import type { App } from "../app";
import type { ExitLine, GoldLine } from "../engine/types";
import { h } from "./dom";
import { openSheet } from "./sheet";

/** The ledger word of the exit `x` ended with: read off the line's own lead (`returned $0 · … · stalled` is a return that
 *  kept nothing — its keep share alone would say `died`); a timed-out return's ledger line is `lost thread D3` or
 *  `stalled D2`. */
const tierWord = (x: ExitLine): RegExp => {
  const lead = /^(banked|returned|died)\b/.exec(x.text)?.[1] ?? (x.keep_pct >= 100 ? "banked" : x.keep_pct <= 0 ? "died" : "returned");
  return lead === "banked" ? /^banked\b/ : lead === "died" ? /^died\b/ : /^(returned|lost|stalled)\b/;
};
const isExit = (g: GoldLine): boolean => /^(returned|banked|died|lost|stalled)\b/.test(g.why);
/** An exit's tail: its salvage and a death's wake pay, at the exit's tick (a supply bought in camp at that tick is the next run's). */
const isTail = (g: GoldLine, exit: GoldLine): boolean => g.t === exit.t && /^(salvage|wake pay)\b/.test(g.why);

/** The ledger index range `[from, to]` of the run that ended with `x`; `skip` = how many newer matching exits to pass over
 *  (the report's i-th exit from the end). Undefined when the ledger no longer holds the exit. */
export function runRange(ledger: GoldLine[], x: ExitLine, skip = 0): [number, number] | undefined {
  const word = tierWord(x);
  let seen = 0;
  for (let i = ledger.length - 1; i >= 0; i--) {
    const g = ledger[i];
    if (!isExit(g) || !word.test(g.why) || g.delta !== x.kept) continue;
    if (seen++ < skip) continue;
    // back to the previous exit, then past that exit's own tail (its salvage, a death's wake pay: same tick, after it)
    let from = i; while (from > 0 && !isExit(ledger[from - 1])) from--;
    if (from > 0) { const prev = ledger[from - 1]; while (from < i && isTail(ledger[from], prev)) from++; }
    // this exit's tail lands after its line
    let to = i; while (to + 1 < ledger.length && isTail(ledger[to + 1], g)) to++;
    return [from, to];
  }
  return undefined;
}

/** Two exits that would match the same ledger line (tier and kept sum). */
const sameExit = (a: ExitLine, b: ExitLine): boolean => tierWord(a).source === tierWord(b).source && a.kept === b.kept;

/** Open the gold sheet; with `only`, filtered to that exit's run (`newer` = the exits after it in the same report, so a
 *  repeated `returned $0` claims its own line). */
export function openGoldSheet(app: App, only?: ExitLine, newer: ExitLine[] = []): void {
  const L = app.lineage; const ledger = L.gold_ledger ?? [];
  const range = only ? runRange(ledger, only, newer.filter((y) => sameExit(y, only)).length) : undefined;
  openSheet(() => {
    const fmt = (d: number): string => `${d < 0 ? "−" : d > 0 ? "+" : ""}$${Math.abs(d)}`;
    const list = h("div", { class: "gold-lines" });
    const body = h("div", { class: "sheet-body gold-sheet" });
    const paint = (filtered: boolean): void => {
      const lines = (filtered && range ? ledger.slice(range[0], range[1] + 1) : ledger).slice().reverse();
      body.dataset.filter = filtered && range ? `${range[0]}-${range[1]}` : "";
      const all = filtered && range ? h("button", { class: "chip mini", onclick: () => paint(false) }, /* copy:button */ "all") : "";
      body.replaceChildren(
        h("div", { class: "label row-label" }, /* copy:label */ "gold", " ", h("span", { class: "num gold" }, `$${L.gold}`), all),
        list);
      list.replaceChildren(
        ...lines.map((g) => h("div", { class: `lrow num${g.delta < 0 ? " down" : g.delta > 0 ? " up" : ""}`, "data-t": g.t }, h("span", { class: "k" }, fmt(g.delta)), h("span", { class: "why" }, g.why.replace(/_/g, " ")))),
        lines.length ? "" : h("div", { class: "lrow num dim" }, "·"));
    };
    paint(!!range);
    return body;
  });
}
