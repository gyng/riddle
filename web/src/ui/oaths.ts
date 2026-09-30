// Cut 28 §1 — oaths: goals the player picks. The camp's oath board is a carved sheet of three standing oaths (the lineage's, drawn by
// the core from a pool): each a constraint read as chips (`D10` `no drink`), a reward that is never a stat (a card, a party slot, a
// title, a waystone, a verb — its icon and word), and the gold it costs to swear (refunded into its reward when kept). One oath is sworn
// at a time: it rides the shaft with the forecast's share of keeping it (`⚜ D10 no drink · 34%`), and the report leads with its
// progress. The board is carved by the reveal ladder when the purse first affords an oath (`oaths` step), never before.
//
// Cut 28b (owner: "it's not clear what oaths do, especially to new players") — the deal reads as a formula, everywhere the oath is: its
// constraint chips, an arrow, its reward (`[D3] [no rest] → ▤ card: gas step`), the way a rule reads `cond → verb`. The price is a stake
// (`stake $280`); once sworn the stake's fate is on the tablet (`$280 · 0/1`: staked, kept 0 of the 1 send it needs) and forswearing says what
// comes back (`forswear +$140`). The run teaches the rest by consequence: the watch beats `OATH KEPT` / `OATH BROKEN · R2 return`, the exit
// line and the report say the same, the chronicle keeps it. The ladder carves the board when it has something to answer — the lineage's first
// band boss seen or first plateau (`Lineage.oath_open`) — not when the purse first covers a price.
//
// Wire (core, optional — every field read through the accessors below so the client runs on a core without oaths):
//   Lineage.oaths: Oath[]           the board (≤ 3), the sworn one flagged `sworn`
//   Forecast.oath: OathShare        the sworn oath's share of the sims that keep it (with ±)
//   ReturnReport.oath: OathReport   the sworn oath's night: kept or not, its progress words, done → the reward
//   engine.swearOath(id) / engine.forswearOath()
import { conceptCap } from "./concepts";
import type { App } from "../app";
import type { Forecast, Lineage, Oath, OathReport, OathReward, OathShare, ReturnReport, Row } from "../engine/types";
import { ruleName } from "./tokens";
import { h, replace, twoTap } from "./dom";
import { icon } from "./skin";
import { openSheet } from "./sheet";
import { moveOf, pmShown, share, lowOf } from "./forecast";

export const oathsOf = (L: Lineage | null | undefined): Oath[] => (L?.oaths ?? []).slice(0, 3);
export const swornOf = (L: Lineage | null | undefined): Oath | undefined => {
  const os = oathsOf(L), id = L?.oath;
  return os.find((o) => o.sworn) ?? (id ? os.find((o) => o.id === id) : undefined);
};
export const oathShareOf = (f: Forecast | null | undefined, o: Oath | undefined): OathShare | undefined => {
  const s = f?.oath;
  return s && o && (!s.id || s.id === o.id) ? s : undefined;
};
/** Cut 29 §1: every sworn oath (the extra slots' too): the core's `sworn` ids, else the board's flags, else the one `oath`. */
export const swornAll = (L: Lineage | null | undefined): Oath[] => {
  const os = oathsOf(L), ids = L?.sworn;
  if (ids?.length) return ids.map((id) => os.find((o) => o.id === id)).filter((o): o is Oath => !!o);
  const flagged = os.filter((o) => o.sworn); if (flagged.length) return flagged;
  const one = swornOf(L); return one ? [one] : [];
};
/** Cut 29 §1: oaths that may stand sworn at once (1–3). */
export const slotsOf = (L: Lineage | null | undefined): number => Math.max(1, L?.oath_slots ?? 1);
export const oathReportOf = (r: ReturnReport): OathReport | undefined => r.oath;
/** Cut 28b: the board is carved at the lineage's first wall or plateau (the core's `oath_open`), or once one is sworn or kept. */
export const oathsEarned = (L: Lineage): boolean => oathsOf(L).length > 0 && (!!L.oath_open || oathsOf(L).some((o) => o.sworn) || !!L.oath || (L.titles?.length ?? 0) > 0);

