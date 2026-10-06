// Cut 29 §3: the diagnostics meters — one module for every surface that shows them (the watch's toggle, the death
// screen's fight, the report's night, the camp's two-run comparison, the desktop's right column). The core meters the
// event stream (`meters.rs`); this file only merges and draws. Units always (`12 dps`, `3 hp/s`, `$40/min`); a rule is
// named by its words (`tokens.ruleName`), never its index.
import type { MeterSides, MeterWire, Row, SnapMeters } from "../engine/types";
import { h } from "./dom";
import { ruleName } from "./tokens";

const r3 = (x: number): number => Math.round(x * 1000) / 1000;
const addSides = (a: MeterSides, b: MeterSides): MeterSides => ({ hero: a.hero + b.hero, pets: a.pets + b.pets, foes: a.foes + b.foes });
const rate = (x: MeterSides, s: number): MeterSides => ({ hero: r3(x.hero / s), pets: r3(x.pets / s), foes: r3(x.foes / s) });

/** Two meters as one (slices of one absence): totals add, rates are recomputed from the summed totals and seconds the way
 *  the core's `meters::wire` derives them (per game second, floor 0.1 s; a row's share of all rule activations). */
export function mergeMeters(a?: MeterWire, b?: MeterWire): MeterWire | undefined {
  if (!a) return b;
  if (!b) return a;
  const seconds = r3(a.seconds + b.seconds), s = Math.max(0.1, seconds);
  const dealt = addSides(a.dealt, b.dealt), taken = addSides(a.taken, b.taken);
  const heal = new Map<string, number>();
  for (const h of [...a.healed, ...b.healed]) heal.set(h.src, (heal.get(h.src) ?? 0) + h.total);
  const healed = [...heal].map(([src, total]) => ({ src, total, per_s: r3(total / s) }));
  const rows = new Map<number, number>();
  for (const r of [...a.rows, ...b.rows]) rows.set(r.row, (rows.get(r.row) ?? 0) + r.fires);
  const actions = a.actions + b.actions;
  const firesTotal = Math.max(1, [...rows.values()].reduce((sum, n) => sum + n, 0));
  const supplies: { [k: string]: number } = { ...a.supplies };
  for (const [k, n] of Object.entries(b.supplies)) supplies[k] = (supplies[k] ?? 0) + n;
  const gold = a.gold + b.gold;
  const T = (k: "fight" | "travel" | "chores" | "rest"): number => a.time[k] + b.time[k];
  const TS = (k: "fight" | "travel" | "chores" | "rest"): number => r3(a.time_s[k] + b.time_s[k]);
  return {
    seconds, dealt, taken, dps_dealt: rate(dealt, s), dps_taken: rate(taken, s),
    healed, hps: r3(healed.reduce((x, h) => x + h.total, 0) / s),
    time: { fight: T("fight"), travel: T("travel"), chores: T("chores"), rest: T("rest") },
    time_s: { fight: TS("fight"), travel: TS("travel"), chores: TS("chores"), rest: TS("rest") },
    rows: [...rows].sort((x, y) => x[0] - y[0]).map(([row, fires]) => ({ row, fires, share: r3(fires / firesTotal) })),
    actions, supplies, gold, gold_per_min: r3(gold / (s / 60)),
    hits_hero: a.hits_hero + b.hits_hero, hits_pets: a.hits_pets + b.hits_pets, fights: a.fights + b.fights,
  };
}

