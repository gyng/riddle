// Death: cause line · ledger line (Cut 6 §1) · last-5 trace (hero actions, t = tick) · row accounting of the last action
// (Cut 6 §3: `R1 none held · R2 no path`, engine data) · candidate patches (tap to insert) · edit · morgue.
// Cut 9 §7: also reached from the chronicle sheet for a kept death (`engine.death(id)`, `Screen.kept`); `edit` leads back to
// the camp, and so does Escape with no sheet open (app.ts; QA on 952e306: "old death screen, no back/close, Escape inert").
// Cut 11 §2: under the trace the row accounting is the chain (ui/chain.ts) — each `because` links the replay when this
// session watched the run; root patches and unlock pseudo-patches (ui/patches.ts). §5: the ledger line opens the gold
// sheet filtered to this run.
// Cut 13 §1: a stalled run's verdict (`verdict: "stall"`) mounts here too — the guard's moment as the headline, the trace and
// the patches like a death's. §4: the run's last two notes (`Death.notes`, engine data verbatim) sit under the headline.
// §5: a `dice` death names what the forecast said for that depth — the camp's own reach line, verbatim (`forecast said D4 100%`)
// when the last forecast knows the floor (QA on 50bb162: "`forecast said 36%` while the camp forecast read `D4 100% ±1`").
import { counterName } from "./counter-name";
import { trainingBlock } from "./tracks";
import { meterPanel } from "./meters";
import { AUTO, autoDismiss } from "./autodismiss";
import type { App, Mounted } from "../app";
import type { Death, DrivenOff, ExitLine, Patch, ReturnReport, Row } from "../engine/types";
import { morgueVerbs } from "./chain";
import { lastRun, replayable } from "./runlog";
import { openReplay } from "./replay";
import { h, copyText, items } from "./dom";
import { clearStrip } from "./runclear";   // run-clear: the death's header strip
import { lowOf, share } from "./forecast";
import { openGoldSheet } from "./gold";
import { applyOf, fillReach, leadFirst, openDropSheet, patchOf, patchRows, sinkHarms } from "./patches";
import { closeX, openSheet, openWindow } from "./sheet";
import { gem, portrait, renderBar, renderConsole, tile, wideCols } from "./frame";
import { lostLabel, noteText, refName, rowLabel, ruleName, setRefRows, verbLabel } from "./tokens";
import { traceTable } from "./trace";
import { mergeFinds, renamer } from "./report";
import { foeSrc, packageIcon } from "./skin";
import { enemyHost } from "./enemy-tips";
import { unitLabel, unitPortrait } from "./unit-icon";
import { itemIcon } from "./items";
import { openPackages, penOpen } from "./packages";
import { openPreparationForge, preparationActions } from "./preparation";
import { kwHost } from "./tips";
import { TIP, type Term } from "./concepts";
/** docs/TOOLTIPS.md: the seal's verdict as a tip (long-press / hover: the seal's tap stays its own) */
const VERDICT_TERM: Record<string, Term> = /* copy:none */ { gap: "v_gap", dice: "v_luck", row: "v_rule", order: "v_order", stall: "v_stall", route: "v_route", driven: "v_repelled" };

/** Cut 10 §3: the core's `3 over` margin reads `3 hp short` wherever it is displayed (`N hp short` and others pass through). */
export const marginText = (m: string): string => m.replace(/^(\d+) over$/, /* copy:callout */ "$1 hp short");

/** The headline's margin segment: a stall's is the guard's reason (`no path`); an hp margin (`3 over` / `3 hp short`) is left
 *  out — four QA players read `1 hp short` as the hp left (the morgue still carries it); an empty margin is no segment. */
export const headlineMargin = (m: string): string => m.split(" · ").filter((x) => x && !/^\d+ (over|hp short)$/.test(x)).map(marginText).join(" · ");   // QA 23ed91f: `1 hp short · 5 unknown unused` kept its hp part

/** Name the core-selected historical action without exposing its row syntax. */
export function deathAction(d: Death): string {
  if (!d.package) return "";
  const parts = d.package.split(" · ");
  const source = parts.slice(0, parts[0] === "drill" ? 2 : 1).join(" · ");
  const index = d.cause_row ?? [...d.trace.turns].reverse().find((t) => t.row >= 0)?.row;
  const rows = d.rules?.rows;
  const turn = index === undefined ? undefined : [...d.trace.turns].reverse().find((t) => t.row === index);
  const action = index !== undefined && rows?.[index] ? ruleName(rows, index) : turn ? verbLabel(turn.verb) : "";
  // Cut 115 §1: whether the deciding row was the player's pick (`picked`), a drill's or a fix's (`taught`), the school's (`default`)
  const who = d.credit ? ` · ${d.credit}` : "";
  return action ? `${source} · ${action}${who}` : `${source}${who}`;
}

/** QA 23ed91f (K: "`died $0` and `keeps 0%` say the same thing twice"): a death's line drops its `keeps 0%` (the lead says it). */
/** QA 1a2a4a9 (O, P, and many before: "`+$40 wake` — no source"): the core's wake pay tops the next heir's purse up to $40 — it reads
 *  as whose it is (`heir purse +$40`), here, on the gold sheet and in the report. */
export const wakeShown = (t: string): string => t.replace(/\+\$(\d+) wake\b/g, /* copy:callout */ "next heir +$$$1").replace(/\bwake pay\b/g, /* copy:callout */ "next heir").replace(/\bpurse full\b/g, /* copy:callout */ "no top-up");
/** QA 0c6e126 (qaZ: `died $0 · $61 lost` — "I read `died $0` as died carrying $0"): the kept and the carried are named apart —
 *  `died · kept $0 · $61 lost`. */
export const ledgerShown = (t: string): string => wakeShown(/^died \$0\b/.test(t) ? t.replace(/ · keeps 0%(?= · |$)/, "").replace(/^died \$0\b/, /* copy:callout */ "died · kept $$0") : t);
/** QA e75ec29 (R: a packed heal stolen on D1, nothing on the exit; `+$40 heir purse` once, then none): what the core adds to an exit
 *  line beside its text — `· stolen heal` (what thieves took and kept), the purse's word, the swaps' toll, the shelved finds.
 *  QA 778fa1b (qaU): `purse full` (read as the camp's $999 cap) is `no top-up` — the core flags it only for a purse under $80; the
 *  top-up is `ExitLine.wake` (`heir purse +$40`, when the text does not say it); a pack swap's cost (`−$37 swapped`, on the watch's
 *  strip and nowhere after) rides the line; the coins thieves kept join the stolen list (`stolen scroll, $3` — STOLEN $3 matched no
 *  line); `name` reads a stolen flavour by its name now (`amber potion?` → `caustic`, LEARNED's word). */
const dTag = (d: number): string => /* copy:none */ `D${d}`;
/** `brief` (the report's lines, QA 0c6e126 qaY: "16 runs of death lines each list 10–20 item names; the lines that differ are buried"):
 *  the `left` finds and the pile print as counts (`left 3`), the core's `bones: N items on D6` already naming the pile. */
export type ExtrasOpts = { brief?: boolean; /** the pile's list is already in the text (`lineShown`) */ pileShown?: boolean;
  /** docs/COPY.md pass 6: the death screen's line — kept, lost, the bones and the thefts; swaps and finds left behind are the gold sheet's */ lean?: boolean };
export const exitExtras = (x: Pick<ExitLine, "text" | "stolen" | "stolen_gold" | "purse_full" | "wake" | "swapped" | "swap_left" | "shelved" | "start" | "start_short" | "found" | "bones">, name: (label: string) => string = (l) => l.replace(/_/g, " "), opts: ExtrasOpts = {}): string => {
  // QA 912e135 (qaW: `stolen blink, $3` read as the blink's worth); QA 524827b (qaAA: `stolen pearly potion? + $3` still read as the
  // potion's value): what the thefts took off the carry is the watch strip's own term, `carry −$3` — never a price beside an item
  const items = (x.stolen ?? []).map(name), coins = x.stolen_gold && x.stolen_gold > 0 ? x.stolen_gold : 0;
  // QA 0c6e126 (qaZ): stolen gear carries its enchant (`leather +1`)
  const stolen = [items.length ? /* copy:callout */ `stolen ${items.join(", ")}` : "", coins ? /* copy:callout */ `stolen $${coins}` : ""].filter(Boolean).join(" · ");
  // QA a946e04 (T: a D5 start at $32 ran from D1 with no word): the run's start fell back — the toll was more than the purse
  return (x.start_short && !/toll short/.test(x.text) ? /* copy:callout */ ` · from ${dTag(x.start ?? 1)} · toll short` : "")
    + (stolen && !/\bstolen\b/.test(x.text) ? ` · ${stolen}` : "")
    // QA 0c6e126 (qaY: `−$5 swapped` named no item): the costly swaps name what they left on the floor. QA 524827b (qaAA: `−$1 left
    // leather` beside `left 14` — "left" twice, a `−$` the gold sheet never shows): a swap reads as one (`swapped out leather`), its
    // cost the carry's (`carry −$1`, the strip's word: no purse movement); `left` is only the finds left behind
    + (x.swapped && x.swapped > 0 && !opts.lean && !/\bswapped\b/.test(x.text) ? x.swap_left?.length ? /* copy:callout */ ` · swapped out ${shelvedText(x.swap_left.map((b) => ({ kind: name(b.kind), n: b.n })))}${opts.lean ? "" : ` · paid $${x.swapped}`}` : opts.lean ? "" : /* copy:callout */ ` · swapped · paid $${x.swapped}` : "")
    + (x.wake && x.wake > 0 && !/\bwake\b|heir purse/.test(x.text) ? /* copy:callout */ ` · next heir +$${x.wake}` : "")
    // QA 912e135 (qaW: `no top-up` on every line — "the top-up it refers to is never shown"); QA 0c6e126 (qaY: `heir purse ≥$40` — "the
    // `≥` has no source", twelve of them against one `+$20 heir purse` in the ledger): only a top-up the ledger carries is named (`wake`)
    // QA 0c6e126 (qaZ: "no shelf appears anywhere"): the loadout's list is SUPPLIES
    + (x.shelved?.length && !/→ (shelf|supplies)\b/.test(x.text) ? /* copy:callout */ ` · found ${shelvedText(x.shelved)} → supplies` : "")
    + (opts.lean ? "" : fatesText(x.found, x.bones ? undefined : "bones", opts.brief))
    // QA 912e135 (qaW: `bones: 12 items on D6` listing 14 — `leash ×3` counted charges; `11 items` listing 10 — only the finds): the
    // pile the count counts, one an item (`ExitLine.bones`, the core's); an older line keeps its finds' list
    // QA 0c6e126 (qaY: `bones leash, violet potion?, …` — "`bones leash` reads as one item"): the pile's list after a colon; brief, the
    // core's own `bones: N items on DX` says it
    + (x.bones?.length && !opts.brief && !opts.pileShown ? /* copy:callout */ ` · bones: ${shelvedText(x.bones.map((b) => ({ kind: name(b.kind), n: b.n })))}` : "");
};
/** QA 778fa1b (V: found items that ended in no named place): the finds the other words don't place — `· left mail · bones leash`.
 *  QA 912e135: `bones` only for a line without the pile's own list (`ExitLine.bones` names every item of it). */
