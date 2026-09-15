// Party strip: kennel cards (kind · L · tags · g), tap → party (≤ party_slots), ≡ → its own rows,
// eggs with hatch_in, hatch (◆2) for eggs from a loss, breed (two level ≥ 2), ledger sheet.
import type { App } from "../app";
import type { Companion, RuleSet } from "../engine/types";
import { h, clear } from "./dom";
import { renderEditor } from "./editor";
import { openSheet } from "./sheet";
import { cloneSet } from "../app";

const nice = (s: string): string => s.replace(/_/g, " ");

export function renderParty(app: App): { el: HTMLElement; refresh(): void } {
  const head = h("div", { class: "label row-label" });
  const cards = h("div", { class: "cards party-cards" });
  const eggs = h("div", { class: "chips eggs" });
  const el = h("section", { class: "party" }, head, cards, eggs);
  let breeding: number[] | null = null;

  function refresh(): void {
    const L = app.lineage; const slots = L.party_slots || 1;
    clear(head); clear(cards); clear(eggs);
    const all = [...L.party, ...L.kennel];
    if (!all.length && !L.eggs.length) { head.append(/* copy:label */ "party", " ", h("span", { class: "num dim" }, `0/${slots}`), ledgerBtn()); return; }
    const canBreed = L.kennel.filter((c) => c.level >= 2).length >= 2;
    head.append(/* copy:label */ "party", " ", h("span", { class: "num dim" }, `${L.party.length}/${slots}`),
      canBreed ? h("button", { class: `mini${breeding ? " on" : ""}`, onclick: () => { breeding = breeding ? null : []; refresh(); } }, /* copy:button */ "breed") : "",
      ledgerBtn());
    for (const c of all) cards.appendChild(card(c, L.party.includes(c)));
    for (const e of L.eggs) {
      eggs.appendChild(h("span", { class: "chip egg" }, "◯ ", nice(e.kind), h("small", { class: "dim" }, ` ${e.tags.map(nice).join(" ")} g${e.gen}`),
        e.from_loss ? h("button", { class: `mini${L.marks >= 2 ? "" : " off"}`, disabled: L.marks < 2, onclick: () => { app.lineage = app.engine.hatch(e.id); app.afterLineage(); } }, "◆2") : h("b", { class: "num" }, ` ${e.hatch_in}`)));
    }
  }
  function ledgerBtn(): HTMLElement { return h("button", { class: "mini", onclick: () => openLedger(app) }, /* copy:button */ "ledger"); }

  function card(c: Companion, inParty: boolean): HTMLElement {
    const picked = breeding?.includes(c.id);
    const onTap = (): void => {
      if (breeding) {
        if (c.level < 2 || inParty) return;
        breeding = picked ? breeding.filter((x) => x !== c.id) : [...breeding, c.id];
        if (breeding.length === 2) { app.lineage = app.engine.breed(breeding[0], breeding[1]); breeding = null; app.afterLineage(); return; }
        refresh(); return;
      }
      const ids = app.lineage.party.map((p) => p.id);
      const next = inParty ? ids.filter((x) => x !== c.id) : [...ids, c.id].slice(-(app.lineage.party_slots || 1));
      app.lineage = app.engine.setParty(next); app.afterLineage();
    };
    return h("div", { class: `card comp${inParty ? " on" : ""}${picked ? " pick" : ""}${breeding && c.level < 2 ? " off" : ""}` },
      h("button", { class: "comp-main", onclick: onTap },
        h("span", { class: "name" }, nice(c.kind), " ", h("b", { class: "num" }, `L${c.level}`), h("small", { class: "dim num" }, ` g${c.gen}`)),
        h("span", { class: "tags dim" }, c.tags.map(nice).join(" · ")),
        h("span", { class: "hp num dim" }, `${c.hp}/${c.max_hp} · ${c.rules.rows.length}/${c.max_rows}`)),
      h("button", { class: "grip", onclick: () => openRules(app, c) }, "≡"));
  }
  refresh();
  return { el, refresh };
}

function openRules(app: App, c: Companion): void {
  const local: RuleSet = cloneSet(c.rules);
  openSheet(() => {
    const ed = renderEditor({
      rules: () => local,
      vocab: () => app.engine.companionVocabulary(c.id),
      changed: () => { app.engine.setCompanionRules(c.id, local); c.rules = cloneSet(local); app.persist(); },
    });
    return h("div", { class: "sheet-body" }, h("div", { class: "sheet-head" }, nice(c.kind), " ", h("b", { class: "num" }, `L${c.level}`)), ed.el);
  });
}

export function openLedger(app: App): void {
  openSheet(() => {
    const L = app.lineage;
    const dot = (on: boolean): HTMLElement => h("span", { class: `dot${on ? " on" : ""}` }, on ? "●" : "○");
    const rows = L.ledger.map((r) => h("div", { class: `lrow${r.seen ? "" : " dim"}` },
      h("span", { class: "k" }, r.seen ? nice(r.kind) : "?"), dot(r.seen), dot(r.known), dot(r.tamed), dot(r.bred)));
    const headRow = h("div", { class: "lrow head" }, h("span", { class: "k" }, ""), /* copy:label */ ...["seen", "known", "tamed", "bred"].map((s) => h("span", { class: "dot-h" }, s)));
    const trophies = L.trophies.filter((t) => t.startsWith("ledger:"));
    return h("div", { class: "sheet-body ledger" }, headRow, ...rows,
      trophies.length ? h("div", { class: "chips" }, ...trophies.map((t) => h("span", { class: "chip fact" }, "★ ", nice(t.slice(7))))) : "");
  });
}
