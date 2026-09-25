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
import type { App, Mounted } from "../app";
import type { Death, ExitLine, Row } from "../engine/types";
import { morgueVerbs } from "./chain";
import { h, copyText, items } from "./dom";
import { lowOf, share } from "./forecast";
import { openGoldSheet } from "./gold";
import { applyOf, fillReach, patchRows } from "./patches";
import { closeX, openSheet } from "./sheet";
import { gem, portrait, renderBar, renderConsole, tile } from "./frame";
import { lostLabel, noteText, rowLabel, verbLabel } from "./tokens";
import { traceTable } from "./trace";
import { mergeFinds, renamer } from "./report";

/** Cut 10 §3: the core's `3 over` margin reads `3 hp short` wherever it is displayed (`N hp short` and others pass through). */
export const marginText = (m: string): string => m.replace(/^(\d+) over$/, /* copy:callout */ "$1 hp short");

/** The headline's margin segment: a stall's is the guard's reason (`no path`); an hp margin (`3 over` / `3 hp short`) is left
 *  out — four QA players read `1 hp short` as the hp left (the morgue still carries it); an empty margin is no segment. */
export const headlineMargin = (m: string): string => m.split(" · ").filter((x) => x && !/^\d+ (over|hp short)$/.test(x)).map(marginText).join(" · ");   // QA 23ed91f: `1 hp short · 5 unknown unused` kept its hp part

/** QA 23ed91f (K: "`died $0` and `keeps 0%` say the same thing twice"): a death's line drops its `keeps 0%` (the lead says it). */
/** QA 1a2a4a9 (O, P, and many before: "`+$40 wake` — no source"): the core's wake pay tops the next heir's purse up to $40 — it reads
 *  as whose it is (`heir purse +$40`), here, on the gold sheet and in the report. */
