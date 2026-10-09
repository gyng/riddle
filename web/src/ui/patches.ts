// Patch rows, shared by the death screen and the report's stall section: tappable `cond → verb` with the survival
// (or reach) share and the forecast delta; tapping applies the patch and opens the camp on the row.
// Cut 11 §2: a patch with `root` names the chain's root under the row (`← den took the heal`, accent); `insert_at: -1` is
// an unlock pseudo-patch (`◆2 cond: alert · buy`): the tap buys the cond's unlock, then opens the camp on the row (the
// set's own locked row, or the patch's row inserted at the top when the set lacks it); §4: a
// `below_bar` candidate renders dimmed with `survives 40% · below bar` — the named alternative of a `dice` death.
// Cut 14 §4's silent `↑ R3` is withdrawn (Cut 15 §3: "the caster patch replaced my drink unknown row"): on a full set an insert
// patch reads `+ drop one` and the tap opens a sheet of the set's own rows (card rows never), each with its fired count when known
// (`R5 depth ≥ 8 → bank · 0/16`), the least-fired marked when it is the only one at its count; tapping a row drops it and the patch
// lands where it was measured (`App.applyPatchOver`); dismissing the sheet (`×`, Escape, the backdrop) leaves the set whole and the death screen up.
import type { App } from "../app";
import type { Patch, Row, Trace } from "../engine/types";
import { h, pct } from "./dom";
import { closeX, openSheet } from "./sheet";
import { condLabel, verbLabel, isCardRow, isPkgRow, refName, rowLabel, ruleName, sameCond, sameVerb } from "./tokens";
/** A row the cap counts and a drop frees (`max_rows` caps the pen's own rows: never a card's, never a package's — the core recompiles
 *  a dropped package row straight back). */
const ownRow = (r: Row): boolean => !isCardRow(r) && !isPkgRow(r);
import { icon, verbIcon } from "./skin";
import { kwHost } from "./tips";
/** gfx round 2 (raters: "the three fix rows are plain brown slabs — give each an icon, as the target does"): the fix's action plaque. */
const patchPlaque = (row: Row | undefined): HTMLElement | "" => { const id = row ? verbIcon(row.verb.v) : null; return id ? h("span", { class: "vplaque", "aria-hidden": "true" }, icon(id)) : ""; };

const sameRow = (a: Row, b: Row): boolean => a.conds.length === b.conds.length && a.conds.every((c, i) => sameCond(c, b.conds[i]) && c.n === b.conds[i].n) && sameVerb(a.verb, b.verb);

/** The unlock behind a pseudo-patch: the wire's `unlock` when sent, else the row's first cond that is an unlockable token
 *  (`alert>=` → `cond_alert`, `on_kill` → `cond_on_kill`; core meta.rs / fake COND_UNLOCK). */
export function unlockOf(app: App, p: Patch): string | undefined {
  if (p.unlock) return p.unlock;
  const locked = new Set((app.vocab?.locked ?? []).map((l) => l.cond.k));
  const offered = new Set((app.vocab?.conds ?? []).map((c) => c.k));
  const c = p.row.conds.find((x) => locked.has(x.k)) ?? p.row.conds.find((x) => !offered.has(x.k));
  return c ? `cond_${c.k.replace(/[<>]=?$/, "")}` : undefined;
}

/** Fractions 0..1 from the core: survive, forecast_delta. A stall patch marks its target row: `R1 ↻` replaces it, `R1 −` removes it.
 *  Cut 4 §2: a row reads `survives 100% · base 75%` (death: `baseline` is the unpatched survival) or `reach 40% · base 35%`
 *  (stall: `survive` is the patched reach, base = survive − delta), then `reach +5%`, or `reach ~0` when the delta rounds to 0
 *  (QA on 50bb162: "reach missing on some patches"; the unlock cards say it that way). */
/** Cut 14 §4 (Cut 15 §3: the row the drop sheet marks): the row a patch replaces on a full set — the own row (never a card's) that fired least: `app.rowFires` (the
 *  watched run's rule events, or the absence's usage lines), else the rows the trace shows firing; among equals the lowest in
 *  the list (the one the rows above it overshadow). −1 when the set has no own row. */
export function leastFiredRow(app: App, trace?: Trace): number {
  const rows = app.rules.rows, fires = firesOf(app, trace);
  let best = -1;
  rows.forEach((r, i) => { if (!ownRow(r)) return; if (best < 0 || (fires[i] ?? 0) <= (fires[best] ?? 0)) best = i; });
  return best;
}

/** Cut 19 §4: the own row the core says an insert on a full set drops (`Patch.drops`), −1 when absent or not an own row of the set now. */
export function dropsOf(app: App, p: Patch): number {
  const i = p.drops; if (i === undefined || i < 0) return -1;
  const r = app.rules.rows[i];
  return r && ownRow(r) ? i : -1;
}

