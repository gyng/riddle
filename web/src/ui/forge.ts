// The forge shows the next starting-kit upgrade per slot. Future steps, salvage,
// and explicitly requested forecasts live under Details; purchases never wait for simulations.
import type { App } from "../app";
import type { KitLadder, Lineage } from "../engine/types";
import { h, replace } from "./dom";
import { openWindow as openSheet } from "./sheet";
import { moveOf } from "./forecast";
import { audio } from "../audio";
import { icon } from "./skin";
import { itemIcon, itemName, itemChip } from "./items";   // run-clear

/** Conservative wire snapshot: simulation inputs and displayed prices belong to this camp. */
const kitKey = (app: App): string => JSON.stringify([app.lineage, app.rules, app.loadout]);
type KitCache = { engine: App["engine"]; done: Map<string, KitLadder[]>; pending: Map<string, Promise<KitLadder[]>> };
const kitCaches = new WeakMap<App, KitCache>();
/** One explicit preview per state, isolated from other Apps/engines; four completed camps retained. */
export function measureKit(app: App): Promise<KitLadder[]> | null {
  const engine = app.engine;
  const query = engine.kitEstimates ?? engine.kitDeltas;
  if (!query || !(app.lineage.kit ?? []).some((k) => k.next)) return null;
  let cache = kitCaches.get(app);
  if (!cache || cache.engine !== engine) {
    cache = { engine, done: new Map(), pending: new Map() };
    kitCaches.set(app, cache);
  }
  const own = cache, key = kitKey(app);
  const done = own.done.get(key);
  if (done) { own.done.delete(key); own.done.set(key, done); return Promise.resolve(done); }
  const pending = own.pending.get(key);
  if (pending) return pending;
  // Schedule after registration so synchronous failures also release the pending request.
  const p = Promise.resolve().then(() => query.call(engine)).then((rows) => {
    if (app.engine === engine && kitKey(app) === key) {
      own.done.set(key, rows);
      if (own.done.size > 4) own.done.delete(own.done.keys().next().value!);
    }
    return rows;
  }).finally(() => { if (own.pending.get(key) === p) own.pending.delete(key); });
  own.pending.set(key, p);
  return p;
}
/* copy:label */
const SLOT_LABEL: Record<string, string> = { weapon: "weapon", armour: "armour", pack: "pack" };

/** Cut 23 §1: steps the purse can buy now (the tile's badge; the reveal ladder's `kit` step). */
export const kitAffordable = (L: Pick<Lineage, "kit">): number => (L.kit ?? []).filter((k) => k.next?.affordable).length;

/** The next step's measured move in the edits' form (Cut 24 §3, AL: `leather +1 · death +7` read as armour raising death): the depth
 *  first, then the ends that clear their ± — `D9 +7 · death −7`, `D9 ≈ ±4 · bank +6`; null until measured. `worse`: more is worse. */
export function kitTerms(n: NonNullable<KitLadder["next"]>): { label: string; text: string; dir: "up" | "down" | "flat"; worse: boolean }[] | null {
  if (n.delta === undefined) return null;
  const d = moveOf({ delta: n.delta, pm: n.pm });
  if (!d) return null;
  const out = [{ label: /* copy:callout */ `reach D${n.depth ?? "?"}`, text: d.text, dir: d.dir, worse: false }];   // docs/COPY.md pass 4: `D7 same` read "no idea"
  const bank = n.bank !== undefined ? moveOf({ delta: n.bank, pm: n.pm }) : null;
  const death = n.death !== undefined ? moveOf({ delta: n.death, pm: n.pm }) : null;
  if (bank && bank.dir !== "flat") out.push({ label: /* copy:label */ "bank", text: bank.text, dir: bank.dir, worse: false });
  if (death && death.dir !== "flat") out.push({ label: /* copy:label */ "death", text: death.text, dir: death.dir, worse: true });
  return out;
}
/** The move as one line (`D9 +7 · death −7`). */
export function kitMove(n: NonNullable<KitLadder["next"]>): string | null {
  const t = kitTerms(n);
  return t ? t.map((x) => `${x.label} ${x.text}`).join(" · ") : null;
}
/** A term's colour: good or bad, whichever way its sign points (a death that falls is good). */


