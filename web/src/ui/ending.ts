// Ascension choices explain their rules; only the reviewed action restarts play.
import type { App, Mounted } from "../app";
import { VARIANTS } from "../engine/types";
import { h } from "./dom";
import { heirOrd } from "./tokens";
import { openWindow } from "./sheet";
import { unitPortrait } from "./unit-icon";

const CHALLENGES: Record<string, { name: string; detail: string }> = /* copy:ending */ {
  no_rest: { name: "No rest", detail: "No dungeon rest · half camp recovery" },
  short_list: { name: "Six rules", detail: "Up to six rules · tactic cards kept" },
  bones_only: { name: "No vault", detail: "Saved gear cleared · vault unavailable" },
  hunted: { name: "Hunted", detail: "Previous nemesis · if still hostile · floor 3+" },
};

export function renderEnding(app: App): Mounted {
  const L = app.lineage, engine = app.engine, selected = L.selected_bloodline ?? 1;
  const tile = (n: string, label: string): HTMLElement => h("div", { class: "tile" }, h("b", { class: "num" }, n), h("span", { class: "label" }, label));
  const status = h("span", { class: "ascension-status", role: "status" });
  let busy = false;
  const chips = h("div", { class: "chips variants" }, ...VARIANTS.map(v => {
    const choice = CHALLENGES[v];
    return h("button", { class: "chip verb ascension-choice", 'data-variant': v, onclick: () => {
      if (busy) return;
      status.textContent = "";
      openWindow(close => {
        const feedback = h("p", { class: "ascension-status", role: "status" });
        const cancel = h("button", { class: "chip", onclick: close }, /* copy:button */ "Cancel");
        const confirm = h("button", { class: "chip ascension-confirm", onclick: () => {
          if (busy) return;
          if (app.engine !== engine || (app.lineage.selected_bloodline ?? 1) !== selected) { close(); return; }
          busy = true; confirm.disabled = true; cancel.disabled = true;
          for (const b of chips.querySelectorAll("button")) b.disabled = true;
          feedback.textContent = "";
          void app.ascend(v).then(ok => {
            if (ok || !chips.isConnected) return;
            busy = false; confirm.disabled = false; cancel.disabled = false;
            for (const b of chips.querySelectorAll("button")) b.disabled = false;
            status.textContent = feedback.textContent = /* copy:callout */ "Try again";
          }).catch(() => {
            if (!chips.isConnected) return;
            busy = false; confirm.disabled = false; cancel.disabled = false;
            for (const b of chips.querySelectorAll("button")) b.disabled = false;
            status.textContent = feedback.textContent = /* copy:callout */ "Try again";
          });
        } }, /* copy:button */ "Start again");
        return h("div", { class: "sheet-body ascension-review" },
          h("h3", null, choice.name), h("p", null, choice.detail),
          h("p", { class: "num" }, /* copy:label */ `Bloodline ${selected}`),
          h("p", null, /* copy:ending */ "Keeps: Legacy · class XP · training · town · Savings · forge progress"),
          h("p", null, /* copy:ending */ "Restarts: heir · depth · supplies · gear upgrades"),
          (L.hero_slots?.length ?? 0) > 1 ? h("p", null, /* copy:ending */ "Other heroes continue") : null,
          h("p", { class: "num ascension-gold" }, /* copy:label */ `Shared gold: $${app.lineage.gold} → $0`),
          h("div", { class: "chips" }, cancel, confirm), feedback);
      }, { stay: true, onClose: () => chips.querySelector<HTMLButtonElement>(`[data-variant="${v}"]`)?.focus() });
      document.querySelector<HTMLButtonElement>('.ascension-review button')?.focus();
    } }, h("b", null, choice.name), h("span", { class: "ascension-description" }, choice.detail));
  }));
  const el = h("main", { class: "ending" },
    h("img", { class: "title-art", src: `${import.meta.env.BASE_URL}art/title.png`, alt: "" }),
    h("div", { class: "ending-body sheet-body" },
      h("div", { class: "ending-victory" }, unitPortrait('mirror_king', 76), h("h1", null, /* copy:callout */ "Dungeon cleared")),
      h("div", { class: "ending-heir num" }, heirOrd(L.heir), " ", h("span", { class: "dim" }, `D${L.best_depth}`)),
      h("div", { class: "tiles" }, tile(`${app.totalRuns()}`, /* copy:label */ "runs"), tile(`${L.graveyard.length}`, /* copy:label */ "deaths"),
        tile(`${L.facts.length}`, /* copy:label */ "facts"), tile(`${L.renown}`, /* copy:label */ "reputation")),
      h("h2", null, /* copy:label */ "Next descent"), chips, status));
  return { el };
}
