# Rendering a lo-fi pixel-art dungeon in three.js with watercolour sprites

Research notes for the riddle frontend (Rust→WASM sim, three.js/WebGL2, phone-first). Date: 2026-09-15.
Method note: web search quota was exhausted mid-task, so most evidence below is from primary pages fetched
directly (developer video descriptions/transcripts, engine docs, three.js source) plus **my own measurements of
Steam store screenshots** (run-length histogram of the JPEGs to find the integer pixel quantum; script in the
scratchpad). Confidence tags: [high] / [med] / [low].

---

## 1. Resolution and texel size

### What the reference games actually do

| Game | Internal res / pixel quantum | Tile / texel | Hero size | Evidence |
|---|---|---|---|---|
| **Downwell** | Steam screenshots are 1140×852 with a clean **3-px quantum → 380×284** internal (that capture). The well is **~166 texels wide** (outer wall to outer wall). Portrait game, 4-colour palette (black/white/red + blue), palette swaps unlockable. | Wall dither repeats every **8 px**; breakable blocks are **~30×14 (a 4×2 or 2×1 tile block)** | **~12×12** texels | Measured from https://store.steampowered.com/app/360740/ screenshots [med: capture may be windowed]; palette from https://en.wikipedia.org/wiki/Downwell_(video_game) [high] |
| **Celeste** | 1920×1080 screenshots show a **6-px quantum → 320×180** | **8×8** tiles (48-px runs at 6×) | Madeline ≈ 8 wide × ~16 tall (hitbox 8×11) | Measured from https://store.steampowered.com/app/504230/ [high for 320×180 and 8-px tiles; med for hero] |
| **Hyper Light Drifter** | **4-px quantum → 480×270** | 16-px tile grid (16/32-px runs dominate) | Drifter ≈ 24–32 px tall | Measured from https://store.steampowered.com/app/257850/ [high for 480×270; low for hero] |
| **Dead Cells** | One of three screenshots shows a dominant 4-px quantum (**≈480×270 art density at 1080p**); the other two show no clean quantum (dynamic lighting, non-integer sprite placement). | Backgrounds ~4 px/texel at 1080p | Beheaded ≈ 40–50 texels tall | Measured, app 588650 [med]. Pipeline: 3D models animated then rendered to sprite sheets, downscaled, outlines + palette reduction applied in a batch "pixelisation" pass, normal maps kept for in-engine dynamic lighting — Thomas Vasseur, "Art Design Deep Dive: Using a 3D pipeline for 2D animation in Dead Cells", Gamasutra 2018 (article now 403/moved) [med, from memory] |
| **Minecraft** | Native res, perspective | **16×16 texels per 1 m block** (needs mipmaps at distance or it shimmers — the in-game "Mipmap Levels" option exists for exactly this) | Steve 1.8 m ≈ 29 texels | https://minecraft.wiki/w/Resource_pack [high] |
| **Octopath Traveler (HD-2D)** | Native res, UE4, perspective + tilt-shift DoF | "SNES-style character sprites and textures with polygonal environments"; point light so sprites cast shadows | ~32 px-class sprites | https://en.wikipedia.org/wiki/HD-2D , https://en.wikipedia.org/wiki/Octopath_Traveler [high for description, low for exact sprite px] |
| **Fez** | 1280×720 native | World is built from **16×16×16-"trixel" cubes**, pixel art painted per face | Gomez ≈ 16 px | https://en.wikipedia.org/wiki/Fez_(video_game) [high] |
| **Cube World** | Native, perspective | 1 voxel = 1 texel; characters ~ 30–40 voxels tall | — | [low, general knowledge] |
| **Songs of Conquest** | Native; no clean pixel quantum in screenshots (measured) | Pixel-art sprites + "3D environments and volumetric lighting" | — | https://en.wikipedia.org/wiki/Songs_of_Conquest [med] |
| **Backbone / The Last Night** | Native; pixel-art sprites composited into 3D-lit scenes (Unity), no integer quantum in Backbone screenshots (measured) | — | — | [med] |
| **Doom / Quake** | Doom 320×200; Quake 320×200–640×480 | **1 map unit = 1 texel** on walls; standard 64×64 / 128-tall wall textures; player 56 units tall (≈56 texels), 32 wide | — | https://book.leveldesignbook.com/process/blockout/metrics [med]; doomwiki blocked by Cloudflare |
| **Ion Fury** | 1080p Build engine, hand-drawn 8-bit-palette textures | Roughly 64–128 texels per wall tile, so ~2–3× denser than Doom | — | [low] |

