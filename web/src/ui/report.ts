import { supplyLimit, reportIncome } from "./report-supplies";
import { goldWords } from "./gold-words";
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
import { conceptTag } from "./concepts";
import { wallOffer, wallTablet } from "./wall";
import { meterPanel } from "./meters";
import { legacyEarnedBlock } from "./legacy-earned";
import { classXpBlock } from "./class-xp";
import { systemIcon, systemLabel } from "./systems";
import { openPreparationForge, preparationActions } from "./preparation";
import { AUTO, autoDismiss } from "./autodismiss";
import type { App, Mounted } from "../app";
import type { Counter, ExitLine, InvItem, Lineage, News, Patch, ReturnReport, Row } from "../engine/types";
import { h, replace, items, spanOf } from "./dom";
import { openDropSheet, patchRows } from "./patches";
import { BANDS, laneTitle, lanes, routeForks, seenForks } from "./route";
import { drivenDeath, exitExtras, wakeShown } from "./death";
import { openUnlockSheet, priceLabel, visible, withRowsGate } from "./unlocks";
import { heirOrd, lostLabel, noteText, rowLabel, setRefRows } from "./tokens";
import { traceChip } from "./trace";
import { closeAllSheets } from "./sheet";
import { openGoldSheet, runRange } from "./gold";
import { gem, portrait, renderBar, renderConsole, tile as cmdTile, wideCols, isWide } from "./frame";
import { icon } from "./skin";
import { revealed } from "./reveal";
import { openLedger } from "./party";
import { oathProgress } from "./oaths";
import { grewBlock, heroFace, reportTrainingBlock, trainingBeats } from "./tracks";
import { reportChoices } from "./report-choices";
import { bossName, reportBosses } from "./report-bosses";
import { workersBlock, workersSpent } from "./works";   // Cut 30.5: the workers' acts, one compact line under what grew
import { onPackages, penOpen } from "./packages";
import { bountyText } from "./forecast";
import { progressGoal, progressGoalRow } from "./progress-goal";
import { kwHost, kwText, detailHost } from "./tips";
import { openRuns } from "./runs";   // RUNS_UI: the runs tile opens the log
import { mountClear } from "./runclear";   // run-clear: the run's card before the town
import { itemIcon, itemName, itemChip } from "./items";
import { enemyTraitName } from "./enemy-tips";
import { labelOf as unlockLabel } from "./unlocks";


/** Persisted unlock IDs belong to the wire; report text uses catalogue words. */
const readableUnlock = (text: string, names?: Map<string, string>): string => text.replace(/^unlock ([\w:-]+)(?: \(\d+\))?/, (_all, id: string) =>
  `Unlock · ${names?.get(id) ?? unlockLabel(id)}`);

type ReadingPosition = { expanded: boolean; scroll: number };
const reportReading = new WeakMap<App, WeakMap<ReturnReport, ReadingPosition>>();
/** The waystone passage the report's sends were paid into the purse at the send: the core's sum over an absence (`gold.passage`), a
 *  watched run's own (`live.run.passage`; an absence's `live` is another run's, never read). */
export function reportPassage(r: Pick<ReturnReport, "gold" | "live">): number {
  return r.gold ? r.gold.passage ?? 0 : r.live?.run?.passage ?? 0;
}

function readingPosition(app: App, report: ReturnReport): ReadingPosition {
  let reports = reportReading.get(app);
  if (!reports) { reports = new WeakMap(); reportReading.set(app, reports); }
  let position = reports.get(report);
  if (!position) { position = { expanded: false, scroll: 0 }; reports.set(report, position); }
  return position;
}

const EXITS_SHOW = 8;
/** An exit line's lead word, the core's (QA 912e135: a timed-out run leads `stalled` / `lost thread`, never `returned`). */
const LEAD = /^(banked|returned|died|stalled|lost thread|driven)\b/;   // QA 0c6e126 (qaY): a drive-off leads `driven`, its tile's word

/** Cut 14 §4: the floor an exit ended on — the ledger's exit line it claims (`returned D5`, the gold sheet's own match), else
 *  the line's own `bones: 7 items on D5`; undefined when neither knows. `newer` = the exits after it in the same report. */
export function exitDepth(app: App, x: ExitLine, newer: ExitLine[] = []): number | undefined {
  if (x.reached !== undefined && x.reached > 0) return x.reached;
  const ledger = app.lineage.gold_ledger ?? [];
  const same = (a: ExitLine, b: ExitLine): boolean => (a.bloodline_id || 1) === (b.bloodline_id || 1) && a.kept === b.kept && LEAD.exec(a.text)?.[1] === LEAD.exec(b.text)?.[1];
  const range = runRange(ledger, x, newer.filter((y) => same(y, x)).length);
  const why = range ? ledger.slice(range[0], range[1] + 1).filter((g) => (g.bloodline_id || 1) === (x.bloodline_id || 1) && /^(returned|banked|died|lost|stalled|driven)\b.*\bD\d+/.test(g.why)).at(-1)?.why : undefined;
  const m = /\bD(\d+)\b/.exec(why ?? "") ?? /\bon D(\d+)\b/.exec(x.text);
  return m ? Number(m[1]) : undefined;
}
/** QA 524827b (qaAB: the absence's two deaths opened only a TRACE — no verdict word, no patches): a death line whose record the core
 *  keeps (`engine.death(run_id)`, the last few deaths) carries a `verdict` chip — the death screen for that run, as the chronicle opens
 *  a kept death; a line without a record (older than the kept few) the core refuses, and the chip says nothing more. */
function verdictChip(app: App, x: ExitLine, label: string, from?: { report: ReturnReport; absence?: boolean }): HTMLElement | "" {
  // Cut 26 §6 (AP: `driven $0 · $297 lost` with no verdict): a drive-off's line opens its verdict too — from the line itself
  if (x.driven) return h("button", { class: "chip mini verdict-chip", onclick: () => { closeAllSheets(); app.go({ kind: "death", death: drivenDeath(x, x.run_id ?? 0), kept: true, from }); } }, /* copy:button */ "verdict");
  if (!x.run_id || !/\bdied\b/.test(label)) return "";
  const btn: HTMLButtonElement = h("button", { class: "chip mini verdict-chip", onclick: () => {
    void app.busy(/* copy:label */ "verdict", () => app.engine.death(x.run_id!)).then((death) => { closeAllSheets(); app.go({ kind: "death", death, kept: true, from }); })
      .catch((e) => { btn.disabled = true; console.warn("offline death", e); });
  } }, /* copy:button */ "verdict");
  return btn;
}

/** Cut 26 §2: the lanes the night walked — each band down to the deepest floor it reached, on the route it played (`D5–8 · the Fens`);
 *  the core's `lanes` when the wire carries them, else the active set's route (the night plays it). Shown once a fork was seen. */
function lanesSection(app: App, r: ReturnReport): HTMLElement | null {
  const line = (text: string, biome?: string): HTMLElement => h("div", { class: "lane-line num", ...(biome ? { "data-biome": biome } : {}) }, text);
  // the core's lines verbatim (`D5–8 · the Fens`), shallowest first
  if (r.lanes?.length) return h("section", { class: "rsec lanes" }, h("div", { class: "label" }, /* copy:label */ "lanes"), ...r.lanes.map((t) => line(t)));
  if (!seenForks(app.lineage).length) return null;
  const deepest = Math.max(r.deepest ?? 0, r.live?.depth ?? 0, ...(r.exits ?? []).map((x) => exitDepthOf(x) ?? 0));
  const ls = deepest >= BANDS[0][0] ? lanes(routeForks(app.rules), deepest) : [];
  if (!ls.length) return null;
  return h("section", { class: "rsec lanes" }, h("div", { class: "label" }, /* copy:label */ "lanes"), ...ls.map((l) => line(`D${l.from}–${l.to} · ${laneTitle(l.biome)}`, l.biome)));
}
const exitDepthOf = (x: ExitLine): number | undefined => { const m = /\bD(\d+)\b/.exec(x.text); return m ? Number(m[1]) : undefined; };

/** Cut 14 §4: a trace chip's label, ≤ 3 words: `D5 · died · trace` (`died · trace` without a depth). */
export function traceLabel(app: App, x: ExitLine, newer: ExitLine[] = []): string {
  const tier = LEAD.exec(x.text)?.[1] ?? exitLead(x).split(" ")[0];
  // QA 1a2a4a9 (O: `… on D8 D8 · DIED · TRACE`): a line that already names the floor keeps it once — the chip reads `died · trace`
  // QA 778fa1b (qaU: `died · trace` beside every `D8 · returned · trace`): the chip sits in its own column now (Cut 20), so it names the
  // floor always, even when the line beside it says `on D6`
  const d = exitDepth(app, x, newer);
  return /* copy:callout */ `${d !== undefined ? `D${d} · ` : ""}${goldWords(tier)} · log`;
}

/** Cut 10 §3: an exit line's lead — the tier from its keep share and the sum kept: `returned $61` · `banked $84` · `died $0`. */
export function exitLead(x: ExitLine): string {
  const tier = x.keep_pct >= 100 ? /* copy:label */ "full haul" : x.keep_pct <= 0 ? /* copy:label */ "died" : /* copy:label */ "returned";
  return `${tier} $${x.kept}`;
}

/** The ledger line with its lead in bold: the engine's text leads with `died $0 · …` (Cut 10 §3) and is split there; a text
 *  without a lead (an older slice) gets one in front — never two (`died $0 · died $0 · $190 carried` on every real report). */
