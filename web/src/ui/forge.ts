// Cut 23 §1 — the forge: the heir's starting kit bought with gold (`Lineage.kit`: a ladder per slot — weapon · armour · pack),
// permanent for the lineage. The `forge` tile's sheet: one carved tablet per slot — its steps as pips (owned lit), the next step
// a button that reads its kit, its measured move and its price (`mail · D9 +7 · $340`, `kitDeltas()` on the background lane, `…`
// until it lands), the steps after it dim with their prices (`mail +1 $900`). A buy takes two taps (`ok $340`); nothing on the
// sheet moves under the finger (each tablet keeps its height: the next step takes the bought one's place). Under the ladders,
// the salvage ladder (Cut 9 §10) as before.
import type { App } from "../app";
import type { KitLadder, Lineage } from "../engine/types";
import { h, replace, twoTap } from "./dom";
import { openSheet } from "./sheet";
import { moveOf } from "./forecast";
import { audio } from "../audio";

/** The last `kitDeltas()` and what it was measured for (the set, the kit owned, the best, the start): the sheet paints it at once. */
let kitMemo: { key: string; kit: KitLadder[] } | null = null;
const kitKey = (app: App): string => JSON.stringify([app.rules.rows, (app.lineage.kit ?? []).map((k) => k.owned), app.lineage.best_depth, app.lineage.start ?? 1]);
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
  const out = [{ label: `D${n.depth ?? "?"}`, text: d.text, dir: d.dir, worse: false }];
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
const kitTone = (x: { dir: "up" | "down" | "flat"; worse: boolean }): string => x.dir === "flat" ? "flat" : (x.dir === "up") !== x.worse ? "up" : "down";

export function openForge(app: App): void {
  openSheet(() => {
    const kit = h("div", { class: "kit" });
    const paint = (measured: KitLadder[] | null, pending: boolean): void => {
      const L = app.lineage, ladders = L.kit ?? [];
      replace(kit, ...ladders.map((lad) => {
        const m = measured?.find((x) => x.slot === lad.slot)?.next;
        const n = lad.next ? { ...lad.next, ...(m && m.label === lad.next.label ? { depth: m.depth, delta: m.delta, pm: m.pm, bank: m.bank, death: m.death } : {}) } : undefined;
        const pips = h("span", { class: "pips", "aria-hidden": "true" }, ...lad.steps.map((s, i) => h("i", { class: `pip${s.owned || i < lad.owned ? " on" : ""}` })));
        const later = lad.steps.slice(lad.owned + 1);
        let act: HTMLElement;
        if (!n) act = h("span", { class: "kit-top num dim" }, /* copy:callout */ "top step");
        else {
          const terms = kitTerms(n);
          const inner = [h("span", { class: "kit-label" }, n.label),
            terms ? h("span", { class: "num kit-move" }, ...terms.flatMap((x) => [" · ", h("b", { class: `dlt ${kitTone(x)}` }, `${x.label} ${x.text}`)])) : pending ? h("small", { class: "num dim kit-move" }, " · …") : "",
            h("b", { class: "num gold kit-price" }, ` · $${n.price}`),
            // QA 912e135 (qaW: `7 nights` at 0 banked, 0 returned — "the income behind it is not on screen"): the net it divides by
            !n.affordable && n.nights !== undefined && n.nights > 0 ? h("small", { class: "num dim kit-nights" }, /* copy:callout */ ` · ${n.nights === 1 ? "1 night" : `${n.nights} nights`}`,
              n.per_night ? h("span", { class: "per-night" }, /* copy:callout */ ` at $${n.per_night}`) : "") : ""];
          if (n.affordable && app.engine.buyKit) {
            // two taps (`ok $340`); armed, it stays armed through the deltas' repaint (the key) until a tap lands elsewhere
            const b = twoTap(inner.filter((x): x is HTMLElement => typeof x !== "string"), /* copy:button */ `ok $${n.price}`, () => void buyStep(lad.slot), { class: "chip kit-next buyable", key: `kit:${lad.slot}:${n.label}` });
            b.dataset.slot = lad.slot; act = b;
          } else act = h("button", { class: "chip kit-next off", disabled: true, "data-slot": lad.slot }, ...inner);
        }
        return h("div", { class: "kit-slot tablet", "data-slot": lad.slot },
          h("div", { class: "kit-head" }, h("span", { class: "kit-name" }, SLOT_LABEL[lad.slot] ?? lad.slot), pips, h("small", { class: "num dim" }, `${lad.owned}/${lad.steps.length}`)),
          act,
          // every step shows its price: the ones after the next, dim, on one line (always there, so a buy never reflows the sheet)
          h("div", { class: "kit-later num dim" }, later.length ? later.map((s) => `${s.label} $${s.price}`).join(" · ") : " "));
      }));
    };
    async function buyStep(slot: string): Promise<void> {
      if (!app.engine.buyKit) return;
      const ok = await app.mutate(() => app.engine.buyKit!(slot), /* copy:callout */ "kit");
      if (!ok) return;
      audio.cue("unlock");
      paint(null, !!app.engine.kitDeltas); measure();
    }
    function measure(): void {
      const k = kitKey(app);
      if (kitMemo?.key === k) { paint(kitMemo.kit, false); return; }
      if (!app.engine.kitDeltas) return;
      void app.engine.kitDeltas().then((m) => { kitMemo = { key: k, kit: m }; if (kit.isConnected && kitKey(app) === k) paint(m, false); }).catch((e) => { console.warn("kitDeltas", e); if (kit.isConnected) paint(null, false); });
    }
    const memo = kitMemo?.key === kitKey(app) ? kitMemo.kit : null;
    paint(memo, !memo && !!app.engine.kitDeltas);
    if (!memo) measure();
    const hasKit = (app.lineage.kit?.length ?? 0) > 0;
    return h("div", { class: "sheet-body forge" }, h("div", { class: "label" }, /* copy:label */ "forge"), hasKit ? kit : "", salvage(app));
  });
}