/** The least-fired own row when its count is unique among the own rows, else −1 (the drop sheet's `↓`). */
function uniqueLeast(app: App, trace?: Trace): number {
  const least = leastFiredRow(app, trace); if (least < 0) return -1;
  const fires = firesOf(app, trace), n = (i: number): number => fires[i] ?? 0;
  return app.rules.rows.some((r, i) => i !== least && ownRow(r) && n(i) === n(least)) ? -1 : least;
}
/** Each row's fires: `app.rowFires`, else the rows the trace shows firing. */
function firesOf(app: App, trace?: Trace): number[] {
  const fires: number[] = app.rowFires ? [...app.rowFires] : app.rules.rows.map(() => 0);
  if (!app.rowFires && trace) for (const t of trace.turns) if (t.row >= 0) fires[t.row] = (fires[t.row] ?? 0) + 1;
  return fires;
}

/** QA 92eb880 (M, the worst death: `DICE` over three patches all `survives 100% · below bar`): `nothingBeatsBase` — the core's
 *  `Death.nothing_beats_base`: one line over the block says so (`nothing beats base · base 100%`) and no patch reads `below bar`.
 *  `depth`: the floor a stall patch's reach is measured on (`reach D7 17%`; N: "reach of which floor?"). */
/** `stall`: a stall's verdict screen — the core's `survive` is the share of replays that end the loop (the hero came home either way):
 *  `unstuck 100% · base 8%`, never `survives` (QA 1a2a4a9, P: "`survives 100%` for a hero who came home"). */
export type PatchOpts = { nothingBeatsBase?: boolean; depth?: number; stall?: boolean; plain?: boolean; select?: (btn: HTMLButtonElement) => void;
  /** QA 912e135 (qaW: `survives 100% · unpatched 50%` read as the run surviving, and as "a coin flip" under GAP): the floor of the death
   *  the shares are replays of — a head over the block says so (`D6 death · replayed`) */
  moment?: number;
  /** QA 0c6e126 (qaY: `unpatched 58%` on two unrelated deaths — "a shared cache?"; three patches `survives 92%` read as the run's odds):
   *  the replays the shares count (`Death.replays`) — `survives 11/12 · unpatched 7/12`, the count of this death's replays, never a % */
  replays?: number };
