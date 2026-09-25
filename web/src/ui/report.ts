// Return report: learned · bests · found · deaths · pending · reel · marks. Delta, not totals.
// Cut 10 §3: the exit tiles read `banked · returned · deaths`, `returned` first when it is the larger; each exit line leads
// with its tier and the kept sum (`returned $61`) before the engine's arithmetic; a lost companion reads `jackal Ashar fell`;
// after an absence the tiles fade in (the merged report is complete by the time this mounts).
// Cut 13 §3: the night's ledger — a `spent` section beside `salvaged` (`heal ×16 · −$640`, `ReturnReport.spent`) and one dim
// `gold` line under the tiles that reconciles the header's delta: `+$412 banked · +$96 returned · +$45 salvage · −$640 spent`
// (the banked / returned sums are the exit lines'; numbers only, a piece shows only when it is not zero).
// Cut 16 §1: the depths the lineage has picked clean (`ReturnReport.picked`) as one dim line under the gold line (`D3 · D4 · picked clean`).
// Cut 14 §4: every exit's `trace` chip carries its exit (`D5 · died · trace`; the depth off the ledger line the exit claims, else
// off the line's own text) — rater S: "the seventh unlabelled TRACE button"; the stalled tile carries what the stalls cost
// (`2 STALLED · $161 lost`, the stalled lines' `carried`); the `R1 fired n of m runs` lines go to `app.rowFires`.
import { openForge } from "./forge";
import type { App, Mounted } from "../app";
import type { Counter, ExitLine, Lineage, ReturnReport } from "../engine/types";
import { h, items, spanOf } from "./dom";
import { patchRows } from "./patches";
import { exitExtras, wakeShown } from "./death";
import { openUnlockSheet, priceLabel, visible, withRowsGate } from "./unlocks";
import { lostLabel, noteText, rowLabel } from "./tokens";
import { traceChip } from "./trace";
import { openGoldSheet, runRange } from "./gold";
import { gem, portrait, renderBar, renderConsole, tile as cmdTile } from "./frame";
import { icon } from "./skin";
import { revealed } from "./reveal";
import { openLedger } from "./party";

const EXITS_SHOW = 8;

/** Cut 14 §4: the floor an exit ended on — the ledger's exit line it claims (`returned D5`, the gold sheet's own match), else
 *  the line's own `bones: 7 items on D5`; undefined when neither knows. `newer` = the exits after it in the same report. */
export function exitDepth(app: App, x: ExitLine, newer: ExitLine[] = []): number | undefined {
  const ledger = app.lineage.gold_ledger ?? [];
  const same = (a: ExitLine, b: ExitLine): boolean => a.kept === b.kept && /^(banked|returned|died)/.exec(a.text)?.[1] === /^(banked|returned|died)/.exec(b.text)?.[1];
  const range = runRange(ledger, x, newer.filter((y) => same(y, x)).length);
  const why = range ? ledger.slice(range[0], range[1] + 1).map((g) => g.why).find((w) => /^(returned|banked|died|lost|stalled)\b.*\bD\d+/.test(w)) : undefined;
  const m = /\bD(\d+)\b/.exec(why ?? "") ?? /\bon D(\d+)\b/.exec(x.text);
  return m ? Number(m[1]) : undefined;
}
/** Cut 14 §4: a trace chip's label, ≤ 3 words: `D5 · died · trace` (`died · trace` without a depth). */
export function traceLabel(app: App, x: ExitLine, newer: ExitLine[] = []): string {
  const tier = /^(banked|returned|died)/.exec(x.text)?.[1] ?? exitLead(x).split(" ")[0];
  // QA 1a2a4a9 (O: `… on D8 D8 · DIED · TRACE`): a line that already names the floor keeps it once — the chip reads `died · trace`
  // QA 778fa1b (qaU: `died · trace` beside every `D8 · returned · trace`): the chip sits in its own column now (Cut 20), so it names the
  // floor always, even when the line beside it says `on D6`
  const d = exitDepth(app, x, newer);
  return /* copy:callout */ `${d !== undefined ? `D${d} · ` : ""}${tier} · trace`;
}

/** Cut 10 §3: an exit line's lead — the tier from its keep share and the sum kept: `returned $61` · `banked $84` · `died $0`. */
export function exitLead(x: ExitLine): string {
  const tier = x.keep_pct >= 100 ? /* copy:label */ "banked" : x.keep_pct <= 0 ? /* copy:label */ "died" : /* copy:label */ "returned";
  return `${tier} $${x.kept}`;
}

/** The ledger line with its lead in bold: the engine's text leads with `died $0 · …` (Cut 10 §3) and is split there; a text
 *  without a lead (an older slice) gets one in front — never two (`died $0 · died $0 · $190 carried` on every real report). */
