import type { Lineage } from "../engine/types";
import { counterName } from "./counter-name";
import { h } from "./dom";
import { detailHost } from "./tips";
import { unitLabel } from "./unit-icon";
import { modifierLines, type EncounterDetails } from "./encounter-modifiers";
const nice = (s: string): string => s.replace(/_/g, " ").replace(/\b[a-z]/g, c => c.toUpperCase());
/** Learned tags describe a captive's possible role, not its current allegiance. */
export const enemyTraitName = (kind: string, tag: string): string => kind === "captive" && tag === "ally" ? /* copy:label */ "potential ally" : tag.replace(/_/g, " ");
export function enemyHost<E extends HTMLElement>(el: E, kind: string, L?: Pick<Lineage, "facts" | "ledger" | "walls" | "counters"> & Partial<Pick<Lineage, "packages">>, defeated = false, instance?:() => EncounterDetails|undefined): E {
  return detailHost(el, () => {
    kind = kind.replace(/ pack$/, "").trim().replace(/ /g, "_");
    const encounter=instance?.();
    const row = L?.ledger?.find(r => r.kind === kind), wall = L?.walls?.find(w => w.boss === kind);
    const counterRaw = row?.counter?.text ?? L?.counters?.find(c => c.boss === kind)?.text ?? (wall?.known ? wall.counter ?? wall.fact.replace(/^[^:]+:\s*/, "") : undefined);
    const counter = counterRaw && counterName(L, counterRaw, row?.counter?.row);   // blind 5331f40: a tactic counter by its package name
    const traits = (L?.facts ?? []).filter(f => f.startsWith(`foe:${kind}:`)).map(f => nice(enemyTraitName(kind, f.slice(`foe:${kind}:`.length))));
    const line = (label: string, value: string): HTMLElement => h("div", { class: "enemy-tip-line" }, h("b", null, label), " ", value);
    return [unitLabel(kind, h("b", null, nice(kind)), { px: 48, className: "kw-tip-head" }),
      ...modifierLines(encounter),
      ...(encounter?.status_effects??[]).map(effect=>line(/* copy:label */"Hexed",effect)),
      ...(wall ? [line(/* copy:label */ "Floor", `D${wall.depth}`)] : []),
      line(/* copy:label */ "Encounter", encounter?.alive ? /* copy:label */ "Alive" : defeated || wall?.slain ? /* copy:label */ "Defeated" : !L ? /* copy:label */ "Unavailable" : row?.seen ? /* copy:label */ "Seen" : /* copy:label */ "Unseen"),
      ...(row?.studied ? [line(/* copy:label */ "Study", /* copy:label */ "Studied")] : []),
      line(/* copy:label */ "Traits", traits.length ? traits.join(" · ") : !L ? /* copy:label */ "Unavailable" : /* copy:label */ "Not learned"),
      line(/* copy:label */ "Counter", counter?.replace(/, boss$/, " at boss") ?? (!L ? /* copy:label */ "Unavailable" : /* copy:label */ "Not learned"))];
  });
}