/** A replay share as its count (`11/12`) when the replays are known, else a %. */
const replayShare = (x: number, n?: number): string => n ? `${Math.round(x * n)}/${n}` : pct(x);
/** Report suggestions keep exact conditions, with plain labels for familiar actions. */
function plainPatchRow(row: Row): HTMLElement {
  const action = row.verb.v === "attack" && row.verb.a === "tag:boss" ? /* copy:callout */ "Target the boss"
    : row.verb.v === "bank" ? /* copy:label */ "Collect gold"
    : row.verb.v === "return" ? /* copy:label */ "Return home" : verbLabel(row.verb);
  const conditions = row.conds.map(c => c.k === "foe_tag" && c.t === "boss" ? /* copy:callout */ "Boss in sight"
    : c.k === "hp<" && c.n !== undefined ? /* copy:rule_token */ `Health below ${c.n}%`
    : c.k === "depth>=" && c.n !== undefined ? /* copy:rule_token */ `Floor ${c.n}+`
    : c.k === "loot>=" && c.n !== undefined ? /* copy:rule_token */ `Carry $${c.n}+` : condLabel(c));
  return h("span", { class: "patch-wording" }, h("b", { class: "patch-action" }, action), " ",
    h("small", { class: "patch-condition" }, conditions.length ? conditions.join(" · ") : /* copy:label */ "Always"));
}
export function patchRows(app: App, patches: Patch[], baseline?: number, trace?: Trace, opts: PatchOpts = {}): HTMLElement {
  const head = opts.nothingBeatsBase && patches.length
    ? h("div", { class: "patches-head num dim" }, /* copy:death_line */ `none beats ${replayShare(baseline ?? 1, opts.replays)} as is`) : null;
  // QA 778fa1b (qaU: PLATEAU `base 8%` on patch 1, `base 9%` on patch 2 — "two bases for one lineage"): a stall block has one base, the
  // unpatched reach (the first patch's reach less its move, unrounded), and each patch's move is its rounded reach less it
  const lead = baseline === undefined ? patches.find((p) => !p.below_bar) : undefined;
  const stallBase = lead ? Math.max(0, Math.round((lead.survive - lead.forecast_delta) * 100)) : undefined;
  // the death screen's rest view (death.ts): each tablet's short name and effect, read by its CSS (`data-short`, `data-eff`) — the
  // rule by its words among the offered fixes and the set (`+ retreat vs gas`), the effect one count (`survives 10/12`)
  const names = [...patches.map((q) => q.row), ...app.rules.rows];
  const rows = patches.map((p, pi) => {
    const delta = stallBase !== undefined ? Math.round(p.survive * 100) - stallBase : Math.round(p.forecast_delta * 100);
    // QA 23ed91f (K: "`reach 92% · base 8% · reach +83%`: 92 − 8 ≠ 83, and `reach` twice"): a stall patch's base is the rounded reach
    // less the rounded delta, so the three numbers add up; its delta then drops the word (`+84%`)
    const unlock = p.insert_at < 0;
    // A row the set already holds (an old death opened from the chronicle, a patch tapped twice) is not inserted again:
    // the row reads `at R2` and the tap opens the camp on it (QA: "tapped patch → R1 inserted AGAIN → 5/4")
    // Cut 25 §2: a move (`moves_from`) puts the set's own row above another — the row is held by design; it reads `move R5 above R2`
    const move = !unlock && p.moves_from !== undefined && p.moves_from >= 0;
    const held = unlock || p.remove || p.replace || move ? -1 : app.rules.rows.findIndex((r) => sameRow(r, p.row));
    const stallish = baseline === undefined && held < 0 && !p.below_bar;   // the first line already says `reach`
    // Cut 15 §3: an insert onto a full set asks which own row to drop (`+ drop one`)
    // Rater A on c4705f9 (`5/4 · drop one` blocked SEND after an apply): a move or a rewrite that takes a package row into the pen
    // takes a row too (`App.patchTakesRow`) — it asks first, like an insert
    const full = !unlock && held < 0 && app.rowsFull && app.patchTakesRow(p) && app.rules.rows.some(ownRow);
    // QA a946e04 (S: `R1 − hp < 20% · foes ≥ 1 → drink unknown` — "delete R1?"): a cut reads as one (`cut R1`); a replace keeps `R1 ↻`
    const target = move ? h("small", { class: "target move-tag" }, /* copy:death_line */ `move above ${refName(p.insert_at)} `)
      : p.remove || p.replace ? h("small", { class: "dim target" }, opts.plain ? p.remove ? /* copy:label */ "Remove " : /* copy:label */ "Change " : p.remove ? /* copy:callout */ "cut " : /* copy:callout */ `replaces ${refName(p.insert_at)} `)
      // Cut 27 §5 (AT: a gas death after cutting the bloat row stamped `dice`): a row the player removed, put back where it was
      : p.restores !== undefined ? h("small", { class: "target restore-tag" }, /* copy:callout */ "restore ")
      // Cut 19 §4: the core names the row the insert drops (`Patch.drops`, the dead run's least-fired own row) — `+ drop R5`; the tap
      // still opens the drop sheet on it (marked), so the player may drop another
      : "";
    // QA 1a2a4a9 (O: `+ drop R2 hp < 40% → drink heal` read as "put it at R2"): the drop trails the row it makes room for — `drops R2`
    // QA 778fa1b (qaU: `drops R4` and the drop sheet then marked R4 `16/16` — the busiest row): the named drop carries its fires when known
    const named = full ? dropsOf(app, p) : -1, fires = named >= 0 && app.rowFires ? app.rowFires[named] ?? 0 : undefined, total = app.rowFiresOf ?? (app.rowFires ? app.rowFires.reduce((a, b) => a + b, 0) : 0);
    const dropTag = full ? h("small", { class: "dim target drop-tag" }, " · ", named >= 0 ? /* copy:callout */ `drops ${refName(named)}` : /* copy:callout */ "drops one",
      fires !== undefined && total > 0 ? h("span", { class: "num fired" }, /* copy:callout */ ` · ${fires}/${total} fires`) : "") : "";
    // QA a946e04 (T: `retreat · survives 75% · base 75%` listed like a fix): a death patch that survives no more than the rules as they
    // are changes nothing — dim, `no gain`
    // Cut 28 §2 (core `Patch.no_gain`): the core's word wins; else the survive share against the unpatched one
    const noGain = (p.no_gain === true && !p.below_bar && held < 0 && !unlock) || (baseline !== undefined && !opts.stall && !p.below_bar && held < 0 && !unlock && Math.round(p.survive * 100) <= Math.round(baseline * 100));
    const rs = (x: number): string => replayShare(x, opts.replays);
    const line = held >= 0
      ? /* copy:callout */ "already written"
      // Cut 26 §6 (control rater AQ: `nothing beats unpatched 12/12` over a tablet reading `survives 12/12`): a candidate that does not
      // beat the unpatched replays was tried, not a help — its count never reads as a headline
      : opts.nothingBeatsBase ? /* copy:callout */ "replayed · no gain"
      : p.below_bar
      // QA 308f045 (qaAD: `move R6 above R4 … tried · 0/12 · below bar` — "I never tried it"): the replays tried it — `replayed`
      ? /* copy:callout */ `replayed · ${rs(p.survive)} · below bar`
      : baseline === undefined
        ? /* copy:callout */ `reach ${opts.depth !== undefined ? `D${opts.depth} ` : ""}${pct(p.survive)} · was ${Math.max(0, Math.round(p.survive * 100) - delta)}%`
        // QA a946e04 (S: `base 25%` on every patch — "the base of what?"): the rules as they ran, replayed — `unpatched 25%`
        : opts.stall ? /* copy:callout */ `unstuck ${rs(p.survive)} · was ${rs(baseline)}`
        : noGain ? /* copy:callout */ `survives ${rs(p.survive)} · no gain` : /* copy:callout */ `survives ${rs(p.survive)} · was ${rs(baseline)}`;
    const onclick = unlock
      ? async (): Promise<void> => {
          const id = unlockOf(app, p);
          if (!id || !(await app.buy(id))) return;   // refused (marks, gate): the row would carry a locked cond
          // the core's pseudo-patch names the set's own locked row (nothing to insert: the camp opens on it); a row the
          // set lacks is inserted at the top
          const have = app.rules.rows.findIndex((r) => sameRow(r, p.row));
          const i = have >= 0 ? have : app.insertRow(p.row, 0, "patch");
          app.go({ kind: "camp", highlight: i });
        }
      : held >= 0
        ? (): void => app.go({ kind: "camp", highlight: held })
        : full
          // QA 778fa1b (qaU friction: `apply` opened the DROP sheet though the patch said `drops R4`): the named drop applies as stated
          // (the patch was measured with it); without one, the sheet asks
          ? named >= 0 ? (): void => { const i = app.applyPatchOver(p, named); app.go({ kind: "camp", highlight: i }); } : (): void => openDropSheet(app, p, trace)
          : (): void => { const i = app.applyPatch(p); app.go({ kind: "camp", highlight: i }); };
    // QA 0c6e126 (qaY: `drink invisibility` applied and the next heir had none — `blocked · no item · none in pack`): a row whose item the
    // next heir will not carry comes with its purchase (`Patch.buys`, its reach measured with it bought): the tap buys it, then applies;
    // a refused buy (the purse moved since) applies nothing
    const act = p.buys && !unlock && held < 0
      ? async (): Promise<void> => { if (!(await app.mutate(() => app.engine.buySupply(p.buys!.kind)))) return; await onclick(); }
      : onclick;
    const buyTag = p.buys && !unlock && held < 0 ? h("small", { class: "num buy-tag gold" }, /* copy:callout */ ` · + ${p.buys.label} $${p.buys.price}`) : "";
    const label = h("span", { class: "chips-inline" }, target, unlock ? h("span", { class: "unlock-label" }, p.root?.text ?? rowLabel(p.row), " · ", h("b", null, /* copy:button */ "buy")) : opts.plain ? plainPatchRow(p.row) : rowLabel(p.row), dropTag, buyTag);
    // an unlock's second line is the row it inserts once bought; a root patch's is the chain's root it answers
    const root = unlock ? (p.root ? h("small", { class: "dim" }, rowLabel(p.row)) : "") : p.root ? h("small", { class: "root" }, "← ", p.root.text) : "";
    // QA 1a2a4a9 (O: "tapping a patch card applied it and jumped to camp; I meant to select it"): with `opts.select` (the death screen)
    // a tap lights the tablet and the gem applies the lit one — one model: tablets choose, the gem acts
    const btn: HTMLButtonElement = h("button", { class: `patch tablet${move ? " move" : ""}${p.remove ? " remove" : ""}${p.below_bar || held >= 0 || opts.nothingBeatsBase || noGain ? " below" : ""}${noGain ? " no-gain" : ""}${unlock ? " unlock" : ""}${held >= 0 ? " held" : ""}`,
      onclick: opts.select ? () => opts.select!(btn) : act, ...(full ? { "data-full": "1" } : {}), ...(p.buys ? { "data-buys": p.buys.kind } : {}) },
      h("b", { class: "rank num", "aria-hidden": "true" }), patchPlaque(p.row), h("span", { class: "patch-main" }, label, root),
      h("span", { class: "patch-nums" },
        // Cut 17 §4: `survives N %` as a gauge on the patch tablet (the number stays beside it)
        // blind b58b431 (A: `survives 9/12` beside `death 29→90%` — "which is it?"): the count names its horizon over the gauge (`this
        // fight`), as the whole run's move names its own (`per run`)
        unlock || held >= 0 ? "" : h("span", { class: "gauge-row" }, baseline !== undefined && !opts.stall ? kwHost(h("small", { class: "dim horizon h-fight" }, /* copy:label */ "this fight"), "h_fight") : "",
          h("span", { class: "gauge", "aria-hidden": "true" }, h("i", { style: `width:${Math.round(Math.max(0, Math.min(1, p.survive)) * 100)}%` }))),
        h("span", { class: "num surv" }, line),
        reachSpan(stallish && stallBase !== undefined ? { ...p, forecast_delta: delta / 100, forecast_pm: undefined } : p, stallish, campBaseAt(app)),
        stallish ? "" : wholeSpan(p)));   // Cut 27 §4: a stall's patches carry their whole-run move too (the gem's guard reads it)
    const nm = ruleName(names, pi);
    btn.dataset.short = unlock ? /* copy:button */ `buy ${p.root?.text ?? nm}` : move ? /* copy:button */ `move ${nm} up` : p.remove ? /* copy:button */ `cut ${nm}` : p.restores !== undefined ? /* copy:button */ `restore ${nm}` : p.buys ? `+ ${p.buys.label} · ${nm}` : `+ ${nm}`;
    btn.dataset.eff = unlock ? "" : held >= 0 ? /* copy:callout */ "already written" : noGain || opts.nothingBeatsBase ? /* copy:callout */ "no gain" : p.below_bar ? /* copy:callout */ "below bar"
      : opts.stall ? /* copy:callout */ `unstuck ${rs(p.survive)}` : baseline === undefined ? /* copy:callout */ `reach ${pct(p.survive)}` : /* copy:callout */ `survives ${rs(p.survive)}`;
    patchOf.set(btn, p);
    quietMoves(btn);
    applyOf.set(btn, act);
    return btn;
  });
  // Cut 20 §4 (AD: "`nothing beats base` … but lists three patches anyway"): under that header the candidates are no patches — dim,
  // folded behind `others` (a tap unfolds them; the gem stays `edit`)
  if (head) {
    const fold = h("div", { class: "patches-fold", hidden: true }, ...rows);
    const more: HTMLButtonElement = h("button", { class: "mini more patches-more game-control", onclick: () => { fold.hidden = false; more.remove(); } }, /* copy:button */ "others", h("small", { class: "num dim" }, ` ${rows.length}`));
    const box = h("div", { class: "patches none-beats" }, head, more, fold); renumber(fold); baseOf.set(box, campBaseAt(app)); return box;
  }
  // QA 0c6e126 (qaZ: three patches all `survives 100% · unpatched 83%` — "I could not tell which the gem ranks first or why"): when the
  // offered patches survive alike the head says so (`tied`): their order is then the reach's, once it lands, else the set's shape
  // QA 524827b (qaAA: `D8 death · replayed` over #1 and #3 both `survives 12/12`, `D4` over two `11/12`): `tied` whenever the lead's
  // survival is matched by another offered patch — the top two, not all three
  const offered = patches.filter((p) => !p.below_bar && p.insert_at >= 0);
  const sv = (p: Patch): number => opts.replays ? Math.round(p.survive * opts.replays) : Math.round(p.survive * 100);
  const best = offered.length ? Math.max(...offered.map(sv)) : -1;
  const tied = offered.filter((p) => sv(p) === best).length > 1;
  const moment = opts.moment !== undefined && baseline !== undefined && !opts.stall && rows.length
    ? h("div", { class: "patches-moment num dim" }, /* copy:label */ "fixes", " · ", opts.replays ? /* copy:callout */ `${opts.replays} replays of D${opts.moment}` : /* copy:callout */ `D${opts.moment} replayed`, tied ? h("span", { class: "tied" }) : "") : null;   // pass 2: the tablets read as his own rules (7/8) — they are fixes; `patches tie` read "no idea".   // QA 308f045 (qaAC: `GAP` over `replayed · tied`, read as "no gap"): what ties is the patches
  const box = h("div", { class: "patches" }, moment, ...rows); renumber(box); baseOf.set(box, campBaseAt(app)); return box;
}
/** QA e75ec29 (Q: "I read the gem as the best fix … rank by what is shown or show the ranking key"): the tablets carry their place
 *  (`1.` `2.` `3.`), renumbered whenever the order changes (the camp's reach landing re-ranks them). */
