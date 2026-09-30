// Juice pass 2 (docs/JUICE.md §5, the deferred "normal maps for heroes and bosses"): derived sprite normal maps feeding the light pass.
// Codex normal maps would need an art round trip per sprite and per look; a painted sprite's own shape carries most of it: every sprite
// texel's height is a dome over its distance to the silhouette's edge (a bevel R texels wide) plus a little of its luma (the painted
// shading's relief), and the normal is that height's gradient. Derived once per sprite-sheet version (the sheet grows lazily: a new
// procedural fallback, the atlas's load) on the CPU — a chamfer distance pass and one gradient over 512² — then drawn by a twin of the
// entity layer (same instances, same squash, flip and dissolve) into a target the blit reads at `high` (FX 2): N·L toward the light
// field's gradient shades each sprite as a lit volume, the rim light stays for the silhouette.
import * as THREE from "three";
import { BAYER_GLSL, QUAD_VERT } from "./layers";

const R = 4;          // the bevel: texels from the edge to the dome's top
const DETAIL = 0.35;  // the painted luma's share of the height
const SETTLE_MS = 400;

const FRAG = /* glsl */ `
uniform sampler2D map;
uniform sampler2D nmap;
varying vec2 vUv;
varying vec4 vParam;
${BAYER_GLSL}
void main() {
  if (texture2D(map, vUv).a < 0.5) discard;
  if (vParam.z > 0.0 && vParam.z > bayer4(gl_FragCoord.xy)) discard;
  vec4 n = texture2D(nmap, vUv);
  if (vParam.w > 0.5) n.x = 1.0 - n.x;   // a flipped sprite faces the other way
  gl_FragColor = vec4(n.rgb, n.a);   // a = 0 where the normals are not derived yet: the blit leaves that texel as painted
}`;

/** the normal map of every sprite in `src` (rgb = n * 0.5 + 0.5 in world orientation, y up; a = coverage) */
export function deriveNormals(src: HTMLCanvasElement, out: HTMLCanvasElement): void {
  const w = src.width, h = src.height;
  const sd = src.getContext("2d", { willReadFrequently: true })!.getImageData(0, 0, w, h).data;
  const INF = 1e6, d = new Float32Array(w * h);
  for (let i = 0; i < w * h; i++) d[i] = sd[i * 4 + 3]! >= 128 ? INF : 0;
  // chamfer (3-4) distance to the nearest uncovered texel, two passes
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const i = y * w + x; if (!d[i]) continue;
    let v = d[i]!;
    if (x > 0) v = Math.min(v, d[i - 1]! + 3); else v = Math.min(v, 3);
    if (y > 0) { v = Math.min(v, d[i - w]! + 3); if (x > 0) v = Math.min(v, d[i - w - 1]! + 4); if (x < w - 1) v = Math.min(v, d[i - w + 1]! + 4); } else v = Math.min(v, 3);
    d[i] = v;
  }
  for (let y = h - 1; y >= 0; y--) for (let x = w - 1; x >= 0; x--) {
    const i = y * w + x; if (!d[i]) continue;
    let v = d[i]!;
    if (x < w - 1) v = Math.min(v, d[i + 1]! + 3); else v = Math.min(v, 3);
    if (y < h - 1) { v = Math.min(v, d[i + w]! + 3); if (x < w - 1) v = Math.min(v, d[i + w + 1]! + 4); if (x > 0) v = Math.min(v, d[i + w - 1]! + 4); } else v = Math.min(v, 3);
    d[i] = v;
  }
  const H = new Float32Array(w * h);
  for (let i = 0; i < w * h; i++) {
    if (!d[i]) continue;
    const t = Math.min(1, d[i]! / 3 / R), dome = Math.sqrt(1 - (1 - t) * (1 - t));
    const l = (sd[i * 4]! * 0.2126 + sd[i * 4 + 1]! * 0.7152 + sd[i * 4 + 2]! * 0.0722) / 255;
    H[i] = dome * (1 - DETAIL) + l * DETAIL;
  }
  const ctx = out.getContext("2d")!, img = ctx.createImageData(w, h), o = img.data;
  const at = (x: number, y: number): number => (x < 0 || y < 0 || x >= w || y >= h ? 0 : H[y * w + x]!);
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const i = y * w + x; if (!d[i]) continue;
    const gx = (at(x + 1, y) - at(x - 1, y)) * 0.5, gy = (at(x, y + 1) - at(x, y - 1)) * 0.5;   // canvas y is down
    let nx = -gx * 2.2, ny = gy * 2.2, nz = 1;                                                // world y is up
    const k = 1 / Math.hypot(nx, ny, nz); nx *= k; ny *= k; nz *= k;
    o[i * 4] = Math.round((nx * 0.5 + 0.5) * 255); o[i * 4 + 1] = Math.round((ny * 0.5 + 0.5) * 255); o[i * 4 + 2] = Math.round((nz * 0.5 + 0.5) * 255); o[i * 4 + 3] = 255;
  }
  ctx.putImageData(img, 0, 0);
}