/** A rate as it reads: `12`, `3.4`, `0.25` — never more than 3 significant figures. */
export const rate1 = (x: number): string => (x >= 100 ? `${Math.round(x)}` : x >= 10 ? x.toFixed(1).replace(/\.0$/, "") : x >= 1 ? x.toFixed(1).replace(/\.0$/, "") : x > 0 ? x.toFixed(2).replace(/0$/, "") : "0");
/** Seconds as they read: `40s`, `3.5m`, `1.2h`. */
export const secs = (s: number): string => (s >= 3600 ? `${(s / 3600).toFixed(1).replace(/\.0$/, "")}h` : s >= 60 ? `${(s / 60).toFixed(1).replace(/\.0$/, "")}m` : `${Math.round(s)}s`);
/** A row's name in a meter: the rule's words (`attack nearest`), `trait` for a trait's or a card's own step, `chores` for the chores. */
export function rowOf(row: number, rows: Row[]): string {
  if (row === -1) return /* copy:label */ "trait";
  if (row === -2) return /* copy:label */ "chores";
  return rows[row] ? ruleName(rows, row) : /* copy:callout */ "an old rule";
}

/** The watch's compact meter — one line: the fight in progress (else the run so far), what the hero (and the pets) deal, take and heal
 *  per second. */
export function compactLine(sm: SnapMeters | undefined): HTMLElement {
  const el = h("div", { class: "meter-line num" });
  if (!sm) return el;
  const m = sm.fighting && sm.fight ? sm.fight : sm.run;
  const pets = m.dps_dealt.pets > 0;
  el.dataset.scope = sm.fighting && sm.fight ? "fight" : "run";
  el.append(
    h("span", { class: "m-scope dim" }, sm.fighting && sm.fight ? /* copy:label */ "fight" : /* copy:label */ "run"), " ",
    h("span", { class: "m dealt" }, /* copy:label */ "dealt ", h("b", null, rate1(m.dps_dealt.hero + m.dps_dealt.pets)), " dps", pets ? h("small", { class: "dim" }, ` · ${/* copy:label */ "pets"} ${rate1(m.dps_dealt.pets)}`) : ""), " ",
    h("span", { class: "m taken" }, /* copy:label */ "taken ", h("b", null, rate1(m.dps_taken.hero)), " dps"), " ",
    m.hps > 0 ? h("span", { class: "m healed" }, /* copy:label */ "healed ", h("b", null, rate1(m.hps)), " hp/s") : "");
  return el;
}

/** The full breakdown of one meter (a fight, a run, a night): damage by side, healing by source, the time split, the rules' shares, the
 *  supplies used, the gold, the blows taken. `rows` names the rules (the set the meter ran). */