function renumber(host: HTMLElement): void {
  host.querySelectorAll<HTMLElement>(":scope > button.patch > .rank").forEach((r, i) => { r.textContent = `${i + 1}.`; });
}
const EXIT_VERBS = new Set(["return", "bank"]);
/** Each patch tablet's action (apply, buy, open the drop sheet, open the camp on a held row) — the gem's, when tablets only select. */
export const applyOf = new WeakMap<HTMLElement, () => void | Promise<void>>();
/** Each patch tablet's patch (the death screen's `why` reads the lead's). */
export const patchOf = new WeakMap<HTMLElement, Patch>();

/** A patch's reach line: `reach +8%`; with the camp's own measure (`deathDeltas`) the depth and its ± — `reach D6 +8% ±3`, and
 *  `reach D6 ~0` inside the ± (as an unlock card's); `reach …` while the camp's measure is pending (`camp_pending`: the verdict's
 *  12-sim estimate is not the camp's number — QA 23ed91f, K: "`reach +8%` … the shaft went D5 79% → 78%"). A stall patch's first
 *  line already says `reach`: its delta is bare (`+84%`). */
/** Cut 26 §6 (AP: `survives 12/12 · reach D5 −76` — "a number I could not read"): with the camp's reach at the patch's floor known,
 *  a move reads from→to (`reach D5 80→4%`), never a signed delta; `base` looks that reach up (0..1). */
