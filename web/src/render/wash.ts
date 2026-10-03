// The wash look (docs/ART_DIRECTION.md §9): a prototype post-process that pulls the current game toward the art direction —
// gothic moonlight, a stark limited palette with one blood accent, ink edges, watercolour washes on the pixel grid, paper grain
// and a print halftone in the shadows. Behind a flag (`?look=wash`, or localStorage `riddle.look = "wash"`); off by default.
// Canvas: a GLSL chunk the blit appends as its last step (`LOOK` define, only when FX > 0 — the `low` tier compiles the blit exactly
// as before, and a wash frame costs ~9 texture reads per device pixel, no extra pass). Chrome: one SVG filter (the same palette
// map) on the DOM chrome and a paper-grain overlay (`installWashChrome`). Render-only: no game truth, nothing read back.

/** the look (art direction phase 2: ON by default — the art now carries the palette, the pass keeps the ink, the wash, the grain and
 *  the halftone); `?look=off` turns it off (sticky in localStorage), `?look=wash` turns it back on */
export function lookWash(): boolean {
  try {
    const q = new URLSearchParams(location.search).get("look");
    if (q === "wash" || q === "grade") { try { localStorage.removeItem("riddle.look"); } catch { /* storage blocked */ } return true; }
    if (q === "off" || q === "none") { try { localStorage.setItem("riddle.look", "off"); } catch { /* storage blocked */ } return false; }
    return localStorage.getItem("riddle.look") !== "off";
  } catch { return true; }
}

/** phase 1's DOM palette map (an SVG filter over the chrome) — the CSS is on the palette now, so only on request (`?look=grade`) */
function lookGrade(): boolean {
  try { return new URLSearchParams(location.search).get("look") === "grade"; } catch { return false; }
}

// The named palette (docs/ART_DIRECTION.md §2) — the one source the shader, the chrome filter and art-qc.py share by value.
export const WASH_PALETTE = {
  ink: "#0d0c14", umbra: "#1c1b2b", dusk: "#2b3350", moon: "#4d6c99", mist: "#a4bcd6", bone: "#eadfc5",
  blood: "#c01530", clot: "#5c0b1c", ember: "#e8923a", gilt: "#b89448",
} as const;

/** §2's per-place tint: the mid-tones (DUSK, MOON) lean toward it; INK, BONE and BLOOD never move */
export const WASH_TINT: Record<string, string> = {
  warrens: "#4a3b2c", burrows: "#5a4527", fens: "#2d5752", crypt: "#2f2c4f", foundry: "#5a2a1e", deep: "#1f2e4f", sanctum: "#6b6048",
  boss_flash: "#5c0b1c", default: "#2b3350",
};

export function hex3(h: string): [number, number, number] {
  return [parseInt(h.slice(1, 3), 16) / 255, parseInt(h.slice(3, 5), 16) / 255, parseInt(h.slice(5, 7), 16) / 255];
}
const g = (h: string): string => `vec3(${hex3(h).map((x) => x.toFixed(4)).join(", ")})`;