/** An oath's constraint as chips: the wire's `chips`, else its text split at ` · `. */
export const chipsOf = (o: Pick<Oath, "chips" | "text"> & { kind?: string }): string[] => {
  const cs = (o.chips?.length ? o.chips : (o.text ?? "").split(/\s*·\s*/)).map((c) => c.replace(/_/g, " ").trim()).filter(Boolean);
  // docs/COPY.md pass 5 (`[Warlord] [fire]` read "no idea"): the fire oath is a kill — `slay Warlord` `with fire`
  return o.kind === "fire" && cs.length === 2 && cs[1] === "fire" ? [/* copy:rule_token */ `slay ${cs[0]}`, /* copy:rule_token */ "with fire"] : cs;
};
const rewardOf = (r: OathReward | string | undefined): { kind: string; label: string } | undefined => (typeof r === "string" ? { kind: r.split(/[\s:]/)[0], label: r } : r);
/** Each reward kind's icon (the console's own sprites) and glyph fallback. */
const REWARD_ICON: Record<string, [string, string]> = { card: ["unlocks", "▤"], slot: ["party", "◯"], party: ["party", "◯"], title: ["renown", "★"], trophy: ["renown", "★"],
  waystone: ["depth", "▼"], verb: ["edit", "✎"], row: ["edit", "✎"], route: ["depth", "⑂"], heir: ["renown", "♛"] };   // Cut 29 §1: route2, heir pick
export function rewardEl(r: OathReward | string | undefined): HTMLElement | "" {
  const w = rewardOf(r); if (!w) return "";
  const [ico, glyph] = REWARD_ICON[w.kind] ?? ["renown", "★"];
  // docs/COPY.md: a card is a `tactic` everywhere (a ready-made rule)
  return h("span", { class: "oath-reward", "data-kind": w.kind }, icon(ico, glyph), h("span", { class: "rw-l" }, w.label.replace(/_/g, " ").replace(/^card:/, /* copy:callout */ "rule:").replace(/^verb:/, /* copy:callout */ "action:")));
}
const chipEls = (cs: string[]): HTMLElement[] => cs.map((c) => h("span", { class: `chip oath-c${/\bD\d+/.test(c) ? " depth" : ""}` }, c));
/** Cut 28b (AW: "the Mother oath sat at 9% ±8 with no lever I could find"): the sworn oath's steps on the panel (`D13 40% · met 34% · burned 6%`),
 *  so a lever that moves a step reads while the kept share sits in its noise. */
function stepsEl(app: App, s: OathShare | undefined): HTMLElement | "" {
  if (!s?.steps?.length) return "";
  const low = lowOf(app.lastForecast);
  return h("div", { class: "oath-steps num" }, ...s.steps.flatMap((x, i) => [i ? h("span", { class: "dim" }, " · ") : "", h("span", { class: "oath-step", "data-k": x.k }, `${x.k} `, h("b", {}, share(x.share, low)))]));
}
/** Cut 28b: the reward's icon alone (the shaft, the tablet): the formula's right side where a word will not fit. */
function rewardIcon(r: OathReward | string | undefined): HTMLElement | "" {
  const w = rewardOf(r); if (!w) return "";
  const [ico, glyph] = REWARD_ICON[w.kind] ?? ["renown", "★"];
  return h("span", { class: "oath-reward icon-only", "data-kind": w.kind, title: w.label.replace(/_/g, " ") }, icon(ico, glyph));
}
/** Cut 28b: the deal as a formula — constraint chips `→` reward — the way a rule tablet reads `cond → verb`. */
export function formula(cs: string[], reward: HTMLElement | "", cls = ""): HTMLElement {
  return h("div", { class: `oath-formula${cls ? ` ${cls}` : ""}` }, h("span", { class: "chips oath-chips" }, ...chipEls(cs)), h("span", { class: "oath-arrow", "aria-hidden": "true" }, "→"), reward);
}
/** Cut 28b: what forswearing gives back (the core's REFUND_PCT, half). */
export const refundOf = (o: Pick<Oath, "price">): number => Math.floor(o.price / 2);
/** Cut 28b: a report's or exit's oath words from the core's event (`OATH KEPT`, `OATH BROKEN · R2 return`). */
export function oathBeat(ev: { kept: boolean; row: number; cause: string }, rows?: Row[]): string {
  if (ev.kept) return /* copy:callout */ "OATH KEPT";
  // docs/COPY.md: the rule that broke it by its words (`return at 20%`), never its place
  const why = ev.row >= 0 && rows?.[ev.row] ? ruleName(rows, ev.row) : ev.cause;
  return /* copy:callout */ `OATH BROKEN · ${why}`;
}
/** The seal: the wax seal sprite (the verdict banner's), the glyph `⚜` without it. */
export const seal = (): HTMLElement => h("span", { class: "oath-seal", "aria-hidden": "true" });

/** The sworn oath's share as the shaft reads it: `34%` (`<2%` at none of the sims), its ± small; `…` while the forecast has none. */
export function shareEl(app: App, o: Oath): HTMLElement {
  const f = app.lastForecast, s = oathShareOf(f, o);
  if (!s) return h("b", { class: "num oath-share pending dim" }, "…");
  // the edit's move on it (`ForecastVs.oath`, the same paired seeds) rides the share as a notch's does (`▲12`)
  const pm = pmShown(s.share, s.pm), mv = moveOf(app.vsShown()?.oath);
  return h("b", { class: `num oath-share${f?.refined === false ? " rough" : ""}` }, share(s.share, lowOf(f)), pm !== undefined ? h("small", { class: "dim pm" }, ` ±${pm}`) : "",
    mv && mv.dir !== "flat" ? h("i", { class: `vsm dlt ${mv.dir}` }, `${mv.dir === "up" ? "▲" : "▼"}${Math.abs(mv.pts)}`) : "");
}

