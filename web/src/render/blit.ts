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
import { hex3, lookWash, WASH_GLSL, WASH_TINT } from "./wash";

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
uniform float uSprHue;    // gfx round 17: how much of the light's hue a sprite takes (0.5; the Fens 0.15)
uniform float uMist;      // gfx round 11: a drifting ground mist over the world (the Fens), 0 = none
uniform float uCon;       // gfx round 10: the world's contrast (1 = the ramp's own; lower pulls the floor's texture toward its mid tone)
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
uniform sampler2D uNormal;    // juice pass 2: the sprites' derived normals (normals.ts; rgb = n / 2 + 0.5, a = 1 where a sprite drew)
#endif
varying vec2 vUv;
${BAYER_GLSL}
#if LOOK > 0
${WASH_GLSL}
#endif
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
  // gfx round 10 (raters, every round: "the floor's texture is louder than the actors"): the world's luminance pulled toward a mid tone
  vec3 sc = s.rgb * (mix(0.3, lum, uCon) / max(lum, 0.02));
  vec3 wc = mix(vec3(dot(sc, vec3(0.2126, 0.7152, 0.0722))), sc, uSat) * uGrade * fog * mix(1.0, lm.a, world_) * (uAmbient * uAmbK + uLiftK * 1.15 * shrink * LF);
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
  c *= mix(vec3(1.0), 0.75 + 0.5 * normalize(LF + vec3(0.05)), min(1.0, lfl) * uSprHue);   // a sprite takes the light's hue (fire reads orange on a foe); round 17: less in the Fens ("a teal hero on a teal floor")
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
    // juice pass 2: the derived normal map — N·L toward the light field's gradient (a soft key from the upper left where the field
    // is flat) shades the sprite as a lit volume; the light's strength scales it, so a sprite in the dark stays as painted
    vec4 nm = texture2D(uNormal, tuv);
    if (spr > 0.5 && nm.a > 0.5) {
      vec3 N = normalize(nm.rgb * 2.0 - 1.0);
      vec3 Ldir = normalize(vec3(mix(vec2(-0.45, 0.55), ld, min(1.0, gm * 6.0)), 0.75));
      float ndl = dot(N, Ldir);
      float k = min(1.0, lfl * 1.4);
      c *= 1.0 + k * (0.34 * clamp(ndl - 0.55, -0.6, 0.45) + 0.1 * pow(max(0.0, ndl), 8.0));
    }
  }
#endif
  float dth = (bayer4(floor(world)) - 0.5) * uDither * env;
  vec3 q = nearestPal(c + dth);
  vec3 o = mix(mix(c, q, uTint), q, env);
  o += uWarm * lit * 0.2;
  o = mix(o, wc, world_);
  o += vec3(1.0, 0.55, 0.18) * 0.32 * min(glow, 1.0) * (1.0 - 0.7 * smoothstep(0.4, 0.85, lum)) * uLightK * step(0.25, s.a) * (1.0 - env * (1.0 - world_));
#if FX > 0
  {  // gfx round 1 (the blind raters: "half the viewport is empty black void"): the unexplored rock is a dark coursed-stone mass in the
     // biome's darkest two colours — texel-crisp blocks and mortar that fade out away from the hero — never a flat black (no truth: it
     // draws only where no tile drew, and never reveals one)
    vec2 wt = floor(world) + 0.5;
    vec2 cell = floor(wt / 7.0);
    float f1 = 1e9, f2 = 1e9, n = 0.0;
    for (int yy = -1; yy <= 1; yy++) for (int xx = -1; xx <= 1; xx++) {
      vec2 c = cell + vec2(float(xx), float(yy));
      vec2 h = fract(sin(vec2(dot(c, vec2(127.1, 311.7)), dot(c, vec2(269.5, 183.3)))) * 43758.5453);
      float dd = length(wt - (c + 0.15 + 0.7 * h) * 7.0);
      if (dd < f1) { f2 = f1; f1 = dd; n = h.x; } else if (dd < f2) { f2 = dd; }
    }
    float crack = 1.0 - smoothstep(0.6, 1.4, f2 - f1);
    float g = fract(sin(dot(wt, vec2(39.3468, 11.1353))) * 24634.6345);
    vec3 rock = mix(uPal[0], uPal[1], 0.16 + 0.3 * n + 0.08 * g) * mix(1.0, 0.45, crack) * (1.0 - 0.25 * smoothstep(1.0, 3.5, f1) * n);
    vec3 rockC = rock;
    rock *= 0.62 * (0.3 + 0.7 * (1.0 - smoothstep(1.5, 11.0, d)));   // gfx round 10: the rock mass stays in view further out ("top half black void")   // gfx round 4: the rock shows at the lit edge, fading to black (raters: "a flat black pattern")
    // gfx round 7 (the lit fog edge; raters, every round: "half the view is void"): the rock face at the explored edge catches the light
    // of the seen ground beside it — eight directions, one to three half-tiles out; only drawn world texels (seen tiles) lend their light,
    // so an unseen room's light never shows through the rock
    vec3 edgeL = vec3(0.0);
    for (int i = 0; i < 8; i++) {
      float ang = float(i) * 0.7853982;
      vec2 dir = vec2(cos(ang), sin(ang));
      for (int j = 1; j <= 3; j++) {
        float r = float(j) * 4.0;
        vec4 s2 = texture2D(tex, tuv + dir * r / uTargetEnv);
        if (s2.a > 0.95) {
          vec2 w2 = world + dir * r;
          vec3 L2 = texture2D(uLightMap, vec2(w2.x / (8.0 * uMap.x), 1.0 + w2.y / (8.0 * uMap.y))).rgb * 2.0 * uLightK;
          edgeL = max(edgeL, L2 * (1.0 - r / 15.0));
        }
      }
    }
    rock += (rockC + 0.07) * min(edgeL, vec3(1.2)) * 1.7 * mix(1.0, 0.5, crack);
    o = mix(rock, o, step(0.25, s.a));
  }
