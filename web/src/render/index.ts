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
// Camera (Cut 5 §5). The target is the hero's body; while a hostile is in view or remembered it is the
// midpoint of the hero and the nearest such hostile, clamped so the hero stays inside the middle third of the
// viewport (|hero − target| ≤ iw/6 × ih/6). A critically damped spring follows the target, and the settled
// position snaps to the env-texel grid as before.
//
// Palette split. Everything renders into ONE target with depth; the target's alpha channel tags
// the layer (env quads write a=1, sprite quads a=0.5; blending is off). The blit quantises a=1
// pixels to the biome palette with 4×4 Bayer dither indexed by world texel, and blends a=0.5
// pixels 50% toward their nearest palette colour without dither. One target, one blit.
// Tiles, items, shadows, glyphs and callout text are env (a=1); entities and the gas/fire
// overlays are sprite-tagged (a=0.5) so they keep a readable hue in every biome.
//
// Cut 8A — the fight frame (docs/PLATEAU.md §A): a second camera on the same event stream. `setFrame("fight")` doubles k
// (sprite texel = 1 target px still, so the ink line thickens rather than blurs), frames the hero + the hostiles in view
// (their bounding box, the hero held inside the middle half of the viewport) and adds, in this frame only: hp bars over
// every combatant (`hud` layer, env density), names under hostiles and the firing row as a caption (text layer), a
// 2-tick flash to the palette's brightest on a hit, a 3-tick screen shake (±2 env texels for the hero, ±1 for a foe),
// telegraph glyphs at 2× size. A frame change is a cut (two dark frames), never a tween. An explicit `focus` fixes the
// camera on a tile and fits its radius (never below the map's k, never above 2×).
//
// Cut 14 §3 — the map frame carries the fight: BASE_TEXELS 200 → 160 (a bigger read; the phone bridge in ui/viewer.ts keeps
// its own 150); a stack (≥ 2 entities on one tile) fans ±⅓ tile sideways in both frames and each member's name has its own
// row (the map frame names a stack's members only; the fight frame names every hostile as before); the room the hero stands
// in is lit whole once seen (`roomLit`: a 4-connected flood over room tiles — passable tiles inside a 2×2 passable block, so
// a one-wide corridor and a door end it — recomputed on a hero tile change); `debugRects` / `debugLabels` for the gates.
import * as THREE from "three";
import { Atlas } from "./atlas";
import { Blit } from "./blit";
import { QuadLayer } from "./layers";
import { GpuTimer, Hist } from "./gputimer";
import { paletteFor } from "./palette";
import { FONT_ADVANCE, FONT_CELL_H, FONT_CELL_W } from "./font";
import { PROPS, ReplayState, type EntState } from "./state";
import type { Ev, Snapshot } from "./types";

const cssHex = (c: readonly number[]): string => "#" + c.slice(0, 3).map((v) => Math.round(v * 255).toString(16).padStart(2, "0")).join("");

export type { Ev, Snapshot } from "./types";

export type Frame = "map" | "fight";
export type Focus = { x: number; y: number; radius: number }; // tiles: centre and half-extent of the square to frame

export type Viewer = {
  load(snap: Snapshot): void;   // full floor state; clears the queue; resets camera to hero
  apply(evs: Ev[]): void;       // queue events; played against the tick clock (Ev.t = tick)
  setSpeed(n: number): void;    // 1 = 10 ticks/s, 4 = 40 ticks/s, 0 = pause
  skipToEvent(): void;          // fast-forward to the next attack/die/telegraph/use/exit/tame
  seek(t: number): void;        // tick scrub: rebuild from the last snapshot up to tick t (O(n))
  sync(snap: Snapshot): void;   // Cut 4 §3: adopt `remembered` flags from a step's snapshot (no reload)
  setFrame(frame: Frame, focus?: Focus): void; // Cut 8A: cut between the map and the fight frame (focus: fix the fight camera)
  frame(): Frame;
  tick(): number;               // current tick
  idle(): boolean;              // queue drained and tails played out
  resize(): void;
  dispose(): void;
  stats(): ViewerStats;
  debugEnts?(): unknown[];
  debugPos?(): unknown[];
  debugRects?(): DebugRect[];     // Cut 14 §3: every entity drawn this frame, its on-screen rect in CSS px (the gates measure a foe's height)
  debugLabels?(): DebugLabel[];   // Cut 14 §3: every name drawn this frame (text, its row's bottom in CSS px)
  atlasInfo?(): unknown;
  debugBiome?(): string;          // Cut 16 §3: the biome the floor draws in (its palette)
  preload?(snap: Snapshot): void;   // add unknown entities before a batch's events
};