const fatesText = (xs: ExitLine["found"], bones?: "bones", brief = false): string => (["left", ...(bones ? [bones] : [])] as const).map((f) => {
  const ys = (xs ?? []).filter((y) => y.fate === f && y.n > 0);
  if (!ys.length) return "";
  // QA 524827b (qaAA: `left dagger`, `left 14` — "left" as a verb and a bare count): the finds left behind, with the unit when counted
  if (f === "left") return brief ? /* copy:callout */ ` · ${ys.reduce((a, y) => a + y.n, 0)} left behind` : /* copy:callout */ ` · left behind: ${shelvedText(ys)}`;
  return brief ? /* copy:callout */ ` · ${f} ${ys.reduce((a, y) => a + y.n, 0)}` : /* copy:callout */ ` · ${f}: ${shelvedText(ys)}`;
}).join("");
/** Cut 21 §2 (AE: "sells heal potions he finds for $2 while I pay $40"): found supplies the exit put on the shelf — `heal ×2, fire`. */
export const shelvedText = (xs: { kind: string; n: number }[]): string => xs.map((y) => `${y.kind.replace(/_/g, " ")}${y.n > 1 ? ` ×${y.n}` : ""}`).join(", ");
export const lineShown = (x: ExitLine, name?: (label: string) => string, opts: ExtrasOpts = {}): string => {
  // QA 0c6e126 (qaY): the pile once — the core's `bones: 10 items on D5` becomes `bones on D5: leash, sword, …` where the list is known
  const nm = name ?? ((l: string) => l.replace(/_/g, " "));
  const m = !opts.brief && x.bones?.length ? /\bbones: \d+ items? on (D\d+)/.exec(x.text) : null;
  const text = m ? x.text.replace(m[0], /* copy:callout */ `bones on ${m[1]}: ${shelvedText(x.bones!.map((b) => ({ kind: nm(b.kind), n: b.n })))}`) : x.text;
  return ledgerShown(text) + exitExtras(x, name, { ...opts, pileShown: !!m });
};

