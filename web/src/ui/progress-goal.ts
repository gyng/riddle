import type { Lineage } from "../engine/types";
import { h } from "./dom";
import { icon } from "./skin";

/** Presentation of the engine's current bonus, independent of this run's start. */
export function progressGoal(lineage: Lineage): { depth: number; reward?: string } | null {
  if (lineage.ended || lineage.town?.home === false || lineage.best_depth < 1) return null;
  const bonus = lineage.bounty;
  if (bonus && bonus.depth > lineage.best_depth) {
    return { depth: bonus.depth, reward: bonus.pays?.replace(/\$×(\d+)/g, "$1× gold") || undefined };
  }
  return { depth: lineage.best_depth + 1 };
}

export function progressGoalRow(goal: NonNullable<ReturnType<typeof progressGoal>>, className = ""): HTMLElement {
  const reward = goal.reward ?? /* copy:label */ "New record";
  return h("span", { class: `progress-goal depth-summary-row ${className}`, title: `D${goal.depth} · ${reward}` },
    h("small", null, /* copy:label */ "Goal"), h("b", { class: "num", "data-goal-floor": goal.depth }, `D${goal.depth}`),
    h("span", { class: "progress-reward" }, goal.reward ? icon("gold", "$") : icon("depth", "↓"), reward));
}