/** Cut 9 §10: each kind's salvage ladder — `sword · salvaged 3/5 → craftable` (the engine's `next` rung); at the top, the count alone. */
function salvage(app: App): HTMLElement {
  const L = app.lineage; const rows = Object.entries(L.forge ?? {}).sort((a, b) => b[1].salvaged - a[1].salvaged);
  const head = h("div", { class: "lrow head" }, h("span", { class: "k" }, ""), h("span", null, ""), /* copy:label */ ...["craft", "tier"].map((s) => h("span", { class: "dot-h" }, s)));
  // with nothing salvaged yet, one dim line says so instead of bare headers (QA on 50bb162: "FORGE sheet shows only the headers")
  if (!rows.length) return h("div", { class: "salvage ledger" }, h("div", { class: "empty-line dim" }, /* copy:callout */ "nothing salvaged"));
  return h("div", { class: "salvage ledger" }, head, ...rows.map(([kind, f]) => h("div", { class: "lrow" },
    h("span", { class: "k" }, kind.replace(/_/g, " ")),
    h("span", { class: "ladder num dim" }, /* copy:label */ "salvaged", " ", f.next ? h("span", null, `${f.salvaged}/${f.next.need}`, " → ", h("span", { class: "rung" }, f.next.label.replace(/_/g, " "))) : `${f.salvaged}`),
    // QA 778fa1b (qaU: `· | ·` read as missing values; qaV: `⚒` looked like a button): a mark, not a tool — `✓` / `–`
    h("span", { class: `dot${f.craftable ? " on" : ""}` }, f.craftable ? "✓" : "–"), h("span", { class: `dot num${f.tier ? " on" : ""}` }, f.tier ? `+${f.tier}` : "–"))));
}