export type DebugRect = { id: number; kind: string; hero: boolean; x: number; y: number; w: number; h: number; stack: number };   // CSS px; `stack`: members on its tile
export type DebugLabel = { text: string; x: number; y: number; id: number; w?: number; h?: number };   // CSS px: the label's centre x and its cell's bottom y; w · h its box (Cut 15 §4)

export type ViewerStats = {
  calls: number; triangles: number; k: number; dpr: number;
  frame: Frame; kMap: number; // Cut 8A: the frame up and the map frame's k (the fight frame's is `k`)
  shake: [number, number]; glyphs: number; caption: string | null; // Cut 8A: this frame's screen shake, glyph quads, caption
  device: [number, number]; envTexels: [number, number]; target: [number, number]; pending: number; tick: number;
  fade: number;           // the global fade this frame (0 lit, 1 dark; ≤ EXIT_DIM once the run has ended)
  hero: [number, number]; // render position in tiles
  projectiles: number;
  ents: number; drawn: number; // entities known to the viewer, and how many are on screen this frame (in view or remembered)
  camera: [number, number]; // camera target in world units (env texels)
  // timing (see docs/RENDER_PERF.md): cpuMs = build + render wall time this frame; gpuMs = GPU
  // elapsed time for the whole pipeline (NaN without EXT_disjoint_timer_query_webgl2); p95 over
  // the last 120 frames; fps over the last second of rAF callbacks.
  cpuMs: number; cpuP95: number; buildMs: number; gpuMs: number; gpuP95: number; gpuTimer: boolean; fps: number;
};