Take-aways [high]:
- Every pixel-perfect 2D reference sits at an internal height of **180–288 texels** (Godot's docs: "Most pixel art
  games use viewport sizes between 256×224 and 640×480. 640×360 is a good baseline, as it scales to 1280×720,
  1920×1080, 2560×1440, and 3840×2160 without any black bars when using integer scaling" —
  https://docs.godotengine.org/en/stable/tutorials/rendering/multiple_resolutions.html). t3ssel8r's 3D pixel-art
  engine renders at **640×360** (https://www.youtube.com/watch?v=NutO1jzuVXU description).
- Hero-to-tile ratio: Downwell 1.5 tiles, Celeste 2 tiles, HLD ~1.5–2 tiles (16-px tiles). "8×8 tiles with a
  12–16 px hero" is the Downwell/Celeste register.
- The "3D games with pixel textures" (Minecraft, Fez, Cube World) all use **16 texels per world unit**; 8 is rare in 3D
  because a lit 3D surface at 8 texels/unit reads as noise unless the palette is tiny (Downwell gets away with 8 px
  because it is 1-bit dither on black).

### Recommendation

Numbers assume the hero is the read-focus and the game runs on a ~6.1" phone at DPR 3 (1080×2340 physical) and
on desktop 1080p.

- **Environment texel = 4 physical pixels** on phones and at 1080p (HLD/Dead Cells density). On a 460-ppi phone
  that is 0.22 mm per texel; an 8-texel tile is 1.8 mm; a 16-texel hero is 3.5 mm — roughly Downwell-on-mobile,
  which is the lower bound of readable. [high]
- **Integer scale k** = `max(1, floor(min(devW, devH) / 270))` → k=4 on 1080-wide phones and 1080p desktops,
  k=5 at 1440p, k=8 at 4K, k=2–3 on old 720p phones. Internal target = `floor(dev / k)` per axis (Godot's
  "expand" aspect: fixed texel size, variable field of view, **no letterbox**). [high]
  - 16:9 desktop 1920×1080 → **480×270**.
  - Phone portrait 1080×2340 → **270×585**; iPhone 1170×2532 → 292×633.
  - Phone landscape → 585×270.
- **Tile 8×8 texels** (as asked) is fine *for the environment* if the palette is ≤ 8 colours and tiles are mostly
  flat/dithered like Downwell; use **16×16 for hero-scale props** (doors, chests) so they carry detail. If the
  dungeon is rendered as 3D geometry (walls with tops), prefer **16 texels per world unit** with 8-texel "half
  blocks", because 30°-pitch tops halve the vertical texel count (see §2). [med]
- **Hero height**: 16 texels if the hero goes through the low-res target as ordinary pixel art; but a watercolour
  sprite needs **≥ 32–48 px** to keep any brushwork and a readable 2-px ink line → see §4 for the two-density
  scheme. [high]
- **Visible field**: portrait 270×585 at 8-px tiles = ~34×73 tiles; Downwell shows ~20 tiles across the well and
  ~35 tall. Keep **~24–32 tiles horizontally visible** in portrait (crop the sides of the dungeon or design corridors
  ≤ 24 tiles wide); landscape shows 60×34 tiles, which is too much context for an auto-player — zoom the ortho
  camera so the hero-to-screen-height ratio stays ~1:16 (hero 16 tx / 270) regardless of aspect. [med]

---

## 2. Pixel-perfect camera follow in 3D

### The t3ssel8r technique (primary sources)
- Sub-pixel camera: "we render our scene at a low pixel art resolution (640x360) and upscale the result to the
  screen resolution … At render-time, the camera is snapped to the nearest 'pixel', and when blitting to screen, the
  snap offset is corrected so that the camera is permitted to smoothly glide over the pixels of the screen at the
  display's native resolution. Some extra projections need to be computed in order to ensure that this operation works
  precisely even when the camera is rotated 45 degrees." — https://www.youtube.com/watch?v=NutO1jzuVXU [high]
- Camera angle: "a 30 degree pitch isometric camera that snaps the yaw to 45 degree increments is used. This can
  perfectly reproduce the 2-pixels-across-1-pixel-down look of rectangular pixel art blocks, while still allowing smooth
  camera motion between snapping regions." — https://www.youtube.com/watch?v=ij555s4mAuI [high]
- Parallax/depth: blend a perspective camera for far geometry into the orthographic camera for near geometry;
  "A depth-based fog helps tie the scene together." — https://www.youtube.com/watch?v=cCUCMBmc9yQ [high]
- Why 30°: `sin 30° = 0.5`, so a horizontal face 16 texels deep projects to exactly 8 px (2:1 dimetric); vertical
  faces project by `cos 30° = 0.866`, so their textures must be authored/UV-scaled 1/0.866 taller (or use non-square
  texels on vertical faces) to land on integer pixels. Any other pitch, and any perspective camera, produces
  non-integer texel→pixel ratios that **shimmer** (texel edges alternate between 1 and 2 pixels as the camera
  moves). [high, derived]

### Unity Pixel Perfect Camera rules (for the checklist)
From https://docs.unity3d.com/Packages/com.unity.2d.pixel-perfect@5.0/manual/index.html [high]:
Assets Pixels Per Unit must match every sprite; a Reference Resolution the art is designed for; "Upscale Render
Texture" renders "to a temporary texture set as close as possible to the Reference Resolution, while maintaining the
full screen aspect ratio", then upscales; "Pixel Snapping … snap Sprite Renderers to a grid in world space at
render-time … prevents subpixel movement"; Crop Frame letterboxes to the reference; Stretch Fill allows non-integer.

### Avoiding shimmer / texture crawl [high]
1. Orthographic camera, pitch from {0°, 30°}, yaw from multiples of 45° (or 90° for a Downwell side view).
2. Camera translation snapped to the texel grid **in camera space** (right/up basis), not world space.
3. `NearestFilter` for mag; for min use `NearestFilter` with `generateMipmaps=false` when the texel→pixel ratio is
   exactly 1 (ortho + snapped camera guarantees this), otherwise `NearestMipmapNearestFilter` — MDN: for 2D resources
   that are never zoomed out "don't pay the 30% memory surcharge for mipmaps"
   (https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices).
4. Geometry vertices on the texel grid (block sizes multiples of 1/16 unit); UVs at texel centres for atlases
   (add a 1-texel gutter or use `THREE.RepeatWrapping` per tile only if each tile is its own texture).
5. Sprites: snap billboard position to the grid; billboards perpendicular to the view direction (not vertical) so
   1 sprite texel = 1 target pixel under the 30° pitch.
6. Post effects that sample neighbours (outline, dither) operate in the low-res target, never after the upscale.
7. If you must show rotated/scaled pixel art, use the pixel-art AA sampler (Cole Cecil,
   https://colececil.io/blog/2017/scaling-pixel-art-without-destroying-it/ ; t3ssel8r's generalisation using
   `fwidth()` for arbitrary distortion, https://www.youtube.com/watch?v=d6tp43wZqps):
   ```glsl
   // texture set to LinearFilter; tc = uv * texSize (texel coords)
   vec2 box = clamp(fwidth(tc), 1e-5, 1.0);          // pixel footprint in texels (t3ssel8r); Cecil uses a uniform
   vec2 f   = fract(tc);
   vec2 interp = clamp(f / box, 0.0, 0.5) + clamp((f - 1.0) / box + 0.5, 0.0, 0.5);
   vec4 c = texture(map, (floor(tc) + interp) / texSize);
   ```
   Nearest-sharpness with exactly one pixel of anti-aliasing at texel boundaries; three.js `Texture` cannot do this
   natively, so it needs a `ShaderMaterial`/`onBeforeCompile` patch. Only needed for sprites that rotate or scale.

### three.js implementation (what exists, what to write)
- **`RenderPixelatedPass`** (examples/jsm/postprocessing) renders to a half-float target of size
  `(width/pixelSize)|0` with `minFilter = magFilter = NearestFilter` and adds depth/normal edge outlines
  (https://raw.githubusercontent.com/mrdoob/three.js/dev/examples/jsm/postprocessing/RenderPixelatedPass.js). The
  example https://threejs.org/examples/webgl_postprocessing_pixel.html uses an **OrthographicCamera pitched 30°**
  (`camera.position.y = 2 * tan(π/6)`, `z = 2`) and a `pixelAlignFrustum()` helper that projects the camera position
  onto its right/up axes, takes the fractional texel remainder and **shifts `camera.left/right/top/bottom` by it**
  — i.e. the t3ssel8r snap done by moving the frustum instead of the camera. It also snaps object positions
  (`pixelAlignedObjects`) and rotations. Use it as the reference implementation, but note it rebuilds the target from
  screen size / pixelSize rather than choosing an integer k; and it has no sub-texel blit offset (the frustum shift
  already does the equivalent). [high]
- Minimal custom pipeline (recommended over EffectComposer on phones — one target, one blit):
  ```js
  const renderer = new THREE.WebGLRenderer({ antialias:false, powerPreference:'high-performance' });
  renderer.setPixelRatio(1);                          // manage DPR yourself
  function resize() {
    const dpr = window.devicePixelRatio, cw = innerWidth, ch = innerHeight;
    const devW = Math.round(cw*dpr), devH = Math.round(ch*dpr);
    const k = Math.max(1, Math.floor(Math.min(devW, devH) / 270));
    const iw = Math.floor(devW / k), ih = Math.floor(devH / k);
    renderer.setSize(devW, devH, false);               // canvas backing store = device px
    renderer.domElement.style.width = cw+'px'; renderer.domElement.style.height = ch+'px';
    rt.setSize(iw + 2, ih + 2);                        // +2 texel overscan for sub-texel scroll
    cam.left = -(iw+2)/2 * texelWorld; cam.right = -cam.left; cam.top = (ih+2)/2 * texelWorld; cam.bottom = -cam.top;
  }
  const rt = new THREE.WebGLRenderTarget(1, 1, { minFilter: THREE.NearestFilter, magFilter: THREE.NearestFilter,
                                                 generateMipmaps: false, depthBuffer: true });
  // per frame: snap camera in its own right/up basis
  const right = new THREE.Vector3(1,0,0).applyQuaternion(cam.quaternion), up = new THREE.Vector3(0,1,0).applyQuaternion(cam.quaternion);
  const pr = target.dot(right)/texelWorld, pu = target.dot(up)/texelWorld;
  const fx = pr - Math.round(pr), fy = pu - Math.round(pu);          // sub-texel remainder, in texels
  cam.position.copy(target).addScaledVector(right, -fx*texelWorld).addScaledVector(up, -fy*texelWorld);
  blitMat.uniforms.uOffset.value.set(fx/(iw+2), fy/(ih+2));           // shift UVs of the fullscreen quad
  blitMat.uniforms.uScale.value.set(iw/(iw+2), ih/(ih+2));            // crop the overscan
  renderer.setRenderTarget(rt); renderer.render(scene, cam);
  renderer.setRenderTarget(null); renderer.render(blitScene, blitCam); // quad with rt.texture, NearestFilter
  ```
  Because the canvas backing store is exactly `k ×` the target, the nearest upscale is integer everywhere and the
  offset makes the image glide at device-pixel granularity (the "smooth scroll" half of t3ssel8r's trick). Snap
  sprite/mesh positions to `texelWorld` in the same right/up basis before rendering (three.js example does this).
  `renderer.setSize(w, h, false)` "prevents any style changes to the output canvas"; `setPixelRatio` "resizes the
  canvas if necessary" — https://threejs.org/docs/pages/WebGLRenderer.html.md [high]
- **Letterboxing** is unnecessary with the expand-aspect scheme; if you insist on a fixed 270-texel width in portrait,
  use `renderer.setViewport/setScissor` on the final blit and clear the bars to the palette's darkest colour.
- DPR notes: never `setPixelRatio(devicePixelRatio)` here — it would make three.js pick the backing size; you want
  to own it so `k` stays integer. Re-run `resize()` on `visualViewport` resize (iOS URL bar) and on orientation
  change; recompute `k` — a k change is a visible zoom, so hysteresis it (only change when the previous k would
  drop below 3.5 or exceed 4.5 texels/px). [med]

---

## 3. Lighting and palette

What the good pixel-in-3D games do [high unless noted]:
- **Downwell**: unlit, 4 colours, entire mood is palette swap + dither texture. Palette swap per run/level is the
  cheapest "biome" system there is.
- **t3ssel8r**: real directional light + shadows, but shading is quantised: "shading the shadowed regions using a
  simple monochromatic Lambertian BRDF, resulting in a soft fake ambient lighting effect" (NutO1jzuVXU), later
  "light-aware edge coloration" (https://www.youtube.com/watch?v=ZsMHY4LDyRE), night scenes with a small number of
  local lights (VEP4INri_1U), depth fog for parallax (cCUCMBmc9yQ).
- **HD-2D**: full PBR-ish 3D lighting, "a point light source is placed into the scene to make characters and objects
  cast shadows", bloom, DoF, fog, particles (https://en.wikipedia.org/wiki/HD-2D). Looks rich, but not lo-fi: the
  sprites stay unlit-ish while the world gets soft light, which is exactly the two-register look (§4).
- **Return of the Obra Dinn** (Lucas Pope): 1-bit ordered dithering at 800×450 with a dither pattern stabilised
  against camera motion (screen-anchored dither "swims" when the view moves). [med: TIGSource devlog is behind
  Cloudflare; https://forums.tigsource.com/index.php?topic=40832.msg1363742]
- **Dead Cells**: sprite normal maps + dynamic point lights on 2D sprites, then no palette lock — which is why its
  screenshots do not have a clean pixel quantum. [med]

Recommended stack for a phone dungeon (cheap → expensive; stop where it looks good):
1. **Unlit + baked AO/vertex tint** (`MeshBasicMaterial` with `vertexColors`) for tiles; bake a 3-step darkness into
   vertex colour by distance from the walkable floor. One draw call per chunk, zero lighting cost. [high]
2. **Quantised directional light**: `MeshToonMaterial` with a 3–4-step `gradientMap` (NearestFilter) — three.js's
   built-in toon banding; or a custom `ShaderMaterial` with `floor(NdotL * 3)/3`. Key light fixed at top-left to match
   the sprite prompt (§4). [high]
3. **Palette post-process in the blit shader**: for palettes ≤ 32 colours, brute-force nearest colour in the fragment
   shader is cheaper than a 3D LUT texture fetch chain on mobile and trivially supports **per-level palette swaps**
   (upload a new `vec3[32]` uniform). Do the search in a perceptual space (Oklab) or the ramps will jump. Add
   **ordered dithering** (4×4 Bayer, threshold amplitude ≈ half a palette step) *before* quantisation, indexed by
   **world-texel coordinate** `(floor(worldPos.xz / texelWorld)) & 3` rather than screen coordinate, so the pattern
   sticks to the dungeon and does not crawl while the camera scrolls (the Obra Dinn problem, solved for free by the
   snapped camera). [high]
4. **Depth fog**: `scene.fog = new THREE.Fog(paletteDark, near, far)` applied in-material, then quantised by step 3 →
   fog becomes 2–3 banded darkness rings, which reads as "dungeon" rather than "smoke". For a 30°-pitch camera use
   height/distance-from-hero fog in the shader instead of view-depth (ortho depth is nearly constant). [med]
5. **Torches**: 2–4 `PointLight`s max on mobile with `MeshLambertMaterial`/toon (each light adds per-fragment
   work to every lit material). Cheaper and more on-style: additive **light sprites** (a radial gradient quad,
   `AdditiveBlending`, quantised afterward by step 3) plus an emissive tile variant. Downwell has no lights at all;
   Octopath uses one point light per scene. [high]
6. **Drop shadows**: a 1-texel-thick dark ellipse quad under each sprite (`depthWrite:false`, rendered before
   sprites), same palette-darkest colour; matches Octopath's "characters cast shadows" cue at ~zero cost. Real
   shadow maps on mobile WebGL2 are affordable only for one directional light at 512–1024² and make aliasing that
   the quantiser turns into crawling stair-steps — skip. [high]

Palette sources: https://lospec.com/palette-list (search 4-, 8-, 16-colour; e.g. "Downwell"-like 1-bit+accent,
"Ammo-8", "Slso8" for dungeon ramps). Store palettes as `Float32Array` uniforms; a palette swap is a uniform update.

---

## 4. Mixing painted watercolour sprites with a pixel environment

### Precedents and what they teach [high unless noted]
- **Paper Mario** (N64, 2000): flat 2D characters in a low-poly 3D world; the mismatch is *framed* as paper cut-outs,
  so it reads as intent. **Octopath / HD-2D**: pixel sprites ("SNES-style") in 3D, sprites deliberately *lower* res
  than the world's lit surfaces; the diorama framing (tilt-shift) sells the two registers.
- **Don't Starve**: hand-drawn 2D characters as billboards in a 3D-ish world — but the *world* is drawn in the same
  hand (same ink, same paper texture), so there is no clash. https://en.wikipedia.org/wiki/Don%27t_Starve
- **aarthificial, "Next-Gen Pixel Art"** (https://www.youtube.com/watch?v=jguyR4yJb1M): the hybrid of "traditional
  upscaling and normal rendering" — render at native res, keep textures pixel-crisp via the AA sampler, allow sub-texel
  positions/rotation. Shows that **mixed pixel densities are acceptable when the *outline weight and shading
  language* match**. [med: from description]
- The pixel-art community term for mismatched densities is "mixels" and it is the #1 flagged sin in feedback threads;
  the objection is specifically about **different pixel sizes in the same visual layer** (a 1-px-line UI over
  4-px-texel art), less about a deliberately different *medium* (paper cutout vs. set). [med]
- **No shipped, well-rated game puts hi-res painterly characters on an 8-px environment.** Closest are Octopath (pixel
  characters on hi-res world, i.e. the *inverse*) and 90s fighting games (lo-res sprites on hi-res painted
  backgrounds, also the inverse). Treat this as a design risk to prototype, not a solved look. [high]

### Style-clash risks
1. Outline weight: a 1-device-px ink line next to a 4-device-px texel looks like a sticker on a photo.
2. Colour: watercolour has hundreds of hues; a 4–8 colour environment makes the hero look pasted from a different game.
3. Lighting: painted sprites carry their own baked light direction; if the environment key light differs the sprite
   floats.
4. Contact: without a shadow/contact the sprite hovers (Octopath's point-light shadow exists to fix precisely this).
5. Motion: sprites at native res glide sub-texel while the world snaps → the hero looks "on glass".

### Options
- **A. Everything through the low-res target** (hero 16–24 texels). Cheapest, unified, but destroys the watercolour;
  you would be shipping AI-generated pixel art, which is the weak case (§5). Only viable if the hero is ≥ 48 texels
  (→ k=2, env texel 2 device px, which stops looking lo-fi).
- **B. Environment low-res, sprites composited at native res** (Paper-Mario register). Preserves the art; guaranteed
  "mixels" and the "on glass" problem; needs a second depth-aware pass.
- **C. Two densities in one target (recommended)**: render into a target whose pixel is **k/2 device pixels** (e.g.
  540×1170 on a 1080×2340 phone, still ¼ the fragments of native). Environment textures are authored at 8/16 texels
  but mapped so **1 env texel = 2 target pixels** (scale the atlas UV or just use 2× nearest-upscaled atlases); sprites
  are mapped **1 sprite texel = 1 target pixel**, pre-downscaled to a fixed sprite height (e.g. hero 48 sprite-texels
  = 24 env-texels = 3 tiles of 8; monsters 32–64). Camera snaps in *env-texel* (2-target-px) increments. Result: the
  world is exactly as lo-fi as before, the hero has 2× the pixel density (enough for a 2-px ink line and visible wash
  granularity), and both share one nearest-upscale, one depth buffer, one palette pass, one dither grid. Cost: a
  4× larger target than A (≈630k px on a phone — fine) and 2× env texel work (nothing, it is nearest sampling). [med]

### Mitigations that apply to B and C
- **Outline**: pre-process each sprite so its ink outline is **≥ 1 env texel (2 target px)** thick — dilate the alpha
  and darken; also give tiles a 1-texel dark rim so the two share the "everything has a dark edge" rule (three.js
  `RenderPixelatedPass` depth/normal edges do this for geometry).
- **Colour unification**: run sprites through the *same* palette post-process but **blended at ~40–60%** with the
  original (a "tint LUT", not a hard quantise), and pre-quantise the sprites offline to ~24–32 colours drawn from
  the dungeon palette's hue families (Pillow `quantize(method=MEDIANCUT)` seeded per level). Both per-level palette
  swaps then also swap the hero's ramps a little, which is what unifies biomes.
- **Consistent key light**: fix the environment key light top-left; generate/regenerate sprites with "lit from upper
  left, soft cast shadow to lower right" in the prompt; flip sprites only horizontally when they face left and accept
  the light flip (Octopath and every 2D game does).
- **Contact shadow**: 1-env-texel-high ellipse under every sprite; monsters get the same. Non-negotiable.
- **Snap sprites** to the env-texel grid too (positions in multiples of 2 target px), so hero and floor move together.
- **Motion**: animate sprites at 8–12 fps (Dead Cells and t3ssel8r both quantise animation timing) so the hero does
  not move "smoother than the world".
- **Framing**: lean into "paper cut-out on a pixel diorama": 1-px paper-white rim inside the ink line, slight
  vertical bob, and shadows that are obviously stamped. If the register mismatch is *legibly deliberate* (Paper
  Mario), players read it as style; if it is half-hidden, they read it as a bug.

Kill-test: build C and A in the same afternoon (same scene, toggle), screenshot both at phone size, ask 5 people
which is "one game". If A wins, the answer is to invest in §5's downscale pipeline and a 48-texel hero.

---

## 5. Asset production

### Hand-made tiles worth knowing (all CC0) [high]
- **Kenney Micro Roguelike** — **8×8**, 320 files, CC0: https://kenney.nl/assets/micro-roguelike (the only good 8×8
  dungeon set with characters; 1-bit style, so it takes any palette).
- **Kenney Tiny Dungeon** — 16×16, 130 files, CC0: https://kenney.nl/assets/tiny-dungeon
- **Kenney 1-Bit Pack** — 16×16, 1078 files, CC0: https://kenney.nl/assets/1-bit-pack (1-bit → trivially
  recoloured through the palette pass; huge coverage).
- **0x72 DungeonTileset II** — 16×16 dungeon + animated monsters, CC0: https://0x72.itch.io/dungeontileset-ii
  (the de-facto roguelike prototyping set; note its characters are 16×16/16×32).
- Also: Kenney Monochrome RPG, Tiny Town/Battle (same 16-px family, consistent outline weight).

### Procedural tiles [med]
- Downwell's walls are essentially a **dither noise texture inside an outline**; at 8×8 and ≤ 4 colours you can
  generate the whole tileset in code: fill = value noise thresholded to 2 shades, 1-texel rim on exposed edges
  (autotile by 4-neighbour mask → 16 variants, or 8-neighbour "blob" → 47), 2–3 random speckle variants per tile to
  kill repetition. This also guarantees seamlessness, which AI does not.
- Use the sim's dungeon grid to drive **autotiling bitmasks** in Rust and emit instance data (tile id + variant)
  so the JS side only uploads an `InstancedBufferAttribute`.

### AI generation reality [high]
- General image models (SDXL/Flux/GPT-image) produce "pixel-looking" images whose pixel grid is **irregular
  (pitch varies 6–12 px across the image, off by fractions), with anti-aliased edges and hundreds of colours** —
  Retro Diffusion's own pitch: "Most AI art tools produce images that *look* pixelated. Retro Diffusion produces
  actual pixel art: true low-resolution grids, deliberate palettes, and clean single-pixel outlines … 16–384px"
  (https://www.retrodiffusion.ai/ ; paid, ~$0.015–0.18/image, has tileset/animation/reference-image modes).
- **PixelLab** (https://www.pixellab.ai/) — pixel-art-specific generator with tilesets, 4/8-directional characters,
  skeleton animation, inpainting, "style-consistent generation"; Aseprite plugin. Both tools are the pragmatic route
  for *monster* pixel sprites if you ever go with option A.
- **Downscale-and-quantise pipeline** for generic models (works for single props/decals, poor for tilesets):
  1. Generate at 8–16× the target (e.g. 1024² for a 64² sprite), prompt "flat colours, thick black outline, no
     gradients, no anti-aliasing, solid background".
  2. **Detect the pitch** — autocorrelation / run-length histogram of the image (the same trick used to measure the
     Steam screenshots above); resample so pitch = 1 px using **box/area** downscale (not nearest, which picks
     random sub-pixel colours), then optionally nearest to snap.
  3. Quantise to the level palette (Pillow `Image.quantize(colors=N, palette=…, dither=NONE)`), then a 1-px erosion of
     the outline and manual clean-up in Aseprite (scripts: https://github.com/aseprite/Aseprite-Script-Examples).
  4. What fails: seamless tiling (edges never match), consistent texel density across a set, 8×8 anything (too few
     pixels for the model to "mean" them), animation frame consistency. What works: 32–64 px one-off props,
     portraits, decals, palette exploration.
- For the **watercolour hero/monsters** (option C) the pipeline is simpler: generate at 1024², remove background,
  box-downscale to the fixed sprite height (48/64 px), dilate+darken the outline to ≥ 2 px, quantise to 24–32
  colours, export as an atlas with 1-px gutters. Do the downscale offline, never in the shader (mipmaps of a painted
  sprite go muddy).

---

## 6. Performance on phone WebGL2

Budgets [high unless noted]:
- Frame: 16.7 ms at 60 Hz; target **≤ 8 ms GPU + ≤ 4 ms JS/WASM** so mid-range Androids hold 60 and low-end hold a
  clean 30 (use `requestAnimationFrame` with a fixed-step sim and render-frame skipping rather than a 30-fps cap).
- Draw calls: mobile WebGL is CPU-bound on draw calls long before triangles; aim **< 50 draw calls/frame**, hard
  ceiling ~150. MDN: "If you have 1000 sprites to paint, try to do it as a single drawArrays() or drawElements() call"
  and "texture atlasing lets you combine more draw calls into fewer, bigger batches"
  (https://developer.mozilla.org/en-US/docs/Web/API/WebGL_API/WebGL_best_practices).
- Fill rate is the phone bottleneck; **the low-res target is the single biggest win**: 270×585 = 158k px vs
  1080×2340 = 2.5M px (16×). Option C's 540×1170 is still 4× cheaper than native. Keep the final blit a single
  fullscreen quad (no EffectComposer chain — each pass is another full-screen fill and a target swap).
- Textures: one 1024² (or 2048²) RGBA atlas for tiles, one for sprites; `NearestFilter`, `generateMipmaps=false`
  (three.js `Texture.generateMipmaps` default is `true` — turn it off; https://threejs.org/docs/pages/Texture.html.md).
  Prefer `alphaTest`-style cut-out (`alphaTest: 0.5`, opaque queue) for tiles/sprites with hard edges, and reserve
  blending for the few light/shadow quads — overdraw with blending is what kills Adreno/Mali.
- Geometry: **InstancedMesh** ("reduce the number of draw calls" — https://threejs.org/docs/pages/InstancedMesh.html.md)
  with a per-instance `InstancedBufferAttribute` for atlas UV offset + tint → **1 draw call for all visible tiles,
  1 for all sprites, 1 for shadows**. For static dungeon chunks, merging into a `BufferGeometry` per 16×16-tile chunk
  is equally fast and lets frustum culling drop whole chunks; `BatchedMesh` is the newer alternative when chunks have
  different geometry. Avoid `THREE.Sprite` for monsters (one draw call each; fine for 1–3 UI elements).
- Lights: every extra `PointLight` re-compiles/costs in every lit material; cap at 2–4 or use additive light quads.
  `MeshBasicMaterial` everywhere + palette pass is the safe mobile baseline.
- Precision: MDN warns `highp` in fragment shaders "will prevent your content from working on some older mobile
  hardware" while `mediump` "often results in corrupted rendering" — the palette-search and dither shaders must
  be written to be `mediump`-safe (small integer maths, no large world coordinates in the fragment stage; pass the
  world-texel index from the vertex shader). [med]
- Creation flags: `antialias:false` (MSAA on a nearest-upscaled target is pointless and expensive),
  `powerPreference:'high-performance'`, `alpha:false`, `stencil:false`, `depth:true` only on the low-res target.
- Memory: keep total texture memory < 64 MB; a 2048² RGBA is 16 MB without mipmaps. iOS Safari kills tabs that
  exceed ~1–1.5 GB total, but WebGL contexts get lost far earlier on memory pressure — handle `webglcontextlost`.
- Measure with `renderer.info.render.calls/triangles` in a debug overlay and with Chrome's Android remote
  profiling; an `EXT_disjoint_timer_query_webgl2` GPU timer is not available on iOS.

---

## Recommended pipeline (one page)

**Numbers**
- Integer scale `k = max(1, floor(min(devW, devH)/270))`; **env texel = k device px** (k=4 on 1080-wide phones and
  1080p desktops). Internal target = `floor(dev/k)` per axis, +2 texel overscan; no letterbox (expand aspect).
  Phone portrait → 270×585 (+2); desktop → 480×270 (+2).
- Option C two-density target: **actual target = 2× that** (540×1170 / 960×540): env texel = 2 target px, sprite
  texel = 1 target px. If you drop C for A, use the 1× target.
- **Tile 8×8 env texels** for floors/walls/dither; 16×16 for props; 16 env texels per world unit (a floor tile = 0.5
  unit). Hero **48 sprite-texels = 24 env-texels = 3 tiles tall**; monsters 32–64 sprite-texels. Hero occupies
  ~1/12 of portrait screen height; 24–32 tiles visible across in portrait.
- **Camera**: `OrthographicCamera`, pitch **30°** (dimetric 2:1) if the dungeon has 3D walls; **0° side/top-down**
  if it is Downwell-flat. Yaw locked to 45° multiples. Camera position snapped to the env-texel grid in the camera's
  right/up basis; sub-texel remainder applied as a UV offset in the final blit (t3ssel8r / three.js
  `pixelAlignFrustum`). Follow target = hero position smoothed with a critically-damped spring (t3ssel8r's
  second-order dynamics, https://www.youtube.com/watch?v=KPoeNZZ6H4s), then snapped.
- **Filtering**: `NearestFilter` mag+min, `generateMipmaps:false`, `antialias:false`, atlases with 1-px gutters,
  UVs at texel centres, vertices on the 1/16-unit grid. Vertical faces at 30° get UV-scaled by 1/cos 30°.
- **Palette**: environment quantised to a 4–8 colour per-level palette in the blit shader (nearest colour in Oklab,
  4×4 Bayer dither indexed by world-texel), palette = uniform array → biome swap is a uniform upload. Sprites pass
  through the same shader at 40–60% blend, and are pre-quantised offline to 24–32 colours seeded from the palette.
- **Lighting**: `MeshBasicMaterial` + baked vertex darkness (+ optional `MeshToonMaterial` 3-step key light from
  top-left). Torches = additive radial quads, max 2 real `PointLight`s. Banded distance-from-hero fog in-shader.
  1-env-texel drop-shadow ellipse under every sprite.
- **Sprite compositing**: option C (two densities, one target, one depth buffer); billboards face the camera, snapped
  to env-texel grid, animated at 8–12 fps, ink outline dilated to ≥ 2 target px, paper-white inner rim. Build A as a
  toggle for the kill-test.
- **Assets**: start from Kenney Micro Roguelike (8×8) / 1-Bit Pack (16×16) recoloured through the palette pass;
  procedural dither fill + autotile masks computed in Rust; AI only for hero/monster watercolour sprites (box
  downscale → outline dilate → quantise) and one-off 32–64 px props.

**three.js primitives**
`WebGLRenderer({antialias:false, powerPreference:'high-performance', alpha:false, stencil:false})`,
`renderer.setPixelRatio(1)` + `setSize(devW, devH, false)`, `WebGLRenderTarget` (Nearest, no mipmaps, depth),
`OrthographicCamera`, `InstancedMesh` + `InstancedBufferAttribute` (tiles, sprites, shadows), `BufferGeometryUtils.
mergeGeometries` or `BatchedMesh` for static chunks, `MeshBasicMaterial` / `MeshToonMaterial` (+`gradientMap`),
`ShaderMaterial` for the fullscreen blit (palette + dither + UV offset), `Texture` with `NearestFilter`/
`generateMipmaps=false`/`colorSpace = SRGBColorSpace`, `Fog` or in-shader fog, `renderer.info` for budgets.
Reference code: `examples/webgl_postprocessing_pixel.html` (`pixelAlignFrustum`) and
`examples/jsm/postprocessing/RenderPixelatedPass.js` (target sizing, edge outline shader).
