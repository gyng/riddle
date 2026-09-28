// Cut 17 §1 — one frame, four wells (docs/UI.md §2). Every place (camp · watch · death · report) is the same three objects:
// the top bar (iron plate: the heir, `$`, `◆`, `★`, the best depth, the settings stud), the well (the place itself), and the
// console (carved stone: the portrait well with its hp ring, a 4 × 2 command card of tiles, the primary gem). Each screen builds
// its frame from these parts (so `main.<screen>` holds its console: the tiles are the screen's buttons).
import type { App } from "../app";
import type { Row } from "../engine/types";
import { h, replace } from "./dom";
import { heirOrd, rowLabel } from "./tokens";
import { renderShaft } from "./forecast";
import { setHeroLook } from "../render/look";
import { icon, portraitSrc, verbIcon } from "./skin";
import { openSettings } from "./settings";
import { openGoldSheet } from "./gold";
import { revealed } from "./reveal";

// --- the top bar ---

export type Bar = { el: HTMLElement; paint(): void; dispose(): void; offers: HTMLElement; freeze(): void };
/** `live`: the camp's bar (the stud the settings; the wake's offers row under it). The `$` opens the gold sheet everywhere but on
 *  the watch (`watch`): a sheet over the run reads as the exit sheet to the tooling. */
