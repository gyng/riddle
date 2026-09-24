// Fullscreen blit: nearest-samples the low-res target with the sub-texel camera remainder as a UV
// offset, applies 2-band distance fog from the hero, then per-biome palette quantisation.
// Layer tag = target alpha: 1.0 (environment) → nearest palette colour after a 4×4 Bayer offset
// indexed by WORLD env texel (so the pattern sticks to the dungeon, not the screen);
// 0.5 (sprites) → 50% blend toward the un-dithered nearest palette colour ("tint", not quantise).
// Art pass (art/ui/ART_GAP.md §5): up to 12 torch lights (world env texels). Each adds a quadratic falloff over LIGHT_R; the
// sum lifts the value BEFORE the quantise (so a torch pool steps up the ramp through the Bayer dither, in the biome's own
// colours) and adds a small warm tint after it (the one departure from the ramp: torchlight reads amber in every biome).
import * as THREE from "three";
import { BAYER_GLSL } from "./layers";
import type { Palette } from "./palette";

const VERT = /* glsl */ `
varying vec2 vUv;
void main() { vUv = uv; gl_Position = vec4(position.xy, 0.0, 1.0); }`;

const FRAG = /* glsl */ `
uniform sampler2D tex;
uniform vec2 uUvScale;
uniform vec2 uUvOff;
uniform vec2 uCam;        // snapped camera centre, world env texels
uniform vec2 uTargetEnv;  // target size in env texels (incl. overscan)
uniform vec2 uHero;       // hero position, world env texels
uniform vec3 uPal[8];
uniform int uPalN;
uniform float uDither;    // dither amplitude (fraction of 0..1 range)
uniform float uTint;      // sprite palette blend
uniform float uFade;      // global fade to palette[0]
uniform vec2 uFog;        // band radii in tiles
uniform vec2 uLights[12]; // torch flames, world env texels
uniform int uLightN;
uniform float uLightK;    // flicker (strength scale)
uniform vec3 uWarm;       // the warm tint a full light adds after the quantise
varying vec2 vUv;
${BAYER_GLSL}
vec3 nearestPal(vec3 c) {
  vec3 best = uPal[0];
  float bd = 1e9;
  for (int i = 0; i < 8; i++) {
    if (i >= uPalN) break;
    vec3 d = c - uPal[i];
    float dd = dot(d, d);
    if (dd < bd) { bd = dd; best = uPal[i]; }
  }
  return best;
}
void main() {
  vec2 tuv = vUv * uUvScale + uUvOff;
  vec4 s = texture2D(tex, tuv);
  vec2 world = uCam + (tuv - 0.5) * uTargetEnv;
  float d = length(world - uHero) / 8.0;
  float fog = d > uFog.y ? 0.6 : (d > uFog.x ? 0.8 : 1.0);
  float lit = 0.0;
  for (int i = 0; i < 12; i++) {
    if (i >= uLightN) break;
    vec2 dl = (world - uLights[i]) * vec2(1.0, 1.25);
    float f = max(0.0, 1.0 - length(dl) / 30.0);
    lit += f * f;
  }
  float env = step(0.75, s.a);
  float world_ = step(0.95, s.a);   // tiles and dressing (a=1); env chrome (items, glyphs, text) is a=0.875
  // the hero carries a little light of his own (the room around him reads, as in the target's frame)
  float hl = max(0.0, 1.0 - length((world - uHero) * vec2(1.0, 1.2)) / 40.0);
  lit += 0.4 * hl * hl;
  lit = min(lit, 1.4) * uLightK * mix(1.0, world_, env) * step(0.25, s.a);   // the void (a=0) stays dark
  // ambient 0.78: away from a torch the env sits a ramp step down; a pool lifts it back up and past
  // the lift shrinks on pale stone (the Sanctum's ramp would burn to white): 0.9 on dark rock, ~0.45 on marble
  vec3 c = s.rgb * fog * (mix(1.0, 0.78, world_) + 0.9 * lit * (1.0 - 0.6 * dot(s.rgb, vec3(0.2126, 0.7152, 0.0722))));
  // world tiles are authored in the ramp: dither only where a torch pool grades them (clean steps elsewhere, soft pool edges)
  float dth = (bayer4(floor(world)) - 0.5) * uDither * env * mix(1.0, min(1.0, lit * 4.0) * 1.6, world_);
  vec3 q = nearestPal(c + dth);
  vec3 o = mix(mix(c, q, uTint), q, env);
  o += uWarm * lit * (env * 0.8 + 0.2);
  o = mix(uPal[0], o, step(0.25, s.a));   // the void is exactly the palette's darkest (as before the art pass)
  o = mix(o, uPal[0], uFade);
  gl_FragColor = vec4(o, 1.0);
}`;

export class Blit {
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  readonly material: THREE.ShaderMaterial;
  private palArr: THREE.Vector3[] = Array.from({ length: 8 }, () => new THREE.Vector3());

  constructor(texture: THREE.Texture) {
    this.material = new THREE.ShaderMaterial({
      vertexShader: VERT,
      fragmentShader: FRAG,
      uniforms: {
        tex: { value: texture },
        uUvScale: { value: new THREE.Vector2(1, 1) },
        uUvOff: { value: new THREE.Vector2(0, 0) },
        uCam: { value: new THREE.Vector2(0, 0) },
        uTargetEnv: { value: new THREE.Vector2(1, 1) },
        uHero: { value: new THREE.Vector2(0, 0) },
        uPal: { value: this.palArr },
        uPalN: { value: 8 },
        uDither: { value: 0.09 },
        uTint: { value: 0.3 },   // was 0.5: monsters lost their painted contrast on same-hue floors
        uFade: { value: 0 },
        uFog: { value: new THREE.Vector2(5, 7.5) },
        uLights: { value: Array.from({ length: 12 }, () => new THREE.Vector2()) },
        uLightN: { value: 0 },
        uLightK: { value: 1 },
        uWarm: { value: new THREE.Vector3(0.16, 0.07, -0.02) },
      },
      depthTest: false,
      depthWrite: false,
    });
    const quad = new THREE.Mesh(new THREE.PlaneGeometry(2, 2), this.material);
    quad.frustumCulled = false;
    this.scene.add(quad);
  }

  setPalette(p: Palette): void {
    for (let i = 0; i < 8; i++) {
      const c = p[Math.min(i, p.length - 1)]!;
      this.palArr[i]!.set(c[0], c[1], c[2]);
    }
    this.material.uniforms.uPalN!.value = Math.min(8, p.length);
  }

  /** art pass: this frame's torch flames (world env texels; the first 12 are used) and the flicker scale */
  setLights(ls: readonly (readonly [number, number])[], k = 1): void {
    const u = this.material.uniforms, arr = u.uLights!.value as THREE.Vector2[];
    const n = Math.min(arr.length, ls.length);
    for (let i = 0; i < n; i++) arr[i]!.set(ls[i]![0], ls[i]![1]);
    u.uLightN!.value = n; u.uLightK!.value = k;
  }

  dispose(): void { this.material.dispose(); }
}
