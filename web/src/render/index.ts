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
// Tiles, items, shadows, glyphs and callout text are env (a=1; art pass: items, shadows, glyphs and text a=0.875 — env,
// quantised, but outside the world's ambient drop and torch light, so a callout never dims); entities and the gas/fire
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
import { Atlas, type Slot } from "./atlas";
import { Blit } from "./blit";
import { QuadLayer } from "./layers";
import { GpuTimer, Hist } from "./gputimer";
import { ETHEREAL, paletteFor, SPRITE_SCALE } from "./palette";
import { CALL_CHAR, CALL_H, CALL_PAD, TAG_CHAR, TAG_H, TAG_PAD, TagLayer, allyName, type Plate, type Tag } from "./tags";
import { PROPS, ReplayState, type EntState } from "./state";
import { Quality } from "./quality";
import { LightField, MAX_FIELD, type FieldLight } from "./light";
import { Bloom, EMISSIVE_TAG } from "./bloom";
import { SpriteNormals } from "./normals";
import { Juice } from "./fx";
import { View2D } from "./view2d";
export { createFallbackViewer } from "./fallback";
import type { Ev, Snapshot } from "./types";

/** deterministic per-tile hash in [0, 1) (render-only dressing; never game truth) */
function hash2(x: number, y: number, s: number): number {
  let h = (x * 374761393 + y * 668265263 + s * 2246822519) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

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
  debugStairs?(): { x: number; y: number; text: string; taken: boolean }[];   // Cut 26 §2: a fork floor's stairs drawn this frame (their plates)
  debugText?(): { kind: "callout" | "caption"; text: string; x?: number; y?: number; w?: number; h?: number }[];   // Cut 18 §2: the lines of text drawn over the fight this frame (Cut 28 §4: + its box, CSS px)
  setKeepOut?(rects: { x: number; y: number; w: number; h: number }[]): void;   // Cut 28 §4: the DOM's chips and plates over the canvas (CSS px, the canvas's) — no pixel text lands on them
  setQuiet?(on: boolean): void;   // Cut 22: a held beat's line is the one line — no callout or caption drawn over the fight meanwhile
  atlasInfo?(): unknown;
  debugBiome?(): string;          // Cut 16 §3: the biome the floor draws in (its palette)
  preload?(snap: Snapshot): void;   // add unknown entities before a batch's events
};

export type DebugRect = { id: number; kind: string; hero: boolean; x: number; y: number; w: number; h: number; stack: number; z: number };   // CSS px; `stack`: members on its tile; `z`: its depth (higher draws in front)
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
  fx: string;   // juice (docs/JUICE.md): the effects tier this frame (low · med · high)
  glLost: boolean;   // cohort 24 (AW): the GL context is gone — the clock runs on, the 2D view (view2d.ts) stands in until it is restored
};

const TILE = 8;
const WALL_TOP_DIM = 0.72; // second art pass: a wall top a step under the floor
const MEMORY_DIM = 0.68;  // second art pass: a remembered tile (Cut 14 §3; was 0.6)
const FLASH_MIX = 0.5;     // QA 1a2a4a9: a hit's flash, the share mixed toward the palette's brightest (was 1: a cream silhouette)
const HERO_FLASH = 0.16;     // gfx round 3: the hero's hurt flash (FLASH_MIX washed him pale on every blow of a long fight); round 10 0.28 → 0.16 (raters: "the hero a pale noisy blob" at half size)
const BOSS_FALL_FLASH = 0.2; // juice pass 3: the flash on a boss's killing blow (the slow-mo holds it; FLASH_MIX washed him out)
const REMEMBERED_DIM = 0.5; // Cut 4 §3: a remembered foe, like a memory tile
const CUT_FRAMES = 0;       // Cut 8A: dark frames on a frame change (a cut, not a tween). gfx round 3: 0 — the cut stays a cut, the two black frames
                            // read as "a full black blank in the strip" to every blind rater
const BAR_W = 8;            // Cut 8A: hp bar width in env texels (1 tall)
const BAR_RED = "#c8302c"; // the missing part of an hp bar; the rest is the palette's brightest
const FIGHT_TOP_CSS = 96;   // Cut 8A: the caption sits this many CSS px below the top edge (under the DOM hud)
const BASE_TEXELS = 160; // Cut 14 §3 (was 200, before that 270): ~20 tiles across at 400 CSS px, a 40-texel foe ≥ 24 CSS px tall
const STACK_SPREAD = TILE * 2 / 3; // Cut 14 §3: a stack's members fan over ±⅓ tile
const ROOM_MAX = 600;              // Cut 14 §3: the room flood's cap in tiles (a cave floor is not one room)
/** 307dbed control rater AR ("the ogre drew as a checkerboard blob"): a sprite's fade (a death's dissolve, a spawn, the fallen hero's
 *  half) was a screen door on the pixel grid — at ½ a checkerboard over the sprite, held on a paused last frame. It darkens instead:
 *  the sprite stays whole and readable, its light going (the fallen hero at ½ reads dimmed, never checkered). */
