import { h } from "./dom";
import { paintSprite } from "./frame";
import { foeSrc, portraitSrc } from "./skin";

/** Identity art uses only the named unit, with a primitive for missing assets. */
export function unitIcon(kind: string, px = 34, hero = false): HTMLElement {
  const id = hero ? `hero_${kind}` : kind;
  const src = hero ? portraitSrc(id) : foeSrc(kind) ?? portraitSrc(`boss_${kind}`);
  const attrs = { class: "unit-icon", "aria-hidden": "true", "data-unit": id, style: `--unit-size:${px}px` };
  const fallback = (): HTMLElement => {
    const face = h("span", { ...attrs, "data-glyph": "?" });
    paintSprite(face, id, px);
    return face;
  };
  if (src) {
    const image = h("img", { ...attrs, src, alt: "", draggable: "false" });
    image.onerror = () => image.replaceWith(fallback());
    return image;
  }
  return fallback();
}

/** A fixed icon beside a wrapping label; callers keep ownership of its text. */
export function unitLabel(kind: string, text: Node | string, options: { px?: number; hero?: boolean; art?: HTMLElement; className?: string } = {}): HTMLElement {
  return h("span", { class: `unit-label${options.className ? ` ${options.className}` : ""}` }, options.art ?? unitIcon(kind, options.px ?? 34, options.hero), h("span", { class: "unit-label-text" }, text));
}

/** A prominent portrait in the same iron well as the game's hero portrait. */
export function unitPortrait(kind: string, px = 96, hero = false): HTMLElement {
  return h("span", { class: "unit-portrait", style: `--portrait-size:${px}px`, "aria-hidden": "true" },
    unitIcon(kind, Math.round(px * .78), hero), h("span", { class: "unit-portrait-rim" }));
}