export function ledgerText(x: ExitLine, name?: (label: string) => string): (string | HTMLElement)[] {
  const m = /^((?:banked|returned|died) \$-?\d+)(?: · )?(.*)$/s.exec(x.text);
  if (m) return [h("b", { class: "lead" }, m[1]), m[2] ? " · " : "", wakeShown(m[2]), exitExtras(x, name)];
  return [h("b", { class: "lead" }, exitLead(x)), " · ", wakeShown(x.text), exitExtras(x, name)];
}

/** Cut 16 §1: the depths picked clean as one line — consecutive depths collapse (`D1–4 · thinned`, `D3 · D5 · thinned`).
 *  Cut 21 §3 (AE: "`D8 · picked clean` (?? what does that mean)"): the word is `thinned` — the floor's loot is thinner for a while. */
export function pickedLine(depths: number[]): string {
  const ds = [...new Set(depths)].sort((a, b) => a - b), runs: string[] = [];
  for (let i = 0; i < ds.length; i++) {
    let j = i; while (j + 1 < ds.length && ds[j + 1] === ds[j] + 1) j++;
    runs.push(j > i ? `D${ds[i]}–${ds[j]}` : `D${ds[i]}`); i = j;
  }
  return `${runs.join(" · ")} · ${/* copy:callout */ "thinned"}`;
}

/** Rows of one name summed (`blue potion? ×8` read as `poison` beside `poison ×3` → `poison ×11`), first-seen order. */
function mergeRows(rows: { kind: string; n: number; gold: number }[]): { kind: string; n: number; gold: number }[] {
  const m = new Map<string, { kind: string; n: number; gold: number }>();
  for (const r of rows) { const x = m.get(r.kind) ?? { kind: r.kind, n: 0, gold: 0 }; x.n += r.n; x.gold += r.gold; m.set(r.kind, x); }
  return [...m.values()];
}
/** QA a946e04 (S: SALVAGED `blue potion? ×8` beside `poison ×3` after the night identified poison): a label stored while its kind was
 *  unknown reads by its name now — the core's `Lineage.renamed`, else the lineage's facts (`item:blue=poison`); `_` never shows. */
export function renamer(L: Pick<Lineage, "renamed" | "facts">): (label: string) => string {
  const idents = new Map((L.facts ?? []).map((f) => /^item:([a-z_]+)=([a-z_]+)$/.exec(f)).filter((m): m is RegExpExecArray => !!m).map((m) => [m[1], m[2]]));
  return (label: string): string => {
    const r = L.renamed?.[label]; if (r) return r.replace(/_/g, " ");
    const m = /^([a-z_]+) (potion|scroll)\?$/.exec(label), k = m && idents.get(m[1]);
    return (k ?? label).replace(/_/g, " ");
  };
}

