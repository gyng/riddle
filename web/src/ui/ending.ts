// Ending: the title art full-bleed, the heir that reached the bottom, four numbers, then the four ascension
// variants as chips (Cut 3): `no rest · short list · bones only · hunted`; a tap ascends and lands in camp.
// Zero prose (copy surface `ending` allows 60 words; the numbers do the telling).
import type { App, Mounted } from "../app";
import { VARIANTS } from "../engine/types";
import { h } from "./dom";
import { heirOrd } from "./tokens";

/* copy:rule_token */
const VARIANT_LABEL: Record<string, string> = { no_rest: "no rest", short_list: "short list", bones_only: "bones only", hunted: "hunted" };

export function renderEnding(app: App): Mounted {
  const L = app.lineage;
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile" }, h("b", { class: "num" }, n), h("span", { class: "label" }, label));
  const cur = L.ascension?.variant ?? "";
  const status = h("span", { class: "ascension-status", role: "status" });
  const chips = h("div", { class: "chips variants" }, ...VARIANTS.map((v) => h("button", { class: `chip verb${v === cur ? " on" : ""}`, onclick: () => {
    for (const b of chips.querySelectorAll("button")) b.disabled = true;
    status.textContent = "";
    const retry = (): void => { if (!chips.isConnected) return; for (const b of chips.querySelectorAll("button")) b.disabled = false; status.textContent = /* copy:callout */ "Try again"; };
    void app.ascend(v).then(ok => { if (!ok) retry(); }).catch(retry);
  } }, VARIANT_LABEL[v] ?? v.replace(/_/g, " "))));
  const el = h("main", { class: "ending" },
    h("img", { class: "title-art", src: `${import.meta.env.BASE_URL}art/title.png`, alt: "" }),
    h("div", { class: "ending-body" },
      h("div", { class: "ending-heir num" }, heirOrd(L.heir), " ", h("span", { class: "dim" }, `D${L.best_depth}`)),
      h("div", { class: "tiles" },
        tile(`${app.totalRuns()}`, /* copy:label */ "runs"),
        tile(`${L.graveyard.length}`, /* copy:label */ "deaths"),
        tile(`${L.facts.length}`, /* copy:label */ "facts"),
        tile(`${L.renown}`, /* copy:label */ "reputation")),
      h("div", { class: "send-bar" }, chips, status)));
  return { el };
}
