// Party strip: kennel cards (kind · L · tags · g), tap → party (≤ party_slots; Cut 20 §2: a tap on a party card does nothing, its `×` drops it), ≡ → its own rows,
// eggs with hatch_in, hatch (◆2) for eggs from a loss, breed (two level ≥ 2), ledger sheet.
// Cut 7 §1: a boss row of the ledger whose counter is known carries the counter row as a chip; a tap inserts it at
// the top of the active set the way a patch does (overflow rules apply) and opens the camp on it.
import { enemyHost } from "./enemy-tips";
import { unitIcon } from "./unit-icon";
import type { App } from "../app";
import type { Companion, Row, RuleSet } from "../engine/types";
import { h, clear, twoTap } from "./dom";
import { renderEditor } from "./editor";
import { closeAllSheets, openWindow as openSheet } from "./sheet";
import { cloneSet } from "../app";
import { paintPortrait, paintSprite } from "./frame";

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
    // Cut 17: `ledger` and `chronicle` are console tiles now (the reveal ladder's 5th heir)
    if (!all.length && !L.eggs.length) { head.append(/* copy:label */ "companions", " ", h("span", { class: "num dim" }, `0/${slots}`)); return; }
    const canBreed = L.kennel.filter((c) => c.level >= 2).length >= 2;
    head.append(/* copy:label */ "companions", " ", h("span", { class: "num dim" }, `${L.party.length}/${slots}`),
      canBreed ? h("button", { class: `mini${breeding ? " on" : ""}`, onclick: () => { breeding = breeding ? null : []; refresh(); } }, /* copy:button */ "breed") : "");
    for (const c of all) cards.appendChild(card(c, L.party.includes(c)));
    for (const e of L.eggs) {
      eggs.appendChild(h("span", { class: "chip egg" }, "◯ ", nice(e.kind), h("small", { class: "dim" }, ` ${e.tags.map(nice).join(" ")} g${e.gen}`),
        // QA e75ec29 (R: "the $50 chip drawn dim charged $50 on one tap … stayed PARTY 0/1"): `hatch $50`, a second tap pays, and the
        // hatchling joins the party when a slot is free
        e.from_loss ? twoTap(/* copy:button */ "hatch $50", /* copy:button */ "ok $50", () => void hatch(e.id), { class: "mini hatch", disabled: L.gold < 50 }) : h("b", { class: "num" }, ` ${e.hatch_in}`)));
    }
  }

  async function hatch(egg: number): Promise<void> {
    const before = new Set([...app.lineage.party, ...app.lineage.kennel].map((c) => c.id));
    if (!(await app.mutate(() => app.engine.hatch(egg)))) return;
    const L = app.lineage, born = L.kennel.find((c) => !before.has(c.id));
    if (born && L.party.length < (L.party_slots || 1)) await app.mutate(() => app.engine.setParty([...L.party.map((p) => p.id), born.id]));
  }
  function card(c: Companion, inParty: boolean): HTMLElement {
    const picked = breeding?.includes(c.id);
    const onTap = (): void => {
      if (breeding) {
        if (c.level < 2 || inParty) return;
        breeding = picked ? breeding.filter((x) => x !== c.id) : [...breeding, c.id];
        if (breeding.length === 2) { const [a, b] = breeding; breeding = null; void app.mutate(() => app.engine.breed(a, b)); return; }
        refresh(); return;
      }
      // Cut 20 §2 (AD: "tapping a pet twice toggles it off"): a tap selects; a second tap never dismisses — the card's `×` does
      if (inParty) return;
      const ids = app.lineage.party.map((p) => p.id);
      void app.mutate(() => app.engine.setParty([...ids, c.id].slice(-(app.lineage.party_slots || 1))));
    };
    const dismiss = (e: Event): void => { e.stopPropagation(); void app.mutate(() => app.engine.setParty(app.lineage.party.map((p) => p.id).filter((x) => x !== c.id))); };
    return h("div", { class: `card comp${inParty ? " on" : ""}${picked ? " pick" : ""}${breeding && c.level < 2 ? " off" : ""}` },
      petFace(c.kind),
      h("button", { class: "comp-main", onclick: onTap },
        h("span", { class: "name" }, nice(c.kind), " ", h("small", { class: "dim" }, c.name), " ", h("b", { class: "num" }, `L${c.level}`), h("small", { class: "dim num" }, ` g${c.gen}`)),
        h("span", { class: "tags dim" }, c.tags.map(nice).join(" · ")),
        h("span", { class: "hp num dim" }, `${c.hp}/${c.max_hp} · ${c.rules.rows.length}/${c.max_rows}`)),
      h("button", { class: "grip", onclick: () => openRules(app, c) }, "≡"),
      inParty && !breeding ? h("button", { class: "x drop-pet", "aria-label": "×", onclick: dismiss }, "×") : "");
  }
  refresh();
  return { el, refresh };
}