/** the blit's last step when `LOOK > 0`: `o` is the lit frame, `tuv` the target uv, `world` world env texels, `sa` the layer tag */
export const WASH_GLSL = /* glsl */ `
uniform vec3 uWashTint;   // the biome's mid-tone tint (WASH_TINT)
uniform float uWashDpr;   // device px per CSS px (the grain and the dots keep their size on every screen)
const vec3 W_INK = ${g(WASH_PALETTE.ink)};
const vec3 W_UMBRA = ${g(WASH_PALETTE.umbra)};
const vec3 W_DUSK = ${g(WASH_PALETTE.dusk)};
const vec3 W_MOON = ${g(WASH_PALETTE.moon)};
const vec3 W_MIST = ${g(WASH_PALETTE.mist)};
const vec3 W_BONE = ${g(WASH_PALETTE.bone)};
const vec3 W_BLOOD = ${g(WASH_PALETTE.blood)};
float wLum(vec3 c) { return dot(c, vec3(0.2126, 0.7152, 0.0722)); }
float wHash(vec2 p) { return fract(sin(dot(p, vec2(127.1, 311.7))) * 43758.5453); }
float wNoise(vec2 p) {
  vec2 i = floor(p), f = fract(p); f = f * f * (3.0 - 2.0 * f);
  return mix(mix(wHash(i), wHash(i + vec2(1.0, 0.0)), f.x), mix(wHash(i + vec2(0.0, 1.0)), wHash(i + vec2(1.0, 1.0)), f.x), f.y);
}
// the gradient map: luminance → INK · UMBRA · DUSK' · MOON' · MIST · BONE at their own luminances (so values hold), the two mid
// stops leaning toward the place's tint
vec3 wRamp(float l) {
  vec3 dk = mix(W_DUSK, uWashTint, 0.6), mn = mix(W_MOON, uWashTint * 1.7, 0.4);
  if (l < 0.07) return mix(W_INK, W_UMBRA, smoothstep(0.02, 0.07, l));
  if (l < 0.16) return mix(W_UMBRA, dk, smoothstep(0.07, 0.16, l));
  if (l < 0.34) return mix(dk, mn, smoothstep(0.16, 0.34, l));
  if (l < 0.62) return mix(mn, W_MIST, smoothstep(0.34, 0.62, l));
  return mix(W_MIST, W_BONE, smoothstep(0.62, 0.85, l));
}
vec3 washLook(vec3 o, vec2 tuv, vec2 world, float sa) {
  // Art direction phase 2: the art itself is on the palette now (sprites, tiles, props painted in moonlit ink and wash), so the pass
  // no longer maps the frame onto the ramp (phase 1's night curve + gradient map sank the new art to mush); it keeps what paint alone
  // cannot give a frame — a light pull toward the ramp's value bands, the ink on hard edges, pigment pooling, granulation on the grid,
  // screen paper grain and the halftone in the shadow band. BLOOD and flame keep their hue as before.
  float l = wLum(o);
  float rel = (o.r - max(o.g, o.b)) / max(o.r, 0.04);
  float orange = smoothstep(0.04, 0.14, o.g - o.b);
  float red = max(smoothstep(0.42, 0.62, rel) * smoothstep(0.05, 0.14, o.r) * (1.0 - orange),
                  smoothstep(0.3, 0.45, rel) * smoothstep(0.55, 0.8, l));
  // 1 · a light pull toward the place's ramp (a fifth), a tenth of the way to six value bands
  vec3 ramp = mix(wRamp(l), wRamp(floor(l * 6.0 + 0.5) / 6.0), 0.1);
  vec3 c = mix(o, ramp * (0.6 + 0.4 * o / max(vec3(l), vec3(0.01))), 0.2 * (1.0 - red));
  // 2 · the ink line on hard luminance edges of the source (silhouettes, wall lips), lighter than phase 1's
  vec2 t = uTexel;
  float lr = wLum(texture2D(tex, tuv + vec2(t.x, 0.0)).rgb), ll = wLum(texture2D(tex, tuv - vec2(t.x, 0.0)).rgb);
  float lu = wLum(texture2D(tex, tuv + vec2(0.0, t.y)).rgb), ld_ = wLum(texture2D(tex, tuv - vec2(0.0, t.y)).rgb);
  float e = max(abs(lr - ll), abs(lu - ld_));
  float ink = smoothstep(0.22, 0.45, e) * step(0.25, sa);
  // 3 · the wash: pigment pools at a darker edge, blooms a little where lighter (3-texel neighbourhood)
  float lb = 0.25 * (wLum(texture2D(tex, tuv + vec2(3.0 * t.x, 0.0)).rgb) + wLum(texture2D(tex, tuv - vec2(3.0 * t.x, 0.0)).rgb)
                   + wLum(texture2D(tex, tuv + vec2(0.0, 3.0 * t.y)).rgb) + wLum(texture2D(tex, tuv - vec2(0.0, 3.0 * t.y)).rgb));
  float dv = wLum(texture2D(tex, tuv).rgb) - lb;
  c *= 1.0 + clamp(dv * 0.8, -0.14, 0.08);
  //     granulation on the pixel grid (world-anchored at the sprite texel)
  vec2 wp = floor(world * 2.0) / 2.0;
  float gran = wNoise(wp / 5.0) * 0.6 + wNoise(wp / 1.7) * 0.4;
  c *= 0.94 + 0.12 * gran * smoothstep(0.03, 0.2, l);
  c = mix(c, W_INK, ink * 0.4);
  // 4 · paper: grain and fibre in CSS px (screen-anchored, like print); the brightest lights lean to BONE
  //    (blind round 27: screen-fixed grain and dots on the ground made the whole floor "shimmer" in motion as the camera glided — on
  //    world texels they are anchored to the world instead, ~4 CSS px per env texel, so the ground carries its grain with it)
  vec2 sp = mix(gl_FragCoord.xy / uWashDpr, world * 4.0, step(0.95, sa));
  float grain = wHash(floor(sp)) * 0.55 + wNoise(sp * vec2(0.08, 0.5)) * 0.45;
  c *= 0.96 + 0.07 * grain;
  c = mix(c, W_BONE * (0.9 + 0.1 * grain), smoothstep(0.7, 0.98, l) * 0.2 * (1.0 - red));
  // 5 · halftone: a 45° dot screen (3.2 CSS px pitch) inside the shadow band only
  vec2 hp = mat2(0.7071, -0.7071, 0.7071, 0.7071) * sp / 3.2;
  float dot_ = length(fract(hp) - 0.5);
  float shadow = smoothstep(0.16, 0.06, l) * smoothstep(0.012, 0.04, l);
  c = mix(c, W_INK, step(dot_, 0.34) * shadow * 0.45);
  return c;
}`;

