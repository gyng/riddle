// The player's effects level (client prefs, never game truth): how much combat juice the viewer draws on top of the quality tier.
//   full — everything (blood and ichor sprays, floor splats, knockback, trauma shake, hit-stop, kill chunks)
//   low  — half the particles, splats fewer and fainter, a small shake, no hit-stop
//   off  — no gore (sprays, chunks, splats), no shake, no hit-stop, no knockback; flashes, numbers and lights stay (they carry meaning)
// Reduced motion (quality.ts) still drops every motion effect at any level. localStorage `riddle.effects`; `?effects=` pins it for a
// page. A change is announced on `window` as `riddle:effects` so a live viewer picks it up without a reload.
export type FxLevel = "full" | "low" | "off";
export const FX_LEVELS: readonly FxLevel[] = ["full", "low", "off"];
const KEY = "riddle.effects";
const EVENT = "riddle:effects";

let cached: FxLevel | null = null;

function read(): FxLevel {
  try {
    const q = new URLSearchParams(location.search).get("effects");
    if (q && (FX_LEVELS as readonly string[]).includes(q)) return q as FxLevel;
    const s = localStorage.getItem(KEY);
    if (s && (FX_LEVELS as readonly string[]).includes(s)) return s as FxLevel;
  } catch { /* storage blocked: the default */ }
  return "full";
}

export function effectsLevel(): FxLevel { return (cached ??= read()); }

export function setEffectsLevel(l: FxLevel): void {
  cached = l;
  try { if (l === "full") localStorage.removeItem(KEY); else localStorage.setItem(KEY, l); } catch { /* storage blocked: this page only */ }
  try { window.dispatchEvent(new CustomEvent(EVENT, { detail: l })); } catch { /* no window (tests) */ }
}

/** the next level in the cycle full → low → off → full (the settings sheet's one button) */
export function nextEffectsLevel(l: FxLevel = effectsLevel()): FxLevel { return FX_LEVELS[(FX_LEVELS.indexOf(l) + 1) % FX_LEVELS.length]!; }

try { window.addEventListener("storage", (e) => { if (e.key === KEY) cached = null; }); } catch { /* no window */ }
