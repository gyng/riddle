// Cut 30 §3 — the town on the camp (docs/TOWN.md): the scene (render/town.ts) in the well, a DOM hit target over everything one can
// tap (≥ 44 px, a hidden label for tests and screen readers), the markers (≤ 3: a sword over the forge when a kit step is affordable,
// a coin over the bank when a night's interest came in, a `!` rune over a building not yet opened), the staked plot's trigger on tap,
// and the building bar (the console's command card: one tile per building standing, in build order; tile and building open the same
// panel and share the badge). Day 0 is a camp — the mouth (tap = send), the tent (the hero), the crate (the pack); nothing else taps.
import "../town.css";
import type { App } from "../app";
import type { Lineage, ReturnReport } from "../engine/types";
import { createTownView, townState, BUILDINGS, type BuildingId, type PlotId, type TownState, type TownView } from "../render/town";
import { h, replace } from "./dom";
import { icon } from "./skin";
import { kitAffordable } from "./forge";
import { openSheet } from "./sheet";
import { tile } from "./frame";
import { audio } from "../audio";
import { questShown } from "./quest";
import { kwHost } from "./tips";
import type { Term } from "./concepts";
/** docs/TOOLTIPS.md: a building's tip (long-press / hover; its tap stays its panel) */
const HIT_TERM: Record<string, Term> = /* copy:none */ { crate: "pack", blacksmith: "forge", storehouse: "vault", kennel: "kennel", bank: "bank", board: "quest", staked: "track" };

/** what a target opens (the camp wires each to its panel or sheet) */
export type TownHooks = {
  send(): void;
  hero(anchor: HTMLElement): void;
  open(what: "loadout" | "vault" | "party", anchor: HTMLElement): void;
  forge(anchor: HTMLElement): void;
  quest(anchor: HTMLElement): void;
};
/* copy:label */
const LABEL: Record<string, string> = { mouth: "dungeon", tent: "hero", crate: "pack", blacksmith: "blacksmith", storehouse: "storehouse", kennel: "kennel", bank: "bank", staked: "next plot", board: "quest board" };
/** a building's tile on the bar: its id (the old console ids, kept: tests and badges key on them), icon and word */
/* copy:button */
export const BUILDING_TILE: Record<BuildingId, { id: string; icon: string; label: string; glyph: string }> = {
  blacksmith: { id: "forge", icon: "forge", label: "forge", glyph: "⚒" }, storehouse: { id: "vault", icon: "vault", label: "vault", glyph: "▣" },
  kennel: { id: "party", icon: "party", label: "kennel", glyph: "🐾" }, bank: { id: "bank", icon: "gold", label: "bank", glyph: "$" },
};
/** the buildings standing, in build order (null: a lineage with no town on the wire — the old console reveal stands) */
export const townBuilt = (L: Lineage): BuildingId[] | null => L.town ? L.town.buildings.map((b) => b.id).filter((id): id is BuildingId => (BUILDINGS as readonly string[]).includes(id)) : null;

type Store = { seen: string[]; opened: string[] };
const KEY = (L: Lineage): string => /* copy:none */ `riddle.town.${L.seed ?? 0}`;
function load(L: Lineage): Store | null { try { const s = localStorage.getItem(KEY(L)); return s ? JSON.parse(s) as Store : null; } catch { return null; } }
function save(L: Lineage, s: Store): void { try { localStorage.setItem(KEY(L), JSON.stringify(s)); } catch { /* a per-viewer convenience */ } }
/** a building's panel opened (its `!` rune goes) */
export function markOpened(L: Lineage, id: string): void { const s = load(L) ?? { seen: [], opened: [] }; if (!s.opened.includes(id)) { s.opened.push(id); save(L, s); } }

