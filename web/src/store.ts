// localStorage persistence: engine save string + loadout + last_seen. (v1 blobs also carried client-side
// set tabs; the engine's saved sets are the truth now, so v1 is read but its sets are ignored.)
// Cut 7 §2: `origins` = each saved set's row origins (`preset | patch | card | player`, by row index), a client-side
// tag the engine save does not carry.
// `watch`: the last chosen watch mode (`fights | fast`), so the next run starts in it (QA on e0f87e7: "`fast` chosen in run 3
// was not remembered: run 4 started on fights").
export type SaveBlob = { v: 1 | 2; engine: string; loadout: number[]; last_seen: number; runs?: number; origins?: string[][]; watch?: "fights" | "fast" | "one" };
const KEY = "riddle.save";

export function readBlob(): SaveBlob | null {
  try {
    const raw = localStorage.getItem(KEY);
    if (!raw) return null;
    const b = JSON.parse(raw) as SaveBlob;
    return b && (b.v === 1 || b.v === 2) && typeof b.engine === "string" ? b : null;
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
