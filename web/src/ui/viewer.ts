// Viewer bridge: the renderer track exports createViewer(canvas) from src/render/index.ts; until it
// exists (or if it throws) a Canvas-2D placeholder stands in. Structural type = the CUT1 contract.
import type { Ev, Snapshot } from "../engine/types";
import { createPlaceholderViewer } from "./placeholder-view";

export interface Viewer {
  load(snap: Snapshot): void;
  apply(evs: Ev[]): void;
  setSpeed(n: number): void;
  skipToEvent(): void;
  dispose(): void;
  resize?(): void;
  idle?(): boolean;   // queue drained and tails played out (real renderer)
  tick?(): number;    // current tick of the playback clock (real renderer)
  sync?(snap: Snapshot): void; // Cut 4 §3: adopt `remembered` flags (and last-seen tiles) from a step's snapshot
  setKeepOut?(rects: { x: number; y: number; w: number; h: number }[]): void;   // Cut 28 §4: the DOM over the canvas (CSS px) — no pixel text on it
  setQuiet?(on: boolean): void;   // Cut 22: a held beat owns the line — no callout or caption drawn over the fight meanwhile
  stats?(): unknown;   // renderer diagnostics (dev: `window.__viewer.stats()` while a run is mounted)
}
type RenderMod = { createViewer(canvas: HTMLCanvasElement, opts?: { baseTexels?: number }): Viewer; createFallbackViewer(canvas: HTMLCanvasElement): Viewer };
const PHONE_TEXELS = 100;   // gfx round 1 (raters: "half the viewport is empty void"; watch.png's ~9 tiles across): k 6 → 8 at 400 px × 2 (was 120)
const DESK_TEXELS = 84;     // gfx round 1: the desktop frame's centre well (~680 px short side): k 4 → 6 (the renderer's default 160 read "tiny"); round 7: 112 → 84 (k 8) with the half-size sprites (raters Q, R: "sprites tiny at 1440")
const mods = import.meta.glob<RenderMod>("../render/index.ts");

export async function makeViewer(canvas: HTMLCanvasElement): Promise<{ viewer: Viewer; real: boolean }> {
  const loader = Object.values(mods)[0];
  // cohort 24 (AW): no GL context (blocked after a loss, `?view=2d`) → the renderer's own 2D viewer, the same replay clock (the old
  // placeholder had none: ▶▶| went dead on it); the placeholder only when the render module itself fails to load
  let m: RenderMod | null = null;
  if (loader) { try { m = await loader(); } catch (e) { console.warn("renderer module unavailable", e); } }
  if (m && new URLSearchParams(location.search).get("view") !== "2d") {
    try {
      // phones: 120 env texels along the short axis (Cut 14 §3, was 150: k = 10 at dpr 3 → 27 CSS-px tiles,
      // ~15 across, a rat 27 px tall in the map frame; k = 6 at dpr 2); desktop takes the renderer's default
      const short = Math.min(canvas.clientWidth || window.innerWidth, canvas.clientHeight || window.innerHeight);
      const dev = import.meta.env.DEV ? Number(new URLSearchParams(location.search).get("texels")) || 0 : 0;   // dev: `?texels=N` tries a texel base
      // gfx round 7 (raters Q, R on the edit's scene, ~210 CSS px tall: "tiny sprites in a black box — zoom the inset so the hero and
      // foe are twice their size"): a view under 320 CSS px shows about half the texels
      const base = short < 600 ? PHONE_TEXELS : DESK_TEXELS;
      return { viewer: m.createViewer(canvas, dev ? { baseTexels: dev } : { baseTexels: short < 320 ? Math.round(base * 0.55) : base }), real: true };
    }
    catch (e) { console.warn(/* copy:none */ "renderer unavailable, 2D view", e); }
  }
  if (m) { try { return { viewer: m.createFallbackViewer(canvas), real: true }; } catch (e) { console.warn(/* copy:none */ "2D view unavailable, placeholder view", e); } }
  return { viewer: createPlaceholderViewer(canvas), real: false };
}
