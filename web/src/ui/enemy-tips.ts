import type { Lineage } from "../engine/types";
import { h } from "./dom";
import { detailHost } from "./tips";
import { foeSrc } from "./skin";
const nice = (s: string): string => s.replace(/_/g, " ").replace(/\b[a-z]/g, c => c.toUpperCase());
export function enemyHost<E extends HTMLElement>(el: E, kind: string, L?: Pick<Lineage, "facts" | "ledger" | "walls" | "counters">, defeated = false): E {
  return detailHost(el, () => {
    kind = kind.replace(/ pack$/, "").trim().replace(/ /g, "_");
    const row = L?.ledger?.find(r => r.kind === kind), wall = L?.walls?.find(w => w.boss === kind);
    const counter = row?.counter?.text ?? L?.counters?.find(c => c.boss === kind)?.text ?? (wall?.known ? wall.counter ?? wall.fact.replace(/^[^:]+:\s*/, "") : undefined);
    const traits = (L?.facts ?? []).filter(f => f.startsWith(`foe:${kind}:`)).map(f => nice(f.slice(`foe:${kind}:`.length)));
    const src = foeSrc(kind);
    const line = (label: string, value: string): HTMLElement => h("div", { class: "enemy-tip-line" }, h("b", null, label), " ", value);
    return [h("div", { class: "kw-tip-head" }, src ? h("img", { class: "enemy-tip-face", src, alt: "" }) : "", h("b", null, nice(kind))),
      ...(wall ? [line(/* copy:label */ "Floor", `D${wall.depth}`)] : []),
      line(/* copy:label */ "Encounter", defeated || wall?.slain ? /* copy:label */ "Defeated" : row?.seen ? /* copy:label */ "Seen" : /* copy:label */ "Unseen"),
      ...(row?.studied ? [line(/* copy:label */ "Study", /* copy:label */ "Studied")] : []),
      line(/* copy:label */ "Traits", traits.length ? traits.join(" · ") : !L ? /* copy:label */ "Unavailable" : /* copy:label */ "Not learned"),
      line(/* copy:label */ "Counter", counter?.replace(/, boss$/, " at boss") ?? (!L ? /* copy:label */ "Unavailable" : /* copy:label */ "Not learned"))];
  });
}