export function ledgerText(x: ExitLine, name?: (label: string) => string): (string | HTMLElement)[] {
  const m = /^((?:banked|returned|died|stalled|lost thread|driven) \$-?\d+)(?: · )?(.*)$/s.exec(x.text);
  // QA 0c6e126 (qaY: 16 death lines of 10–20 item names each, the killer and the floor buried): a report line is brief — what was left
  // and the pile are counts (`left 3`, the core's `bones: 12 items on D6`); the death screen and the gold sheet name them
  const killer = x.cause ? [" · ", h("b", { class: "killer" }, /* copy:callout */ `to ${x.cause}`)] : [];
  if (m) return [h("b", { class: "lead" }, m[1] === "driven" ? /* copy:callout */ "repelled" : goldWords(m[1])), ...killer, m[2] ? " · " : "", goldWords(wakeShown(m[2])), exitExtras(x, name, { brief: true })];
  return [h("b", { class: "lead" }, goldWords(exitLead(x))), " · ", goldWords(wakeShown(x.text)), exitExtras(x, name, { brief: true })];
}

/** Cut 16 §1: the depths picked clean as one line — consecutive depths collapse (`D1–4 · thinned`, `D3 · D5 · thinned`).
 *  Cut 21 §3 (AE: "`D8 · picked clean` (?? what does that mean)"): the word is `thinned` — the floor's loot is thinner for a while. */
