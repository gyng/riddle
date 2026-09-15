// Addendum C — class XP tables (data only; shared by the fake engine and the UI).
export const XP_LEVEL_CAP = 10;
export const xpToNext = (level: number): number => 40 * level * level;
/** Verbs unlocked at each level per class. */
export const CLASS_VERBS: Record<string, Record<number, string[]>> = {
  fighter: { 1: ["shield_bash"], 3: ["cleave"], 5: ["taunt"], 7: ["second_wind"], 9: ["bulwark"] },
  rogue:   { 1: ["vanish", "throw"], 3: ["backstab"], 5: ["smoke"], 7: ["ambush"], 9: ["shadowstep"] },
};
export function verbsUpTo(cls: string, level: number): string[] {
  const t = CLASS_VERBS[cls] ?? {}; const out: string[] = [];
  for (let l = 1; l <= level; l++) out.push(...(t[l] ?? []));
  return out;
}
export const verbsAt = (cls: string, level: number): string[] => (CLASS_VERBS[cls]?.[level] ?? []);