export const wakeShown = (t: string): string => t.replace(/\+\$(\d+) wake\b/g, /* copy:callout */ "heir purse +$$$1").replace(/\bwake pay\b/g, /* copy:callout */ "heir purse").replace(/\bpurse full\b/g, /* copy:callout */ "no top-up");
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
export type ExtrasOpts = { brief?: boolean; /** the pile's list is already in the text (`lineShown`) */ pileShown?: boolean };
export const exitExtras = (x: Pick<ExitLine, "text" | "stolen" | "stolen_gold" | "purse_full" | "wake" | "swapped" | "swap_left" | "shelved" | "start" | "start_short" | "found" | "bones">, name: (label: string) => string = (l) => l.replace(/_/g, " "), opts: ExtrasOpts = {}): string => {
  // QA 912e135 (qaW: `stolen blink, $3` read as the blink's worth): the coins thieves kept are their own term — `stolen blink + $3`
  const items = (x.stolen ?? []).map(name), coins = x.stolen_gold && x.stolen_gold > 0 ? `$${x.stolen_gold}` : "";
  // QA 0c6e126 (qaZ): stolen gear carries its enchant (`leather +1`); the coins stay their own term (`stolen leather +1 + $6`, qaW's form)
  const stolen = items.length ? `${items.join(", ")}${coins ? ` + ${coins}` : ""}` : coins;
  // QA a946e04 (T: a D5 start at $32 ran from D1 with no word): the run's start fell back — the toll was more than the purse
  return (x.start_short && !/toll short/.test(x.text) ? /* copy:callout */ ` · from ${dTag(x.start ?? 1)} · toll short` : "")
    + (stolen && !/\bstolen\b/.test(x.text) ? /* copy:callout */ ` · stolen ${stolen}` : "")
    // QA 0c6e126 (qaY: `−$5 swapped` named no item): the costly swaps' toll names what they left on the floor (`−$5 left axe`)
    + (x.swapped && x.swapped > 0 && !/\bswapped\b/.test(x.text) ? x.swap_left?.length ? /* copy:callout */ ` · −$${x.swapped} left ${shelvedText(x.swap_left.map((b) => ({ kind: name(b.kind), n: b.n })))}` : /* copy:callout */ ` · −$${x.swapped} swapped` : "")
    + (x.wake && x.wake > 0 && !/\bwake\b|heir purse/.test(x.text) ? /* copy:callout */ ` · heir purse +$${x.wake}` : "")
    // QA 912e135 (qaW: `no top-up` on every line — "the top-up it refers to is never shown"); QA 0c6e126 (qaY: `heir purse ≥$40` — "the
    // `≥` has no source", twelve of them against one `+$20 heir purse` in the ledger): only a top-up the ledger carries is named (`wake`)
    // QA 0c6e126 (qaZ: "no shelf appears anywhere"): the loadout's list is SUPPLIES
    + (x.shelved?.length && !/→ (shelf|supplies)\b/.test(x.text) ? /* copy:callout */ ` · found ${shelvedText(x.shelved)} → supplies` : "")
    + fatesText(x.found, x.bones ? undefined : "bones", opts.brief)
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
  return brief ? /* copy:callout */ ` · ${f} ${ys.reduce((a, y) => a + y.n, 0)}` : /* copy:callout */ ` · ${f}${f === "bones" ? ":" : ""} ${shelvedText(ys)}`;
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

export function renderDeath(app: App, d: Death, lost: string[] = [], kept = false): Mounted {
  // a stall's margin is the guard's reason (or empty): the headline never carries an empty segment
  const seg = headlineMargin(d.margin ?? "");
  const margin = seg ? ` · ${seg}` : "";
  // Cut 19 §4: a `row` verdict — the player's own row was the dying action; the headline names it (`R2 drink unknown`), the seal reads ROW
  const causeRow = d.verdict === "row" && d.cause_row !== undefined ? rowName(d.cause_row, (d.rules?.rows ?? app.rules.rows)[d.cause_row]) : "";
  // Cut 17 §4: the line is laid on the defeat banner — the cause and depth in the display face, the verdict in the seal under it
  // (one word, engine data: `gap` · `dice` · `stall`); the text reads as before (`goblin archer · D6 · gap`)
  const causeText = /* copy:death_line */ `${d.cause.replace(/_/g, " ")} · D${d.depth}${causeRow ? ` · ${causeRow}` : ""}${margin}`;
  // QA 1a2a4a9 (P: `STALLED · R2 RETREAT ↔ EXPLORE · D6 · KEEPS $0` ran off both edges at 400 px): a stall's headline wraps between its
  // ` · ` segments (each whole: the loop `R2 retreat ↔ explore` never breaks) and steps its face down until the widest segment fits
  const causeEl = d.verdict === "stall"
    ? h("span", { class: "cause" }, ...causeText.split(" · ").flatMap((seg, i, all) => [i ? " " : "", h("span", { class: "seg" }, seg, i < all.length - 1 ? " ·" : "")]))
    : h("span", { class: "cause" }, causeText);
  // QA 912e135 (qaW: the seal `GAP` and the banner answered no tap): the seal names what the patches answer — a tap brings them up and
  // lights the first; the banner names the moment — a tap brings up the trace
  const seal = h("button", { class: /* copy:none */ `verdict ${d.verdict}`, onclick: () => { patches.scrollIntoView({ block: "center", behavior: "smooth" }); const p = patches.querySelector<HTMLElement>(".patch.top") ?? patches.querySelector<HTMLElement>(".patch"); if (p) { p.classList.remove("flash"); void p.offsetWidth; p.classList.add("flash"); } } }, d.verdict);
  const line = h("h1", { class: "death-line" }, h("button", { class: "cause-btn", onclick: () => tracePanel.scrollIntoView({ block: "center", behavior: "smooth" }) }, causeEl), h("span", { class: "sep" }, " · "), seal);
  // Cut 13 §4: the run's last two notes, engine data verbatim (`The green one: fire. Gambled: fire potion.`)
  // QA 92eb880: never a `… saved him.` over a death (M, N: read as the verdict), nor the cage's loot beat (`Took the axe +1 from the cage.`,
  // M: "unrelated to the ogre") — the core filters the first; the client keeps both off whatever the build
  const shownNotes = (d.notes ?? []).filter((n) => !/ saved him\.$/.test(n) && !/^The cage opens\b|^Took .* from the cage\.$/.test(n));
  // QA 912e135 (qaX: `ogre: telegraph` / `ogre: heavy` under the banner read as tappable because-lines): a fact the run learned says so
  const noteLine = (n: string): string => /^[A-Z][a-z]+(?: [a-z]+)?: [a-z_]+\.$/.test(n) ? /* copy:callout */ `learned ${noteText(n)}` : noteText(n);
  // Cut 24 §2: what was new this run (the core's `ExitLine.news`: a first, a record, a named kill), under the banner before the notes —
  // a death that also slew the Warlord says so; the `differ` line of a run with nothing new is left to the report
  const newsTexts = mergeFinds((d.line?.news ?? []).filter((n) => n.k !== "differ")).map((n) => n.text);
  const news = newsTexts.length && !kept ? h("div", { class: "death-news num" }, newsTexts.slice(0, 3).join(" · ")) : null;
  const notes = shownNotes.length ? h("div", { class: "death-notes num dim" }, ...shownNotes.slice(-2).map((n) => h("div", { class: "note" }, noteLine(n)))) : null;
  // Cut 13 §5: a `dice` death says what the forecast said for that depth — the reach the camp showed for the floor, verbatim
  // an old death (the chronicle) was sent under another forecast: today's would be a false number (QA on 56f2a1d: `forecast said D7 0%`)
  const said = d.verdict === "dice" && !kept ? forecastSaid(app, d.depth) : undefined;
  const forecastLine = said !== undefined ? h("div", { class: "forecast-said num dim" }, /* copy:callout */ `forecast said D${d.depth} ${share(said, lowOf(app.lastForecast))}`) : null;
  // Cut 6 §1: the exit's arithmetic, verbatim from the engine (`$144 carried · death keeps 0% → $0 · bones: 7 items on D5`)
  // Cut 11 §5: tappable — the gold sheet filtered to this run's movements
  // Cut 20 §4 (AC: "$80 gone after death, `repeat · $80` — only understood when removing refunded $40"): the loadout's re-pack for
  // the next heir, charged at this exit, is a line under it (`repeat −$80`) — the gold sheet it opens lists it too
  const repeat = kept ? 0 : repeatAfterExit(app.lineage.gold_ledger ?? []);
  const ledger = d.line?.text ? h("div", { class: "ledger-line num dim" }, h("button", { class: "ledger-btn", onclick: () => openGoldSheet(app, d.line) }, lineShown(d.line, renamer(app.lineage))),
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
  const light = (btn: HTMLButtonElement): void => { patches.querySelector(".patch.top")?.classList.remove("top"); top = topPatch(patches, btn); const g = makeGem(); gemBtn.replaceWith(g); gemBtn = g; };
  const select = (btn: HTMLButtonElement): void => { picked = true; light(btn); };
  const patches = patchRows(app, d.patches, d.baseline ?? 0, d.trace, { nothingBeatsBase: d.nothing_beats_base, stall: d.verdict === "stall", select, moment: d.depth, replays: d.replays });   // Cut 14 §4: the trace names the least-fired row on a full set
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
  const stalled = d.verdict === "stall";
  const face = portrait(app, stalled ? { hp: 1, label: `D${d.depth}` } : { hp: 0, dead: true, label: `D${d.depth}` });
  const makeGem = (): HTMLButtonElement => top
    // QA 23ed91f (K: "the gem reads `100%` with no label … I read it as the run's result"): the number, and the word the tap does under it
    ? gem({ label: h("span", { class: "gem-in" }, h("span", { class: "gem-n" }, top.label), top.label !== "buy" && top.label !== "edit" ? h("small", { class: "gem-w" }, /* copy:label */ "apply") : ""), cls: "patch-gem", pulse: true, onclick: () => { if (top) void applyOf.get(top.btn)?.(); } })
    : gem({ label: /* copy:button */ "edit", pulse: true, onclick: () => app.go({ kind: "camp" }) });
  let gemBtn = makeGem();
  const cons = renderConsole({ portrait: face.el, gem: gemBtn, tiles: [
    top ? tile({ id: "edit", label: /* copy:button */ "edit", icon: "edit", onclick: () => { app.editing = true; app.go({ kind: "camp" }); } }) : null,
    tile({ id: "morgue", label: /* copy:button */ "morgue", icon: "morgue", onclick: openMorgue }),
    tile({ id: "camp", label: /* copy:button */ "camp", icon: "camp", onclick: () => app.go({ kind: "camp" }) }),
  ] });
  const tracePanel = h("div", { class: "parchment trace-panel" }, ...trace);
  const well = h("div", { class: "well death-well" },
    h("div", { class: "defeat" }, h("div", { class: "banner-cloth" }, line), news, notes, forecastLine, ledger, eggs, bones),
    // QA 23ed91f (K: "the patches sit below the fold, under the console"): the patches, the screen's point, before the trace
    patches,
    tracePanel);
  const el = h("main", { class: `death frame${stalled ? " stalled" : ""}` }, bar.el, well, cons.el);
  // Cut 18 §4: a stall's cause is the rows' loop (`R2 retreat ↔ explore`) — it reads whole on one line: the face steps down until it fits
  if (d.verdict === "stall") { line.classList.add("loop"); fitLine(line.querySelector<HTMLElement>(".cause")); }
  // QA 23ed91f: the patches' reach is the camp's own measure, landing after the paint (`deathDeltas`: seconds in wasm) — the
  // screen never waits on it; a reach still pending reads `reach …` until then
  let gone = false;
  if (d.patches.some((p) => p.camp_pending) && app.engine.deathDeltas) {
    const shown = d.patches;
    // QA 778fa1b (qaU: the lit tablet and the gem moved 1 → 2 on their own ~5 s after arrival): the landing fills each tablet's reach
    // and dims a loss; the order and the lit tablet (the gem's) stay as the screen first put them
    // QA 912e135 (qaX: the gem's `100% APPLY` stayed on `foe: telegraph → retreat · drops R1` once its reach landed at `D8 −62`): the
    // screen's own lit tablet (not the player's pick) that the landing shows losing 10 points or more yields the light to the first
    // tablet that does not lose — a patch that costs that much never leads
    setTimeout(() => { if (!gone) void app.engine.deathDeltas!(d.run_id).then((f) => {
      if (gone || !f?.length) return;
      fillReach(patches, shown, f);
      const btns = [...patches.querySelectorAll<HTMLButtonElement>("button.patch")];
      // (an exit's cost is its word, `return early`, and the core keeps a costly exit off the lead: only a row that stays in the fight moves)
      const cost = (b: HTMLButtonElement): number => { const p = shown[btns.indexOf(b)]; return p && p.insert_at >= 0 && !p.camp_pending && !(p.exits ?? /^(return|bank)$/.test(p.row.verb.v)) ? Math.round(p.forecast_delta * 100) : 0; };
      const lit = top?.btn;
      if (!picked && lit && cost(lit) <= -10) { const alt = btns.find((b) => b !== lit && !b.classList.contains("below") && !b.classList.contains("neg") && cost(b) > -10); if (alt) light(alt); }
    }).catch((e) => console.warn("deathDeltas", e)); }, 0);
  }
  return { el, dispose: () => { gone = true; bar.dispose(); } };
}

/** QA 0c6e126 (qaZ): a rule row named under a death's trace — the row, how often it acted in the trace's turns and why it did not
 *  on the last one (engine data: `TraceTurn.rows`), and `edit` (the camp on the row); the sheet's `×` leaves the death screen as it was. */
function openRowSheet(app: App, d: Death, i: number): void {
  const row = (d.rules?.rows ?? app.rules.rows)[i]; if (!row) return;
  const turns = d.trace.turns, acted = turns.filter((t) => t.row === i).length;
  const last = turns[turns.length - 1], why = last?.row === i ? /* copy:callout */ "fired" : last?.rows?.find((w) => w.row === i)?.why;
  openSheet((close) => h("div", { class: "sheet-body row-sheet" },
    h("div", { class: "label row-label" }, `R${i + 1}`, " ", h("small", { class: "dim" }, rowLabel(row)), closeX(close)),
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
export function rowName(i: number, row?: Row): string {
  const v = row ? verbLabel(row.verb).trim().split(/\s+/).slice(0, 2).join(" ") : "";
  return `R${i + 1}${v ? ` ${v}` : ""}`;
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
  return { btn, label: unlock ? /* copy:button */ "buy" : btn.classList.contains("held") ? /* copy:button */ "edit" : surv ?? "" };
}


/** Cut 13 §5: the last forecast's reach at `depth` (a 0..1 fraction, the number the camp's bar showed); undefined without a
 *  forecast, or when the floor is past what it knew (`known_to`). */
export function forecastSaid(app: App, depth: number): number | undefined {
  const f = app.lastForecast; if (!f || depth > f.known_to) return undefined;
  return f.depths.find((x) => x.depth === depth)?.reach;
}
