// Camp: lineage strip · set tabs · rule editor · forecast · vault loadout · unlocks · send.
// Cut 6: the strip's gold opens the `gold` sheet (the last 20 movements, newest first, §1); the disabled send reads
// `6/5 · drop one` and a card's buy reads `◆3 · takes a row` on a full set (§4); owned cards and automations stay on the
// shelf as chips that open their rows (§6); `rest 12m` is a chip that answers `send skips rest` (§7).
// Cut 9: an unlock card opens its sheet, the buy is there (§2); the forge sheet shows each kind's ladder (§10).
// Cut 10 §3: the rest chip reads `rest 20m · send skips` permanently; a greyed supply says why under its price (`3/3 slots`,
// the engine's `needs`, `$12 short`); the `+1 row` card is dimmed `fill rows` while free rows exist; a card's reach delta is
// labelled `at end` (a bought card becomes the last row). Cut 10 §4: the camp drone (biome of the next floor) while mounted.
// Cut 12 §1: rows are own rows — `fill rows` and `5/4 · drop one` count them against `max_rows`; a card never takes a row (it
// sits outside the cap) and its reach delta is labelled where it goes (`at R3`, the catalogue's `insert_at`). §6: the unlock
// shelf refetches when a rule edit crosses `max_rows` (`app.onShelf`); a supply line has its own `×`; a free line reads `· kennel`.
// Cut 13 §2: a new heir's trait is chosen — while `Lineage.trait_offer` holds two names the strip shows two chips beside `♟3`
// (`brave | curious`, the chosen one `on`, each with its rule as a small under-label); a tap is `setTrait(name)`; the chips
// vanish once the offer is empty (the send took it).
// Cut 16 §2: beside them, while `Lineage.class_offer` stands, a chip per owned class with its signature verb (`rogue · vanish`;
// one not yet open reads `ranger · mark L7`); the chosen one `on`; a tap is `setClass(name)` (it sticks until changed). The chip
// row stands in for the class button while it is up.
import type { App, Mounted } from "../app";
import type { CageOption, Lineage, StartOption, SupplyEntry, UnlockInfo } from "../engine/types";
import { h, clear, flash, pct, replace, spanOf, twoTap } from "./dom";
import { heroBinding, renderEditor } from "./editor";
import { renderParty } from "./party";
import { renderForecast, renderShaft, signedPts } from "./forecast";
import { gem, portrait, renderBar, renderConsole, stud, tile } from "./frame";
import { revealed, type Step } from "./reveal";
import { openLedger } from "./party";
import { openChronicle } from "./chronicle";
import { stallLabel, classList, deltaClass, deltaLabel, deltaPts, goldAffordable, addCard, isCard, openOwnedSheet, openUnlockSheet, ownedRows, priceLabel, supplyCap, visible, vaultSlots, withRowsGate } from "./unlocks";
import { audio, biomeOf } from "../audio";
import { salvageValue } from "./salvage";
import { CLASS_VERBS } from "../engine/classes";
import { isFreeSupply, ownRowCount, verbLabel } from "./tokens";
import { openSheet, setPanelEscape } from "./sheet";
import { setBusyHost } from "./progress";
import { icon } from "./skin";

const SET_NAME_MAX = 12;
/** Cut 19 §1: the cage's preferences, the picker's order. */
const CAGE_PREFS = ["weapon", "armour", "potion", "scroll"];
/** Cut 19 §1: the last `cageForecast()` and what it was measured for (the set, the preference, the best) — the picker paints it at once. */
let cageMemo: { key: string; opts: CageOption[] } | null = null;
/** Cut 19 §1: an option's headline delta (`+36%`, the bank share's move when either panel banks, else the reach's); none on the
 *  current preference or a move that rounds to 0; `dim` inside its ±. */
/** QA 1a2a4a9 (O, P: "`armour +15%` — % of what?"; "the selected option never has a number"; "`scroll` stayed blank"): each option
 *  names what it measures — `bank +15%` (the bank share, when either panel banks) or `D7 +15%` (the reach at the option's depth);
 *  the current one its own level (`D7 73%`), a move that rounds to 0 `D7 +0%` (dim). */
export function cageDelta(o: Pick<CageOption, "current" | "depth" | "reach" | "bank" | "bank_delta" | "delta" | "pm">): { text: string; cls: string } | null {
  const banks = o.delta === o.bank_delta && (o.bank > 0 || o.bank - o.bank_delta > 0);
  const at = banks ? /* copy:label */ "bank" : `D${o.depth}`;
  if (o.current) return { text: `${at} ${Math.round((banks ? o.bank : o.reach) * 100)}%`, cls: "cur" };
  const d = Math.round(o.delta * 100);
  const pm = Math.round(o.pm * 100);
  // Cut 22 §4 (AG: "the red `−18%` confused me; is it a delta?"): a move is signed points in the delta look (`bank −18`), never a `%`
  return { text: `${at} ${signedPts(d)}`, cls: `dlt ${d > 0 ? "up" : d < 0 ? "down" : "flat"}${Math.abs(d) <= pm ? " flat" : ""}` };
}
/** Cut 21 §1: the last `startForecast()` and what it was measured for (the set, the start, the lit waystones, the best). */
let startMemo: { key: string; opts: StartOption[] } | null = null;
/** Cut 21 §1: a start's toll — the engine's, else the contract's `$10 × depth` (0 at D1). */
export const startToll = (d: number, o?: Pick<StartOption, "toll">): number => o?.toll ?? (d > 1 ? 10 * d : 0);
/** QA a946e04 (T: `start → D5 · $50` at $32, and the run began on D1 with no word): can the purse pay a start's toll now? The engine's
 *  word when it sends one (`StartOption.short`, `Lineage.start_payable` for the current start), else the gold against the toll. */
export function startShort(L: Pick<Lineage, "gold" | "start" | "start_payable" | "start_toll">, st: number, o?: Pick<StartOption, "toll" | "short">): boolean {
  const toll = st === (L.start ?? 1) && L.start_toll !== undefined ? L.start_toll : startToll(st, o);
  if (toll <= 0) return false;
  if (o?.short !== undefined) return o.short;
  if (st === (L.start ?? 1) && L.start_payable !== undefined) return !L.start_payable;
  return L.gold < toll;
}
/** Cut 17 §3: which step carves each console tile (the tile glints on its first appearance). */
const STEP_OF: Record<string, Step> = { edit: "edit", loadout: "loadout", unlocks: "unlocks", vault: "vault", forge: "forge", party: "party", ledger: "heirs", chronicle: "heirs" };
const ALL_KEY = "riddle.unlocks.all";
const unlocksAll = (): boolean => { try { return localStorage.getItem(ALL_KEY) === "1"; } catch { return false; } };
const setUnlocksAll = (on: boolean): void => { try { localStorage.setItem(ALL_KEY, on ? "1" : "0"); } catch { /* a per-viewer convenience */ } };
/** Cut 13 §2: each trait's one-line rule, ≤ 3 words (the core's: cowardly retreats under 50 % hp with foes in view; brave skips a
 *  retreat row vs one foe, once a floor — `holds a retreat` read as its opposite, QA 23ed91f; curious drinks an unknown when clear; greedy steps onto adjacent loot). */
