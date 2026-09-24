// Cut 17 §1: the UI skin — 9-slice frames and command icons packed by tools/ui-skin.py into web/public/ui/, listed in
// skin.json. Only listed files are ever referenced (a missing PNG never 404s: art never blocks the game). Frames are CSS
// `border-image` / `background-image` rules gated on a `skin-<frame>` class on <html>; without it every element keeps its flat
// CSS fallback. Icons are <img> when present; absent, a glyph drawn by CSS (`data-glyph`), so a button's text stays its word.
import skin from "./skin.json";
import { h } from "./dom";

type Skin = { frames: Record<string, { w: number; h: number; slice?: number }>; icons: string[] };
const S = skin as Skin;
const ICONS = new Set(S.icons);

export const hasFrame = (name: string): boolean => name in S.frames;
export const hasIcon = (name: string): boolean => ICONS.has(name);

/** Sets `skin-<frame>` on <html> for every packed frame (the stylesheet's frame rules key on them). */
export function applySkin(): void {
  const root = document.documentElement;
  for (const name of Object.keys(S.frames)) root.classList.add(`skin-${name.replace(/_/g, "-")}`);
}

/** An icon: the packed PNG, or a CSS-drawn glyph (no text node, so `textContent` is the caller's label alone). */
export function icon(name: string, glyph = ""): HTMLElement {
  if (hasIcon(name)) return h("img", { class: `ico ico-${name}`, src: `/ui/icons/${name}.png`, alt: "", draggable: "false", "aria-hidden": "true" });
  return h("span", { class: `ico glyph ico-${name}`, "data-glyph": glyph, "aria-hidden": "true" });
}
