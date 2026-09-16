// Addendum C + Cut 2 §4 — class XP tables and verb ladders (data only; shared by the fake engine and the UI).
// The class list is data too: the picker shows CLASSES ∪ lineage.classes keys, never a hardcoded pair.
export const XP_LEVEL_CAP = 10;
export const xpToNext = (level: number): number => 40 * level * level;
/** Every class the client knows a ladder for; `fighter` is the free one, the rest are unlock ids. */
export const CLASSES = ["fighter", "rogue", "ranger", "caster"] as const;
export const isFreeClass = (cls: string): boolean => cls === "fighter";
/** Verbs unlocked at each level per class. */
export const CLASS_VERBS: Record<string, Record<number, string[]>> = {
  fighter: { 1: ["shield_bash"], 3: ["cleave"], 5: ["taunt"], 7: ["second_wind"], 9: ["bulwark"] },
  rogue:   { 1: ["vanish", "throw"], 3: ["backstab"], 5: ["smoke"], 7: ["ambush"], 9: ["shadowstep"] },
  ranger:  { 1: ["shoot", "kite"], 3: ["volley"], 5: ["trap"], 7: ["mark"], 9: ["double_shot"] },
  caster:  { 1: ["bolt", "ward"], 3: ["blink"], 5: ["slow"], 7: ["nova"], 9: ["drain"] },
};
export function verbsUpTo(cls: string, level: number): string[] {
  const t = CLASS_VERBS[cls] ?? {}; const out: string[] = [];
  for (let l = 1; l <= level; l++) out.push(...(t[l] ?? []));
  return out;
}
export const verbsAt = (cls: string, level: number): string[] => (CLASS_VERBS[cls]?.[level] ?? []);
