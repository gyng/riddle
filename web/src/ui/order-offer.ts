// Cut 120, option B (docs/CUT120_AUTOMATION_FILL.md "Defaults: option C measured, option B taken"): the Legacy and ranks orders
// default `off` for every lineage, so the camp offers each one once, quietly, the first time it is useful — Legacy when it can first
// buy an upgrade, ranks when a worker rank is first due. One chip row, never a modal; either chip dismisses it for good (per lineage,
// browser storage). Nothing waits on it: `collect & send` walks past it, and the orders sheet keeps both orders settable any time.
import type { App } from "../app";
import type { Lineage, StandingOrders } from "../engine/types";
import { h, replace } from "./dom";
import { manyBloodlines } from "./orders";

export type OfferKey = "legacy" | "ranks";
const KEY = (L: Pick<Lineage, "seed">): string => /* copy:none */ `riddle.offers.${L.seed ?? 0}`;

function load(L: Pick<Lineage, "seed">): Record<string, boolean> {
  try { const v = JSON.parse(localStorage.getItem(KEY(L)) ?? "{}"); return v && typeof v === "object" ? v as Record<string, boolean> : {}; } catch { return {}; }
}
/** The offer `k` dismissed for good on this lineage (taken or `not now`). */
export function offerDone(L: Pick<Lineage, "seed">, k: OfferKey): boolean { return !!load(L)[k]; }
export function markOffer(L: Pick<Lineage, "seed">, k: OfferKey): void {
  const s = load(L); if (s[k]) return; s[k] = true;
  try { localStorage.setItem(KEY(L), JSON.stringify(s)); } catch { /* private mode: offered again next visit */ }
}

/** Legacy can buy an upgrade now. */
export const legacyUseful = (L: Lineage): boolean => !!L.legacy_upgrades?.some((u) => u.affordable);
/** A worker rank is due now (on offer: `Works.lit_rank`, or a hired hand whose wait is over). */
export const rankDue = (L: Lineage): boolean => !!L.tree?.lit_rank
  || !!L.tree?.nodes.some((n) => n.kind === "worker" && n.state === "done" && n.rank_wait_d === 0 && !!n.rank_price);

/** The offer showing now, if any: Legacy first, then ranks; only while the order is still `off`. */
export function pendingOffer(L: Lineage): OfferKey | null {
  const o = L.orders;
  if (!o) return null;
  // an order already set off `off` (the sheet) is a choice made: never offered after
  if (o.legacy && o.legacy !== "off") markOffer(L, "legacy");
  if (o.ranks && o.ranks !== "off") markOffer(L, "ranks");
  if (o.legacy === "off" && !offerDone(L, "legacy") && legacyUseful(L)) return "legacy";
  if (o.ranks === "off" && !offerDone(L, "ranks") && rankDue(L)) return "ranks";
  return null;
}

/** The orders sent on accepting: the order on, shared to every bloodline when the town keeps several (unless already shared). */
export function acceptPatch(L: Lineage, k: OfferKey): StandingOrders {
  const o = L.orders!, shared = o.shared ?? [];
  const share = manyBloodlines(L) && !shared.includes(k) ? { shared: [...shared, k] } : {};
  return { ...o, ...(k === "legacy" ? { legacy: "balanced" } : { ranks: "auto" }), ...share };
}

/** The camp's offer row: one label, the order's chip, `not now`. Hidden when nothing is offered. */
export function orderOffer(app: App): { el: HTMLElement; dispose: () => void } {
  const el = h("div", { class: "camp-feats order-offer", hidden: true });
  let last = "";
  const paint = (): void => {
    const L = app.lineage;
    const home = !L.live && !L.ended && L.town?.home !== false;
    const k = home && app.engine.setOrders ? pendingOffer(L) : null;
    const key = String(k);
    if (key === last) return;
    last = key;
    el.hidden = !k;
    el.dataset.offer = k ?? "";
    if (!k) { replace(el); return; }
    const done = (): void => { markOffer(app.lineage, k); last = ""; paint(); };
    const yes = h("button", { class: "chip feat-chip order-offer-yes", "data-offer-yes": k,
      title: k === "legacy" ? /* copy:tooltip */ "health → damage → armour in turn · run setup" : /* copy:tooltip */ "ranks bought when due · run setup",
      onclick: () => { markOffer(app.lineage, k); void app.mutate(() => app.engine.setOrders!(acceptPatch(app.lineage, k)), /* copy:callout */ k === "legacy" ? "legacy" : "ranks").then(done); } },
      k === "legacy" ? /* copy:button */ "balanced" : /* copy:button */ "auto");
    const no = h("button", { class: "chip feat-chip order-offer-no", "data-offer-no": k, onclick: done }, /* copy:button */ "not now");
    replace(el, h("div", { class: "chips feat-chips" },
      h("small", { class: "dim" }, k === "legacy" ? /* copy:callout */ "Legacy auto-spend" : /* copy:callout */ "rank auto-buy"), " ", yes, " ", no));
  };
  paint();
  const off = app.onChange(paint), offL = app.onLive(paint);
  return { el, dispose: () => { off(); offL(); } };
}