/** chrome: the palette map as an SVG filter on the DOM chrome (reds and flame kept) and a paper-grain overlay over everything */
export function installWashChrome(): void {
  if (!lookWash() || typeof document === "undefined") return;
  document.documentElement.classList.add("look-wash");
  const [ink, dusk, moon, mist, bone] = [WASH_PALETTE.ink, WASH_PALETTE.dusk, WASH_PALETTE.moon, WASH_PALETTE.mist, WASH_PALETTE.bone].map(hex3);
  const tbl = (i: number): string => [ink![i], dusk![i], moon![i], mist![i], bone![i]].map((x) => x!.toFixed(3)).join(" ");
  const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="0" height="0" style="position:absolute" aria-hidden="true">
<filter id="wash-grade" color-interpolation-filters="sRGB">
  <feColorMatrix in="SourceGraphic" type="matrix" values="0.2126 0.7152 0.0722 0 0  0.2126 0.7152 0.0722 0 0  0.2126 0.7152 0.0722 0 0  0 0 0 1 0" result="grey"/>
  <feComponentTransfer in="grey" result="ramp"><feFuncR type="table" tableValues="${tbl(0)}"/><feFuncG type="table" tableValues="${tbl(1)}"/><feFuncB type="table" tableValues="${tbl(2)}"/></feComponentTransfer>
  <feComposite in="ramp" in2="SourceGraphic" operator="arithmetic" k2="0.78" k3="0.22" result="mapped"/>
  <feColorMatrix in="SourceGraphic" type="matrix" values="1 0 0 0 0  0 1 0 0 0  0 0 1 0 0  3.2 -2.4 -0.8 0 -0.45" result="reds"/>
  <feComposite in="reds" in2="mapped" operator="over" result="kept"/>
  <feComposite in="kept" in2="SourceGraphic" operator="in"/>
</filter></svg>`;
  const grain = `url("data:image/svg+xml;utf8,${encodeURIComponent(`<svg xmlns='http://www.w3.org/2000/svg' width='220' height='220'><filter id='n'><feTurbulence type='fractalNoise' baseFrequency='0.85' numOctaves='2' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 0.92  0 0 0 0 0.87  0 0 0 0 0.77  0 0 0 0.55 0'/></filter><rect width='100%' height='100%' filter='url(#n)'/></svg>`)}")`;
  const blot = `url("data:image/svg+xml;utf8,${encodeURIComponent(`<svg xmlns='http://www.w3.org/2000/svg' width='640' height='640'><filter id='b'><feTurbulence type='fractalNoise' baseFrequency='0.006' numOctaves='3' seed='7' stitchTiles='stitch'/><feColorMatrix values='0 0 0 0 0.11  0 0 0 0 0.11  0 0 0 0 0.17  0 0 0 -1.6 1.05'/></filter><rect width='100%' height='100%' filter='url(#b)'/></svg>`)}")`;
  const style = document.createElement("style");
  style.id = "look-wash";
  // the chrome (everything but the viewer canvases, which the blit grades itself) takes the palette map; one fixed overlay lays
  // the paper grain (multiply) and a faint wash bloom over the whole screen, clicks pass through
  style.textContent = `
html.look-wash body { background: ${WASH_PALETTE.ink}; }
${lookGrade() ? "html.look-wash :is(main.frame > :not(:has(canvas)), main.frame > :has(canvas) > :not(:has(canvas)):not(canvas), .sheet-wrap) { filter: url(#wash-grade); }" : ""}
html.look-wash::after { content: ""; position: fixed; inset: 0; pointer-events: none; z-index: 2147483000;
  background-image: ${grain}, ${blot}; background-size: 220px 220px, 640px 640px; mix-blend-mode: multiply; opacity: 0.4; }`;
  const host = document.createElement("div");
  host.innerHTML = svg;
  const ready = (): void => { document.head.appendChild(style); document.body.appendChild(host.firstElementChild!); };
  if (document.body) ready(); else addEventListener("DOMContentLoaded", ready, { once: true });
}