export function renderDeath(app: App, d: Death, lost: string[] = [], kept = false, from?: { report: ReturnReport; absence?: boolean }): Mounted {
  setRefRows(() => d.rules?.rows ?? app.rules.rows);   // docs/COPY.md §2: the core's `R2` on this screen names the rules that ran
  const drove = isDriven(d) ? d.line!.driven! : undefined;   // Cut 26 §6: a drive-off's verdict (below)
  // Cut 30 §2: before the pen opens the screen is the cause and ONE cheapest lever (`Death.lever`: spend · package · wait); the trace,
  // `gap`/`dice`, the replays and the fixes' odds come with the pen
  const prePen = !penOpen(app.lineage);
  // a stall's margin is the guard's reason (or empty): the headline never carries an empty segment
  const seg = headlineMargin(d.margin ?? "");
  const margin = seg ? ` · ${seg}` : "";
  // Cut 19 §4: a `row` verdict — the player's own row was the dying action; the headline names it (`R2 drink unknown`), the seal reads ROW
  // Cut 25 §2: an `order` verdict names both rows — the one that would have acted and the one above that won every tick (`R5 under R2`)
  const causeRow = d.verdict === "row" && d.cause_row !== undefined ? rowName(d.cause_row, (d.rules?.rows ?? app.rules.rows)[d.cause_row], d.rules?.rows ?? app.rules.rows)
    : d.verdict === "order" && d.cause_row !== undefined && d.order_over !== undefined ? /* copy:death_line */ `${refName(d.cause_row)} under ${refName(d.order_over)}`
    // Cut 26 (core, risks): a `route` verdict — the far stair the set's route took (`D5 fens`), named like a row
    : d.verdict === "route" && d.route_cause ? /* copy:death_line */ `D${d.route_cause.fork} ${d.route_cause.taken}` : "";
  // Cut 17 §4: the line is laid on the defeat banner — the cause and depth in the display face, the verdict in the seal under it
  // (one word, engine data: `gap` · `dice` · `stall`); the text reads as before (`goblin archer · D6 · gap`)
  // death screen v2 (owner, 2026-10-01: "too wordy and incomprehensible"): the headline answers what killed him — the killer, the floor and (`hero at`: blind check read a bare `at 1 hp` as the foe's)
  // the hp he had before the blow (`gas · D7 · at 3 hp`); a stall keeps its loop (its moment is the rules'); the rule, the margin and
  // causal evidence moves to the why line and the details
  const hpAt = drove?.hp ?? momentHp(d);
  const causeText = d.verdict === "stall" ? /* copy:death_line */ `${d.cause.replace(/_/g, " ")} · D${d.depth}${causeRow ? ` · ${causeRow}` : ""}${margin}`
    : /* copy:death_line */ `${d.cause.replace(/_/g, " ")} · D${d.depth}${hpAt !== undefined ? ` · hero at ${hpAt} hp` : ""}`;
  const marginText_ = d.verdict === "stall" ? "" : [causeRow, seg].filter(Boolean).join(" · ");
  // QA 1a2a4a9 (P: `STALLED · R2 RETREAT ↔ EXPLORE · D6 · KEEPS $0` ran off both edges at 400 px): a stall's headline wraps between its
  // ` · ` segments (each whole: the loop `R2 retreat ↔ explore` never breaks) and steps its face down until the widest segment fits
  // gfx raters (the desktop banner broke `D8 · NO / RULE FOR IT`): every headline wraps between its segments, a segment kept whole
  const causeEl = h("span", { class: "cause" }, ...causeText.split(" · ").flatMap((seg, i, all) => [i ? " " : "", h("span", { class: "seg" }, seg, i < all.length - 1 ? " ·" : "")]));
  // QA 912e135 (qaW: the seal `GAP` and the banner answered no tap): the seal names what the patches answer — a tap brings them up and
  // lights the first; the banner names the moment — a tap brings up the trace
  // Cut 26 §6 (AO: `GAP` beside `unpatched 10/12` — "my fault or luck?"): the stamp and its counts agree — a gap, row or order most of
  // whose unpatched replays survive (the core's `lean`, else > half the replays) carries `dice-leaning` beside the stamp; the stamp stays
  const leanCounts = (d.verdict === "gap" || d.verdict === "row" || d.verdict === "order") && (d.replays ? Math.round((d.baseline ?? 0) * d.replays) * 2 > d.replays : (d.baseline ?? 0) > 0.5);
  const lean = !prePen && !drove && (d.lean === "dice" || leanCounts);
  const word = d.verdict;
  // docs/COPY.md §2: the stamp blames the right thing in a plain word — `dice` is luck, `row` the player's own rule
  // passes 2–3: `gap` alone read "no idea" 4/4; `unanswered` did not fit the seal and `unmet` read "a goal not met" 6/6 — the seal keeps
  // the plain death stamp; specific causal evidence is shown separately
  // Cut 29 (owner, 2026-09-28): the gap's seal reads YOU DIED (the report,
  // reel, trace and patches keep their own words for a gap)
  const stamp = prePen && word !== "stall" && !drove ? /* copy:verdict */ "you died" : word === "gap" ? /* copy:verdict */ "you died" : word === "dice" ? /* copy:verdict */ "luck" : word === "row" ? /* copy:verdict */ "rule" : (word as string) === "driven" ? /* copy:verdict */ "repelled" : word;
  // Cut 28 §2 (AU: `8/12 live unpatched` deaths "felt like the dungeon's decision"; AV: `10/12 live unpatched` under GAP read as blame): a
  // death most of whose replays live — or a `dice` — leads with the event that killed him and its odds (`goblin −6 at 6 hp · 1 in 6`), over
  // the stamp, which steps back (the patches still answer it)
  const luck = !prePen && !drove && d.verdict !== "stall" && d.verdict !== "route" && (lean || word === "dice") ? luckOf(d) : null;
  const luckLead = luck ? h("div", { class: "luck-lead num" }, h("span", { class: "luck-event" }, luck.event), h("span", { class: "luck-odds" }, /* copy:callout */ ` · 1 in ${luck.oneIn}`)) : null;
  const explainOutcome = (): void => {
    const term = VERDICT_TERM[word];
    const action = deathAction(d);
    openWindow(() => h("div", { class: "sheet-body outcome-explanation" },
      h("h3", null, causeText),
      term && stamp !== "you died" ? h("p", null, TIP[term]) : null,
      drove?.defence ? h("p", { class: "num" }, drove.defence) : null,
      action ? h("p", { class: "num" }, action) : null,
      d.lever ? h("div", null, h("h4", null, /* copy:label */ "Next run"), h("p", { class: "num" }, leverLine(d.lever))) : null));
  };
  const seal = h("button", { class: /* copy:none */ `verdict ${word}${luck ? " lean-seal" : ""}`, onclick: () => { if (prePen) { explainOutcome(); return; } patches.scrollIntoView({ block: "center", behavior: "smooth" }); const p = patches.querySelector<HTMLElement>(".patch.top") ?? patches.querySelector<HTMLElement>(".patch"); if (p) { p.classList.remove("flash"); void p.offsetWidth; p.classList.add("flash"); } } , "data-size": stamp.includes(" ") ? "w2" : stamp.length > 6 ? "l" : stamp.length > 3 ? "m" : undefined }, stamp);
  // QA 524827b (qaAB: tapped `gas · D6` expecting the clip; it only scrolled to the trace): the cause opens the moment — the killing
  // blow's replay when this session still holds the run — else brings up the trace
  const moment = d.trace.blow ? { text: d.cause.replace(/_/g, " "), t: d.trace.blow.t, depth: d.depth } : null;
  const onCause = (): void => { const log = lastRun(); if (moment && log && replayable(d.run_id, moment)) openReplay(log, moment); else if (prePen) explainOutcome(); else { if (!el.classList.contains("full")) more.click(); tracePanel.scrollIntoView({ block: "center", behavior: "smooth" }); } };
  if (!(prePen && stamp === "you died") && VERDICT_TERM[word]) kwHost(seal, VERDICT_TERM[word]!);
  const line = h("h1", { class: "death-line" }, h("button", { class: "cause-btn", onclick: onCause }, causeEl), h("span", { class: "sep" }, " · "), seal,
    // QA 308f045 (qaAD: `GAP` over `dice-leaning` — "two verdicts on one death"): beside the stamp the lean is the count it rests on, a fact
    // and not a second verdict (`10/12 live unpatched`; without the count, `most live unpatched`)
    lean ? h("small", { class: "lean num" }, d.replays ? /* copy:callout */ `${Math.round((d.baseline ?? 0) * d.replays)}/${d.replays} replays survive` : /* copy:callout */ "most replays survive") : "");
  // Cut 13 §4: the run's last two notes, engine data verbatim (`The green one: fire. Gambled: fire potion.`)
  // QA 92eb880: never a `… saved him.` over a death (M, N: read as the verdict), nor the cage's loot beat (`Took the axe +1 from the cage.`,
  // M: "unrelated to the ogre") — the core filters the first; the client keeps both off whatever the build
  const shownNotes = (d.notes ?? []).filter((n) => !/ saved him\.$/.test(n) && !/^The cage opens\b|^Took .* from the cage\.$/.test(n));
  // QA 912e135 (qaX: `ogre: telegraph` / `ogre: heavy` under the banner read as tappable because-lines): a fact the run learned says so
  const noteLine = (n: string): string => /^[A-Z][a-z]+(?: [a-z]+)?: [a-z_]+\.$/.test(n) ? /* copy:callout */ `learned ${noteText(n)}` : noteText(n);
  // Cut 24 §2: what was new this run (the core's `ExitLine.news`: a first, a record, a named kill), under the banner before the notes —
  // a death that also slew the Warlord says so; the `differ` line of a run with nothing new is left to the report
  // docs/COPY.md pass 7 (`LEARNED 3` read as "3 things" 4/4): the count names what it counts
  const newsTexts = mergeFinds((d.line?.news ?? []).filter((n) => n.k !== "differ")).map((n) => n.text.replace(/^learned (\d+)$/, /* copy:callout */ "$1 facts learned"));
  const news = newsTexts.length && !kept ? h("div", { class: "death-news num" }, newsTexts.slice(0, 3).join(" · ")) : null;
  // Cut 26 §6 (AP: "a drive-off at 25/36 hp"): the hp he was driven off at
  const drivenHp = drove && drove.hp !== undefined && drove.max_hp ? h("div", { class: "death-news num driven-hp" }, /* copy:callout */ `driven at ${drove.hp}/${drove.max_hp} hp`) : null;
  const notes = shownNotes.length ? h("div", { class: "death-notes num dim" }, ...shownNotes.slice(-2).map((n) => h("div", { class: "note" }, noteLine(n)))) : null;
  // Cut 13 §5: a `dice` death says what the forecast said for that depth — the reach the camp showed for the floor, verbatim
  // an old death (the chronicle) was sent under another forecast: today's would be a false number (QA on 56f2a1d: `forecast said D7 0%`)
  const said = (word === "dice" || lean) && !kept ? forecastSaid(app, d.depth) : undefined;
  const forecastLine = said !== undefined ? h("div", { class: "forecast-said num dim" }, forecastSaidText(app.lastForecast, d.depth, said)) : null;
  // Cut 6 §1: the exit's arithmetic, verbatim from the engine (`$144 carried · death keeps 0% → $0 · bones: 7 items on D5`)
  // Cut 11 §5: tappable — the gold sheet filtered to this run's movements
  // Cut 20 §4 (AC: "$80 gone after death, `repeat · $80` — only understood when removing refunded $40"): the loadout's re-pack for
  // the next heir, charged at this exit, is a line under it (`repeat −$80`) — the gold sheet it opens lists it too
  const repeat = kept ? 0 : repeatAfterExit(app.lineage.gold_ledger ?? []);
  const ledger = d.line?.text ? h("div", { class: "ledger-line num dim" }, h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, d.line) }, lineShown(d.line, renamer(app.lineage), { brief: true, lean: true })),   // docs/COPY.md pass 2: counts, not item lists (the gold sheet under the tap lists them)
    repeat > 0 ? h("button", { class: "ledger-btn repeat-line down", onclick: () => openGoldSheet(app, d.line) }, /* copy:callout */ `repeat −$${repeat}`) : "") : null;
  // The trace holds one row per hero action (~10 ticks apart at base speed); the last five, with the row accounting of
  // the last action under it (Cut 6 §3). Cut 9 §5: the table lives in ui/trace.ts, shared with every exit.
  // Cut 11 §2: the accounting is the chain; the rules that ran label its rows (the morgue's, else the editing copy)
  // The run's own rules label the accounting (`R1 drink unknown · no use`, as the editor spells it); a death from before
  // the wire carried them falls back to the morgue's short forms
  // QA 912e135 (qaW: the trace rows and `R1 unknown item` answered no tap): a row's name opens the editor on it (the rules that ran,
  // when they are the set now — an older death's rows name rows the set may not hold)
  const sameRules = !d.rules || JSON.stringify(d.rules.rows.map((r) => [r.conds, r.verb])) === JSON.stringify(app.rules.rows.map((r) => [r.conds, r.verb]));
  // QA 0c6e126 (qaZ: tapping `R1 drink heal` under the trace jumped to the camp editor — the death screen, APPLY and the morgue gone with
  // no way back): a row's name opens the row's sheet over the death screen (what it did in the trace); its `edit` goes to the camp
  const onRow = sameRules && !kept ? (i: number): void => { if (i < app.rules.rows.length) openRowSheet(app, d, i); } : undefined;
  const trace = traceTable(d.trace, { rows: d.rules?.rows ?? app.rules.rows, verbs: d.rules ? undefined : morgueVerbs(d.morgue), runId: d.run_id, chain: d.chain, depth: d.depth, onRow });
  // Fractions 0..1 from the core: baseline (survival of the unpatched rules) is on every row (Cut 4 §2).
  // QA 1a2a4a9 (O): a tap on a tablet lights it (the gem takes its number); the gem applies the lit one — the only apply on this screen
  let picked = false;   // the player lit a tablet (the landing never moves his pick)
  const light = (btn: HTMLButtonElement): void => { patches.querySelector(".patch.top")?.classList.remove("top"); top = topPatch(patches, btn); const g = makeGem(); gemBtn.replaceWith(g); gemBtn = g; rest(); };
  const select = (btn: HTMLButtonElement): void => { picked = true; light(btn); };
  // Cut 26 §6 (AP: a drive-off at 25/36 hp, `driven $0 · $297 lost`, and no verdict screen): a drive-off opens its verdict — the boss,
  // the floor, the defence; the seal `driven`; one tablet, the counter (`try: attack boss`), the gem writes it
  const patches = drove ? drivenBlock(app, drove, select, d.trace)
    : d.verdict === "route" && d.route_cause ? routeBlock(app, d, select)
    : patchRows(app, d.patches, d.baseline ?? 0, d.trace, { nothingBeatsBase: d.nothing_beats_base, stall: d.verdict === "stall", select, moment: d.depth, replays: d.replays });   // Cut 14 §4: the trace names the least-fired row on a full set
  // Cut 115 §4: a tactic (or variant) that answers the death leads the fixes, before a raw row (`try · gas step · burn`); taken, it is
  // worn and credited taught. Before the pen it is the lever itself.
  if (!prePen && !drove && d.pick?.id && app.engine.takeFix) patches.prepend(pickTablet(app, d.pick));
  // The morgue is the shareable text of the run: show it in a sheet (the clipboard is a bonus, not the point).
  const openMorgue = (): void => {
    // QA a946e04 (S: `slain by goblin_archer`): ids read as words (`goblin archer`), in the sheet and the copy alike
    const text = d.morgue.replace(/([a-z])_(?=[a-z])/g, "$1 ");
    void copyText(text);
    openSheet(() => h("div", { class: "morgue" }, h("div", { class: "label row-label" }, /* copy:label */ "morgue"), h("pre", { class: "morgue-text" }, text)));
  };
  // Cut 10 §3: `◯ jackal Ashar fell` (a companion leaves an egg); Cut 12 §6: a summoned ally reads `ally hound fell`, no egg
  // QA 92eb880 (N: "`ally hound fell` is drawn as a button … tapping it does nothing"): a line of text, not a chip
  const eggs = lost.length ? h("div", { class: "eggs-line num dim" }, ...lost.flatMap((k, i) => [i ? " · " : "", h("span", { class: "egg" }, k.includes(" · ") ? "◯ " : "", lostLabel(k))])) : null;
  // Cut 2 §2: what this death left on the floor — the pile whose heir the matching grave names; silent when absent, and silent
  // when the exit line already says it (`… · bones: 8 items on D4`; two QA players on 50bb162 read the pair as two piles)
  const L = app.lineage;
  const grave = [...(L.graveyard ?? [])].reverse().find((g) => g.depth === d.depth && g.cause === d.cause);
  const pile = (L.bones ?? []).find((b) => (grave ? b.heir === grave.heir : false) && b.depth === d.depth);
  const bones = pile && !/\bbones:/.test(d.line?.text ?? "") ? h("div", { class: "bones-line dim num" }, /* copy:callout */ `bones left · ${items(pile.items)}`) : null;
  // Cut 17 §4: the frame — the banner over the dimmed floor (the line, the seal), the trace on a parchment panel, the patches as
  // tablets with a gauge; the console: edit · morgue · camp, and the gem applies the top patch (its survival on the stone)
  let top = topPatch(patches);
  // QA 92eb880 (M: "the header already shows the next hero (`♟2 · greedy`) above ♟1's death"): the bar names the hero who died
  const bar = renderBar(app, kept ? {} : deadHero(d.morgue));
  // QA 1a2a4a9 (P: "the hero came home but the portrait is greyed like a corpse"): a stall is no death — the face stays lit
  const stalled = d.verdict === "stall" || !!drove;
  const face = portrait(app, stalled ? { hp: 1, label: `D${d.depth}` } : { hp: 0, dead: true, label: `D${d.depth}` });
  // QA 308f045 (qaAD: `retreat · drops R3 · survives 12/12` applied from the gem, then `vs sent · bank −30 · death +29`): the gem offers no
  // one-tap apply until the lit patch's whole run is measured (`deathDeltas` landed) — it reads `…` — and a lit tablet that harms whole runs
  // says so on the stone (`harms`, never `apply`: the player's own pick still applies, warned)
  // Cut 27 §4 (AS: the stall screen's gem applied `depth ≥ 5 → return · drops R10` over `cut R8`): a stall's gem goes through the same
  // guard — its patches are measured on whole runs (`deathDeltas`) before the gem offers one, and never a harming one
  const measureAll = !prePen && !drove && !(d.verdict === "route" && d.route_cause) && (d.patches.some((p) => p.camp_pending) || (d.verdict === "stall" && d.patches.some((p) => p.insert_at >= 0)));
  let measuring = measureAll && !!app.engine.deathDeltas;
  const lever = prePen ? (d as Death & { lever?: Lever }).lever : undefined;   // Cut 30 (core): before the pen opens, the one cheapest lever (spend · package · wait) leads
  const canSend = !kept && app.rules.rows.length > 0 && !app.overBudget;
  // blind ad71e72 (B: `WAIT Guarded L5` "unreadable" — a lever names what to do now): a stance's next level reads as the act that
  // gets it — `level · Guarded L5` when the marks pay for it now (a tap buys it), else `train · Guarded L5` (a send trains it); a
  // level already worn, or the drill and the scars to come, read `send · …`
  const waitPkg = lever?.kind === "wait" ? (() => {
    const m = /^(.+) L(\d+)$/.exec(lever.text); if (!m) return undefined;
    const p = app.lineage.packages?.all.find((x) => x.name === m[1] || x.id === m[1].toLowerCase());
    return p ? { p, to: Number(m[2]) } : undefined;
  })() : undefined;
  const levelNow = !!waitPkg && waitPkg.p.level < waitPkg.to && !!waitPkg.p.level_price && app.lineage.marks >= waitPkg.p.level_price && !!app.engine.spendLevel;
  /* copy:label */
  const waitWord = !lever || lever.kind !== "wait" ? undefined : levelNow ? "level" : waitPkg && waitPkg.p.level < waitPkg.to ? "train" : "send";
  const waitText = lever?.kind === "wait" && waitPkg && waitPkg.p.level >= waitPkg.to ? /* copy:label */ "again" : lever?.text ?? "";
  /** The lever's act: `spend` opens the blacksmith on its step, `package` wears it (free, instant) and goes to camp, `wait` levels the
   *  stance with marks when it can, else sends again. */
  const leverAct = (): void => {
    if (!lever) { app.go({ kind: "camp" }); return; }
    if (levelNow && waitPkg) { void app.mutate(() => app.engine.spendLevel!(waitPkg.p.id), /* copy:callout */ `L${waitPkg.p.level + 1}`, true).then(() => app.go({ kind: "camp" })); return; }
    if (lever.kind === "spend") { openPreparationForge(app); return; }
    if (lever.kind === "tactic" && lever.id && app.engine.takeFix) { void app.mutate(() => app.engine.takeFix!(lever.id!, lever.variant ?? 0), /* copy:callout */ lever.text, true).then(() => app.go({ kind: "camp" })); return; }
    if (lever.kind === "package") {
      const p = app.lineage.packages?.all.find((x) => x.name === lever.text || x.id === lever.text.toLowerCase());
      if (p && p.owned && app.engine.equipPackage) void app.mutate(() => app.engine.equipPackage!(p.id, 0), /* copy:callout */ p.name, true).then(() => app.go({ kind: "camp" }));
      else app.go({ kind: "camp" });
      return;
    }
    if (canSend) app.go({ kind: "watch" }); else app.go({ kind: "camp" });
  };
  /* copy:label */
  const LEVER_WORD: Record<string, string> = { spend: "buy", package: "wear", wait: "wait", tactic: "try" };
  /* copy:button */
  const LEVER_GEM: Record<string, string> = { spend: "forge", package: "wear", wait: "send", tactic: "try" };
  /** The lever in words, as its tablet reads (`train · Guarded L5`, `buy · sword +2`). */
  function leverLine(l: Lever): string { return l === lever && l.kind === "wait" ? `${waitWord} · ${waitText}` : `${LEVER_WORD[l.kind] ?? l.kind} · ${l.text}`; }
  const leverBtn = lever ? h("button", { class: "death-lever tablet", "data-kind": lever.kind, onclick: leverAct },
    h("span", { class: "lever-kind" }, waitWord ?? LEVER_WORD[lever.kind] ?? lever.kind), h("b", { class: "lever-name" },
      lever.kind === "spend" ? itemIcon({ kind: lever.text, label: lever.text }) : lever.kind === "tactic" && lever.id ? packageIcon(lever.id) : lever.kind === "package" ? (() => {
        const p = app.lineage.packages?.all.find((x) => x.name === lever.text || x.id === lever.text.toLowerCase());
        return p ? packageIcon(p.id) : "";
      })() : waitPkg && waitText !== "again" ? packageIcon(waitPkg.p.id) : "", lever.kind === "wait" ? waitText : lever.text), h("span", { class: "lever-go", "aria-hidden": "true" }, "›")) : null;
  if (leverBtn) kwHost(leverBtn, "lever");
  function leverGem(): HTMLButtonElement { return gem({ label: levelNow ? /* copy:button */ "level" : lever ? LEVER_GEM[lever.kind] ?? /* copy:button */ "town" : /* copy:button */ "town", cls: "lever-gem", pulse: true, onclick: leverAct }); }
  const isPatchTop = (): boolean => !!top && !top.btn.classList.contains("unlock") && !top.btn.classList.contains("held") && top.btn.classList.contains("patch") && !top.btn.classList.contains("driven-line");
  const makeGem = (): HTMLButtonElement => prePen ? leverGem() : top && measuring && isPatchTop()
    ? gem({ label: h("span", { class: "gem-in" }, h("span", { class: "gem-n" }, "…"), h("small", { class: "gem-w" }, /* copy:label */ "measuring")), cls: "patch-gem pending", onclick: () => undefined })
    : top
    // QA 23ed91f (K: "the gem reads `100%` with no label … I read it as the run's result"): the number, and the word the tap does under it
    ? gem({ label: h("span", { class: "gem-in" }, h("span", { class: "gem-n" }, top.label), top.label !== "buy" && top.label !== "edit" && top.label !== "write" && top.label !== "move" ? h("small", { class: "gem-w" }, top.btn.classList.contains("harms") ? /* copy:label */ "risky" : /* copy:label */ "apply") : ""), cls: `patch-gem${top.btn.classList.contains("harms") ? " harms" : ""}`, pulse: !top.btn.classList.contains("harms"), onclick: () => { if (top) void applyOf.get(top.btn)?.(); } })
    : gem({ label: /* copy:button */ "town", pulse: true, onclick: () => app.go({ kind: "camp" }) });
  let gemBtn = makeGem();
  const cons = renderConsole({ portrait: face.el, gem: gemBtn, compact: true, tiles: [
    top && !prePen ? tile({ id: "edit", label: /* copy:button */ "edit", icon: "edit", onclick: () => { app.editing = true; app.go({ kind: "camp" }); } }) : null,
    // QA 308f045 (qaAD: a verdict opened from the return report had no way back — `edit · morgue · camp`, browser back to the camp): a
    // verdict opened from a report leads back to it
    from ? tile({ id: "report", label: /* copy:button */ "report", icon: "trace", onclick: () => app.go({ kind: "report", report: from.report, absence: from.absence }) }) : null,
    tile({ id: "camp", label: /* copy:button */ "town", icon: "camp", onclick: () => app.go({ kind: "camp" }) }),
  ] });
  const enemyKind = drove?.boss ?? d.cause;
  const killerSrc = d.verdict === "stall" ? null : foeSrc(enemyKind);
  const enemyKnowledge = (kept && !d.hero) || (d.hero?.bloodline_id !== undefined && d.hero.bloodline_id !== (app.lineage.selected_bloodline ?? 1)) ? undefined : app.lineage;
  const knownKiller = drove || killerSrc || (d.verdict !== "stall" && app.lineage.ledger?.some(r => r.kind === enemyKind));
  const killerPortrait = knownKiller ? enemyHost(unitPortrait(enemyKind, 112), enemyKind, enemyKnowledge, false, () => d) : null;
  if (killerPortrait) {
    killerPortrait.classList.add("killer");
    killerPortrait.removeAttribute("aria-hidden");
    killerPortrait.setAttribute("role", "img");
    killerPortrait.setAttribute("aria-label", d.cause.replace(/_/g, " "));
    const causeBtn = line.querySelector<HTMLElement>(".cause-btn")!;
    causeBtn.prepend(killerPortrait);
    causeBtn.classList.add("has-unit-icon");
    enemyHost(causeBtn, enemyKind, enemyKnowledge, false, () => d);
  }
  const tracePanel = h("div", { class: "parchment trace-panel", hidden: !!drove && !d.trace.turns.length }, ...trace);   // a drive-off's line may carry no trace
  // death screen v2 — the three questions, in order: what killed him (the banner), why (one line, the player's words), what now (the
  // lit fix and at most two others, each its short name and its effect; `send again` beside them for a luck death or none to offer);
  // everything else — the margin, the notes, the ledger, the trace, the tablets' numbers, the fight, the morgue — behind `details`
  const rowsRan = d.rules?.rows ?? app.rules.rows;
  const whyEl = h("div", { class: "death-why" });
  const leadPatch = (): Patch | undefined => { const b = top?.btn; return b ? patchOf.get(b) : undefined; };
  // Before the pen, name the historical package and action in readable words.
  const setWhy = (): void => { whyEl.textContent = prePen ? deathAction(d) : whyOf(app, d, leadPatch(), rowsRan, luck, drove, causeRow); whyEl.hidden = !whyEl.textContent; };
  const sendAgain = canSend && !prePen ? h("button", { class: "death-send chip game-control", onclick: () => app.go({ kind: "watch" }) }, /* copy:button */ "send again") : null;
  const preparation = preparationActions(app, { obstacle: d.cause.replace(/_/g, " "), showObstacle: false });
  const now = h("div", { class: "death-now" }, leverBtn, prePen ? null : patches, sendAgain, preparation.el);
  // the rest view: one recommended fix; `send again` when the death leaned on luck or nothing helps (both when there is room)
  function rest(): void { if (prePen) return; const n = restLayout(patches, 1, measuring); if (sendAgain) sendAgain.hidden = !(luck || n === 0 || !!drove && n < 2); }
  const marginEl = marginText_ ? h("div", { class: "death-margin num dim" }, marginText_) : null;
  const meters = d.fight ? h("div", { class: "parchment fight-meters" }, meterPanel(d.fight, rowsRan, { title: /* copy:label */ "last fight" })) : null;
  const morgueBtn = d.morgue ? h("button", { class: "chip mini death-morgue game-control", onclick: openMorgue }, /* copy:button */ "morgue") : null;
  const details = h("div", { class: "death-details", hidden: true }, marginEl, news, drivenHp, notes, forecastLine, ledger, eggs, bones);
  const tail = h("div", { class: "death-details death-tail", hidden: true }, tracePanel, meters, morgueBtn);
  // one `details` tap, the report's own control (Cut 28 §2): the old screen, whole — the margin, the ledger, every tablet with its numbers
  const more: HTMLButtonElement = h("button", { class: "details-fold death-more num game-control", "aria-expanded": "false", onclick: () => {
    const open = !el.classList.contains("full"); el.classList.toggle("full", open); details.hidden = !open; tail.hidden = !open;
    more.setAttribute("aria-expanded", String(open)); more.classList.toggle("on", open);
    if (open) details.scrollIntoView({ block: "nearest", behavior: "smooth" });
  } }, h("span", { class: "fold-mark", "aria-hidden": "true" }, "▸ "), /* copy:button */ "details", foldHint(patches));
  // blind 77030eb (B): a training plaque opens the tactics panel (the drill's On / Off, the level)
  const training = app.lineage.packages && app.engine.equipPackage ? (a: HTMLElement): void => openPackages(app, a) : undefined;
  const well = h("div", { class: "well death-well" },
    d.hero?.name ? unitLabel(d.hero.class, h("span", null, d.hero.name, ` · ${d.hero.class}`, /* copy:label */ ` · Bloodline ${d.hero.bloodline_id}`), { hero: true, art: unitPortrait(d.hero.class, 64, true), className: "death-hero num dim" }) : null,
    h("div", { class: "defeat" }, h("div", { class: `banner-cloth${luck ? " luck" : ""}${killerPortrait ? " has-killer" : ""}` },
      // The framed killer portrait stays centered above the cause text.
      luckLead, line)),
    // run-clear: the death screen is a death's card — its header carries the floor, a new best, the finds left in the bones
    kept || from ? null : clearStrip(d.line),
    ...(prePen ? [whyEl, trainingBlock(d.line?.packages, training), now] : [whyEl, trainingBlock(d.line?.packages, training), details, now, more, tail]));
  const wide = wideCols(app, null);   // desktop: the rules left, the shaft right (wide.css); the fight is under details
  const el = h("main", { class: `death frame${stalled ? " stalled" : ""}${drove ? " driven" : ""}${prePen ? " prepen" : ""}` }, bar.el, well, cons.el, ...wide.els);
  setWhy(); rest();
  // docs/UI.md §7: after 12 s the default — the gem when it sends or goes to camp, else `send again` when shown, else the camp tile (a
  // fix, `forge` and `wear` are decisions); a death opened from the report or the chronicle goes back after a panel's 20 s
  const campTile = cons.el.querySelector<HTMLElement>('[data-tile="camp"]')!, opened = kept || !!from;
  // The first sends are manual; only the hired Scout authorizes another run without a tap.
  const manualSend = app.lineage.tree?.auto_send === false;
  const autoAt = opened ? cons.el.querySelector<HTMLElement>('[data-tile="report"]') ?? campTile : manualSend ? campTile : prePen && (!lever || lever.kind === "wait") ? gemBtn : sendAgain && !sendAgain.hidden ? sendAgain : campTile;
  autoDismiss(autoAt, { ms: opened ? AUTO.panel : AUTO.death, yieldToSheets: true, onExpire: () => (autoAt.isConnected && autoAt.getClientRects().length ? autoAt : campTile).click() });
  // Cut 18 §4: a stall's cause is the rows' loop (`R2 retreat ↔ explore`) — it reads whole on one line: the face steps down until it fits
  if (d.verdict === "stall") { line.classList.add("loop"); fitLine(line.querySelector<HTMLElement>(".cause")); }
  // QA 23ed91f: the patches' reach is the camp's own measure, landing after the paint (`deathDeltas`: seconds in wasm) — the
  // screen never waits on it; a reach still pending reads `reach …` until then
  let gone = false;
  if (measureAll && app.engine.deathDeltas) {
    const shown = d.patches;
    // QA 778fa1b (qaU: the lit tablet and the gem moved 1 → 2 on their own ~5 s after arrival): the landing fills each tablet's reach
    // and dims a loss; the order and the lit tablet (the gem's) stay as the screen first put them
    // QA 912e135 (qaX: the gem's `100% APPLY` stayed on `foe: telegraph → retreat · drops R1` once its reach landed at `D8 −62`): the
    // screen's own lit tablet (not the player's pick) that the landing shows losing 10 points or more yields the light to the first
    // tablet that does not lose — a patch that costs that much never leads
    setTimeout(() => { if (!gone) void app.engine.deathDeltas!(d.run_id).then((f) => {
      if (gone) return;
      if (!f?.length) { measuring = false; const g1 = makeGem(); gemBtn.replaceWith(g1); gemBtn = g1; return; }
      fillReach(patches, shown, f);
      // Cut 27 §4 (core): the landing names the gem (`Patch.gem`: the best whole-run patch that does not harm, always the core's first) —
      // the tablets take the core's order and the gem lights it; none flagged, no gem (`edit`). An older core falls to the client's rules below
      if (f.some((p) => p.gem !== undefined)) {
        const tabs = [...patches.querySelectorAll<HTMLButtonElement>("button.patch:not(.unlock)")];
        if (tabs.length && tabs.every((b) => b.classList.contains("harms"))) {
          const head = patches.querySelector<HTMLElement>(":scope > .patches-moment");
          if (head && !head.querySelector(".all-harm")) head.appendChild(h("span", { class: "all-harm down" }, /* copy:callout */ " · all harm"));
        }
        if (!picked) {
          leadFirst(patches, shown, f, null);
          const gi = shown.findIndex((p) => p.gem === true), host = patches.querySelector<HTMLElement>(":scope > .patches-fold") ?? patches;
          const gb = gi >= 0 ? host.querySelectorAll<HTMLButtonElement>(":scope > button.patch")[gi] : undefined;
          if (gb) light(gb); else { top?.btn.classList.remove("top"); top = null; }
          el.dataset.gemFirst = top ? (top.btn === patches.querySelector("button.patch") ? "1" : "0") : "";
        }
        measuring = false; const g3 = makeGem(); gemBtn.replaceWith(g3); gemBtn = g3;
        if (!picked) setWhy();
        rest(); return;
      }
      const btns = [...patches.querySelectorAll<HTMLButtonElement>("button.patch")];
      // (an exit's cost is its word, `return early`, and the core keeps a costly exit off the lead: only a row that stays in the fight moves)
      const cost = (b: HTMLButtonElement): number => { const p = shown[btns.indexOf(b)]; return p && p.insert_at >= 0 && !p.camp_pending && !(p.exits ?? /^(return|bank)$/.test(p.row.verb.v)) ? Math.round(p.forecast_delta * 100) : 0; };
      const lit = top?.btn;
      if (!picked && lit && cost(lit) <= -10) { const alt = btns.find((b) => b !== lit && !b.classList.contains("below") && !b.classList.contains("neg") && cost(b) > -10); if (alt) light(alt); }
      // QA 524827b (qaAA: `drink unknown · 12/12` lit, and applied the camp's killers read fire · poison; `read unknown · 11/12`, then
      // `death +6`): a tablet that harms whole runs (`PatchWhole.harms`) is never the lead nor the gem's — unless the player lit it, the
      // harming tablets go after the rest and the light takes the first that does not harm; when every one harms the gem reads `edit`
      // Cut 26 §6 (control rater AQ: `survives 12/12 … death +74` led — every shown patch harmed whole runs, nothing to sink it under):
      // when every tablet harms, the block says so over them
      const tabs = [...patches.querySelectorAll<HTMLButtonElement>("button.patch:not(.unlock)")];
      if (tabs.length && tabs.every((b) => b.classList.contains("harms"))) {
        const head = patches.querySelector<HTMLElement>(":scope > .patches-moment");
        if (head && !head.querySelector(".all-harm")) head.appendChild(h("span", { class: "all-harm down" }, /* copy:callout */ " · all harm"));
      }
      if (!picked) {
        sinkHarms(patches, shown);
        const cur = top?.btn;
        if (cur && cur.classList.contains("harms")) {
          const alt = [...patches.querySelectorAll<HTMLButtonElement>("button.patch")].find((b) => !b.classList.contains("below") && !b.classList.contains("neg") && !b.classList.contains("harms"));
          if (alt) light(alt);
          else { cur.classList.remove("top"); top = null; const g = makeGem(); gemBtn.replaceWith(g); gemBtn = g; }
        }
      }
      // Cut 27 §4 (AS: the stall gem lit `cut R1` 1/12 above a 12/12): unless the player lit one, the gem takes the best whole-run patch —
      // the landed order (the core's rank on whole runs), the first that neither loses nor harms — and it is the first tablet shown
      if (!picked) {
        leadFirst(patches, shown, f, null);
        // (a reach loss the lead rule above let stand keeps its light — only a harm or a below-bar tablet yields it)
        const lit0 = top?.btn;
        const best = [...patches.querySelectorAll<HTMLButtonElement>("button.patch")].find((b) => !b.classList.contains("below") && !b.classList.contains("harms") && (!b.classList.contains("neg") || b === lit0));
        if (best && best !== top?.btn) light(best);
        leadFirst(patches, shown, f, top?.btn ?? null);
        el.dataset.gemFirst = top?.btn === patches.querySelector("button.patch") ? "1" : "0";
      }
      measuring = false; const g2 = makeGem(); gemBtn.replaceWith(g2); gemBtn = g2;
      if (!picked) setWhy();
      rest();
    }).catch((e) => { console.warn("deathDeltas", e); measuring = false; const g2 = makeGem(); gemBtn.replaceWith(g2); gemBtn = g2; }); }, 0);
  }
  return { el, dispose: () => { gone = true; preparation.dispose(); bar.dispose(); wide.dispose(); } };
}

