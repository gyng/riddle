// localStorage persistence: engine save string + client-side set tabs + last_seen.
import type { RuleSet } from "./engine/types";

export type SaveBlob = { v: 1; engine: string; sets: RuleSet[]; active: number; loadout: number[]; last_seen: number };
const KEY = "riddle.save";

export function readBlob(): SaveBlob | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const b = JSON.parse(raw) as SaveBlob;
    return b && b.v === 1 && typeof b.engine === "string" ? b : null;
  } catch { return null; }
}
export function writeBlob(b: SaveBlob): void {
  try { localStorage.setItem(KEY, JSON.stringify(b)); } catch { /* quota or private mode: play on */ }
}
export function clearBlob(): void { try { localStorage.removeItem(KEY); } catch { /* ignore */ } }
export function randomSeed(): number {
  const a = new Uint32Array(1);
  crypto.getRandomValues(a);
  return a[0];
}
