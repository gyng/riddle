// Cut 121 §1 (core `Lineage.descent`): the numbered descent under way — its tier, the floor it starts from (an ascension starts
// mid-dungeon: `endgame::ascent_start`) and the foes' gains and modifiers at work — `Ascension 1 · from D14 · foes +5% hp · +2% dmg`,
// the modifiers named under it (each its effect and counter in its tip). Shown on the camp, the run setup and the report; the offer
// sheet names the start the next descent would take.
import type { Lineage } from "../engine/types";
import { h } from "./dom";

export type DescentWire = NonNullable<Lineage["descent"]>;

/** The line's segments: `Ascension 1` · `from D14` · `foes +5% hp` · `+2% dmg` (a zero gain left out). */
export function descentBits(d: DescentWire): string[] {
  return [
    /* copy:callout */ `Ascension ${d.tier}`,
    /* copy:callout */ `from D${d.start}`,
    d.hp_pct > 0 ? /* copy:callout */ `foes +${d.hp_pct}% hp` : "",
    d.atk_pct > 0 ? /* copy:callout */ `+${d.atk_pct}% dmg` : "",
  ].filter(Boolean);
}

/** The descent's line with its modifiers (`Armoured · Shielded`, each titled `+1 armour · counter: poison or fire`); null at tier 0. */
export function descentLine(L: Pick<Lineage, "descent">, cls = ""): HTMLElement | null {
  const d = L.descent;
  if (!d || d.tier <= 0) return null;
  const [head, ...rest] = descentBits(d);
  const mods = d.modifiers ?? [];
  return h("div", { class: `descent-line num ${cls}`.trim(), "data-tier": d.tier, "data-start": d.start },
    h("b", null, head), rest.length ? ` · ${rest.join(" · ")}` : "",
    mods.length ? h("small", { class: "descent-mods dim" }, " · ", ...mods.flatMap((m, i) => [i ? " · " : "",
      h("span", { class: "descent-mod", "data-mod": m.id, title: `${m.effect} · ${/* copy:callout */ "counter"}: ${m.counter.toLowerCase()}` }, m.name)])) : "");
}

/** The floor the next numbered descent would start from — a stand-in for the core's `endgame::ascent_start` (the offer does not
 *  carry it): the deepest lit stone at or under `deepest − 15`, never above D9; D1 with none. */
export const ASCENT_BAND = 15, ASCENT_MIN = 9;
export function ascentStartOf(L: Pick<Lineage, "waystones" | "lanes">): number {
  const stones = [...(L.waystones ?? []), ...(L.lanes ?? []).map((x) => x.depth)];
  if (!stones.length) return 1;
  const deepest = Math.max(...stones), cap = Math.max(deepest - ASCENT_BAND, ASCENT_MIN);
  const at = stones.filter((w) => w <= cap);
  return at.length ? Math.max(...at) : 1;
}
