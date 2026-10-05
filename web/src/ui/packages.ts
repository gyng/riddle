// Cut 30 §2 — the packages panel (docs/CUT30.md): pre-written rule bundles that level from runs. Three kinds of slot — the stance ×1
// (never empty), the tactic ×1 (×2 at its stage), the temperament ×1 (from heir 3) — each its chip `<name> L<n>` with the level's
// progress; the owned others under it, each priced on the camp's paired panel in one line (`Guarded · death −8`, `packageOptions()` on a
// background lane, `…` until it lands); a tap equips (free, instant). The compiled rows fold under `rows`: a row a same-role row above it
// always pre-empts is greyed with the winner (`Guarded wins`). Drills (`drill · Warlord`, revocable) and scars (`scarred ×3`) sit under
// the slots. The panel's head is the forecast's ONE headline (`reach D9 72%`). No tutorial text: a package's name and level are the copy.
//
// The pen (§2 "the pen is a late stage"): `penOpen` is the client's one read of it — the rule editor, the tablets, the trace and the
// verdict's detail wait for it. A lineage with no packages on the wire (an older core, a harness's literal set) has the pen open.
import "../cut30.css";
import type { App } from "../app";
import type { Lineage, Package, Packages, PkgOption } from "../engine/types";
import { h, replace, twoTap } from "./dom";
import { openWindow as openSheet } from "./sheet";
import { lowOf, share } from "./forecast";
import { rowLabel, verbLabel } from "./tokens";
import { packageIcon, foeSrc, icon, verbIcon } from "./skin";
import { itemIcon } from "./items";
import { sysOpen } from "./systems";
import { kw, kwHost } from "./tips";
import type { Term } from "./concepts";

/** The lineage climbs on packages (a Cut 30 core, not a harness's literal set). */
export const onPackages = (L: Pick<Lineage, "packages"> | undefined): boolean => !!L?.packages && !L.packages.literal;
/** The pen is open: the editor, the rule tablets, the trace and the verdict's detail. Open on an older core (no packages) and on a
 *  literal set; else the core's word (`Packages.pen_open`: the Mother met and 72 h, or 5 days). */
export const penOpen = (L: Pick<Lineage, "packages"> | undefined): boolean => !onPackages(L) || !!L!.packages!.pen_open;
/** The packages tile stands once there is a choice to make: a second stance arrived (the core's `stances` system), or anything but
 *  the school stance is owned. */
export const packagesShown = (L: Lineage | undefined): boolean => onPackages(L) && (sysOpen(L, "stances") || L!.packages!.all.some((p) => p.owned && p.id !== "steady" && p.kind === "stance"));

/** `Guarded L3` — a package's chip text. */
/* copy:label */
export const chipText = (p: Pick<Package, "name" | "level">): string => `${p.name} L${p.level}`;
/** The level's progress toward the next (0..1; 1 at the top). */
export const levelFill = (p: Pick<Package, "runs" | "next_at" | "level">): number => p.next_at ? Math.max(0, Math.min(1, p.runs / p.next_at)) : 1;
/** A beat's text as the report and the watch show it: the core's (`STEADY L3`, `DRILLED · Warlord`, `+Guarded`, `QUEST DONE · …`). */
export const beatText = (b: string): string => b;

/** The one-line price of a move on the paired panel: the term that moves most — `death −8`, `past +5`, `bank +3` — else `same`.
 *  `good`: whether the move helps (a death share that falls helps). */
/** A rough noise filter, not calibrated confidence. Panels can stop after five sends;
 *  the wire omits actual counts, so never assume more than five for this filter. */
