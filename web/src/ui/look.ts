// Hero looks: the heir's cosmetic look (`Lineage.look`: male | female | cat), inherited by every heir, never read by a run. The camp's
// portrait well carries a small stud; its sheet shows the class's three headshots and one tap swaps (`app.setLook`). Portraits and
// the renderer's hero draw `hero_<class>_<look>`, falling back to `hero_<class>`, then the procedural silhouette.
import type { App } from "../app";
import { LOOKS, setHeroLook } from "../render/look";
import { h } from "./dom";
import { paintFace } from "./frame";
import { openWindow as openSheet } from "./sheet";

/** The renderer's hero follows the lineage's look (called before a viewer loads, and on every portrait paint). */
export function syncLook(app: App): void { setHeroLook(app.lineage.look); }

/** The stud on the camp's portrait well: opens the look sheet (its tap never reaches the well's own class picker). */
export function lookStud(app: App): HTMLElement {
  const open = (e: Event): void => { e.stopPropagation(); e.preventDefault(); openLooks(app); };
  return h("span", { class: "look-stud", role: "button", tabindex: "0", "aria-label": /* copy:label */ "Appearance", "data-look": app.lineage.look ?? "",
    onclick: open, onkeydown: (e: Event) => { const k = (e as KeyboardEvent).key; if (k === "Enter" || k === " ") open(e); } }, "◐");
}

/** The look sheet: the current class's three headshots; one tap swaps and closes. */
export function openLooks(app: App): void {
  openSheet((close) => {
    const cls = app.lineage.class;
    const row = h("div", { class: "looks" }, ...LOOKS.map((look) => {
      const face = h("span", { class: "face" });
      paintFace(face, cls, 72, look);
      const on = look === app.lineage.look;
      return h("button", { class: `look game-control${on ? " on" : ""}`, "data-look": look, "aria-pressed": on ? "true" : "false",
        onclick: async () => { if (!on && await app.setLook(look)) syncLook(app); close(); } },
        h("span", { class: "look-well" }, face), h("span", { class: "look-name" }, look));
    }));
    return h("div", { class: "sheet-body look-sheet" }, h("div", { class: "label row-label" }, /* copy:label */ "Appearance"), row);
  });
}
