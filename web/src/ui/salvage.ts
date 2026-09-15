// Addendum D salvage values (client mirror of the contract table; the engine is the truth).
// Unidentified consumables arrive as kind "potion" / "scroll" (core README) and price as their class.
const V: Record<string, number> = { dagger: 10, sword: 20, axe: 30, bow: 20, leather: 15, mail: 25, plate: 40, leash: 5, potion: 8, scroll: 12, gold: 0 };
const POTIONS = new Set(["heal", "strength", "speed", "invisibility", "poison", "caustic", "confusion", "fire"]);
export function salvageValue(kind: string, tier: string): number {
  const base = V[kind] ?? (POTIONS.has(kind) ? 8 : 12);
  return Math.round(base * (tier === "bank" ? 1 : tier === "return" ? 0.6 : 0.3));
}
