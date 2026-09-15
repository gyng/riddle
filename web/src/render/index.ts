// Riddle replay viewer: pixel pipeline per research/art-tech.md (option C, two densities).
//
// Coordinates. World unit = 1 environment texel. Tile = 8×8 units at world (tx*8, -ty*8): tile row 0
// is at the top of the map, world +Y is screen up. The OrthographicCamera sits at z=100 looking
// down -Z (top-down, no pitch). Layer z: tiles 0, overlays 0.5, items 1, shadows 1.5, entities
// 2..3 (sorted by tile row so lower rows draw in front), glyphs 3.5, callout text 4.
//
// Sizes. k = max(1, floor(min(devW, devH)/270)) device px per env texel; iw×ih = floor(dev/k) env
// texels visible; the target is (iw+2)×(ih+2) env texels of overscan rendered at 2 target px per
// env texel, i.e. (iw+2)*2 × (ih+2)*2 target px (sprite texel = 1 target px = k/2 device px).
// The camera centre snaps to the env-texel grid; the sub-texel remainder (quantised to device px)
// is applied as a UV offset in the fullscreen blit, which upscales the target k× with nearest
// sampling into a viewport of exactly iw*k × ih*k device px (any sliver left by floor() shows
// the palette's darkest colour).
//
// Palette split. Everything renders into ONE target with depth; the target's alpha channel tags
// the layer (env quads write a=1, sprite quads a=0.5; blending is off). The blit quantises a=1
// pixels to the biome palette with 4×4 Bayer dither indexed by world texel, and blends a=0.5
// pixels 50% toward their nearest palette colour without dither. One target, one blit.
// Tiles, items, shadows, glyphs and callout text are env (a=1); entities and the gas/fire
// overlays are sprite-tagged (a=0.5) so they keep a readable hue in every biome.
import * as THREE from "three";
import { Atlas } from "./atlas";
import { Blit } from "./blit";
import { QuadLayer } from "./layers";
import { paletteFor } from "./palette";
import { FONT_ADVANCE, FONT_CELL_H, FONT_CELL_W } from "./font";
import { ReplayState, type EntState } from "./state";
import type { Ev, Snapshot } from "./types";

export type { Ev, Snapshot } from "./types";

export type Viewer = {
  load(snap: Snapshot): void;
  apply(evs: Ev[]): void;
  setSpeed(n: number): void;
  skipToEvent(): void;
  idle(): boolean;
  resize(): void;
  dispose(): void;
  stats(): ViewerStats;
};

export type ViewerStats = {
  calls: number; triangles: number; k: number; dpr: number;
  device: [number, number]; envTexels: [number, number]; target: [number, number]; pending: number;
};

const TILE = 8;
const BASE_TEXELS = 270;

export type ViewerOpts = {
  atlasUrl?: string;   // default "/art/atlas.json" (+ atlas.png beside it)
  baseTexels?: number; // env texels along the short screen axis used to pick k (default 270)
};

