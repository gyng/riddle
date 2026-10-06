// Cohort 24 (AW): the viewer when no GL context can be made at all (a page Chrome blocked from 3D after a context loss, a GPU
// blocklist, `?view=2d`). The same replay clock as the GL viewer (ReplayState: tick, idle, seek, skipToEvent — the watch's pacing and
// ▶▶| work unchanged), drawn by view2d.ts into the canvas's own 2D context. The old placeholder had no clock: ▶▶| went dead on it.
import { ReplayState } from "./state";
import { View2D } from "./view2d";
import type { Frame, Viewer, ViewerStats } from "./index";

export function createFallbackViewer(canvas: HTMLCanvasElement): Viewer {
  const st = new ReplayState();
  // (a canvas a failed GL attempt left bound has no 2D context: the view goes on an overlay beside it)
  const view = canvas.getContext("2d") ? new View2D({ own: canvas }) : new View2D({ host: canvas });
  view.show(true);
  let mode: Frame = "map", raf = 0, last = performance.now(), disposed = false;
  const times: number[] = [];
  const stats: ViewerStats = { calls: 0, triangles: 0, k: 1, dpr: window.devicePixelRatio || 1, frame: "map", kMap: 1, shake: [0, 0], glyphs: 0, caption: null,
    device: [0, 0], envTexels: [0, 0], target: [0, 0], pending: 0, tick: 0, fade: 0, hero: [0, 0], projectiles: 0, ents: 0, drawn: 0, camera: [0, 0],
    cpuMs: NaN, cpuP95: NaN, buildMs: NaN, gpuMs: NaN, gpuP95: NaN, gpuTimer: false, fps: NaN, fx: "low", glLost: true };
  function frame(now: number): void {
    if (disposed) return;
    raf = requestAnimationFrame(frame);
    times.push(now); while (times.length > 2 && now - times[0]! > 1000) times.shift();
    if (times.length > 1) stats.fps = (times.length - 1) / ((now - times[0]!) / 1000);
    const dt = Math.max(0, Math.min(250, now - last)); last = now;
    st.tick(dt, now);
    st.cameraSnap = false;
    const t0 = performance.now();
    view.draw(st);
    stats.cpuMs = performance.now() - t0;
    stats.tick = st.tickNow(); stats.pending = st.pending(); stats.fade = st.fade; stats.ents = st.ents.size;
    const h = st.hero; if (h) stats.hero = [h.px, h.py];
    stats.caption = st.caption?.text ?? null; stats.device = [canvas.width, canvas.height];
  }
  raf = requestAnimationFrame(frame);
  return {
    load(snap) { st.load(snap); },
    apply(evs) { st.apply(evs); },
    setSpeed(n) { st.speed = Math.max(0, n); },
    skipToEvent() { st.skipToEvent(); },
    seek(t) { st.seek(t); },
    sync(snap) { st.sync(snap); },
    setFrame(f) { mode = f; st.fight = f === "fight"; stats.frame = f; },
    frame() { return mode; },
    gun() { return st.gun; },
    tick() { return st.tickNow(); },
    idle() { return st.idle(); },
    resize() { /* the view fits the canvas each frame */ },
    dispose() { disposed = true; cancelAnimationFrame(raf); view.dispose(); },
    stats() { return { ...stats }; },
    preload(snap) { st.preload(snap); },
    setKeepOut() { /* no pixel text drawn */ },
    setQuiet() { /* no pixel text drawn */ },
    setTacticRows(rows, meaningful) { st.setTacticRows(rows, meaningful); },
    debugBiome() { return st.biome; },
    debugRects() { return []; },
    debugLabels() { return []; },
    debugText() { return []; },
  };
}
