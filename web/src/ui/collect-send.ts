// Cut 118 §6 (research/IDLE_STEAM_2026-10.md: offline that matches online, ≤ 2 taps a return): the report's one action — `collect &
// send`. It opens the chest (the porter's chore, when he is not hired), takes the camp's routine buys (checkin.ts: the unbranched forge
// steps, the lit hire), carries the standing orders as they stand, and sends: the camp mounts and the hero walks into the mouth (or, a
// run already under way, the watch opens on it). A paid batch arms first (`ok $540`, the second tap pays — the house rule for gold); with
// nothing to pay it is one tap. What it carries folds under it (`carried`): the orders, the chest, each buy and its price.
import "../collect.css";
import type { App } from "../app";
import { h, replace, toast, twoTap } from "./dom";
import { routineItems, takeRoutine } from "./checkin";
import { defaultOffer } from "./return-pick";
import { featOrderBits } from "./feats";
import { audio } from "../audio";

/* copy:callout */
const KEEP: Record<string, string> = { best_weapon: "keep weapon", best_armour: "keep armour", none: "keep none" };

/** The standing orders as the fold reads them (≤ 3 words each). */
export function ordersLine(o: NonNullable<App["lineage"]["orders"]>): string[] {
  return [KEEP[o.keep] ?? /* copy:callout */ `keep ${o.keep}`,
    o.repeat ? /* copy:callout */ "restock on" : /* copy:callout */ "restock off",
    o.insure ? /* copy:callout */ "insure on" : /* copy:callout */ "insure off",
    o.start > 1 ? /* copy:callout */ `start D${o.start}` : "",
    // Cut 118: the heir and sink orders off their defaults (the scout's retired wall order is never named)
    ...featOrderBits(o)].filter(Boolean);
}

/** Can the camp send (or watch) now — the gem's own gates. */
export function canSend(app: App): boolean {
  const L = app.lineage;
  if (L.ended || L.town?.home === false) return false;
  if (L.live && L.live.turn > 0) return true;
  return !app.overBudget && app.rules.rows.length > 0;
}

export function collectSend(app: App): { el: HTMLElement; dispose: () => void } {
  const el = h("section", { class: "collect-send", "data-collect": "" });
  let busy = false;
  const go = async (): Promise<void> => {
    if (busy) return;
    busy = true;
    try {
      const chest = app.lineage.tree?.chest ?? 0;
      if (chest > 0 && app.engine.openChest) await app.mutate(() => app.engine.openChest!());
      // round 2 §6: a pick left untapped goes to its default (the tile marked `default`)
      const d = app.engine.takeReturnPick ? defaultOffer(app.lineage.return_pick?.offers, app.lineage.return_pick?.default) : undefined;
      if (d) await app.mutate(() => app.engine.takeReturnPick!(d.id), /* copy:callout */ "Picked");
      const n = await takeRoutine(app);
      if (n || chest > 0) audio.cue("unlock");
      if (n) toast(/* copy:callout */ `${n} taken`);
    } finally { busy = false; }
    if (!el.isConnected) return;
    app.go({ kind: "camp", send: true });
  };
  let last = "";
  const paint = (): void => {
    const L = app.lineage;
    const show = canSend(app);
    el.hidden = !show;
    const items = routineItems(L).filter((x) => (x.kind === "kit" ? !!app.engine.buyKit : !!app.engine.hire));
    const chest = app.engine.openChest ? L.tree?.chest ?? 0 : 0;
    const live = !!L.live && L.live.turn > 0;
    const pick = app.engine.takeReturnPick ? defaultOffer(L.return_pick?.offers, L.return_pick?.default) : undefined;
    const total = items.reduce((s, x) => s + x.price, 0);
    const key = JSON.stringify([show, items, chest, live, total, L.orders, pick?.id]);
    if (key === last) return;
    last = key;
    el.dataset.collect = String(items.length + (chest > 0 ? 1 : 0));
    if (!show) { replace(el); return; }
    const collects = items.length > 0 || chest > 0 || !!pick;
    const label = collects ? (live ? /* copy:button */ "collect & watch" : /* copy:button */ "collect & send") : (live ? /* copy:button */ "watch" : /* copy:button */ "send");
    const btn = total > 0
      ? twoTap(label, /* copy:button */ `ok $${total}`, () => void go(), { class: "btn primary wide collect-go", key: "collect-send" })
      : h("button", { class: "btn primary wide collect-go", onclick: () => void go() }, label);
    const lines: (string | HTMLElement)[] = [
      ...(L.orders ? ordersLine(L.orders).map((t) => h("li", { class: "collect-order" }, t)) : []),
      ...(pick ? [h("li", { class: "collect-pick", "data-pick": pick.id }, /* copy:callout */ `pick · ${pick.title}`)] : []),
      ...(chest > 0 ? [h("li", { class: "collect-chest num" }, /* copy:callout */ `chest +$${chest}`)] : []),
      ...items.map((x) => h("li", { class: "collect-buy num", "data-buy": `${x.kind}:${x.id}` }, x.label, " ", h("span", { class: "dim" }, `$${x.price}`))),
    ];
    replace(el, btn, lines.length ? h("details", { class: "collect-fold" }, h("summary", { class: "dim" }, /* copy:button */ "carried"), h("ul", { class: "lines collect-lines" }, ...lines)) : "");
  };
  paint();
  const off = app.onChange(paint), offR = app.onRules(paint), offL = app.onLive(paint);
  return { el, dispose: () => { off(); offR(); offL(); } };
}