#else
  o = mix(uPal[0], o, step(0.25, s.a));   // the void is exactly the palette's darkest (as before the art pass)
#endif
#if FX > 0
  {  // gfx round 7 (raters: "the hero doesn't pop against the busy teal floor"): a one-texel dark rim round every sprite, on the ground
     // beside it (the classic pixel-art outline; the sprite itself is untouched)
    float isSpr = 1.0 - step(0.03, abs(s.a - 0.5));
    if (isSpr < 0.5) {
      float nbS = 0.0;
      nbS += 1.0 - step(0.03, abs(texture2D(tex, tuv + vec2(uTexel.x, 0.0)).a - 0.5));
      nbS += 1.0 - step(0.03, abs(texture2D(tex, tuv - vec2(uTexel.x, 0.0)).a - 0.5));
      nbS += 1.0 - step(0.03, abs(texture2D(tex, tuv + vec2(0.0, uTexel.y)).a - 0.5));
      nbS += 1.0 - step(0.03, abs(texture2D(tex, tuv - vec2(0.0, uTexel.y)).a - 0.5));
      o *= 1.0 - 0.6 * min(1.0, nbS);
    }
  }
  if (uMist > 0.0) {   // gfx round 11 (raters: "add fog layers so the Fens feel wet"): two slow octaves of value noise in world space
    vec2 mp = world / 26.0 + vec2(uTime * 0.035, uTime * 0.012);
    vec2 mi = floor(mp), mf = fract(mp); mf = mf * mf * (3.0 - 2.0 * mf);
    float h00 = fract(sin(dot(mi, vec2(127.1, 311.7))) * 43758.5), h10 = fract(sin(dot(mi + vec2(1.0, 0.0), vec2(127.1, 311.7))) * 43758.5);
    float h01 = fract(sin(dot(mi + vec2(0.0, 1.0), vec2(127.1, 311.7))) * 43758.5), h11 = fract(sin(dot(mi + vec2(1.0, 1.0), vec2(127.1, 311.7))) * 43758.5);
    float n1 = mix(mix(h00, h10, mf.x), mix(h01, h11, mf.x), mf.y);
    vec2 mp2 = world / 11.0 - vec2(uTime * 0.05, 0.0);
    vec2 ni = floor(mp2), nf = fract(mp2); nf = nf * nf * (3.0 - 2.0 * nf);
    float g00 = fract(sin(dot(ni, vec2(269.5, 183.3))) * 43758.5), g10 = fract(sin(dot(ni + vec2(1.0, 0.0), vec2(269.5, 183.3))) * 43758.5);
    float g01 = fract(sin(dot(ni + vec2(0.0, 1.0), vec2(269.5, 183.3))) * 43758.5), g11 = fract(sin(dot(ni + vec2(1.0, 1.0), vec2(269.5, 183.3))) * 43758.5);
    float n2 = mix(mix(g00, g10, nf.x), mix(g01, g11, nf.x), nf.y);
    float m = smoothstep(0.35, 0.85, n1 * 0.65 + n2 * 0.35) * uMist * step(0.25, s.a);
    o = mix(o, vec3(0.64, 0.74, 0.84) * (0.35 + 0.65 * min(1.0, lfl + 0.3)), m);   // phase 2: MIST
  }
  o += texture2D(uBloom, tuv).rgb * uBloomK * fog;
  float vr = length((vUv - 0.5) * vec2(0.9, 1.0));
  float vg = smoothstep(0.32, 0.78, vr);
  o *= 1.0 - uVigBase * vg;
  o = mix(o, uVig.rgb, clamp(uVig.a * (0.35 + 0.65 * vg), 0.0, 1.0) * smoothstep(0.15, 0.7, vr));
  o = mix(o, vec3(dot(o, vec3(0.2126, 0.7152, 0.0722))), uDesat);
#endif
#if LOOK > 0
  o = washLook(o, tuv, world, s.a);   // the wash look (wash.ts, docs/ART_DIRECTION.md §9): FX > 0 and the flag only