/** QA 0c6e126 (qaZ): a rule row named under a death's trace — the row, how often it acted in the trace's turns and why it did not
 *  on the last one (engine data: `TraceTurn.rows`), and `edit` (the camp on the row); the sheet's `×` leaves the death screen as it was. */
function openRowSheet(app: App, d: Death, i: number): void {
  const row = (d.rules?.rows ?? app.rules.rows)[i]; if (!row) return;
  const turns = d.trace.turns, acted = turns.filter((t) => t.row === i).length;
  const last = turns[turns.length - 1], why = last?.row === i ? /* copy:callout */ "fired" : last?.rows?.find((w) => w.row === i)?.why;
  openSheet((close) => h("div", { class: "sheet-body row-sheet" },
    h("div", { class: "label row-label" }, h("small", { class: "dim" }, rowLabel(row)), closeX(close)),
    h("div", { class: "num dim row-acted" }, /* copy:callout */ `acted ${acted}/${turns.length}`, why ? ` · ${why}` : ""),
    h("button", { class: "chip row-edit", onclick: () => { close(); app.editing = true; app.go({ kind: "camp", highlight: i }); } }, /* copy:button */ "edit")));
}

/** Cut 20 §4: the gold the loadout's repeat charged after the newest exit (`repeat heal` ledger lines, the core's re-pack), 0 when none. */
export function repeatAfterExit(ledger: { delta: number; why: string }[]): number {
  let i = ledger.length - 1; while (i >= 0 && !/^(returned|banked|died|lost|stalled|driven)\b/.test(ledger[i].why)) i--;
  if (i < 0) return 0;
  return ledger.slice(i + 1).filter((g) => g.delta < 0 && /^repeat\b/.test(g.why)).reduce((a, g) => a - g.delta, 0);
}