export function meterPanel(m: MeterWire, rows: Row[], o: { title?: string; cls?: string } = {}): HTMLElement {
  const line = (label: string, ...v: (HTMLElement | string)[]): HTMLElement => h("div", { class: "mrow num" }, h("span", { class: "mlab" }, label), " ", h("span", { class: "mval" }, ...v));
  const pets = m.dealt.pets > 0 || m.taken.pets > 0;
  const T = m.time_s, tot = Math.max(0.001, T.fight + T.travel + T.chores + T.rest);
  const split = h("div", { class: "msplit", role: "img", "aria-label": `fight ${secs(T.fight)} · travel ${secs(T.travel)} · chores ${secs(T.chores)} · rest ${secs(T.rest)}` },
    ...(["fight", "travel", "chores", "rest"] as const).filter((k) => T[k] > 0).map((k) => h("span", { class: `seg ${k}`, style: `flex-grow:${T[k] / tot}`, title: `${k} ${secs(T[k])}` })));
  const splitKey = h("div", { class: "msplit-key num" }, ...(["fight", "travel", "chores", "rest"] as const).filter((k) => T[k] > 0)
    .map((k) => h("span", { class: `k ${k}` }, h("i", { class: "sw", "aria-hidden": "true" }), `${k} `, h("b", null, `${Math.round((T[k] / tot) * 100)}%`), " ")));
  const top = [...m.rows].sort((a, b) => b.fires - a.fires).slice(0, 3);
  const sup = Object.entries(m.supplies).filter(([, n]) => n > 0).sort((a, b) => b[1] - a[1]);
  return h("section", { class: `meters${o.cls ? ` ${o.cls}` : ""}`, "data-seconds": m.seconds },
    o.title ? h("div", { class: "label mhead" }, o.title, h("small", { class: "dim num" }, ` · ${secs(m.seconds)}`)) : "",
    line(/* copy:label */ "dealt", h("b", null, rate1(m.dps_dealt.hero)), " dps", pets ? h("small", { class: "dim" }, ` · ${/* copy:label */ "pets"} ${rate1(m.dps_dealt.pets)} dps`) : "", h("small", { class: "dim" }, ` · ${m.dealt.hero + m.dealt.pets} hp`)),
    line(/* copy:label */ "taken", h("b", null, rate1(m.dps_taken.hero)), " dps", pets ? h("small", { class: "dim" }, ` · ${/* copy:label */ "pets"} ${rate1(m.dps_taken.pets)} dps`) : "", h("small", { class: "dim" }, ` · ${m.taken.hero} hp · ${m.hits_hero} hits`)),
    m.hps > 0 ? line(/* copy:label */ "healed", h("b", null, rate1(m.hps)), " hp/s", h("small", { class: "dim" }, ` · ${[...m.healed].sort((a, b) => b.total - a.total).map((x) => `${x.src} ${x.total} hp`).join(" · ")}`)) : "",
    tot > 0.001 ? h("div", { class: "mrow mtime" }, h("span", { class: "mlab" }, /* copy:label */ "time"), " ", h("span", { class: "mval" }, split, splitKey)) : "",
    top.length ? line(/* copy:label */ "rules", ...top.flatMap((r, i) => [i ? h("i", { class: "sep dim" }, " · ") : "", h("span", { class: "mrule", "data-row": r.row, title: `${r.fires} rule activations · ${Math.round(r.share * 100)}% of recorded activations` }, rowOf(r.row, rows), " ", h("b", null, `${Math.round(r.share * 100)}%`))])) : "",
    sup.length ? line(/* copy:label */ "used", sup.map(([k, n]) => `${k.replace(/_/g, " ")} ×${n}`).join(" · ")) : "",
    m.gold > 0 ? line(/* copy:label */ "gold", h("b", null, `$${rate1(m.gold_per_min)}`), "/min", h("small", { class: "dim" }, ` · $${m.gold}`)) : "",
  );
}

/** The camp's two-run comparison: the last two runs side by side (older first), each figure with the newer's move. */
export function meterCompare(a: MeterWire, b: MeterWire): HTMLElement {
  const fig = (label: string, x: number, y: number, unit: string, worseUp = false): HTMLElement => {
    const d = y - x, dir = Math.abs(d) < 0.05 * Math.max(1, Math.abs(x)) ? "flat" : (d > 0) !== worseUp ? "up" : "down";
    return h("div", { class: "mcmp-row num" }, h("span", { class: "mlab" }, label), " ", h("span", { class: "was dim" }, `${rate1(x)}`), " ", h("span", { class: "arrow dim" }, "→"), " ",
      h("b", { class: "now" }, `${rate1(y)}`), h("small", { class: "unit dim" }, ` ${unit}`), " ", h("span", { class: `dlt ${dir}` }, dir === "flat" ? "" : `${d > 0 ? "+" : "−"}${rate1(Math.abs(d))}`));
  };
  const fightShare = (m: MeterWire): number => { const t = m.time_s; const s = t.fight + t.travel + t.chores + t.rest; return s > 0 ? (t.fight / s) * 100 : 0; };
  return h("section", { class: "meters mcmp" },
    h("div", { class: "label mhead" }, /* copy:label */ "last runs"),
    fig(/* copy:label */ "dealt", a.dps_dealt.hero + a.dps_dealt.pets, b.dps_dealt.hero + b.dps_dealt.pets, "dps"),
    fig(/* copy:label */ "taken", a.dps_taken.hero, b.dps_taken.hero, "dps", true),
    fig(/* copy:label */ "healed", a.hps, b.hps, "hp/s"),
    fig(/* copy:label */ "fighting", fightShare(a), fightShare(b), "%"),
    fig(/* copy:label */ "gold", a.gold_per_min, b.gold_per_min, "$/min"));
}
