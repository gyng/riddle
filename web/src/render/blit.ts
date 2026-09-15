// Fullscreen blit: nearest-samples the low-res target with the sub-texel camera remainder as a UV
// offset, applies 2-band distance fog from the hero, then per-biome palette quantisation.
// Layer tag = target alpha: 1.0 (environment) → nearest palette colour after a 4×4 Bayer offset
// indexed by WORLD env texel (so the pattern sticks to the dungeon, not the screen);
// 0.5 (sprites) → 50% blend toward the un-dithered nearest palette colour ("tint", not quantise).
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
  vec3 c = s.rgb * fog;
  float env = step(0.75, s.a);
  float dth = (bayer4(floor(world)) - 0.5) * uDither * env;
  vec3 q = nearestPal(c + dth);
  vec3 o = mix(mix(c, q, uTint), q, env);
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
        uTint: { value: 0.5 },
        uFade: { value: 0 },
        uFog: { value: new THREE.Vector2(5, 7.5) },
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

  dispose(): void { this.material.dispose(); }
}