/** Cut 19 §4: the row a `row` verdict names — `R2 drink unknown` (its verb, two words at most), `R2` alone when the row is not known. */
export function rowName(i: number, row?: Row, rows?: Row[]): string {
  if (rows?.[i]) return ruleName(rows, i);
  return row ? verbLabel(row.verb) : refName(i);
}

/** Cut 18 §4: shrink a one-line headline's face until it fits its box (from its CSS size down to 12 px), once it is laid out. */
function fitLine(el: HTMLElement | null): void {
  if (!el) return;
  const fit = (tries: number): void => {
    if (!el.isConnected) { if (tries > 0) requestAnimationFrame(() => fit(tries - 1)); return; }
    let px = parseFloat(getComputedStyle(el).fontSize) || 19;
    const over = (): boolean => el.scrollWidth > el.clientWidth + 0.5 || [...el.querySelectorAll<HTMLElement>(".seg")].some((s) => s.offsetWidth > el.clientWidth + 0.5);
    while (over() && px > 12) { px -= 1; el.style.fontSize = `${px}px`; }
  };
  requestAnimationFrame(() => fit(10));
}

/** QA 92eb880: the hero a death's morgue names — `heir 3` and his trait (`trait greedy`; the fake's `heir 3 · fighter · greedy`). */
export function deadHero(morgue: string): { heir?: number; trait?: string } {
  const heir = /\bheir (\d+)\b/.exec(morgue)?.[1];
  const trait = /\btrait ([a-z_]+)/.exec(morgue)?.[1] ?? /\bheir \d+ · [a-z_]+ · ([a-z_]+)/.exec(morgue)?.[1];
  return { heir: heir ? Number(heir) : undefined, trait };
}

