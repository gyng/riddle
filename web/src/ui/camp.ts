// Camp: lineage strip · set tabs · rule editor · forecast · vault loadout · unlocks · send.
import type { App, Mounted } from "../app";
import { h, clear, replace } from "./dom";
import { heroBinding, renderEditor } from "./editor";
import { renderParty } from "./party";
import { renderForecast } from "./forecast";
import { openSettings } from "./settings";
import { available, vaultSlots } from "./unlocks";

export function renderCamp(app: App, highlight?: number): Mounted {
  const strip = h("header", { class: "strip" });
  const tabs = h("nav", { class: "tabs" });
  const editor = renderEditor(heroBinding(app), highlight);
  const party = renderParty(app);
  const fc = renderForecast(app);
  const vault = h("section", { class: "vault" });
  const unlocks = h("section", { class: "unlocks" });
  const send = h("button", { class: "btn primary send", onclick: () => app.go({ kind: "watch" }) }, /* copy:button */ "send");
  const el = h("main", { class: "camp" }, strip, tabs, editor.el, fc.el, party.el, vault, unlocks, h("div", { class: "send-bar" }, send));

  function paintStrip(): void {
    const L = app.lineage;
    replace(strip,
      h("span", { class: "num" }, `♟${L.heir}`),
      h("span", null, L.trait),
      h("span", null, L.class),
      h("span", { class: "num" }, `D${L.best_depth}`),
      h("span", { class: "num marks" }, `◆${L.marks}`),
      h("button", { class: "gear", onclick: () => openSettings(app) }, "⚙"),
    );
  }
  function paintTabs(): void {
    clear(tabs);
    app.sets.forEach((s, i) => tabs.appendChild(h("button", { class: `tab num${i === app.active ? " on" : ""}`, onclick: () => app.selectSet(i) }, `${i + 1}`, h("small", { class: "dim" }, ` ${s.rows.length}`))));
  }
  function paintVault(): void {
    const L = app.lineage; const slots = vaultSlots(L.unlocks);
    clear(vault);
    vault.appendChild(h("div", { class: "label" }, /* copy:label */ "vault", " ", h("span", { class: "num dim" }, `${L.vault.length}/${slots}`)));
    const chips = h("div", { class: "chips" });
    for (const it of L.vault) {
      const on = app.loadout.includes(it.id);
      chips.appendChild(h("button", { class: `chip item${on ? " on risk" : ""}`, onclick: () => {
        app.setLoadout(on ? app.loadout.filter((x) => x !== it.id) : [...app.loadout, it.id]);
      } }, on ? "⚠ " : "", it.label));
    }
    for (let i = L.vault.length; i < slots; i++) chips.appendChild(h("span", { class: "chip empty" }, "·"));
    vault.appendChild(chips);
  }
  function paintUnlocks(): void {
    const L = app.lineage;
    clear(unlocks);
    const list = available(L.unlocks);
    if (!list.length) return;
    unlocks.appendChild(h("div", { class: "label" }, /* copy:label */ "unlocks"));
    const grid = h("div", { class: "cards" });
    for (const u of list) {
      const can = L.marks >= u.cost;
      grid.appendChild(h("button", { class: `card${can ? "" : " off"}`, disabled: !can, onclick: () => app.buy(u.id) }, h("span", null, u.label), h("span", { class: "num cost" }, `◆${u.cost}`)));
    }
    unlocks.appendChild(grid);
  }
  function paintAll(): void { paintStrip(); paintTabs(); paintVault(); paintUnlocks(); party.refresh(); editor.refresh(); }
  paintAll();
  const off = app.onChange(paintAll);
  return { el, dispose: () => { off(); fc.dispose(); } };
}
