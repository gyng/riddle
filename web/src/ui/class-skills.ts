import { h } from "./dom";
import { icon, verbIcon } from "./skin";
import { verbLabel } from "./tokens";

/** Shared skill rung for both class pickers; art can be absent without hiding the skill. */
export function classSkillChip(v: string, level: number, reached: boolean): HTMLElement {
  const id = verbIcon(v);
  return h("span", { class: `chip rung num${reached ? " on" : ""}`, "data-skill": v },
    id ? icon(id) : "", `L${level} `, verbLabel({ v }));
}
