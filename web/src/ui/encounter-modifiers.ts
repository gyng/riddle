// Read-only descriptions from the played encounter's Rust catalogue.
import type { EncounterModifiers, ModifierInfo } from "../engine/types";
import { h } from "./dom";
export type EncounterDetails = { modifiers?:EncounterModifiers; modifier_catalogue?:ModifierInfo[]; alive?:boolean };
export function modifierLines(details?:EncounterDetails): Node[] {
  const m=details?.modifiers;
  if (!m || m.tier<=0) return [];
  const rows=(details?.modifier_catalogue??[]).filter(r => r.mask>0 ? (m.affixes&r.mask)!==0
    : r.id===m.elite || (r.id==='tight_mirror'&&m.tight_mirror));
  return [h("div", {class:"enemy-tip-line num"}, /* copy:label */ `Ascension ${m.tier}`),
    ...rows.flatMap(r => [h("div", {class:"enemy-tip-line"}, h("b",null,r.name), " ",r.effect),
      h("div",{class:"enemy-tip-line dim"}, /* copy:label */ "Counter", " ",r.counter)]),
    ...(!rows.length ? [h("div",{class:"enemy-tip-line dim"}, /* copy:label */ "Details unavailable")] : [])];
}
