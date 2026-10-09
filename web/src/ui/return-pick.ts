// Cut 113 §3: each return carries a decision — one pick of three (a package drill, Legacy, a forge step at cost), sized by
// the absence, shown as three chunky tiles on the report; on the camp one tile opens the three. Never lost: the pick waits on the camp until taken
// (the core grows it at the next return). The offers are the core's (`Lineage.return_pick`); `takeReturnPick(id)` takes one.
import type { App } from "../app";
import type { ReturnOffer } from "../engine/types";
import { h, replace } from "./dom";
import { tile } from "./frame";
import { audio } from "../audio";
import { openWindow, closeAllSheets } from "./sheet";
import "../return-pick.css";

/* copy:label */
const WORD: Record<string, string> = { drill: "drill", legacy: "legacy", legacy2: "legacy", forge: "forge", marks: "marks" };
const ICON: Record<string, [string, string]> = { drill: ["node_drillmaster", "✦"], legacy: ["legacy", "❖"], legacy2: ["legacy", "❖"], forge: ["forge", "⚒"], marks: ["mark", "◆"] };

/** Cut 118 round 2 §6: the pick's default — what `collect & send` takes when no tile is tapped: the core's (`ReturnPick.default`) when it
 *  names an available offer; else (an older core) Legacy (never a shop, never gold), the drill, the first free offer, the first available. */
export function defaultOffer(offers: ReturnOffer[] | undefined, core?: string): ReturnOffer | undefined {
  const ok = (offers ?? []).filter((o) => o.available);
  const named = core ? ok.find((o) => o.id === core) : undefined;
  if (named) return named;
  return ok.find((o) => o.id === "legacy") ?? ok.find((o) => o.id === "legacy2") ?? ok.find((o) => o.id === "drill") ?? ok.find((o) => !o.price) ?? ok[0];
}

function offerTile(app: App, o: ReturnOffer, busy: { on: boolean }, done?: () => void, isDefault = false): HTMLElement {
  const [ico, glyph] = ICON[o.id] ?? ["mark", "◆"];
  const t = tile({ id: `pick-${o.id}`, label: WORD[o.id] ?? o.id, icon: ico, glyph, disabled: !o.available || !app.engine.takeReturnPick, onclick: () => {
    if (busy.on || !app.engine.takeReturnPick) return;
    busy.on = true;
    void app.mutate(() => app.engine.takeReturnPick!(o.id), /* copy:callout */ "Picked", o.id === "drill").then((ok) => { busy.on = false; if (ok) { audio.cue("unlock"); done?.(); } });
  } });
  t.dataset.offer = o.id;
  t.title = `${o.title} ${o.line}`;
  t.append(h("small", { class: "pick-what" }, o.title), h("small", { class: "pick-line num" }, o.line, o.price ? ` · $${o.price}` : ""));
  if (isDefault) { t.classList.add("pick-default"); t.dataset.default = "1"; t.append(h("small", { class: "pick-default-tag" }, /* copy:label */ "default")); }
  return t;
}

/** The pick's three tiles (hidden when none waits); repaints on every lineage change. On the camp the waiting pick is one tile
 *  (the camp's fold keeps its budget of controls) that opens the three in a sheet. */
export function returnPick(app: App, where: "report" | "camp"): { el: HTMLElement; dispose: () => void } {
  const el = h("section", { class: `return-pick at-${where}`, "data-return-pick": where });
  const busy = { on: false };
  const three = (done?: () => void): HTMLElement => { const os = app.lineage.return_pick?.offers ?? [], d = defaultOffer(os, app.lineage.return_pick?.default); return h("div", { class: "cmd pick-tiles" }, ...os.map((o) => offerTile(app, o, busy, done, o === d))); };
  const head = (size: number): HTMLElement => h("div", { class: "pick-head" }, h("b", { class: "row-label" }, /* copy:label */ "Pick one"), h("small", { class: "dim num" }, "◇".repeat(size)));
  const paint = (): void => {
    const p = app.lineage.return_pick;
    el.hidden = !p || !app.engine.takeReturnPick;
    if (!p || el.hidden) { replace(el); return; }
    if (where === "report") { replace(el, head(p.size), three()); return; }
    const t = tile({ id: "pick", label: /* copy:button */ "pick", icon: "mark", glyph: "◇", fresh: true, onclick: () => {
      closeAllSheets();
      openWindow((close) => h("div", { class: "sheet-body return-pick sheet", "data-return-pick": "sheet" }, head(p.size), three(close)), { anchor: t });
    } });
    t.append(h("small", { class: "pick-line num" }, "◇".repeat(p.size)));
    replace(el, h("div", { class: "cmd pick-one" }, t));
  };
  paint();
  const un = app.onChange(paint);
  return { el, dispose: un };
}