export type TownUi = { el: HTMLElement; view: TownView; paint(): void; send(after: () => void): void; dispose(): void; anchorOf(id: string): HTMLElement | null };
export function renderTown(app: App, hooks: TownHooks): TownUi {
  const el = h("div", { class: "town" });
  const hits = h("div", { class: "town-hits" });
  const tag = h("div", { class: "town-tag num", hidden: true, "aria-live": "polite" });
  el.append(hits, tag);
  const view = createTownView(el);
  // the store: what this viewer has seen built (a building first seen goes up: scaffold → built) and opened (its rune goes)
  let store = load(app.lineage);
  // (an old save's first camp shows its town standing: no scaffold on everything)
  if (!store) { store = { seen: townBuilt(app.lineage) ?? [], opened: townBuilt(app.lineage) ?? [] }; save(app.lineage, store); }
  // the absence's parties: the report just read (once per absence)
  const ab = app.lastAbsence && !app.lastAbsence.played ? app.lastAbsence : null;
  if (ab) ab.played = true;
  const absence: ReturnReport | null = ab?.report ?? null;
  let state: TownState | null = null;
  const btns = new Map<string, HTMLButtonElement>();
  let sending = false;
  let tagTimer = 0;

  function kitInfo(): { n: number; price?: number } {
    const n = kitAffordable(app.lineage);
    const price = (app.lineage.kit ?? []).filter((k) => k.next?.affordable).map((k) => k.next!.price).sort((a, b) => a - b)[0];
    return { n, price };
  }
  function paint(): void {
    const L = app.lineage;
    const s0 = load(L) ?? store!;
    state = townState(L, absence, { seen: s0.seen, opened: s0.opened, kit: kitInfo(), absenceNew: !!absence, board: questShown(L) });
    view.setState(state);
    // seen now (a re-mount does not raise it again)
    const built = state.buildings.map((b) => b.id);
    if (built.some((b) => !s0.seen.includes(b))) save(L, { ...s0, seen: [...new Set([...s0.seen, ...built])] });
    targets();
  }
  /** the hit targets: one button per thing one can tap, over its sprite (≥ 44 px), its label hidden */
  function targets(): void {
    const s = state; if (!s) return;
    const want: { id: string; plot: PlotId | "staked" }[] = [{ id: "mouth", plot: "mouth" }, { id: "tent", plot: "tent" }, { id: "crate", plot: "crate" },
      ...s.buildings.map((b) => ({ id: b.id, plot: b.id as PlotId })), ...(s.board ? [{ id: "board", plot: "board" as PlotId }] : [])];
    // the staked plot shows from day 0; it taps (its trigger) once the first building stands — day 0 keeps its four surfaces
    if (s.staked && s.stage >= 1) want.push({ id: "staked", plot: "staked" });
    for (const [id, b] of btns) if (!want.some((w) => w.id === id)) { b.remove(); btns.delete(id); }
    for (const w of want) {
      let b = btns.get(w.id);
      if (!b) {
        const label = w.id === "staked" ? `${LABEL.staked}` : LABEL[w.id] ?? w.id;
        b = h("button", { class: `town-hit hit-${w.id}`, "data-building": w.id, "aria-label": label, onclick: (e: Event) => tap(w.id, e) }, h("span", { class: "vh" }, label));
        if (HIT_TERM[w.id]) kwHost(b, HIT_TERM[w.id]!);
        btns.set(w.id, b); hits.appendChild(b);
      }
      // markers ride their building's target (one surface, the same panel)
      const mk = s.markers.filter((m) => m.at === w.id);
      const holder = b.querySelector(".town-markers") ?? b.appendChild(h("span", { class: "town-markers" }));
      replace(holder, ...mk.map((m) => h("span", { class: `town-marker mk-${m.kind}`, "data-marker": m.kind },
        icon(m.kind === "sword" ? "v_attack" : m.kind === "coin" ? "gold" : "alert", m.kind === "sword" ? "⚔" : m.kind === "coin" ? "$" : "!"), m.label ? h("b", { class: "num" }, m.label) : "")));
      b.dataset.markers = mk.map((m) => m.kind).join(" ");
    }
    layout();
  }
  function layout(): void {
    const s = state; if (!s) return;
    for (const [id, b] of btns) {
      const r = view.rectOf(id === "staked" ? "staked" : id as PlotId);
      if (!r) { b.hidden = true; continue; }
      b.hidden = false;
      // ≥ 44 px each way, centred on the sprite (a building's box trimmed to its body: the roof's sky is not the building)
      const w = Math.max(44, r.w * (id === "mouth" ? 0.6 : 0.86)), hh = Math.max(44, r.h * (id === "mouth" ? 0.7 : 0.8));
      const cx = r.x + r.w / 2, by = r.y + r.h;
      Object.assign(b.style, { left: `${Math.round(cx - w / 2)}px`, top: `${Math.round(by - hh + (hh > r.h ? (hh - r.h) / 2 : 0))}px`, width: `${Math.round(w)}px`, height: `${Math.round(hh)}px` });
    }
    if (!tag.hidden) placeTag();
  }
  view.onLayout(layout);
  function placeTag(): void {
    const r = view.rectOf("staked"); if (!r) { tag.hidden = true; return; }
    Object.assign(tag.style, { left: `${Math.round(r.x + r.w / 2)}px`, top: `${Math.round(r.y - 6)}px` });
  }
  function tap(id: string, e: Event): void {
    view.poke();
    const b = btns.get(id)!;
    const L = app.lineage;
    if (id === "mouth") { hooks.send(); return; }
    if (id === "staked") {
      const s = state?.staked; if (!s) return;
      // the next building and what raises it (`kennel · first tame`), on tap, a few seconds
      replace(tag, h("b", null, s.id), s.trigger ? h("span", { class: "dim" }, ` · ${s.trigger}`) : "");
      tag.hidden = false; placeTag(); clearTimeout(tagTimer); tagTimer = window.setTimeout(() => { tag.hidden = true; }, 3200);
      return;
    }
    const marker = (e.target as HTMLElement | null)?.closest?.(".town-marker") as HTMLElement | null;
    if (marker?.dataset.marker === "sword" && id === "blacksmith") { void buyKit(); return; }
    if ((BUILDINGS as readonly string[]).includes(id)) { markOpened(L, id); paint(); }
    if (id === "tent") hooks.hero(b);
    else if (id === "crate") hooks.open("loadout", b);
    else if (id === "storehouse") hooks.open("vault", b);
    else if (id === "kennel") hooks.open("party", b);
    else if (id === "blacksmith") hooks.forge(b);
    else if (id === "bank") openBank(app, b);
    else if (id === "board") hooks.quest(b);
  }
  /** the sword marker: one tap buys the cheapest kit step the purse can pay (the check-in's spend) */
  async function buyKit(): Promise<void> {
    const lad = (app.lineage.kit ?? []).filter((k) => k.next?.affordable).sort((a, b) => a.next!.price - b.next!.price)[0];
    if (!lad || !app.engine.buyKit) return;
    const ok = await app.mutate(() => app.engine.buyKit!(lad.slot), /* copy:callout */ "kit");
    if (ok) audio.cue("unlock");
  }
  function send(after: () => void): void {
    if (sending) return;
    sending = true;
    const reduce = document.documentElement.dataset.juice === "off" || matchMedia?.("(prefers-reduced-motion: reduce)").matches;
    void (reduce ? Promise.resolve() : view.walkIn(650)).then(() => { after(); });
    setTimeout(() => { sending = false; }, 2500);
  }
  paint();
  return { el, view, paint, send, anchorOf: (id) => btns.get(id) ?? null, dispose: () => { clearTimeout(tagTimer); view.dispose(); } };
}