export function renderReport(app: App, r: ReturnReport, absence = false): Mounted {
  const L = app.lineage;
  const named = renamer(L);
  const deathsN = r.deaths.reduce((n, d) => n + d.n, 0);
  // Cut 17 §4: the tiles are engraved score plaques on the parchment (an icon per count)
  const PLAQUE: Record<string, string> = { runs: "fast", deaths: "morgue", deepest: "depth", best: "depth", marks: "mark", banked: "gold", returned: "bail", stalled: "pause" };
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile plaque" }, icon(PLAQUE[label] ?? "depth"),
    h("b", { class: "num" }, ...(n.startsWith("◆") ? [h("span", { class: "g" }, "◆"), n.slice(1)] : [n])), h("span", { class: "label" }, label));
  // Cut 2 §1: `banked · returned · deaths` as a second row of three when the core reports exits; else the Cut 1 four
  const exits = r.banked !== undefined || r.returned !== undefined;
  // a stall is inside the core's `returned` (a return that kept nothing); the tiles count it apart — `returned` is the returns
  // that came home with something, `stalled` its own tile when there were any (a QA player on 50bb162 read `1 RETURNED` for a
  // run whose line said `returned $0 · … · stalled`). With no banks the stall tile takes `banked`'s place so the row stays three.
  const stalledN = r.stalled ?? 0, bankedN = r.banked ?? 0, returnedN = Math.max(0, (r.returned ?? 0) - stalledN);
  const banked = tile(`${bankedN}`, /* copy:label */ "banked"), returned = tile(`${returnedN}`, /* copy:label */ "returned");
  // Cut 14 §4: the stalls' cost — what the stalled runs carried home for nothing (their lines' `carried`; lines the slices
  // dropped are not counted, so the sum is a floor) — `2 STALLED · $161 lost` (rater T: "`2 STALLED` says nothing about what the
  // stalls cost")
  const stallLost = (r.exits ?? []).filter((x) => /\bstalled\b/.test(x.text)).reduce((a, x) => a + Math.max(0, x.carried), 0);
  const stalled = stalledN > 0 ? tile(`${stalledN}`, /* copy:label */ "stalled") : null;
  if (stalled) stalled.appendChild(h("span", { class: "num cost down" }, /* copy:callout */ `$${stallLost} lost`));
  const exitTiles = (): (HTMLElement | null)[] => {
    // QA 92eb880 (M, N: "the first report orders `BANKED · RETURNED`, the absence report `RETURNED · BANKED`; I read the wrong tile"):
    // one order always, the camp's gems' — banked, returned (Cut 10 §3's larger-first swap withdrawn); a stall with no bank takes
    // `banked`'s slot, so `returned` never moves
    if (stalled && bankedN === 0) return [stalled, returned];
    return [banked, returned, stalled];
  };
  const tiles = h("div", { class: `tiles${exits ? " six" : ""}${absence ? " fade-in" : ""}` },
    tile(`${r.sampled ? "~" : ""}${r.runs}`, /* copy:label */ "runs"),
    exits ? null : tile(`${deathsN}`, /* copy:label */ "deaths"),
    // the send's deepest floor, a delta like the tiles beside it (the lineage best is in the header; both QA players read
    // `1 RUNS · D4 BEST` as this send's); an old wire without it shows the lineage best
    r.deepest !== undefined ? tile(`D${r.deepest}`, /* copy:label */ "deepest") : tile(`D${L.best_depth}`, /* copy:label */ "best"),
    tile(`◆${r.marks_earned > 0 ? "+" : ""}${r.marks_earned}`, /* copy:label */ "marks"),
    ...(exits ? exitTiles() : []),
    exits ? tile(`${deathsN}`, /* copy:label */ "deaths") : null,
  );
  // Cut 16 §1: the shallows the lineage has farmed thin (`D3 · D4 · picked clean`), one dim line under the tiles
  const picked = r.picked?.length ? h("div", { class: "picked-line dim num" }, pickedLine(r.picked)) : null;
  // Cut 20 §5: the night's bounty floor — `bounty D12 · taken $412`, or `bounty D12 · missed` (what the safe set left on the table)
  const bounty = r.bounty ? h("div", { class: `bounty-line num${r.bounty.taken ? " taken" : " missed dim"}` },
    r.bounty.taken ? /* copy:callout */ `bounty D${r.bounty.depth} · taken $${r.bounty.gold}` : /* copy:callout */ `bounty D${r.bounty.depth} · missed`) : null;
  // QA a946e04 (T: 3 of 19 runs went from D1, the night's pass unpaid, nothing said so): `D5 short · 3 runs from D1`
  const startShort = r.start_short ? h("div", { class: "start-short-line warn num" }, /* copy:callout */ `${"D" + r.start_short.depth} short · ${r.start_short.runs} runs from ${"D1"}`) : null;
  const rested = r.rested_s ? h("div", { class: "rest-line dim num" }, /* copy:label */ "rested", " ", spanOf(r.rested_s)) : null;
  // Cut 13 §3: the gold line — what the exits brought (banked / returned, off the exit lines), the salvage, the automations' spending
  const goldLine = (): HTMLElement | null => {
    if (!r.spent && !r.salvaged && !r.gold && !r.restock_capped && !r.repeat_short && !r.swapped) return null;
    const ex = r.exits ?? [];
    const bankedG = ex.filter((x) => x.keep_pct >= 100).reduce((a, x) => a + x.kept, 0), returnedG = ex.filter((x) => x.keep_pct > 0 && x.keep_pct < 100).reduce((a, x) => a + x.kept, 0);
    const salvageG = (r.salvaged ?? []).reduce((a, x) => a + x.gold, 0), spentG = (r.spent ?? []).reduce((a, x) => a + x.gold, 0);
    const pieces: (string | HTMLElement)[] = [];
    const WORD = /* copy:callout */ { banked: "banked", returned: "returned", salvage: "salvage", wake: "purse", spent: "spent" };
    const piece = (n: number, sign: string, word: string, cls: string): void => { if (n > 0) pieces.push(h("span", { class: cls }, `${sign}$${n} ${word}`)); };
    // the core's summary is to the coin over every run of the absence (the exit lines are capped per slice): it wins
    // QA 92eb880 (M, N: "`+$892 home` where the gold sheet and rows say `returned`"): the exits' coins by the rows' own words — `banked` /
    // `returned` when the exit lines account for the core's sum, else the word of the only tier there was; `home` never
    const homeWord = (): string => { const b = r.banked ?? 0, rt = r.returned ?? 0; return rt === 0 && b > 0 ? WORD.banked : b === 0 ? WORD.returned : ""; };
    if (r.gold) {
      if (bankedG + returnedG === r.gold.home && r.gold.home > 0) { piece(bankedG, "+", WORD.banked, "up"); piece(returnedG, "+", WORD.returned, "up"); }
      else { const w = homeWord(); if (w) piece(r.gold.home, "+", w, "up"); else pieces.push(h("span", { class: "up" }, `+$${r.gold.home}`)); }
      piece(r.gold.salvage, "+", WORD.salvage, "up");
      // QA e75ec29 (Q, R: `+$40 heir purse` after one death, `+$30` after others, none after three): the purse rule — each death tops the
      // next heir's purse up to `wake_cap`; the top-ups counted (`+$70 purse ×2`), the deaths that found it full named (`purse full ×1`)
      if (r.gold.wake > 0) pieces.push(h("span", { class: "up" }, `+$${r.gold.wake} ${WORD.wake}`, r.gold.wake_n && r.gold.wake_n > 1 ? ` ×${r.gold.wake_n}` : ""));
      // QA 778fa1b (qaU: `purse full` beside $999 read as the camp's cap): the deaths the core flags (a purse just over the top-up line,
      // `ExitLine.purse_full`) read `no top-up`; a richer lineage's deaths say nothing of the purse
      const full = ex.filter((x) => x.purse_full).length;
      if (full > 0) pieces.push(h("span", { class: "dim purse-full" }, /* copy:callout */ `no top-up${full > 1 ? ` ×${full}` : ""}`));
      piece(r.gold.spent, "−", WORD.spent, "down");
    }
    else { piece(bankedG, "+", WORD.banked, "up"); piece(returnedG, "+", WORD.returned, "up"); piece(salvageG, "+", WORD.salvage, "up"); piece(spentG, "−", WORD.spent, "down"); }
    // Cut 19 §3: the repeat stopped once the night's spending reached what it brought home — Cut 21 §3 (AE, AF: `restock capped`
    // unread): it says the rule, `restock ≤ income`; both it and `repeat short` open the gold sheet, where the ledger lines are
    // QA 778fa1b (qaU: `carry $61 −$37 swapped` on the strip, in no ledger): what the pack's swaps took off the carry (`ReturnReport.swapped`,
    // else the lines' own) — already out of `carried`, so a dim note, not a movement of the purse
    const swapped = r.swapped ?? ex.reduce((a, x) => a + (x.swapped ?? 0), 0);
    if (swapped > 0) pieces.push(h("span", { class: "dim swapped" }, /* copy:callout */ `−$${swapped} swapped`));
    if (r.restock_capped) pieces.push(h("button", { class: "capped warn ledger-link", onclick: () => openGoldSheet(app) }, /* copy:callout */ "restock ≤ income"));
    // QA 1a2a4a9 (P: "the restock was skipped with no word"): a re-pack the purse could not pay
    if (r.repeat_short) pieces.push(h("button", { class: "capped warn ledger-link", onclick: () => openGoldSheet(app) }, /* copy:callout */ "repeat short"));
    if (!pieces.length) return null;
    const out: (string | HTMLElement)[] = []; pieces.forEach((p, i) => { if (i) out.push(" · "); out.push(p); });
    return h("div", { class: "gold-line dim num" }, ...out);
  };
  // Cut 6 §1: one ledger line per exit, verbatim from the engine, under the tiles (oldest first; the core keeps the last 5
  // per slice and the client merges slices, so a long absence shows its last EXITS_SHOW)
  // Cut 9 §5: an exit that carries its trace gets a `trace` chip after the line
  // Cut 10 §3: `returned $61` leads each line, the engine's arithmetic after it
  // Cut 11 §5: the line is tappable — the gold sheet filtered to that run (an exit claims the newest matching ledger exit the
  // exits after it in this report have not); its `trace` chip shows the chain (§3)
  const allExits = r.exits ?? [];
  // the client merges the absence's slices, so the report can hold more lines than EXITS_SHOW: the last EXITS_SHOW show and
  // `· N more` is a button that expands to every line (QA on e0f87e7: "`· 8 more` is inert text"); runs the tiles count that
  // have no line at all (the core keeps the last few per slice) are `· N unlisted`, dim and inert, so the count and the list
  // agree (QA on 952e306: "17 RUNS · 17 RETURNED but only 8 lines")
  const exitLines = allExits.length ? h("div", { class: "exit-lines" }) : null;
  const paintExits = (all: boolean): void => {
    if (!exitLines) return;
    const shown = all ? allExits : allExits.slice(-EXITS_SHOW);
    const hidden = allExits.length - shown.length, unlisted = Math.max(0, r.runs - allExits.length);
    // QA 23ed91f (K: "the run rows list oldest → newest … the gold sheet newest → oldest; I read the first row as the latest"): newest
    // first, like the gold sheet; `· N earlier` stays under them (the older ones)
    // Cut 20 (AD: "tapping `DIED · TRACE` opened the GOLD sheet"): a wrapped line put its chip directly under the line's own button,
    // two tap targets stacked 0 px apart — the line is a row now: the text (the gold sheet) on the left, the chip (the trace) in its
    // own column on the right, never under the text
    exitLines.replaceChildren(...shown.map((x, i) => h("div", { class: "ledger-line exit-row num dim" },
      h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, x, shown.slice(i + 1)) }, ...ledgerText(x, named)),
      h("span", { class: "sep", "aria-hidden": "true" }, " · "),   // a break between the line and its chip (QA on 3d71c33: `keeps 60%D7`; QA 1a2a4a9, O: `◆+2 D3 · RETURNED` glued)
      traceChip(x.trace, "chip mini", { rows: app.rules.rows, runId: x.run_id }, x.text, traceLabel(app, x, shown.slice(i + 1))))).reverse(),   // Cut 11 §2: with the run, the chain's links get `watch`; the sheet's header is the line; Cut 14 §4: the chip names its exit
      hidden > 0 ? h("button", { class: "ledger-line ledger-more num", onclick: () => paintExits(true) }, /* copy:button */ `· ${hidden} earlier`) : "",
      unlisted > 0 ? h("div", { class: "ledger-line num dim unlisted" }, /* copy:callout */ `· ${unlisted} unlisted`) : "");
  };
  paintExits(false);
  // Stall verdict (core README): every run came home and nothing got deeper — the row that ended them, then patches as on
  // the death screen (tap: replace / remove / insert, camp on the row). The core's line is the copy (≤ 12 words).
  const stall = r.stall ? h("section", { class: "rsec stall" },
    h("div", { class: "label" }, /* copy:label */ "plateau"),   // every run came home, none deeper — not a stalled run (QA on 56f2a1d: `STALL` over `14 RETURNED`)
    h("div", { class: "stall-line num" }, r.stall.text, " ", traceChip(r.stall.trace, "chip mini", { rows: app.rules.rows, runId: stallRun(r) })),   // Cut 9 §5: the trace of the last run the row ended; its rows labelled like the exits' (QA: "R1 · no item" lacked the verb); its run: the exit whose trace it is (QA on e0f87e7: no `watch` from a report)
    r.stall.patches.length ? patchRows(app, r.stall.patches, undefined, undefined, { depth: stallDepth(r.stall.text) }) : null) : null;
  // Cut 2 §2: one line per pile recovered this send (the core sends `heir 3 · D7 · 4 items`, `bones:7:4` too; the watch
  // `D5 · 7 items`). Every line says it was found — `found ♟3's bones · D7 · 4 items` — since `bones D8 · 11 items · ♟3` read
  // as a pile still lying there (QA on e0f87e7: "survived 16 offline runs", "persisted through run 3")
  const bonesLine = (x: string): string => {
    const m = /^bones:(\d+):(\d+)$/.exec(x); if (m) return /* copy:callout */ `found bones · D${m[1]} · ${items(+m[2])}`;
    const c = /^heir (\d+) · (D\d+) · (\d+) items?$/.exec(x); if (c) return /* copy:callout */ `found ♟${c[1]}'s bones · ${c[2]} · ${items(+c[3])}`;
    const w = /^(D\d+) · (.*)$/.exec(x); if (w) return /* copy:callout */ `found bones · ${w[1]} · ${w[2]}`;
    return /^found\b/.test(x) ? x : /* copy:callout */ `found bones · ${x.replace(/^bones\s*/, "")}`;
  };
  const section = (label: string, body: Node | null): HTMLElement | null => body ? h("section", { class: "rsec" }, h("div", { class: "label" }, label), body) : null;
  const nice = (x: string): string => x.replace(/_/g, " ");
  // `R1 fired 3 of 16 runs: HP<50% → drink ?` names the row in the engine's short form; the report spells it as the
  // editor and the death screen do (`hp < 50% → drink unknown`) when the row is still in the set
  const rowSpelt = (x: string): string => {
    // the trailing ` · heal unknown` is the engine's reason for a never-fired drink row; ` · shadowed by R1` (QA 92eb880) names the row above that takes its moments
    const m = /^R(\d+) fired (\d+) of (\d+) runs: (.*?)( · (?:\w+ unknown|shadowed by R\d+))?$/.exec(x);
    const row = m && app.rules.rows[Number(m[1]) - 1];
    return row ? /* copy:death_line */ `R${m[1]} fired ${m[2]} of ${m[3]} runs: ${rowLabel(row)}${m[5] ?? ""}` : x;
  };
  // repeats (three goblin archers tamed) collapse to one chip with a count
  const chips = (xs: string[], cls = "chip"): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x, (n.get(x) ?? 0) + 1);
    // "kind · name" (companions) renders the name small
    const label = (x: string): (string | HTMLElement)[] => { const i = x.indexOf(" · "); return i < 0 ? [nice(x)] : [nice(x.slice(0, i)), h("small", { class: "dim" }, ` ${x.slice(i + 3).replace(/ · fell$/, " fell")}`)]; };
    return n.size ? h("div", { class: "chips" }, ...[...n].map(([x, k]) => h("span", { class: cls }, ...label(x), k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  const lines = (xs: string[]): HTMLElement | null => xs.length ? h("ul", { class: "lines" }, ...xs.map((x) => h("li", null, nice(rowSpelt(x))))) : null;
  // identical reel lines (the same pattern in several runs) collapse to one with a count
  const reel = (xs: string[]): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x, (n.get(x) ?? 0) + 1);
    return n.size ? h("ul", { class: "lines" }, ...[...n].map(([x, k]) => h("li", null, x, k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  // pending: the engine's lines, with affordable unlocks shown as cards once the catalogue arrives
  const pendingBody = h("div", null);
  const pendingSec = section(/* copy:label */ "pending", pendingBody);
  const paintPending = (affordable: ReturnType<typeof visible>): void => {
    // `R1 fired n of m runs` lines come for every row (summed across slices); only the quiet ones are decisions
    const quiet = (p: string): boolean => { const m = /^R\d+ fired (\d+) of (\d+) runs/.exec(p); return !m || Number(m[1]) * 3 < Number(m[2]); };
    const pendingLines = (affordable.length ? r.pending.filter((p) => !/^unlock\b/.test(p)) : r.pending).filter(quiet);
    pendingBody.replaceChildren();
    // Cut 23 §1: the core's `forge sword +1 · $300` (a kit step the purse buys now) opens the forge
    const forgeLines = pendingLines.filter((p) => /^forge /.test(p));
    const ul = lines(pendingLines.filter((p) => !/^forge /.test(p))); if (ul) pendingBody.appendChild(ul);
    if (forgeLines.length) pendingBody.appendChild(h("div", { class: "chips forge-pending" }, ...forgeLines.map((p) => h("button", { class: "chip mini forge-line num", onclick: () => openForge(app) }, p))));
    // Cut 9 §2: the card opens its sheet; the buy is there, and the report repaints itself after one
    // QA 1a2a4a9: the core's short list (`UnlockInfo.short`) when sent — the camp's shelf shows the same three
    const coreShort = affordable.some((u) => u.short !== undefined);
    affordable = affordable.map((u) => withRowsGate(u, app.ownRows(), app.vocab.max_rows)).filter((u) => coreShort ? u.short : u.available);   // Cut 10 §3; Cut 12 §1: own rows
    // QA 23ed91f (L: "PENDING lists the whole unlock shop, identical across five reports"): the next three, as the camp's panel
    // (the larger reach gain, then the cheaper); the camp's `more` has the rest
    if (!coreShort) affordable = affordable.map((u, i) => ({ u, i })).sort((a, b) => (b.u.delta ?? 0) - (a.u.delta ?? 0) || a.u.cost - b.u.cost || a.i - b.i).slice(0, 3).map((x) => x.u);
    if (affordable.length) pendingBody.appendChild(h("div", { class: "cards" }, ...affordable.map((u) => h("button", { class: "card", onclick: () => openUnlockSheet(app, u, () => app.go({ kind: "report", report: r })) }, h("span", null, u.label), h("span", { class: "num cost" }, priceLabel(u))))));   // Cut 18 §5: both prices
    if (pendingSec) pendingSec.hidden = !pendingBody.childElementCount;
  };
  paintPending([]);
  // Cut 14 §4: the absence's usage lines are the death screen's drop sheet counts (Cut 15 §3); a watched run's report has none and
  // leaves the watch's own counts
  const usage = r.pending.map((p) => /^R(\d+) fired (\d+) of (\d+) runs/.exec(p)).filter((m): m is RegExpExecArray => !!m);
  if (usage.length) { const fires = app.rules.rows.map(() => 0); for (const m of usage) if (Number(m[1]) - 1 < fires.length) fires[Number(m[1]) - 1] = Number(m[2]); app.rowFires = fires; app.rowFiresOf = Number(usage[0][3]); }
  void app.engine.unlocks().then((cat) => paintPending(visible(cat).filter((u) => u.available))).catch(() => { /* lines only */ });
  // Cut 17 §4: the console — `open` (the worst death's verdict) · `gold` (the ledger of this absence's gold) · `ledger` (the
  // bestiary, from the 5th heir); the gem is `camp`
  const bar = renderBar(app);
  const cons = renderConsole({
    portrait: portrait(app, { hp: 1, label: `♟${L.heir}` }).el,
    gem: gem({ label: /* copy:button */ "camp", cls: "camp-gem", pulse: true, onclick: () => app.go({ kind: "camp" }) }),
    tiles: [
      // QA e75ec29 (Q: "`open` opens ♟5's death, not the newest; the label names nothing"): it is the absence's worst death — it says so
      r.worst_death ? cmdTile({ id: "open", label: /* copy:button */ "worst", icon: "trace", onclick: () => app.go({ kind: "death", death: r.worst_death!, lost: r.lost ?? [] }) }) : null,
      cmdTile({ id: "gold", label: /* copy:button */ "gold", icon: "gold", onclick: () => openGoldSheet(app) }),
      revealed(app).has("heirs") ? cmdTile({ id: "ledger", label: /* copy:button */ "ledger", icon: "ledger", onclick: () => openLedger(app) }) : null,
    ],
  });
  const sheet = h("div", { class: "parchment report-sheet" },
    tiles, goldLine(), startShort, bounty, picked, exitLines, rested, stall,
    // QA 23ed91f (K, L: `bones D7` among LEARNED): a heir's bones are a find (the BONES section), not a fact learned
    section(/* copy:label */ "learned", factChips(r.learned.filter((f) => !/^bones:\d+$/.test(f)), L.counters ?? [])),
    section(/* copy:label */ "tamed", chips(r.tamed ?? [], "chip ally")),
    section(/* copy:label */ "hatched", chips(r.hatched ?? [], "chip ally")),
    // Cut 10 §3: a companion `◯ jackal · Ashar fell` (the name small); Cut 12 §6: a summoned ally `ally hound fell`
    section(/* copy:label */ "lost", chips((r.lost ?? []).map((k) => k.includes(" · ") ? /* copy:callout */ `◯ ${k} fell` : lostLabel(k)), "chip egg")),
    section(/* copy:label */ "bests", lines(collapseBests(r.bests).map(bestLabel))),
    r.xp && (r.xp.gained > 0 || r.xp.level_ups > 0) ? section(/* copy:label */ "xp", h("div", { class: "xp-line num" }, `${r.xp.class} +${r.xp.gained}`, " · ", /* copy:label */ `L${L.classes?.[r.xp.class]?.level ?? 1}`, r.xp.level_ups > 0 ? h("b", null, ` ↑${r.xp.level_ups}`) : "")) : null,
    section(/* copy:label */ "found", chips(r.found.map((i) => named(i.label)))),
    // QA e75ec29 (R: six thefts in one run, "the report and gold sheet say nothing"): what thieves took and no run got back
    // QA a946e04 (S: `leash ×4` beside `leash (2)`, `black potion?` after LEARNED said confusion): one chip per name, identified kinds by
    // their name; T (`−$36 stolen` on the strip, only items here): the carry the thefts took leads (`$36`)
    section(/* copy:label */ "stolen", r.stolen?.length || r.stolen_gold ? h("div", { class: "chips" },
      r.stolen_gold ? h("span", { class: "chip stolen gold num" }, `$${r.stolen_gold}`) : "",
      ...mergeRows((r.stolen ?? []).map((x) => ({ kind: named(x.label.replace(/\s*\(\d+\)$/, "")), n: x.n, gold: x.gold ?? 0 }))).map((x) => h("span", { class: "chip stolen" }, x.kind, x.n > 1 ? h("b", { class: "num" }, ` ×${x.n}`) : "", x.gold > 0 ? h("small", { class: "num dim" }, ` $${x.gold}`) : ""))) : null),
    section(/* copy:label */ "bones", lines((r.bones_found ?? []).map(bonesLine))),
    section(/* copy:label */ "deaths", r.deaths.length ? h("ul", { class: "lines" }, ...r.deaths.map((d) => h("li", null, d.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${d.n}`)))) : null),
    // Cut 21 §2: found supplies the exits put on the shelf (the next send packs them free), before what was sold
    section(/* copy:label */ "shelved", r.shelved?.length ? h("div", { class: "chips shelved" }, ...r.shelved.map((x) => h("span", { class: "chip shelf" }, /* copy:callout */ `found ${x.kind.replace(/_/g, " ")}`, x.n > 1 ? h("b", { class: "num" }, ` ×${x.n}`) : "", /* copy:callout */ " → shelf"))) : null),
    section(/* copy:label */ "salvaged", r.salvaged?.length ? h("ul", { class: "lines" }, ...mergeRows(r.salvaged.map((x) => ({ ...x, kind: named(x.kind) }))).map((s) => h("li", null, s.kind.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num gold" }, `$${s.gold}`)))) : null),
    // Cut 13 §3: what the automations bought this absence, per kind (`heal ×16 · −$640`)
    section(/* copy:label */ "spent", r.spent?.length ? h("ul", { class: "lines" }, ...r.spent.map((s) => h("li", null, s.kind.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num down" }, `−$${s.gold}`)))) : null),
    section(/* copy:label */ "renown", r.renown && r.renown.gained > 0 ? h("div", { class: "num" }, `+${r.renown.gained} · ★${r.renown.rank}`, r.renown.ranks_up > 0 ? h("b", { class: "up" }, ` ↑${r.renown.ranks_up}`) : "", r.renown.ranks_up > 0 ? ` · ◆+${r.renown.ranks_up}` : "") : null),   // a rank pays a mark: the tiles' ◆ reconciles with the rows (QA on 56f2a1d: ◆+9 vs rows ◆+6)
    pendingSec,
    section(/* copy:label */ "reel", reel(r.reel.map((x) => noteText(x.text)))),
  );
  const el = h("main", { class: "report frame" }, bar.el, h("div", { class: "well report-well" }, sheet), cons.el);
  return { el, dispose: () => bar.dispose() };
}

/** QA 92eb880 (N: "plateau patches … `reach 17%` — reach of which floor?"): the floor past the plateau (`none past D6` → 7). */
export function stallDepth(text: string): number | undefined {
  const m = /\bpast D(\d+)\b/.exec(text) ?? /\bD(\d+)\b/.exec(text);
  return m ? Number(m[1]) + 1 : undefined;
}

/** The run a stall's trace belongs to: the exit line that carries the same trace (the stall has no run id on the wire; the
 *  same turns, tick for tick, name the run), so its chain links can open the replay when the client holds that run. */
function stallRun(r: ReturnReport): number | undefined {
  const t = r.stall?.trace; if (!t) return undefined;
  const key = JSON.stringify(t.turns);
  return r.exits?.find((x) => x.trace && JSON.stringify(x.trace.turns) === key)?.run_id;
}

/** Facts grouped for reading: `foe:x`, `foe:x:t1`, `foe:x:t2` → one chip "x · t1 · t2"; `item:f=k` → "k (f)";
 *  `biome:x` → "x"; `boss:x:counter[=row]` → "x counter: attack boss" (Cut 6 §5: the lineage's counter text names the row);
 *  others verbatim. */
function factChips(facts: string[], counters: Counter[] = []): HTMLElement | null {
  const nice = (x: string): string => x.replace(/_/g, " ");
  const foes = new Map<string, string[]>();
  const rest: HTMLElement[] = [], itemChips: HTMLElement[] = [];
  const bossCounters = new Map<string, string>();
  for (const f of facts) {
    const m = /^foe:([^:]+)(?::(.+))?$/.exec(f);
    if (m) { const tags = foes.get(m[1]) ?? []; if (m[2]) tags.push(m[2]); foes.set(m[1], tags); continue; }
    const it = /^item:([^=]+)=(.+)$/.exec(f);
    // QA 92eb880 (M: "LEARNED flattens potion/scroll pairs … `stray lock` and `blink` read as pairs off by one"): an identity reads
    // `blink (ashen)`, on its own row under the foes
    if (it) { itemChips.push(h("span", { class: "chip fact item" }, nice(it[2]), h("small", null, ` (${nice(it[1])})`))); continue; }
    const b = /^biome:(.+)$/.exec(f);
    if (b) { rest.push(h("span", { class: "chip fact" }, nice(b[1]))); continue; }
    const bn = /^bones:(\d+)$/.exec(f);
    if (bn) { rest.push(h("span", { class: "chip fact" }, /* copy:label */ "bones", h("small", null, ` D${bn[1]}`))); continue; }
    const c = /^boss:([^:]+):counter(?:=.*)?$/.exec(f);
    if (c) { bossCounters.set(c[1], counters.find((k) => k.boss === c[1])?.text ?? ""); continue; }
    // any other `kind:detail` fact reads like the foe chips (`alert · rising`, not `alert:rising`; QA on 50bb162)
    const kv = /^([^:]+):(.+)$/.exec(f);
    if (kv) { rest.push(h("span", { class: "chip fact" }, nice(kv[1]), h("small", null, ` · ${kv[2].split(":").map(nice).join(" · ")}`))); continue; }
    rest.push(h("span", { class: "chip fact" }, nice(f)));
  }
  // QA 1a2a4a9 (P: `goblin warlord · boss · telegraph · …` and again `goblin warlord · counter: attack boss`, two chips): a boss's counter
  // rides on its foe chip when the foe is learned in the same report
  for (const [boss, text] of bossCounters) {
    const tags = foes.get(boss);
    if (tags) tags.push(/* copy:none */ `counter${text ? `: ${text}` : ""}`);
    else rest.push(h("span", { class: "chip fact" }, nice(boss), h("small", null, /* copy:label */ " counter", text ? `: ${text}` : "")));
  }
  const out = [...foes].map(([k, tags]) => h("span", { class: "chip fact" }, nice(k), tags.length ? h("small", null, ` · ${tags.map(nice).join(" · ")}`) : ""));
  if (!out.length && !rest.length && !itemChips.length) return null;
  return h("div", { class: "facts" }, out.length + rest.length ? h("div", { class: "chips" }, ...out, ...rest) : "", itemChips.length ? h("div", { class: "chips items" }, ...itemChips) : "");
}

/** QA 23ed91f (K: "a lone `D8` and `rank 2` in among the trophies"): a depth best reads `new best D8`, a rank `★ rank 3`; trophies as sent. */
export const bestLabel = (x: string): string => /^D\d+$/.test(x) ? /* copy:callout */ `new best ${x}` : /^rank \d+$/.test(x) ? `★ ${x}` : x;

/** "rank 1 … rank 8", "fighter L2 … L4", "D3 … D6": one line per family, the highest, in first-seen order (`rank 3` with its
 *  space: QA 23ed91f, `rank 2` and `rank 3` both stood). */
function collapseBests(xs: string[]): string[] {
  const fam = (x: string): string | null => { const m = /^(rank |D|[a-z]+ L)(\d+)$/.exec(x); return m ? m[1] : null; };
  const best = new Map<string, string>(); const out: (string | null)[] = [];
  for (const x of xs) {
    const f = fam(x);
    if (!f) { out.push(x); continue; }
    if (!best.has(f)) { best.set(f, x); out.push(null); }
    else best.set(f, x);
  }
  const it = [...best.values()];
  return out.map((x) => x ?? it.shift()!);
}