/** Cut 17 §4: the gem's patch — the first that applies (not a held row's `at R2`, not a below-bar alternative): its button and its
 *  number (`92%`, the survival the patch reads; an unlock pseudo-patch's `buy`). The tablet it stands for is lit. */
function topPatch(patches: HTMLElement, pick?: HTMLButtonElement): { btn: HTMLButtonElement; label: string } | null {
  const btn = pick ?? patches.querySelector<HTMLButtonElement>("button.patch:not(.below):not(.neg)");
  if (!btn) return null;
  btn.classList.add("top");
  const unlock = btn.classList.contains("unlock");
  const surv = /(\d+(?:%|\/\d+))/.exec(btn.querySelector(".surv")?.textContent ?? "")?.[1];
  // a lit held row (`at R2`) opens the camp on it: the gem reads `edit`
  return { btn, label: unlock ? /* copy:button */ "buy" : btn.classList.contains("held") ? /* copy:button */ "edit" : btn.dataset.gem ?? surv ?? "" };
}


/** Cut 26 §6: a drive-off's verdict — the exit line's `driven` as a death record the screen reads (the core keeps none for a return-tier
 *  exit): the boss kind as the cause, its defence as the margin, the seal `driven`, the exit's trace. No patches: the counter is the tablet. */
export function drivenDeath(line: ExitLine | DrivenOff, runId: number, trace?: Death["trace"]): Death {
  // (a report's `drives` entry carries no exit line: its line is the drive-off's own numbers)
  if (!("driven" in line) || !line.driven) { const d = line as DrivenOff; line = { carried: d.lost ?? 0, keep_pct: 60, kept: 0, spent: 0, spent_on: [], text: d.lost ? /* copy:callout */ `driven $0 · $${d.lost} lost` : /* copy:callout */ "driven $0", run_id: d.run_id, driven: d }; }
  const dv = line.driven!;
  return { run_id: runId, depth: dv.depth, cause: dv.title || dv.boss, margin: dv.defence, verdict: "driven" as Death["verdict"], baseline: 0,
    trace: line.trace ?? trace ?? { turns: [] }, patches: [], morgue: "", line };
}
export const isDriven = (d: Death): boolean => (d.verdict as string) === "driven" && !!d.line?.driven;
/** The drive-off's one tablet: the boss's counter as a row (`try: attack boss`, or `at R2` when the set holds it); the gem writes it at
 *  the top (a full set asks which row it replaces, as a patch does). */