const TILE = 8;
const REMEMBERED_DIM = 0.5; // Cut 4 §3: a remembered foe, like a memory tile
const CUT_FRAMES = 2;       // Cut 8A: dark frames on a frame change (a cut, not a tween)
const BAR_W = 8;            // Cut 8A: hp bar width in env texels (1 tall)
const BAR_RED = "#c8302c"; // the missing part of an hp bar; the rest is the palette's brightest
const FIGHT_TOP_CSS = 96;   // Cut 8A: the caption sits this many CSS px below the top edge (under the DOM hud)
const BASE_TEXELS = 160; // Cut 14 §3 (was 200, before that 270): ~20 tiles across at 400 CSS px, a 40-texel foe ≥ 24 CSS px tall
const STACK_SPREAD = TILE * 2 / 3; // Cut 14 §3: a stack's members fan over ±⅓ tile
const ROOM_MAX = 600;              // Cut 14 §3: the room flood's cap in tiles (a cave floor is not one room)
const HERO_TEXELS = 24;            // Cut 14 §3: the hero sprite's height in env texels (48 sprite texels); the fight k keeps it ≤ 1/5 of the screen

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
  const gpu = new GpuTimer(renderer.getContext() as WebGL2RenderingContext);
  const cpuHist = new Hist(), buildHist = new Hist();
  const frameTimes: number[] = [];
  let frameNo = 0;

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
    tiles: new QuadLayer(env, 32 * 32 + 64, 1, 0), // a fully seen 32×32 floor plus its props
    overlays: new QuadLayer(env, 512, 0.5, 1), // tagged as sprite so gas/fire keep their hue
    items: new QuadLayer(env, 128, 1, 2),
    shadows: new QuadLayer(env, 128, 1, 3),
    ents: new QuadLayer(spr, 128, 0.5, 4),
    glyphs: new QuadLayer(env, 128, 1, 5),
    text: new QuadLayer(env, 192, 1, 6),  // Cut 8A: callout + caption + a name under each hostile
    hud: new QuadLayer(env, 128, 0.5, 7), // Cut 8A: hp bars (sprite-tagged so the red keeps its hue in every biome)
  };
  for (const l of Object.values(L)) scene.add(l.mesh);
  const blit = new Blit(rt.texture);
  const clear = new THREE.Color();

  const st = new ReplayState();
  const cam = { x: 0, y: 0, vx: 0, vy: 0, tx: 0, ty: 0 };
  const stats: ViewerStats = { calls: 0, triangles: 0, k: 1, dpr: 1, frame: "map", kMap: 1, shake: [0, 0], glyphs: 0, caption: null, device: [0, 0], envTexels: [0, 0], target: [0, 0], pending: 0, tick: 0, fade: 0, hero: [0, 0], projectiles: 0, ents: 0, drawn: 0, camera: [0, 0],
    cpuMs: NaN, cpuP95: NaN, buildMs: NaN, gpuMs: NaN, gpuP95: NaN, gpuTimer: gpu.available, fps: NaN };
  let k = 1, kMap = 1, iw = 1, ih = 1, W = 3, H = 3, devW = 0, devH = 0, dpr = 1;
  let lastCss = "";
  let mode: Frame = "map", fixedFocus: Focus | null = null, cutFrames = 0; // Cut 8A
  let camSX = 0, camSY = 0; // the snapped camera centre this frame (build() places the caption from it)
  // Cut 14 §3: the hero's room, lit whole once seen — a flood from the hero's tile over room tiles (index → 1), recomputed when
  // the hero's logical tile changes (or the floor reloads); empty in a corridor
  let roomLit = new Uint8Array(0), roomKey = "";
  const rects: DebugRect[] = [], labels: DebugLabel[] = [];   // Cut 14 §3: what this frame drew, for the gates
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
    kMap = Math.max(1, Math.floor(Math.min(devW, devH) / baseTexels));
    k = mode === "fight" ? fightK() : kMap;
    iw = Math.floor(devW / k);
    ih = Math.floor(devH / k);
    W = iw + 2; H = ih + 2;
    renderer.setSize(devW, devH, false);
    rt.setSize(W * 2, H * 2);
    camera.left = -W / 2; camera.right = W / 2; camera.top = H / 2; camera.bottom = -H / 2;
    camera.updateProjectionMatrix();
    blit.material.uniforms.uUvScale!.value.set(iw / W, ih / H);
    blit.material.uniforms.uTargetEnv!.value.set(W, H);
    stats.k = k; stats.kMap = kMap; stats.frame = mode; stats.dpr = dpr; stats.device = [devW, devH]; stats.envTexels = [iw, ih]; stats.target = [W * 2, H * 2];
    return true;
  }
  // Cut 8A: the fight frame's k — 2× the map's; with a fixed focus, the largest even k that fits the (2r+1)-tile square on
  // the short axis, never below the map's k (odd k would split sprite texels across device pixels)
  function fightK(): number {
    // Cut 14 §3: never so big that the hero is over a fifth of the screen's height (even k, never below the map's)
    const cap = Math.floor(devH / (5 * HERO_TEXELS));
    const kf = Math.max(kMap, Math.min(2 * kMap, cap - (cap & 1)));
    if (!fixedFocus) return kf;
    const fit = Math.floor(Math.min(devW, devH) / ((2 * Math.max(1, fixedFocus.radius) + 1) * TILE));
    return Math.max(kMap, Math.min(kf, fit - (fit & 1)));
  }

  // world position of an entity's feet (bottom-centre), snapped to env texels
  function feet(e: EntState): [number, number] {
    const [lx, ly] = st.lungeOffset(e);
    return [Math.round(e.px * TILE) + TILE / 2 + lx, -Math.round(e.py * TILE) - TILE + 1 - ly];
  }

  // the nearest hostile the player can see (in view, or remembered at its last seen tile); dying ones let the camera go
  function nearestHostile(h: EntState): EntState | null {
    let best: EntState | null = null, bd = Infinity;
    for (const e of st.ents.values()) {
      if (e.hero || e.ally || e.neutral || e.dying || e.kind === "bones") continue;
      if (!e.remembered && !st.visible[e.y * st.w + e.x]) continue;
      const d = Math.max(Math.abs(e.x - h.x), Math.abs(e.y - h.y));
      if (d < bd) { bd = d; best = e; }
    }
    return best;
  }
  // Cut 8A: a combatant the fight frame keeps in the picture (hostiles in view or remembered; dying ones too)
  function inFight(e: EntState): boolean {
    if (e.hero || e.ally || e.neutral || e.kind === "bones") return false;
    return e.remembered || !!st.visible[e.y * st.w + e.x];
  }
  // Cut 8A: the fight camera's target — the fixed focus, else the centre of the hero + hostiles bounding box — clamped so
  // the hero stays inside the middle half of the viewport
  function fightTarget(hx: number, hy: number): void {
    let cx: number, cy: number;
    if (fixedFocus) { cx = fixedFocus.x * TILE + TILE / 2; cy = -fixedFocus.y * TILE - TILE / 2; }
    else {
      let x0 = hx, x1 = hx, y0 = hy, y1 = hy;
      for (const e of st.ents.values()) {
        if (!inFight(e)) continue;
        const [ex, ey] = feet(e);
        const cy2 = ey + TILE;
        if (ex < x0) x0 = ex; if (ex > x1) x1 = ex; if (cy2 < y0) y0 = cy2; if (cy2 > y1) y1 = cy2;
      }
      cx = (x0 + x1) / 2; cy = (y0 + y1) / 2;
    }
    const lx = iw / 4, ly = ih / 4;
    cam.tx = Math.max(hx - lx, Math.min(hx + lx, cx));
    cam.ty = Math.max(hy - ly, Math.min(hy + ly, cy));
  }
  function updateCamera(dt: number, snapNow: boolean): void {
    const h = st.hero;
    if (h) {
      const [fx, fy] = feet(h);
      const hx = fx, hy = fy + TILE;
      const foe = mode === "fight" ? null : nearestHostile(h);
      if (mode === "fight") fightTarget(hx, hy);
      else if (foe) {
        const [ex, ey] = feet(foe);
        const mx = (hx + ex) / 2, my = (hy + ey + TILE) / 2, lx = iw / 6, ly = ih / 6;
        cam.tx = Math.max(hx - lx, Math.min(hx + lx, mx));
        cam.ty = Math.max(hy - ly, Math.min(hy + ly, my));
      } else { cam.tx = hx; cam.ty = hy; }
    }
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

  // Cut 14 §3: a room tile is a passable tile inside some 2×2 passable block (a one-wide corridor has none); a door is not passable
  // here, so the flood ends at the door. The flood is 4-connected from the hero's tile, capped at ROOM_MAX tiles.
  function passable(i: number): boolean { const t = st.tiles[i]; return t !== undefined && t !== "wall" && t !== "door"; }
  function roomTile(x: number, y: number): boolean {
    const w = st.w, h = st.h;
    if (x < 0 || y < 0 || x >= w || y >= h || !passable(y * w + x)) return false;
    for (const [dx, dy] of [[-1, -1], [0, -1], [-1, 0], [0, 0]] as const) {
      const x0 = x + dx, y0 = y + dy;
      if (x0 < 0 || y0 < 0 || x0 + 1 >= w || y0 + 1 >= h) continue;
      if (passable(y0 * w + x0) && passable(y0 * w + x0 + 1) && passable((y0 + 1) * w + x0) && passable((y0 + 1) * w + x0 + 1)) return true;
    }
    return false;
  }
  function updateRoom(): void {
    const hh = st.hero;
    const key = hh ? `${st.depth}:${st.w}x${st.h}:${hh.x},${hh.y}` : "";
    if (key === roomKey) return;
    roomKey = key;
    if (roomLit.length !== st.w * st.h) roomLit = new Uint8Array(st.w * st.h); else roomLit.fill(0);
    if (!hh || !roomTile(hh.x, hh.y)) return;
    const w = st.w, stack = [hh.y * w + hh.x];
    roomLit[stack[0]!] = 1;
    let n = 0;
    while (stack.length && n < ROOM_MAX) {
      const i = stack.pop()!, x = i % w, y = (i - x) / w;
      n++;
      for (const [dx, dy] of [[1, 0], [-1, 0], [0, 1], [0, -1]] as const) {
        const nx = x + dx, ny = y + dy, j = ny * w + nx;
        if (nx < 0 || ny < 0 || nx >= w || ny >= st.h || roomLit[j] || !roomTile(nx, ny)) continue;
        roomLit[j] = 1; stack.push(j);
      }
    }
    if (n >= ROOM_MAX) roomLit.fill(0);   // not a room: a cave; the memory dim stands
  }
  /** Cut 14 §3: a seen tile's light — 1 in view or inside the hero's room, else the memory's 0.6. */
  const light = (i: number): number => (st.visible[i] || roomLit[i] ? 1 : 0.6);
  /** Cut 14 §3: world → CSS px (the viewport's top-left is the canvas's; the sub-texel blit offset is ignored, ≤ 1 texel). */
  const toCss = (wx: number, wy: number): [number, number] => [((wx - (camSX - iw / 2)) * k) / dpr, (((camSY + ih / 2) - wy) * k) / dpr];

  function build(now: number): void {
    const p = paletteFor(st.clock < st.bossFlashUntil ? "boss_flash" : st.biome);
    blit.setPalette(p);
    clear.setRGB(p[0]![0], p[0]![1], p[0]![2]);
    const b = st.biome;
    const fight = mode === "fight";
    updateRoom();
    rects.length = 0; labels.length = 0;
    const bright = p[p.length - 1]!;
    // Cut 8A: a hit flashes to the palette's brightest in the fight frame; the map keeps its paper white
    if (fight) L.ents.setFlash(bright[0], bright[1], bright[2]); else L.ents.setFlash(0.98, 0.95, 0.9);

    // tiles: one instance per seen tile; visible ones full, remembered ones dimmed. Cut 5 §4 props (shrine, vault,
    // nest) stand on a floor tile in the same layer: shrine flames flicker at 1 Hz, a nest shows awake once woken.
    const propFrame = Math.floor(now / 1000) & 1;
    L.tiles.begin();
    for (let y = 0; y < st.h; y++) for (let x = 0; x < st.w; x++) {
      const i = y * st.w + x;
      if (!st.seen[i]) continue;
      const t = st.tiles[i]!;
      const prop = PROPS.has(t);
      const s = atlas.tile(b, prop ? "floor" : t, ((x * 7 + y * 13) % 11) < 2);
      const dim = light(i);   // Cut 14 §3: the hero's room stays lit
      L.tiles.push(x * TILE + TILE / 2, -(y + 1) * TILE, 0, TILE, TILE, s.u0, s.v0, s.u1, s.v1, dim);
      if (prop) {
        const p = atlas.prop(b, t, t === "nest" ? (st.nestWoken.has(i) ? 1 : 0) : propFrame);
        L.tiles.push(x * TILE + TILE / 2, -(y + 1) * TILE, 0.1, TILE, TILE, p.u0, p.v0, p.u1, p.v1, dim);
      }
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

    // bones piles (Cut 2 §2) arrive as item kind or entity kind `bones`: env density, 2 frames at 1 fps
    const bonesFrame = Math.floor(now / 1000) & 1;
    L.items.begin();
    for (const it of st.items) {
      const i = it.y * st.w + it.x;
      if (!st.seen[i]) continue;
      const s = it.kind === "bones" ? atlas.bones(b, bonesFrame) : atlas.item(it.known ? it.kind : /scroll/.test(it.label) ? "scroll" : "potion");
      L.items.push(it.x * TILE + TILE / 2, -(it.y + 1) * TILE, 1, TILE, TILE, s.u0, s.v0, s.u1, s.v1, light(i));
    }
    for (const e of st.ents.values()) {
      if (e.kind !== "bones") continue;
      const i = e.y * st.w + e.x;
      if (!st.seen[i]) continue;
      const s = atlas.bones(b, bonesFrame);
      L.items.push(e.x * TILE + TILE / 2, -(e.y + 1) * TILE, 1, TILE, TILE, s.u0, s.v0, s.u1, s.v1, light(i));
    }
    L.items.end();

    // entities (sprite density: 1 sprite texel = 0.5 world), shadows, glyphs. A remembered foe (Cut 4 §3) is drawn
    // at its last seen tile dimmed like a memory tile: no shadow, no flash, no telegraph glyph.
    L.shadows.begin(); L.ents.begin(); L.glyphs.begin(); L.hud.begin(); L.text.begin();
    stats.ents = st.ents.size; stats.drawn = 0;
    // Cut 13 §4: the callout's box (clamped inside the frame, see `textX`), so a hostile's name never shares its line (rater Q's
    // `JACKALCKAL`: the callout above the hero and the name under a foe one tile up, on one row)
    const heroEnt = st.hero;
    let calloutBox: [number, number, number, number] | null = null;   // x0, y0, x1, y1 (world; y up)
    if (st.callout && heroEnt) {
      const [hx, hy] = feet(heroEnt);
      const width = st.callout.text.length * FONT_ADVANCE + 1;
      const cy = hy + atlas.entity(heroEnt.kind).h / 2 + (fight ? (heroEnt.glyph ? (fight ? TILE * 2 : TILE) + 6 : 5) : 6);
      const cx = textX(width, hx);
      calloutBox = [cx - width / 2, cy - 1, cx + width / 2, cy + FONT_CELL_H + 1];
    }
    const barBg = fight ? atlas.solid(BAR_RED) : null, barFg = fight ? atlas.solid(cssHex(bright)) : null;
    const gs = fight ? TILE * 2 : TILE; // Cut 8A: telegraph glyphs at 2× in the fight frame
    // Cut 14 §3: a stack — the drawn entities that share a logical tile (the hero's tile included: a foe on it fans off him).
    // Member i of n fans (i/(n−1) − ½) × ⅔ tile sideways, whole texels, and its name takes row i (rater S: "five foes on one
    // tile rendered as one smear", the labels on one line)
    const stacks = new Map<number, number[]>();
    for (const e of st.ents.values()) {
      if (e.kind === "bones" || e.dying) continue;
      const vi = e.y * st.w + e.x;
      if (!e.hero && !st.visible[vi] && !e.remembered) continue;
      const g = stacks.get(vi); if (g) g.push(e.id); else stacks.set(vi, [e.id]);
    }
    const nameScale = fight ? 0.5 : 1;   // the map frame's k is half the fight frame's: the same size on screen
    const nameRow = FONT_CELL_H * nameScale + 1;
    const tagBoxes: [number, number, number, number][] = [];   // Cut 15 §4: the name tags drawn so far this frame (world x0, y0, x1, y1)
    for (const e of st.ents.values()) {
      if (e.kind === "bones") continue; // drawn in the items layer above
      const vi = e.y * st.w + e.x;
      if (!e.hero && !st.visible[vi] && !e.remembered) continue;
      stats.drawn++;
      let [fx, fy] = feet(e);
      const group = stacks.get(vi), stackN = group?.length ?? 1, stackI = group ? group.indexOf(e.id) : 0;
      if (stackN > 1 && stackI >= 0) fx += Math.round((stackI / (stackN - 1) - 0.5) * STACK_SPREAD);
      // Fight frame: a hostile on the tile straight above the hero would vanish behind the hero's tall
      // sprite (its whole body covers that tile). Nudge it half a tile sideways, away from the hero's
      // facing, so both silhouettes read; the shadow stays on the true tile.
      {
        // any hostile whose feet land within one tile above the hero (tweened positions) is nudged
        // sideways so the hero's tall body does not cover it; direction away from the hero's facing
        const hh = st.hero;
        if (fight && !e.hero && hh && !e.dying) {
          const dx = e.px - hh.px, dy = hh.py - e.py;
          // side by side is how every auto-battler frames a melee: clear the hero's sprite width, not a tile
          if (Math.abs(dx) < 0.75 && dy > 0.25 && dy < 1.75) { const hw = atlas.entity(hh.kind).w / 2; fx += (hh.flip ? 1 : -1) * (hw / 2 + 4); }
        }
      }
      const s = atlas.entity(e.kind);
      const w = s.w / 2, h = s.h / 2; // world units (env texels)
      const z = 2 + Math.min(1, e.py / Math.max(1, st.h)) + (stackN > 1 ? stackI * 0.001 : 0);   // a stack's members never z-fight
      { const [cx, cy] = toCss(fx - w / 2, fy + h); rects.push({ id: e.id, kind: e.kind, hero: !!e.hero, x: cx, y: cy, w: (w * k) / dpr, h: (h * k) / dpr, stack: stackN }); }
      if (e.remembered) {
        L.ents.push(fx, fy, z, s.w / 2, s.h / 2, s.u0, s.v0, s.u1, s.v1, REMEMBERED_DIM, 0, e.fade, e.flip ? 1 : 0);
        continue;
      }
      if (!e.dying) {
        // contact shadow; companions (ally + cid, or tamed this run) get a 1-texel light ring
        const ring = st.ringShown(e);
        const sh = atlas.shadow(Math.min(w - 2, 12), ring);
        L.shadows.push(fx, fy - (ring ? 2 : 1), 1.5, sh.w, sh.h, sh.u0, sh.v0, sh.u1, sh.v1, 1, 0, e.fade);
      }
      const flash = st.flashing(e) ? 1 : 0;
      L.ents.push(fx, fy, z, s.w / 2, s.h / 2, s.u0, s.v0, s.u1, s.v1, e.ally && !e.hero ? 1.1 : 1, flash, e.fade, e.flip ? 1 : 0);
      // Cut 8A: in the fight frame every combatant carries an hp bar (BAR_W×1, red under the palette's brightest) 1 texel
      // above its sprite; a hostile has its name under its feet; glyphs sit above the bar
      let top = fy + h + 2;
      if (fight && barBg && barFg && !e.dying && !e.neutral && e.maxHp > 0) {
        const fill = Math.max(0, Math.min(BAR_W, Math.round((BAR_W * e.hp) / e.maxHp)));
        L.hud.push(fx, fy + h + 1, 3.7, BAR_W, 1, barBg.u0, barBg.v0, barBg.u1, barBg.v1);
        if (fill > 0) L.hud.push(fx - BAR_W / 2 + fill / 2, fy + h + 1, 3.8, fill, 1, barFg.u0, barFg.v0, barFg.u1, barFg.v1);
        top = fy + h + 4;
      }
      // Cut 8A: a hostile's name under its feet in the fight frame; Cut 14 §3: in the map frame too when it stands in a stack, and a
      // stack's i-th member takes the i-th row down so no two names share a row
      if (!e.hero && !e.ally && !e.neutral && !e.dying && (fight || stackN > 1)) {
        let ny = fy - FONT_CELL_H * nameScale - 2 - (stackN > 1 && stackI > 0 ? stackI * nameRow : 0);
        const nw = e.name.length * FONT_ADVANCE * nameScale + nameScale, nx = textX(nw, fx), th = FONT_CELL_H * nameScale;
        // Cut 15 §4: a tag whose box would intersect a tag already drawn moves down a row (rater V: `CAPTIMONKEY`), in both frames
        const hits = (y: number): boolean => tagBoxes.some((b) => nx - nw / 2 < b[2] && nx + nw / 2 > b[0] && y < b[3] && y + th > b[1]);
        for (let i = 0; i < 8 && hits(ny); i++) ny -= nameRow;
        const clash = calloutBox && nx + nw / 2 > calloutBox[0] && nx - nw / 2 < calloutBox[2] && ny + th > calloutBox[1] && ny < calloutBox[3];
        if (!clash) {   // Cut 13 §4: the callout has the line for its second
          drawText(e.name, nx, ny, 4, nameScale);
          tagBoxes.push([nx - nw / 2, ny, nx + nw / 2, ny + th]);
          const [cx, cy] = toCss(nx, ny); labels.push({ text: e.name, x: cx, y: cy, id: e.id, w: (nw * k) / dpr, h: (th * k) / dpr });
        }
      }
      if (e.glyph) {
        const g = atlas.glyph(e.glyph);
        L.glyphs.push(fx, top, 3.5, gs, gs, g.u0, g.v0, g.u1, g.v1);
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
    // projectiles: a 2×2 env-texel dot travelling 1 tile per tick along the path
    {
      const d = atlas.dot();
      for (const [px, py] of st.projectilePositions()) {
        L.glyphs.push(Math.round(px * TILE) + TILE / 2, -Math.round(py * TILE) - TILE / 2 - 1, 3.6, 2, 2, d.u0, d.v0, d.u1, d.v1);
      }
    }
    L.shadows.end(); L.ents.end(); L.glyphs.end(); L.hud.end();

    // callout: bitmap text above the hero (env density)
    const hero = st.hero;
    if (st.callout && hero) {
      const [fx, fy] = feet(hero);
      // in the fight frame the callout clears the hp bar and, when up, the 2× glyph above it
      // Cut 13 §4: centred on the hero but kept inside the frame — a hero at the edge used to lose its callout's right half
      fitText(st.callout.text, fx, fy + atlas.entity(hero.kind).h / 2 + (fight ? (hero.glyph ? gs + 6 : 5) : 6), 4, true);
    }
    // Cut 8A: the firing row as a caption at the top of the fight frame (`R2 attack goblin`), under the DOM hud
    if (fight && st.caption) fitText(st.caption.text, camSX, camSY + ih / 2 - Math.ceil((FIGHT_TOP_CSS * dpr) / k) - FONT_CELL_H, 4.1, false);
    L.text.end();
  }

  /** Cut 14 §3: text that never clips — at env density when it fits the frame's width, else at sprite density (half size), else
   *  two rows at half size split at the last space that fits (the caption at a big fight k read `R2 ATTACK JACK`). `y` is the
   *  bottom row's baseline; `follow`: the text wants `cx` and is kept inside the frame (`textX`), else it is centred on `cx`. */
  function fitText(text: string, cx: number, y: number, z: number, follow: boolean): void {
    const room = iw - 2;
    const width = (t: string, sc: number): number => t.length * FONT_ADVANCE * sc + sc;
    const place = (t: string, sc: number, yy: number): void => drawText(t, follow ? textX(width(t, sc), cx) : cx, yy, z, sc);
    if (width(text, 1) <= room) { place(text, 1, y); return; }
    if (width(text, 0.5) <= room) { place(text, 0.5, y); return; }
    const max = Math.max(1, Math.floor((room - 0.5) / (FONT_ADVANCE * 0.5)));
    let cut = text.lastIndexOf(" ", max); if (cut <= 0) cut = max;
    const a = text.slice(0, cut).trimEnd(), b = text.slice(cut).trimStart().slice(0, max);
    place(a, 0.5, y + FONT_CELL_H * 0.5 + 1); if (b) place(b, 0.5, y);
  }
  /** Cut 13 §4: the centre x for a text `width` wide that wants `cx`, kept inside the frame's iw texels (1-texel margin);
   *  a text wider than the frame stays centred on the frame. */
  function textX(width: number, cx: number): number {
    const half = width / 2, lo = camSX - iw / 2 + 1 + half, hi = camSX + iw / 2 - 1 - half;
    return lo > hi ? camSX : Math.max(lo, Math.min(hi, cx));
  }
  // bitmap text centred on x, its baseline (cell bottom) at y. scale 1 = env density; 0.5 = sprite density (one font
  // texel per target px, half the size on screen: the fight frame's names), positions snapped to that grid
  function drawText(text: string, cx: number, y: number, z: number, scale = 1): void {
    const t = text.toUpperCase();
    const adv = FONT_ADVANCE * scale, cw = FONT_CELL_W * scale, chh = FONT_CELL_H * scale;
    const width = t.length * adv + scale;
    let x = Math.round((cx - width / 2) / scale) * scale;
    for (const ch of t) {
      const g = atlas.font(ch);
      L.text.push(x + cw / 2, y, z, cw, chh, g.u0, g.v0, g.u1, g.v1);
      x += adv;
    }
  }

  function frame(now: number): void {
    if (disposed) return;
    raf = requestAnimationFrame(frame);
    frameTimes.push(now);
    while (frameTimes.length > 2 && now - frameTimes[0]! > 1000) frameTimes.shift();
    if (frameTimes.length > 1) stats.fps = (frameTimes.length - 1) / ((now - frameTimes[0]!) / 1000);
    // clamp long stalls (tab switch) but keep 4-fps devices on time; never negative (a rAF stamp can precede
    // the `performance.now()` the viewer was created at, which at 8× would wind the clock back seconds)
    const dt = Math.max(0, Math.min(250, now - last));
    last = now;
    const resized = measure();
    st.tick(dt, now);
    const snapCam = st.cameraSnap || (resized && !st.hero);
    st.cameraSnap = false;
    updateCamera(dt / 1000, snapCam);
    if (!st.loaded) return;
    const t0 = performance.now();
    // snap camera to the env-texel grid; remainder → blit UV offset quantised to device px. Cut 8A: the fight frame's
    // screen shake displaces the snapped centre by whole texels
    const [shx, shy] = mode === "fight" ? st.shakeOffset() : [0, 0];
    const sx = Math.round(cam.x) + shx, sy = Math.round(cam.y) + shy;
    camSX = sx; camSY = sy;
    build(now);
    const t1 = performance.now();

    const fx = Math.round((cam.x + shx - sx) * k) / k, fy = Math.round((cam.y + shy - sy) * k) / k;
    camera.position.set(sx, sy, 100);
    camera.updateMatrixWorld();
    const u = blit.material.uniforms;
    u.uUvOff!.value.set((1 + fx) / W, (1 + fy) / H);
    u.uCam!.value.set(sx, sy);
    const hero = st.hero;
    if (hero) { const [hx, hy] = feet(hero); u.uHero!.value.set(hx, hy + TILE / 2); }
    u.uFade!.value = cutFrames > 0 ? 1 : st.fade; // Cut 8A: a frame change is a cut through dark
    if (cutFrames > 0) cutFrames--;
    u.uFog!.value.set(st.vision - 1, st.vision + 0.5); // fog bands follow the floor's vision (Cut 3: the Deep is 4)

    renderer.info.reset();
    gpu.begin();
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
    gpu.end();
    const t2 = performance.now();
    buildHist.push(t1 - t0); cpuHist.push(t2 - t0);
    stats.buildMs = t1 - t0; stats.cpuMs = t2 - t0; stats.gpuMs = gpu.ms; stats.gpuTimer = gpu.available;
    if ((++frameNo & 31) === 0) { stats.cpuP95 = cpuHist.pct(0.95); stats.gpuP95 = gpu.pct(0.95); }
    stats.calls = renderer.info.render.calls;
    stats.triangles = renderer.info.render.triangles;
    stats.pending = st.pending(); stats.fade = u.uFade!.value as number;
    stats.tick = st.tickNow();
    if (hero) stats.hero = [hero.px, hero.py];
    stats.camera = [cam.tx, cam.ty];
    stats.projectiles = st.projectilePositions().length;
    stats.shake = [shx, shy]; stats.glyphs = L.glyphs.count; stats.caption = st.caption?.text ?? null;
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
    seek(t) { st.seek(t); },
    sync(snap) { st.sync(snap); },
    setFrame(f, focus) {
      const changed = f !== mode;
      if (!changed && !focus && !fixedFocus) return;
      mode = f; fixedFocus = f === "fight" && focus ? { ...focus } : null;
      st.fight = f === "fight";
      lastCss = ""; measure();
      cutFrames = CUT_FRAMES;
      updateCamera(0, true);
    },
    frame() { return mode; },
    tick() { return st.tickNow(); },
    idle() { return st.idle(); },
    resize() { lastCss = ""; measure(); },
    dispose() {
      disposed = true;
      cancelAnimationFrame(raf);
      for (const l of Object.values(L)) l.dispose();
      blit.dispose();
      gpu.dispose();
      rt.dispose();
      env.dispose(); spr.dispose();
      renderer.dispose();
    },
    preload(snap) { st.preload(snap); },
    atlasInfo() { const out: Record<string, unknown> = {}; for (const k of ["hero_fighter", "monkey", "goblin", "jackal"]) { const e = atlas.entity(k); out[k] = { w: e.w, h: e.h, u0: +e.u0.toFixed(3), v0: +e.v0.toFixed(3), u1: +e.u1.toFixed(3), v1: +e.v1.toFixed(3), fallback: (e as { fallback?: boolean }).fallback ?? "?" }; } return out; },
    debugRects() { return rects.map((r) => ({ ...r })); },
    debugBiome() { return st.biome; },
    debugLabels() { return labels.map((l) => ({ ...l })); },
    debugPos() { return [...st.ents.values()].filter((e) => !e.dying).map((e) => ({ kind: e.kind, hero: !!e.hero, ally: !!e.ally, x: e.x, y: e.y, px: +e.px.toFixed(2), py: +e.py.toFixed(2), flip: !!e.flip })); },
    stats() { return { ...stats }; },
    /** dev: every entity the state holds and whether the draw loop would show it */
    debugEnts() { return [...st.ents.values()].map((e) => ({ id: e.id, kind: e.kind, x: e.x, y: e.y, hero: !!e.hero, rem: !!e.remembered, dying: !!e.dying, vis: !!st.visible[e.y * st.w + e.x], seen: !!st.seen[e.y * st.w + e.x] })); },
  };
}