#endif
  o = mix(o, uPal[0], uFade);
  gl_FragColor = vec4(o, 1.0);
}`;

// Second art pass (art/ui/ART_GAP.md): the world's grade per biome — rgb multipliers on the authored ramp colours, the ambient
// light away from a torch, the torch pools' strength and the saturation. The Warrens' olive ramp is pulled toward the target's warm brown
// stone (watch.png's floor averages (63, 50, 27)); the pale Sanctum gets a low ambient and a weak lift so a torch never blooms.
const GRADES: Record<string, [number, number, number, number, number, number]> = {
  // art direction phase 2 (docs/ART_DIRECTION.md §2, §6): the art carries the palette and the place's tint itself, so the grade is a cold
  // moonlit night over it — rgb a touch toward MOON, a low ambient (~60 % of the frame in INK/UMBRA), the EMBER pools (lift) the warm
  // counterpoint. The v2 grades (warm brown Warrens, ochre Burrows) are in git history at 7253bb9.
  default: [0.96, 0.99, 1.05, 0.74, 0.95, 1],
  warrens: [0.96, 0.98, 1.04, 0.74, 1.0, 1],
  burrows: [1, 0.98, 0.97, 0.74, 0.95, 1],
  fens: [0.94, 1, 1.05, 0.62, 1.0, 0.95],
  crypt: [0.95, 0.97, 1.06, 0.74, 0.85, 0.95],
  foundry: [1, 0.97, 0.97, 0.76, 0.8, 1],
  deep: [0.94, 0.98, 1.08, 0.7, 0.85, 1],
  sanctum: [0.96, 0.97, 1.0, 0.78, 0.45, 0.9],
  town: [0.95, 1, 1.04, 0.74, 1.0, 1],
  boss_flash: [1, 1, 1, 1, 0.3, 1],
};

/** gfx round 10: the world's contrast per biome (FX > 0; the Fens' plank stripes and the Warrens' flagstones read as noise behind the cast) */
const CONTRAST: Record<string, number> = { fens: 1.02, warrens: 1, burrows: 1, crypt: 0.9, foundry: 0.9, deep: 0.9 };   // gfx round 18: the painted register carries its own values

const MIST: Record<string, number> = { fens: 0.18, crypt: 0.12, deep: 0.14 };   // (round 18: 0.42 "flattens the floor, walls and items together")   // gfx round 11: ground mist per biome (FX > 0)

export class Blit {
  readonly scene = new THREE.Scene();
  readonly camera = new THREE.OrthographicCamera(-1, 1, 1, -1, 0, 1);
  readonly material: THREE.ShaderMaterial;
  private readonly wash = lookWash();
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
        uWarm: { value: new THREE.Vector3(0.04, 0.07, 0.12) },   // phase 2: a lit sprite takes a cold MIST edge, not amber
        uGrade: { value: new THREE.Vector3(1, 1, 1) },
        uAmbient: { value: 0.86 },
        uLiftK: { value: 0.6 },
        uLightCol: { value: new THREE.Vector3(1.0, 0.62, 0.26) },   // phase 2: EMBER
        uSat: { value: 1 }, uCon: { value: 1 }, uMist: { value: 0 }, uSprHue: { value: 0.5 },
        // juice (docs/JUICE.md): read only when FX > 0
        uLightMap: { value: null }, uMap: { value: new THREE.Vector2(1, 1) }, uBloom: { value: null }, uBloomK: { value: 0 },
        uTexel: { value: new THREE.Vector2(1, 1) }, uVig: { value: new THREE.Vector4(0, 0, 0, 0) }, uVigBase: { value: 0 }, uDesat: { value: 0 },
        uAmbK: { value: 0.8 }, uTime: { value: 0 }, uNormal: { value: null }, uHeat: { value: Array.from({ length: 8 }, () => new THREE.Vector2()) }, uHeatN: { value: 0 },
        uWashTint: { value: new THREE.Vector3(...hex3(WASH_TINT.default!)) }, uWashDpr: { value: 1 },
      },
      defines: { FX: 0, LOOK: 0 },
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
    u.uAmbient!.value = g[3]; u.uLiftK!.value = g[4]; u.uSat!.value = g[5]; u.uCon!.value = CONTRAST[biome] ?? 1; u.uMist!.value = MIST[biome] ?? 0; u.uSprHue!.value = biome === "fens" ? 0.15 : 0.5;
    (u.uWashTint!.value as THREE.Vector3).set(...hex3(WASH_TINT[biome] ?? WASH_TINT.default!));
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
    const d = this.material.defines as { FX: number; LOOK: number };
    const look = level > 0 && this.wash ? 1 : 0;   // the wash look never reaches the `low` tier
    if (d.FX === level && d.LOOK === look) return;
    d.FX = level; d.LOOK = look; this.material.needsUpdate = true;
  }

  dispose(): void { this.material.dispose(); }
}