type BaseAt = (depth: number) => number | undefined;
const baseOf = new WeakMap<HTMLElement, BaseAt>();
export const campBaseAt = (app: App): BaseAt => (d) => { const f = app.lastForecast; if (!f || d > f.known_to) return undefined; return f.depths.find((x) => x.depth === d)?.reach; };
function reachSpan(p: Patch, stallish = false, base?: BaseAt): HTMLElement {
  // QA a946e04 (S: `hp < 20% → return · reach D7 +0%` — "a return ends the run") hid an exit's reach; QA 778fa1b (qaU: `hp < 40% →
  // return · survives 100%` looked best, applied: `D5 −54 · death −94`): a patch whose row ends the run (`Patch.exits`, else its verb)
  // survives by going home — its cost is floors, so it says so, `return early`, beside the reach it costs (`reach D6 −49`)
  const exits = !stallish && !p.remove && (p.exits ?? (p.moves_from === undefined && EXIT_VERBS.has(p.row.verb.v)));   // Cut 25 §2: a move adds no exit unless the core says so
  // Cut 23 §3 (AI: `survives 92% … reach D5 −88` — "a number I could not read"): one form — an exit's point is surviving, so it loses its
  // number and says what it costs in a word (`return early`); every other patch reads its reach move (`reach D6 +8 ±3`)
  // QA 308f045 (qaAC: the gem lit `10/12` over `hp < 20% → return · survives 11/12`, `9/12` over a `12/12` return — "a visible reason the
  // lower one leads"): once the whole run is measured, an exit says the floors it costs beside its word (`return early · D6 56→20%`) —
  // why it survives more and still does not lead
  const w0 = p.whole, cost = exits && w0 && !p.camp_pending && w0.reach_from !== undefined && w0.reach_to !== undefined && w0.reach < 0 && Math.abs(w0.reach) > w0.reach_pm
    ? h("span", { class: "exit-cost down" }, /* copy:callout */ ` · reach D${w0.depth ?? p.forecast_depth ?? ""} ${Math.round(w0.reach_from * 100)}→${Math.round(w0.reach_to * 100)}%`) : "";
  if (exits) return h("span", { class: "num delta exit early" }, p.row.verb.v === "bank" ? /* copy:callout */ "secure early" : /* copy:callout */ "return early", cost);
  if (p.camp_pending) return h("span", { class: "num delta pending" }, /* copy:callout */ "reach …");
  const delta = Math.round(p.forecast_delta * 100);
  // QA 524827b: a move's ± is the paired one (`PatchWhole.reach_pm`, the camp's `vs sent` measure) once the whole run is measured
  const pmRaw = p.whole ? p.whole.reach_pm : p.forecast_pm;
  const pm = pmRaw !== undefined ? Math.max(1, Math.round(pmRaw * 100)) : undefined;
  const flat = delta === 0 || (pm !== undefined && Math.abs(delta) <= pm);
  const word = stallish ? "" : /* copy:label */ "reach ";
  const at = p.forecast_depth !== undefined ? `D${p.forecast_depth} ` : "";
  // docs/COPY.md pass 4 (`reach D6 78→26% ±14` — the ±14 read "no idea" by 4 of 6): the tablet's colour says a move is outside its noise
  const pmTag = "";
  // QA 92eb880 (M: "`reach D7 ~0` … the camp then shows D7 12%"): a move inside the ± reads as a move, never as a reach of ~0
  // Cut 22 §4: a move is signed points in the delta look (`reach D6 +8 ±3`), `≈` inside its ± — a move, never a reach level or a chance.
  // QA 778fa1b (qaU: `reach D6 ≈ ±14` — "a spread with no value"): `≈` is no call and stands alone; the ± rides only a move
  // Cut 24 §4: …but a ± the move sits inside is what makes it `≈` — it reads with it (`reach D6 ≈ ±5`): unresolved, not "no change"
  // the core's own pair once the whole run is measured (`PatchWhole.reach_from/_to`), else the camp bar's level plus the move
  const w = !stallish ? p.whole : undefined;
  const from = !stallish && p.forecast_depth !== undefined ? base?.(p.forecast_depth) : undefined;
  const fromTo = w?.reach_from !== undefined && w.reach_to !== undefined ? `${Math.round(w.reach_from * 100)}→${Math.round(w.reach_to * 100)}%`
    : from !== undefined ? (() => { const a = Math.round(from * 100); return `${a}→${Math.max(0, Math.min(100, a + delta))}%`; })() : undefined;
  // docs/COPY.md (pass 1: both blind readers took `reach D5 ≈ ±14` for "about D5, give or take 14"): a move inside its ± is `≈` alone
  return flat ? h("span", { class: "num delta flat" }, `${word}${at}`, /* copy:callout */ "same")
    : h("span", { class: `num delta ${delta > 0 ? "up" : "down"}` }, `${word}${at}${fromTo ?? `${delta > 0 ? "+" : "−"}${Math.abs(delta)}`}`, pmTag);
}

