// Fullscreen blit: nearest-samples the low-res target with the sub-texel camera remainder as a UV offset. Layer tag = target
// alpha: 1.0 (world: tiles and dressing), 0.875 (env chrome: items, glyphs, text) → nearest palette colour after a 4×4 Bayer
// offset indexed by WORLD env texel; 0.5 (sprites) → a 30 % blend toward the nearest palette colour ("tint", not quantise).
// Art pass (art/ui/ART_GAP.md §5): up to 12 torch lights (world env texels), quadratic falloff; the hero carries a little light.
// Second art pass: the WORLD is authored in the ramp and is no longer re-quantised or dithered here — a smooth light multiplies
// it (a per-biome grade `GRADES`, ambient, warm torch pools, a halo on each flame, one soft fog falloff past the vision), so
// the memory dim and the fog darken stone instead of breaking it into speckle. Lift and halo shrink on pale stone.
// Juice (docs/JUICE.md): `FX` (a define, the quality tier) — 0 compiles exactly the blit above; 1 reads the light field (light.ts:
// coloured, shadowed, wall-foot AO) in place of the torch pools, adds bloom (bloom.ts), the vignette and the death's desaturation;
// 2 adds the rim light on sprites, the stone's relief under the light and heat shimmer over fire.
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
uniform vec3 uWarm;       // the warm tint a full light adds to a sprite
uniform vec3 uGrade;      // second art pass: the world's per-biome colour grade (render-only)
uniform float uAmbient;   // world light away from any torch
uniform float uLiftK;     // torch/hero pool strength on the world
uniform vec3 uLightCol;   // the torch colour
uniform float uSat;       // the world's saturation (1 = the ramp's own)
#if FX > 0
// juice (docs/JUICE.md): the light field (light.ts), bloom (bloom.ts), the vignette and the death's desaturation
uniform sampler2D uLightMap;  // rgb = light / 2, a = wall-foot AO; S texels per tile, row space
uniform vec2 uMap;            // the floor in tiles
uniform sampler2D uBloom;
uniform float uBloomK;
uniform vec2 uTexel;          // one target texel in uv
uniform vec4 uVig;            // pulse colour, strength
uniform float uVigBase;       // the corners' static darkening
uniform float uDesat;
uniform float uAmbK;          // the ambient's share under the light field (the pools read against a darker room)
uniform float uTime;
uniform vec2 uHeat[8];        // fires (world env texels): the air shimmers over them
uniform int uHeatN;
#endif
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
#if FX > 1
  {  // heat shimmer: a whole-texel wobble in the air over a fire (high)
    vec2 w0 = uCam + (tuv - 0.5) * uTargetEnv;
    float hs = 0.0;
    for (int i = 0; i < 8; i++) {
      if (i >= uHeatN) break;
      vec2 d = w0 - uHeat[i];
      hs = max(hs, (1.0 - smoothstep(0.0, 9.0, abs(d.x))) * smoothstep(-3.0, 2.0, d.y) * (1.0 - smoothstep(6.0, 16.0, d.y)));
    }
    tuv.x += floor(sin(w0.y * 1.6 + uTime * 7.0) * 1.2 * hs + 0.5) * uTexel.x;
  }
#endif
  vec4 s = texture2D(tex, tuv);
  vec2 world = uCam + (tuv - 0.5) * uTargetEnv;
  float d = length(world - uHero) / 8.0;
  // second art pass: one soft falloff past the floor's vision (was two hard bands, 0.8 / 0.6, that stacked with the memory dim)
  float fog = 1.0 - 0.3 * smoothstep(uFog.x, uFog.y + 2.0, d);
  float lit = 0.0, glow = 0.0;
  for (int i = 0; i < 12; i++) {
    if (i >= uLightN) break;
    vec2 dl = (world - uLights[i]) * vec2(1.0, 1.25);
    float f = max(0.0, 1.0 - length(dl) / 30.0);
    lit += f * f;
    float g = max(0.0, 1.0 - length(world - uLights[i]) / 7.0);   // second art pass: the flame's own halo (watch.png's sconces glow)
    glow += g * g;
  }
  float env = step(0.75, s.a);
  float world_ = step(0.95, s.a);   // tiles and dressing (a=1); env chrome (items, glyphs, text) is a=0.875
  // the hero carries a little light of his own (the room around him reads, as in the target's frame)
  float hl = max(0.0, 1.0 - length((world - uHero) * vec2(1.0, 1.2)) / 40.0);
  lit += 0.35 * hl * hl;
  lit = min(lit, 1.4) * uLightK * mix(1.0, world_, env) * step(0.25, s.a);   // the void (a=0) stays dark
  float lum = dot(s.rgb, vec3(0.2126, 0.7152, 0.0722));
