// The trace as a table (hero actions, t = tick) with the row accounting of the last action under it
// (Cut 6 §3: `R1 none held · R2 no path`, engine data). Cut 9 §5: shared by the death screen, the exit sheet's
// `trace` chip and the report's exit lines — every exit has one now, not only a death.
// Cut 11 §2–3: when the wire carries a `because`, the accounting is the chain (ui/chain.ts: `R1 drink heal · no item ←
// den took the heal, D3 [watch]`); exit traces show their last 10 turns, the death screen keeps 5 above its chain.
import type { Trace } from "../engine/types";
import { chainOf, rowRef, type ChainCtx } from "./chain";
import { lastRun } from "./runlog";
import { h } from "./dom";
import { openSheet } from "./sheet";
import { refName, verbLabel } from "./tokens";

export const TRACE_ROWS = 5;
export const EXIT_TRACE_ROWS = 10;

/** The table and, under it, the chain (Cut 11) or the dim `R1 why · R2 why` line when the trace carries no `because`. */
export function traceTable(trace: Trace, ctx: ChainCtx = {}, rows = TRACE_ROWS): HTMLElement[] {
  const turns = trace.turns.slice(-rows);
  // QA 23ed91f (K: "the `tele` column is empty on every row"): the column only when a shown turn has a telegraph
  const tele = turns.some((t) => t.telegraphs.length > 0);
  // Cut 28 §2 (AV: `R1 hp not <30%` at 6 hp — the drained max not shown): once the hero's max moved in the run, every hp reads over the max
  // it had then (`6/29`) and each step is a row of its own where it moved (`max 44→29 · drain`)
  const mx = maxHpOf(trace, ctx.runId);
  const hpCell = (hp: number, t: number): string => { const m = mx?.at(t); return m !== undefined ? `${hp}/${m}` : `${hp}`; };
  const blowRow = (b: { t: number; by: string; dmg: number; hp: number }, foes?: number): HTMLElement => h("tr", { class: "blow" },
    h("td", null, `${b.t}`),
    h("td", { class: "r" }, b.by.replace(/_/g, " "), " ", h("small", { class: "dim" }, `−${b.dmg}`)),
    h("td", null, hpCell(b.hp, b.t)),
    h("td", null, `${foes ?? ""}`),
    tele ? h("td", { class: "tele" }) : "");
  const stepRow = (s: MaxStep): HTMLElement => h("tr", { class: "max-step" },
    h("td", null, `${s.t}`),
    h("td", { class: "r" }, /* copy:callout */ `max ${s.max - s.delta}→${s.max}`, " ", h("small", { class: "dim" }, s.cause.replace(/_/g, " "))),
    h("td", { class: s.delta < 0 ? "down" : "up" }, `${s.delta > 0 ? "+" : "−"}${Math.abs(s.delta)}`),
    h("td", null, ""),
    tele ? h("td", { class: "tele" }) : "");
  // the steps between two shown rows (by tick), before the later one; the window's own (none before its first turn)
  const t0 = turns[0]?.t ?? Infinity;
  const stepsAt = (from: number, to: number): (readonly [number, HTMLElement])[] => (mx?.steps ?? []).filter((s) => s.t > from && s.t <= to && s.t >= t0).map((s) => [s.t, stepRow(s)] as const);
  const table = h("table", { class: `trace num${ctx.home ? " home" : ""}` },
    h("thead", null, h("tr", null, /* copy:label */ ...["t", "rule", "hp", "foes", ...(tele ? ["tele"] : [])].map((s) => h("th", null, s)))),
    // QA 524827b (qaAB: `8002 R1 return 12 · 8013 R1 return 6` — hp fell with no row): the blows between two shown actions are rows of
    // their own before the second (engine data: `TraceTurn.blows`), so every hp step reads
    h("tbody", null, ...turns.flatMap((t, i) => [...(i > 0 ? mergeByT((t.blows ?? []).map((b) => [b.t, blowRow(b)] as const), stepsAt(turns[i - 1].t, t.t)) : []), h("tr", null,
      h("td", null, `${t.t}`),
      // QA 912e135: a player row's turn names the row as a button when the screen can open it (`ChainCtx.onRow`)
      h("td", { class: "r" }, ...(t.row >= 0 && ctx.onRow ? [rowRef(t.row, verbLabel(t.verb), ctx.onRow)]
        : [t.row >= 0 ? refName(t.row) : t.row === -1 ? h("span", null, /* copy:label */ "trait", " ", h("small", { class: "dim" }, verbLabel(t.verb))) : h("span", null, "· ", h("small", { class: "dim" }, verbLabel(t.verb)))])),
      h("td", null, hpCell(t.hp, t.t)),
      h("td", null, `${t.foes}`),
      tele ? h("td", { class: "tele" }, t.telegraphs.join(" · ")) : "",
    )]),
    // QA 0c6e126 (qaY: "the last row is never the killing blow — 05 ends at `hp 1`"): a death's table ends on the blow that killed —
    // its tick, what hit and for how much, `hp 0` (engine data: `Trace.blow`)
    // Cut 25 §6 (AN: `14 → 0` on one `goblin −2` row): every blow after the last action, one row each, hp after each (`Trace.blows`,
    // the last is `blow`); an older core's lone `blow`
    ...mergeByT((trace.blows?.length ? trace.blows : trace.blow ? [trace.blow] : []).map((b) => [b.t, blowRow(b, turns[turns.length - 1]?.foes)] as const), stepsAt(turns[turns.length - 1]?.t ?? -Infinity, Infinity))));
  // QA 524827b (qaAA: the table opens at 1 hp exploring — "where 36 hp went takes a replay"): over it, where the hp went since he was
  // last at full hp (engine data: `Trace.hp_lost`, most first; the few largest, the rest counted)
  const lost = trace.hp_lost ?? [];
  // QA 308f045 (qaAC: killer `goblin conjurer −2`, the loss line `goblin −22 · jackal −15 · … · others −2` never naming him): the killing
  // blow's cause is always one of the named, in its place by size (it takes the last named slot when it would fall under `others`)
  const HP_LOST_SHOW = 4, killer = (trace.blows?.length ? trace.blows[trace.blows.length - 1] : trace.blow)?.by;
  let named = lost.slice(0, HP_LOST_SHOW);
  const k = killer ? lost.find((x) => x.by === killer) : undefined;
  if (k && !named.includes(k)) named = [...named.slice(0, HP_LOST_SHOW - 1), k];
  const rest = lost.filter((x) => !named.includes(x)).reduce((a, x) => a + x.dmg, 0);
  // QA 308f045 (qaAC: `since full hp` summing to 72 on a 36-hp hero — "the heals are part of it; nothing says so"): what was healed over
  // the stretch reads beside it (`healed +36`, the core's `Trace.hp_healed`)
  const healed = trace.hp_healed ?? 0;
  const lostLine = lost.length ? h("div", { class: "hp-lost num dim" }, /* copy:callout */ "since full hp",
    ...named.map((x) => h("span", { class: `hp-by${x.by === killer ? " killer" : ""}` }, ` · ${x.by.replace(/_/g, " ")} `, h("b", { class: "down" }, `−${x.dmg}`))),
    rest > 0 ? h("span", { class: "hp-by" }, /* copy:callout */ ` · others −${rest}`) : "",
    healed > 0 ? h("span", { class: "hp-by healed" }, /* copy:callout */ ` · healed `, h("b", { class: "up" }, `+${healed}`)) : "") : null;
  // Cut 28 §2: the run's max-hp arc over the table (`max hp 44→29 · hunger −12 · drain −3`)
  const maxLine = mx && mx.from !== mx.to ? h("div", { class: "hp-max num dim" }, /* copy:callout */ `max hp ${mx.from}→${mx.to}`,
    ...mx.causes.map((c) => h("span", { class: "hp-by", ...(/^(hunger|starv)/.test(c.cause) ? { "data-answer": "light", title: /* copy:tooltip */ "unlit floor · shrine or lantern stops it" } : {}) }, ` · ${c.cause.replace(/_/g, " ")} `, h("b", { class: c.delta < 0 ? "down" : "up" }, `${c.delta > 0 ? "+" : "−"}${Math.abs(c.delta)}`)))) : null;
  const heads: HTMLElement[] = [];
  if (maxLine) heads.push(maxLine);
  if (lostLine) heads.push(lostLine);
  const chain = chainOf(trace, { window: rows, ...ctx, ...(mx ? { hpAt: (t: number, hp: number) => hpCell(hp, t) } : {}) });
  if (chain) return [...heads, table, chain];
  return [...heads, table, ...traceTableRest(turns, { ...ctx, ...(mx ? { hpAt: (t: number, hp: number) => hpCell(hp, t) } : {}) })];
}
/** Cut 28 §2 — a max-hp step: the tick, the max after it, the move, the cause (`drain` · `hunger` · `curse` · `shrine`). */
export type MaxStep = { t: number; max: number; delta: number; cause: string };
type TurnMax = Trace["turns"][number] & { max_hp?: number };
type TraceMax = Trace & { max_steps?: MaxStep[] };
/** Cut 28 §2: the hero's max hp over a trace — the core's steps (`Trace.max_steps`) or per-turn max (`TraceTurn.max_hp`), else the
 *  watched run's own `max_hp` events (this session's log of that run); undefined when the max never moved in the run. `at(t)` is the
 *  max he had at tick `t`; `causes` sums the moves per cause, most first. */