/** the sprites' normal pass: a twin of the entity layer drawn into its own target (the size of the main one) */
export class SpriteNormals {
  readonly target = new THREE.WebGLRenderTarget(4, 4, { minFilter: THREE.NearestFilter, magFilter: THREE.NearestFilter, generateMipmaps: false, depthBuffer: true, colorSpace: THREE.NoColorSpace });
  readonly scene = new THREE.Scene();
  readonly material: THREE.ShaderMaterial;
  private canvas = document.createElement("canvas");
  private tex: THREE.CanvasTexture;
  private version = -1;
  private seen = -1;
  private since = 0;
  private clear = new THREE.Color(0.5, 0.5, 1);

  constructor(spriteTex: THREE.Texture) {
    this.tex = new THREE.CanvasTexture(this.canvas);
    this.tex.minFilter = THREE.NearestFilter; this.tex.magFilter = THREE.NearestFilter; this.tex.generateMipmaps = false;
    this.tex.colorSpace = THREE.NoColorSpace; this.tex.flipY = true; this.tex.premultiplyAlpha = false;
    this.material = new THREE.ShaderMaterial({ vertexShader: QUAD_VERT, fragmentShader: FRAG, uniforms: { map: { value: spriteTex }, nmap: { value: this.tex } },
      transparent: false, depthTest: true, depthWrite: true, side: THREE.DoubleSide });
  }
  add(mesh: THREE.Mesh): void { this.scene.add(mesh); }
  /** re-derive when the sprite sheet changed (cheap otherwise) */
  sync(sheet: { canvas: HTMLCanvasElement; version: number }, now = performance.now()): void {
    if (sheet.version === this.version) return;
    // a derivation is ~20 ms: it waits for the sheet to settle (the atlas's load allocates hundreds of slots in a burst; a new kind's
    // fallback one) — until then a new sprite simply draws unshaded (its normal texels are empty)
    if (sheet.version !== this.seen) { this.seen = sheet.version; this.since = now; return; }
    if (this.version >= 0 && now - this.since < SETTLE_MS) return;
    this.version = sheet.version;
    if (this.canvas.width !== sheet.canvas.width || this.canvas.height !== sheet.canvas.height) { this.canvas.width = sheet.canvas.width; this.canvas.height = sheet.canvas.height; }
    deriveNormals(sheet.canvas, this.canvas);
    this.tex.needsUpdate = true;
  }
  /** cohort 24: a GL context restore re-uploads the derived normals */
  invalidate(): void { this.tex.needsUpdate = true; }
  setSize(w: number, h: number): void { this.target.setSize(w, h); }
  render(renderer: THREE.WebGLRenderer, camera: THREE.Camera): void {
    renderer.setRenderTarget(this.target);
    renderer.setClearColor(this.clear, 0);
    renderer.clear(true, true, false);
    renderer.render(this.scene, camera);
  }
  dispose(): void { this.target.dispose(); this.material.dispose(); this.tex.dispose(); }
}
