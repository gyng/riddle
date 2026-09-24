// Cut 17 §1 — one frame, four wells (docs/UI.md §2). Every place (camp · watch · death · report) is the same three objects:
// the top bar (iron plate: the heir, `$`, `◆`, `★`, the best depth, the settings stud), the well (the place itself), and the
// console (carved stone: the portrait well with its hp ring, a 4 × 2 command card of tiles, the primary gem). Each screen builds
// its frame from these parts (so `main.<screen>` holds its console: the tiles are the screen's buttons).
import type { App } from "../app";
import { h, replace } from "./dom";
import { icon } from "./skin";
import { openSettings } from "./settings";
import { openGoldSheet } from "./gold";
import { revealed } from "./reveal";

// --- the top bar ---

export type Bar = { el: HTMLElement; paint(): void; dispose(): void; offers: HTMLElement; freeze(): void };
/** `live`: the camp's bar (the `$` opens the gold sheet, the stud the settings; the wake's offers row under it). Elsewhere the
 *  bar is read-only but for the stud: a sheet over the run reads as the exit sheet to the tooling. */
export function renderBar(app: App, opts: { live?: boolean } = {}): Bar {
  const offers = h("div", { class: "offers", hidden: true });
  const el = h("header", { class: "strip topbar" });
  const mini = portraitMini(app);
  function paint(): void {
    const L = app.lineage, R = revealed(app);
    // the glyph (`◆` `★`) stays in the text (the tooling reads `◆7`) but the icon stands for it on screen
    const stat = (cls: string, ico: string, glyph: string, n: string | number, on = true): HTMLElement | "" => on ? h("span", { class: `num stat ${cls}` }, icon(ico), glyph ? h("span", { class: "g" }, glyph) : "", String(n)) : "";
    replace(el,
      h("span", { class: "num heir" }, mini.el, `♟${L.heir}`),
      // the wake's trait chips stand in for the plain trait while the offer stands (the camp fills `offers`)
      opts.live && (L.trait_offer?.length ?? 0) >= 2 ? "" : h("span", { class: "trait" }, L.trait),
      (L.ascension?.level ?? 0) > 0 ? h("span", { class: "num asc" }, `↑${L.ascension!.level} ${L.ascension!.variant.replace(/_/g, " ")}`) : "",
      h("div", { class: "stats" },
        opts.live ? h("button", { class: "num stat gold", onclick: () => openGoldSheet(app) }, icon("gold"), `$${L.gold}`) : h("span", { class: "num stat gold" }, icon("gold"), `$${L.gold}`),
        stat("marks", "mark", "◆", L.marks, R.has("unlocks")),
        stat("rank", "renown", "★", L.rank ?? 0, R.has("rank")),
        stat("best", "depth", "", `D${L.best_depth}`, R.has("depth")),
      ),
      h("button", { class: "gear stud", "aria-label": "settings", onclick: () => openSettings(app) }, icon("settings", "⚙")),
      offers,
    );
    mini.paint();
  }
  paint();
  const off = app.onChange(paint);
  // `freeze`: the bar stops following the lineage (the watch's last frame keeps the heir that ran)
  return { el, paint, dispose: off, offers, freeze: () => { off(); } };
}

// --- the hero's portrait (atlas sprite `hero_<class>`) ---

type Frame = { x: number; y: number; w: number; h: number };
let atlas: Promise<{ frames: Record<string, Frame>; w: number; h: number } | null> | null = null;
function loadAtlas(): Promise<{ frames: Record<string, Frame>; w: number; h: number } | null> {
  atlas ??= fetch("/art/atlas.json").then((r) => r.json()).then((a) => ({ frames: a.frames, w: a.meta.atlas.w, h: a.meta.atlas.h })).catch(() => null);
  return atlas;
}
/** Paints the class sprite into `face` (a head-and-shoulders crop, `px` CSS px wide); a missing atlas leaves the well dark. */
function paintFace(face: HTMLElement, cls: string, px: number): void {
  void loadAtlas().then((a) => {
    const f = a?.frames[`hero_${cls}`] ?? a?.frames.hero_fighter; if (!a || !f) return;
    const s = px / (f.w * 0.78);   // the sprite's width minus its weapon reach fills the well
    face.style.backgroundImage = "url(/art/atlas.png)";
    face.style.backgroundSize = `${a.w * s}px ${a.h * s}px`;
    face.style.backgroundPosition = `${-(f.x + f.w * 0.1) * s}px ${-(f.y + f.h * 0.02) * s}px`;
  });
}
function portraitMini(app: App): { el: HTMLElement; paint(): void } {
  const face = h("span", { class: "face" });
  const el = h("span", { class: "mini-portrait", "aria-hidden": "true" }, face);
  let cls = "";
  return { el, paint: () => { if (app.lineage.class !== cls) { cls = app.lineage.class; paintFace(face, cls, 26); } } };
}

