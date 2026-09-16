// Camp: lineage strip · set tabs · rule editor · forecast · vault loadout · unlocks · send.
import type { App, Mounted } from "../app";
import type { UnlockInfo } from "../engine/types";
import { h, clear, replace, spanOf } from "./dom";
import { heroBinding, renderEditor } from "./editor";
import { renderParty } from "./party";
import { renderForecast } from "./forecast";
import { openSettings } from "./settings";
import { classList, supplyCap, visible, vaultSlots } from "./unlocks";
import { salvageValue } from "./salvage";
import { CLASS_VERBS, xpToNext } from "../engine/classes";
import { verbLabel } from "./tokens";
import { openSheet } from "./sheet";

const SET_NAME_MAX = 12;
export const setName = (s: { name?: string }, i: number): string => (s.name ?? "").trim().slice(0, SET_NAME_MAX) || `${i + 1}`;

export function renderCamp(app: App, highlight?: number): Mounted {
  const strip = h("header", { class: "strip" });
  const tabs = h("nav", { class: "tabs" });
  const editor = renderEditor(heroBinding(app), highlight);
  const party = renderParty(app);
  const fc = renderForecast(app);
  const vault = h("section", { class: "vault" });
  const supplies = h("section", { class: "supplies" });
  const unlocks = h("section", { class: "unlocks" });
  const send = h("button", { class: "btn primary send", onclick: () => { if (!app.overBudget) app.go({ kind: "watch" }); } }, /* copy:button */ "send");
  const rest = h("span", { class: "rest num" });
  const el = h("main", { class: "camp" }, strip, tabs, editor.el, fc.el, party.el, vault, supplies, unlocks, h("div", { class: "send-bar" }, rest, send));

  function paintStrip(): void {
    const L = app.lineage; const lvl = L.classes?.[L.class] ?? { level: 1, xp: 0 };
    replace(strip,
      h("span", { class: "num" }, `♟${L.heir}`),
      // Cut 3: `↑2 no rest` once ascended
      (L.ascension?.level ?? 0) > 0 ? h("span", { class: "num asc" }, `↑${L.ascension!.level} ${L.ascension!.variant.replace(/_/g, " ")}`) : "",
      h("span", null, L.trait),
      h("button", { class: "cls", onclick: () => pickClass() }, h("span", null, L.class, " ", h("b", { class: "num" }, `L${lvl.level}`)),
        h("span", { class: "xp" }, h("span", { class: "fill", style: `width:${Math.round((lvl.xp / xpToNext(lvl.level)) * 100)}%` }))),
      h("span", { class: "num" }, `D${L.best_depth}`),
      h("span", { class: "num rank" }, `★${L.rank ?? 0}`),
      h("span", { class: "num gold" }, `$${L.gold}`),
      h("span", { class: "num marks" }, `◆${L.marks}`),
      h("button", { class: "gear", onclick: () => openSettings(app) }, "⚙"),
    );
    // Cut 2 §1: camp rest remaining; `send` skips it, so the number just disappears
    const restS = L.rest_left_s ?? 0;
    replace(rest, /* copy:label */ "rest", " ", spanOf(restS));
    rest.hidden = restS <= 0;
  }
  function openForge(app2: App): void {
    openSheet(() => {
      const L = app2.lineage; const rows = Object.entries(L.forge ?? {}).sort((a, b) => b[1].salvaged - a[1].salvaged);
      const head = h("div", { class: "lrow head" }, h("span", { class: "k" }, ""), /* copy:label */ ...["salvaged", "craft", "tier"].map((s) => h("span", { class: "dot-h" }, s)));
      return h("div", { class: "sheet-body ledger forge" }, head, ...rows.map(([kind, f]) => h("div", { class: "lrow" },
        h("span", { class: "k" }, kind.replace(/_/g, " ")), h("span", { class: "dot num" }, `${f.salvaged}`),
        h("span", { class: `dot${f.craftable ? " on" : ""}` }, f.craftable ? "⚒" : "○"), h("span", { class: `dot num${f.tier ? " on" : ""}` }, f.tier ? `+${f.tier}` : "·"))));
    });
  }
  // Cut 2 §4: whatever the lineage and the unlock catalogue provide (fighter · rogue · ranger · caster).
  // Cut 5 §6: each row carries the class's verb ladder as chips (`L1 shield bash · L3 cleave · …`), reached rungs lit.
  function pickClass(): void {
    openSheet((close) => {
      const L = app.lineage; const grid = h("div", { class: "classes" });
      const paint = (cat: Parameters<typeof classList>[1]): void => {
        clear(grid);
        for (const { cls, owned, level } of classList(L, cat)) {
          const ladder = Object.entries(CLASS_VERBS[cls] ?? {}).flatMap(([l, vs]) => vs.map((v) => h("span", { class: `chip rung num${Number(l) <= level && owned ? " on" : ""}` }, `L${l} `, verbLabel({ v }))));
          grid.appendChild(h("div", { class: "class-row" },
            h("button", { class: `chip verb${cls === L.class ? " on" : ""}${owned ? "" : " off"}`, disabled: !owned, onclick: () => { void app.setClass(cls); close(); } }, cls, " ", h("b", { class: "num" }, `L${level}`)),
            ladder.length ? h("div", { class: "chips ladder" }, ...ladder) : ""));
        }
      };
      paint(unlockCat);
      if (!unlockCat) void app.engine.unlocks().then((cat) => paint(cat)).catch(() => { /* ladders only */ });
      return h("div", { class: "sheet-body" }, grid);
    });
  }
  // Cut 5 §6: sets carry a player-typed name (≤ 12 chars, the game's only free text; default `1 · 2 · 3`); ✎ on the active tab renames
  function paintTabs(): void {
    clear(tabs);
    app.sets.forEach((s, i) => {
      tabs.appendChild(h("button", { class: `tab num${i === app.active ? " on" : ""}`, onclick: () => app.selectSet(i) }, setName(s, i), h("small", { class: "dim" }, ` ${s.rows.length}`)));
      if (i === app.active) tabs.appendChild(h("button", { class: "tab edit", onclick: () => renameSet(i) }, "✎"));
    });
  }
  function renameSet(i: number): void {
    openSheet((close) => {
      const input = h("input", { class: "name-input", type: "text", maxlength: SET_NAME_MAX, autocomplete: "off", spellcheck: "false", value: app.sets[i].name ?? "", placeholder: `${i + 1}` });
      const commit = (): void => { app.renameSet(i, input.value); close(); };
      input.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); commit(); } });
      setTimeout(() => input.focus(), 0);
      return h("div", { class: "sheet-body" }, input, h("button", { class: "btn primary wide", onclick: commit }, /* copy:button */ "ok"));
    });
  }
  function paintVault(): void {
    const L = app.lineage; const slots = vaultSlots(L.unlocks);
    clear(vault);
    vault.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "vault", " ", h("span", { class: "num dim" }, `${L.vault.length}/${slots}`),
      h("button", { class: "mini", onclick: () => openForge(app) }, /* copy:button */ "forge")));
    const chips = h("div", { class: "chips" });
    for (const it of L.vault) {
      const on = app.loadout.includes(it.id);
      chips.appendChild(h("button", { class: `chip item${on ? " on risk" : ""}`, onclick: () => {
        app.setLoadout(on ? app.loadout.filter((x) => x !== it.id) : [...app.loadout, it.id]);
      } }, on ? "⚠ " : "", it.label));
      if (on) {
        const ins = (L.insured ?? []).includes(it.id);
        const price = Math.ceil(salvageValue(it.kind, "bank") * 10 / 4);
        chips.appendChild(h("button", { class: `chip mini${ins ? " on" : ""}`, disabled: ins || L.gold < price,
          onclick: () => void app.mutate(() => app.engine.insure(it.id)) }, ins ? /* copy:label */ "insured" : `$${price}`));
      }
    }
    for (let i = L.vault.length; i < slots; i++) chips.appendChild(h("span", { class: "chip empty" }, "·"));
    vault.appendChild(chips);
    // keep preference for offline exits
    const prefs = h("div", { class: "chips prefs" }, h("span", { class: "dim" }, /* copy:label */ "keep"),
      /* copy:label */ ...[["best_weapon", "weapon"], ["best_armour", "armour"], ["none", "none"]].map(([id, lbl]) =>
        h("button", { class: `chip${(L.keep_pref ?? "best_weapon") === id ? " on" : ""}`, onclick: () => void app.mutate(() => app.engine.setKeepPref(id)) }, lbl)));
    vault.appendChild(prefs);
  }
  function paintSupplies(): void {
    const L = app.lineage; const picks = L.supplies ?? []; const cap = supplyCap(L.unlocks); const full = picks.length >= cap;
    clear(supplies);
    supplies.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "supplies", " ", h("span", { class: "num dim" }, `${picks.length}/${cap}`),
      picks.length ? h("button", { class: "mini", onclick: () => void app.mutate(() => app.engine.clearSupplies()) }, "×") : ""));
    const chips = h("div", { class: "chips" });
    for (const p of picks) chips.appendChild(h("span", { class: "chip item on" }, p.label));
    supplies.appendChild(chips);
    const gen = ++supplyGen;
    void app.engine.supplyCatalogue().then((cat) => {
      if (gen !== supplyGen) return;
      for (const e of cat) {
        const can = !full && L.gold >= e.price;
        chips.appendChild(h("button", { class: `chip buy${can ? "" : " off"}`, disabled: !can, onclick: () => void app.mutate(() => app.engine.buySupply(e.kind)) }, e.label, " ", h("b", { class: "num gold" }, `$${e.price}`)));
      }
    }).catch((e) => console.warn("catalogue", e));
  }
  let supplyGen = 0, unlockGen = 0;
  let unlockCat: Parameters<typeof classList>[1];
  function paintUnlocks(): void {
    const gen = ++unlockGen;
    void app.engine.unlocks().then((cat) => {
      if (gen !== unlockGen) return;
      unlockCat = cat;
      // forecast deltas arrive later (0.3–2 s of sims); repaint once with them, never blocking the shelf
      if (!cat.some((u) => u.delta !== undefined)) {
        void app.engine.unlockDeltas().then((withDeltas) => { if (gen === unlockGen && withDeltas.some((u) => u.delta)) { unlockGen++; paintFrom(withDeltas); } }).catch(() => { /* deltas are optional */ });
      }
      paintFrom(cat);
    }).catch((e) => console.warn("unlocks", e));
  }
  function paintFrom(cat: UnlockInfo[]): void {
    {
      clear(unlocks);
      const list = visible(cat);
      if (!list.length) return;
      unlocks.appendChild(h("div", { class: "label" }, /* copy:label */ "unlocks"));
      const grid = h("div", { class: "cards" });
      for (const u of list) {
        // `available` = prerequisite + fact gate + affordable (engine truth). Two dims: gated (the `needs` line
        // is what is missing, marks are there) and unaffordable.
        // Cut 4 §9: the forecast delta of buying (tactic cards), only when the catalogue carries one and it is not 0
        const d = u.delta === undefined ? 0 : Math.round(u.delta * 100);
        grid.appendChild(h("button", { class: `card${u.available ? "" : u.gated ? " gated" : " off"}`, disabled: !u.available, onclick: () => void app.buy(u.id) },
          h("span", { class: "card-main" }, h("span", null, u.label), u.needs ? h("small", { class: "needs dim" }, u.gated ? "⊘ " : "", u.needs.replace(/_/g, " ")) : "",
            d ? h("small", { class: `num delta ${d > 0 ? "up" : "down"}` }, /* copy:unlock_card */ `reach ${d > 0 ? "+" : "−"}${Math.abs(d)}%`) : ""),
          h("span", { class: "num cost" }, `◆${u.cost}`)));
      }
      unlocks.appendChild(grid);
    }
  }
  // Cut 4 §1: `send` waits while the set is over budget (the editor shows which row to drop)
  function paintSend(): void { send.disabled = app.overBudget; paintTabs(); }
  function paintAll(): void { paintStrip(); paintTabs(); paintVault(); paintSupplies(); paintUnlocks(); party.refresh(); editor.refresh(); paintSend(); }
  paintAll();
  const off = app.onChange(paintAll), offRules = app.onRules(paintSend);
  return { el, dispose: () => { off(); offRules(); fc.dispose(); } };
}