/** The camp's oath tablet (beside the cage and start tablets): the sworn oath as its formula (`[D3] [no rest] → ▤`) and share, else `oaths`
 *  and how many stand. */
export function paintOathTab(app: App, tab: HTMLElement): void {
  const L = app.lineage, board = oathsOf(L), on = board.length > 0 && oathsEarned(L);
  tab.hidden = !on; if (!on) return;
  const o = swornOf(L);
  tab.classList.toggle("sworn", !!o);
  replace(tab, h("span", { class: "rn num" }, seal()),
    o ? h("span", { class: "rtext" }, formula(chipsOf(o), rewardIcon(o.reward), "tab"), " ", shareEl(app, o), swornAll(L).length > 1 ? h("small", { class: "num dim more" }, ` +${swornAll(L).length - 1}`) : "")
      : h("span", { class: "rtext" }, /* copy:rule_token */ "oaths", " ", h("small", { class: "num dim" }, `${board.length}`), conceptCap("oaths")));
}

/** The oath board: three carved tablets on the parchment, each its deal as a formula (constraint chips → reward) and its stake (two taps
 *  swear it); the sworn one its stake's fate (`$280 · 0/1`), its share, and what forswearing returns (`forswear +$140`). */
export function openOathBoard(app: App, anchor: HTMLElement): void {
  openSheet(() => {
    const list = h("div", { class: "oath-board" });
    const paint = (): void => {
      const L = app.lineage, board = oathsOf(L), sworn = swornAll(L), slots = slotsOf(L), full = sworn.length >= slots;
      replace(list, ...board.map((o) => {
        const isSworn = sworn.some((x) => x.id === o.id), short = L.gold < o.price;
        const sh = isSworn ? oathShareOf(app.lastForecast, o) : undefined;
        const foot = isSworn
          ? [h("span", { class: "oath-fate num", "data-oath": o.id }, h("b", { class: "stake" }, `$${o.price}`), " · ", h("b", { class: "tally" }, "0/1"), " ", shareEl(app, o),
              sh ? h("small", { class: "dim oath-night" }, /* copy:callout */ ` · night ${share(sh.night, lowOf(app.lastForecast))}`) : ""),
            // Cut 29 §1: with two or three slots the forswear names its oath (`forswearOathId`)
            eng(app).forswearOathId || eng(app).forswearOath ? twoTap(/* copy:button */ `forswear +$${refundOf(o)}`, /* copy:button */ `ok +$${refundOf(o)}`, () => void app.mutate(() => eng(app).forswearOathId ? eng(app).forswearOathId!(o.id) : eng(app).forswearOath!(), /* copy:callout */ "oath").then(paint), { class: "chip mini forswear num", key: `forswear:${o.id}` }) : ""]
          : [twoTap(/* copy:button */ `stake $${o.price}`, /* copy:button */ `ok $${o.price}`, () => void app.mutate(() => eng(app).swearOath!(o.id), /* copy:callout */ "oath").then(paint),
              { class: "chip swear num", disabled: short || !eng(app).swearOath || full, key: `swear:${o.id}` }),
            short ? h("small", { class: "num dim why" }, /* copy:callout */ `$${o.price - L.gold} short`) : ""];
        return h("div", { class: `oath tablet${isSworn ? " sworn" : ""}${short && !isSworn ? " short" : ""}`, "data-oath": o.id, "data-kind": o.kind },
          isSworn ? seal() : "",
          formula(chipsOf(o), rewardEl(o.reward), "oath-head"),
          h("div", { class: "oath-foot" }, ...foot),
          isSworn ? stepsEl(app, sh) : "",
          // Cut 28 §1: an oath at a band boss carries the boss's counter as the lineage knows it (`mother: ?` until met)
          // docs/COPY.md (`warlord: aim` read as the boss's move): a known counter reads as the rule that beats him
          // (not on an oath that names its own means — `slay Warlord · with fire` beside `counter: attack boss` read as a contradiction)
          o.counter && o.kind !== "fire" ? h("small", { class: `num oath-counter${/\?$/.test(o.counter) ? " unknown" : ""}` }, counterShown(L, o.counter)) : "");
      }));
      if (sinks.isConnected) paintSinks();   // the slots' count follows a swear
    };
    // Cut 29 §1/§5: the late sinks under the board — the slots sworn of the slots owned, a fresh oath drawn for marks (`drawOath`), and the
    // next work commissioned for gold (`commission`, policy-neutral)
    const sinks = h("div", { class: "oath-sinks" });
    const paintSinks = (): void => {
      const L = app.lineage, d = L.oath_draw, c = L.commission, works = L.works ?? [];
      replace(sinks,
        slotsOf(L) > 1 ? h("div", { class: "oath-slots num" }, /* copy:label */ "sworn ", h("b", null, `${swornAll(L).length}/${slotsOf(L)}`)) : "",
        d && eng(app).drawOath ? h("div", { class: "oath-draw" },
          twoTap(/* copy:button */ `draw ◆${d.cost}`, /* copy:button */ `ok ◆${d.cost}`, () => void app.mutate(() => eng(app).drawOath!(), /* copy:callout */ "oath").then(() => { paint(); paintSinks(); }), { class: "chip draw num", disabled: !d.available, key: "draw" }),
          !d.available && d.needs ? h("small", { class: "num dim why" }, ` ⊘ ${d.needs}`) : "") : "",
        c && eng(app).commission ? h("div", { class: "oath-works" },
          h("span", { class: "label dim" }, /* copy:label */ "works"), " ",
          works.length ? h("span", { class: "works-built num" }, works.join(" · "), " ") : "",
          twoTap(/* copy:button */ `build ${c.label} $${c.price}`, /* copy:button */ `ok $${c.price}`, () => void app.mutate(() => eng(app).commission!(), /* copy:callout */ "works").then(paintSinks), { class: "chip commission num", disabled: !c.available, key: "commission" }),
          !c.available ? h("small", { class: "num dim why" }, /* copy:callout */ ` $${Math.max(0, c.price - L.gold)} short`) : "") : "");
    };
    paint(); paintSinks();
    const off = app.onForecast(() => { if (list.isConnected) paint(); else off(); });
    return h("div", { class: "sheet-body oath-sheet" }, h("div", { class: "label row-label" }, /* copy:label */ "oaths"), list, sinks);
  }, { anchor });
}
/** `warlord: aim` → `counter: attack boss` when the lineage knows the counter's rule; the core's fact otherwise (`mother: ?`). */
const counterShown = (L: Lineage, fact: string): string => { const boss = fact.split(":")[0].trim(); const c = (L.counters ?? []).find((x) => x.boss.endsWith(boss.replace(/ /g, "_")) || x.boss.endsWith(boss)); return c?.text ? /* copy:callout */ `counter: ${c.text}` : fact; };
const eng = (app: App): Pick<App["engine"], "swearOath" | "forswearOath" | "forswearOathId" | "drawOath" | "commission"> => app.engine;

