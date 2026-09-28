// Juice (docs/JUICE.md) — the light field: a 2D light map over the whole floor, S texels per tile, recomputed every frame on the
// GPU from the floor's wall mask and up to MAX_FIELD lights (torches, the hero's own light, fire, bolts in flight, the short "pop"
// lights of a hit, a spell or a death). Each light marches FIELD_STEPS samples through the wall mask (linear-filtered: a soft
// penumbra about a tile wide) toward the texel, so walls cast shadows and a doorway spills light into the corridor; the alpha
// channel is the ambient occlusion at the foot of every wall. The blit reads it in place of the old unshadowed torch pools.
// Cost: (w·S)·(h·S) texels × lights × steps, e.g. 128² × 24 × 10 ≈ 4 M mask fetches — a fraction of a millisecond on any GPU;
// the mask uploads only when the floor's tiles change.
import * as THREE from "three";

export const FIELD_S = 4;          // light-map texels per tile
export const MAX_FIELD = 24;       // lights per frame (nearest the camera first)
const FIELD_STEPS = 10;

/** one light: tile coordinates (x right, y DOWN the map: row space), radius in tiles, linear rgb (intensity folded in) */
export type FieldLight = { x: number; y: number; r: number; c: [number, number, number] };

const VERT = /* glsl */ `
varying vec2 vUv;
void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }`;

const FRAG = /* glsl */ `
uniform sampler2D uMask;     // R = wall (1) / open (0); linear filtered, one texel per tile
uniform vec2 uMap;           // map size in tiles
uniform vec4 uL[${MAX_FIELD}];   // x, y (tiles, row space), radius (tiles), unused
uniform vec3 uC[${MAX_FIELD}];   // colour × intensity
uniform int uN;
uniform float uAO;           // wall-foot occlusion strength
varying vec2 vUv;
float wall(vec2 p) { return texture2D(uMask, p / uMap).r; }
void main() {
  vec2 p = vec2(vUv.x, 1.0 - vUv.y) * uMap;   // tile coords, row space (the target's v runs up)
  float self = wall(p);
  vec3 acc = vec3(0.0);
  for (int i = 0; i < ${MAX_FIELD}; i++) {
    if (i >= uN) break;
    vec4 L = uL[i];
    vec2 d = L.xy - p;
    float dist = length(d * vec2(1.0, 1.15));
    float f = max(0.0, 1.0 - dist / L.z);
    if (f <= 0.0) continue;
    // march the segment's middle (skip ~0.55 tile at each end: a torch sits inside its wall, a wall face lights from its own edge)
    float len = length(d);
    float vis = 1.0;
    if (len > 1.1) {
      float a = 0.55 / len, b = 1.0 - 0.55 / len;
      for (int s = 0; s < ${FIELD_STEPS}; s++) {
        float t = a + (b - a) * (float(s) + 0.5) / float(${FIELD_STEPS});
        vis *= 1.0 - 0.85 * wall(p + d * t);
      }
    }
    acc += uC[i] * f * f * vis;
  }
  // wall-foot AO: the wall density a little around an open texel (walls themselves are the unlit mass: no AO on them)
  float dens = wall(p + vec2(0.55, 0.0)) + wall(p - vec2(0.55, 0.0)) + wall(p + vec2(0.0, 0.55)) + 2.2 * wall(p - vec2(0.0, 0.6));   // (round 21: the wall above casts the deepest shadow onto the floor at its foot)
  float ao = 1.0 - uAO * (1.0 - self) * smoothstep(0.0, 2.2, dens);
  gl_FragColor = vec4(min(acc * 0.5, vec3(1.0)), ao);   // stored halved: sums up to 2 survive the 8-bit target
}`;

export class LightField {
  readonly target: THREE.WebGLRenderTarget;
  private mask: THREE.DataTexture;
  private maskData = new Uint8Array(4);
  private mw = 1; private mh = 1;
  private scene = new THREE.Scene();
  private camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  private mat: THREE.ShaderMaterial;
  private maskKey = "";

  constructor() {
    this.mask = new THREE.DataTexture(this.maskData, 1, 1, THREE.RGBAFormat);
    this.mask.magFilter = THREE.LinearFilter; this.mask.minFilter = THREE.LinearFilter;
    this.mask.wrapS = this.mask.wrapT = THREE.ClampToEdgeWrapping;
    this.mask.generateMipmaps = false; this.mask.flipY = false; this.mask.needsUpdate = true;
    this.target = new THREE.WebGLRenderTarget(4, 4, {
      minFilter: THREE.LinearFilter, magFilter: THREE.LinearFilter, generateMipmaps: false, depthBuffer: false, stencilBuffer: false,
      colorSpace: THREE.NoColorSpace,
    });
    this.mat = new THREE.ShaderMaterial({
      vertexShader: VERT, fragmentShader: FRAG, depthTest: false, depthWrite: false,
      uniforms: {
        uMask: { value: this.mask }, uMap: { value: new THREE.Vector2(1, 1) },
        uL: { value: Array.from({ length: MAX_FIELD }, () => new THREE.Vector4()) },
        uC: { value: Array.from({ length: MAX_FIELD }, () => new THREE.Vector3()) },
        uN: { value: 0 }, uAO: { value: 0.5 },   // gfx round 21 (raters: "cast wall shadows so the room reads as built"): 0.32 → 0.5
      },
    });
    const q = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.mat);
    q.frustumCulled = false;
    this.scene.add(q);
  }

  /** cohort 24: after a GL context restore, the next setMask re-uploads (three re-creates its textures; this drops our key) */
  invalidate(): void { this.maskKey = ""; this.mask.needsUpdate = true; }
  /** the wall mask (re-uploaded only when `key` — the floor and its tile revision — changes) */
  setMask(w: number, h: number, isWall: (i: number) => boolean, key: string): void {
    if (key === this.maskKey) return;
    this.maskKey = key;
    if (w !== this.mw || h !== this.mh || this.maskData.length !== w * h * 4) {
      this.mw = w; this.mh = h;
      this.maskData = new Uint8Array(w * h * 4);
      this.mask.dispose();
      this.mask = new THREE.DataTexture(this.maskData, w, h, THREE.RGBAFormat);
      this.mask.magFilter = THREE.LinearFilter; this.mask.minFilter = THREE.LinearFilter;
      this.mask.wrapS = this.mask.wrapT = THREE.ClampToEdgeWrapping;
      this.mask.generateMipmaps = false; this.mask.flipY = false;
      this.mat.uniforms.uMask!.value = this.mask;
      this.target.setSize(w * FIELD_S, h * FIELD_S);
      (this.mat.uniforms.uMap!.value as THREE.Vector2).set(w, h);
    }
    for (let i = 0; i < w * h; i++) { const v = isWall(i) ? 255 : 0; this.maskData[i * 4] = v; this.maskData[i * 4 + 3] = 255; }
    this.mask.needsUpdate = true;
  }

  setLights(ls: readonly FieldLight[]): void {
    const u = this.mat.uniforms, L = u.uL!.value as THREE.Vector4[], C = u.uC!.value as THREE.Vector3[];
    const n = Math.min(MAX_FIELD, ls.length);
    for (let i = 0; i < n; i++) { const l = ls[i]!; L[i]!.set(l.x, l.y, l.r, 0); C[i]!.set(l.c[0], l.c[1], l.c[2]); }
    u.uN!.value = n;
  }

  render(renderer: THREE.WebGLRenderer): void {
    renderer.setRenderTarget(this.target);
    renderer.render(this.scene, this.camera);
  }

  dispose(): void { this.target.dispose(); this.mask.dispose(); this.mat.dispose(); }
}