export function priceOf(o: Pick<PkgOption, "d_past" | "d_death" | "d_bank" | "past" | "death" | "bank">, sims = PRICE_SIMS): { text: string; good: boolean | null; score: number } {
  /* copy:label */
  const terms: [string, number, boolean, number][] = [["deaths", o.d_death, true, o.death], ["deeper", o.d_past, false, o.past], ["full haul", o.d_bank, false, o.bank]];
  const variance = (p: number): number => Math.max(0.01, p * (1 - p));
  const band = (p: number, delta: number): number => {
    const base = Math.max(0, Math.min(1, p - delta));
    return 1.96 * Math.sqrt((variance(p) + variance(base)) / Math.max(1, Math.min(5, sims)));
  };
  const clear = terms.filter((t) => Math.abs(t[1]) > band(t[3], t[1]));
  if (!clear.length) return { text: "—", good: null, score: 0 };
  const [label, d, worse] = clear.reduce((a, b) => (Math.abs(b[1]) > Math.abs(a[1]) ? b : a));
  const pts = Math.round(d * 20) * 5;
  const good = (pts > 0) !== worse;
  return { text: `${label} ≈${pts > 0 ? "+" : "−"}${Math.abs(pts)}`, good, score: good ? Math.abs(pts) : -Math.abs(pts) };
}
/** Small first estimate: actual runs remain unchanged. */
export const PRICE_SIMS = 8;
/** The forecast's one headline: the reach of the next floor (`reach D9 72%`); null before a forecast. */
export function headline(app: App): string | null {
  const f = app.lastForecast; if (!f) return null;
  const next = Math.max(f.start ?? app.lineage.start ?? 1, app.lineage.best_depth + 1);
  const d = f.depths.find((x) => x.depth === next) ?? f.depths[f.depths.length - 1];
  return d ? /* copy:callout */ `floor ${d.depth} ${share(d.reach, lowOf(f))}` : null;
}

/** Cache replies and in-flight requests per app snapshot. A new lineage invalidates
 * every hero/simulation input, without reproducing Rust's input key in the client. */
type OptionsRead = { engine: App["engine"]; lineage: Lineage; inputs: string; selection: string; opts: PkgOption[] | null; promise: Promise<PkgOption[]> };
const optionReads = new WeakMap<App, OptionsRead>();
const optionInputs = (app: App): string => JSON.stringify([app.rules, app.loadout]);
const selectionKey = (app: App, choices: [string, number][]): string => app.engine.packageOptionsFor ? JSON.stringify(choices) : "all";
const currentRead = (app: App, read: OptionsRead, choices: [string, number][]): boolean => read.engine === app.engine && read.lineage === app.lineage && read.inputs === optionInputs(app) && read.selection === selectionKey(app, choices);
function measure(app: App, choices: [string, number][]): OptionsRead | null {
  if (!app.engine.packageOptions && !app.engine.packageOptionsFor) return null;
  const cached = optionReads.get(app);
  if (cached && currentRead(app, cached, choices)) return cached;
  const engine = app.engine;
  let read: OptionsRead;
  const promise = Promise.resolve().then(() => engine.packageOptionsFor ? engine.packageOptionsFor(PRICE_SIMS, choices) : engine.packageOptions!(PRICE_SIMS)).then((opts) => {
    read.opts = opts;
    return opts;
  }).catch((error: unknown) => {
    read.opts = [];
    if (optionReads.get(app) === read) optionReads.delete(app);
    throw error;
  });
  read = { engine, lineage: app.lineage, inputs: optionInputs(app), selection: selectionKey(app, choices), opts: null, promise };
  optionReads.set(app, read);
  return read;
}