/** Art pass: a companion's face on its card — the painted `pet_<kind>` headshot, else a crop of its atlas sprite. */
function petFace(kind: string): HTMLElement {
  const face = h("span", { class: "face" });
  if (!paintPortrait(face, `pet_${kind}`)) paintSprite(face, kind, 34);
  return h("span", { class: "pet-face", "aria-hidden": "true" }, face);
}

function openRules(app: App, c: Companion): void {
  const local: RuleSet = cloneSet(c.rules);
  void app.engine.companionVocabulary(c.id).then((vocab) => openSheet(() => {
    const ed = renderEditor({
      rules: () => local,
      vocab: () => vocab,
      changed: () => { void app.engine.setCompanionRules(c.id, local).catch((e) => console.warn("companion rules", e)); c.rules = cloneSet(local); app.persist(); },
    });
    return h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "rules"), h("div", { class: "sheet-head" }, nice(c.kind), " ", c.name, " ", h("b", { class: "num" }, `L${c.level}`)), ed.el);
  })).catch((e) => console.warn("companion vocabulary", e));
}

const sameRow = (a: Row, b: Row): boolean =>
  a.verb.v === b.verb.v && (a.verb.a ?? "") === (b.verb.a ?? "") && a.conds.length === b.conds.length &&
  a.conds.every((c, i) => c.k === b.conds[i].k && (c.n ?? "") === (b.conds[i].n ?? "") && (c.t ?? "") === (b.conds[i].t ?? ""));

export function openLedger(app: App): void {
  openSheet(() => {
    const L = app.lineage;
    const dot = (on: boolean): HTMLElement => h("span", { class: `dot${on ? " on" : ""}` }, on ? "●" : "○");
    const rows = L.ledger.flatMap((r) => {
      const row = h("div", { class: `lrow${r.seen ? "" : " dim"}` },
        r.seen ? enemyHost(h("span", { class: "k unit-name" }, unitIcon(r.kind), h("span", { class: "unit-name-text" }, nice(r.kind))), r.kind, L) : h("span", { class: "k" }, "?"), dot(r.seen), dot(r.known), dot(!!r.studied), dot(r.tamed), dot(r.bred));
      if (!r.counter) return [row];
      // Cut 7 §1: the counter row as a chip; lit when the active set already holds it
      const c = r.counter; const held = app.rules.rows.some((x) => sameRow(x, c.row));
      const chip = h("button", { class: `chip verb counter${held ? " on" : ""}`, disabled: held, onclick: () => {
        const i = app.applyPatch({ row: c.row, insert_at: 0, survive: 0, forecast_delta: 0 });
        closeAllSheets(); app.go({ kind: "camp", highlight: i });
      } }, h("small", { class: "dim" }, /* copy:rule_token */ "[counter]"), " ", c.text);
      return [row, h("div", { class: "lrow counter" }, chip)];
    });
    const headRow = h("div", { class: "lrow head" }, h("span", { class: "k" }, ""), /* copy:label */ ...["seen", "known", "studied", "tamed", "bred"].map((s) => h("span", { class: "dot-h" }, s)));
    const trophies = L.trophies.filter((t) => t.startsWith("ledger:"));
    // the sheet's title (QA on 952e306: "ledger: 36 rows of '? ○○○○○', no title")
    return h("div", { class: "sheet-body ledger" }, h("div", { class: "label" }, /* copy:label */ "enemy guide"), headRow, ...rows,
      trophies.length ? h("div", { class: "chips" }, ...trophies.map((t) => h("span", { class: "chip fact" }, "★ ", nice(t.slice(7))))) : "");
  });
}