export function openForge(app: App, anchor?: HTMLElement | null): void {
  let unChange: (() => void) | undefined, unRules: (() => void) | undefined;
  openSheet(() => {
    let state = kitKey(app), engine = app.engine, request = 0;
    const kit = h("div", { class: "kit simple-kit" });
    const advanced = h("details", { class: "forge-details" }, h("summary", null, /* copy:button */ "Details"));
    const forecasts = h("div", { class: "forge-forecasts" });
    const estimateLabel = h("small", { class: "dim forge-estimate" }, /* copy:label */ "rough estimate");
    const forecastButton = h("button", { class: "chip", onclick: () => {
      const asked = ++request, key = kitKey(app), owner = app.engine;
      const current = (): boolean => forecasts.isConnected && request === asked && app.engine === owner && kitKey(app) === key;
      forecastButton.disabled = true;
      forecastButton.textContent = "Measuring…";
      const ask = measureKit(app);
      if (!ask) { forecastButton.textContent = "Forecast"; forecastButton.disabled = false; return; }
      void ask.then((rows) => {
        if (!current()) return;
        replace(forecasts, ...rows.map((row) => h("div", { class: "num" }, `${SLOT_LABEL[row.slot]} · ${row.next ? kitMove(row.next) ?? "No change" : "Complete"}`)));
        forecastButton.textContent = "Forecast"; forecastButton.disabled = false;
      }).catch(() => { if (!current()) return; forecastButton.textContent = /* copy:button */ "Retry forecast"; forecastButton.disabled = false; });
    } }, /* copy:button */ "Forecast");
    const paint = (): void => {
      const ladders = app.lineage.kit ?? [];
      const guns = (app.lineage.guns ?? []).map((gun) => {
        const label = gun.kind === "long_gun" ? /* copy:label */ "Long gun" : /* copy:label */ "Short gun";
        const button = h("button", { class: "chip forge-buy", disabled: gun.selected || !gun.available || !app.engine.buyKit,
          "data-gun": gun.kind, "aria-pressed": String(gun.selected), title: gun.blocked ?? label, onclick: () => {
            if (!app.engine.buyKit) return;
            button.disabled = true;
            void app.mutate(() => app.engine.buyKit!(gun.kind), /* copy:callout */ "Selected").then((ok) => {
              if (ok) audio.cue("unlock"); if (kit.isConnected) paint();
            });
          } }, gun.selected ? /* copy:button */ "Selected" : gun.owned ? /* copy:button */ "Select" : /* copy:button */ `Forge $${gun.price}`);
        return h("section", { class: "kit-slot tablet", "data-gun-choice": gun.kind },
          h("div", { class: "forge-action" }, h("div", { class: "forge-item" },
            itemIcon({ kind: gun.kind, label }, { size: "s" }), itemName({ kind: gun.kind, label })), button),
          h("small", { class: "num dim" }, /* copy:label */ `${gun.damage[0]}–${gun.damage[1]} damage`, " · ",
            /* copy:label */ `Range ${gun.range}`, " · ", gun.capacity === 1 ? /* copy:label */ "1 shot" : /* copy:label */ `${gun.capacity} shots`, " · ",
            /* copy:label */ `Reload ${gun.reload_ticks / 10}s`));
      });
      replace(kit, ...guns, ...ladders.map((lad) => {
        const next = lad.next;
        const item = lad.steps[lad.owned] ?? lad.steps[lad.owned - 1];
        const current = lad.owned > 0 ? lad.steps[lad.owned - 1]?.label : null;
        const button = h("button", { class: "chip forge-buy", disabled: !next?.affordable || !app.engine.buyKit, "data-slot": lad.slot, onclick: () => {
          if (!app.engine.buyKit || !next) return;
          button.disabled = true;
          void app.mutate(() => app.engine.buyKit!(lad.slot), /* copy:callout */ "Forged").then((ok) => { if (ok) audio.cue("unlock"); if (kit.isConnected) paint(); });
        } }, next ? /* copy:button */ `Forge $${next.price}` : /* copy:button */ "Complete");
        return h("section", { class: "kit-slot tablet", "data-slot": lad.slot },
          h("div", { class: "kit-head" }, h("b", null, SLOT_LABEL[lad.slot]), h("small", { class: "dim" }, current ? itemChip({kind:lad.steps[lad.owned - 1]?.kind ?? lad.slot,label:current,rarity:lad.steps[lad.owned - 1]?.rarity}) : /* copy:label */ "Starting kit")),
          h("div", { class: "forge-action" }, h("div", { class: "forge-item" }, ...(item?.kind ? [itemIcon({ kind: item.kind, label: next?.label ?? item.label, rarity: item.rarity }, { size: "s" }), itemName({ kind: item.kind, label: next?.label ?? item.label, rarity: item.rarity })] : [h("span", { class: "icon-socket" }, icon("loadout", "▤")), next?.label ?? current ?? "Complete"])), button));
      }));
      replace(advanced, h("summary", null, /* copy:button */ "Details"),
        h("div", { class: "forge-ladders num dim" }, ...ladders.map((lad) => h("p", null, `${SLOT_LABEL[lad.slot]} · `, ...(lad.steps.slice(lad.owned + 1).length ? lad.steps.slice(lad.owned + 1).flatMap((s, i) => [i ? " · " : "", itemChip({kind: s.kind ?? lad.slot, label: s.label, rarity: s.rarity}), ` $${s.price}`]) : ["Complete"])))),
        salvage(app), (app.engine.kitEstimates ?? app.engine.kitDeltas) ? forecastButton : null, estimateLabel, forecasts);
    };
    const changed = (): void => {
      const key = kitKey(app);
      if (key === state && engine === app.engine) return;
      state = key; engine = app.engine; request++;
      replace(forecasts);
      forecastButton.textContent = "Forecast"; forecastButton.disabled = false;
      paint();
    };
    paint();
    unChange = app.onChange(changed); unRules = app.onRules(changed);
    return h("div", { class: "sheet-body forge" }, h("div", { class: "label" }, /* copy:label */ "Forge"), kit, advanced);
  }, { anchor, onClose: () => { unChange?.(); unRules?.(); } });
}