/* copy:callout */
const TRAIT_RULE: Record<string, string> = { cowardly: "flees under 50%", brave: "skips a retreat", curious: "drinks unknowns", greedy: "grabs loot" };
/** QA a946e04 (S, T: `cowardly · flees under 50%` never fled): the trait's rule as the core states it (`Lineage.trait_rules`), else the table's. */
export const traitRule = (L: Pick<Lineage, "trait_rules">, t: string): string | undefined => { const r = L.trait_rules?.[t]; return (r && ruleShort(r)) ?? TRAIT_RULE[t]; };
/** The core's rule within the chip's three words: `backs off once a floor under 50%` → `backs off <50% 1×/floor`, `tries one unknown a
 *  floor` → `tries unknown 1×/floor`; undefined when it will not fit (the table's words stand). */
export function ruleShort(rule: string): string | undefined {
  const perFloor = /\b(once|one \w+) a floor\b/.test(rule);
  const t = rule.replace(/\bonce a floor\b/, "").replace(/\bone (\w+) a floor\b/, "$1").replace(/\bunder (\d+%)/, "<$1").replace(/\s+/g, " ").trim() + (perFloor ? /* copy:none */ " 1×/floor" : "");
  return t.split(/\s+/).filter((w) => /\p{L}/u.test(w)).length <= 3 ? t : undefined;
}
export const setName = (s: { name?: string }, i: number): string => (s.name ?? "").trim().slice(0, SET_NAME_MAX) || `${i + 1}`;

const SEND_ARM_MS = 800;
/** QA 92eb880 (M: "kept `sealed scroll?` … reappear as `summon ally scroll` with no line saying they were identified"): an identified
 *  item's flavour off the lineage's facts (`item:sealed=summon_ally` → `sealed`); undefined while unknown or unmatched. */