#if FX > 0
  // the light field replaces the torch pools: coloured, shadowed by the walls, the hero's own light in it; AO at the wall's foot
  vec2 luv = vec2(world.x / (8.0 * uMap.x), 1.0 + world.y / (8.0 * uMap.y));
  vec4 lm = texture2D(uLightMap, luv);
  vec3 LF = lm.rgb * 2.0 * uLightK * mix(1.0, world_, env) * step(0.25, s.a);
  float lfl = dot(LF, vec3(0.3, 0.5, 0.2));
  lit = min(lfl, 1.4);
#endif
  // the lift shrinks on pale stone (the Sanctum's ramp would burn to white): full on dark rock, a quarter on marble
  float lift = lit * (1.0 - 0.75 * smoothstep(0.25, 0.8, lum));
  // WORLD (a=1): tiles are authored in the ramp, so they are not re-quantised: a smooth light multiplies them (an ambient a
  // little under 1, graded per biome toward the target's warm stone, and a warm torch colour in the pools). No dither, no
  // steps: the memory dim and the fog darken stone, they no longer break it into speckle.
#if FX > 0
  float shrink = 1.0 - 0.75 * smoothstep(0.25, 0.8, lum);
  vec3 wc = mix(vec3(lum), s.rgb, uSat) * uGrade * fog * mix(1.0, lm.a, world_) * (uAmbient * uAmbK + uLiftK * 1.15 * shrink * LF);
#if FX > 1
  // the stone's relief under the light: the luma as a height, embossed along the light's gradient (high)
  vec2 gl2 = vec2(dot(texture2D(uLightMap, luv + vec2(0.5 / (4.0 * uMap.x), 0.0)).rgb - texture2D(uLightMap, luv - vec2(0.5 / (4.0 * uMap.x), 0.0)).rgb, vec3(1.0)),
                  dot(texture2D(uLightMap, luv + vec2(0.0, 0.5 / (4.0 * uMap.y))).rgb - texture2D(uLightMap, luv - vec2(0.0, 0.5 / (4.0 * uMap.y))).rgb, vec3(1.0)));
  float gm = length(gl2);
  vec2 ld = gm > 1e-4 ? gl2 / gm : vec2(0.0);
  vec4 nb = texture2D(tex, tuv + ld * uTexel);
  float nl = dot(nb.rgb, vec3(0.2126, 0.7152, 0.0722));
  float same = step(0.95, nb.a);
  wc *= 1.0 + world_ * same * clamp((lum - nl) * 1.6, -0.22, 0.22) * min(1.0, lfl * 1.5);
#endif
#else
  vec3 wc = mix(vec3(lum), s.rgb, uSat) * uGrade * fog * (uAmbient + uLiftK * lift * uLightCol);
#endif
  // SPRITES (a=0.5) and ENV CHROME (a=0.875): as before — sprites lifted and tinted 30 % toward the ramp, chrome quantised
  vec3 c = s.rgb * fog * (1.0 + 0.9 * lift);
#if FX > 0
  c *= mix(vec3(1.0), 0.75 + 0.5 * normalize(LF + vec3(0.05)), min(1.0, lfl) * 0.5);   // a sprite takes the light's hue (fire reads orange on a foe)
#endif
#if FX > 1
  {  // rim light: a sprite's edge that faces the light catches it (high)
    float spr = 1.0 - step(0.03, abs(s.a - 0.5));
    if (spr > 0.5 && gm > 1e-4) {
      float toward = 1.0 - step(0.03, abs(texture2D(tex, tuv + ld * uTexel).a - 0.5));
      float away = 1.0 - step(0.03, abs(texture2D(tex, tuv - ld * uTexel).a - 0.5));
      c += (1.0 - toward) * normalize(LF + vec3(0.05)) * 0.55 * min(1.0, lfl * 1.2);
      c *= 1.0 - 0.14 * (1.0 - away) * min(1.0, lfl);
    }
  }