export function renderBar(app: App, opts: { live?: boolean; heir?: number; trait?: string; watch?: boolean } = {}): Bar {
  const offers = h("div", { class: "offers", hidden: true });
  const el = h("header", { class: "strip topbar" });
  const mini = portraitMini(app);
  function paint(): void {
    const L = app.lineage, R = revealed(app), past = opts.heir !== undefined && opts.heir !== L.heir;
    // the glyph (`◆` `★`) stays in the text (the tooling reads `◆7`) but the icon stands for it on screen
    const stat = (cls: string, ico: string, glyph: string, n: string | number, on = true): HTMLElement | "" => on ? h("span", { class: `num stat ${cls}` }, icon(ico), glyph ? h("span", { class: "g" }, glyph) : "", String(n)) : "";
    replace(el,
      h("span", { class: "num heir" }, mini.el, heirOrd(opts.heir ?? L.heir)),   // a death's bar names the hero who died (QA 92eb880)
      // the wake's trait chips stand in for the plain trait while the offer stands (the camp fills `offers`)
      opts.live && (L.trait_offer?.length ?? 0) >= 2 ? "" : h("span", { class: "trait" }, opts.heir !== undefined ? opts.trait ?? "" : L.trait),
      (L.ascension?.level ?? 0) > 0 ? h("span", { class: "num asc" }, `↑${L.ascension!.level} ${L.ascension!.variant.replace(/_/g, " ")}`) : "",
      h("div", { class: "stats" },
        // QA 0c6e126 (qaY: `♟14 cowardly` beside `$40 · ◆23 · D8` — "I took $40 as ♟14's purse"): a past heir's bar marks the totals as
        // the lineage's now
        // QA 524827b (qaAA: `♟13 curious · now · $40 ◆21 D8` — "I read ◆21 as that hero's"): the word says whose (`lineage`), and the
        // best depth and rank, which read as the dead hero's floor and standing, stay on the camp's bar
        // docs/COPY.md pass 3: the word over it (`lineage`, then `family`) read "no idea" 11 of 14 times — the purse needs none
        // QA 912e135 (qaW: "the header `$40` is not a button on the death screen; on camp it opens GOLD"): the purse opens the ledger on
        // every screen but the watch (a sheet over the run is the exit sheet's place)
        !opts.watch ? h("button", { class: "num stat gold", onclick: () => openGoldSheet(app) }, icon("gold"), `$${L.gold}`) : h("span", { class: "num stat gold" }, icon("gold"), `$${L.gold}`),
        stat("marks", "mark", "◆", L.marks, R.has("unlocks")),
        stat("rank", "renown", "★", L.rank ?? 0, R.has("rank") && !past),
        stat("best", "depth", "", /* copy:callout */ `best D${L.best_depth}`, R.has("depth") && !past),   // docs/COPY.md pass 2 (`D8` read as "current depth")
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

// --- the hero's portrait (art pass: the painted headshot `hero_<class>`; fallback the atlas sprite `hero_<class>`) ---

type Frame = { x: number; y: number; w: number; h: number };
let atlas: Promise<{ frames: Record<string, Frame>; w: number; h: number } | null> | null = null;
function loadAtlas(): Promise<{ frames: Record<string, Frame>; w: number; h: number } | null> {
  atlas ??= fetch("/art/atlas.json").then((r) => r.json()).then((a) => ({ frames: a.frames, w: a.meta.atlas.w, h: a.meta.atlas.h })).catch(() => null);
  return atlas;
}
/** Paints the class portrait into `face`: the painted headshot when packed (skin.json `portraits`), else a head-and-shoulders
 *  crop of the class sprite (`px` CSS px wide); a missing atlas leaves the well dark. */
export function paintFace(face: HTMLElement, cls: string, px: number, look = ""): void {
  // hero looks: `hero_<class>_<look>`, then `hero_<class>`, then the fighter's sprite (art never blocks the game)
  const ids = [...(look ? [`hero_${cls}_${look}`] : []), `hero_${cls}`];
  if (ids.some((id) => paintPortrait(face, id))) return;
  paintSprite(face, ids[0]!, px, ...ids.slice(1), "hero_fighter");
}
/** Art pass: the painted headshot `id` as `face`'s background (a `painted` face: smooth, cover-fit); false when not packed. */
export function paintPortrait(face: HTMLElement, id: string): boolean {
  const src = portraitSrc(id);
  face.classList.toggle("painted", !!src);
  if (!src) return false;
  face.style.backgroundImage = `url(${src})`; face.dataset.art = id;
  face.style.backgroundSize = "cover";
  face.style.backgroundPosition = "50% 30%";
  return true;
}
/** The fallback: a head-and-shoulders crop of the atlas frame `id` (else `alt`), `px` CSS px wide. */
export function paintSprite(face: HTMLElement, id: string, px: number, ...alts: string[]): void {
  void loadAtlas().then((a) => {
    const k = [id, ...alts].find((x) => a?.frames[x]); const f = k ? a?.frames[k] : undefined; if (!a || !f) return;
    face.dataset.art = k!;
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
  return { el, paint: () => { const k = `${app.lineage.class}_${app.lineage.look ?? ""}`; if (k !== cls) { cls = k; paintFace(face, app.lineage.class, 26, app.lineage.look); } } };
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
  paintFace(face, app.lineage.class, 66, app.lineage.look); setHeroLook(app.lineage.look);   // hero looks: the renderer's hero follows
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

// --- the wide frame (desktop, ≥ 1024 px: web/src/wide.css) ---

/** The wide frame's breakpoint (wide.css keys on the same width). */
export const WIDE_MQ = "(min-width: 1024px)";
export const isWide = (): boolean => typeof matchMedia !== "undefined" && matchMedia(WIDE_MQ).matches;
/** An empty slot under the shaft for the diagnostics meters (no words; the next cut fills it). */
export const metersSlot = (content?: HTMLElement | null): HTMLElement => content ? h("div", { class: "meters-slot filled" }, content) : h("div", { class: "meters-slot", "aria-hidden": "true" });   // Cut 29 §3: the meters fill it
/** Desktop: the columns around a screen's well — the rules left (the set's tablets, read-only: the camp's own are its editor) and the
 *  shaft right with the meters' slot under it. Built only on a wide screen (a phone's DOM is unchanged); `els` go straight into the
 *  screen's `main.frame` (wide.css places them), `dispose` unhooks the shaft. */
export function wideCols(app: App, meters?: HTMLElement | null): { els: HTMLElement[]; dispose(): void; slot?: HTMLElement } {
  if (!isWide()) return { els: [], dispose: () => undefined };
  const plaque = (r: Row): HTMLElement | "" => { const id = verbIcon(r.verb.v); return id ? h("span", { class: "vplaque", "aria-hidden": "true" }, icon(id)) : ""; };   // gfx round 2: as the camp's tablets
  const rows = h("div", { class: "rows" }, ...app.rules.rows.map((r, i) => h("div", { class: "row tablet compact ro", "data-i": i }, h("span", { class: "rn num" }, `${i + 1}`), h("span", { class: "rtext" }, rowLabel(r)), plaque(r))));
  const left = h("aside", { class: "rules-col" }, h("section", { class: "editor compact" }, h("small", { class: "rows-head dim" }, /* copy:label */ "priority"), rows));
  const shaft = renderShaft(app, () => undefined, () => revealed(app).has("gems"));
  shaft.el.tabIndex = -1;
  const slot = metersSlot(meters);
  const right = h("aside", { class: "side-col" }, shaft.el, slot);
  // gfx round 2 (raters: "a reach of '?' clutter" on the desktop death and report): a screen booted straight into them has no forecast
  // yet — the shaft asks for the rules' one (desktop only; its listener is the shaft's)
  if (!app.lastForecast) setTimeout(() => { if (shaft.el.isConnected && !app.lastForecast) void app.emitForecast(); }, 1500);
  return { els: [left, right], dispose: shaft.dispose, slot };
}