/** Cut 9 §10: each kind's salvage ladder — `sword · salvaged 3/5 → craftable` (the engine's `next` rung); at the top, the count alone. */
function salvage(app: App): HTMLElement {
  const L = app.lineage; const rows = Object.entries(L.forge ?? {}).sort((a, b) => b[1].salvaged - a[1].salvaged);
  const head = h("div", { class: "lrow head" }, h("span", { class: "k" }, ""), h("span", null, ""), /* copy:label */ ...["craft", "tier"].map((s) => h("span", { class: "dot-h" }, s)));
  // with nothing salvaged yet, one dim line says so instead of bare headers (QA on 50bb162: "FORGE sheet shows only the headers")
  if (!rows.length) return h("div", { class: "salvage ledger" }, h("div", { class: "empty-line dim" }, /* copy:callout */ "nothing salvaged"));
  return h("div", { class: "salvage ledger" }, head, ...rows.map(([kind, f]) => h("div", { class: "lrow" },
    h("span", { class: "k" }, itemChip({kind, label: kind.replace(/_/g, " ")})),
    h("span", { class: "ladder num dim" }, /* copy:label */ "salvaged", " ", f.next ? h("span", null, `${f.salvaged}/${f.next.need}`, " → ", h("span", { class: "rung" }, f.next.label.replace(/_/g, " "))) : `${f.salvaged}`),
    // QA 778fa1b (qaU: `· | ·` read as missing values; qaV: `⚒` looked like a button): a mark, not a tool — `✓` / `–`
    h("span", { class: `dot${f.craftable ? " on" : ""}` }, f.craftable ? "✓" : "–"), h("span", { class: `dot num${f.tier ? " on" : ""}` }, f.tier ? `+${f.tier}` : "–"))));
}
