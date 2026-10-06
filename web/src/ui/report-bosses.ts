import type { Lineage, ReturnReport } from "../engine/types";
import { enemyHost } from "./enemy-tips";
import { h } from "./dom";
import { unitLabel } from "./unit-icon";

export const bossName = (kind: string): string => kind.replace(/_/g, " ").replace(/\b[a-z]/g, c => c.toUpperCase());

/** First victories are reported deltas, never today's slain walls or selection. */
export function reportBosses(r: Pick<ReturnReport, "bests" | "bloodlines">, lineage?: Lineage): HTMLElement | null {
  const groups = r.bloodlines?.some(s => s.bests !== undefined)
    ? [...r.bloodlines].sort((a, b) => a.id - b.id).map(s => ({ owner: s.id, name: s.name, bests: s.bests ?? [], knowledge: s.boss_knowledge }))
    : [{ owner: undefined, name: undefined, bests: r.bests, knowledge: undefined }];
  const victories = groups.flatMap(s => [...new Set(s.bests.flatMap(best => {
    const match = /^boss:\s*([a-z][a-z0-9_]*)$/.exec(best);
    return match ? [match[1]!] : [];
  }))].map(boss => ({ boss, owner: s.owner, name: s.name, knowledge: s.knowledge?.find(k=>k.boss===boss) })));
  if (!victories.length) return null;
  return h("section", { class: "report-bosses" }, h("b", { class: "row-label" }, /* copy:label */ "Boss defeated"),
    ...victories.map(v => {
      const row = unitLabel(v.boss, h("span", { class: "report-boss-copy" }, h("b", null, bossName(v.boss)), v.name ? h("small", null, v.name) : ""), { px: 44 });
      row.classList.add("report-boss-row"); row.dataset.boss = v.boss;
      if (v.owner !== undefined) row.dataset.bloodline = String(v.owner);
      row.querySelector(".unit-icon")!.classList.add("report-boss-face");
      return enemyHost(row, v.boss,
          v.knowledge ? { facts: v.knowledge.facts, ledger: v.knowledge.ledger ? [v.knowledge.ledger] : [], walls: v.knowledge.wall ? [v.knowledge.wall] : [], counters: [] }
            : v.owner === undefined || v.owner === (lineage?.selected_bloodline ?? 1) ? lineage : undefined, true);
    }));
}
