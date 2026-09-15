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
  const supplies = h("section", { class: "supplies" });
  const unlocks = h("section", { class: "unlocks" });
  const send = h("button", { class: "btn primary send", onclick: () => app.go({ kind: "watch" }) }, /* copy:button */ "send");
  const el = h("main", { class: "camp" }, strip, tabs, editor.el, fc.el, party.el, vault, supplies, unlocks, h("div", { class: "send-bar" }, send));

  function paintStrip(): void {
    const L = app.lineage;
    replace(strip,
      h("span", { class: "num" }, `♟${L.heir}`),
      h("span", null, L.trait),
      h("span", null, L.class),
      h("span", { class: "num" }, `D${L.best_depth}`),
      h("span", { class: "num gold" }, `$${L.gold}`),
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
  function paintSupplies(): void {
    const L = app.lineage; const picks = L.supplies ?? []; const full = picks.length >= 3;
    clear(supplies);
    supplies.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "supplies", " ", h("span", { class: "num dim" }, `${picks.length}/3`),
      picks.length ? h("button", { class: "mini", onclick: () => { app.lineage = app.engine.clearSupplies(); app.afterLineage(); } }, "×") : ""));
    const chips = h("div", { class: "chips" });
    for (const p of picks) chips.appendChild(h("span", { class: "chip item on" }, p.label));
    for (const e of app.engine.supplyCatalogue()) {
      const can = !full && L.gold >= e.price;
      chips.appendChild(h("button", { class: `chip buy${can ? "" : " off"}`, disabled: !can, onclick: () => { app.lineage = app.engine.buySupply(e.kind); app.afterLineage(); } }, e.label, " ", h("b", { class: "num gold" }, `$${e.price}`)));
    }
    supplies.appendChild(chips);
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
  function paintAll(): void { paintStrip(); paintTabs(); paintVault(); paintSupplies(); paintUnlocks(); party.refresh(); editor.refresh(); }
  paintAll();
  const off = app.onChange(paintAll);
  return { el, dispose: () => { off(); fc.dispose(); } };
}