const CORPSE_T = 4;                // gfx round 10: ticks after a boss's killing blow before his death pose replaces him
const fadeDim = (e: { fade: number }): number => e.fade <= 0 ? 1 : Math.max(0.12, 1 - e.fade * 0.9);
/** the cast of the hero's and the torches' light per biome (rgb multipliers on the warm amber; the Warrens and the Burrows keep it) */
const LIGHT_TINT: Record<string, [number, number, number]> = {
  default: [1, 1, 1], fens: [0.95, 0.92, 0.8],   // gfx round 18 (raters: the Fens "no light source, the room feels dead"): lantern-warm torches (was cold 0.6/1/1.1)
  crypt: [0.8, 0.88, 1.2], deep: [0.7, 0.85, 1.25], sanctum: [0.95, 0.95, 1.05], foundry: [1.08, 0.9, 0.8],
};
const HERO_Z = 3.2, HERO_COVER = 0.3, BOSS_COVER = 0.1, HIDDEN_MAX = 0.5; // Cut 18 §2: the hero's depth (over every sprite, under the glyphs) and the most of his rect a sprite may cover
const MAX_LIGHTS = 12;             // art pass: torches lighting the blit (nearest the camera)
const ROOM_LOOK = 9;               // gfx round 5: the camera's lean looks this many tiles around the hero
const ATLAS_WAIT_MS = 1500;        // gfx round 6: the longest the first frame waits for the atlas
const PLATE_MAX = 5;               // gfx round 1: name plates in a crowd (the nearest hostiles; round 6: 3 → 5, the stacking rule culls a pile)
const BOSS_TITLE_MS = 2200;        // gfx round 1: the boss's title plate on his entrance
const HERO_TEXELS = 24 * SPRITE_SCALE;   // Cut 14 §3: the hero sprite's height in env texels (48 sprite texels; gfx round 7: × SPRITE_SCALE); the fight k keeps it ≤ 1/5 of the screen

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
  let gpu = new GpuTimer(renderer.getContext() as WebGL2RenderingContext);
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
    items: new QuadLayer(env, 128, 0.875, 2),
    shadows: new QuadLayer(env, 128, 0.875, 3),
    ents: new QuadLayer(spr, 128, 0.5, 4),
    glyphs: new QuadLayer(env, 128, 0.875, 5),
    text: new QuadLayer(env, 192, 0.875, 6),  // Cut 8A: callout + caption + a name under each hostile
    hud: new QuadLayer(env, 128, 0.5, 7), // Cut 8A: hp bars (sprite-tagged so the red keeps its hue in every biome)
    decor: new QuadLayer(env, 512, 1, 0),       // art pass: floor decals (moss, cracks, rubble, bones) and props (barrel, crate, pot)
    decorHue: new QuadLayer(env, 256, 0.5, 1),  // art pass: blood, torches, banners — sprite-tagged so red and flame keep their hue
    ghosts: new QuadLayer(spr, 64, 0.5, 4, true),  // gfx round 10: ethereal sprites, half over what is under them (FX > 0; `low` draws them in `ents`)
  };
  for (const l of Object.values(L)) scene.add(l.mesh);
  const blit = new Blit(rt.texture);
  const clear = new THREE.Color();

  const st = new ReplayState();
  const cam = { x: 0, y: 0, vx: 0, vy: 0, tx: 0, ty: 0 };
  let held: [number, number] | null = null;   // gfx round 22: the map camera's held framing (updateCamera)
  const stats: ViewerStats = { calls: 0, triangles: 0, k: 1, dpr: 1, frame: "map", kMap: 1, shake: [0, 0], glyphs: 0, caption: null, device: [0, 0], envTexels: [0, 0], target: [0, 0], pending: 0, tick: 0, fade: 0, hero: [0, 0], projectiles: 0, ents: 0, drawn: 0, camera: [0, 0],
    cpuMs: NaN, cpuP95: NaN, buildMs: NaN, gpuMs: NaN, gpuP95: NaN, gpuTimer: gpu.available, fps: NaN, fx: "low", glLost: false };
  let k = 1, kMap = 1, iw = 1, ih = 1, W = 3, H = 3, devW = 0, devH = 0, dpr = 1;
  let lastCss = "";
  let mode: Frame = "map", fixedFocus: Focus | null = null, cutFrames = 0; // Cut 8A
  let camSX = 0, camSY = 0; // the snapped camera centre this frame (build() places the caption from it)
  // Cut 14 §3: the hero's room, lit whole once seen — a flood from the hero's tile over room tiles (index → 1), recomputed when
  // the hero's logical tile changes (or the floor reloads); empty in a corridor
  let roomLit = new Uint8Array(0), roomKey = "";
  const ghostAt: [number, number][] = [];   // gfx round 10: this frame's ethereal sprites (world), for their light
  const rects: DebugRect[] = [], labels: DebugLabel[] = [];   // Cut 14 §3: what this frame drew, for the gates
  const texts: { kind: "callout" | "caption"; text: string; x?: number; y?: number; w?: number; h?: number }[] = [];   // Cut 18 §2
  // Cut 28 §4 (AU, AV: `R3 BANK` over the docked fold chips `gas>pack · bp 5/40`; `archer` printed across `R7 ATTACK ARCHER`): the
  // watch's DOM over the canvas (the HUD, the fold line's chips, the banner, the ticker) as keep-out rects in CSS px — the callout,
  // the caption and the name plates are placed off them (and the plates off the caption)
  let keepOut: { x: number; y: number; w: number; h: number }[] = [];
  let quiet = false;   // Cut 22: `setQuiet` — the watch's held beat owns the line
  const tags: Tag[] = [], plates: (Plate | null)[] = [null, null, null], tagLayer = new TagLayer(canvas);
  const numCss: { x: number; y: number; text: string; col: readonly number[]; sc: number; a: number; big: boolean }[] = [];   // gfx round 6: this frame's numbers (CSS px)
  let bossTitle: { text: string; until: number } | null = null; const bossTitled = new Set<number>();   // gfx round 1: the boss's title plate   // second art pass: the hostiles' serif name plates (DOM)
  const lights: [number, number][] = [];   // art pass: this frame's torch flames (world env texels)
  const candles: [number, number][] = [];   // gfx round 10: candle clusters (a small warm light each)
  const syncPx = new Uint8Array(4);
  // juice (docs/JUICE.md): the effects tier, the light field, bloom and the event-driven feedback; `low` keeps the pre-juice look
  const quality = new Quality(renderer.getContext());
  const field = new LightField();
  const bloom = new Bloom(rt.texture);
  const juice = new Juice(env, quality, () => atlas.solid("#ffffff"), (ch) => atlas.font(ch), () => atlas.coin());
  const normals = new SpriteNormals(spr);   // juice pass 2: derived sprite normals, drawn by a twin of the entity layer (high)
  normals.add(L.ents.twin(normals.material));
  scene.add(juice.emit.mesh, juice.matte.mesh, juice.nums.mesh);
  st.onEvent = (ev) => juice.onEvent(ev);
  const fieldLights: FieldLight[] = [];
  const stairsSeen: [number, number][] = [];                                       // Cut 26 §2: this frame's seen down stairs
  const stairPlates: { x: number; y: number; text: string; taken: boolean }[] = [];   // …and the plates a fork floor gave them
  const fires: [number, number][] = [], gases: [number, number][] = [], waters: [number, number][] = [];
  let fxLevel = -1, simDt = 0;
  function applyFx(): void {
    const lv = quality.at("high") ? 2 : quality.at("med") ? 1 : 0;
    if (lv === fxLevel) return;
    fxLevel = lv; stats.fx = quality.fx;
    blit.setFx(lv);
    const u = blit.material.uniforms;
    u.uLightMap!.value = field.target.texture; u.uBloom!.value = bloom.a.texture; u.uNormal!.value = normals.target.texture;
    // emissive layers (torch flames, fire and gas) write the bloom's tag; the pre-juice blit treats it as a sprite, like 0.5
    for (const l of [L.decorHue, L.overlays]) ((l.mesh.material as THREE.ShaderMaterial).uniforms.uTag!.value = lv ? EMISSIVE_TAG : 0.5);
    if (!lv) juice.reset();
  }
  quality.onChange = () => applyFx();
  applyFx();
  let raf = 0;
  let last = performance.now();
  let disposed = false;
  // cohort 24 (AW: `CONTEXT_LOST_WEBGL` froze the watch for 7 minutes, ▶▶| dead until a reload): a lost context never stops the
  // clock — the frame loop keeps ticking the replay (the watch's pacing reads `tick()` / `idle()`), draws a 2D view of it over the
  // canvas meanwhile, and on `webglcontextrestored` three.js re-creates its GL state (textures, the atlas, render targets and the
  // light / bloom / normal passes re-upload on first use); the GPU timer and the keyed uploads are rebuilt here
  let glLost = false, view2d: View2D | null = null;
  const onLost = (e: Event): void => {
    e.preventDefault();   // ask for a restore
    if (disposed) return;
    glLost = true; stats.glLost = true;
    tagLayer.sync([]); tagLayer.plates([]); tagLayer.numbers([]);
    (view2d ??= new View2D({ host: canvas })).show(true);
  };
  const onRestored = (): void => {
    if (disposed) return;
    glLost = false; stats.glLost = false;
    gpu = new GpuTimer(renderer.getContext() as WebGL2RenderingContext); stats.gpuTimer = gpu.available;
    env.needsUpdate = true; spr.needsUpdate = true;
    field.invalidate(); normals.invalidate();
    lastCss = ""; measure();
    view2d?.show(false);
  };
  canvas.addEventListener("webglcontextlost", onLost, false);
  canvas.addEventListener("webglcontextrestored", onRestored, false);

  // gfx round 6 (raters: "a white 'F' placeholder silhouette" — the edit's scene drew its first frames before the atlas landed): the
  // viewer waits for the atlas up to ATLAS_WAIT_MS (a dark frame, as between floors); past that, or on a failed load, the primitives draw
  let atlasReady = false; const born = performance.now();
  void atlas.load(opts.atlasUrl ?? "/art/atlas.json").finally(() => { atlasReady = true; });

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
    normals.setSize(W * 2, H * 2);
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
    // gfx round 6 (raters: "tiny sprites in a black box" — the edit's scene is ~210 CSS px tall): a small view (< 320 CSS px) lets
    // the hero take up to 2/5 of its height
    const small = (canvas.clientHeight || 1000) < 320;
    const cap = Math.floor(devH / ((small ? 2.5 : 5) * HERO_TEXELS));
    // gfx round 7 (pick raters O, P: at half-size sprites the fight's 2× zoom "smears the hero and rat into blocky noise"): ≤ 1.25× the map's
    // (raters Q, R on the scene's inset: "zoom the inset so the hero and foe are twice their size" — ui/viewer.ts halves a small view's texels; under 480 CSS px 1.5×)
    const most = small ? 2 * kMap : (canvas.clientHeight || 1000) < 480 ? Math.round(kMap * 1.5) : Math.max(kMap, Math.round(kMap * 1.25));
    const kf = Math.max(kMap, Math.min(most - (most & 1), cap - (cap & 1)));
    if (!fixedFocus) return kf;
    const fit = Math.floor(Math.min(devW, devH) / ((2 * Math.max(1, fixedFocus.radius) + 1) * TILE));
    return Math.max(kMap, Math.min(kf, fit - (fit & 1)));
  }

  // world position of an entity's feet (bottom-centre), snapped to env texels
  /** A world point (feet coordinates) over a seen, open tile. */
  function openFeet(x: number, y: number): boolean {
    const tx = Math.floor(x / TILE), ty = Math.round((1 - TILE - y) / TILE);
    if (tx < 0 || ty < 0 || tx >= st.w || ty >= st.h) return false;
    const i = ty * st.w + tx;
    return !!st.seen[i] && st.tiles[i] !== "wall";
  }
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
      } else {
        // gfx round 5 (raters: "the hero crammed at the edge, the lower half an empty void"): with no hostile to frame, the camera leans
        // toward the middle of the ground he can see (the seen open tiles within ROOM_LOOK tiles), at most a sixth of the view — the
        // explored floor fills the frame; nothing unseen is drawn
        const [ox, oy] = seenLean(h);
        cam.tx = hx + Math.max(-iw / 6, Math.min(iw / 6, ox)); cam.ty = hy + Math.max(-ih / 6, Math.min(ih / 6, oy));
      }
      if (mode !== "fight" && !fixedFocus) {
        frameSeen(hx, hy);
        // gfx round 22 (rater AW: "the camera pan reds the whole map", "the whole screen swims"): the map camera holds still while its
        // framing moves less than a fifth of the view across / a sixth down, then glides to the new framing in one move — the room
        // stays put while the hero and his foes move in it
        if (!snapNow && held && Math.abs(cam.tx - held[0]) < iw * 0.2 && Math.abs(cam.ty - held[1]) < ih * 0.16) { cam.tx = held[0]; cam.ty = held[1]; }
        else held = [cam.tx, cam.ty];
      } else held = null;
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

  /** gfx round 22 (raters: "on early floors the void eats 40 % of the screen"): the map camera keeps the explored floor in frame — on an
   *  axis where the seen tiles span less than the view, the view centres on them; where they span more, its edge never passes theirs;
   *  either way the hero stays within the middle ~45 % of the view. */
  let boxKey = "", box: [number, number, number, number] = [0, 0, 0, 0];
  function frameSeen(hx: number, hy: number): void {
    const h = st.hero; if (!h) return;
    const key = `${st.depth}:${h.x},${h.y}`;
    if (key !== boxKey) {
      boxKey = key;
      let x0 = st.w, x1 = -1, y0 = st.h, y1 = -1;
      for (let y = 0; y < st.h; y++) for (let x = 0; x < st.w; x++) if (st.seen[y * st.w + x]) { if (x < x0) x0 = x; if (x > x1) x1 = x; if (y < y0) y0 = y; if (y > y1) y1 = y; }
      box = x1 < 0 ? [0, 0, 0, 0] : [x0 * TILE, (x1 + 1) * TILE, -(y1 + 1.5) * TILE, -(y0 - 0.5) * TILE];   // world: left, right, bottom, top
    }
    if (box[1] <= box[0]) return;
    const fit = (t: number, lo: number, hi: number, half: number, hero: number): number => {
      const c = hi - lo <= 2 * half ? (lo + hi) / 2 : Math.max(lo + half, Math.min(hi - half, t));
      const m = half * 0.45;   // the hero stays in the middle ~45 % of the view (a sixth put him under the top bar's lines)
      return Math.max(hero - m, Math.min(hero + m, c));
    };
    cam.tx = fit(cam.tx, box[0], box[1], iw / 2, hx);
    cam.ty = fit(cam.ty, box[2], box[3], ih / 2, hy);
  }
  /** gfx round 5: half the offset (world units) from the hero to the centroid of the seen open tiles around him (cached per hero tile). */
  let leanKey = "", lean: [number, number] = [0, 0];
  function seenLean(h: EntState): [number, number] {
    const key = `${st.depth}:${h.x},${h.y}:${st.seen.length}`;
    if (key === leanKey) return lean;
    leanKey = key;
    let sx = 0, sy = 0, n = 0;
    for (let y = Math.max(0, h.y - ROOM_LOOK); y <= Math.min(st.h - 1, h.y + ROOM_LOOK); y++) for (let x = Math.max(0, h.x - ROOM_LOOK); x <= Math.min(st.w - 1, h.x + ROOM_LOOK); x++) {
      const i = y * st.w + x; if (!st.seen[i] || st.tiles[i] === "wall") continue; sx += x; sy += y; n++;
    }
    lean = n < 6 ? [0, 0] : [((sx / n - h.x) * TILE) * 0.5, (-(sy / n - h.y) * TILE) * 0.5];
    return lean;
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
  /** Cut 14 §3: a seen tile's light — 1 in view or inside the hero's room, else the memory's MEMORY_DIM (second art pass: 0.68, was 0.6 — the blit no longer re-quantises, so the dim is a plain multiply). */
  const light = (i: number): number => (st.visible[i] || roomLit[i] ? 1 : MEMORY_DIM);
  /** Cut 14 §3: world → CSS px (the viewport's top-left is the canvas's; the sub-texel blit offset is ignored, ≤ 1 texel). */
  const toCss = (wx: number, wy: number): [number, number] => [((wx - (camSX - iw / 2)) * k) / dpr, (((camSY + ih / 2) - wy) * k) / dpr];
  /** Cut 28 §4: the keep-out rects as world boxes (x0, y0, x1, y1; y up). */
  const keepBoxes = (): [number, number, number, number][] => keepOut.map((r) => {
    const x0 = camSX - iw / 2 + (r.x * dpr) / k, x1 = camSX - iw / 2 + ((r.x + r.w) * dpr) / k;
    const yTop = camSY + ih / 2 - (r.y * dpr) / k, yBot = camSY + ih / 2 - ((r.y + r.h) * dpr) / k;
    return [x0, yBot, x1, yTop];
  });
  const cssBox = (b: [number, number, number, number] | null): { x?: number; y?: number; w?: number; h?: number } => {
    if (!b) return {};
    const [x0, y0] = toCss(b[0], b[3]), [x1, y1] = toCss(b[2], b[1]);
    return { x: x0, y: y0, w: x1 - x0, h: y1 - y0 };
  };
  const boxHit = (a: [number, number, number, number], b: [number, number, number, number]): boolean => a[0] < b[2] && a[2] > b[0] && a[1] < b[3] && a[3] > b[1];

  // ---- art pass (art/ui/ART_GAP.md): register 3 dressing, render-only, seeded by tile position ----------------------------
  const isWall = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < st.w && y < st.h && st.tiles[y * st.w + x] === "wall";
  /** a seen tile that is open (not wall) — a wall top rims the sides that meet one */
  const openSeen = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < st.w && y < st.h && !!st.seen[y * st.w + x] && st.tiles[y * st.w + x] !== "wall";
  /** a wall whose tile below is seen open ground shows its front face (capstone + courses); any other wall shows its top */
  const wallFace = (x: number, y: number): boolean => isWall(x, y) && openSeen(x, y + 1);
  function envTileFor(b: string, x: number, y: number, t: string, prop: boolean): Slot | undefined {
    const h = hash2(x, y, 11);
    if (t === "wall") {
      if (wallFace(x, y)) return atlas.envTile(b, h < 0.72 ? "wall_face_0" : "wall_face_1");
      const mask = (openSeen(x, y - 1) ? 1 : 0) | (openSeen(x + 1, y) ? 2 : 0) | (openSeen(x - 1, y) ? 8 : 0);
      return atlas.wallTop(b, mask);
    }
    if (t === "floor" || prop) return atlas.envTile(b, h < 0.46 ? "floor_0" : h < 0.76 ? "floor_1" : h < 0.92 ? "floor_2" : "floor_3");
    if (t === "door" && !portcullis(x, y)) return atlas.envTile(b, h < 0.5 ? "floor_0" : "floor_1");   // an open doorway
    return atlas.envTile(b, t);   // door (portcullis), stairs, water, chasm
  }
  const isWater = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < st.w && y < st.h && st.tiles[y * st.w + x] === "water";
  const isDoor = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < st.w && y < st.h && st.tiles[y * st.w + x] === "door";
  /** second art pass: a portcullis only where it reads as one — a door in a horizontal wall (wall or door on both sides); a run
   *  of doors (the generator's room mouths) gets one, at its middle; a door in a vertical wall is an open doorway (the
   *  portcullis art is front-on). Render-only: the tile is a door either way. */
  function portcullis(x: number, y: number): boolean {
    if (isDoor(x, y - 1) || isDoor(x, y + 1)) return false;
    const l = isWall(x - 1, y) || isDoor(x - 1, y), r = isWall(x + 1, y) || isDoor(x + 1, y);
    if (!l || !r) return false;
    let x0 = x, x1 = x;
    while (isDoor(x0 - 1, y) && x - x0 < 16) x0--;
    while (isDoor(x1 + 1, y) && x1 - x < 16) x1++;
    if (x1 === x0) return true;
    if (!(isWall(x0 - 1, y) && isWall(x1 + 1, y))) return false;
    return x === Math.floor((x0 + x1) / 2);
  }
  /** decals, props and wall dressing for one seen tile (deterministic in x, y; nothing here is game truth) */
  function dress(b: string, x: number, y: number, t: string, dim: number, torchFrame: number): void {
    const wx = x * TILE + TILE / 2, wy = -(y + 1) * TILE;
    if (t === "wall") {
      if (!wallFace(x, y) || st.tiles[(y + 1) * st.w + x] !== "floor") return;
      if ((x + 2 * y) % 4 === 0 && hash2(x, y, 3) < 0.85) {   // gfx round 2: a sconce every ~4 wall faces (was 5; watch.png's rooms are ringed with them)
        const f = atlas.hue(`torch_${torchFrame}`) ?? atlas.hue("torch_0");
        if (!f) return;
        L.decorHue.push(wx, wy + 1, 0.35, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, Math.max(dim, 0.85));
        lights.push([wx, wy + f.h / 2 - 2]);
      } else if (hash2(x, y, 5) < 0.09 && isWall(x - 1, y) && isWall(x + 1, y)) {
        const f = atlas.hue("banner");
        if (f) L.decorHue.push(wx, wy + 1, 0.35, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim);
      }
      return;
    }
    // gfx round 22 (raters, every round: the Fens "a flat undressed teal field"): the Fens dress with their own pieces — lily pads on
    // the water, reeds along its edge, logs and stumps in the corners, a ruined pillar or a warm lantern post against a wall
    const fens = b === "fens" && !!atlas.envTile(b, "reeds");
    if (fens && t === "water" && hash2(x, y, 17) < 0.2) {
      const f = atlas.envTile(b, "lilies");
      if (f) L.decor.push(wx, wy, 0.2, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim);
      return;
    }
    if (t !== "floor") return;
    const n = isWall(x, y - 1), sw = isWall(x, y + 1), e = isWall(x + 1, y), w = isWall(x - 1, y);
    const walls = +n + +sw + +e + +w, corner = (n || sw) && (e || w);
    const hp = hash2(x, y, 7);
    const free = !st.items.some((it) => it.x === x && it.y === y);
    if (fens && free) {
      const wet = +isWater(x - 1, y) + +isWater(x + 1, y) + +isWater(x, y - 1) + +isWater(x, y + 1);
      if (wet && hash2(x, y, 18) < 0.34) {   // (round 22b: 0.6 → 0.34, raters: "thin out the weed props")
        const f = atlas.envTile(b, hash2(x, y, 19) < 0.55 ? "reeds" : "reeds_1");
        if (f) { L.decor.push(wx, wy, 0.3, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim); return; }
      }
      if (walls === 0 && hash2(x, y, 20) < 0.05) {
        const f = atlas.envTile(b, hash2(x, y, 21) < 0.7 ? "reeds_1" : "stump");
        if (f) { L.decor.push(wx, wy, 0.25, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim); return; }
      }
    }
    if (walls >= 1 && walls <= 2 && (corner ? hp < 0.78 : hp < 0.15) && free) {   // (round 21: set dressing denser — raters: "bare grey tile fields")   // gfx round 2: props at more corners and along walls
      // gfx round 10 (raters: "rooms are empty brown grids — barrels, bones, banners"): skull piles and chests join the corners
      const k = hash2(x, y, 8), name = fens ? (k < 0.22 ? "log" : k < 0.42 ? "stump" : k < 0.62 ? "reeds" : k < 0.76 ? "barrel" : k < 0.88 ? "pot" : "skulls")
        : k < 0.34 ? "barrel" : k < 0.6 ? "crate" : k < 0.74 ? "pot" : k < 0.9 ? "skulls" : "chest";
      const f = atlas.envTile(b, name) ?? atlas.envTile(b, "barrel");
      if (f) { L.decor.push(wx, wy, 0.3, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim); return; }
    }
    // gfx round 10: against a north wall, now and then a weapon rack or a broken statue; candles at a wall's foot (a small warm light)
    if (n && !sw && !e && !w && free && hp > 0.5 && hp < 0.66) {
      const r = hash2(x, y, 13), f = atlas.envTile(b, fens ? (r < 0.5 ? "lantern" : "ruin") : r < 0.5 ? "rack" : "statue");
      if (f) {
        L.decor.push(wx, wy + 1, 0.3, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, fens && r < 0.5 ? Math.max(dim, 0.9) : dim);
        if (fens && r < 0.5 && dim > 0.5) lights.push([wx + 2, wy + f.h / 2 - 3]);   // the lantern's warm light
        return;
      }
    }
    if (walls >= 1 && free && hp > 0.9 && hp < 0.935) {
      const f = atlas.hue("candles");
      if (f) { L.decorHue.push(wx, wy, 0.3, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, Math.max(dim, 0.9)); if (dim > 0.5) candles.push([wx, wy + 3]); return; }
    }
    // gfx round 21: the odd bones or skull heap on the open floor (floor detail, painted per biome)
    if (walls === 0 && free && hash2(x, y, 15) < 0.025) {
      const f = atlas.envTile(b, hash2(x, y, 16) < 0.6 ? "bones" : "skulls");
      if (f) { L.decor.push(wx, wy, 0.25, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim); return; }
    }
    // a standing brazier in the open floor of a big room (its fire lights the room)
    if (walls === 0 && free && hash2(x, y, 14) < 0.018 && !isWall(x - 2, y) && !isWall(x + 2, y) && !isWall(x, y - 2) && !isWall(x, y + 2)) {
      const f = atlas.hue("brazier");
      if (f) { L.decorHue.push(wx, wy, 0.3, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, Math.max(dim, 0.9)); if (dim > 0.5) lights.push([wx, wy + f.h / 2 - 2]); return; }
    }
    // gfx round 18: the painted floors carry their own wear — the ramp register's decals at 40 % of their old density
    const hd = hash2(x, y, 9) / 0.4, near = walls > 0 ? 0.1 : 0;
    const pick = hd < 0.07 + near ? "moss_" + (hash2(x, y, 10) < 0.5 ? 0 : 1)
      : hd < 0.12 + near ? "crack" : hd < 0.16 + near ? "rubble" : hd < 0.19 + near ? "blood" : hd < 0.205 + near ? "bones" : null;
    if (!pick) return;
    if (pick === "blood") {
      const f = atlas.hue(hash2(x, y, 12) < 0.5 ? "blood_0" : "blood_1");
      if (f) L.decorHue.push(wx, wy, 0.2, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim * 0.9);
      return;
    }
    const f = atlas.envTile(b, pick);
    if (f) L.decor.push(wx, wy, 0.2, f.w / 2, f.h / 2, f.u0, f.v0, f.u1, f.v1, dim);
  }

  function build(now: number): void {
    const p = paletteFor(st.clock < st.bossFlashUntil ? "boss_flash" : st.biome);
    blit.setPalette(p); blit.setGrade(st.clock < st.bossFlashUntil ? "boss_flash" : st.biome);
    clear.setRGB(p[0]![0], p[0]![1], p[0]![2]);
    const b = st.biome;
    const fight = mode === "fight";
    updateRoom();
    rects.length = 0; labels.length = 0; texts.length = 0; tags.length = 0; plates.fill(null); ghostAt.length = 0;
    const bright = p[p.length - 1]!;
    // Cut 8A: a hit flashes to the palette's brightest in the fight frame; the map keeps its paper white
    if (fight) L.ents.setFlash(bright[0], bright[1], bright[2]); else L.ents.setFlash(0.98, 0.95, 0.9);

    // tiles: one instance per seen tile; visible ones full, remembered ones dimmed. Cut 5 §4 props (shrine, vault,
    // nest) stand on a floor tile in the same layer: shrine flames flicker at 1 Hz, a nest shows awake once woken.
    const propFrame = Math.floor(now / 1000) & 1;
    const torchFrame = Math.floor(now / 180) & 1;
    const hd = atlas.envTile(b, "floor_0") !== undefined;   // art pass: register 3 loaded for this biome
    L.tiles.begin(); L.decor.begin(); L.decorHue.begin();
    lights.length = 0; candles.length = 0;
    stairsSeen.length = 0;
    for (let y = 0; y < st.h; y++) for (let x = 0; x < st.w; x++) {
      const i = y * st.w + x;
      if (!st.seen[i]) continue;
      const t = st.tiles[i]!;
      if (t === "stairs_down") stairsSeen.push([x, y]);
      const prop = PROPS.has(t);
      const dim = light(i);   // Cut 14 §3: the hero's room stays lit
      const s = (hd ? envTileFor(b, x, y, t, prop) : undefined) ?? atlas.tile(b, prop ? "floor" : t, ((x * 7 + y * 13) % 11) < 2);
      // second art pass: a wall's top is the unlit mass above the rooms — a step under the floor, so the lit room reads (watch.png)
      const tdim = hd && t === "wall" && !wallFace(x, y) ? dim * WALL_TOP_DIM : dim;
      L.tiles.push(x * TILE + TILE / 2, -(y + 1) * TILE, 0, TILE, TILE, s.u0, s.v0, s.u1, s.v1, tdim);
      if (prop) {
        const p = atlas.prop(b, t, t === "nest" ? (st.nestWoken.has(i) ? 1 : 0) : propFrame);
        L.tiles.push(x * TILE + TILE / 2, -(y + 1) * TILE, 0.1, TILE, TILE, p.u0, p.v0, p.u1, p.v1, dim);
      }
      if (hd) dress(b, x, y, t, dim, torchFrame);
    }
    L.tiles.end(); L.decor.end(); L.decorHue.end();
    // the torches nearest the camera light the blit (MAX_LIGHTS; a 2-frame flicker in strength)
    lights.sort((a, c) => (a[0] - camSX) ** 2 + (a[1] - camSY) ** 2 - ((c[0] - camSX) ** 2 + (c[1] - camSY) ** 2));
    blit.setLights(lights.slice(0, MAX_LIGHTS), torchFrame ? 0.94 : 1);

    // overlays: 2-frame animation at 4 fps (real time)
    const frame = Math.floor(now / 250) & 1;
    L.overlays.begin();
    for (const o of st.overlays) {
      const i = o.y * st.w + o.x;
      if (!st.visible[i]) continue;
      const s = atlas.overlay(o.k, frame);
      // gfx round 5 (raters: "the gas is flat green squares"): with the juice on, a gas tile is a smaller drifting blob inside its glowing
      // puffs (fx.ts), not a tile-sized square
      if (o.k === "gas" && fxLevel > 0) { const b = Math.sin(now / 420 + i) * 0.8; L.overlays.push(o.x * TILE + TILE / 2 + b, -(o.y + 1) * TILE + 1 + Math.abs(b), 0.5, 6, 6, s.u0, s.v0, s.u1, s.v1); continue; }
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
    L.shadows.begin(); L.ents.begin(); L.ghosts.begin(); L.glyphs.begin(); L.hud.begin(); L.text.begin();
    stats.ents = st.ents.size; stats.drawn = 0;
    // Cut 13 §4: the callout's box (clamped inside the frame, see `textX`), so a hostile's name never shares its line (rater Q's
    // `JACKALCKAL`: the callout above the hero and the name under a foe one tile up, on one row)
    const heroEnt = st.hero;
    let calloutBox: [number, number, number, number] | null = null;   // x0, y0, x1, y1 (world; y up)
    const keeps = keepBoxes();
    if (st.callout && heroEnt && !quiet) {
      const [hx, hy] = feet(heroEnt);
      // gfx round 1: the callout is an iron plate (tags.ts `Plate`) — its box in world texels from its CSS metrics; the notch under it
      const px = dpr / k, width = (st.callout.text.length * CALL_CHAR + CALL_PAD) * px, ph = CALL_H * px, notch = 7 * px;
      const above = hy + atlas.entity(heroEnt.kind).h / 2 + notch + (fight ? (heroEnt.glyph ? (fight ? TILE * 2 : TILE) + 6 : 5) : 6);
      const cx = textX(width, hx);
      const boxAt = (y: number): [number, number, number, number] => [cx - width / 2, y, cx + width / 2, y + ph];
      const clear = (y: number): boolean => !keeps.some((b) => boxHit(boxAt(y), b));
      let cy = above;
      if (!clear(cy)) {
        const below = hy - ph - 3, floor = camSY - ih / 2 + 2;
        cy = below;
        for (let i = 0; i < 24 && !clear(cy) && cy - ph > floor; i++) cy -= ph * 0.5;
        if (!clear(cy)) { cy = above; for (let i = 0; i < 24 && !clear(cy); i++) cy += ph * 0.5; }
      }
      calloutBox = boxAt(cy);
    }
    // Cut 28 §4: the caption's box (the fight frame's top line), placed under any DOM it would sit on; the plates keep off it too
    let captionBox: [number, number, number, number] | null = null;
    if (fight && st.caption && !quiet && !(st.callout && heroEnt)) {
      const px = dpr / k, w = Math.min(iw - 2, (st.caption.text.length * CALL_CHAR + CALL_PAD) * px), hgt = CALL_H * px;   // gfx round 1: a plate
      const boxAt = (y: number): [number, number, number, number] => [camSX - w / 2, y, camSX + w / 2, y + hgt];
      let y = camSY + ih / 2 - Math.ceil((FIGHT_TOP_CSS * dpr) / k) - hgt;
      for (let i = 0; i < 8; i++) { const hit = keeps.find((b) => boxHit(boxAt(y), b)); if (!hit) break; y = hit[1] - hgt - 3; }
      captionBox = boxAt(y);
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
      if (!e.hero && !st.visible[vi] && !(e.remembered && st.seen[vi])) continue;
      const g = stacks.get(vi); if (g) g.push(e.id); else stacks.set(vi, [e.id]);
    }
    const tagBoxes: [number, number, number, number][] = [];   // Cut 15 §4: the name tags drawn so far this frame (world x0, y0, x1, y1)
    // Cut 26 §2: a fork floor shows its two stairs — each seen stair carries its lane's plate (the core's `Snapshot.stairs`), the stair the
    // set's route takes lit (a gold plate and a warm light), the other dim; with no lanes on the wire both stairs still glow
    stairPlates.length = 0;
    if (stairsSeen.length >= 2) {
      const px = dpr / k;
      for (const [x, y] of stairsSeen) {
        const info = st.stairs.find((s) => s.x === x && s.y === y);
        const taken = !!info?.taken, text = info?.biome ?? "";
        stairPlates.push({ x, y, text, taken });
        if (!text) continue;
        const nw = (text.length * TAG_CHAR + TAG_PAD) * px, th = TAG_H * px, wx = x * TILE + TILE / 2, wy = -y * TILE + 1;
        tagBoxes.push([wx - nw / 2, wy, wx + nw / 2, wy + th]);
        const [cx, cy] = toCss(wx, wy);
        tags.push({ id: -1000 - y * st.w - x, text, x: cx, y: cy, w: (nw * k) / dpr, hp: -1, stair: taken ? "taken" : "other" });
      }
    }
    // Cut 22 (AH: "goblin nameplates stacked — cluttered at the key moment"): while a boss is in view his plate is the one drawn (and
    // the allies'); his horde carries the small pixel hp bar instead of a plate
    let bossInView = false;
    for (const e of st.ents.values()) if (e.boss && !e.dying && !e.remembered && st.visible[e.y * st.w + e.x]) {
      bossInView = true;
      if (st.speed > 0 && !e.ally) {
        juice.bossIn(e.id);   // juice pass 2: the entrance
        // gfx round 1 (raters: "no entrance title card"): the boss's name (his plate's word) steps in as a title plate for BOSS_TITLE_MS
        if (!bossTitled.has(e.id) && fxLevel > 0) { bossTitled.add(e.id); bossTitle = { text: e.name, until: now + BOSS_TITLE_MS }; }
      }
      break;
    }
    if (bossTitle && now > bossTitle.until) bossTitle = null;
    // gfx round 1 (raters: "six labels pile up; the hero lost inside the swarm"): the PLATE_MAX hostiles nearest the hero carry their name
    // plate; the rest of a crowd carries the small pixel hp bar (the fight frame) or nothing (the map)
    const plated = new Set<number>();
    if (heroEnt) {
      const near = [...st.ents.values()].filter((e) => !e.hero && !e.ally && !e.neutral && !e.dying && e.kind !== "bones" && !e.remembered && st.visible[e.y * st.w + e.x])
        .sort((a, c) => Math.max(Math.abs(a.x - heroEnt.x), Math.abs(a.y - heroEnt.y)) - Math.max(Math.abs(c.x - heroEnt.x), Math.abs(c.y - heroEnt.y)) || a.id - c.id);
      for (const e of near.slice(0, quiet ? 1 : PLATE_MAX)) plated.add(e.id);   // (a quiet view — a held beat, the edit's scene — names one)
    }
    // Cut 18 §2: the hero's drawn rect (world: x0, x1, y0, y1) — his feet with his stack's fan — so a sprite over him can be moved off
    let heroBox: [number, number, number, number] | null = null;
    if (heroEnt && !heroEnt.dying) {
      let [hx, hy] = feet(heroEnt);
      const g = stacks.get(heroEnt.y * st.w + heroEnt.x), n = g?.length ?? 1, i = g ? g.indexOf(heroEnt.id) : 0;
      if (n > 1 && i >= 0) hx += Math.round((i / (n - 1) - 0.5) * STACK_SPREAD);
      const hs = atlas.entity(heroEnt.kind), hw = hs.w / 2, hh = hs.h / 2;
      heroBox = [hx - hw / 2, hx + hw / 2, hy, hy + hh];
    }
    for (const e of st.ents.values()) {
      if (e.kind === "bones") continue; // drawn in the items layer above
      const vi = e.y * st.w + e.x;
      // QA 1a2a4a9 (P: jackal silhouettes on black left of a wall): a remembered foe is drawn only over a tile the map draws
      if (!e.hero && !st.visible[vi] && !(e.remembered && st.seen[vi])) continue;
      stats.drawn++;
      // gfx round 10 (raters: the fallen boss "a dark smear"): a boss's body lies in its death pose from CORPSE_T ticks after the blow, for
      // the rest of the floor; without the art he dissolves as before (then is not drawn)
      if (e.boss && e.dying) {
        const dead = st.clock - e.dying.t0 >= CORPSE_T ? atlas.corpse(e.kind) : undefined;
        if (dead) { const [cx, cy] = feet(e); L.ents.push(cx, cy, 1.9, dead.w / 2, dead.h / 2, dead.u0, dead.v0, dead.u1, dead.v1, 0.92, 0, 0, e.flip ? 1 : 0); continue; }
        if (e.fade >= 1) continue;
      }
      let [fx, fy] = feet(e);
      const fx0 = fx;
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
      // QA 1a2a4a9 (P: a monkey drawn half outside the floor over black void): a sideways nudge (the stack's fan, the step-aside) never
      // stands a sprite on a tile the map does not draw as open ground — the other side, else its own tile
      if (fx !== fx0 && !openFeet(fx, fy)) fx = openFeet(2 * fx0 - fx, fy) ? 2 * fx0 - fx : fx0;
      const s = atlas.entity(e.kind);
      const w = s.w / 2, h = s.h / 2; // world units (env texels)
      // Cut 18 §2: the hero is never covered (rater Y: "the warlord sprite hid my hero completely"; Z: "sprites overlap") — he draws
      // in front of every sprite, and a sprite whose rect would cover more than HERO_COVER of his moves sideways, away from him (the
      // side it stands on, else behind his facing), until it covers no more; a boss keeps its size, his head stays clear
      if (heroBox && !e.hero) {   // a dying sprite too (it dissolves over him otherwise)
        const [x0, x1, y0, y1] = heroBox, hw = x1 - x0, hh = y1 - y0;
        const oy = Math.max(0, Math.min(fy + h, y1) - Math.max(fy, y0)) / hh;
        const ox = Math.max(0, Math.min(fx + w / 2, x1) - Math.max(fx - w / 2, x0));
        const cover = e.boss ? BOSS_COVER : HERO_COVER;   // gfx round 4 (raters: "unstack the hero and the boss — the hit is not visible")
        if (oy > 0 && (ox / hw) * oy > cover) {
          const cx = (x0 + x1) / 2, dir = fx !== cx ? Math.sign(fx - cx) : st.hero?.flip ? 1 : -1;
          const keep = Math.floor((cover * hw) / oy);   // the most of his width it may still cover
          fx = dir > 0 ? Math.ceil(x1 - keep + w / 2) : Math.floor(x0 + keep - w / 2);
          // gfx round 7 (half-size sprites: a stack pushed off the hero landed its members on one x): they keep their fan, outward
          if (stackN > 1 && stackI >= 0) fx += dir * stackI * Math.round(Math.max(STACK_SPREAD, w / 2));
        }
        // gfx round 5 (raters: "the rat hides under the hero"): the hero draws in front, so a small foe at his feet vanished behind him — a
        // live foe the hero's rect would hide more than HIDDEN_MAX of steps sideways, away from him, until at least half of it shows
        const oyF = Math.max(0, Math.min(fy + h, y1) - Math.max(fy, y0)) / Math.max(1, h);
        const oxF = Math.max(0, Math.min(fx + w / 2, x1) - Math.max(fx - w / 2, x0)) / Math.max(1, w);
        if (!e.dying && !e.boss && stackN <= 1 && oxF * oyF > HIDDEN_MAX && oyF > 0) {   // (a stack's members fan instead)
          const cx = (x0 + x1) / 2, dir = fx !== cx ? Math.sign(fx - cx) : st.hero?.flip ? 1 : -1;
          const nx = dir > 0 ? Math.ceil(x1 - w * 0.5 + w / 2) : Math.floor(x0 + w * 0.5 - w / 2);
          if (openFeet(nx, fy) || Math.abs(nx - fx0) <= TILE) fx = nx;
        }
      }
      // entities sort by row (lower rows in front); the hero above them all (under the glyphs at 3.5)
      const z = e.hero ? HERO_Z : 2 + Math.min(1, e.py / Math.max(1, st.h)) + (stackN > 1 ? stackI * 0.001 : 0);   // a stack's members never z-fight
      { const [cx, cy] = toCss(fx - w / 2, fy + h); rects.push({ id: e.id, kind: e.kind, hero: !!e.hero, x: cx, y: cy, w: (w * k) / dpr, h: (h * k) / dpr, stack: stackN, z }); }
      if (e.remembered) {
        L.ents.push(fx, fy, z, s.w / 2, s.h / 2, s.u0, s.v0, s.u1, s.v1, REMEMBERED_DIM * fadeDim(e), 0, 0, e.flip ? 1 : 0);
        continue;
      }
      if (!e.dying) {
        // contact shadow; companions (ally + cid, or tamed this run) get a 1-texel light ring
        const ring = st.ringShown(e);
        const sh = atlas.shadow(Math.min(w - 2, 12), ring);
        L.shadows.push(fx, fy - (ring ? 2 : 1), 1.5, sh.w, sh.h, sh.u0, sh.v0, sh.u1, sh.v1, 1, 0, e.fade >= 0.75 ? 1 : 0);
      }
      // QA 1a2a4a9 (P: "on `R1 drank heal` a large cream blob covers the hero and the conjurer for the whole moment"): a hit is a tint
      // toward the palette's brightest, never a solid silhouette (the hero hit every tick read as a blob), and a stopped clock (⏸, the
      // replay's pause on its last frame) holds no flash
      // juice pass 3: a falling boss's killing blow flashes lightly — the slow-mo stretched its two-tick flash into a pale silhouette
      // gfx round 3 (raters: "the hero whited out mid-flash reads as a pale smear"): the hero's own hurt flash is lighter (HERO_FLASH)
      const flash = !st.flashing(e) || st.speed <= 0 ? 0 : e.boss && e.dying ? BOSS_FALL_FLASH : e.hero ? HERO_FLASH : Math.max(FLASH_MIX, juice.hitFlash(e.id, st.clock, e.hero));
      // juice: squash & stretch (a hit, a lunge, a spawn's pop, a death's slump) — the feet stay put; `rects` keep the true size
      const [sqx, sqy] = juice.squash(e.id, st.clock);
      // gfx round 20 (raters: "the hero only slides between tiles"): a walking step bobs one texel on each half of the move (FX > 0)
      const step = fxLevel > 0 && e.move && !e.dying && !e.boss ? ((st.clock - e.move.t0) / Math.max(0.01, e.move.dur) < 0.5 ? 1 : 0) : 0;
      const lift = juice.lift(e.id, st.clock) + step;   // gfx round 5: a boss drops into his arena
      const ghost = fxLevel > 0 && !e.hero && ETHEREAL.has(e.kind);
      if (ghost) ghostAt.push([fx, fy + h / 2]);
      (ghost ? L.ghosts : L.ents).push(fx, fy + lift + (ghost ? Math.round(Math.sin(now / 380 + e.id) * 1.5) + 1 : 0), z, Math.round(s.w * sqx) / 2, Math.round(s.h * sqy) / 2, s.u0, s.v0, s.u1, s.v1, (e.hero ? 1.12 : e.ally ? 1.1 : 1) * fadeDim(e), flash, 0, e.flip ? 1 : 0);   // (round 16: the hero a touch brighter — "muddy on the ochre floor")
      // Cut 8A: in the fight frame the hero and his allies carry an hp bar (BAR_W×1, red under the palette's brightest) 1 texel
      // above the sprite; glyphs sit above the bar. Second art pass: a hostile's bar is on its name tag instead.
      let top = fy + h + 2;
      // QA e75ec29 (R: "the pet is an unlabelled sprite with no hp"): an ally carries a tag too — green-tinted, its kind and name
      // (`jackal Skog`; a summoned ally, unnamed, its kind), with the same short hp bar
      const tagged = !e.hero && !e.neutral && !e.dying && (!bossInView || e.boss || !!e.ally) && (e.boss || !!e.ally || plated.has(e.id));
      const tagText = e.ally ? allyName(e.kind, e.name) : e.name;
      if (fight && barBg && barFg && !tagged && !e.dying && !e.neutral && e.maxHp > 0) {
        const fill = Math.max(0, Math.min(BAR_W, Math.round((BAR_W * e.hp) / e.maxHp)));
        L.hud.push(fx, fy + h + 1, 3.7, BAR_W, 1, barBg.u0, barBg.v0, barBg.u1, barBg.v1);
        if (fill > 0) L.hud.push(fx - BAR_W / 2 + fill / 2, fy + h + 1, 3.8, fill, 1, barFg.u0, barFg.v0, barFg.u1, barFg.v1);
        top = fy + h + 4;
      }
      // Second art pass (watch.png): every hostile in view carries its name tag — a small serif plate over its head with a short hp
      // bar (tags.ts, DOM) — in both frames. The boxes are laid out here in world texels: Cut 14 §3 / Cut 15 §4 a tag whose box
      // would intersect one already placed (a stack, a crowd) moves up a row; Cut 13 §4 the callout keeps its line (a tag that
      // cannot clear it is not drawn).
      if (tagged) {
        const px = dpr / k;   // world texels per CSS px
        const nw = (tagText.length * TAG_CHAR + TAG_PAD) * px, th = TAG_H * px, nx = textX(nw, fx);
        let ny = fy + h + 1;
        const hits = (y: number): boolean => tagBoxes.some((b) => nx - nw / 2 < b[2] && nx + nw / 2 > b[0] && y < b[3] && y + th > b[1]);
        const clash = (y: number): boolean => [calloutBox, captionBox, ...keeps].some((b) => !!b && nx + nw / 2 > b[0] && nx - nw / 2 < b[2] && y + th > b[1] && y < b[3]);
        const ny0 = ny;
        let stacked = 0;   // gfx round 6: rows climbed past other plates (not past the callout or the DOM)
        for (let i = 0; i < 8 && (hits(ny) || clash(ny)); i++) { if (hits(ny)) stacked++; ny += th + px; }
        // Cut 28 §4: a plate pushed up into the HUD's chips tries under its foe instead
        if (hits(ny) || clash(ny)) { ny = ny0 - th - px - h - 1; for (let i = 0; i < 8 && (hits(ny) || clash(ny)); i++) ny -= th + px; }
        // gfx round 6 (raters: "the labels pile onto the hero"): a plate that would climb over more than two other plates is left off — a
        // crowd keeps two rows of names, not a tower (a boss's and an ally's always draw)
        if (!hits(ny) && !clash(ny) && (stacked <= 2 || e.boss || !!e.ally)) {
          tagBoxes.push([nx - nw / 2, ny, nx + nw / 2, ny + th]);
          const [cx, cy] = toCss(nx, ny);
          labels.push({ text: tagText, x: cx, y: cy, id: e.id, w: (nw * k) / dpr, h: TAG_H });
          tags.push({ id: e.id, text: tagText, x: cx, y: cy, w: (nw * k) / dpr, hp: e.maxHp > 0 ? e.hp / e.maxHp : -1, ally: e.ally, boss: !!e.boss });
          top = ny + th + 1;
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
    L.shadows.end(); L.ents.end(); L.ghosts.end(); L.glyphs.end(); L.hud.end();
    // juice: particles, numbers and this frame's events (fx.ts); nothing at `low`
    fires.length = 0; gases.length = 0;
    if (fxLevel > 0) {
      for (const o of st.overlays) if (st.visible[o.y * st.w + o.x]) (o.k === "fire" ? fires : gases).push([o.x * TILE + TILE / 2, -(o.y + 1) * TILE + TILE / 2]);
      // gfx round 5 (raters: "the water is static, no reflections"): the visible pools, for the juice's glints and mist
      waters.length = 0;
      const hh0 = st.hero;
      if (hh0) for (let y = Math.max(0, hh0.y - 9); y <= Math.min(st.h - 1, hh0.y + 9) && waters.length < 48; y++) for (let x = Math.max(0, hh0.x - 7); x <= Math.min(st.w - 1, hh0.x + 7) && waters.length < 48; x++) {
        const i = y * st.w + x; if (st.tiles[i] === "water" && (st.visible[i] || roomLit[i])) waters.push([x * TILE + TILE / 2, -(y + 1) * TILE + TILE / 2]);
      }
      const hh = st.hero, dustC = p[Math.min(3, p.length - 1)]!;
      juice.update(simDt, now, st.clock, {
        ent: (id) => { const e = st.ents.get(id); if (!e) return null; const [x, y] = feet(e); return { x, y, h: atlas.entity(e.kind).h / 2, w: atlas.entity(e.kind).w / 2, kind: e.kind, hero: e.hero, boss: e.boss, maxHp: e.maxHp, ally: e.ally }; },
        heroId: st.heroId, speed: st.speed, fight, quiet, cam: [camSX, camSY], half: [iw / 2, ih / 2], fires, gases, waters, torches: lights, ghosts: ghostAt, fog: st.biome === "fens" ? 1 : 0,
        motes: !!hh && !!roomLit[hh.y * st.w + hh.x], dust: [dustC[0], dustC[1], dustC[2]],
      });
      numCss.length = 0;
      // gfx round 11 (raters Y, Z: "the red '1' sits on top of 'DRAINED'"): a number that would land on the callout plate or a name plate
      // steps beside it (world texels; a number is ~2.5 CSS glyphs wide)
      const npx = dpr / k, nw = 22 * npx, nh = 20 * npx;
      const boxes = [calloutBox, ...tagBoxes].filter((b): b is [number, number, number, number] => !!b);
      for (const n of juice.shown) {
        let x = n.x;
        for (const b of boxes) if (x + nw / 2 > b[0] && x - nw / 2 < b[2] && n.y + nh > b[1] && n.y < b[3]) x = x < (b[0] + b[2]) / 2 ? b[0] - nw / 2 - 2 * npx : b[2] + nw / 2 + 2 * npx;
        const [cx, cy] = toCss(x, n.y); numCss.push({ ...n, x: cx, y: cy });
      }
      for (const id of juice.breaks.splice(0)) { const e = st.ents.get(id); if (e) { const [x, y] = feet(e); const [cx, cy] = toCss(x, y + atlas.entity(e.kind).h / 2 * 0.95); tagLayer.shatter(cx, cy); const [sx, sy] = toCss(x, y + atlas.entity(e.kind).h / 2 + 10); tagLayer.stamp(/* copy:callout */ "BROKEN", sx, sy - 96, "broken"); } }
      for (const id of juice.falls.splice(0)) { const e = st.ents.get(id); if (e) { const [x, y] = feet(e); const [sx, sy] = toCss(x, y + atlas.entity(e.kind).h / 2 + 8); tagLayer.stamp(/* copy:callout */ "SLAIN", sx, sy - 24, "slain"); } }
    } else { juice.falls.length = 0; juice.update(0, now, st.clock, { ent: () => null, heroId: -1, speed: 0, fight, quiet, cam: [0, 0], half: [0, 0], fires, gases, waters, torches: lights, motes: false, dust: [0, 0, 0] }); }

    // callout: bitmap text above the hero (env density)
    const hero = st.hero;
    if (st.callout && hero && !quiet) {
      // in the fight frame the callout clears the hp bar and, when up, the 2× glyph above it
      // Cut 13 §4: centred on the hero but kept inside the frame — a hero at the edge used to lose its callout's right half
      // gfx round 1: drawn as the DOM plate over its box (tags.ts)
      if (calloutBox) { const [x0, y0] = toCss(calloutBox[0], calloutBox[1]), [x1] = toCss(calloutBox[2], calloutBox[1]); plates[1] = ({ kind: "callout", text: st.callout.text, x: (x0 + x1) / 2, y: y0, w: x1 - x0 }); }
      texts.push({ kind: "callout", text: st.callout.text, ...cssBox(calloutBox) });
    }
    // Cut 8A: the firing row as a caption at the top of the fight frame (`R2 attack goblin`), under the DOM hud
    // Cut 18 §2: one line over the fight — a telegraph (the core's callout over the hero) takes it; the row goes to the ticker (watch)
    if (bossTitle && !quiet) plates[0] = ({ kind: "boss", text: bossTitle.text, x: (iw * k) / dpr / 2, y: (ih * k) / dpr * 0.2, w: bossTitle.text.length * 15 + 56 });
    if (fight && st.caption && !quiet && !(st.callout && hero)) {
      if (captionBox && !bossTitle) { const [x0, y0] = toCss(captionBox[0], captionBox[1]), [x1] = toCss(captionBox[2], captionBox[1]); plates[2] = ({ kind: "caption", text: st.caption.text, x: (x0 + x1) / 2, y: y0, w: x1 - x0 }); }
      texts.push({ kind: "caption", text: st.caption.text, ...cssBox(captionBox) });
    }
    L.text.end();
  }

  /** Cut 13 §4: the centre x for a text `width` wide that wants `cx`, kept inside the frame's iw texels (1-texel margin);
   *  a text wider than the frame stays centred on the frame. */
  function textX(width: number, cx: number): number {
    const half = width / 2, lo = camSX - iw / 2 + 1 + half, hi = camSX + iw / 2 - 1 - half;
    return lo > hi ? camSX : Math.max(lo, Math.min(hi, cx));
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
    quality.sample(dt, stats.cpuMs, now); applyFx();
    simDt = dt;
    st.tick(dt * juice.timeScale(now), now);   // juice: a hit-stop holds the clock, a death's slow-mo slows it (med+, motion on)
    const snapCam = st.cameraSnap || (resized && !st.hero);
    st.cameraSnap = false;
    updateCamera(dt / 1000, snapCam);
    if (!st.loaded) return;
    if (!atlasReady && now - born < ATLAS_WAIT_MS) return;
    if (glLost) { view2d?.draw(st); stats.tick = st.tickNow(); stats.pending = st.pending(); stats.calls = 0; return; }
    const t0 = performance.now();
    // snap camera to the env-texel grid; remainder → blit UV offset quantised to device px. Cut 8A: the fight frame's
    // screen shake displaces the snapped centre by whole texels
    const [shx0, shy0] = mode === "fight" ? st.shakeOffset() : [0, 0];
    const [kx, ky] = juice.shake(now);
    const shx = shx0 + kx, shy = shy0 + ky;
    const sx = Math.round(cam.x) + shx, sy = Math.round(cam.y) + shy;
    camSX = sx; camSY = sy;
    build(now);
    tagLayer.sync(cutFrames > 0 ? [] : tags); tagLayer.plates(cutFrames > 0 ? [] : plates); tagLayer.numbers(fxLevel > 0 ? numCss : []);
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
    if (fxLevel > 0) {   // juice: the light field for this frame (walls → shadows; torches, the hero, fire, bolts, pops)
      let wh = st.depth * 7919 + st.w * 131 + st.h;
      for (let i = 0; i < st.tiles.length; i++) if (st.tiles[i] === "wall") wh = (Math.imul(wh, 31) + i) | 0;
      field.setMask(st.w, st.h, (i) => st.tiles[i] === "wall", `${st.w}x${st.h}:${wh}`);
      fieldLights.length = 0;
      // 307dbed control rater AR ("the Fens looked like the Burrows"): the pools were one warm amber everywhere, and an amber pool on the
      // Fens' teal reads as the Burrows' clay — the hero's and the torches' light take the biome's own cast (the flames stay flames)
      const tint = LIGHT_TINT[st.biome] ?? LIGHT_TINT.default!;
      // gfx round 16 (raters, the Fens: "light the hero with a warm radius"): the hero's own light stays warm in every biome (torches keep the cast)
      // (round 22, the Fens at 5.5: "warm the hero's pool further" — there it is wider and a lantern's amber against the cold water)
      const fensHero = st.biome === "fens";
      if (hero) { const [hx, hy] = feet(hero); fieldLights.push({ x: hx / TILE, y: -(hy + TILE / 2) / TILE, r: Math.min(fensHero ? 8 : 7, st.vision + (fensHero ? 2.5 : 1.5)), c: fensHero ? [1.12, 0.8, 0.46] : [0.95, 0.74, 0.48] }); }   // gfx round 1: a wider, warmer pool ("dim flat lighting", "brown mush")
      juice.lights(now, fieldLights);
      for (const [px, py] of st.projectilePositions()) fieldLights.push({ x: px + 0.5, y: py + 0.5, r: 2, c: [0.7, 0.6, 0.4] });
      for (const [cx, cy] of candles.slice(0, 3)) if (fieldLights.length < MAX_FIELD - 6) fieldLights.push({ x: cx / TILE, y: -cy / TILE, r: 2.2, c: [0.7, 0.45, 0.2] });
      for (const [gx, gy] of ghostAt.slice(0, 4)) if (fieldLights.length < MAX_FIELD - 6) fieldLights.push({ x: gx / TILE, y: -gy / TILE, r: 3.2, c: [0.3, 0.5, 0.9] });   // gfx round 10: a ghost's own cold glow (round 22: brighter, wider)
      const fl = (i: number, a: number, b: number): number => quality.at("high") ? 0.88 + 0.08 * Math.sin(now * a + i * 1.7) + 0.05 * Math.sin(now * b + i * 4.1) : 1;
      fires.forEach(([x, y], i) => { if (fieldLights.length < MAX_FIELD - 4) { const k = fl(i, 0.017, 0.041) * 0.9; fieldLights.push({ x: x / TILE, y: -y / TILE, r: 3, c: [1 * k, 0.5 * k, 0.15 * k] }); } });
      gases.slice(0, 4).forEach(([x, y]) => fieldLights.push({ x: x / TILE, y: -y / TILE, r: 1.8, c: [0.12, 0.2, 0.03] }));
      for (const sp of stairPlates) if (fieldLights.length < MAX_FIELD - 2) fieldLights.push({ x: sp.x + 0.5, y: sp.y + 0.5, r: sp.taken ? 2.6 : 1.6, c: sp.taken ? [0.95, 0.75, 0.35] : [0.35, 0.3, 0.22] });
      lights.forEach(([x, y], i) => { if (fieldLights.length < MAX_FIELD) { const k = fl(i, 0.011, 0.029); fieldLights.push({ x: x / TILE, y: -y / TILE, r: 4.8, c: [1 * k * tint[0], 0.72 * k * tint[1], 0.4 * k * tint[2]] }); } });
      field.setLights(fieldLights);
      field.render(renderer);
    }
    renderer.setRenderTarget(rt);
    renderer.setClearColor(clear, 0);   // art pass: a=0 = the void (no quad drew here): never lit, never dithered
    renderer.clear(true, true, false);
    renderer.render(scene, camera);
    if (fxLevel > 1) { normals.sync(atlas.sprite); normals.render(renderer, camera); }   // juice pass 2: the sprites' normals (high)
    if (fxLevel > 0) {   // juice: bloom off the emissive tag; the blit's juice uniforms
      bloom.setSize(W * 2, H * 2);
      bloom.render(renderer);
      const vg = juice.vignette(now, hero && hero.maxHp > 0 && !hero.dying ? hero.hp / hero.maxHp : 1);
      (u.uMap!.value as THREE.Vector2).set(st.w, st.h);
      (u.uTexel!.value as THREE.Vector2).set(1 / (W * 2), 1 / (H * 2));
      (u.uVig!.value as THREE.Vector4).set(vg.c[0], vg.c[1], vg.c[2], vg.a);
      u.uVigBase!.value = vg.base; u.uDesat!.value = vg.desat; u.uBloomK!.value = 1.35; u.uAmbK!.value = 0.95; u.uTime!.value = now / 1000;
      const heat = u.uHeat!.value as THREE.Vector2[];
      const hn = quality.at("high") && quality.motion ? Math.min(8, fires.length) : 0;
      for (let i = 0; i < hn; i++) heat[i]!.set(fires[i]![0], fires[i]![1]);
      u.uHeatN!.value = hn;
    }

    renderer.setRenderTarget(null);
    renderer.setViewport(0, 0, devW, devH);
    renderer.setScissorTest(false);
    renderer.setClearColor(clear, 1);
    renderer.clear(true, false, false);
    renderer.setViewport(0, devH - ih * k, iw * k, ih * k);
    renderer.render(blit.scene, blit.camera);
    gpu.end();
    // juice (docs/JUICE.md): `window.__riddleSyncTiming` drains the GPU each frame (a 1-px readback) so cpuMs is the whole
    // pipeline's cost where the timer query never resolves (this harness); measurement only, never on in play
    if ((window as { __riddleSyncTiming?: boolean }).__riddleSyncTiming) { const g = renderer.getContext(); g.readPixels(0, 0, 1, 1, g.RGBA, g.UNSIGNED_BYTE, syncPx); }
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
      canvas.removeEventListener("webglcontextlost", onLost, false);
      canvas.removeEventListener("webglcontextrestored", onRestored, false);
      view2d?.dispose(); view2d = null;
      cancelAnimationFrame(raf);
      for (const l of Object.values(L)) l.dispose();
      blit.dispose();
      field.dispose(); bloom.dispose(); juice.dispose(); normals.dispose();
      tagLayer.dispose();
      gpu.dispose();
      rt.dispose();
      env.dispose(); spr.dispose();
      renderer.dispose();
      renderer.forceContextLoss();   // free the context now (Chrome loses the oldest past ~16 live ones: the scene and its stills make many)
    },
    preload(snap) { st.preload(snap); },
    atlasInfo() { const out: Record<string, unknown> = {}; for (const k of ["hero_fighter", "monkey", "goblin", "jackal"]) { const e = atlas.entity(k); out[k] = { w: e.w, h: e.h, u0: +e.u0.toFixed(3), v0: +e.v0.toFixed(3), u1: +e.u1.toFixed(3), v1: +e.v1.toFixed(3), fallback: (e as { fallback?: boolean }).fallback ?? "?" }; } return out; },
    debugRects() { return rects.map((r) => ({ ...r })); },
    debugBiome() { return st.biome; },
    debugLabels() { return labels.map((l) => ({ ...l })); },
    debugStairs() { return stairPlates.map((p) => ({ ...p })); },
    debugText() { return texts.map((t) => ({ ...t })); },
    setQuiet(on) { quiet = on; },
    setKeepOut(r) { keepOut = r; },
    debugPos() { return [...st.ents.values()].filter((e) => !e.dying).map((e) => ({ kind: e.kind, hero: !!e.hero, ally: !!e.ally, x: e.x, y: e.y, px: +e.px.toFixed(2), py: +e.py.toFixed(2), flip: !!e.flip })); },
    stats() { return { ...stats }; },
    /** dev: every entity the state holds and whether the draw loop would show it */
    debugEnts() { return [...st.ents.values()].map((e) => ({ id: e.id, kind: e.kind, x: e.x, y: e.y, hero: !!e.hero, rem: !!e.remembered, dying: !!e.dying, vis: !!st.visible[e.y * st.w + e.x], seen: !!st.seen[e.y * st.w + e.x], flash: st.flashing(e), fade: e.fade, px: e.px, py: e.py })); },
  };
}