export type Portrait = { el: HTMLElement; set(hp: number, label?: Node | string): void };
/** The portrait well: the class sprite in a round iron ring, an hp ring around it (`hp` 0..1), a label plate under it.
 *  With `onclick` it is a button (`cls`: the camp's class picker). */
export function portrait(app: App, o: { hp?: number; label?: Node | string; onclick?: () => void; cls?: string; dead?: boolean } = {}): Portrait {
  const face = h("span", { class: "face" });
  const label = h("span", { class: "plabel num" });
  const kids = [face, h("span", { class: "ring" }), h("span", { class: "rim" }), label];
  const cls = `portrait${o.cls ? ` ${o.cls}` : ""}${o.dead ? " dead" : ""}`;
  const el = o.onclick ? h("button", { class: cls, onclick: o.onclick }, ...kids) : h("div", { class: cls }, ...kids);
  paintFace(face, app.lineage.class, 66);
  const set = (hp: number, text?: Node | string): void => {
    const p = Math.max(0, Math.min(1, hp));
    el.style.setProperty("--hp", p.toFixed(3));
    el.classList.toggle("low", p < 0.3);
    if (text !== undefined) replace(label, text);
  };
  set(o.hp ?? 1, o.label ?? "");
  return { el, set };
}

// --- the console: portrait well · command card (4 × 2) · the primary gem ---

export type TileSpec = { id: string; label: string; icon: string; glyph?: string; onclick: (e: Event) => void; cls?: string; on?: boolean; fresh?: boolean; disabled?: boolean };
/** A command tile: the icon, its one word under it. Pressed on tap (`:active`), `on` while its panel or mode is up. */
export function tile(t: TileSpec): HTMLButtonElement {
  return h("button", { class: `tile${t.cls ? ` ${t.cls}` : ""}${t.on ? " on" : ""}${t.fresh ? " reveal" : ""}`, "data-tile": t.id, disabled: !!t.disabled, onclick: t.onclick },
    icon(t.icon, t.glyph), h("span", { class: "tl" }, t.label));
}
/** The primary gem (`red` for bail's danger), its one word laid over the stone. `pulse` while its action waits. */
export function gem(o: { label: Node | string; onclick: (e: Event) => void; cls?: string; red?: boolean; pulse?: boolean }): HTMLButtonElement {
  return h("button", { class: `gem${o.red ? " red" : ""}${o.pulse ? " pulse" : ""}${o.cls ? ` ${o.cls}` : ""}`, onclick: o.onclick }, o.label);
}

export type Console = { el: HTMLElement; setTiles(tiles: (HTMLElement | null | undefined | false)[]): void };
const SLOTS = 8;
export function renderConsole(o: { portrait: HTMLElement; tiles: (HTMLElement | null | undefined | false)[]; gem: HTMLElement; top?: HTMLElement; cls?: string }): Console {
  const cmd = h("div", { class: "cmd" });
  const setTiles = (tiles: (HTMLElement | null | undefined | false)[]): void => {
    const live = tiles.filter((t): t is HTMLElement => !!t).slice(0, SLOTS);
    replace(cmd, ...live, ...Array.from({ length: SLOTS - live.length }, () => h("span", { class: "tile empty", "aria-hidden": "true" })));
  };
  setTiles(o.tiles);
  const el: HTMLElement = h("footer", { class: `console${o.cls ? ` ${o.cls}` : ""}` }, o.top ?? "",
    h("div", { class: "console-row" }, h("div", { class: "well-slot" }, o.portrait), cmd, h("div", { class: "gem-slot" }, o.gem)));
  // the sheets unfold above the console (`--console-h`, CSS): its measured height, kept current
  if (typeof ResizeObserver !== "undefined") new ResizeObserver(() => { if (el.isConnected) document.documentElement.style.setProperty("--console-h", `${Math.round(el.getBoundingClientRect().height)}px`); }).observe(el);
  return { el, setTiles };
}

/** A panel's close stud (×) — every sheet and panel carries one (Cut 15 QA: "no close control"). */
export const stud = (close: () => void): HTMLButtonElement => h("button", { class: "stud close-stud", "aria-label": "close", onclick: (e: Event) => { e.stopPropagation(); close(); } });   // no text node: the × is drawn (a sheet's text stays its own)
