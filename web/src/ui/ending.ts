// Ending: the title art full-bleed, the heir that reached the bottom, four numbers, `again`.
// Zero prose (copy surface `ending` allows 60 words; the numbers do the telling).
import type { App, Mounted } from "../app";
import { h } from "./dom";

export function renderEnding(app: App): Mounted {
  const L = app.lineage;
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile" }, h("b", { class: "num" }, n), h("span", { class: "label" }, label));
  const again = h("button", { class: "btn primary send", onclick: () => { again.disabled = true; void app.again(); } }, /* copy:button */ "again");
  const el = h("main", { class: "ending" },
    h("img", { class: "title-art", src: "/art/title.png", alt: "" }),
    h("div", { class: "ending-body" },
      h("div", { class: "ending-heir num" }, `♟${L.heir}`, " ", h("span", { class: "dim" }, `D${L.best_depth}`)),
      h("div", { class: "tiles" },
        tile(`${app.totalRuns()}`, /* copy:label */ "runs"),
        tile(`${L.graveyard.length}`, /* copy:label */ "deaths"),
        tile(`${L.facts.length}`, /* copy:label */ "facts"),
        tile(`${L.renown}`, /* copy:label */ "renown")),
      h("div", { class: "send-bar" }, again)));
  return { el };
}
