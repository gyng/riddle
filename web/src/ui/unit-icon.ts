import { h } from "./dom";
import { paintSprite } from "./frame";
import { foeSrc, portraitSrc } from "./skin";

/** Identity art uses only the named unit, with a primitive for missing assets. */
export function unitIcon(kind: string, px = 34, hero = false): HTMLElement {
  const id = hero ? `hero_${kind}` : kind;
  const src = hero ? portraitSrc(id) : foeSrc(kind) ?? portraitSrc(`boss_${kind}`);
  const attrs = { class: "unit-icon", "aria-hidden": "true", "data-unit": id, style: `--unit-size:${px}px` };
  if (src) return h("img", { ...attrs, src, alt: "", draggable: "false" });
  const face = h("span", { ...attrs, "data-glyph": "?" });
  paintSprite(face, id, px);
  return face;
}

/** A fixed icon beside a wrapping label; callers keep ownership of its text. */
export function unitLabel(kind: string, text: Node | string, options: { px?: number; hero?: boolean; art?: HTMLElement; className?: string } = {}): HTMLElement {
  return h("span", { class: `unit-label${options.className ? ` ${options.className}` : ""}` }, options.art ?? unitIcon(kind, options.px ?? 34, options.hero), h("span", { class: "unit-label-text" }, text));
}