function drivenBlock(app: App, dv: DrivenOff, select: (b: HTMLButtonElement) => void, trace?: Death["trace"]): HTMLElement {
  const same = (a: Row, b: Row): boolean => a.verb.v === b.verb.v && (a.verb.a ?? "") === (b.verb.a ?? "") && JSON.stringify(a.conds) === JSON.stringify(b.conds);
  const dvo = dv as DrivenOff & { held?: number; over?: number; verdict?: string };
  const have = dvo.verdict === "order" && dvo.held !== undefined && dvo.held < app.rules.rows.length ? dvo.held : app.rules.rows.findIndex((r) => same(r, dv.row));
  // Cut 27 §5 (AT: the drive-off said `counter unwritten` and offered the counter row his R7 already was): a counter the set holds lost on
  // priority — `order`: the row above it that took the moments (`R7 under R2`; the core's `order_over`, else the trace's busiest row
  // above it), and the tablet moves it above that row
  const over = have >= 0 ? orderOver(dv, have, trace) : -1;
  const write = (): void => {
    if (have >= 0 && over >= 0) {
      const mv: Patch = { row: app.rules.rows[have], insert_at: over, moves_from: have, survive: 0, forecast_delta: 0 };
      // a package row moved up joins the pen: on a full set it asks which row makes room first
      if (app.rowsFull && app.patchTakesRow(mv)) { openDropSheet(app, mv); return; }
      const i = app.applyPatch(mv); app.go({ kind: "camp", highlight: i }); return;
    }
    if (have >= 0) { app.go({ kind: "camp", highlight: have }); return; }
    if (app.rowsFull) { openDropSheet(app, { row: { ...dv.row, origin: "patch" }, insert_at: 0, survive: 0, forecast_delta: 0 } as Patch); return; }
    app.go({ kind: "camp", highlight: app.insertRow(dv.row, 0, "patch") });
  };
  const moveIt = have >= 0 && over >= 0;
  const btn: HTMLButtonElement = h("button", { class: `patch tablet driven-line${have >= 0 && !moveIt ? " held" : ""}${moveIt ? " move" : ""}`, onclick: () => select(btn), "data-gem": moveIt ? /* copy:button */ "move" : have >= 0 ? /* copy:button */ "edit" : /* copy:button */ "write",
    "data-short": moveIt ? /* copy:button */ `move ${ruleName(app.rules.rows, have)} up` : have >= 0 ? ruleName(app.rules.rows, have) : `+ ${counterName(app.lineage, dv.counter, dv.row)}`, "data-eff": /* copy:callout */ "counter" },
    h("b", { class: "rank num", "aria-hidden": "true" }, "1."),
    h("span", { class: "patch-main" }, h("span", { class: "chips-inline" }, moveIt ? h("small", { class: "target move-tag" }, /* copy:death_line */ `move above ${ruleName(app.rules.rows, over)} `) : "", rowLabel(dv.row))),
    h("span", { class: "patch-nums" }, h("span", { class: "num surv" }, moveIt ? /* copy:callout */ `under ${ruleName(app.rules.rows, over)}` : have >= 0 ? /* copy:callout */ "already written" : /* copy:callout */ `try: ${counterName(app.lineage, dv.counter, dv.row)}`)));
  applyOf.set(btn, write);
  return h("div", { class: "patches driven" }, h("div", { class: "patches-moment num dim" }, have >= 0 ? /* copy:callout */ `D${dv.depth} · order` : /* copy:callout */ `D${dv.depth} · counter unwritten`), btn);
}
/** Cut 27 §5: the row above the held counter `have` that won the boss's moments — the core's (`DrivenOff.order_over`), else the row above
 *  it that acted most in the drive-off's trace; -1 when neither says. */
export function orderOver(dv: DrivenOff, have: number, trace?: Death["trace"]): number {
  const core = (dv as DrivenOff & { order_over?: number; over?: number }).over ?? (dv as DrivenOff & { order_over?: number }).order_over;
  if (typeof core === "number" && core >= 0 && core < have) return core;
  const n = new Map<number, number>();
  for (const t of trace?.turns ?? []) if (t.row >= 0 && t.row < have) n.set(t.row, (n.get(t.row) ?? 0) + 1);
  let best = -1, most = 0; for (const [r, k] of n) if (k > most || (k === most && r < best)) { best = r; most = k; }
  return best;
}

/** Cut 26 (core, risks): a `route` death's lead is a route edit — the other stair at that fork (`take burrows`), its replays' count beside
 *  the unpatched ones; the gem writes it (`setRules` with `route_cause.route`). The core's row patches follow under it. */
function routeBlock(app: App, d: Death, select: (b: HTMLButtonElement) => void): HTMLElement {
  const rc = d.route_cause!, n = d.replays ?? 12, cnt = (x: number): string => `${Math.round(x * n)}/${n}`;
  const same = JSON.stringify([...(app.rules.route ?? [])].sort()) === JSON.stringify([...rc.route].sort());
  const write = (): void => { if (!same) { if (rc.route.length) app.rules.route = [...rc.route]; else delete app.rules.route; app.rulesChanged(); } app.go({ kind: "camp" }); };
  const btn: HTMLButtonElement = h("button", { class: "patch tablet route-fix", onclick: () => select(btn), "data-gem": cnt(rc.survive), "data-short": /* copy:button */ `take ${rc.other}`, "data-eff": /* copy:callout */ `survives ${cnt(rc.survive)}` },
    h("b", { class: "rank num", "aria-hidden": "true" }, "1."),
    h("span", { class: "patch-main" }, h("span", { class: "chips-inline" }, h("small", { class: "target move-tag" }, /* copy:death_line */ `fork D${rc.fork} `), /* copy:callout */ `take ${rc.other}`)),
    h("span", { class: "patch-nums" }, h("span", { class: "gauge", "aria-hidden": "true" }, h("i", { style: `width:${Math.round(Math.max(0, Math.min(1, rc.survive)) * 100)}%` })),
      h("span", { class: "num surv" }, /* copy:callout */ `survives ${cnt(rc.survive)} · was ${cnt(d.baseline ?? 0)}`)));
  applyOf.set(btn, write);
  const rest = patchRows(app, d.patches, d.baseline ?? 0, d.trace, { select, moment: d.depth, replays: d.replays });
  rest.insertBefore(btn, rest.querySelector(":scope > button.patch"));
  rest.querySelectorAll<HTMLElement>(":scope > button.patch > .rank").forEach((r, i) => { r.textContent = `${i + 1}.`; });
  return rest;
}

/** Cut 28 §2 — a luck-leaning death's lead: the event that killed him (the core's `Death.luck.text`, else the killing blow: `goblin −6 at 6
 *  hp`, the hp he had before it) and how often the replays die of it — the core's `one_in`, else the unpatched replays that died
 *  (`1 in round(n / died)`; none died: `1 in 12+`). */