/** QA 524827b (qaAA: `hp < 20% → drink unknown · survives 12/12` led, and applied the camp's killers read fire 28 % · poison 26 %;
 *  `read unknown · 11/12` applied, `death +6`): beside the moment's count, the whole run's — the death share's paired move once the camp
 *  measured it (`death −8`, `death +6` in the loss colour; nothing inside its ±) and the gamble's own harm when it rises (`risk fire`).
 *  Empty (a placeholder `fillReach` fills) until then. */
function wholeSpan(p: Patch): HTMLElement {
  const w = p.whole;
  if (!w || p.camp_pending) return h("span", { class: "num whole pending" });
  const d = Math.round(w.death * 100), pm = Math.max(1, Math.round(w.death_pm * 100));
  const moved = d !== 0 && Math.abs(w.death) > w.death_pm + 1e-9;
  const parts: HTMLElement[] = [];
  // QA 308f045 (qaAC: `death −100 ±1` — "points? percent of runs?"): the death share before and after, as the reach reads (`death 100→0%`);
  // an older core without `death_from` keeps the signed move
  const from = w.death_from !== undefined ? Math.round(w.death_from * 100) : undefined;
  const deathText = from !== undefined ? `${from}→${Math.max(0, Math.min(100, Math.round((w.death_from! + w.death) * 100)))}%` : `${d > 0 ? "+" : "−"}${Math.abs(d)}`;
  if (moved) parts.push(h("span", { class: `dlt ${d > 0 ? "down" : "up"}` }, /* copy:callout */ `death ${deathText}`, from !== undefined ? "" : h("small", { class: "dim pm" }, /* copy:none */ ` ±${pm}`)));
  if (w.risk) parts.push(h("span", { class: "dlt down risk" }, /* copy:callout */ `risk ${w.risk}`));
  // blind 3ab97ea (A: `survives 11/12 · was 6/12 … death 56→89%` — "survives more, yet dies more?"): the count is this death's fight
  // replayed, the death move whole runs from the send — the move names its horizon (`per run`), and its tip says which is which
  const el = h("span", { class: `num whole${w.harms ? " harms" : ""}` }, ...(parts.length ? [kwHost(h("small", { class: "dim horizon" }, /* copy:label */ "per run"), "h_run"), " · "] : []), ...parts.flatMap((x, i) => (i ? [" · ", x] : [x])));
  if (parts.length) el.title = /* copy:tooltip */ "whole runs from the send · the count is this fight";
  return el;
}