#endif
  float dth = (bayer4(floor(world)) - 0.5) * uDither * env;
  vec3 q = nearestPal(c + dth);
  vec3 o = mix(mix(c, q, uTint), q, env);
  o += uWarm * lit * 0.2;
  o = mix(o, wc, world_);
  o += vec3(1.0, 0.55, 0.18) * 0.32 * min(glow, 1.0) * (1.0 - 0.7 * smoothstep(0.4, 0.85, lum)) * uLightK * step(0.25, s.a) * (1.0 - env * (1.0 - world_));
  o = mix(uPal[0], o, step(0.25, s.a));   // the void is exactly the palette's darkest (as before the art pass)
#if FX > 0
  o += texture2D(uBloom, tuv).rgb * uBloomK * fog;
  float vr = length((vUv - 0.5) * vec2(0.9, 1.0));
  float vg = smoothstep(0.32, 0.78, vr);
  o *= 1.0 - uVigBase * vg;
  o = mix(o, uVig.rgb, clamp(uVig.a * (0.35 + 0.65 * vg), 0.0, 1.0) * smoothstep(0.15, 0.7, vr));
  o = mix(o, vec3(dot(o, vec3(0.2126, 0.7152, 0.0722))), uDesat);
#endif
  o = mix(o, uPal[0], uFade);
  gl_FragColor = vec4(o, 1.0);
}`;

// Second art pass (art/ui/ART_GAP.md): the world's grade per biome — rgb multipliers on the authored ramp colours, the ambient
// light away from a torch, the torch pools' strength and the saturation. The Warrens' olive ramp is pulled toward the target's warm brown
// stone (watch.png's floor averages (63, 50, 27)); the pale Sanctum gets a low ambient and a weak lift so a torch never blooms.
const GRADES: Record<string, [number, number, number, number, number, number]> = {
  default: [1, 1, 1, 0.84, 0.65, 1],
  warrens: [1.14, 0.96, 0.78, 0.8, 0.95, 0.55],
  burrows: [1, 0.94, 0.88, 0.8, 0.7, 0.85],
  fens: [0.96, 1, 1, 0.82, 0.65, 0.9],
  crypt: [1, 1, 1.02, 0.82, 0.65, 0.9],
  foundry: [1, 0.94, 0.88, 0.82, 0.6, 0.9],
  deep: [1, 1, 1, 0.9, 0.7, 1],
  sanctum: [0.92, 0.9, 0.88, 0.78, 0.3, 0.8],
  boss_flash: [1, 1, 1, 1, 0.3, 1],
};

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
        uGrade: { value: new THREE.Vector3(1, 1, 1) },
        uAmbient: { value: 0.86 },
        uLiftK: { value: 0.6 },
        uLightCol: { value: new THREE.Vector3(1.0, 0.72, 0.4) },
        uSat: { value: 1 },
        // juice (docs/JUICE.md): read only when FX > 0
        uLightMap: { value: null }, uMap: { value: new THREE.Vector2(1, 1) }, uBloom: { value: null }, uBloomK: { value: 0 },
        uTexel: { value: new THREE.Vector2(1, 1) }, uVig: { value: new THREE.Vector4(0, 0, 0, 0) }, uVigBase: { value: 0 }, uDesat: { value: 0 },
        uAmbK: { value: 0.8 }, uTime: { value: 0 }, uHeat: { value: Array.from({ length: 8 }, () => new THREE.Vector2()) }, uHeatN: { value: 0 },
      },
      defines: { FX: 0 },
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

  /** second art pass: the world's colour grade and light for a biome (GRADES; unknown biomes neutral) */
  setGrade(biome: string): void {
    const g = GRADES[biome] ?? GRADES.default!, u = this.material.uniforms;
    (u.uGrade!.value as THREE.Vector3).set(g[0], g[1], g[2]);
    u.uAmbient!.value = g[3]; u.uLiftK!.value = g[4]; u.uSat!.value = g[5];
  }

  /** art pass: this frame's torch flames (world env texels; the first 12 are used) and the flicker scale */
  setLights(ls: readonly (readonly [number, number])[], k = 1): void {
    const u = this.material.uniforms, arr = u.uLights!.value as THREE.Vector2[];
    const n = Math.min(arr.length, ls.length);
    for (let i = 0; i < n; i++) arr[i]!.set(ls[i]![0], ls[i]![1]);
    u.uLightN!.value = n; u.uLightK!.value = k;
  }

  /** juice: the effects tier compiled into the shader (0 = low: the pre-juice blit exactly; 1 = med; 2 = high) */
  setFx(level: number): void {
    const d = this.material.defines as { FX: number };
    if (d.FX === level) return;
    d.FX = level; this.material.needsUpdate = true;
  }

  dispose(): void { this.material.dispose(); }
}