/** A level bar under a chip (no words: the chip says the level). */
const levelBar = (p: Package): HTMLElement => h("span", { class: "lvl-bar", "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(levelFill(p) * 100)}%` }));

/** Opens the packages panel (a sheet anchored to `anchor`, the tile that opened it). */
export function openPackages(app: App, anchor?: HTMLElement | null): void {
  let dispose = (): void => {};
  openSheet((close) => {
    const body = h("div", { class: "sheet-body pkg-panel" });
    let reading: OptionsRead | null = null;
    let compare = false, details = false, tacticSlot: number | null = null;
    const choosing = new Set<string>();
    const equip = (p: Package, slot: number): void => { void app.mutate(() => app.engine.equipPackage!(p.id, slot), /* copy:callout */ p.name, true).then((ok) => { if (ok) choosing.delete(p.kind); paint(); }); };
    const paint = (): void => {
      const L = app.lineage, P = L.packages; if (!P) { close(); return; }
      const freeSlot = Math.min(tacticSlot ?? Math.max(0, (P.tactics ?? []).length), Math.max(0, (P.tactic_slots ?? 0) - 1));
      const selected: [string, number][] = P.all.filter((p) => p.owned && ((choosing.has("stance") && p.kind === "stance" && p.id !== P.stance) || (choosing.has("tactic") && p.kind === "tactic" && !(P.tactics ?? []).includes(p.id)))).map((p) => [p.id, p.kind === "tactic" ? freeSlot : 0]);
      const next = !compare ? null : reading && currentRead(app, reading, selected) ? reading : measure(app, selected);
      if (next !== reading) {
        reading = next;
        const settle = (): void => {
          if (!body.isConnected || reading !== next) return;
          // The town clock may advance repeatedly while this query runs. A stale
          // result must not start an endless series of replacement simulations.
          if (next && !currentRead(app, next, selected)) { compare = false; reading = null; }
          paint();
        };
        if (next) void next.promise.then(settle).catch(settle);
      }
      const opts = reading?.opts ?? null;
      const owned = (kind: string): Package[] => P.all.filter((p) => p.kind === kind && p.owned);
      const byId = new Map(P.all.map((p) => [p.id, p]));
      const optOf = new Map((opts ?? []).filter((o) => o.action === "equip").map((o) => [`${o.id}:${o.slot ?? 0}`, o]));
      /** An alternative: its chip and its one-line price (`death −8`); nothing while measuring or inside the noise (owner check: a row of
       *  `…` read as broken). */
      const alt = (p: Package, slot: number): HTMLElement => {
        const o = optOf.get(`${p.id}:${slot}`) ?? (opts ?? []).find((x) => x.id === p.id && x.action === "equip");
        const pr = o ? priceOf(o) : null;
        return h("button", { class: "chip pkg alt", "data-pkg": p.id, "data-kind": p.kind, onclick: () => equip(p, slot) },
          packageIcon(p.id), h("span", { class: "pkg-copy" }, h("span", { class: "pkg-name" }, chipText(p)), p.description ? h("small", { class: "pkg-description" }, p.description) : "", kwHost(h("small", { class: `pkg-price num${pr ? pr.good === null ? " flat" : pr.good ? " up" : " down" : " pending"}` }, pr && pr.good !== null ? pr.text : ""), "price")));   // docs/TOOLTIPS.md: the price's tip (blind check: `past +27` the most opaque words)
      };
      /** The alternatives best first (a clear gain, then the noise, then a clear loss), once priced; the catalogue's order until then. */
      const ranked = (ps: Package[], slot: number): Package[] => {
        if (!opts) return ps;
        const sc = (p: Package): number => { const o = optOf.get(`${p.id}:${slot}`) ?? (opts ?? []).find((x) => x.id === p.id && x.action === "equip"); return o ? priceOf(o).score : 0; };
        return [...ps].sort((a, b) => sc(b) - sc(a));
      };
      /** The next one still to come of a kind: dim, its trigger (`⊘ slay Warlord`). */
      const locked = (kind: string): HTMLElement | "" => {
        const p = P.all.find((x) => x.kind === kind && !x.owned); if (!p) return "";
        return h("span", { class: "chip pkg locked", "data-pkg": p.id, "aria-disabled": "true" }, h("span", { class: "pkg-name" }, p.name), h("small", { class: "pkg-price dim" }, `⊘ ${p.trigger ?? ""}`));
      };
      /** The worn package of a slot: its chip `Steady L3`, the level bar, a level bought with marks when the purse holds them. */
      const worn = (p: Package | undefined, kind: string, slot: number): HTMLElement => {
        const change = kind === "tactic" ? h("button", { class: "chip mini pkg-change", "data-edit-slot": slot, onclick: () => { tacticSlot = slot; choosing.add(kind); paint(); } }, p ? /* copy:button */ "change" : /* copy:button */ "choose tactic") : "";
        // (an open slot reads as a place to put one: an outlined socket, the chips under it fill it)
        if (!p) return h("div", { class: "pkg-slot empty", "data-kind": kind, "data-slot": slot }, change);
        const lv = p.level_price && L.marks >= p.level_price && app.engine.spendLevel
          ? twoTap(/* copy:button */ `◆${p.level_price} L${p.level + 1}`, /* copy:button */ `ok ◆${p.level_price}`, () => void app.mutate(() => app.engine.spendLevel!(p.id), /* copy:callout */ `L${p.level + 1}`, true).then(() => paint()), { class: "chip mini pkg-level", key: `lvl:${p.id}` })
          : "";
        const off = kind === "tactic" && app.engine.unequipPackage ? h("button", { class: "chip mini pkg-off", "aria-label": "remove", onclick: () => void app.mutate(() => app.engine.unequipPackage!(p.id), undefined, true).then(() => paint()) }, "×") : "";
        return h("div", { class: "pkg-slot", "data-kind": kind, "data-slot": slot },
          h("span", { class: "chip pkg on", "data-pkg": p.id }, packageIcon(p.id), h("span", { class: "pkg-copy" }, h("span", { class: "pkg-name" }, chipText(p)), p.description ? h("small", { class: "pkg-description" }, p.description) : "", levelBar(p))), change, lv, off);
      };
      const section = (label: string, kind: string, ...kids: (HTMLElement | "")[]): HTMLElement => h("section", { class: "pkg-sec", "data-kind": kind }, h("div", { class: "label pkg-head" }, kw(kind as Term, label)), ...kids);   // docs/TOOLTIPS.md: the slot's word is its keyword
      const choices = (kind: string, ...kids: HTMLElement[]): HTMLElement => h("div", { class: "pkg-choices", hidden: !choosing.has(kind) }, ...kids);
      const changeKind = (kind: string): HTMLElement => h("button", { class: "chip mini pkg-change", "data-change-kind": kind, "aria-expanded": String(choosing.has(kind)), onclick: () => { if (choosing.has(kind)) choosing.delete(kind); else choosing.add(kind); paint(); } }, /* copy:button */ "change");
      // the stance: worn, the others priced
      const stance = byId.get(P.stance);
      const stanceAlts = owned("stance").filter((p) => p.id !== P.stance);
      const secs: HTMLElement[] = [section(/* copy:label */ "combat style", "stance", h("div", { class: "pkg-equipped" }, worn(stance, "stance", 0), stanceAlts.length ? changeKind("stance") : ""), choices("stance", h("div", { class: "chips pkg-alts" }, ...ranked(stanceAlts, 0).map((p) => alt(p, 0)))))];
      // the tactics: one slot (two at its stage), each worn or empty; the owned others priced for the first empty slot (else slot 1)
      const slots = P.tactic_slots ?? 0;
      if (slots > 0 || owned("tactic").length) {
        const worn2 = Array.from({ length: Math.max(1, slots) }, (_, i) => worn(byId.get((P.tactics ?? [])[i] ?? ""), "tactic", i));
        const free = Math.min(tacticSlot ?? Math.max(0, (P.tactics ?? []).length), Math.max(0, slots - 1));
        const tAlts = owned("tactic").filter((p) => !(P.tactics ?? []).includes(p.id));
        secs.push(section(/* copy:label */ "extra tactics", "tactic", h("div", { class: "pkg-slots" }, ...worn2), choices("tactic", h("div", { class: "chips pkg-alts" }, ...ranked(tAlts, free).map((p) => alt(p, free))))));
      }
      // the temperament: from heir 3 — the wake's three cards while the offer stands (card 1 worn until one is picked)
      if (P.temperament_open || P.offer?.length) {
        const offer = (P.offer ?? []).map((id) => byId.get(id)).filter((p): p is Package => !!p);
        const cards = offer.length ? offer : owned("temperament");
        const personality = h("div", { class: "chips pkg-alts temper" }, ...cards.map((p) => h("button", { class: `chip pkg temper${p.id === P.temperament ? " on" : ""}`, "data-pkg": p.id, "aria-pressed": p.id === P.temperament ? "true" : "false",
          onclick: () => { if (p.id === P.temperament) return; void app.mutate(() => (P.offer?.includes(p.id) && app.engine.pickTemperament ? app.engine.pickTemperament(p.id) : app.engine.equipPackage!(p.id, 0)), /* copy:callout */ p.name, true).then(() => paint()); } },
          packageIcon(p.id), h("span", { class: "pkg-copy" }, h("span", { class: "pkg-name" }, chipText(p)), p.description ? h("small", { class: "pkg-description" }, p.description) : "", p.id === P.temperament ? levelBar(p) : ""))));
        if (offer.length) secs.push(section(/* copy:label */ "personality", "temperament", personality));
        else secs.push(section(/* copy:label */ "personality", "temperament", h("div", { class: "pkg-equipped" }, worn(byId.get(P.temperament ?? ""), "temperament", 0), cards.length > 1 ? changeKind("temperament") : ""), choices("temperament", personality)));
      }
      // drills and scars: `drill · Warlord` (one tap revokes, it stays revoked), `scarred ×3`
      const drills = P.drills ?? [];
      const scars = new Map(P.scars ?? []);
      const extra: HTMLElement[] = [];
      if (drills.length || scars.size) {
        const boss = (b: string): string => b.replace(/^goblin_/, "").replace(/_/g, " ").replace(/^./, (c) => c.toUpperCase());
        const lines = [...new Set([...drills.map((d) => d.boss), ...scars.keys()])].map((b) => {
          const d = drills.find((x) => x.boss === b), sc = scars.get(b) ?? d?.scar ?? 0;
          const face = foeSrc(b);
          const toggle = d ? h("button", { class: `chip mini drill${d.revoked ? "" : " on"}`, disabled: !app.engine.revokeDrill,
            "aria-pressed": String(!d.revoked), "aria-label": d.revoked ? /* copy:label */ `Enable ${boss(b)} tactic` : /* copy:label */ `Disable ${boss(b)} tactic`,
            onclick: (e: Event) => {
              const button = e.currentTarget as HTMLButtonElement; button.disabled = true;
              if (app.engine.revokeDrill) void app.mutate(() => app.engine.revokeDrill!(b, !d.revoked), undefined, true).then(() => paint()).finally(() => { if (button.isConnected) button.disabled = !app.engine.revokeDrill; });
            } }, d.revoked ? /* copy:button */ "Off" : /* copy:button */ "On") : null;
          const actions = (d?.rows ?? []).map((r) => {
            const card = r.verb.v === "tactic" ? app.unlockCat?.find((u) => u.id === r.verb.a) : undefined;
            const action = card?.carries ?? verbLabel(r.verb).replace(/^tactic /, "");
            const consumable = /^(drink|read|throw)$/.test(r.verb.v);
            const kind = r.verb.a?.split(",")[0] || (r.verb.v === "read" ? "scroll" : "potion");
            return h("span", { class: "counter-action", title: rowLabel(r) },
              consumable ? itemIcon({ kind, label: kind.replace(/_/g, " ") }, { size: "xs" }) : icon(verbIcon(r.verb.v) ?? "unlocks", "✦"), action);
          });
          return h("div", { class: `pkg-drill${d?.revoked ? " revoked" : ""}`, "data-boss": b },
            h("span", { class: "counter-face", "aria-hidden": "true" }, face ? h("img", { src: face, alt: "", draggable: "false" }) : icon("v_attack", "⚔")),
            h("div", { class: "counter-copy" }, h("b", { class: "counter-name" }, boss(b)),
              actions.length ? h("div", { class: "counter-actions" }, ...actions) : null,
              sc > 0 ? h("small", { class: "scar num" }, /* copy:label */ `Max HP −${sc}%`) : null), toggle);
        });
        extra.push(h("section", { class: "pkg-sec drills" }, h("div", { class: "label pkg-head" }, kw("drill", /* copy:label */ "boss counters")), ...lines));
      }
      // the compiled rows, folded: each its package, a shadowed one greyed with its winner
      extra.push(rowsFold(app, P));
      const more = h("details", { class: "pkg-advanced", open: details, ontoggle: (e: Event) => { details = (e.currentTarget as HTMLDetailsElement).open; } }, h("summary", null, /* copy:button */ "details"), ...extra, h("div", { class: "chips pkg-unlocks" }, locked("stance"), locked("tactic")));
      const compareButton = h("button", { class: "chip pkg-compare", disabled: compare && !!reading && !opts, onclick: () => { if (!choosing.has("stance") && !choosing.has("tactic")) { if (stanceAlts.length) choosing.add("stance"); else if (owned("tactic").some((p) => !(P.tactics ?? []).includes(p.id))) choosing.add("tactic"); } compare = true; reading = null; paint(); } }, compare && reading && !opts ? /* copy:button */ "comparing…" : /* copy:button */ "compare outcomes");
      const head = headline(app);
      replace(body, h("div", { class: "pkg-top" }, h("div", { class: "label row-label" }, kw("package", /* copy:label */ "tactics")), head ? h("b", { class: "pkg-headline num" }, kw("reach", head)) : ""), ...secs, compareButton, opts ? h("small", { class: "dim pkg-estimate", title: /* copy:tooltip */ "Small sample · minor differences unclear" }, /* copy:label */ "rough estimate") : "", more);
    };
    const changed = (): void => { compare = false; reading = null; paint(); };
    const offChange = app.onChange?.(changed), offRules = app.onRules?.(changed);
    dispose = () => { offChange?.(); offRules?.(); };
    paint();
    return body;
  }, { anchor, onClose: () => dispose() });
}

/** The compiled set under one `rows` fold: each row its words and its package (`Steady`), a row always pre-empted greyed with its winner
 *  (`Guarded wins`). Folded by default: the slots are the panel. */
function rowsFold(app: App, P: Packages): HTMLElement {
  const rows = app.lineage.sets?.[app.lineage.active_set ?? 0]?.rows ?? app.rules.rows;
  const src = P.rows ?? [];
  const stance = P.all.find((p) => p.id === P.stance)?.name ?? P.stance;
  const label = (i: number): string => src[i]?.label || (P.pen_open ? /* copy:label */ "custom rules" : stance);   // (before the pen no row is the pen's)
  const list = h("div", { class: "pkg-rows", hidden: true }, ...rows.map((r, i) => {
    const s = src[i]?.shadowed_by;
    return h("div", { class: `pkg-row${s !== undefined && s !== null ? " shadowed" : ""}`, "data-i": i },
      h("span", { class: "pkg-row-text num" }, rowLabel(r)), h("small", { class: "pkg-row-src dim" }, s !== undefined && s !== null ? /* copy:callout */ `${label(s)} wins` : label(i)));
  }));
  const n = src.filter((x) => x.shadowed_by !== undefined && x.shadowed_by !== null).length;
  const btn: HTMLButtonElement = h("button", { class: "details-fold pkg-rows-btn num", "aria-expanded": "false", onclick: () => { list.hidden = !list.hidden; btn.setAttribute("aria-expanded", String(!list.hidden)); btn.classList.toggle("on", !list.hidden); } },
    h("span", { class: "fold-mark", "aria-hidden": "true" }, "▸ "), /* copy:button */ "rules", h("small", { class: "dim" }, n ? /* copy:callout */ ` · ${rows.length} · ${n} greyed` : ` · ${rows.length}`));
  return h("section", { class: "pkg-sec rows" }, btn, list);
}

/** The camp's packages strip (the tablets' place before the pen): the worn packages as carved plaques, `Steady L3` with its level bar;
 *  a tap opens the panel once the panel is shown (before that it is a plaque, not a control). */
export function packagesStrip(app: App, opts: { ro?: boolean } = {}): { el: HTMLElement; paint(): void } {
  const el = h("div", { class: "pkg-strip" });
  const paint = (): void => {
    const L = app.lineage, P = L.packages;
    el.hidden = !onPackages(L) || penOpen(L);
    if (el.hidden || !P) return;
    const byId = new Map(P.all.map((p) => [p.id, p]));
    const worn = [byId.get(P.stance), ...(P.tactics ?? []).map((t) => byId.get(t)), P.temperament ? byId.get(P.temperament) : undefined].filter((p): p is Package => !!p);
    const live = packagesShown(L) && !opts.ro;
    replace(el, ...worn.map((p) => {
      const label = p.kind === "stance" ? /* copy:label */ "combat style" : p.kind === "temperament" ? /* copy:label */ "personality" : /* copy:label */ "extra tactic";
      const kids = [h("span", { class: "pkg-kind dim" }, kw(p.kind as Term, label)), h("span", { class: "pkg-name" }, chipText(p)), levelBar(p)];
      return live ? h("button", { class: "row tablet compact pkg-tab", "data-pkg": p.id, onclick: (e: Event) => openPackages(app, e.currentTarget as HTMLElement) }, ...kids)
        : h("div", { class: "row tablet compact pkg-tab plaque", "data-pkg": p.id }, ...kids);
    }));
  };
  paint();
  return { el, paint };
}
