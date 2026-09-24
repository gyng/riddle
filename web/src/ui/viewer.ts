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
  stats?(): unknown;   // renderer diagnostics (dev: `window.__viewer.stats()` while a run is mounted)
}
type RenderMod = { createViewer(canvas: HTMLCanvasElement, opts?: { baseTexels?: number }): Viewer };
const PHONE_TEXELS = 120;
const mods = import.meta.glob<RenderMod>("../render/index.ts");

export async function makeViewer(canvas: HTMLCanvasElement): Promise<{ viewer: Viewer; real: boolean }> {
  const loader = Object.values(mods)[0];
  if (loader && new URLSearchParams(location.search).get("view") !== "2d") {
    try {
      const m = await loader();
      // phones: 120 env texels along the short axis (Cut 14 §3, was 150: k = 10 at dpr 3 → 27 CSS-px tiles,
      // ~15 across, a rat 27 px tall in the map frame; k = 6 at dpr 2); desktop takes the renderer's default
      const short = Math.min(canvas.clientWidth || window.innerWidth, canvas.clientHeight || window.innerHeight);
      const dev = import.meta.env.DEV ? Number(new URLSearchParams(location.search).get("texels")) || 0 : 0;   // dev: `?texels=N` tries a texel base
      return { viewer: m.createViewer(canvas, dev ? { baseTexels: dev } : short < 600 ? { baseTexels: PHONE_TEXELS } : {}), real: true };
    }
    catch (e) { console.warn("renderer unavailable, placeholder view", e); }
  }
  return { viewer: createPlaceholderViewer(canvas), real: false };
}