export function luckOf(d: Death): { event: string; oneIn: string } {
  const core = (d as Death & { luck?: { text?: string; one_in?: number; odds?: number } }).luck;
  const blow = d.trace.blows?.length ? d.trace.blows[d.trace.blows.length - 1] : d.trace.blow;
  // docs/COPY.md pass 2 (`a max hit at 4 hp` read "hit for 4", 2/2): the roll is named as one — `max-damage hit at 4 hp`
  const event = core?.text?.replace(/^a max hit at\b/, /* copy:callout */ "max-damage hit at") ?? (blow ? /* copy:callout */ `${blow.by.replace(/_/g, " ")} −${blow.dmg} at ${blow.hp + blow.dmg} hp` : d.cause.replace(/_/g, " "));
  const n = d.replays ?? 12, died = n - Math.round((d.baseline ?? 0) * n);
  const one = core?.one_in ?? (core?.odds ? Math.max(1, Math.round(1 / core.odds)) : undefined);
  return { event, oneIn: one !== undefined ? `${one}` : died > 0 ? `${Math.max(1, Math.round(n / died))}` : `${n}+` };
}

/** Cut 13 §5: the last forecast's reach at `depth` (a 0..1 fraction, the number the camp's bar showed); undefined without a
 *  forecast, or when the floor is past what it knew (`known_to`). */
export function forecastSaid(app: App, depth: number): number | undefined {
  const f = app.lastForecast; if (!f || depth > f.known_to) return undefined;
  return f.depths.find((x) => x.depth === depth)?.reach;
}

/** Blind c4705f9 (A: `forecast said D13 100%` over a D13 death read as a contradiction): reach is the share of sends that got TO the
 *  floor, not past it. With the sample size on the wire the line counts sends and adds the floor's own odds — the share that reached the
 *  next floor (`forecast · reach D13 8/8 · past 5/8`); without it (an older core) the camp's share verbatim. */
export function forecastSaidText(f: { depths: { depth: number; reach: number }[]; known_to: number; sims?: number; low?: number } | null | undefined, depth: number, said: number): string {
  const n = f?.sims && f.sims > 0 ? Math.round(f.sims) : 0;
  if (!f || !n) return /* copy:callout */ `forecast said D${depth} ${share(said, lowOf(f))}`;
  const of = (x: number): string => `${Math.round(x * n)}/${n}`;
  const next = depth + 1 <= f.known_to ? f.depths.find((x) => x.depth === depth + 1)?.reach : undefined;
  return /* copy:death_line */ `forecast · reach D${depth} ${of(said)}${next !== undefined ? ` · past ${of(Math.min(said, next))}` : ""}`;
}

/** Cut 30 (core, `Death.lever`, before the pen opens): the one cheapest lever — `spend` (buy the counter item), `package` (a swap), `wait`
 *  (the heir rests, the floor's odds rise) — its words (`+ heal potion`) and its effect (`survives more`). Shape provisional: the core's. */
export type Lever = { kind: "spend" | "package" | "wait" | "tactic" | string; text: string; id?: string; variant?: number };   // Cut 115 §4: a `tactic` lever names its package and variant

/** The hp the hero had before the blow that killed him (the last blow's hp plus its damage); none without a blow. */
export function momentHp(d: Death): number | undefined {
  const blow = d.trace.blows?.length ? d.trace.blows[d.trace.blows.length - 1] : d.trace.blow;
  return blow ? Math.max(1, blow.hp + blow.dmg) : undefined;
}

/** Death screen v2 — why, from the player's side, one line: what would have changed it (the lead fix, inverted: a retreat fix reads `never
 *  stepped back`, a return `returned too late`, a heal bought `no heal left`), luck as luck (`bad luck · 1 in 6`), a boss's `needs a
 *  counter`; the verdict word (gap · row · order · route · stall) becomes this wording, never a term. Hide the line when no specific explanation is known. */
export function whyOf(app: App, d: Death, lead: Patch | undefined, rows: Row[], luck: { oneIn: string } | null, drove?: DrivenOff, causeRow?: string): string {
  const pkg = (d as Death & { package?: string }).package;   // Cut 30 (core): the package whose row acted — `Steady · …`
  const pre = pkg ? `${pkg} · ` : "";
  if (drove) return pre + /* copy:callout */ "needs a counter";
  if (luck) return /* copy:callout */ `bad luck · 1 in ${luck.oneIn}`;
  if (d.verdict === "stall") return /* copy:callout */ "stuck looping";
  if (d.verdict === "route" && d.route_cause) return /* copy:callout */ `wrong stairs · D${d.route_cause.fork}`;
  if (d.verdict === "row" && causeRow) return pre + /* copy:callout */ `${causeRow} backfired`;
  if (d.verdict === "order" && d.cause_row !== undefined) return pre + /* copy:callout */ `${refName(d.cause_row)} never fired`;
  if (!lead) return "";
  const v = lead.row.verb.v, a = lead.row.verb.a ?? "";
  if (lead.insert_at < 0) return "";
  if (lead.moves_from !== undefined && lead.moves_from >= 0) return pre + /* copy:callout */ `${ruleName(rows, lead.moves_from)} never fired`;
  if (lead.remove) return pre + /* copy:callout */ `${ruleName(rows, lead.insert_at)} backfired`;
  if (v === "return" || v === "bank") return rows.some((r) => r.verb.v === "return" || r.verb.v === "bank") ? /* copy:callout */ "returned too late" : /* copy:callout */ "never turned back";
  if (/^(retreat|kite|back_corridor|blink|vanish|shadowstep|smoke)$/.test(v)) return /* copy:callout */ "never stepped back";
  if (v === "drink" || v === "read" || v === "throw") {
    const item = a.split(",")[0].replace(/^tag:/, "").replace(/_/g, " ") || "potion";
    if (lead.buys) return /* copy:callout */ `no ${item} left`;
    return rows.some((r) => r.verb.v === v && (r.verb.a ?? "") === a) ? /* copy:callout */ `${verbLabel(lead.row.verb)} too late` : /* copy:callout */ `never used ${item}`;
  }
  if (/boss/.test(a) || /boss/.test(JSON.stringify(lead.row.conds))) return /* copy:callout */ `${d.cause.replace(/_/g, " ")} needs a counter`;
  void app;
  return "";
}

/** The rest view: the lit tablet and up to `max − 1` others that help (not dim, not harming, not held, not folded); the rest wait for
 *  `details` (`rest-off`). Returns the tablets shown. */
export function restLayout(box: HTMLElement, max: number, measuring = false): number {
  const tabs = [...box.querySelectorAll<HTMLButtonElement>("button.patch")];
  const shown = new Set<HTMLButtonElement>();
  const lit = tabs.find((b) => b.classList.contains("top"));
  if (lit && !lit.matches(".below, .held")) shown.add(lit);
  // an alternative earns its place by the whole run: none while the reach is measuring; once landed, only one that moves it up (a fix
  // the replays like but the whole run does not — `reach D8 same`, an exit's `return early` — waits under details)
  const landed = !!box.querySelector(".delta:not(.pending)") && ![...box.querySelectorAll(".delta")].some((x) => /…/.test(x.textContent ?? ""));
  for (const b of tabs) {
    if (shown.size >= max) break;
    if (b.closest(".patches-fold") || b.matches(".below, .harms, .held, .neg")) continue;
    if (shown.size && (measuring || (landed && !b.querySelector(".delta.up")))) continue;
    shown.add(b);
  }
  tabs.forEach((b) => b.classList.toggle("rest-off", !shown.has(b)));
  // once the whole run is measured a gain reads the reach it moves (`reach D8 42→51%`, `reach D8 same`; an exit's `return early`) — the
  // camp's measure the gem ranks by, so the lit fix never reads worse than the one beside it
  for (const b of shown) {
    const dl = b.querySelector<HTMLElement>(".delta"), t = dl?.classList.contains("exit") ? dl.firstChild?.textContent ?? "" : dl?.textContent ?? "";
    // (a reach that does not move keeps the replays' count: `reach D8 same` read as "no help")
    if (t && !/…/.test(t) && (dl?.classList.contains("up") || dl?.classList.contains("exit"))) b.dataset.eff = t.replace(/\s*±\d+%?$/, "").trim();
  }
  box.classList.toggle("rest-none", !shown.size);
  return shown.size;
}

/** The `details` fold's hint (blind check: "no preview of what is inside"): the fixes it holds, then the trace — `· 3 fixes · trace`. */
function foldHint(patches: HTMLElement): HTMLElement {
  const n = patches.querySelectorAll("button.patch").length;
  return h("small", { class: "fold-hint num" }, n ? /* copy:callout */ ` · ${n} ${n === 1 ? "fix" : "fixes"} · trace` : /* copy:callout */ " · trace");
}

/** Cut 115 §4: the death's tactic fix as a tablet above the patches — `try · gas step · burn`; a tap wears it (credited taught). */
function pickTablet(app: App, pick: Lever): HTMLElement {
  const btn = h("button", { class: "death-pick tablet", "data-pick": pick.id ?? "", "data-variant": String(pick.variant ?? 0),
    onclick: () => { if (pick.id && app.engine.takeFix) void app.mutate(() => app.engine.takeFix!(pick.id!, pick.variant ?? 0), /* copy:callout */ pick.text, true).then(() => app.go({ kind: "camp" })); } },
    h("span", { class: "lever-kind" }, /* copy:button */ "try"), h("b", { class: "lever-name" }, pick.id ? packageIcon(pick.id) : "", pick.text), h("span", { class: "lever-go", "aria-hidden": "true" }, "›"));
  return btn;
}