export function maxHpOf(trace: Trace, runId?: number): { at: (t: number) => number | undefined; steps: MaxStep[]; from: number; to: number; causes: { cause: string; delta: number }[] } | undefined {
  const tr = trace as TraceMax;
  let steps: MaxStep[] = tr.max_steps ?? [];
  if (!steps.length) {
    const log = lastRun();
    if (log && runId !== undefined && log.runId === runId) {
      const hero = log.floors[0]?.snap.hero.id;
      for (const f of log.floors) for (const e of f.evs) if (e.k === "max_hp" && e.id === (hero ?? e.id) && e.delta) steps.push({ t: e.t, max: e.max, delta: e.delta, cause: e.cause });
    }
  }
  steps = [...steps].sort((a, b) => a.t - b.t);
  const turns = tr.turns as TurnMax[];
  const perTurn = turns.filter((t) => typeof t.max_hp === "number");
  if (!steps.length && perTurn.length && perTurn.some((t) => t.max_hp !== perTurn[0].max_hp)) {
    // (per-turn maxima only: a step where two neighbours differ)
    for (let i = 1; i < perTurn.length; i++) if (perTurn[i].max_hp !== perTurn[i - 1].max_hp) steps.push({ t: perTurn[i].t, max: perTurn[i].max_hp!, delta: perTurn[i].max_hp! - perTurn[i - 1].max_hp!, cause: "" });
  }
  if (!steps.length) return undefined;
  const from = steps[0].max - steps[0].delta, to = steps[steps.length - 1].max;
  const at = (t: number): number | undefined => {
    const own = turns.find((x) => x.t === t)?.max_hp; if (typeof own === "number") return own;
    let m = from; for (const s of steps) { if (s.t <= t) m = s.max; else break; } return m;
  };
  const by = new Map<string, number>();
  for (const s of steps) if (s.cause) by.set(s.cause, (by.get(s.cause) ?? 0) + s.delta);
  const causes = [...by].map(([cause, delta]) => ({ cause, delta })).filter((c) => c.delta).sort((a, b) => Math.abs(b.delta) - Math.abs(a.delta)).slice(0, 3);
  return { at, steps, from, to, causes };
}
/** Rows of a table section in tick order (a max step and a blow on one tick: the step first). */
function mergeByT(a: readonly (readonly [number, HTMLElement])[], b: readonly (readonly [number, HTMLElement])[]): HTMLElement[] {
  return [...b.map((x) => [x[0] - 0.5, x[1]] as const), ...a].sort((x, y) => x[0] - y[0]).map((x) => x[1]);
}
/** The last turn's row reasons as one line (an older core's chain), or nothing. */
function traceTableRest(turns: Trace["turns"], ctx: ChainCtx): HTMLElement[] {
  const lastRows = turns[turns.length - 1]?.rows ?? [];
  // each `R2 foes appeared after` whole on its line (the list wraps between reasons, never inside one)
  const last = turns[turns.length - 1];
  const hpAt = (why: string): string => ctx.hpAt && last && /\bhp\b/.test(why) ? ` ${ctx.hpAt(last.t, last.hp)}` : "";
  const rowsLine = lastRows.length ? h("div", { class: "rows-line num dim" }, ...lastRows.flatMap((r, i) => [i ? " · " : "", ctx.onRow ? h("span", { class: "rw" }, rowRef(r.row, undefined, ctx.onRow), ` ${r.why}${hpAt(r.why)}`) : h("span", { class: "rw" }, `${refName(r.row)} ${r.why}${hpAt(r.why)}`)])) : null;
  return rowsLine ? [rowsLine] : [];
}
/** Cut 9 §5: a `trace` chip; tapping it opens the table (Cut 11 §3: the last 10 turns and the chain) in a sheet.
 *  `null` when the exit carries no trace. `head` is the sheet's header line under the label — the exit's ledger line, engine
 *  data verbatim (QA on 50bb162: "TRACE sheet from the ledger: no header"). */
export function traceChip(trace: Trace | undefined, cls = "chip mini", ctx: ChainCtx = {}, head?: string, label?: string): HTMLElement | null {
  if (!trace?.turns.length) return null;
  // Cut 14 §4: `label` names the chip's exit (`D5 · died · trace`, ui/report.ts); the plain chip stays `trace`
  const home = ctx.home ?? (!!head && /^(banked|returned|driven)\b/.test(head));   // the exit line's first word is the tier (engine data)
  return h("button", { class: cls, onclick: () => openSheet(() => h("div", { class: "sheet-body trace-sheet" },
    h("div", { class: "label row-label" }, /* copy:label */ "decision log"),
    head ? h("div", { class: "trace-head ledger-line num dim" }, head) : null,
    ...traceTable(trace, { ...ctx, provenance: true, home }, EXIT_TRACE_ROWS))) }, label ?? /* copy:button */ "decision log");
}