/** QA 23ed91f: the camp's reach for a death's patches landed (`deathDeltas`, same order): each patch takes its numbers, and each
 *  tablet in `el` (patchRows' own, in order) repaints its reach line. */
export function fillReach(el: HTMLElement, patches: Patch[], filled: Patch[]): void {
  const host = el.querySelector<HTMLElement>(":scope > .patches-fold") ?? el;   // Cut 20 §4: the folded block's tablets
  const buttons = [...host.querySelectorAll<HTMLElement>(":scope > button.patch")];
  // QA 308f045 (qaAC: patch 1 read `survives 12/12 · reach D9 ≈ ±1`, applied, the camp read `D6 −21`): the core re-ranks the list on its
  // camp numbers (`death_deltas`: survival first, a harm sunk), so the landed list is matched patch by patch — by its row and its place —
  // never by index (by index, a tablet showed another patch's reach and the gem applied a patch whose numbers were not the ones shown)
  const keyOf = (p: Patch): string => JSON.stringify([p.row.conds.map((c) => [c.k, c.n ?? null, c.t ?? null]), p.row.verb.v, p.row.verb.a ?? null, p.insert_at, !!p.replace, !!p.remove, p.moves_from ?? null]);
  const byKey = new Map(filled.map((f) => [keyOf(f), f]));
  patches.forEach((p, i) => {
    const f = byKey.get(keyOf(p)); if (!f) return;
    Object.assign(p, { forecast_delta: f.forecast_delta, forecast_depth: f.forecast_depth, forecast_pm: f.forecast_pm, whole: f.whole, camp_pending: false, gem: f.gem });
    buttons[i]?.querySelector(".delta")?.replaceWith(reachSpan(p, false, baseOf.get(el)));
    buttons[i]?.querySelector(".whole")?.replaceWith(wholeSpan(p));
    buttons[i]?.classList.toggle("harms", !!p.whole?.harms);
    if (buttons[i]) quietMoves(buttons[i]);
  });
  el.dataset.reach = "camp";
  // QA 92eb880 (N): a loss once the camp's reach is in is dim (`.neg`) and says its move. QA 778fa1b (qaU: patch 1 lit with `100% apply`,
  // ~5 s later patch 2 lit and `67% apply`, no tap): the landing never moves the list or the lit tablet — the order is the core's
  // (`rank_patches`, which already keeps a costly exit off the lead), the player has read it by the time the camp's reach lands
  const pmOf = (p: Patch): number => p.forecast_pm !== undefined ? Math.max(1, Math.round(p.forecast_pm * 100)) : 0;
  // (an exit's reach is its price, `return early · D6 54→30%` — not a loss that dims it; the core keeps a costly exit off the lead)
  const moveOf = (p: Patch): number => { const d = Math.round(p.forecast_delta * 100); return p.insert_at < 0 || Math.abs(d) <= pmOf(p) || (!p.remove && (p.exits ?? (p.moves_from === undefined && EXIT_VERBS.has(p.row.verb.v)))) ? 0 : d; };
  // QA 524827b: a patch that harms whole runs (`PatchWhole.harms`: death up or reach down beyond its ±) is a loss too
  patches.forEach((p, i) => { const b = buttons[i]; if (b && !(b.classList.contains("below") && !b.classList.contains("neg"))) b.classList.toggle("neg", moveOf(p) < 0 || !!p.whole?.harms); });
}

/** Cut 28 §2 (AU: `survives 0/12 · no gain` beside a green `reach D14 0→24%` — "contradictory at a glance"): a tablet that reads `no gain`
 *  (or below the bar, or under `nothing beats unpatched`) never carries a move in the gain colour — its reach and death moves read neutral
 *  (the numbers stay). */
export function quietMoves(btn: HTMLElement): void {
  if (!btn.classList.contains("below")) return;
  btn.querySelectorAll<HTMLElement>(".delta.up, .whole .dlt.up").forEach((e) => { e.classList.remove("up"); e.classList.add("quiet"); });
}

