// Cut 28 §1 — oaths: goals the player picks. The camp's oath board is a carved sheet of three standing oaths (the lineage's, drawn by
// the core from a pool): each a constraint read as chips (`D10` `no drink`), a reward that is never a stat (a card, a party slot, a
// title, a waystone, a verb — its icon and word), and the gold it costs to swear (refunded into its reward when kept). One oath is sworn
// at a time: it rides the shaft with the forecast's share of keeping it (`⚜ D10 no drink · 34%`), and the report leads with its
// progress. The board is carved by the reveal ladder when the purse first affords an oath (`oaths` step), never before.
//
// Wire (core, optional — every field read through the accessors below so the client runs on a core without oaths):
//   Lineage.oaths: Oath[]           the board (≤ 3), the sworn one flagged `sworn`
//   Forecast.oath: OathShare        the sworn oath's share of the sims that keep it (with ±)
//   ReturnReport.oath: OathReport   the sworn oath's night: kept or not, its progress words, done → the reward
//   engine.swearOath(id) / engine.forswearOath()
import type { App } from "../app";
import type { Forecast, Lineage, Oath, OathReport, OathReward, OathShare, ReturnReport } from "../engine/types";
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
export const oathReportOf = (r: ReturnReport): OathReport | undefined => r.oath;
/** Cut 28 §1: the board is carved when an oath is first affordable (or one is sworn). */
export const oathsEarned = (L: Lineage): boolean => oathsOf(L).some((o) => o.sworn || L.gold >= o.price) || !!L.oath || (L.titles?.length ?? 0) > 0;

/** An oath's constraint as chips: the wire's `chips`, else its text split at ` · `. */
export const chipsOf = (o: Pick<Oath, "chips" | "text">): string[] => (o.chips?.length ? o.chips : (o.text ?? "").split(/\s*·\s*/)).map((c) => c.replace(/_/g, " ").trim()).filter(Boolean);
const rewardOf = (r: OathReward | string | undefined): { kind: string; label: string } | undefined => (typeof r === "string" ? { kind: r.split(/[\s:]/)[0], label: r } : r);
/** Each reward kind's icon (the console's own sprites) and glyph fallback. */
const REWARD_ICON: Record<string, [string, string]> = { card: ["unlocks", "▤"], slot: ["party", "◯"], party: ["party", "◯"], title: ["renown", "★"], trophy: ["renown", "★"],
  waystone: ["depth", "▼"], verb: ["edit", "✎"], row: ["edit", "✎"] };
export function rewardEl(r: OathReward | string | undefined): HTMLElement | "" {
  const w = rewardOf(r); if (!w) return "";
  const [ico, glyph] = REWARD_ICON[w.kind] ?? ["renown", "★"];
  return h("span", { class: "oath-reward", "data-kind": w.kind }, icon(ico, glyph), h("span", { class: "rw-l" }, w.label.replace(/_/g, " ")));
}
const chipEls = (cs: string[]): HTMLElement[] => cs.map((c) => h("span", { class: `chip oath-c${/^D\d+/.test(c) ? " depth" : ""}` }, c));
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

/** The camp's oath tablet (beside the cage and start tablets): the sworn oath's chips and share, else `oaths` and how many stand. */
export function paintOathTab(app: App, tab: HTMLElement): void {
  const L = app.lineage, board = oathsOf(L), on = board.length > 0 && oathsEarned(L);
  tab.hidden = !on; if (!on) return;
  const o = swornOf(L);
  tab.classList.toggle("sworn", !!o);
  replace(tab, h("span", { class: "rn num" }, seal()),
    o ? h("span", { class: "rtext" }, /* copy:rule_token */ "oath", h("span", { class: "arrow" }, " → "), ...chipEls(chipsOf(o)), " ", shareEl(app, o))
      : h("span", { class: "rtext" }, /* copy:rule_token */ "oaths", " ", h("small", { class: "num dim" }, `${board.length}`)));
}