export function pickedLine(depths: number[]): string {
  const ds = [...new Set(depths)].sort((a, b) => a - b), runs: string[] = [];
  for (let i = 0; i < ds.length; i++) {
    let j = i; while (j + 1 < ds.length && ds[j + 1] === ds[j] + 1) j++;
    runs.push(j > i ? `D${ds[i]}–${ds[j]}` : `D${ds[i]}`); i = j;
  }
  // QA 0c6e126 (qaZ: `D4–6 · thinned` with no source): what thinned — `D4–6 · loot thinned`; QA 524827b (qaAA: `D5–6 · loot thinned`
  // still unexplained — thinned by what?): the cause, which is also why — the floors were looted lately
  return `${runs.join(" · ")} · ${/* copy:callout */ "recently looted"}`;
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

/** Cut 24 §2 (AK: "identical end-of-run summaries, run after run"): what was new leads the report, before the counts — a watched run's
 *  own lines (the core's order, ≤ 3; a run with nothing new has its one `differ` line); an absence's, the newest runs' firsts
 *  across its exits (no `differ`, no `learned` — LEARNED lists them), ≤ 4, each once. Nothing on an older core. */
export function newsLines(r: Pick<ReturnReport, "runs" | "exits"> & Partial<Pick<ReturnReport, "new_finds">>, name: (label: string) => string = (l) => l): News[] {
  const all = (r.exits ?? []).map((x) => x.news ?? []);
  if (r.runs <= 1 || all.length <= 1) return mergeFinds((all[all.length - 1] ?? []).slice(0, 3));
  const seen = new Set<string>(), out: News[] = [];
  // QA 524827b (qaAA: `avenged Zelul, Drul, Sketh +2` over `grudge: Zelul the goblin`): the newest state of a named foe only — a grudge
  // an absence's later run avenged is not an open grudge in its header
  const avenged = new Set<string>();
  // QA 524827b (qaAA: `new find: bow, blink, summon ally +5` — 8 — over FOUND's 11): the header's finds are FOUND's own list
  // (`ReturnReport.new_finds`), counted the same, in the first find's place
  const finds = r.new_finds?.length ? r.new_finds.map(name) : null;
  let findsAt = false;
  for (const ns of [...all].reverse()) for (const n of ns) {
    if (n.k === "differ" || n.k === "learned" || seen.has(n.text)) continue;
    const av = n.k === "named" ? /^avenged (.+)$/.exec(n.text) : null, gr = n.k === "named" ? /^grudge: (\S+)/.exec(n.text) : null;
    if (av) avenged.add(av[1]);
    if (gr && avenged.has(gr[1])) continue;
    if (n.k === "find" && finds) {
      if (findsAt) continue;
      findsAt = true;
      out.push({ k: "find", text: /* copy:callout */ `new find: ${finds.slice(0, 3).join(", ")}${finds.length > 3 ? ` +${finds.length - 3}` : ""}` });
      continue;
    }
    seen.add(n.text); out.push(n);
  }
  return mergeFinds(out).slice(0, 4);
}
/** `new find: bow` · `new find: amber potion?` → `new find: bow, amber potion?` (one line, in the first's place). QA 0c6e126 (qaY: the
 *  header named three avenged, the lines four others, ♟14's line one of its two): `avenged Zelul` · `avenged Morim` → `avenged Zelul,
 *  Morim` likewise — every name in one list, wherever the news is read. */
export function mergeFinds(ns: News[], most = 3): News[] {
  const out: News[] = []; const kinds = new Map<News, { head: string; sep: string; items: string[] }>(); const lead = new Map<string, News>();
  for (const n of ns) {
    const m = n.k === "find" ? /^([^:]+)(: )(.+)$/.exec(n.text) : n.k === "named" ? /^(avenged)( )(.+)$/.exec(n.text) : null;
    const at = m ? lead.get(`${n.k}|${m[1]}`) : undefined, k = at ? kinds.get(at) : undefined;
    if (m && k) { if (!k.items.includes(m[3])) k.items.push(m[3]); continue; }
    const x = { ...n }; out.push(x); if (m) { lead.set(`${n.k}|${m[1]}`, x); kinds.set(x, { head: m[1], sep: m[2], items: [m[3]] }); }
  }
  // a night's finds stay one short line: the first `most`, then `+N`
  // QA 308f045 (qaAC: `avenged Theth, Morix, Grim +1` with two avenged lines in view — "what the +N counts"): the names avenged are
  // named, all of them up to five (a name is short; the lines that carry them may sit under `· N earlier`); past that `+N more`
  for (const [x, k] of kinds) { const cap = k.head === "avenged" ? Math.max(most, 5) : most;
    x.text = `${k.head}${k.sep}${k.items.slice(0, cap).join(", ")}${k.items.length > cap ? (k.head === "avenged" ? /* copy:callout */ ` +${k.items.length - cap} more` : ` +${k.items.length - cap}`) : ""}`; }
  return out;
}
/** Cut 24 §2: an absence's exit line leads with its run's first news (`record: D10 · returned $48 · …`); a watched run's report already
 *  leads with its news, so its one line does not repeat it. */
function newsLead(x: ExitLine, runs: number): (string | HTMLElement)[] {
  // QA 0c6e126 (qaY: `learned 3` at the head of a death line, no source): the facts are LEARNED's — a line never leads with their count
  // QA 524827b (qaAA: `driven off: Warlord · driven $0 · … · by Warlord`): a drive-off's line already names who (`by Warlord`)
  const n = runs > 1 ? mergeFinds((x.news ?? []).filter((y) => y.k !== "differ" && y.k !== "learned" && !(y.k === "driven" && / · by /.test(x.text))))[0] : undefined;
  return n ? [h("b", { class: "news-lead" }, n.text.replace(/^driven off: (.+)$/, /* copy:callout */ "repelled by $1")), " · "] : [];
}
function newsBlock(r: ReturnReport, name?: (label: string) => string, counters: { boss: string; text: string }[] = [], shop = true, pen = true): HTMLElement | null {
  // Cut 28 §2 (core `ReturnReport.lead`): the first screen leads with the decisions (`oath kept: D10 · no drink`, `plateau: none past
  // D13`, `mother: fire learned`, `D9 death · order`), then what was new that they do not already say
  const lead = r.lead ?? [];
  const said = new Set(lead.map((l) => l.text));
  const ns = newsLines(r, name).filter((n) => !said.has(n.text) && !lead.some((l) => l.k === "record" && n.k === "record"));
  if (!ns.length && !lead.length) return null;
  // docs/COPY.md pass 2 (`warlord: aim learned` read as the boss's move, 2/2): a counter learned reads as the rule that beats him
  const said2 = (t: string): string => t.replace(/^driven off: (.+)$/, /* copy:callout */ "repelled by $1").replace(/^deeper: (D\d+), last (D\d+)$/, /* copy:callout */ "reached $1 · was $2").replace(/^(\w+): (.+) learned$/, (m, boss: string) => { const c = counters.find((x) => x.boss.endsWith(boss)); return c?.text ? /* copy:callout */ `${boss} counter: ${c.text}` : m; });
  // (a drive-off's line carries the counter fact, `· warlord: aim`: the counter's own line says it) — then each line once
  // c30-legible (the coordinator: `D1 death · gap` before the pen opens): a death's verdict word is the pen's vocabulary — before it
  // opens the lead says the floor alone (`D1 death`)
  const prePenT = (t: string): string => pen ? t : t.replace(/^(D\d+ death) · .+$/, "$1");
  const said3 = (t: string): string => prePenT(said2(t)).replace(/ · (\w+): [a-z ?]+$/, (m, boss: string) => (counters.some((x) => x.boss.endsWith(boss)) ? "" : m));
  const shown = new Set<string>(); const once = (t: string): boolean => !shown.has(t) && !!shown.add(t);
  // Cut 30 integration: before the unlocks open (the pen's catalogue) no line names one (`unlock throw (3)` read as a shop nowhere on screen)
  const shown2 = (t: string): boolean => shop || !/^unlock\b/.test(t);
  const leadT = lead.map((l) => ({ l, t: said3(l.text) })).filter((x) => shown2(x.t) && once(x.t)), nsT = ns.map((n) => ({ n, t: said3(n.text) })).filter((x) => shown2(x.t) && once(x.t));
  const learnedCounter = (t: string): HTMLElement | string => {
    const m = /^(\w+) counter: (.+)$/.exec(t);
    if (!m) return t;
    const boss = m[1][0].toUpperCase() + m[1].slice(1);
    return h("div", { class: "counter-news" },
      h("b", { class: "counter-learned" }, /* copy:callout */ `${boss} weakness learned`),
      h("span", { class: "counter-action" }, m[2] === "attack boss" ? /* copy:callout */ "Target the boss" : m[2],
        m[1] === "warlord" && m[2] === "attack boss" ? h("span", {}, " · ", /* copy:callout */ "bypass shields") : null));
  };
  return h("div", { class: `news${lead.length ? " with-lead" : ""}` },
    ...leadT.map(({ l, t }, i) => h("div", { class: `news-line decision k-${l.k}${i === 0 ? " lead" : ""}`, "data-k": l.k },
      l.k === "plateau" ? h("div", { class: "plateau-title" }, /* copy:label */ "Record", h("span", { class: "plateau-floor" }, t.replace(/^plateau: none past D(\d+)$/, /* copy:label */ "floor $1"))) : l.k === "counter" ? learnedCounter(t) : readableUnlock(t),
      l.k === "plateau" && r.stall && stallDepth(r.stall.text) !== undefined ? h("div", { class: "plateau-summary" }, /* copy:label */ "Recent best", " · ", /* copy:label */ `floor ${stallDepth(r.stall.text)! - 1}`) : null)),
    ...nsT.slice(0, lead.length ? 2 : 4).map(({ n, t }, i) => h("div", { class: `news-line k-${n.k}${i === 0 && !lead.length ? " lead" : ""}` }, readableUnlock(t))));
}

/** Two rows with the same tokens (conds in order with their numbers, the verb). */
const sameRowShape = (a: Row, b: Row): boolean => a.verb.v === b.verb.v && (a.verb.a ?? "") === (b.verb.a ?? "") && a.conds.length === b.conds.length &&
  a.conds.every((c, i) => c.k === b.conds[i].k && (c.n ?? "") === (b.conds[i].n ?? "") && (c.t ?? "") === (b.conds[i].t ?? ""));

/** QA 912e135: `♟2–17` (one heir `♟5`) under the runs tile — the heirs who ran, the core's `ReturnReport.heirs`. */
function withHeirs(el: HTMLElement, heirs: number[] | undefined): HTMLElement {
  if (heirs?.length === 2) el.appendChild(h("span", { class: "num heirs-ran dim" }, heirs[0] === heirs[1] ? /* copy:label */ `heir ${heirs[0]}` : /* copy:callout */ `by heirs ${heirs[0]}–${heirs[1]}`));
  return el;
}

export function renderReport(app: App, r: ReturnReport, absence = false): Mounted {
  setRefRows(() => app.rules.rows);
  const reading = readingPosition(app, r);
  // Cut 25 §4: after an absence the forge's steps are measured while the report is read (the forge's own lane), so the camp's forge sheet
  // paints them at once
  const L = app.lineage;
  const named = renamer(L);
  const deathsN = r.deaths.reduce((n, d) => n + d.n, 0);
  // Cut 17 §4: the tiles are engraved score plaques on the parchment (an icon per count)
  const PLAQUE: Record<string, string> = { runs: "fast", deaths: "morgue", deepest: "depth", best: "depth", marks: "mark", "upgrade tokens": "mark", "full haul": "gold", banked: "gold", returned: "bail", stalled: "pause", driven: "bail" };
  // Cut 29 (owner: labels may be 2 words; docs/COPY.md §4 blocker 2 — `0/16 BANKED` still read as "some exit type"): an exit tile says
  // what it counts, `runs banked` (the key stays the one word, `data-k`)
  const SAYS: Record<string, string> = /* copy:label */ { banked: "full hauls", returned: "runs returned", stalled: "runs stalled" };
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile plaque", "data-k": label }, icon(PLAQUE[label] ?? "depth"),
    h("b", { class: "num" }, ...(n.startsWith("◆") ? [h("span", { class: "g" }, "◆"), n.slice(1)] : [n])), h("span", { class: "label" }, ...kwText(SAYS[label] ?? label, ["banked", "returned", "marks", "death"])));   // docs/TOOLTIPS.md
  // Cut 2 §1: `banked · returned · deaths` as a second row of three when the core reports exits; else the Cut 1 four
  const exits = r.banked !== undefined || r.returned !== undefined;
  // a stall is inside the core's `returned` (a return that kept nothing); the tiles count it apart — `returned` is the returns
  // that came home with something, `stalled` its own tile when there were any (a QA player on 50bb162 read `1 RETURNED` for a
  // run whose line said `returned $0 · … · stalled`). With no banks the stall tile takes `banked`'s place so the row stays three.
  // Cut 24 §1: a send a boss drove off (`no counter`, among the core's `returned`) is counted apart too — its own tile
  const stalledN = r.stalled ?? 0, drivenN = r.driven ?? 0, bankedN = r.banked ?? 0, returnedN = Math.max(0, (r.returned ?? 0) - stalledN - drivenN);
  // docs/COPY.md pass 4 (`BANKED 0` read as gold banked by all 4 readers): an end's count is a share of the runs (`0/16`)
  const ofRuns = (k: number): string => (exits && r.runs > 0 ? `${k}/${r.runs}` : `${k}`);
  const banked = tile(ofRuns(bankedN), /* copy:label */ "full haul"), returned = tile(ofRuns(returnedN), /* copy:label */ "returned");
  // c30-legible (the owner, a new player: "I didn't understand … why the run ended early"): an end's tile says why, the core's ≤ 3
  // words — the most common reason among this report's exits of that end (`hurt · banked`, `hurt · went home`)
  const why = (lead: RegExp): string | undefined => {
    const n = new Map<string, number>();
    for (const x of r.exits ?? []) if (x.reason && lead.test(x.text)) n.set(x.reason, (n.get(x.reason) ?? 0) + 1);
    return [...n].sort((a, b) => b[1] - a[1])[0]?.[0];
  };
  for (const [t, lead, n] of [[banked, /^banked\b/, bankedN], [returned, /^returned\b/, returnedN]] as const) {
    const w = n > 0 ? why(lead) : undefined;
    if (w) t.appendChild(h("span", { class: "tile-why num", "data-why": w }, goldWords(w)));
  }
  // Cut 14 §4: the stalls' cost — what the stalled runs carried home for nothing (their lines' `carried`; lines the slices
  // dropped are not counted, so the sum is a floor) — `2 STALLED · $161 lost` (rater T: "`2 STALLED` says nothing about what the
  // stalls cost")
  const stallLost = (r.exits ?? []).filter((x) => /\bstalled\b/.test(x.text)).reduce((a, x) => a + Math.max(0, x.carried), 0);
  const stalled = stalledN > 0 ? tile(ofRuns(stalledN), /* copy:label */ "stalled") : null;
  if (stalled) stalled.appendChild(h("span", { class: "num cost down" }, /* copy:callout */ `$${stallLost} lost`));
  const drivenTile = drivenN > 0 ? tile(ofRuns(drivenN), /* copy:label */ "repelled") : null;   // docs/COPY.md pass 2: `DRIVEN` read as "I drove off enemies"
  const exitTiles = (): (HTMLElement | null)[] => {
    // QA 92eb880 (M, N: "the first report orders `BANKED · RETURNED`, the absence report `RETURNED · BANKED`; I read the wrong tile"):
    // one order always, the camp's gems' — banked, returned (Cut 10 §3's larger-first swap withdrawn); a stall with no bank takes
    // `banked`'s slot, so `returned` never moves
    if (stalled && bankedN === 0) return [stalled, returned, drivenTile];
    return [banked, returned, stalled, drivenTile];
  };
  // RUNS_UI: the runs tile opens the runs log on these runs (the absence's fold) — what happened while away, run by run
  const toLog = (t: HTMLElement): HTMLElement => {
    if (!(L.runs ?? []).some((x) => x.id > 0)) return t;
    t.classList.add("to-log"); t.setAttribute("role", "button"); t.tabIndex = 0; t.dataset.log = "1";
    t.onclick = () => openRuns(app, { focus: (r.exits ?? []).map((x) => x.run_id ?? 0).filter((x) => x > 0).pop(), tab: "runs" });
    t.onkeydown = (e: KeyboardEvent) => { if (e.key === "Enter") t.click(); };
    return kwHost(t, "log");
  };
  const tokenPlaque = tile(`◆${r.marks_earned > 0 ? "+" : ""}${r.marks_earned}`, /* copy:label */ "upgrade tokens");
  tokenPlaque.append(h("small", { class: "resource-purpose" }, /* copy:label */ "Classes · styles"));
  const tiles = h("div", { class: `tiles${exits ? " six" : ""}${absence ? " fade-in" : ""}` },
    // QA 912e135 (qaW: `♟18` over a report of ♟2–♟17, read as the heir who ran): the runs tile names whose runs they were
    toLog(withHeirs(tile(`${r.sampled ? "~" : ""}${r.runs}`, /* copy:label */ "runs"), r.heirs)),
    exits ? null : tile(`${deathsN}`, /* copy:label */ "deaths"),
    // the send's deepest floor, a delta like the tiles beside it (the lineage best is in the header; both QA players read
    // `1 RUNS · D4 BEST` as this send's); an old wire without it shows the lineage best
    r.deepest !== undefined ? tile(`D${r.deepest}`, /* copy:label */ "deepest") : tile(`D${L.best_depth}`, /* copy:label */ "best"),
    tokenPlaque,
    ...(exits ? exitTiles() : []),
    exits ? tile(ofRuns(deathsN), /* copy:label */ "deaths") : null,
  );
  // Cut 16 §1: the shallows the lineage has farmed thin (`D3 · D4 · picked clean`), one dim line under the tiles
  const picked = r.picked?.length ? h("div", { class: "picked-line dim num" }, pickedLine(r.picked)) : null;
  // Cut 20 §5: the night's bounty floor — `bounty D12 · taken $412`, or `bounty D12 · missed` (what the safe set left on the table)
  const bounty = r.bounty ? h("div", { class: `bounty-line num${r.bounty.taken ? " taken" : " missed dim"}` },
    // Cut 28 §1 (AV: `bounty D13 · missed` never said what it pays or needs): a missed bounty says both, then `missed`
    ...(r.bounty.taken ? [/* copy:callout */ `bounty D${r.bounty.depth} · taken $${r.bounty.gold}`] : [bountyText({ ...(L.bounty?.depth === r.bounty.depth ? L.bounty : {}), depth: r.bounty.depth }), h("b", { class: "missed-w" }, /* copy:callout */ " · missed")])) : null;
  // QA a946e04 (T: 3 of 19 runs went from D1, the night's pass unpaid, nothing said so): `D5 short · 3 runs from D1`
  const startShort = r.start_short ? h("div", { class: "start-short-line warn num" }, /* copy:callout */ `${"D" + r.start_short.depth} short · ${r.start_short.runs} runs from ${"D1"}`) : null;
  const rested = r.rested_s ? h("div", { class: "rest-line dim num" }, /* copy:label */ "Rest assigned", " ", spanOf(r.rested_s), h("small", null, /* copy:label */ " · Includes pending")) : null;
  // Cut 13 §3: the gold line — what the exits brought (banked / returned, off the exit lines), the salvage, the automations' spending
  const goldLine = (): HTMLElement | null => {
    if (!r.spent && !r.salvaged && !r.gold && !r.restock_capped && !r.repeat_short) return null;
    const ex = r.exits ?? [];
    const bankedG = ex.filter((x) => x.keep_pct >= 100).reduce((a, x) => a + x.kept, 0), returnedG = ex.filter((x) => x.keep_pct > 0 && x.keep_pct < 100).reduce((a, x) => a + x.kept, 0);
    const salvageG = (r.salvaged ?? []).reduce((a, x) => a + x.gold, 0), spentG = (r.spent ?? []).reduce((a, x) => a + x.gold, 0);
    const pieces: (string | HTMLElement)[] = [];
    const WORD = /* copy:callout */ { banked: "collected", returned: "returned", salvage: "salvage", wake: "next heir", spent: "spent" };
    const piece = (n: number, sign: string, word: string, cls: string): void => { if (n > 0) pieces.push(h("span", { class: cls }, `${sign}$${n} ${word}`)); };
    // the core's summary is to the coin over every run of the absence (the exit lines are capped per slice): it wins
    // QA 92eb880 (M, N: "`+$892 home` where the gold sheet and rows say `returned`"): the exits' coins by the rows' own words — `banked` /
    // `returned` when the exit lines account for the core's sum, else the word of the only tier there was; `home` never
    const homeWord = (): string => { const b = r.banked ?? 0, rt = r.returned ?? 0; return rt === 0 && b > 0 ? WORD.banked : b === 0 ? WORD.returned : ""; };
    if (r.gold) {
      if (bankedG + returnedG === r.gold.home && r.gold.home > 0) { piece(bankedG, "+", WORD.banked, "up"); piece(returnedG, "+", WORD.returned, "up"); }
      else { const w = homeWord(); if (w) piece(r.gold.home, "+", w, "up"); else pieces.push(h("span", { class: "up" }, `+$${r.gold.home}`)); }
      piece(r.gold.salvage, "+", WORD.salvage, "up");
      piece(reportPassage(r), "+", /* copy:callout */ "passage", "up");
      // QA e75ec29 (Q, R: `+$40 heir purse` after one death, `+$30` after others, none after three): the purse rule — each death tops the
      // next heir's purse up to `wake_cap`; the top-ups counted (`+$70 purse ×2`), the deaths that found it full named (`purse full ×1`)
      if (r.gold.wake > 0) pieces.push(h("span", { class: "up" }, `+$${r.gold.wake} ${WORD.wake}`, r.gold.wake_n && r.gold.wake_n > 1 ? ` ×${r.gold.wake_n}` : ""));
      // QA 778fa1b (qaU: `purse full` beside $999 read as the camp's cap): the deaths the core flags (a purse just over the top-up line,
      // `ExitLine.purse_full`) read `no top-up`; a richer lineage's deaths say nothing of the purse
      // QA 912e135 (qaW: `no top-up ×16` — "the top-up it refers to is never shown"): the purse the deaths found, `heir purse ≥$40`
      // QA 0c6e126 (qaY: `heir purse ≥$40 ×12` beside one `+$20 heir purse` in the gold sheet — "the ≥ has no source"): only the top-ups
      // the ledger holds are named (`+$20 heir purse`, the gold sheet's own words); a death that found the purse full moved no gold
      piece(r.gold.spent, "−", WORD.spent, "down");
      // blind ad71e72 (A: `$6712` earned, the purse up $78): the rest of the purse's change — the workers' forge steps, hires, the bank
      if (r.gold.net !== undefined) { const bought = workersSpent(r.workers), other = r.gold.net - (r.gold.home + r.gold.salvage + reportPassage(r) + r.gold.wake - r.gold.spent) + bought; piece(bought, "−", /* copy:callout */ "apprentice forge", "down"); piece(-other, "−", /* copy:callout */ "workers, other", "down"); piece(other, "+", /* copy:callout */ "other", "up"); }
      // QA 912e135 (qaW: `STALLED $224 lost` and ~$1,900 carried by the dead, named nowhere on the gold side): what the exits did not keep —
      // a dim note beside the movements (it never was in the purse)
      // QA 524827b (qaAA: `$936 lost` on a report with 0 deaths): what a return did not keep (its 40 %) is `not kept`; `lost` is only
      // the whole carry of an exit that kept nothing (a death, a stall, a drive-off)
      const unkept = Math.min(r.gold.lost ?? 0, r.gold.unkept ?? 0), lostG = (r.gold.lost ?? 0) - unkept;
      if (lostG > 0) pieces.push(h("span", { class: "dim lost" }, /* copy:callout */ `$${lostG} lost`));
      if (unkept > 0) pieces.push(h("span", { class: "dim unkept" }, /* copy:callout */ `$${unkept} not kept`));
    }
    else { piece(bankedG, "+", WORD.banked, "up"); piece(returnedG, "+", WORD.returned, "up"); piece(salvageG, "+", WORD.salvage, "up"); piece(reportPassage(r), "+", /* copy:callout */ "passage", "up"); piece(spentG, "−", WORD.spent, "down"); }
    // Cut 19 §3: the repeat stopped once the night's spending reached what it brought home — Cut 21 §3 (AE, AF: `restock capped`
    // unread): it says the rule, `restock ≤ income`; both it and `repeat short` open the gold sheet, where the ledger lines are
    // QA 778fa1b (qaU: `carry $61 −$37 swapped` on the strip, in no ledger): what the pack's swaps took off the carry (`ReturnReport.swapped`,
    // else the lines' own) — already out of `carried`, so a dim note, not a movement of the purse
    // QA 0c6e126 (qaZ: `+$767 returned · +$215 salvage · −$480 spent · −$7 swapped` read as +$495 against a +$502 balance): the swaps
    // are already out of the carry the exits brought home — never a term of the headline; each run's line names its own (`−$5 left axe`)
    // QA 0c6e126 (qaY: `restock ≤ income` unexplained): the cap with its number — what the absence brought in (the core's `Batch::income`:
    // the exits, the salvage, the heir purses), `restock ≤ $0 earned`
    if (r.restock_capped) pieces.push(supplyLimit(app, r));
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
    exitLines.replaceChildren(...shown.map((x, i) => h("div", { class: "ledger-line exit-row num dim", "data-lead": x.text },
      h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, x, shown.slice(i + 1)) }, ...newsLead(x, r.runs), ...ledgerText(x, named)),
      // QA 912e135 (qaW, qaX: a lone `·` before every `D7 · died · trace`): the chip is its own flex column (Cut 20) — no separator glyph
      traceChip(x.trace, "chip mini", { rows: app.rules.rows, runId: x.run_id }, x.text, traceLabel(app, x, shown.slice(i + 1))),
      verdictChip(app, x, traceLabel(app, x, shown.slice(i + 1)), { report: r, absence }))).reverse(),   // Cut 11 §2: with the run, the chain's links get `watch`; the sheet's header is the line; Cut 14 §4: the chip names its exit
      hidden > 0 ? h("button", { class: "ledger-line ledger-more num", onclick: () => paintExits(true) }, /* copy:button */ `· ${hidden} earlier`) : "",
      unlisted > 0 ? h("div", { class: "ledger-line num dim unlisted" }, /* copy:callout */ `· ${unlisted} unlisted`) : "");
    // QA 524827b (qaAA: "`grudge: Zelul` (older) below `avenged Zelul` (newer) — I read the grudge as coming back"): the order is named
    if (shown.length > 1) exitLines.prepend(h("div", { class: "exit-order num dim" }, /* copy:callout */ "newest first"));
  };
  paintExits(false);
  /** QA 0c6e126 (qaY: `1 STALLED · $265 lost` with no stalled line among the eight shown): a stalled or driven tile reaches its lines —
   *  the tap unfolds the earlier ones and brings the newest of its kind into view. */
  const reach = (el: HTMLElement | null, word: RegExp): void => {
    if (!el || !exitLines || !allExits.some((x) => word.test(x.text))) return;
    el.classList.add("tile-link");
    el.onclick = () => {
      paintExits(true);
      const row = [...exitLines.querySelectorAll<HTMLElement>(".exit-row")].find((r) => word.test(r.dataset.lead ?? ""));
      if (!row) return;
      row.scrollIntoView({ block: "center", behavior: "smooth" }); row.classList.remove("flash"); void row.offsetWidth; row.classList.add("flash");
    };
  };
  reach(stalled, /^stalled\b/); reach(drivenTile, /^driven\b/);
  // Cut 24 §1 (AL: a Warlord fight > 4 min, the boss bar full): a boss no blow could move drove the hero off (`ExitLine.driven`, core) —
  // one tablet a boss, `Warlord · no counter · shield wall`, `try: attack boss` under it; a tap writes the counter at the top (or finds it)
  // Cut 26 §6 (core): the absence's drive-offs (`ReturnReport.drives`, each with its run) join the exit lines' — one tablet a boss
  const drivenOff = [...new Map([...allExits.filter((x) => x.driven).map((x) => ({ ...x.driven!, run_id: x.driven!.run_id ?? x.run_id })), ...(r.drives ?? [])].map((d) => [d.boss, d])).values()];
  // (the tile's count when one boss drove every send off — the lines the slices kept may be fewer; else the lines')
  const drivenBy = (boss: string): number => drivenOff.length === 1 && drivenN > 0 ? drivenN : allExits.filter((x) => x.driven?.boss === boss).length;
  const driven = drivenOff.length ? h("section", { class: "rsec driven" }, h("div", { class: "label" }, /* copy:label */ "counter"),
    ...drivenOff.map((d) => {
      const have = app.rules.rows.findIndex((r) => sameRowShape(r, d.row));
      // QA 0c6e126 (qaZ: the tap inserted the counter as R1 onto a full set — `7/6 rows`, SEND greyed): a full set asks which row it
      // replaces, as a death's patch does (`openDropSheet`: the row lands at the top, the dropped one goes)
      const write = (): void => {
        if (have >= 0) { app.go({ kind: "camp", highlight: have }); return; }
        if (app.rowsFull) { openDropSheet(app, { row: { ...d.row, origin: "patch" }, insert_at: 0, survive: 0, forecast_delta: 0 } as Patch); return; }
        app.go({ kind: "camp", highlight: app.insertRow(d.row, 0, "patch") });
      };
      return h("button", { class: `patch tablet driven-line${have >= 0 ? " held" : ""}`, onclick: write },
        // QA 524827b (qaAA: COUNTER `no counter` beside LEARNED `counter: attack boss`): the counter is known — the set lacked it
        // QA 524827b (qaAB: 12 of 16 sends driven off, twelve like lines): the tablet counts the sends that boss drove off (`Warlord ×12`)
        h("span", { class: "chips-inline" }, [/* copy:callout */ `${d.title} repelled him${drivenBy(d.boss) > 1 ? ` ×${drivenBy(d.boss)}` : ""}`, d.verdict === "no counter" ? (have >= 0 ? /* copy:callout */ "order" : "") : d.verdict, d.defence].filter(Boolean).join(" · ")),   // docs/COPY.md pass 6: `counter unwritten` said what `try:` under it says
        h("small", { class: "try" }, have >= 0 ? /* copy:callout */ "already written" : /* copy:callout */ `try: ${d.counter}`),
        // Cut 26 §6 (AP): the drive-off opens its verdict, as a death's line does (here when no exit line of his carries its own chip)
        // Cut 28 §2: the tablet is above the fold and the exit lines under it — it carries the verdict always
        h("span", { class: "chip mini verdict-chip", role: "button", onclick: (e: Event) => { e.stopPropagation(); closeAllSheets(); const x = allExits.find((y) => y.driven?.boss === d.boss && (d.run_id === undefined || y.run_id === d.run_id)); app.go({ kind: "death", death: drivenDeath(x ?? d, d.run_id ?? x?.run_id ?? 0), kept: true, from: { report: r } }); } }, /* copy:button */ "verdict"));
    })) : null;
  // Stall verdict (core README): every run came home and nothing got deeper — the row that ended them, then patches as on
  // the death screen (tap: replace / remove / insert, camp on the row). The core's line is the copy (≤ 12 words).
  // Cut 30 integration: before the pen the plateau is its line alone, as the death screen before the pen (no trace, no row patches)
  const prePen = !penOpen(L), shopOpen = !prePen || revealed(app).has("unlocks");
  const stall = r.stall ? h("section", { class: "rsec stall" },
    h("div", { class: "label" }, /* copy:label */ "Recent runs"),
    h("div", { class: "stall-line num" }, recentRunText(r.stall.text), " ", prePen ? "" : traceChip(r.stall.trace, "chip mini", { rows: app.rules.rows, runId: stallRun(r), home: true })),
    r.stall.patches.length && !prePen ? h("div", { class: "label stall-next" }, /* copy:label */ "Suggested changes") : null,
    r.stall.patches.length && !prePen ? patchRows(app, r.stall.patches, undefined, undefined, { depth: stallDepth(r.stall.text), plain: true }) : null) : null;
  // Cut 2 §2: one line per pile recovered this send (the core sends `heir 3 · D7 · 4 items`, `bones:7:4` too; the watch
  // `D5 · 7 items`). Every line says it was found — `found ♟3's bones · D7 · 4 items` — since `bones D8 · 11 items · ♟3` read
  // as a pile still lying there (QA on e0f87e7: "survived 16 offline runs", "persisted through run 3")
  const bonesLine = (x: string): string => {
    const m = /^bones:(\d+):(\d+)$/.exec(x); if (m) return /* copy:callout */ `found bones · D${m[1]} · ${items(+m[2])}`;
    const c = /^heir (\d+) · (D\d+) · (\d+) items?$/.exec(x); if (c) return /* copy:callout */ `heir ${c[1]} bones found · ${c[2]} · ${items(+c[3])}`;
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
    // docs/COPY.md pass 5 (`bones on D5 · D8` read "no idea"): what the bones are — dead heirs' packs to recover
    if (!row) return x.replace(/^bones on (D\d+(?: · D\d+)*)$/, /* copy:callout */ "bones to recover: $1");
    return /* copy:death_line */ `${rowLabel(row)} · fired in ${m![2]} of ${m![3]} runs${m![5] ?? ""}`;
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
  // Cut 25 §5: a line the absence merged by its shape carries its count (`Highlight.n`, app.ts `mergeReel`)
  const reel = (xs: { text: string; n?: number }[]): HTMLElement | null => {
    const n = new Map<string, number>(); for (const x of xs) n.set(x.text, (n.get(x.text) ?? 0) + (x.n ?? 1));
    return n.size ? h("ul", { class: "lines" }, ...[...n].map(([x, k]) => h("li", null, x, k > 1 ? h("b", { class: "num" }, ` ×${k}`) : ""))) : null;
  };
  // pending: the engine's lines, with affordable unlocks shown as cards once the catalogue arrives
  const pendingBody = h("div", null);
  const unlockNames = new Map<string, string>();
  const pendingSec = section(/* copy:label */ "pending", pendingBody);
  const paintPending = (affordable: ReturnType<typeof visible>): void => {
    // `R1 fired n of m runs` lines come for every row (summed across slices); only the quiet ones are decisions
    const quiet = (p: string): boolean => { const m = /^R\d+ fired (\d+) of (\d+) runs/.exec(p); return !m || Number(m[1]) * 3 < Number(m[2]); };
    // Cut 30: before the pen no line speaks the pen's words (a patch, a rule's fires, the marks' catalogue)
    const penWords = (p: string): boolean => !penOpen(L) && /^(patch|unlock|R\d+)\b|\bfired\b/.test(p);
    const pendingLines = (affordable.length ? r.pending.filter((p) => !/^unlock\b/.test(p)) : r.pending).filter(quiet).filter((p) => !penWords(p));
    pendingBody.replaceChildren();
    // Cut 23 §1: the core's `forge sword +1 · $300` (a kit step the purse buys now) opens the forge
    const forgeLines = pendingLines.filter((p) => /^forge /.test(p));
    const ul = lines(pendingLines.filter((p) => !/^forge /.test(p)).map((text) => readableUnlock(text, unlockNames))); if (ul) pendingBody.appendChild(ul);
    if (forgeLines.length) pendingBody.appendChild(h("div", { class: "chips forge-pending" }, ...forgeLines.map((p) => h("button", { class: "chip mini forge-line num", onclick: () => openPreparationForge(app) }, p))));
    // Cut 9 §2: the card opens its sheet; the buy is there, and the report repaints itself after one
    // QA 1a2a4a9: the core's short list (`UnlockInfo.short`) when sent — the camp's shelf shows the same three
    const coreShort = affordable.some((u) => u.short !== undefined);
    affordable = affordable.map((u) => withRowsGate(u, app.ownRows(), app.vocab.max_rows)).filter((u) => coreShort ? u.short : u.available);   // Cut 10 §3; Cut 12 §1: own rows
    // QA 23ed91f (L: "PENDING lists the whole unlock shop, identical across five reports"): the next three, as the camp's panel
    // (the larger reach gain, then the cheaper); the camp's `more` has the rest
    if (!coreShort) affordable = affordable.map((u, i) => ({ u, i })).sort((a, b) => (b.u.delta ?? 0) - (a.u.delta ?? 0) || a.u.cost - b.u.cost || a.i - b.i).slice(0, 3).map((x) => x.u);
    // QA 524827b (qaAA: `card: kite archers · free` — "no card anywhere on screen"): the cards are the camp's unlocks — they say so
    if (!shopOpen) affordable = [];   // Cut 30 integration: the unlock cards with the catalogue (the camp shows no unlocks before it opens)
    if (affordable.length) pendingBody.appendChild(h("div", { class: "cards-head num dim" }, /* copy:label */ "unlocks"));
    if (affordable.length) pendingBody.appendChild(h("div", { class: "cards" }, ...affordable.map((u) => h("button", { class: "card", onclick: () => openUnlockSheet(app, u, () => app.go({ kind: "report", report: r })) }, h("span", null, u.label), h("span", { class: "num cost" }, priceLabel(u))))));   // Cut 18 §5: both prices
    if (pendingSec) pendingSec.hidden = !pendingBody.childElementCount;
  };
  paintPending([]);
  // Cut 14 §4: the absence's usage lines are the death screen's drop sheet counts (Cut 15 §3); a watched run's report has none and
  // leaves the watch's own counts
  const usage = r.pending.map((p) => /^R(\d+) fired (\d+) of (\d+) runs/.exec(p)).filter((m): m is RegExpExecArray => !!m);
  if (usage.length) { const fires = app.rules.rows.map(() => 0); for (const m of usage) if (Number(m[1]) - 1 < fires.length) fires[Number(m[1]) - 1] = Number(m[2]); app.rowFires = fires; app.rowFiresOf = Number(usage[0][3]); }
  void app.engine.unlocks().then((cat) => { for (const u of cat) unlockNames.set(u.id, `${unlockLabel(u.id)} · ${priceLabel(u)}`); paintPending(visible(cat, app.lineage).filter((u) => u.available)); }).catch(() => { /* lines only */ });
  // Cut 17 §4: the console — `open` (the worst death's verdict) · `gold` (the ledger of this absence's gold) · `ledger` (the
  // bestiary, from the 5th heir); the gem is `camp`
  const bar = renderBar(app);
  const consTiles = [
      // QA e75ec29 (Q: "`open` opens ♟5's death, not the newest; the label names nothing"): it is the absence's worst death — it says so
      // QA 912e135 (qaW: "`worst` — I read it as the shallowest death"): the tile says what it ranks by — the deepest death (a stall at
      // its floor yields to it)
      r.worst_death ? cmdTile({ id: "open", label: /* copy:button */ "deepest", icon: "trace", onclick: () => app.go({ kind: "death", death: r.worst_death!, lost: r.lost ?? [], from: { report: r } }) }) : null,
      cmdTile({ id: "gold", label: /* copy:button */ "gold", icon: "gold", onclick: () => openGoldSheet(app) }),
      revealed(app).has("heirs") ? cmdTile({ id: "ledger", label: /* copy:button */ "enemy guide", icon: "ledger", onclick: () => openLedger(app) }) : null,
  ];
  const cons = renderConsole({
    portrait: portrait(app, { hp: 1, label: heirOrd(L.heir) }).el,
    gem: gem({ label: /* copy:button */ "town", cls: "camp-gem", pulse: true, onclick: () => app.go({ kind: "camp" }) }),
    tiles: consTiles, compact: true,
  });
  // QA 524827b (qaAA: KEPT `axe +7 → vault` after the cage's `took axe +1`): an item enchant scrolls raised says by how many
  const keptItems: { label: string; enchanted?: number }[] = r.kept ? r.kept.map((label) => ({ label })) : r.new_finds ? r.found : [];
  // Cut 28 §2 (AU: "the report is a wall of salvage/found lines before anything to decide"): the first screen is what changed and what
  // to do — the news, the tiles, the oath's progress, the plateau, a boss's counter (driven off, or newly learned), the bounty, pending;
  // the ledger (exits, gold, salvage, bones, finds, the reel) folds under one `details` tap
  // Cut 29 §1 (E1): the wall's edit lands here when the core's search answers (after the paint; never waited on)
  const wallHost = h("div", { class: "wall-host" });
  if (penOpen(L)) void wallOffer(app).then((w) => { if (w && wallHost.isConnected) replace(wallHost, h("div", { class: "label" }, /* copy:label */ "wall fix"), wallTablet(app, w, () => app.go({ kind: "camp" }))); });
  const details = h("div", { class: "report-details", hidden: !reading.expanded });
  if(r.bloodlines?.length)details.append(h("section",{class:"bloodline-report"},h("div",{class:"label"},/* copy:label */"Bloodlines"),...r.bloodlines.map(s=>h("div",{class:"num"},s.name,/* copy:label */` · ${s.runs} runs · D${s.deepest} · $${s.gold}`))));
  const detailsBtn: HTMLButtonElement = h("button", { class: `details-fold num${reading.expanded ? " on" : ""}`, "aria-expanded": String(reading.expanded), onclick: () => {
    reading.expanded = !reading.expanded; details.hidden = !reading.expanded; detailsBtn.setAttribute("aria-expanded", details.hidden ? "false" : "true"); detailsBtn.classList.toggle("on", !details.hidden);
  } }, h("span", { class: "fold-mark", "aria-hidden": "true" }, "▸ "), /* copy:button */ "details");
  // run-clear: a find or a kept item in its rarity rim — the rarity is the core's (the exit lines' finds, the vault), matched by label
  const rarityOf = new Map<string, InvItem>([...(r.exits ?? []).flatMap((x) => x.finds ?? []), ...L.vault].map((it) => [it.label, it]));
  const withRim = (label: string, text: string): (HTMLElement | string)[] => { const it = rarityOf.get(label); return it ? [itemIcon(it, { size: "s" }), itemName(it, text)] : [itemChip({kind:label, label:text})]; };
  const foundChips = (): HTMLElement | null => {
    const names = r.found.map((i) => i.label);
    if (!names.length) return null;
    const seen = new Map<string, number>(); for (const n of names) seen.set(n, (seen.get(n) ?? 0) + 1);
    return h("div", { class: "chips" }, ...[...seen].map(([label, n]) => h("span", { class: "chip" }, ...withRim(label, named(label).replace(/_/g, " ")), n > 1 ? h("b", { class: "num" }, ` ×${n}`) : "")));
  };
  const learnedFacts = r.learned.filter((f) => !/^bones:\d+$/.test(f));
  const counterFacts = learnedFacts.filter((f) => /^boss:[^:]+:counter/.test(f) || /^counter_hint:/.test(f));
  const runGold = r.gold?.home ?? (r.exits ?? []).reduce((n, x) => n + x.kept, 0);
  const lootGold = r.gold?.salvage ?? (r.salvaged ?? []).reduce((n, x) => n + x.gold, 0);
  // blind ad71e72 (A: `$8 GOLD EARNED` while the purse rose ~$1000 on a D19 start; `$6712` with the purse up $78): the waystone passage
  // paid at the send is income too (the core's `gold.passage`, a watched run's `live.run.passage`), and an absence states the purse's
  // actual change under it (`gold.net`: the workers' forge steps, hires and bank moves included)
  const passageGold = reportPassage(r);
  const net = r.gold?.net;
  const otherGold = net === undefined || !r.gold ? 0 : net - (r.gold.home + r.gold.salvage + passageGold + r.gold.wake - r.gold.spent);
  const signed = (n: number): string => `${n < 0 ? "−" : "+"}$${Math.abs(n)}`;
  // blind c4705f9 (A, B: `purse −$12562` from a 20 min absence, unexplained): the workers' purchases are named under the purse
  // (`forge −$12562`), and the rest of the other movements apart
  const boughtGold = net === undefined ? 0 : workersSpent(r.workers);
  const earnedGold = detailHost(h("button", { type: "button", class: "tile plaque report-gold", "data-k": "gold", onclick: () => openGoldSheet(app) },
    icon("gold"), h("b", { class: "num" }, `$${runGold + lootGold + passageGold}`), h("span", { class: "label" }, /* copy:label */ "Gold earned"),
    net !== undefined && net !== runGold + lootGold + passageGold ? h("small", { class: "report-net num" }, /* copy:callout */ `purse ${signed(net)}`) : "",
    boughtGold > 0 ? h("small", { class: "report-bought num" }, /* copy:callout */ `forge −$${boughtGold}`) : ""), () => [
      h("div", { class: "kw-tip-head" }, h("b", null, /* copy:label */ "Gold earned")),
      h("div", { class: "num" }, /* copy:label */ "Run gold", ` · $${runGold}`),
      h("div", { class: "num" }, /* copy:label */ "Loot sold", ` · $${lootGold}`),
      ...(passageGold > 0 ? [h("div", { class: "num" }, /* copy:label */ "Passage", ` · $${passageGold}`)] : []),
      ...(net !== undefined && r.gold ? [h("div", { class: "num" }, /* copy:label */ "Heir grants", ` · +$${r.gold.wake}`)] : []),
      ...(net !== undefined && r.gold ? [h("div", { class: "num" }, /* copy:label */ "Supplies, tolls", ` · −$${r.gold.spent}`)] : []),
      ...(boughtGold > 0 ? [h("div", { class: "num" }, /* copy:label */ "Apprentice forge", ` · −$${boughtGold}`)] : []),
      ...(otherGold + boughtGold ? [h("div", { class: "num" }, /* copy:label */ "Workers, other", ` · ${signed(otherGold + boughtGold)}`)] : []),
      ...(net !== undefined ? [h("div", { class: "num" }, h("b", null, /* copy:label */ "Purse change"), ` · ${signed(net)}`)] : []),
      h("div", { class: "kw-tip-gloss" }, net !== undefined ? /* copy:tooltip */ "Earned before spending; purse change counts everything" : /* copy:tooltip */ "Before spending; excludes heir grants")]);
  const summary = h("div", { class: "report-summary" },
    h("h2", null, absence ? /* copy:label */ "While away" : r.runs === 1 && deathsN ? /* copy:label */ "You died" : /* copy:label */ "Delve ended"),
    h("div", { class: `tiles report-basics${absence ? " fade-in" : ""}` },
      toLog(tile(String(r.runs), /* copy:label */ "runs")),
      tile(`D${r.deepest ?? L.best_depth}`, r.deepest !== undefined ? /* copy:label */ "deepest" : /* copy:label */ "record"),
      earnedGold));
  const repeatedDeath = [...r.deaths].sort((a, b) => b.n - a.n).find(d => d.n >= 2);
  const obstacle = r.stall ? recentRunText(r.stall.text) : repeatedDeath?.cause.replace(/_/g, " ");
  const preparation = preparationActions(app, { report: true, obstacle, cause: r.stall ? undefined : repeatedDeath?.cause });
  const upgradeHost = preparation.el;
  const firstActs = (r.workers ?? []).filter((a) => a.first && (a.n > 0 || a.what));
  const goal = progressGoal(L);
  const newChoices = reportChoices(app, r);
  const firstWorkers = firstActs.length ? h("section", { class: "report-first-workers" },
    h("b", { class: "row-label" }, /* copy:label */ "Workers started"),
    workersBlock(L, { workers: firstActs }, firstActs.length)) : null;
  const sheet = h("div", { class: "parchment report-sheet" },
    // Cut 30 §4: the report leads with what grew on each track (and the packages' beats); the oath's progress is an older core's
    summary, goal ? progressGoalRow(goal, "report-progress-goal") : null, r.restock_capped && reportIncome(r) === 0 ? supplyLimit(app, r, true) : null, reportBosses(r, app.lineage), classXpBlock(r), legacyEarnedBlock(r), newChoices.el, upgradeHost, reportTrainingBlock(r), firstWorkers,
    detailsBtn, details);
  // Cut 29 §3: the night's meters (an absence: its real runs summed), a watched run's own — under `details` on the phone, beside the
  // shaft on the desktop
  const meterOf = r.meters ?? (r.exits?.length === 1 ? r.exits[0].meters : undefined);
  const meterTitle = /* copy:label */ "Completed runs";
  const meterScope = (): HTMLElement => h("span", null, h("span", null, absence ? /* copy:callout */ "Includes before away" : /* copy:label */ "Whole runs"), " · ", h("span", null, /* copy:callout */ "Camp rest separate"));
  details.append(...[pendingSec, tiles, grewBlock(r, heroFace(L), trainingBeats(r.packages)), workersBlock(L, { workers: (r.workers ?? []).filter((a) => !a.first), chest: r.chest }), newsBlock(r, named, L.counters ?? [], shopOpen, !prePen), opened(r), wallHost, onPackages(L) ? null : oathProgress(app, r), fallenLines(r), stall, driven, counterFacts.length ? section(/* copy:label */ "counters", factChips(counterFacts, L.counters ?? [])) : null, bounty, startShort, meterOf && !isWide() ? meterPanel(meterOf, app.rules.rows, { title: meterTitle, scope: meterScope() }) : null, goldLine(), picked, exitLines, rested,
    // QA 23ed91f (K, L: `bones D7` among LEARNED): a heir's bones are a find (the BONES section), not a fact learned
    section(/* copy:label */ "learned", factChips(learnedFacts.filter((f) => !counterFacts.includes(f)), L.counters ?? [], (app.vocab?.locked ?? []).find((l) => l.cond.k === "alert>=" && /^◆\d+/.test(l.needs))?.needs)),
    section(/* copy:label */ "tamed", chips(r.tamed ?? [], "chip ally")),
    section(/* copy:label */ "hatched", chips(r.hatched ?? [], "chip ally")),
    // Cut 10 §3: a companion `◯ jackal · Ashar fell` (the name small); Cut 12 §6: a summoned ally `ally hound fell`
    section(/* copy:label */ "lost", chips((r.lost ?? []).map((k) => k.includes(" · ") ? /* copy:callout */ `◯ ${k} fell` : lostLabel(k)), "chip egg")),
    section(/* copy:label */ "bests", lines(collapseBests(r.bests).map(bestLabel))),
    lanesSection(app, r),
    r.xp && (r.xp.gained > 0 || r.xp.level_ups > 0) ? section(/* copy:label */ "xp", h("div", { class: "xp-line num" }, `${r.xp.class} +${r.xp.gained}`, " · ", /* copy:label */ `L${L.classes?.[r.xp.class]?.level ?? 1}`, r.xp.level_ups > 0 ? h("b", null, ` ↑${r.xp.level_ups}`) : "")) : null,
    // QA 0c6e126 (qaY: the header's `new find: bow, leather` beside FOUND `mail +1` — the item the send brought from the vault): an
    // absence's FOUND is its first finds (`ReturnReport.new_finds`, the core's, every one the header's `new find` names); what went to
    // the vault is KEPT (`→ vault`)
    section(/* copy:label */ "found", r.new_finds ? chips(r.new_finds.map(named)) : foundChips()),
    // QA e75ec29 (R: six thefts in one run, "the report and gold sheet say nothing"): what thieves took and no run got back
    // QA a946e04 (S: `leash ×4` beside `leash (2)`, `black potion?` after LEARNED said confusion): one chip per name, identified kinds by
    // their name; T (`−$36 stolen` on the strip, only items here): the carry the thefts took leads (`$36`)
    section(/* copy:label */ "stolen", r.stolen?.length || r.stolen_gold ? h("div", { class: "chips" },
      // QA 524827b (qaAA: `heal ×2 $8` read as the heals' value): the thefts' toll is the carry's, once (`carry −$35`, the watch strip's
      // word); each chip names what went, coins as `coins`
      r.stolen_gold ? h("span", { class: "chip stolen gold num" }, /* copy:callout */ `carry −$${r.stolen_gold}`) : "",
      ...mergeRows((r.stolen ?? []).map((x) => ({ kind: named(x.label.replace(/\s*\(\d+\)$/, "")), n: x.n, gold: x.gold ?? 0 }))).map((x) => h("span", { class: "chip stolen" }, itemChip({kind:x.kind, label:x.kind === "gold" ? /* copy:label */ "coins" : x.kind}), x.n > 1 ? h("b", { class: "num" }, ` ×${x.n}`) : ""))) : null),
    section(/* copy:label */ "bones", (r.bones_found ?? []).length ? h("div", null, conceptTag("bones"), lines((r.bones_found ?? []).map(bonesLine))) : null),
    section(/* copy:label */ "deaths", r.deaths.length ? h("ul", { class: "lines" }, ...r.deaths.map((d) => h("li", null, d.cause.replace(/_/g, " "), " ", h("b", { class: "num" }, `×${d.n}`)))) : null),
    // Cut 24 §5 (AK, AL: the tapped chip read as salvaged — a twin or the return's cut sold, the kept one renamed by the vault): what
    // the keep sheet sent to the vault leads the sell-off
    section(/* copy:label */ "kept", keptItems.length ? h("div", { class: "chips kept" }, ...keptItems.map((x) => h("span", { class: "chip kept" }, ...withRim(x.label, named(x.label)), flavourTag(L, x.label), /* copy:callout */ " → storage",
      x.enchanted && x.enchanted > 0 ? h("small", { class: "num dim enchanted" }, /* copy:callout */ ` · enchanted ×${x.enchanted}`) : ""))) : null),
    // Cut 21 §2: found supplies the exits put on the shelf (the next send packs them free), before what was sold
    section(/* copy:label */ "shelved", r.shelved?.length ? h("div", { class: "chips shelved" }, ...r.shelved.map((x) => h("span", { class: "chip shelf" }, /* copy:callout */ "found ", itemChip({kind:x.kind, label:x.kind.replace(/_/g, " ")}), x.n > 1 ? h("b", { class: "num" }, ` ×${x.n}`) : "", /* copy:callout */ " → supplies"))) : null),
    section(/* copy:label */ "salvaged", r.salvaged?.length ? h("ul", { class: "lines" }, ...mergeRows(r.salvaged.map((x) => ({ ...x, kind: named(x.kind) }))).map((s) => h("li", null, itemChip({kind:s.kind, label:s.kind.replace(/_/g, " ")}), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num gold" }, `$${s.gold}`)))) : null),
    // Cut 13 §3: what the automations bought this absence, per kind (`heal ×16 · −$640`)
    section(/* copy:label */ "spent", r.spent?.length ? h("ul", { class: "lines" }, ...r.spent.map((s) => h("li", null, itemChip({kind:s.kind, label:s.kind.replace(/_/g, " ")}), " ", h("b", { class: "num" }, `×${s.n}`), " · ", h("span", { class: "num down" }, `−$${s.gold}`)))) : null),
    // QA 524827b (qaAB: `+40 · ★0`, `+51 · ★0`, then `+102 · ★1 ↑1` — "no threshold on screen"): the renown toward the next ★ (the core's
    // rule: rank n+1 at 100·(n+1)² renown) — `★0 · 91/100`
    section(/* copy:label */ "reputation", r.renown && r.renown.gained > 0 ? h("div", { class: "num" }, `+${r.renown.gained} · ★${r.renown.rank}`, r.renown.ranks_up > 0 ? h("b", { class: "up" }, ` ↑${r.renown.ranks_up}`) : "", r.renown.ranks_up > 0 ? ` · ◆+${r.renown.ranks_up}` : "",
      typeof L.renown === "number" ? h("small", { class: "dim next-rank" }, ` · ${L.renown}/${100 * ((L.rank ?? r.renown.rank) + 1) ** 2}`) : "") : null),   // a rank pays a mark: the tiles' ◆ reconciles with the rows (QA on 56f2a1d: ◆+9 vs rows ◆+6)
    section(/* copy:label */ "reel", reel(r.reel.map((x) => ({ text: readableUnlock(noteText(x.text)), n: x.n })))),
  ].filter((x): x is HTMLElement => !!x));
  detailsBtn.hidden = !details.childElementCount;
  const wide = wideCols(app, meterOf ? meterPanel(meterOf, app.rules.rows, { title: meterTitle, scope: meterScope() }) : null);   // desktop: the rules left, the shaft right (wide.css)
  const reportWell = h("div", { class: "well report-well" }, sheet);
  const el = h("main", { class: "report frame" }, bar.el, reportWell, cons.el, ...wide.els);
  const gemEl = cons.el.querySelector<HTMLElement>(".gem")!;
  // run-clear (the owner: "each run should have the clear screen"): the run's card over the report — a watched run's, or an absence's
  // last; its own clock, then the report's
  const clear = mountClear(app, r, absence, reportWell, gemEl, () => cons.setTiles(consTiles));
  if (clear.tile) cons.setTiles([clear.tile, ...consTiles]);
  if (!clear.shown) autoDismiss(gemEl, { ms: AUTO.report, yieldToSheets: true });   // docs/UI.md §7: on to the town
  const restore = requestAnimationFrame(() => { if (reportWell.isConnected) reportWell.scrollTop = reading.scroll; });
  return { el, dispose: () => { cancelAnimationFrame(restore); reading.scroll = reportWell.scrollTop; preparation.dispose(); newChoices.dispose?.(); bar.dispose(); wide.dispose(); } };
}

/** Cut 29 §6 (AX: Greth the tamed ogre, L5, gone with only `party −1 ogre`): each companion that fell, by name — `Greth · ogre L5 · fell D12
 *  to lurker` — among the decisions, not the ledger. */
function fallenLines(r: ReturnReport): HTMLElement | null {
  const f = r.fallen ?? [];
  if (!f.length) return null;
  return h("section", { class: "rsec fallen-sec" }, h("div", { class: "label" }, /* copy:label */ "fallen"),
    h("ul", { class: "lines fallen" }, ...f.map((x) => h("li", { class: "fallen-line num" }, h("b", null, x.name), ` · ${x.kind.replace(/_/g, " ")} L${x.level} · `, h("span", { class: "dim" }, x.why)))));
}

/** Cut 29 §2: the systems this absence opened — each its icon and name on a plaque that glints once (no tutorial text: the camp's tile or
 *  tablet is where it is used). */
function opened(r: ReturnReport): HTMLElement | null {
  const ids = (r.systems_opened ?? []).filter((id) => !["send", "dial", "headline"].includes(id));
  if (!ids.length) return null;
  // gfx round 22 (raters, every round: "OPENED chip rows are noisy" — "group them into one row of icons"): past four, one row of
  // medallions (the names stay as their titles)
  return h("div", { class: ids.length > 4 ? "sys-opened icons" : "sys-opened" }, h("small", { class: "label dim" }, /* copy:label */ "opened"),
    ...ids.map((id, i) => { const [ic, gl] = systemIcon(id); return h("span", { class: "sys-plaque reveal", "data-sys": id, title: systemLabel(id), style: `animation-delay:${0.15 * i}s` }, icon(ic, gl), h("span", { class: "sys-name" }, systemLabel(id))); }));
}

/** QA 524827b (qaAB: kept `crimson scroll?`, the report's KEPT `summon ally scroll → vault` — "no line that it was identified"): a kept
 *  potion or scroll names the flavour it was known by, as LEARNED does (`summon ally scroll (crimson)`); nothing for gear. */
function flavourTag(L: Pick<Lineage, "vault" | "facts">, label: string): HTMLElement | "" {
  const v = (L.vault ?? []).find((x) => x.label === label); if (!v) return "";
  const f = (L.facts ?? []).find((x) => x.startsWith("item:") && x.endsWith(`=${v.kind}`));
  return f ? h("small", { class: "dim flavour" }, ` (${f.slice(5, f.indexOf("=")).replace(/_/g, " ")})`) : "";
}

/** QA 92eb880 (N: "plateau patches … `reach 17%` — reach of which floor?"): the floor past the plateau (`none past D6` → 7). */
export function stallDepth(text: string): number | undefined {
  const m = /\bpast D(\d+)\b/.exec(text) ?? /\bD(\d+)\b/.exec(text);
  return m ? Number(m[1]) + 1 : undefined;
}

/** Display the core's window counts, including runs before this absence. */
export function recentRunText(text: string): string {
  return text.replace(/^R\d+ bank ended /, /* copy:callout */ "Recent full hauls · ")
    .replace(/^R\d+ return ended /, /* copy:callout */ "Recent returns · ")
    .replace(/^R(\d+) /, "Rule $1: ")
    .replace(/none past D(\d+)/, "none beyond floor $1")
    .replace(/(\d+) before;/, "$1 earlier;");
}

/** The run a stall's trace belongs to: the exit line that carries the same trace (the stall has no run id on the wire; the
 *  same turns, tick for tick, name the run), so its chain links can open the replay when the client holds that run. */
function stallRun(r: ReturnReport): number | undefined {
  const t = r.stall?.trace; if (!t) return undefined;
  const key = JSON.stringify(t.turns);
  return r.exits?.find((x) => x.trace && JSON.stringify(x.trace.turns) === key)?.run_id;
}

/** The floor events a bare fact names (`lock`, `shrine`): each opens its `on see` condition. */
const SEEN_FACTS = ["den", "lock", "captive", "nest", "shrine", "stray", "hunger"];
/** Facts grouped for reading: `foe:x`, `foe:x:t1`, `foe:x:t2` → one chip "x · t1 · t2"; `item:f=k` → "k (f)";
 *  `biome:x` → "x"; `boss:x:counter[=row]` → "x counter: attack boss" (Cut 6 §5: the lineage's counter text names the row);
 *  others verbatim. */
function factChips(facts: string[], counters: Counter[] = [], alertLock?: string): HTMLElement | null {
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
    if (it) { itemChips.push(h("span", { class: "chip fact item" }, itemChip({kind:it[2], label:nice(it[2])}), h("small", null, ` (${nice(it[1])})`))); continue; }
    const b = /^biome:(.+)$/.exec(f);
    if (b) { rest.push(h("span", { class: "chip fact" }, nice(b[1]))); continue; }
    const bn = /^bones:(\d+)$/.exec(f);
    if (bn) { rest.push(h("span", { class: "chip fact" }, /* copy:label */ "bones", h("small", null, ` D${bn[1]}`))); continue; }
    const c = /^boss:([^:]+):counter(?:=.*)?$/.exec(f);
    if (c) { bossCounters.set(c[1], counters.find((k) => k.boss === c[1])?.text ?? ""); continue; }
    // QA 912e135 (qaW: LEARNED `vault`, `alert · rising`, `counter · gas>pack` named no action or number): each says what it is and what it
    // opens — the cage seen (its tablet), the alert rising (`cond: alert`), a tag that beats another (the companions' counters)
    if (f === "vault") { rest.push(h("span", { class: "chip fact" }, /* copy:callout */ "loot choice seen")); continue; }
    // QA 0c6e126 (qaY: `alert rises · alert ≥ open`, `lock`, `shrine` on their own lines, tied to nothing): each says what it opens — the
    // `alert ≥` card for sale; a floor event seen, the `on see` condition that names it
    // QA 524827b (qaAA: `unlocks alert ≥` read as a verb with no object): the fact, then the cond it makes writable (`cond alert ≥`)
    // QA 308f045 (qaAC: `alert rises · cond alert ≥`, and the cond picker still read `⊘ alert ≥ ◆2`): while the cond is still for sale the
    // fact says its gate (`cond alert ≥ · ◆2`), the picker's own words
    if (f === "alert:rising") { rest.push(h("span", { class: "chip fact" }, /* copy:callout */ "alert rises", h("small", null, /* copy:callout */ " · cond alert ≥", alertLock ? h("span", { class: "gate" }, ` · ${alertLock}`) : ""))); continue; }
    if (SEEN_FACTS.includes(f)) { rest.push(h("span", { class: "chip fact" }, /* copy:callout */ `${nice(f)} seen`, h("small", null, /* copy:callout */ ` · on see ${nice(f)}`))); continue; }
    const ct = /^counter:([^>]+)>(.+)$/.exec(f);
    // QA 0c6e126 (qaZ: `pack beats lone` with no source): a companion's counter — it says whose (`· allies`)
    // QA 524827b (qaAA: `gas beats pack · allies` — "not a fact I could read"): a tag against a tag, as a fact — `gas beats packs`,
    // `undead resist poison` (the immune pairs); the tags are the foes' LEARNED tags
    if (ct) { const immune = ["water>fire", "undead>poison"].includes(`${ct[1]}>${ct[2]}`); const b = ct[2] === "pack" ? /* copy:none */ "packs" : nice(ct[2]);
      rest.push(h("span", { class: "chip fact" }, immune ? /* copy:callout */ `${nice(ct[1])} resist ${b}` : /* copy:callout */ `${nice(ct[1])} beats ${b}`)); continue; }
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
  const out = [...foes].map(([k, tags]) => h("span", { class: "chip fact" }, nice(k), tags.length ? h("small", null, ` · ${tags.map(t => enemyTraitName(k, t)).join(" · ")}`) : ""));
  if (!out.length && !rest.length && !itemChips.length) return null;
  return h("div", { class: "facts" }, out.length + rest.length ? h("div", { class: "chips" }, ...out, ...rest) : "", itemChips.length ? h("div", { class: "chips items" }, ...itemChips) : "");
}

/** QA 23ed91f (K: "a lone `D8` and `rank 2` in among the trophies"): a depth best reads `new best D8`, a rank `★ rank 3`; trophies as sent. */
export const bestLabel = (x: string): string => /^D\d+$/.test(x) ? /* copy:callout */ `new best ${x}` : /^rank \d+$/.test(x) ? `★ ${x}` : x.replace(/^boss:\s*([a-z][a-z0-9_]*)$/, (_all, id: string) => /* copy:callout */ `${bossName(id)} defeated`);

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