export function keptAs(facts: string[], it: { kind: string; known: boolean }): string | undefined {
  if (!it.known) return undefined;
  const f = facts.find((x) => x.startsWith("item:") && x.endsWith(`=${it.kind}`));
  return f ? f.slice(5, f.indexOf("=")).replace(/_/g, " ") : undefined;
}
export function renderCamp(app: App, highlight?: number): Mounted {
  // Cut 17: the frame — the bar (the strip: heir, `$`, `◆`, `★`, best, the stud; the wake's offers under it), the well (the
  // tablets and the depth shaft; the set tabs from the 5th heir; the panels over it), the console (the portrait, the command
  // card as revealed, the gem `send`)
  const bar = renderBar(app, { live: true });
  const strip = bar.el;
  const tabs = h("nav", { class: "tabs" });
  // editing (`app.editing`): the tablets carry the editor (chips, ▲▼, ×, `+`); off, each is one carved tablet
  const editor = renderEditor(heroBinding(app), highlight, {
    compact: () => !app.editing,
    onTablet: (i) => { app.editing = true; closePanel(); editor.refresh(); paintTiles(); flashRow(i); },
  });
  // Cut 19 §1: the cage is a camp decision — its own tablet under the rules (`cage → armour`), revealed once a cage was seen; the
  // tap opens the picker with each preference's forecast delta
  const cageTab = h("button", { class: "row tablet compact cage-tab", hidden: true, onclick: () => openCagePicker() });
  // Cut 21 §1: where the send starts — its own tablet beside the rules and the cage (`start → D9 · $90`), carved when the first
  // waystone lights; the tap opens the picker (D1 and each lit waystone, each with its forecast move and toll)
  const startTab = h("button", { class: "row tablet compact start-tab", hidden: true, onclick: () => openStartPicker() });
  const party = renderParty(app);
  const fc = renderForecast(app);
  const shaft = renderShaft(app, () => togglePanel("forecast"), () => revealed(app).has("gems"));
  const vault = h("section", { class: "vault" });
  const supplies = h("section", { class: "supplies" });
  const unlocks = h("section", { class: "unlocks" });
  // Cut 18 §4 (rater Z: "APPLY also sent the next heir immediately"): the death screen's gem and this one share the console's gem slot —
  // a camp opened on a lit tablet (a patch applied, `highlight`) keeps its send deaf for SEND_ARM_MS, so the tap (or a second one) that
  // applied never lands on `send`; the player sends
  const armedAt = performance.now() + (highlight !== undefined ? SEND_ARM_MS : 0);
  const send = gem({ label: /* copy:button */ "send", cls: "send", pulse: true, onclick: () => { if (performance.now() < armedAt) return; if (!app.overBudget) app.go({ kind: "watch" }); } });
  // Cut 10 §3: the rest chip says what it means all the time (`rest 20m · send skips`), no tap needed
  const rest = h("span", { class: "rest chip num" });
  // the engine's busy label (`forecast` · `offline`) in its own strip under the header (QA on 50bb162: it drew over `D4 ★0`)
  const busyStrip = h("div", { class: "busy-strip num" });
  // Cut 17 §2: the camp's secondary objects are panels over the well, one at a time, each opened by its console tile (the
  // forecast's by the shaft), each with a close stud; Escape closes the open one
  const PANELS: Record<string, HTMLElement> = { forecast: fc.el, loadout: supplies, unlocks, vault, party: party.el };
  // QA 23ed91f (K: "the SUPPLIES sheet covers R4's verb chip; you have to close it to edit R4"): a tap on the well around the panel
  // closes it (the stud and Escape still do)
  const panelHost = h("div", { class: "panel-host", onclick: (e: Event) => { if (e.target === panelHost) closePanel(); } });
  // a closed panel's content waits in the DOM, unrendered (its engine fetches keep painting it; it is not on screen, so not in the
  // page's text either — a rater's text view reads what the player sees)
  const panelStore = h("div", { class: "panel-store", hidden: true, inert: true }, ...Object.values(PANELS));
  let open: string | null = null;
  function closePanel(): void { if (!open) return; open = null; panelStore.append(...Object.values(PANELS)); panelHost.replaceChildren(); panelHost.classList.remove("open"); paintTiles(); }
  function togglePanel(name: string): void {
    if (open === name) { closePanel(); return; }
    if (open) panelStore.append(...Object.values(PANELS));
    open = name;
    panelHost.replaceChildren(h("section", { class: "panel", "data-panel": name }, stud(closePanel), h("div", { class: "panel-body" }, PANELS[name])));
    panelHost.classList.add("open");
    paintTiles();
  }
  setPanelEscape(() => { if (!open) return false; closePanel(); return true; });
  // the vista over the camp (the title art: the stair down into the Warrens), cropped to a band, framed
  const vista = h("div", { class: "vista", "aria-hidden": "true" });
  const well = h("div", { class: "well camp-well" }, busyStrip, vista, tabs, h("div", { class: "camp-main" }, h("div", { class: "tablets" }, editor.el, cageTab, startTab), shaft.el), h("div", { class: "rest-line" }, rest), shaft.vsEl);
  const face = portrait(app, { label: "" });
  const cons = renderConsole({ portrait: face.el, tiles: [], gem: send });
  const el = h("main", { class: "camp frame" }, strip, h("div", { class: "well-wrap" }, well, panelHost, panelStore), cons.el);
  setBusyHost(busyStrip);
  function flashRow(i: number): void { const r = editor.el.querySelector<HTMLElement>(`.row[data-i="${i}"]`); if (r) { flash(r, "hl", 1600); r.scrollIntoView({ block: "center" }); } }

  // QA 1a2a4a9 (P: "nothing on the camp shows what's packed — only opening SUPPLIES does"): the loadout tile carries the pack's count
  const withPack = (el: HTMLElement): HTMLElement => { const n = app.lineage.supplies?.length ?? 0; if (n) el.appendChild(h("span", { class: "pack-n num" }, `${n}/${supplyCap(app.lineage.unlocks)}`)); return el; };
  const withBadge = (el: HTMLElement, badge: HTMLElement | null): HTMLElement => { if (badge) { el.appendChild(badge); el.classList.add("badged"); } return el; };
  /** Cut 17 §1/§3: the command card as revealed — edit · loadout · unlocks · vault · forge · party · ledger · chronicle. */
  function paintTiles(): void {
    const R = revealed(app);
    const t = (id: string, label: string, ico: string, onclick: () => void, on = false): HTMLElement => tile({ id, label, icon: ico, onclick, on, fresh: R.fresh(STEP_OF[id]) });
    cons.setTiles([
      // QA 92eb880 (N: "the `edit` tile toggles: tapping it while editing closes the editor (I lost the next tap twice)"): it turns
      // editing on and stays lit; a second tap closes the open panel, never the editor
      R.has("edit") && t("edit", /* copy:button */ "edit", "edit", () => { closePanel(); if (!app.editing) { app.editing = true; editor.refresh(); } paintTiles(); }, app.editing),
      R.has("loadout") && withBadge(withPack(t("loadout", /* copy:button */ "loadout", "loadout", () => togglePanel("loadout"), open === "loadout")), repeatBadge()),
      R.has("unlocks") && t("unlocks", /* copy:button */ "unlocks", "unlocks", () => togglePanel("unlocks"), open === "unlocks"),
      R.has("vault") && t("vault", /* copy:button */ "vault", "vault", () => togglePanel("vault"), open === "vault"),
      R.has("forge") && t("forge", /* copy:button */ "forge", "forge", () => openForge(app)),
      R.has("party") && t("party", /* copy:button */ "party", "party", () => togglePanel("party"), open === "party"),
      R.has("heirs") && t("ledger", /* copy:button */ "ledger", "ledger", () => openLedger(app)),
      R.has("heirs") && t("chronicle", /* copy:button */ "chronicle", "chronicle", () => openChronicle(app)),
    ]);
    shaft.el.classList.toggle("on", open === "forecast");
  }

  function paintStrip(): void {
    const L = app.lineage; const lvl: { level: number; xp: number; next?: number } = L.classes?.[L.class] ?? { level: 1, xp: 0 };
    bar.paint();
    // Cut 13 §2: the offer as chips while it stands (the bar's plain trait otherwise); Cut 16 §2: the class chips beside them
    const traits = (L.trait_offer?.length ?? 0) >= 2
      ? h("span", { class: "chips traits" }, ...L.trait_offer!.map((t) => h("button", { class: `chip trait${t === L.trait ? " on" : ""}`, disabled: t === L.trait, onclick: () => void pickTrait(t), "aria-pressed": t === L.trait ? "true" : "false" },
          // QA 1a2a4a9 (O: "the ✓ is only visual; the text shows no selection"): the mark is text, not a CSS `::before`
          t === L.trait ? h("b", { class: "tick" }, "✓ ") : "", h("span", null, t), traitRule(L, t) ? h("small", { class: "rule dim" }, traitRule(L, t)) : "")))
      : "";
    const offer = (L.class_offer?.length ?? 0) >= 2;
    const classes = offer
      ? h("span", { class: "chips classes-offer" }, ...L.class_offer!.map((c) => h("button", { class: `chip cls-offer${c.class === L.class ? " on" : ""}`, disabled: c.class === L.class, "data-class": c.class, onclick: () => void app.setClass(c.class), "aria-pressed": c.class === L.class ? "true" : "false" },
          c.class === L.class ? h("b", { class: "tick" }, "✓ ") : "", h("span", null, c.class, " ", h("b", { class: "num" }, `L${c.level}`)),
          c.signature ? h("small", { class: `rule dim${c.level < c.opens ? " locked" : ""}` }, verbLabel({ v: c.signature }), c.level < c.opens ? ` ⊘L${c.opens}` : "") : "")))   // QA 1a2a4a9 (P: "`mark L7` under an L1 ranger"): locked until L7
      : "";
    replace(bar.offers, traits, classes);
    bar.offers.hidden = !traits && !classes;
    // the portrait: the class and its level, the xp under it; a tap opens the class picker once classes can be had (the chip
    // row stands in for it while the wake's class offer is up)
    const R = revealed(app);
    const picker = !offer && (R.has("edit") || R.has("unlocks"));   // from the first death (a second heir may take another class)
    const next = portrait(app, { hp: 1, cls: picker ? "cls" : "", onclick: picker ? () => pickClass() : undefined,
      label: h("span", { class: "plabel-in" }, h("span", null, L.class, " ", h("b", { class: "num" }, `L${lvl.level}`)),
        // QA 92eb880: the bar is the core's own ladder (`classes[c].next`, the XP the next level costs; 0 at the top: full)
        h("span", { class: "xp" }, h("span", { class: "fill", style: `width:${Math.round(Math.min(1, lvl.next ? lvl.xp / lvl.next : lvl.next === 0 ? 1 : 0) * 100)}%` }))) });
    face.el.replaceWith(next.el); face.el = next.el;
    paintRest();
  }
  /** Cut 13 §2: the tap picks the heir's trait (`setTrait`); an engine without it keeps the default (the first offered). */
  async function pickTrait(name: string): Promise<void> {
    if (!app.engine.setTrait) return;
    await app.mutate(() => app.engine.setTrait!(name));
  }
  // Cut 2 §1: camp rest remaining; `send` skips it, so the number just disappears
  function paintRest(): void {
    const restS = app.lineage.rest_left_s ?? 0;
    replace(rest, /* copy:callout */ `rest ${spanOf(restS)} · send skips`);
    rest.hidden = restS <= 0;
  }
  // Cut 9 §10: each kind shows its ladder — `sword · salvaged 3/5 → craftable` (the engine's `next` rung); at the top, the count alone
  function openForge(app2: App): void {
    openSheet(() => {
      const L = app2.lineage; const rows = Object.entries(L.forge ?? {}).sort((a, b) => b[1].salvaged - a[1].salvaged);
      const head = h("div", { class: "lrow head" }, h("span", { class: "k" }, ""), h("span", null, ""), /* copy:label */ ...["craft", "tier"].map((s) => h("span", { class: "dot-h" }, s)));
      // the sheet's title (QA on 952e306: "forge: 'CRAFT TIER' header only"); with nothing salvaged yet, one dim line says so
      // instead of bare headers (QA on 50bb162: "FORGE sheet shows only the headers")
      if (!rows.length) return h("div", { class: "sheet-body ledger forge" }, h("div", { class: "label" }, /* copy:label */ "forge"), h("div", { class: "empty-line dim" }, /* copy:callout */ "nothing salvaged"));
      return h("div", { class: "sheet-body ledger forge" }, h("div", { class: "label" }, /* copy:label */ "forge"), head, ...rows.map(([kind, f]) => h("div", { class: "lrow" },
        h("span", { class: "k" }, kind.replace(/_/g, " ")),
        h("span", { class: "ladder num dim" }, /* copy:label */ "salvaged", " ", f.next ? h("span", null, `${f.salvaged}/${f.next.need}`, " → ", h("span", { class: "rung" }, f.next.label.replace(/_/g, " "))) : `${f.salvaged}`),
        h("span", { class: `dot${f.craftable ? " on" : ""}` }, f.craftable ? "⚒" : "·"), h("span", { class: `dot num${f.tier ? " on" : ""}` }, f.tier ? `+${f.tier}` : "·"))));
    });
  }
  // Cut 2 §4: whatever the lineage and the unlock catalogue provide (fighter · rogue · ranger · caster).
  // Cut 5 §6: each row carries the class's verb ladder as chips (`L1 shield bash · L3 cleave · …`), reached rungs lit.
  // Cut 8B §2: a class not yet owned shows its door (`◆0 · bank once`); once open it is taken from here (the rogue is
  // free at the first bank), so a second class is one tap from the strip.
  function pickClass(): void {
    openSheet((close) => {
      const L = app.lineage; const grid = h("div", { class: "classes" });
      const paint = (cat: Parameters<typeof classList>[1]): void => {
        clear(grid);
        for (const { cls, owned, level } of classList(L, cat)) {
          const ladder = Object.entries(CLASS_VERBS[cls] ?? {}).flatMap(([l, vs]) => vs.map((v) => h("span", { class: `chip rung num${Number(l) <= level && owned ? " on" : ""}` }, `L${l} `, verbLabel({ v }))));
          const u = owned ? undefined : cat?.find((x) => x.id === cls);
          const door = u ? h("small", { class: "num dim door" }, u.cost ? ` ◆${u.cost}` : "", u.needs ? `${u.cost ? " · " : " "}${u.needs.replace(/_/g, " ")}` : "") : "";   // no `◆0` (QA 23ed91f)
          const take = async (): Promise<void> => { if (u && !(await app.buy(cls))) return; void app.setClass(cls); close(); };
          grid.appendChild(h("div", { class: "class-row" },
            h("button", { class: `chip verb${cls === L.class ? " on" : ""}${owned || u?.available ? "" : " off"}`, disabled: !(owned || u?.available), onclick: () => void take() }, cls, " ", h("b", { class: "num" }, `L${level}`), door),
            ladder.length ? h("div", { class: "chips ladder" }, ...ladder) : ""));
        }
      };
      paint(unlockCat);
      if (!unlockCat) void app.engine.unlocks().then((cat) => paint(cat)).catch(() => { /* ladders only */ });
      return h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "class"), grid);
    });
  }
  // Cut 5 §6: sets carry a player-typed name (≤ 12 chars, the game's only free text; default `1 · 2 · 3`); ✎ on the active tab renames.
  // An unnamed set's tab reads `set 2 · 0` — the word, then the row count small (QA on 50bb162, the fourth reader of `2 0` as a
  // party or class count); a named one keeps `fighter 2`
  function paintTabs(): void {
    clear(tabs);
    // Cut 17 §3: the set tabs are carved with the 5th heir (a fresh lineage writes one set)
    tabs.hidden = !revealed(app).has("heirs") && app.active === 0;
    if (tabs.hidden) return;
    app.sets.forEach((s, i) => {
      const named = !!(s.name ?? "").trim();
      tabs.appendChild(h("button", { class: `tab num${i === app.active ? " on" : ""}`, onclick: () => app.selectSet(i) },
        named ? setName(s, i) : /* copy:label */ `set ${i + 1}`, h("small", { class: "dim" }, ` · ${ownRowCount(s.rows)}`)));   // `fighter · 3` like `set 2 · 0` (QA on 56f2a1d: `fighter 3` read as a hero number); own rows, the editor's `6/6` (QA on 3d71c33: `fighter · 8` beside `6/6 · 3 cards`)
      if (i === app.active) tabs.appendChild(h("button", { class: "tab edit", onclick: () => renameSet(i) }, "✎"));
    });
  }
  function renameSet(i: number): void {
    openSheet((close) => {
      const input = h("input", { class: "name-input", type: "text", maxlength: SET_NAME_MAX, autocomplete: "off", spellcheck: "false", value: app.sets[i].name ?? "", placeholder: `${i + 1}` });
      const commit = (): void => { app.renameSet(i, input.value); close(); };
      input.addEventListener("keydown", (e) => { if (e.key === "Enter") { e.preventDefault(); commit(); } });
      setTimeout(() => input.focus(), 0);
      return h("div", { class: "sheet-body" }, h("div", { class: "label row-label" }, /* copy:label */ "name"), input, h("button", { class: "btn primary wide", onclick: commit }, /* copy:button */ "ok"));
    });
  }
  function paintVault(): void {
    const L = app.lineage; const slots = vaultSlots(L.unlocks);
    clear(vault);
    vault.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "vault", " ", h("span", { class: "num dim" }, `${L.vault.length}/${slots}`)));   // Cut 17: `forge` is its console tile
    const chips = h("div", { class: "chips" });
    for (const it of L.vault) {
      const on = app.loadout.includes(it.id);
      chips.appendChild(h("button", { class: `chip item${on ? " on risk" : ""}`, onclick: () => {
        app.setLoadout(on ? app.loadout.filter((x) => x !== it.id) : [...app.loadout, it.id]);
      } }, on ? "⚠ " : "", it.label, keptAs(L.facts, it) ? h("small", { class: "dim flav" }, ` · ${keptAs(L.facts, it)}`) : ""));
      if (on) {
        const ins = (L.insured ?? []).includes(it.id);
        const price = Math.ceil(salvageValue(it.kind, "bank") * 10 / 4);
        // QA e75ec29 (R: "an unlabelled `$75` chip; one tap charged $75"): `insure $75`, a second tap pays
        chips.appendChild(ins ? h("button", { class: "chip mini on", disabled: true }, /* copy:label */ "insured")
          : twoTap(/* copy:button */ `insure $${price}`, /* copy:button */ `ok $${price}`, () => void app.mutate(() => app.engine.insure(it.id)), { class: "chip mini insure", disabled: L.gold < price }));
      }
    }
    // an empty slot is a plain marker, never a tap target (QA on 952e306: "vault slot '·' tap: nothing happened")
    for (let i = L.vault.length; i < slots; i++) chips.appendChild(h("span", { class: "chip empty", "aria-hidden": "true" }, ""));   // an empty slot is an empty chip (QA on 56f2a1d: `·` read as a chip that says `·`)
    vault.appendChild(chips);
    // keep preference for offline exits
    // QA 23ed91f: two rows that cannot be confused — `home` (what an unwatched exit keeps for the vault) and `cage` (what an
    // unanswered cage in the dungeon takes); K set `vault potion` as "what the home vault keeps"
    const prefs = h("div", { class: "chips prefs home" }, h("span", { class: "dim" }, /* copy:label */ "home"),
      /* copy:label */ ...[["best_weapon", "weapon"], ["best_armour", "armour"], ["none", "none"]].map(([id, lbl]) =>
        h("button", { class: `chip${(L.keep_pref ?? "best_weapon") === id ? " on" : ""}`, onclick: () => void app.mutate(() => app.engine.setKeepPref(id)) }, lbl)));
    // what the home pref does at an unwatched exit, on its row (the core's `keep_auto`, in order: `keeps armour · weapon`)
    const auto = L.keep_auto;
    // QA a946e04 (S: `VAULT 1/1 · axe · keeps armour`, the axe stayed through 17 runs): a keep replaces only a weaker item of its own
    // kind — a vault full of other kinds takes none, and the line says so (`keeps armour · vault full`)
    const CAT: Record<string, RegExp> = { weapon: /^(dagger|sword|axe|bow|spear|mace)$/, armour: /^(leather|mail|plate|scale)$/ };   // core defs.rs
    const blocked = !!auto?.length && L.vault.length >= slots && auto.every((k) => !L.vault.some((v) => CAT[k]?.test(v.kind) ?? v.kind === k));
    if (auto) prefs.appendChild(h("small", { class: "keep-auto dim num" }, auto.length ? /* copy:callout */ `keeps ${auto.join(" · ")}` : /* copy:callout */ "keeps nothing",
      blocked ? h("b", { class: "warn vault-full" }, /* copy:callout */ " · vault full") : ""));
    vault.appendChild(prefs);
    // Cut 19 §1: the cage's preference left this panel for its own tablet beside the rules (`cage → armour`)
  }
  function paintCage(): void {
    const on = revealed(app).has("cage");
    cageTab.hidden = !on;
    if (!on) return;
    replace(cageTab, h("span", { class: "rn num" }, icon("vault", "▣")),
      h("span", { class: "rtext" }, /* copy:rule_token */ "cage", h("span", { class: "arrow" }, " → "), app.lineage.vault_pref ?? "weapon"));
  }
  /** Cut 19 §1: the picker — the four preferences, each with its forecast delta against the current one (`armour +36%`); the tap sets it.
   *  The deltas are `cageForecast()` (three extra camp panels, memoised by the core; seconds in wasm): the last measure paints at once
   *  when it is this set's, `…` until the fresh one lands. */
  function openCagePicker(): void {
    const key = (): string => JSON.stringify([app.rules.rows, app.lineage.vault_pref ?? "weapon", app.lineage.best_depth]);
    openSheet((close) => {
      const list = h("div", { class: "chips cage-opts" });
      const paint = (opts: CageOption[] | null, pending: boolean): void => {
        const cur = app.lineage.vault_pref ?? "weapon";
        replace(list, ...CAGE_PREFS.map((p) => {
          const o = opts?.find((x) => x.pref === p); const d = o ? cageDelta(o) : null;
          return h("button", { class: `chip cage-opt${p === cur ? " on" : ""}`, "data-pref": p, onclick: async () => { close(); if (p !== cur) await app.mutate(() => app.engine.setVaultPref(p)); } },
            h("span", null, p), d ? h("b", { class: `num ${d.cls === "cur" ? "level cur" : `delta ${d.cls}`}` }, ` ${d.text}`) : pending && p !== cur ? h("small", { class: "num dim" }, " …") : "");
        }));
      };
      const k = key(), memo = cageMemo?.key === k ? cageMemo.opts : null;
      paint(memo, !memo && !!app.engine.cageForecast);
      if (!memo && app.engine.cageForecast) void app.engine.cageForecast().then((opts) => { cageMemo = { key: k, opts }; if (list.isConnected) paint(opts, false); }).catch((e) => { console.warn("cageForecast", e); if (list.isConnected) paint(null, false); });
      return h("div", { class: "sheet-body cage-picker" }, h("div", { class: "label row-label" }, /* copy:label */ "cage"), list);
    });
  }
  function paintStart(): void {
    const L = app.lineage, on = revealed(app).has("start");
    startTab.hidden = !on;
    if (!on) return;
    const st = L.start ?? 1, o = startMemo?.opts.find((x) => x.start === st && x.current), toll = L.start_toll ?? startToll(st, o);
    // QA a946e04 (T): a toll the purse cannot pay says so on the tablet — that send starts on D1 (`start → D5 · $50 short`)
    // QA a946e04 (core: the toll buys a pass for the night): a held pass reads `pass`, not a toll the next send will not pay
    const pass = st > 1 && (L.start_pass === true || o?.pass === true);
    const short = !pass && startShort(L, st, o);
    startTab.classList.toggle("short", short);
    replace(startTab, h("span", { class: "rn num" }, icon("depth", "▼")),
      h("span", { class: "rtext" }, /* copy:rule_token */ "start", h("span", { class: "arrow" }, " → "), h("span", { class: "num" }, `D${st}`),
        pass ? h("small", { class: "num toll pass dim" }, /* copy:rule_token */ " · pass")
          : toll > 0 ? h("small", { class: `num toll${short ? " short warn" : " dim"}` }, short ? /* copy:rule_token */ ` · $${toll} short` : ` · $${toll}`) : ""));
  }
  /** Cut 21 §1: the start picker — D1 and each lit waystone, each with its forecast move against the current start (`bank +12%`, the
   *  cage picker's measure) and its toll (`D9 · bank +12% · $90`); the tap is `setStart`. The moves are `startForecast()` (extra camp
   *  panels, memoised by the core; seconds in wasm): the last measure paints at once when it is this set's, `…` until one lands. */
  /** The current start's option with the camp forecast's own numbers (`app.lastForecast`, the shaft's and the panel's) when it is
   *  this set's: its bank share, its reach at the option's depth. */
  function currentFromPanel(o: StartOption): StartOption {
    const f = app.lastForecast; if (!f?.ends || (f.start ?? app.lineage.start ?? 1) !== o.start) return o;
    const r = f.depths.find((x) => x.depth === o.depth)?.reach;
    return { ...o, bank: f.ends.bank, reach: r ?? o.reach, pm: f.ends.pm ?? o.pm, death: f.ends.death };
  }
  function openStartPicker(): void {
    const L0 = app.lineage;
    const key = (): string => JSON.stringify([app.rules.rows, app.lineage.start ?? 1, app.lineage.waystones ?? [], app.lineage.best_depth]);
    openSheet((close) => {
      const list = h("div", { class: "chips start-opts" });
      const paint = (opts: StartOption[] | null, pending: boolean): void => {
        const cur = app.lineage.start ?? 1;
        const starts = [...new Set([1, ...(app.lineage.waystones ?? L0.waystones ?? [])])].sort((a, b) => a - b);
        replace(list, ...starts.map((st) => {
          const o = opts?.find((x) => x.start === st); const toll = startToll(st, o);
          // QA a946e04 (T: `START D1 · bank 86%` beside the panel's `bank 85%`): the current start's own level is the camp forecast's —
          // the number the shaft and the panel show — never a second measure of the same set
          const oo = o ? (o.current ? currentFromPanel(o) : o) : undefined;
          const d = oo ? cageDelta(oo) : null, death = oo?.death;
          // QA a946e04 (T): a toll the purse cannot pay dims its option and says so (`D5 · $50 short`); the current one stays lit
          const pass = st > 1 && (o?.pass === true || (st === cur && app.lineage.start_pass === true));
          const short = !pass && startShort(app.lineage, st, o);
          return h("button", { class: `chip start-opt${st === cur ? " on" : ""}${short ? " off short" : ""}`, "data-start": st, disabled: short && st !== cur,
            onclick: async () => { close(); if (st !== cur && app.engine.setStart) await app.mutate(() => app.engine.setStart!(st)); } },
            h("span", { class: "num" }, `D${st}`),
            d ? h("b", { class: `num ${d.cls === "cur" ? "level cur" : `delta ${d.cls}`}` }, ` · ${d.text}`) : pending && st !== cur ? h("small", { class: "num dim" }, " …") : "",
            // Cut 22 §4 (AG: "`D9 · bank +3% · $90` — but the shaft then says death 61%"): the start's death share beside its bank move
            d && death !== undefined ? h("span", { class: `num start-death${death >= 0.5 ? " warn" : ""}` }, /* copy:callout */ ` · death ${pct(death)}`) : "",
            pass ? h("small", { class: "num toll pass" }, /* copy:callout */ " · pass") : toll > 0 ? h("small", { class: `num toll${short ? " warn" : ""}` }, short ? /* copy:callout */ ` · $${toll} short` : ` · $${toll}`) : "");
        }));
      };
      const k = key(), memo = startMemo?.key === k ? startMemo.opts : null;
      paint(memo, !memo && !!app.engine.startForecast);
      if (!memo && app.engine.startForecast) void app.engine.startForecast().then((opts) => { startMemo = { key: k, opts }; if (list.isConnected) paint(opts, false); paintStart(); }).catch((e) => { console.warn("startForecast", e); if (list.isConnected) paint(null, false); });
      return h("div", { class: "sheet-body start-picker" }, h("div", { class: "label row-label" }, /* copy:label */ "start"), list);
    });
  }
  /** Cut 19 §3: the loadout repeats by default — the tile carries `repeat · $120` (the kinds the next send re-packs, at the shelf's
   *  price); a tap on it clears the repeat (`setRestock(false)`, the shelf refunded), `repeat off` a tap turns it back on. */
  function repeatBadge(): HTMLElement | null {
    const L = app.lineage;
    if (!app.engine.setRestock || L.repeat === undefined) return null;
    const on = L.repeat !== false;
    if (on && !(L.repeat_kinds?.length)) return null;   // nothing to re-pack
    return h("span", { class: `repeat-badge num${on ? " on" : ""}${on && L.repeat_short?.length ? " short" : ""}`, role: "switch", "aria-checked": on ? "true" : "false", "data-repeat": on ? "1" : "0",
      onclick: (e: Event) => { e.stopPropagation(); void app.mutate(() => app.engine.setRestock!(!on)); } },
      // QA 1a2a4a9 (P: "the restock was skipped with no word"): a re-pack the purse could not pay reads so on the tile
      // QA a946e04 (T: "`repeat · $40` reads like a price to pay"; its tap refunded $40): the badge is a switch and reads as one —
      // `repeat on · $40` (the tap turns it off and refunds the re-packed shelf) / `repeat off`
      on ? (L.repeat_short?.length ? /* copy:callout */ "repeat on · short" : /* copy:callout */ `repeat on · $${L.repeat_gold ?? 0}`) : /* copy:callout */ "repeat off");
  }
  function paintSupplies(): void {
    const L = app.lineage; const picks = L.supplies ?? []; const cap = supplyCap(L.unlocks); const full = picks.length >= cap;
    clear(supplies);
    supplies.appendChild(h("div", { class: "label row-label" }, /* copy:label */ "supplies", " ", h("span", { class: "num dim" }, `${picks.length}/${cap}`)));
    const chips = h("div", { class: "chips" });
    // Cut 12 §6: each line carries its own `×` (the header's cleared the whole shelf: "I lost the leash"); a free line reads `· kennel`
    // QA 92eb880 (M, N: "`leash · kennel` — kennel?"; "× drops the free leash at once, no undo; re-buying costs $30"): a free line reads
    // `· free`, and its `×` takes two taps — the first arms it (`drop`), the second within 3 s drops it
    for (const p of picks) {
      const free = isFreeSupply(L, p);
      let armed = 0;
      const x: HTMLButtonElement = h("button", { class: "x", onclick: () => {
        if (!free || (armed && performance.now() - armed < 3000)) { void app.dropSupply(p.id); return; }
        armed = performance.now(); x.classList.add("armed"); replace(x, /* copy:button */ "drop");
        setTimeout(() => { if (x.isConnected) { armed = 0; x.classList.remove("armed"); replace(x, "×"); } }, 3000);
      } }, "×");
      // Cut 21 §2: a line an exit shelved reads `· found` (packed free); with the repeat on, a kind no row names reads `· no row` — the
      // next send will not re-buy it (the core's narrowed `repeat_kinds`)
      const noRow = !free && !p.found && L.repeat !== false && L.repeat_kinds !== undefined && !L.repeat_kinds.includes(p.kind);
      chips.appendChild(h("span", { class: `chip item on${noRow ? " no-row" : ""}` }, p.label, free ? h("small", { class: "dim found" }, /* copy:callout */ " · free") : p.found ? h("small", { class: "dim found shelf" }, /* copy:callout */ " · found") : "",
        noRow ? h("small", { class: "dim no-row" }, /* copy:callout */ " · no row") : "", x));
    }
    supplies.appendChild(chips);
    // the shop from the last catalogue at once, the engine's replacing it when it arrives (QA B on 952e306: "while FORECAST
    // shows '…' the SUPPLIES shop chips are gone"); the gold and the slots are read live either way
    const shop = (cat: SupplyEntry[]): void => {
      for (const b of [...chips.querySelectorAll(".chip.buy")]) b.remove();
      for (const e of cat) {
        const can = !full && !e.needs && L.gold >= e.price;
        // Cut 10 §3: a greyed supply says why under its price — the slots, the engine's gate, or the gold missing
        const why = full ? /* copy:callout */ `${picks.length}/${cap} slots` : e.needs ? e.needs.replace(/_/g, " ") : L.gold < e.price ? /* copy:callout */ `$${e.price - L.gold} short` : "";
        chips.appendChild(h("button", { class: `chip buy${can ? "" : " off"}`, disabled: !can, onclick: () => void app.mutate(() => app.engine.buySupply(e.kind)) },
          h("span", { class: "buy-main" }, h("span", null, e.label, " ", h("b", { class: "num gold" }, `$${e.price}`)), why ? h("small", { class: "why num dim" }, why) : "")));
      }
    };
    if (app.supplyCat.length) shop(app.supplyCat);
    const gen = ++supplyGen;
    void app.engine.supplyCatalogue().then((cat) => {
      if (gen !== supplyGen) return;
      app.supplyCat = cat; shop(cat);
    }).catch((e) => console.warn("catalogue", e));
  }
  let supplyGen = 0, unlockGen = 0;
  let unlockCat: Parameters<typeof classList>[1];
  function paintUnlocks(): void {
    const gen = ++unlockGen;
    // the shelf from the last catalogue at once (QA B on 952e306: "the whole UNLOCKS list is gone"); the engine's replaces it
    if (!unlockCat && app.unlockCat.length) { unlockCat = app.unlockCat; paintFrom(unlockCat); }
    void app.engine.unlocks().then((fresh) => {
      if (gen !== unlockGen) return;
      // QA 92eb880 (M: "the reach line vanishes after buying +1 row"; N: "the tile list reshuffles twice within ~4 s"): until the new
      // deltas land, each card keeps its last measured delta (the shelf's order and lines hold still); prices and gates are the fresh ones
      const prev = new Map((unlockCat ?? []).filter((u) => u.delta !== undefined).map((u) => [u.id, u]));
      const cat = fresh.some((u) => u.delta !== undefined) ? fresh : fresh.map((u) => { const p = prev.get(u.id); return p && !u.owned ? { ...u, delta: p.delta, pm: p.pm, stall: u.stall ?? p.stall, insert_at: p.insert_at ?? u.insert_at, situation: u.situation ?? p.situation, auto_insert: u.auto_insert ?? p.auto_insert } : u; });   // QA a946e04: the measured place rides with its measure (the sheet read `joins at R4`, the buy went in at R2)
      unlockCat = cat; app.unlockCat = cat;
      if (cat.some((u) => u.owned && u.rows?.length)) editor.refresh();   // Cut 6 §6: `[card]` chips open their rows once the catalogue is here
      // forecast deltas arrive later (0.3–2 s of sims); repaint once with them, never blocking the shelf
      if (!fresh.some((u) => u.delta !== undefined)) {
        // N (QA 92eb880: a tile read `$450` while its sheet charged `$562` after a gold buy): the deltas' catalogue keeps its deltas only —
        // the price, the gate and ownership are always the fresh catalogue's
        const byId = new Map(fresh.map((u) => [u.id, u]));
        void app.engine.unlockDeltas().then((withDeltas) => { if (gen === unlockGen && withDeltas.some((u) => u.delta)) {
          const merged = withDeltas.map((u) => { const f = byId.get(u.id); return f ? { ...f, delta: u.delta, pm: u.pm, stall: u.stall ?? f.stall, insert_at: u.insert_at ?? f.insert_at, situation: u.situation ?? f.situation, auto_insert: u.auto_insert ?? f.auto_insert } : u; });   // QA a946e04: the measured `auto_insert` rides the merge (it was dropped: every card inserted)
          unlockGen++; unlockCat = merged; app.unlockCat = merged; paintFrom(merged);
        } }).catch(() => { /* deltas are optional */ });
      }
      paintFrom(cat);
    }).catch((e) => console.warn("unlocks", e));
  }
  function paintFrom(cat: UnlockInfo[]): void {
    {
      clear(unlocks);
      // Cut 10 §3: `+1 row` waits for the rows to fill (a client-side gate; the core may send the same `needs`)
      const list = visible(cat).map((u) => withRowsGate(u, app.ownRows(), app.vocab.max_rows));   // Cut 12 §1: own rows
      // Cut 6 §6: owned cards and automations stay on the shelf as chips that open their rows
      const owned = ownedRows(cat);
      if (!list.length && !owned.length) return;
      unlocks.appendChild(h("div", { class: "label" }, /* copy:label */ "unlocks"));
      const grid = h("div", { class: "cards" });
      // Cut 17 §3: a short list, not a wall — the next three (affordable first, then gated, then short of marks; the larger reach
      // gain, then the cheaper, then the catalogue's order); `more` opens the whole catalogue (remembered for this viewer)
      // Cut 18 §5: a tile the gold buys ranks with the ones the marks buy (and glows like them)
      const buyable = (u: typeof list[number]): boolean => u.available || goldAffordable(u, app.lineage.gold);
      const rank = (u: typeof list[number]): number => (buyable(u) ? 0 : u.gated ? 1 : 2);
      let next = list.map((u, i) => ({ u, i })).sort((a, b) => rank(a.u) - rank(b.u) || (b.u.delta ?? 0) - (a.u.delta ?? 0) || a.u.cost - b.u.cost || a.i - b.i).slice(0, 3).map((x) => x.u);
      // Cut 19 §3 (AA: "`+1 row` vanished"): a pinned unlock (the next `+1 row`) is always on the short list — it takes the last place
      const pins = list.filter((u) => u.pinned);
      if (pins.some((u) => !next.includes(u))) next = [...next.filter((u) => !u.pinned).slice(0, Math.max(0, 3 - pins.length)), ...pins];
      // QA 1a2a4a9 (O, P: "offers stay put between two opens"; the report's PENDING and the shelf differed): the core's short list
      // (`UnlockInfo.short`, from the lineage alone) when it sends one — the same three here and in the report
      if (list.some((u) => u.short !== undefined)) next = list.filter((u) => u.short);
      const shown = unlocksAll() || list.length <= 3 ? list : list.filter((u) => next.includes(u));
      for (const u of shown) {
        // `available` = prerequisite + fact gate + affordable (engine truth). Two dims: gated (the `needs` line
        // is what is missing, marks are there) and unaffordable.
        // Cut 4 §9: the forecast delta of buying (tactic cards), only when the catalogue carries one and it is not 0
        const d = deltaPts(u);   // Cut 13 §5: a delta within its ± paints as `reach ~0`
        // Cut 9 §2: the tap opens the sheet (rows, cost, needs, reach); the buy is on the sheet. A gated or unaffordable card
        // still opens it (the `needs` line is the answer), so nothing on the shelf is disabled.
        // Cut 12 §1: a card's delta is measured where it goes — `at R3` (the catalogue's `insert_at`), else `at end`
        const byGold = !u.available && goldAffordable(u, app.lineage.gold);
        grid.appendChild(h("button", { class: `card${u.available ? " buyable" : byGold ? " buyable gold-ok" : u.gated ? " gated" : " off"}`, onclick: () => openUnlockSheet(app, u) },
          // QA 92eb880 (N: "`AUTO: RESTOCK · ⊘ ◆1 more` while `$ buy` is enabled"): a marks shortfall the gold covers carries no `⊘`
          h("span", { class: "card-main" }, h("span", null, u.label), u.needs ? h("small", { class: "needs dim" }, u.gated && !byGold ? "⊘ " : "", u.needs.replace(/_/g, " ")) : "",
            d ? h("small", { class: `num delta ${deltaClass(u, d)}` }, deltaLabel(u, d, app.rules.rows.length)) : "",
            stallLabel(u) ? h("small", { class: "num delta down stall-risk" }, stallLabel(u)) : ""),   // QA 92eb880: the stall risk before buying
          h("span", { class: "num cost" }, priceLabel(u))));   // QA 23ed91f: a free door reads no `◆0`; Cut 18 §5: both prices, `◆3 · $450`
      }
      unlocks.appendChild(grid);
      if (shown.length < list.length) unlocks.appendChild(h("button", { class: "mini more", onclick: () => { setUnlocksAll(true); paintFrom(cat); } }, /* copy:button */ "more"));
      // an owned chip reads `card: thief guard · owned` (QA on 952e306: "bought card appears at the end with no cost"); its sheet
      // carries the title and, for a card whose row was dropped, `insert`
      // QA e75ec29: a card owned but off the set carries its own `add` (the buy put it in only where the core measured it helps)
      if (owned.length) unlocks.appendChild(h("div", { class: "chips owned" }, ...owned.flatMap((u) => [h("button", { class: "chip mini owned", onclick: () => openOwnedSheet(app, u) }, u.label, h("small", { class: "dim" }, /* copy:callout */ " · owned")),
        isCard(u) && !app.holdsCard(u.id) ? h("button", { class: "chip mini add-card", "data-card": u.id, onclick: () => addCard(app, u) }, /* copy:button */ "add") : ""])));
    }
  }
  // Cut 4 §1: `send` waits while the set is over budget (the editor shows which row to drop). Cut 6 §4: it says so: `6/5 · drop one`.
  function paintSend(): void {
    send.disabled = app.overBudget;
    send.classList.toggle("pulse", !app.overBudget);
    send.classList.toggle("small", app.overBudget);
    // Cut 22 (AG, AH: "the watch stayed on `fast 4×` from the earlier run — I hadn't noticed"): the remembered mode is kept (QA on
    // e0f87e7 asked for it) and the gem says it — `send` over a small `fast` — so the next run's pace is never a surprise
    const fast = app.watchMode === "fast";
    send.dataset.mode = app.watchMode;
    replace(send, app.overBudget ? /* copy:callout */ `${app.ownRows()}/${app.vocab.max_rows} · drop one`
      : fast ? h("span", { class: "send-l" }, /* copy:button */ "send", h("small", { class: "send-mode" }, /* copy:label */ "fast")) : /* copy:button */ "send");   // Cut 12 §1: own rows
    paintTabs();
    if (unlockCat) paintFrom(unlockCat);   // `+1 row` reads `⊘ fill rows` only while a free own row exists
  }
  function paintAll(): void { paintStrip(); paintTiles(); paintTabs(); paintVault(); paintCage(); paintStart(); paintSupplies(); paintUnlocks(); party.refresh(); editor.refresh(); paintSend(); audio.drone(biomeOf(app.lineage.best_depth + 1)); }
  paintAll();
  // Cut 12 §6: `+1 row ⊘ fill rows` is the engine's read of its own set — refetched once an edit crossed `max_rows`
  const off = app.onChange(paintAll), offRules = app.onRules(paintSend), offShelf = app.onShelf(paintUnlocks);
  const offShadow = app.onForecast(() => editor.paintShadow());   // QA 92eb880: a shadowed row's mark lands with the forecast of the rules now
  return { el, dispose: () => { off(); offRules(); offShelf(); offShadow(); fc.dispose(); shaft.dispose(); bar.dispose(); setPanelEscape(null); audio.drone(null); setBusyHost(null); } };
}