/** The oath board: three carved tablets on the parchment, each its constraint chips, its reward, its price (two taps swear it). */
export function openOathBoard(app: App, anchor: HTMLElement): void {
  openSheet(() => {
    const list = h("div", { class: "oath-board" });
    const paint = (): void => {
      const L = app.lineage, board = oathsOf(L), sworn = swornOf(L);
      replace(list, ...board.map((o) => {
        const isSworn = sworn?.id === o.id, short = L.gold < o.price;
        const swear = isSworn ? h("span", { class: "oath-state" }, shareEl(app, o), eng(app).forswearOath ? twoTap(/* copy:button */ "forswear", /* copy:button */ "ok", () => void app.mutate(() => eng(app).forswearOath!(), /* copy:callout */ "oath").then(paint), { class: "chip mini forswear", key: `forswear:${o.id}` }) : "")
          : twoTap(/* copy:button */ `swear $${o.price}`, /* copy:button */ `ok $${o.price}`, () => void app.mutate(() => eng(app).swearOath!(o.id), /* copy:callout */ "oath").then(paint),
            { class: "chip swear num", disabled: short || !eng(app).swearOath || !!sworn, key: `swear:${o.id}` });
        return h("div", { class: `oath tablet${isSworn ? " sworn" : ""}${short && !isSworn ? " short" : ""}`, "data-oath": o.id, "data-kind": o.kind },
          isSworn ? seal() : "",
          h("div", { class: "oath-head" }, h("span", { class: "chips oath-chips" }, ...chipEls(chipsOf(o)))),
          h("div", { class: "oath-foot" }, rewardEl(o.reward), swear,
            short && !isSworn ? h("small", { class: "num dim why" }, /* copy:callout */ `$${o.price - L.gold} short`) : ""),
          // Cut 28 §1: an oath at a band boss carries the boss's counter as the lineage knows it (`mother: ?` until met)
          o.counter ? h("small", { class: `num oath-counter${/\?$/.test(o.counter) ? " unknown" : ""}` }, o.counter) : "",
          // the sworn oath's night: the chance a night of sends keeps it once (`night 61%`)
          isSworn && oathShareOf(app.lastForecast, o) ? h("small", { class: "num dim oath-night" }, /* copy:callout */ `night ${share(oathShareOf(app.lastForecast, o)!.night, lowOf(app.lastForecast))}`) : "");
      }));
    };
    paint();
    const off = app.onForecast(() => { if (list.isConnected) paint(); else off(); });
    return h("div", { class: "sheet-body oath-sheet" }, h("div", { class: "label row-label" }, /* copy:label */ "oaths"), list);
  }, { anchor });
}
const eng = (app: App): Pick<App["engine"], "swearOath" | "forswearOath"> => app.engine;

/** The sworn oath on the shaft: the seal, its chips, the forecast's share of keeping it. */
export function shaftOath(app: App): HTMLElement | "" {
  const o = swornOf(app.lineage); if (!o) return "";
  return h("span", { class: "shaft-oath num", "data-oath": o.id }, seal(), shareEl(app, o), h("span", { class: "so-c" }, chipsOf(o).join(" ")));
}

/** The report's oath line (first, among the decisions): the chips, then `kept 3/16` or `kept → card: gas step`. */
export function oathProgress(app: App, r: ReturnReport): HTMLElement | null {
  const x = oathReportOf(r); if (!x) return null;
  const o = oathsOf(app.lineage).find((b) => b.id === x.id);
  const cs = chipsOf(x.chips?.length || x.text ? x : o ?? { chips: [], text: "" });
  return h("section", { class: `rsec oath-sec${x.done ? " done" : ""}` }, h("div", { class: "label" }, /* copy:label */ "oath"),
    h("div", { class: `oath tablet report-oath${x.done ? " done" : ""}`, "data-oath": x.id }, seal(),
      h("div", { class: "oath-head" }, h("span", { class: "chips oath-chips" }, ...chipEls(cs))),
      h("div", { class: "oath-foot num" },
        h("b", { class: `oath-tally ${x.kept > 0 ? "up" : "dim"}` }, /* copy:callout */ `kept ${x.kept}/${x.runs}`),
        x.done ? h("span", { class: "oath-kept" }, " → ", rewardEl(x.reward ?? o?.reward)) : o ? rewardEl(o.reward) : "")));
}
