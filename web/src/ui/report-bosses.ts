import type { Lineage, ReturnReport } from "../engine/types";
import { enemyHost } from "./enemy-tips";
import { h } from "./dom";
import { foeSrc, icon } from "./skin";

export const bossName = (kind: string): string => kind.replace(/_/g, " ").replace(/\b[a-z]/g, c => c.toUpperCase());

/** First victories are reported deltas, never today's slain walls or selection. */
export function reportBosses(r: Pick<ReturnReport, "bests" | "bloodlines">, lineage?: Lineage): HTMLElement | null {
  const groups = r.bloodlines?.some(s => s.bests !== undefined)
    ? [...r.bloodlines].sort((a, b) => a.id - b.id).map(s => ({ owner: s.id, name: s.name, bests: s.bests ?? [] }))
    : [{ owner: undefined, name: undefined, bests: r.bests }];
  const victories = groups.flatMap(s => [...new Set(s.bests.flatMap(best => {
    const match = /^boss:\s*([a-z][a-z0-9_]*)$/.exec(best);
    return match ? [match[1]!] : [];
  }))].map(boss => ({ boss, owner: s.owner, name: s.name })));
  if (!victories.length) return null;
  return h("section", { class: "report-bosses" }, h("b", { class: "row-label" }, /* copy:label */ "Boss defeated"),
    ...victories.map(v => {
      const src = foeSrc(v.boss);
      return enemyHost(h("div", { class: "report-boss-row", "data-boss": v.boss, ...(v.owner === undefined ? {} : { "data-bloodline": String(v.owner) }) },
        src ? h("img", { class: "report-boss-face", src, alt: "", "aria-hidden": "true", draggable: "false" }) : icon("unlocks", "✦"),
        h("span", { class: "report-boss-copy" }, h("b", null, bossName(v.boss)),
          v.name ? h("small", null, v.name) : "")), v.boss,
          v.owner === undefined || v.owner === (lineage?.selected_bloodline ?? 1) ? lineage : undefined, true);
    }));
}
