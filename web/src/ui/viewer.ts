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
  idle?(): boolean;
}
type RenderMod = { createViewer(canvas: HTMLCanvasElement, opts?: { baseTexels?: number }): Viewer };
const mods = import.meta.glob<RenderMod>("../render/index.ts");

export async function makeViewer(canvas: HTMLCanvasElement): Promise<{ viewer: Viewer; real: boolean }> {
  const loader = Object.values(mods)[0];
  if (loader && new URLSearchParams(location.search).get("view") !== "2d") {
    try { const m = await loader(); return { viewer: m.createViewer(canvas), real: true }; }
    catch (e) { console.warn("renderer unavailable, placeholder view", e); }
  }
  return { viewer: createPlaceholderViewer(canvas), real: false };
}