export function createViewer(canvas: HTMLCanvasElement, opts: ViewerOpts = {}): Viewer {
  const baseTexels = opts.baseTexels ?? BASE_TEXELS;
  const renderer = new THREE.WebGLRenderer({
    canvas, antialias: false, alpha: false, stencil: false, depth: false,
    powerPreference: "high-performance", premultipliedAlpha: false,
  });
  renderer.setPixelRatio(1);
  renderer.autoClear = true;
  renderer.info.autoReset = false;

  const rt = new THREE.WebGLRenderTarget(4, 4, {
    minFilter: THREE.NearestFilter, magFilter: THREE.NearestFilter, generateMipmaps: false,
    depthBuffer: true, stencilBuffer: false, colorSpace: THREE.NoColorSpace,
  });
  const scene = new THREE.Scene();
  const camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 1, 200);
  camera.position.set(0, 0, 100);
  scene.add(camera);

  const atlas = new Atlas();
  const env = atlas.env.texture, spr = atlas.sprite.texture;
  const L = {
    tiles: new QuadLayer(env, 24 * 24, 1, 0),
    overlays: new QuadLayer(env, 512, 0.5, 1), // tagged as sprite so gas/fire keep their hue
    items: new QuadLayer(env, 128, 1, 2),
    shadows: new QuadLayer(env, 128, 1, 3),
    ents: new QuadLayer(spr, 128, 0.5, 4),
    glyphs: new QuadLayer(env, 128, 1, 5),
    text: new QuadLayer(env, 64, 1, 6),
  };
  for (const l of Object.values(L)) scene.add(l.mesh);
  const blit = new Blit(rt.texture);
  const clear = new THREE.Color();

  const st = new ReplayState();
  const cam = { x: 0, y: 0, vx: 0, vy: 0, tx: 0, ty: 0 };
  const stats: ViewerStats = { calls: 0, triangles: 0, k: 1, dpr: 1, device: [0, 0], envTexels: [0, 0], target: [0, 0], pending: 0 };
  let k = 1, iw = 1, ih = 1, W = 3, H = 3, devW = 0, devH = 0, dpr = 1;
  let lastCss = "";
  let raf = 0;
  let last = performance.now();
  let disposed = false;

  void atlas.load(opts.atlasUrl ?? "/art/atlas.json");

  function measure(): boolean {
    const cw = canvas.clientWidth || canvas.width, ch = canvas.clientHeight || canvas.height;
    const d = window.devicePixelRatio || 1;
    const key = `${cw}x${ch}@${d}`;
    if (key === lastCss) return false;
    lastCss = key;
    dpr = d;
    devW = Math.max(1, Math.round(cw * d));
    devH = Math.max(1, Math.round(ch * d));
    k = Math.max(1, Math.floor(Math.min(devW, devH) / baseTexels));
    iw = Math.floor(devW / k);
    ih = Math.floor(devH / k);
    W = iw + 2; H = ih + 2;
    renderer.setSize(devW, devH, false);
    rt.setSize(W * 2, H * 2);
    camera.left = -W / 2; camera.right = W / 2; camera.top = H / 2; camera.bottom = -H / 2;
    camera.updateProjectionMatrix();
    blit.material.uniforms.uUvScale!.value.set(iw / W, ih / H);
    blit.material.uniforms.uTargetEnv!.value.set(W, H);
    stats.k = k; stats.dpr = dpr; stats.device = [devW, devH]; stats.envTexels = [iw, ih]; stats.target = [W * 2, H * 2];
    return true;
  }

  // world position of an entity's feet (bottom-centre), snapped to env texels
  function feet(e: EntState): [number, number] {
    const [lx, ly] = st.lungeOffset(e);
    return [Math.round(e.px * TILE) + TILE / 2 + lx, -Math.round(e.py * TILE) - TILE + 1 - ly];
  }

  function updateCamera(dt: number, snapNow: boolean): void {
    const h = st.hero;
    if (h) { const [fx, fy] = feet(h); cam.tx = fx; cam.ty = fy + TILE; }
    if (snapNow) { cam.x = cam.tx; cam.y = cam.ty; cam.vx = cam.vy = 0; return; }
    // critically damped spring (ζ = 1), semi-implicit Euler, then snap when settled
    const w0 = 9, s = Math.min(dt, 0.05);
    for (const a of ["x", "y"] as const) {
      const v = a === "x" ? "vx" : "vy", t = a === "x" ? "tx" : "ty";
      const acc = w0 * w0 * (cam[t] - cam[a]) - 2 * w0 * cam[v];
      cam[v] += acc * s;
      cam[a] += cam[v] * s;
      if (Math.abs(cam[t] - cam[a]) < 0.02 && Math.abs(cam[v]) < 0.5) { cam[a] = cam[t]; cam[v] = 0; }
    }
  }

  function build(now: number): void {
    const p = paletteFor(st.biome);
    blit.setPalette(p);
    clear.setRGB(p[0]![0], p[0]![1], p[0]![2]);
    const b = st.biome;

    // tiles: one instance per seen tile; visible ones full, remembered ones dimmed
    L.tiles.begin();
    for (let y = 0; y < st.h; y++) for (let x = 0; x < st.w; x++) {
      const i = y * st.w + x;
      if (!st.seen[i]) continue;
      const t = st.tiles[i]!;
      const s = atlas.tile(b, t, ((x * 7 + y * 13) % 11) < 2);
      L.tiles.push(x * TILE + TILE / 2, -(y + 1) * TILE, 0, TILE, TILE, s.u0, s.v0, s.u1, s.v1, st.visible[i] ? 1 : 0.6);
    }
    L.tiles.end();

    // overlays: 2-frame animation at 4 fps (real time)
    const frame = Math.floor(now / 250) & 1;
    L.overlays.begin();
    for (const o of st.overlays) {
      const i = o.y * st.w + o.x;
      if (!st.visible[i]) continue;
      const s = atlas.overlay(o.k, frame);
      L.overlays.push(o.x * TILE + TILE / 2, -(o.y + 1) * TILE, 0.5, TILE, TILE, s.u0, s.v0, s.u1, s.v1);
    }
    L.overlays.end();

    L.items.begin();
    for (const it of st.items) {
      const i = it.y * st.w + it.x;
      if (!st.seen[i]) continue;
      const s = atlas.item(it.known ? it.kind : /scroll/.test(it.label) ? "scroll" : "potion");
      L.items.push(it.x * TILE + TILE / 2, -(it.y + 1) * TILE, 1, TILE, TILE, s.u0, s.v0, s.u1, s.v1, st.visible[i] ? 1 : 0.6);
    }
    L.items.end();

    // entities (sprite density: 1 sprite texel = 0.5 world), shadows, glyphs
    L.shadows.begin(); L.ents.begin(); L.glyphs.begin();
    for (const e of st.ents.values()) {
      const vi = e.y * st.w + e.x;
      if (!e.hero && !st.visible[vi]) continue;
      const [fx, fy] = feet(e);
      const s = atlas.entity(e.kind);
      const w = s.w / 2, h = s.h / 2; // world units (env texels)
      void e.cid;
      const z = 2 + Math.min(1, e.py / Math.max(1, st.h));
      if (!e.dying) {
        // contact shadow; companions (ally + cid, or tamed this run) get a 1-texel light ring
        const ring = e.ally && !e.hero;
        const sh = atlas.shadow(Math.min(w - 2, 12), ring);
        L.shadows.push(fx, fy - (ring ? 2 : 1), 1.5, sh.w, sh.h, sh.u0, sh.v0, sh.u1, sh.v1, 1, 0, e.fade);
      }
      const flash = st.flashing(e) ? 1 : 0;
      L.ents.push(fx, fy, z, s.w / 2, s.h / 2, s.u0, s.v0, s.u1, s.v1, e.ally && !e.hero ? 1.1 : 1, flash, e.fade, e.flip ? 1 : 0);
      if (e.glyph) {
        const g = atlas.glyph(e.glyph);
        L.glyphs.push(fx, fy + h + 2, 3.5, TILE, TILE, g.u0, g.v0, g.u1, g.v1);
      }
    }
    // tame leash: dots along an arc from the hero's chest to the target, revealed over the leash time
    const lp = st.leashProgress();
    if (lp !== null && st.leash) {
      const a = st.ents.get(st.leash.from), t = st.ents.get(st.leash.to);
      if (a && t) {
        const [ax, ay] = feet(a), [tx, ty] = feet(t);
        const d = atlas.dot();
        const n = 9, shown = Math.floor(lp * n);
        for (let i = 1; i <= shown; i++) {
          const u = i / n;
          const x = Math.round(ax + (tx - ax) * u);
          const y = Math.round(ay + 10 + (ty - ay) * u + 4 * u * (1 - u) * 12);
          L.glyphs.push(x, y, 3.6, 2, 2, d.u0, d.v0, d.u1, d.v1);
        }
      }
    }
    L.shadows.end(); L.ents.end(); L.glyphs.end();

    // callout: bitmap text above the hero (env density)
    L.text.begin();
    const hero = st.hero;
    if (st.callout && hero) {
      const text = st.callout.text.toUpperCase();
      const [fx, fy] = feet(hero);
      const width = text.length * FONT_ADVANCE + 1;
      let x = Math.round(fx - width / 2);
      const y = fy + atlas.entity(hero.kind).h / 2 + 6;
      for (const ch of text) {
        const g = atlas.font(ch);
        L.text.push(x + FONT_CELL_W / 2, y, 4, FONT_CELL_W, FONT_CELL_H, g.u0, g.v0, g.u1, g.v1);
        x += FONT_ADVANCE;
      }
    }
    L.text.end();
  }

  function frame(now: number): void {
    if (disposed) return;
    raf = requestAnimationFrame(frame);
    const dt = Math.min(100, now - last);
    last = now;
    const resized = measure();
    st.tick(dt, now);
    const hadHero = !!st.hero;
    updateCamera(dt / 1000, resized && !hadHero);
    if (!st.loaded) return;
    build(now);

    // snap camera to the env-texel grid; remainder → blit UV offset quantised to device px
    const sx = Math.round(cam.x), sy = Math.round(cam.y);
    const fx = Math.round((cam.x - sx) * k) / k, fy = Math.round((cam.y - sy) * k) / k;
    camera.position.set(sx, sy, 100);
    camera.updateMatrixWorld();
    const u = blit.material.uniforms;
    u.uUvOff!.value.set((1 + fx) / W, (1 + fy) / H);
    u.uCam!.value.set(sx, sy);
    const hero = st.hero;
    if (hero) { const [hx, hy] = feet(hero); u.uHero!.value.set(hx, hy + TILE / 2); }
    u.uFade!.value = st.fade;

    renderer.info.reset();
    renderer.setRenderTarget(rt);
    renderer.setClearColor(clear, 1);
    renderer.clear(true, true, false);
    renderer.render(scene, camera);

    renderer.setRenderTarget(null);
    renderer.setViewport(0, 0, devW, devH);
    renderer.setScissorTest(false);
    renderer.setClearColor(clear, 1);
    renderer.clear(true, false, false);
    renderer.setViewport(0, devH - ih * k, iw * k, ih * k);
    renderer.render(blit.scene, blit.camera);
    stats.calls = renderer.info.render.calls;
    stats.triangles = renderer.info.render.triangles;
    stats.pending = st.pending();
  }

  measure();
  raf = requestAnimationFrame(frame);

  return {
    load(snap) {
      st.load(snap);
      updateCamera(0, true);
    },
    apply(evs) { st.apply(evs); },
    setSpeed(n) { st.speed = Math.max(0, n); },
    skipToEvent() { st.skipToEvent(); },
    idle() { return st.idle(); },
    resize() { lastCss = ""; measure(); },
    dispose() {
      disposed = true;
      cancelAnimationFrame(raf);
      for (const l of Object.values(L)) l.dispose();
      blit.dispose();
      rt.dispose();
      env.dispose(); spr.dispose();
      renderer.dispose();
    },
    stats() { return { ...stats }; },
  };
}