/** The building bar's tile for a standing building (`forge` · `vault` · `kennel` · `bank`): the same panel as the building. */
export function buildingTile(id: BuildingId, o: { on?: boolean; fresh?: boolean; onclick: () => void }): HTMLButtonElement {
  const t = BUILDING_TILE[id];
  const b = tile({ id: t.id, label: t.label, icon: t.icon, glyph: t.glyph, on: o.on, fresh: o.fresh, onclick: o.onclick });
  b.dataset.building = id;
  return b;
}

/** Cut 30 §3: the bank — deposits earn ~2 % a night, capped at `bank_cap` (the core's); one tap each way. */
export function openBank(app: App, anchor?: HTMLElement | null): void {
  openSheet((close) => {
    const body = h("div", { class: "sheet-body bank-sheet" });
    const paint = (): void => {
      const L = app.lineage, T = L.town;
      if (!T) { close(); return; }
      const room = Math.max(0, T.bank_cap - T.bank), put = Math.min(L.gold, room);
      replace(body, h("div", { class: "label row-label" }, /* copy:label */ "bank"),
        h("div", { class: "bank-line num" }, h("b", { class: "gold" }, `$${T.bank}`), h("span", { class: "dim" }, ` / $${T.bank_cap}`)),
        h("div", { class: "bank-bar", "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(Math.min(1, T.bank / Math.max(1, T.bank_cap)) * 100)}%` })),
        T.interest > 0 ? h("div", { class: "num dim bank-interest" }, /* copy:callout */ `interest $${T.interest}`) : "",
        h("div", { class: "chips bank-acts" },
          h("button", { class: "chip bank-in", disabled: put <= 0 || !app.engine.bankDeposit, onclick: () => void act(() => app.engine.bankDeposit!(put)) }, /* copy:button */ `deposit $${put}`),
          h("button", { class: "chip bank-out", disabled: T.bank <= 0 || !app.engine.bankWithdraw, onclick: () => void act(() => app.engine.bankWithdraw!(T.bank)) }, /* copy:button */ `withdraw $${T.bank}`)));
    };
    const act = async (fn: () => Promise<Lineage>): Promise<void> => { if (await app.mutate(fn, /* copy:callout */ "bank") && body.isConnected) paint(); };
    paint();
    return body;
  }, { anchor });
}

/** The tent: the hero — the class, its level and xp (the class picker once classes can be had: the camp's). */
export function openHero(app: App, anchor?: HTMLElement | null): void {
  openSheet(() => {
    const L = app.lineage, lvl = L.classes?.[L.class] ?? { level: 1, xp: 0, next: undefined as number | undefined };
    const p = Math.min(1, lvl.next ? lvl.xp / lvl.next : 0);
    return h("div", { class: "sheet-body hero-sheet" }, h("div", { class: "label row-label" }, /* copy:label */ "hero"),
      h("div", { class: "hero-line num" }, h("b", null, L.class), " ", h("span", null, `L${lvl.level}`)),
      h("div", { class: "bank-bar xp-bar", "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(p * 100)}%` })));
  }, { anchor });
}

/** A camp panel opened from a building stands over it (its foot on the building's top), or under it when the room above is short. */
export function anchorPanel(panel: HTMLElement, anchor: HTMLElement, host: HTMLElement): void {
  const a = anchor.getBoundingClientRect(), r = host.getBoundingClientRect();
  // (only over a building on screen: the well scrolled to its rows leaves the town above, and a panel hung from it would be off screen)
  if (!a.height || !r.height || a.bottom <= r.top + 8 || a.top >= r.bottom - 8 || anchor.hidden) return;
  const above = a.top - r.top, below = r.bottom - a.bottom;
  panel.classList.add("town-anchored");
  if (above >= 220 || above >= below) Object.assign(panel.style, { top: "auto", bottom: `${Math.round(r.bottom - a.top + 6)}px`, maxHeight: `${Math.round(above - 12)}px` });
  else Object.assign(panel.style, { bottom: "auto", top: `${Math.round(a.bottom - r.top + 6)}px`, maxHeight: `${Math.round(below - 12)}px` });
}

/** dev/test inspection: `window.__town` while a camp stands */
export function exposeTown(ui: TownUi | null): void {
  const w = window as unknown as { __town?: unknown };
  if (!ui) { delete w.__town; return; }
  w.__town = { stats: () => ui.view.stats(), stress: (n: number) => ui.view.stress(n), setHour: (x: number | null) => ui.view.setHour(x), targets: () => [...ui.el.querySelectorAll<HTMLElement>(".town-hit")].filter((b) => !b.hidden).map((b) => { const r = b.getBoundingClientRect(); return { id: b.dataset.building, x: r.x, y: r.y, w: r.width, h: r.height, markers: b.dataset.markers ?? "" }; }) };
}
