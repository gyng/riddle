// Party strip: kennel cards (kind · L · tags · g), tap → party (≤ party_slots; Cut 20 §2: a tap on a party card does nothing, its `×` drops it), ≡ → its own rows,
// eggs with hatch_in, hatch (◆2) for eggs from a loss, breed (two level ≥ 2), ledger sheet.
// Cut 7 §1: a boss row of the ledger whose counter is known carries the counter row as a chip; a tap inserts it at
// the top of the active set the way a patch does (overflow rules apply) and opens the camp on it.
import { openDropSheet } from "./patches";
import { openUnlockSheet, visible } from "./unlocks";
import { enemyHost } from "./enemy-tips";
import { unitLabel } from "./unit-icon";
import type { App } from "../app";
import type { Companion, Row, RuleSet } from "../engine/types";
import { h, clear, flash, toast, twoTap } from "./dom";
import { renderEditor } from "./editor";
import { closeAllSheets, openWindow as openSheet } from "./sheet";
import { cloneSet } from "../app";
import { paintPortrait, paintSprite } from "./frame";
import { petMeta, petTip, roleBadge, xpBar } from "./pets";   // Cut 119: the role first, the XP bar, lame · heirs · grudge, the tip

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
      canBreed ? h("button", { class: `mini game-control${breeding ? " on" : ""}`, onclick: () => { breeding = breeding ? null : []; refresh(); if (breeding) toast(/* copy:callout */ "pick two"); } }, /* copy:button */ "breed") : "");
    for (const c of all) cards.appendChild(card(c, L.party.includes(c)));
    for (const e of L.eggs) {
      eggs.appendChild(h("span", { class: "chip egg", "data-egg": e.id }, "◯ ", nice(e.kind), e.sire ? h("small", { class: "egg-sire" }, ` · ${e.sire}`) : "", h("small", { class: "dim" }, ` ${e.tags.map(nice).join(" ")} g${e.gen}`),   // Cut 119: a bred egg names its sire
        // QA e75ec29 (R: "the $50 chip drawn dim charged $50 on one tap … stayed PARTY 0/1"): `hatch $50`, a second tap pays, and the
        // hatchling joins the party when a slot is free
        e.from_loss ? twoTap(/* copy:button */ "hatch $50", /* copy:button */ "ok $50", () => void hatch(e.id), { class: "mini hatch game-control", disabled: L.gold < 50 }) : h("b", { class: "num" }, ` ${e.hatch_in}`)));
    }
  }

  async function hatch(egg: number): Promise<void> {
    const before = new Set([...app.lineage.party, ...app.lineage.kennel].map((c) => c.id));
    if (!(await app.mutate(() => app.engine.hatch(egg)))) return;
    const L = app.lineage, born = L.kennel.find((c) => !before.has(c.id));
    if (born && L.party.length < (L.party_slots || 1)) await app.mutate(() => app.engine.setParty([...L.party.map((p) => p.id), born.id]));
  }
  /** Cut 117 §2: a breed lands as an egg — the toast names it and the new egg chip glints; a refusal says so. */
  async function breed(a: number, b: number): Promise<void> {
    const before = new Set(app.lineage.eggs.map((e) => e.id));
    const ok = await app.mutate(() => app.engine.breed(a, b));
    refresh();
    const born = app.lineage.eggs.find((e) => !before.has(e.id));
    toast(ok ? /* copy:callout */ "egg laid" : /* copy:callout */ "breed refused");
    if (!ok || !born) return;
    const chip = eggs.querySelector<HTMLElement>(`.chip.egg[data-egg="${born.id}"]`);
    if (chip) flash(chip, "hl", 1600);
  }
  function card(c: Companion, inParty: boolean): HTMLElement {
    const picked = breeding?.includes(c.id);
    const onTap = (): void => {
      if (breeding) {
        if (c.level < 2 || inParty) return;
        breeding = picked ? breeding.filter((x) => x !== c.id) : [...breeding, c.id];
        if (breeding.length === 2) { const [a, b] = breeding; breeding = null; void breed(a, b); return; }
        // Cut 117 §2 (blind 8cf9050 A: "breed taps gave no visible result"): the first pick says what the next tap does
        if (breeding.length === 1) toast(/* copy:callout */ "pick a mate");
        refresh(); return;
      }
      // Cut 20 §2 (AD: "tapping a pet twice toggles it off"): a tap selects; a second tap never dismisses — the card's `×` does
      if (inParty) return;
      const ids = app.lineage.party.map((p) => p.id);
      void app.mutate(() => app.engine.setParty([...ids, c.id].slice(-(app.lineage.party_slots || 1))));
    };
    const dismiss = (e: Event): void => { e.stopPropagation(); void app.mutate(() => app.engine.setParty(app.lineage.party.map((p) => p.id).filter((x) => x !== c.id))); };
    // Cut 119: the role first (glyph + word), then the name — its generation is the core's (`Rook III`) — and the level, the XP bar, the
    // quiet meta line (`lame 2 · 4 heirs · grudge King`); the kind, the signatures and the deeds are the tip. A pre-Cut 119 pet (no
    // `life`) keeps the old face: kind · name · L · g.
    const role = roleBadge(c), lame = c.life?.lame ?? 0;
    const name = role
      ? h("span", { class: "name" }, role, h("span", { class: "pet-name" }, c.name), " ", h("b", { class: "num" }, `L${c.level}`))
      : h("span", { class: "name" }, nice(c.kind), " ", h("small", { class: "dim pet-name" }, c.name), " ", h("b", { class: "num" }, `L${c.level}`), h("small", { class: "dim num" }, ` g${c.gen}`));
    const main = h("button", { class: "comp-main game-control", onclick: onTap, "data-role": c.life?.role ?? "" },
      name, xpBar(c) ?? "", petMeta(app.lineage, c) ?? "",
      h("span", { class: "tags dim" }, c.tags.map(nice).join(" · ")),
      h("span", { class: "hp num dim" }, `${c.hp}/${c.max_hp} · ${c.rules.rows.length}/${c.max_rows}`));
    petTip(main, app.lineage, c);
    return h("div", { class: `card comp${inParty ? " on" : ""}${picked ? " pick" : ""}${breeding && c.level < 2 ? " off" : ""}${lame ? " lamed" : ""}`, "data-pet": c.id },
      petFace(c.kind),
      main,
      h("button", { class: "grip game-control", "aria-label": `${c.name} rules`, onclick: () => openRules(app, c) }, "≡"),
      inParty && !breeding ? h("button", { class: "x drop-pet game-control", "aria-label": "×", onclick: dismiss }, "×") : "");
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
        r.seen ? enemyHost(unitLabel(r.kind, nice(r.kind), { className: "k unit-name" }), r.kind, L) : h("span", { class: "k" }, "?"), dot(r.seen), dot(r.known), dot(!!r.studied), dot(r.tamed), dot(r.bred));
      if (!r.counter) return [row];
      // Cut 7 §1: the counter row as a chip; lit when the active set already holds it
      const c = r.counter; const held = app.rules.rows.some((x) => sameRow(x, c.row));
      // Cut 122 §6: a counter on a card not owned is never `added` for the core to refuse — it offers the card's unlock instead
      const card = c.row.verb.v === "tactic" && c.row.verb.a && !app.cardOwned(c.row.verb.a) ? c.row.verb.a : null;
      if (card && !held) {
        const u = visible(app.unlockCat, L).find((x) => x.id === card);
        const lock = h("button", { class: "chip verb counter locked-card", "data-card": card, disabled: !u, onclick: () => { if (u) { closeAllSheets(); openUnlockSheet(app, u); } } },
          h("small", { class: "dim" }, /* copy:rule_token */ "[counter]"), " ", c.text, " ", h("b", { class: "counter-unlock" }, /* copy:button */ "unlock"));
        return [row, h("div", { class: "lrow counter" }, lock)];
      }
      const chip = h("button", { class: `chip verb counter${held ? " on" : ""}`, disabled: held, onclick: () => {
        const p = { row: c.row, insert_at: 0, survive: 0, forecast_delta: 0 };
        closeAllSheets();
        if (app.rowsFull && app.patchTakesRow(p)) { openDropSheet(app, p); return; }   // a full set asks which row makes room
        const i = app.applyPatch(p); app.go({ kind: "camp", highlight: i });
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