/** QA 524827b (qaAA): once the whole run is measured, a tablet that harms it never leads — unless the player lit one, the tablets that
 *  harm go after the rest (in their order) and the block is renumbered. Returns whether the order changed. */
export function sinkHarms(el: HTMLElement, patches: Patch[]): boolean {
  const host = el.querySelector<HTMLElement>(":scope > .patches-fold") ?? el;
  const buttons = [...host.querySelectorAll<HTMLElement>(":scope > button.patch")];
  const bad = buttons.filter((_, i) => !!patches[i]?.whole?.harms);
  if (!bad.length || bad.length === buttons.length || buttons.slice(buttons.length - bad.length).every((b) => bad.includes(b))) return false;
  const good = buttons.filter((b) => !bad.includes(b));
  const order = [...good, ...bad];
  const moved = order.map((b) => patches[buttons.indexOf(b)]);
  patches.splice(0, patches.length, ...moved);
  for (const b of order) host.appendChild(b);
  renumber(host);
  return true;
}

/** Cut 27 §4 (AS: the stall gem pre-selected `cut R1` 1/12 above a 12/12 patch): once the whole run is measured, the tablets take the
 *  landed order (the core's rank on the camp's numbers, `deathDeltas`) and the tablet the gem lights goes first — the gem is always the
 *  first shown. `lead` is that tablet; the block is renumbered. */
export function leadFirst(el: HTMLElement, patches: Patch[], filled: Patch[], lead: HTMLElement | null): void {
  const host = el.querySelector<HTMLElement>(":scope > .patches-fold") ?? el;
  const buttons = [...host.querySelectorAll<HTMLElement>(":scope > button.patch")];
  if (buttons.length !== patches.length) return;
  const keyOf = (p: Patch): string => JSON.stringify([p.row.conds.map((c) => [c.k, c.n ?? null, c.t ?? null]), p.row.verb.v, p.row.verb.a ?? null, p.insert_at, !!p.replace, !!p.remove, p.moves_from ?? null]);
  const rank = new Map(filled.map((f, i) => [keyOf(f), i]));
  const idx = patches.map((_, i) => i);
  const sunk = (i: number): number => (patches[i]?.whole?.harms ? 1 : 0);   // (a tablet that harms whole runs stays after the rest: `sinkHarms`)
  idx.sort((a, b) => sunk(a) - sunk(b) || (rank.get(keyOf(patches[a])) ?? 1e6 + a) - (rank.get(keyOf(patches[b])) ?? 1e6 + b));
  const li = lead ? buttons.indexOf(lead) : -1;
  if (li >= 0) { idx.splice(idx.indexOf(li), 1); idx.unshift(li); }
  const order = idx.map((i) => buttons[i]), moved = idx.map((i) => patches[i]);
  patches.splice(0, patches.length, ...moved);
  const anchor = order[0]?.previousElementSibling ?? null;   // (a head or a moment line stays above the tablets)
  let prev: Element | null = anchor;
  for (const b of order) { if (prev) prev.after(b); else host.prepend(b); prev = b; }
  renumber(host);
}

/** Cut 15 §3: the drop sheet — the set's own rows (a card's row never; it sits outside `max_rows`), `R5 <row> · 0/16` with the
 *  run's fired count out of all fires when it is known (the watched run's rule events, or the absence's `R5 fired 0 of 16 runs`),
 *  the least-fired row marked (`.least`, `↓`) when its count is unique. A tap drops that row and inserts the patch at its measured
 *  place; the `×`, the backdrop or Escape closes the sheet and nothing changes. */
export function openDropSheet(app: App, p: Patch, trace?: Trace): void {
  const rows = app.rules.rows;
  const fires = app.rowFires, total = app.rowFiresOf ?? (fires ? fires.reduce((a, b) => a + b, 0) : 0);
  // QA on 3d71c33: the `↓` marks the least-fired row only when it is the only one at that count — among ties (nothing fired, two
  // rows at 0) no row is singled out (the lowest in the list was an arbitrary pick)
  // Cut 19 §4: the row the core named (`+ drop R5`) is the marked one
  const named = dropsOf(app, p);
  const least = named >= 0 ? named : uniqueLeast(app, trace);
  openSheet((close) => h("div", { class: "sheet-body drop-sheet" },
    h("div", { class: "label row-label" }, /* copy:label */ "drop", " ", h("small", { class: "dim" }, rowLabel(p.row)), closeX(close)),
    ...rows.map((r, i) => !ownRow(r) ? null : h("button", {
      class: `drop-row${i === least ? " least" : ""}`, "data-row": String(i),
      onclick: () => { close(); const at = app.applyPatchOver(p, i); app.go({ kind: "camp", highlight: at }); },
    },
    h("span", { class: "chips-inline" }, rowLabel(r)),
    fires ? h("span", { class: "num fired dim" }, /* copy:callout */ ` · ${fires[i] ?? 0}/${total} fires`) : "",   // QA 778fa1b (qaU: `· 15/16` with no unit)
    i === least ? h("span", { class: "num mark" }, "↓") : ""))));
}