/** The sworn oath on the shaft: the seal and the share, then its formula (`D3 no rest → ▤`). */
export function shaftOath(app: App): HTMLElement | "" {
  const o = swornOf(app.lineage); if (!o) return "";
  return h("span", { class: "shaft-oath num", "data-oath": o.id }, seal(), shareEl(app, o),
    h("span", { class: "so-c" }, chipsOf(o).join(" "), h("span", { class: "oath-arrow", "aria-hidden": "true" }, " → "), rewardIcon(o.reward)));
}

/** The report's oath line (first, among the decisions): its formula, then `kept 3/16`, and what broke it (`broken 11 · R2 return`), or
 *  `kept → card: gas step` when done. */
export function oathProgress(app: App, r: ReturnReport): HTMLElement | null {
  const x = oathReportOf(r); if (!x) return null;
  const o = oathsOf(app.lineage).find((b) => b.id === x.id);
  const cs = chipsOf(x.chips?.length || x.text ? x : o ?? { chips: [], text: "" });
  const reward = x.reward ?? o?.reward;
  return h("section", { class: `rsec oath-sec${x.done ? " done" : ""}` }, h("div", { class: "label" }, /* copy:label */ "oath"),
    h("div", { class: `oath tablet report-oath${x.done ? " done" : ""}`, "data-oath": x.id }, seal(),
      formula(cs, rewardEl(reward), "oath-head"),
      h("div", { class: "oath-foot num" },
        h("b", { class: `oath-tally ${x.kept > 0 ? "up" : "dim"}` }, /* copy:callout */ `kept ${x.kept}/${x.runs}`),
        x.done ? h("span", { class: "oath-kept" }, /* copy:callout */ " · granted") : "",
        !x.done && (x.broken ?? 0) > 0 ? h("span", { class: "oath-broke" }, /* copy:callout */ ` · broken ${x.broken}`, x.cause ? ` · ${x.cause}` : "") : "")),
    // Cut 29 §1: the extra slots' oaths kept this absence, each its reward
    ...(r.oaths_kept ?? []).map((k) => h("div", { class: "oath-kept-extra num" }, seal(), h("b", null, /* copy:callout */ "kept "), rewardEl(k))));
}
