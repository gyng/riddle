// Two generated canvas atlases, one per density:
//   env    (1 texel = 1 env texel = k device px): tiles, items, overlays, glyphs, shadow, font
//   sprite (1 texel = 1 sprite texel = k/2 device px): entity silhouettes
// Every id gets a procedural fallback drawn on first request (lazy shelf packing), so the viewer is
// fully usable with no art. An external atlas (`atlas.json` {frames:{id:{x,y,w,h}}} + `atlas.png`)
// overrides slots by id:  `<biome>_<tile>` → tile, entity kind → entity, `gas`/`fire`(`_0`/`_1`,
// or `ov_` prefix) → overlay, `<biome>_shrine_0/1`, `<biome>_vault[_open]`, `<biome>_nest_0/1` → prop (Cut 5
// §4 situations, env density in the tiles layer), `item_<kind>` (or a bare unknown id) → item.
// Cut 16 §3: a biome in `TILE_ALIAS` (the Burrows → the Warrens) has no tile art of its own: each of its alias's loaded tile
// frames is copied to `tile:<biome>_…`, every pixel moved from the alias's ramp index to the same index of the biome's ramp.
// Art pass (2026-09-24, art/ui/ART_GAP.md): register 3 — `<biome>_env_<name>` (16×16 ramp-indexed tiles, decals and props;
// 16×24 torch) and `env_<name>` (hue assets: blood, torch frames, banner) load at their native size into `tile:<id>` (two
// texels per env texel: a tile is still an 8×8 world quad). They are optional: `envTile`/`hue` return undefined without
// them and the viewer draws the 8×8 register as before. `wallTop(biome, mask)` derives an edge-rimmed wall top per
// 4-neighbour mask (N 1, E 2, S 4, W 8 = the side that meets open floor) on first request.
import { heroBase } from "./look";
/** gfx round 1: a kind drawn with another's sprite (the fake engine's summoned `blade` is the core's `spectral_blade`; its fallback read
 *  as a magenta placeholder jug) */
const KIND_ALIAS: Record<string, string> = { blade: "spectral_blade" };
import * as THREE from "three";
import { css, ENTITY_BOX, ENTITY_COLOURS, ENTITY_SIZE, PALETTES, paletteFor, setPalettes, SPRITE_SCALE, TILE_ALIAS, TILE_IDS, type Rgb } from "./palette";
import { FONT_CELL_H, FONT_CELL_W, FONT_H, FONT_W, glyphBits } from "./font";

export type Rect = { x: number; y: number; w: number; h: number };
type AtlasJson = {
  image?: string;
  frames?: Record<string, Rect>;
  meta?: { palettes?: Record<string, string[]>; sprites?: Record<string, { texel_h?: number }> };
};
export type Slot = Rect & { u0: number; v0: number; u1: number; v1: number };

class Sheet {
  readonly canvas: HTMLCanvasElement;
  readonly ctx: CanvasRenderingContext2D;
  readonly texture: THREE.CanvasTexture;
  private slots = new Map<string, Slot>();
  private shelfY = 1;
  private shelfX = 1;
  private shelfH = 0;

  readonly w: number;
  readonly h: number;
  /** juice pass 2: bumped on every slot drawn (normals.ts re-derives the sprite normals when it moves) */
  version = 0;

  constructor(w: number, h: number) {
    this.w = w;
    this.h = h;
    this.canvas = document.createElement("canvas");
    this.canvas.width = w;
    this.canvas.height = h;
    this.ctx = this.canvas.getContext("2d", { willReadFrequently: false })!;
    this.ctx.imageSmoothingEnabled = false;
    this.texture = new THREE.CanvasTexture(this.canvas);
    this.texture.minFilter = THREE.NearestFilter;
    this.texture.magFilter = THREE.NearestFilter;
    this.texture.generateMipmaps = false;
    this.texture.colorSpace = THREE.NoColorSpace; // raw values; palette pass works in the same space
    this.texture.flipY = true;
    this.texture.premultiplyAlpha = false;
  }

  has(id: string): boolean { return this.slots.has(id); }
  get(id: string): Slot | undefined { return this.slots.get(id); }

  // Allocate (or re-allocate) a w×h slot with a 1-px gutter; returns the rect to draw into.
  alloc(id: string, w: number, h: number): Slot {
    if (this.shelfX + w + 1 > this.w) { this.shelfY += this.shelfH + 1; this.shelfX = 1; this.shelfH = 0; }
    if (this.shelfY + h + 1 > this.h) throw new Error(`atlas full (${id})`);
    const x = this.shelfX, y = this.shelfY;
    this.shelfX += w + 1;
    this.shelfH = Math.max(this.shelfH, h);
    // UVs at texel edges; flipY=true means v=1 is canvas row 0.
    const slot: Slot = { x, y, w, h, u0: x / this.w, v0: 1 - (y + h) / this.h, u1: (x + w) / this.w, v1: 1 - y / this.h };
    this.slots.set(id, slot);
    this.version++;
    this.ctx.clearRect(x, y, w, h);
    this.texture.needsUpdate = true;
    return slot;
  }
}

export function itemCategory(kind: string): string {
  if (/dagger|sword|axe|bow|weapon/.test(kind)) return "weapon";
  if (/leather|mail|plate|armour|armor/.test(kind)) return "armour";
  if (/heal|strength|speed|invis|poison|caustic|confus|fire|potion/.test(kind)) return "potion";
  if (/teleport|blink|fear|mapping|identify|enchant|darkness|summon|aggravate|scroll/.test(kind)) return "scroll";
  if (/gold/.test(kind)) return "gold";
  return "unknown";
}

// Deterministic tiny hash for speckle patterns.
function hash(x: number, y: number, s: number): number {
  let h = (x * 374761393 + y * 668265263 + s * 2246822519) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
}

export class Atlas {
  readonly env = new Sheet(1024, 1024);   // art pass: 16×16 env tiles × 7 biomes + the 8×8 register
  readonly sprite = new Sheet(512, 512);
  constructor() {
    this.envSlot("shadow:8");
  }

  // ---- env-density ids ----------------------------------------------------------------------
  tile(biome: string, tile: string, alt = false): Slot {
    if (alt) { const a = this.env.get(`tile:${biome}_${tile}_alt`); if (a) return a; }
    return this.envSlot(`tile:${biome}_${tile}`);
  }
  // exact kind slot if the atlas has one, else the atlas's category frame, else a procedural fallback
  item(kind: string): Slot {
    // second art pass: the 16-texel item art (`env_item_<category>`, hue assets) first
    const hd = this.env.get(`tile:env_item_${kind}`) ?? this.env.get(`tile:env_item_${itemCategory(kind)}`);
    if (hd) return hd;
    const exact = this.env.get(`item:${kind}`);
    if (exact) return exact;
    const cat = this.env.get(`item:${itemCategory(kind)}`);
    return cat ?? this.envSlot(`item:${kind}`);
  }
  overlay(kind: string, frame: number): Slot { return this.envSlot(`ov:${kind}_${frame}`); }
  glyph(ch: string): Slot { return this.envSlot(`glyph:${ch}`); }
  font(ch: string): Slot { return this.envSlot(`font:${ch.toUpperCase()}`); }
  shadow(width: number, ally = false): Slot { return this.envSlot(`${ally ? "ring" : "shadow"}:${Math.max(6, Math.round(width))}`); }
  dot(): Slot { return this.envSlot("dot:2"); }
  // Cut 8A: a 2×2 flat colour (`#rrggbb`) for the fight frame's hp bars; the hud layer stretches it to any texel size
  solid(hex: string): Slot { return this.envSlot(`solid:${hex.replace("#", "")}`); }
  // Cut 5 §4 props: shrine (2 frames at 1 Hz), vault / vault_open, nest (frame 0 asleep, 1 woken). Per-biome atlas
  // art when present, else a procedural altar / barred square / mound (`drawProp`).
  prop(biome: string, tile: string, frame: number): Slot {
    const id = tile === "shrine" || tile === "nest" ? `${tile}_${frame & 1}` : tile;
    return this.env.get(`tile:${biome}_env_${id}`) ?? this.env.get(`tile:${biome}_${id}`) ?? this.env.get(`tile:${biome}_${tile}_0`) ?? this.envSlot(`prop:${id}`);
  }
  // ---- register 3 (art pass): 16-texel env art, optional -------------------------------------
  envTile(biome: string, name: string): Slot | undefined { return this.env.get(`tile:${biome}_env_${name}`); }
  hue(name: string): Slot | undefined { return this.env.get(`tile:env_${name}`); }
  /** the wall top with a rim on each side in `mask` (N 1, E 2, S 4, W 8): ramp-0 at the edge, a lit ramp step inside on N/W. */
  wallTop(biome: string, mask: number): Slot | undefined {
    const id = `tile:${biome}_env_wall_top_${mask}`;
    const have = this.env.get(id); if (have) return have;
    const src = this.envTile(biome, "wall_top"); if (!src) return undefined;
    if (!mask) return src;
    const g = this.env, w = src.w, h = src.h;
    const img = g.ctx.getImageData(src.x, src.y, w, h);
    const dst = g.alloc(id, w, h);
    g.ctx.putImageData(img, dst.x, dst.y);
    const p = paletteFor(biome), P = (i: number) => css(p[Math.min(i, p.length - 1)]!);
    const line = (x: number, y: number, lw: number, lh: number, col: string) => { g.ctx.fillStyle = col; g.ctx.fillRect(dst.x + x, dst.y + y, lw, lh); };
    if (mask & 1) { line(0, 0, w, 1, P(0)); line(0, 1, w, 1, P(4)); }
    if (mask & 8) { line(0, 0, 1, h, P(0)); line(1, 1, 1, h - 1, P(4)); }
    if (mask & 2) { line(w - 1, 0, 1, h, P(0)); line(w - 2, 1, 1, h - 1, P(1)); }
    if (mask & 4) { line(0, h - 1, w, 1, P(0)); line(0, h - 2, w, 1, P(1)); }
    return dst;
  }
  // bones pile (Cut 2 §2): per-biome 2-frame tile art if the atlas has it, else the `bones` item
  // glyph (atlas or procedural). Env density, 8×8, no shadow.
  bones(biome: string, frame: number): Slot {
    return this.env.get(`tile:${biome}_env_bones`) ?? this.env.get(`tile:${biome}_bones_${frame & 1}`) ?? this.env.get(`tile:${biome}_bones_0`) ?? this.envSlot("item:bones");
  }
  // ---- sprite-density ids -------------------------------------------------------------------
  // hero looks: `hero_<class>_<look>` when packed, else `hero_<class>` (atlas or procedural)
  entity(kind: string): Slot { kind = KIND_ALIAS[kind] ?? kind; const base = heroBase(kind); return (base && this.sprite.get(`ent:${kind}`)) || this.spriteSlot(`ent:${base ?? kind}`); }

  private envSlot(id: string): Slot {
    const s = this.env.get(id);
    if (s) return s;
    const [cat, rest] = id.split(":", 2) as [string, string];
    const g = this.env;
    if (cat === "tile") {
      const i = rest.indexOf("_");
      const biome = rest.slice(0, i), tile = rest.slice(i + 1);
      const slot = g.alloc(id, 8, 8);
      drawTile(g.ctx, slot, biome, tile);
      return slot;
    }
    if (cat === "item") { const slot = g.alloc(id, 8, 8); drawItem(g.ctx, slot, rest); return slot; }
    if (cat === "prop") { const slot = g.alloc(id, 8, 8); drawProp(g.ctx, slot, rest); return slot; }
    if (cat === "ov") {
      const i = rest.lastIndexOf("_");
      const slot = g.alloc(id, 8, 8);
      drawOverlay(g.ctx, slot, rest.slice(0, i), Number(rest.slice(i + 1)));
      return slot;
    }
    if (cat === "glyph") { const slot = g.alloc(id, 8, 8); drawGlyph(g.ctx, slot, rest); return slot; }
    if (cat === "font") { const slot = g.alloc(id, FONT_CELL_W, FONT_CELL_H); drawFontCell(g.ctx, slot, rest); return slot; }
    if (cat === "dot") { const slot = g.alloc(id, 2, 2); g.ctx.fillStyle = "#f4ecd8"; g.ctx.fillRect(slot.x, slot.y, 2, 2); return slot; }
    if (cat === "solid") { const slot = g.alloc(id, 2, 2); g.ctx.fillStyle = `#${rest}`; g.ctx.fillRect(slot.x, slot.y, 2, 2); return slot; }
    // shadow:<w>: w×2 ellipse (top row w-2 wide, bottom row w-4), one slot per exact width so
    // no quad is ever stretched to a non-integer texel size. ring:<w>: the same ellipse inside a
    // 1-texel light ring (companion marker), w×3.
    const w = Number(rest) || 8;
    if (cat === "ring") {
      const slot = g.alloc(id, w, 3);
      g.ctx.fillStyle = "#f4ecd8";
      g.ctx.fillRect(slot.x + 1, slot.y, w - 2, 1);
      g.ctx.fillRect(slot.x, slot.y + 1, 1, 1); g.ctx.fillRect(slot.x + w - 1, slot.y + 1, 1, 1);
      g.ctx.fillRect(slot.x + 1, slot.y + 2, w - 2, 1);
      g.ctx.fillStyle = "rgba(6,5,8,1)";
      g.ctx.fillRect(slot.x + 1, slot.y + 1, w - 2, 1);
      return slot;
    }
    const slot = g.alloc(id, w, 2);
    g.ctx.fillStyle = "rgba(6,5,8,1)";
    g.ctx.fillRect(slot.x + 1, slot.y, w - 2, 1);
    g.ctx.fillRect(slot.x + 2, slot.y + 1, w - 4, 1);
    return slot;
  }

  private spriteSlot(id: string): Slot {
    const s = this.sprite.get(id);
    if (s) return s;
    const kind = id.slice(4);
    const [w, h] = ENTITY_SIZE[kind] ?? [16, 32];
    if (SPRITE_SCALE === 1) { const slot = this.sprite.alloc(id, w, h); drawEntity(this.sprite.ctx, slot, kind); return slot; }
    // gfx round 7: drawn at its authored size, then cut down by area like a loaded sprite
    const c = document.createElement("canvas"); c.width = w; c.height = h;
    drawEntity(c.getContext("2d")!, { x: 0, y: 0, w, h, u0: 0, v0: 0, u1: 1, v1: 1 }, kind);
    return putDown(this.sprite, id, c, { x: 0, y: 0, w, h }, Math.max(1, Math.round(w * SPRITE_SCALE)), Math.max(1, Math.round(h * SPRITE_SCALE)));
  }

  // ---- external atlas ----------------------------------------------------------------------
  async load(url: string): Promise<boolean> {
    try {
      const res = await fetch(url);
      if (!res.ok) return false;
      const json = (await res.json()) as AtlasJson;
      if (!json.frames) return false;
      const base = url.slice(0, url.lastIndexOf("/") + 1);
      const img = await loadImage(base + (json.image ?? "atlas.png"));
      if (json.meta?.palettes) setPalettes(json.meta.palettes);
      for (const [id, r] of Object.entries(json.frames)) this.override(id, img, r, json.meta?.sprites?.[id]?.texel_h);
      this.loadedIds = new Set(Object.keys(json.frames));
      this.aliasTiles();
      this.waterTiles();
      return true;
    } catch {
      return false;
    }
  }

  loadedIds = new Set<string>(); // frame ids provided by the external atlas (for diagnostics)

  /** Cut 16 §3: every `tile:<alias>_…` slot the atlas loaded, recoloured into `tile:<biome>_…` (ramp index for index). */
  private aliasTiles(): void {
    const g = this.env;
    for (const [biome, alias] of Object.entries(TILE_ALIAS)) {
      const from = paletteFor(alias).map((c) => c.map((x) => Math.round(x * 255))), to = paletteFor(biome).map((c) => c.map((x) => Math.round(x * 255)));
      for (const id of this.loadedIds) {
        if (!id.startsWith(`${alias}_`)) continue;
        if (this.loadedIds.has(`${biome}_${id.slice(alias.length + 1)}`)) continue;   // juice pass 2: the biome's own art wins
        const src = g.get(`tile:${id}`); if (!src) continue;
        const px = g.ctx.getImageData(src.x, src.y, src.w, src.h), d = px.data;
        for (let i = 0; i < d.length; i += 4) {
          if (!d[i + 3]) continue;
          let best = 0, bd = Infinity;
          for (let k = 0; k < from.length; k++) { const f = from[k]!, e = (d[i]! - f[0]!) ** 2 + (d[i + 1]! - f[1]!) ** 2 + (d[i + 2]! - f[2]!) ** 2; if (e < bd) { bd = e; best = k; } }
          const t = to[Math.min(best, to.length - 1)]!; d[i] = t[0]!; d[i + 1] = t[1]!; d[i + 2] = t[2]!;
        }
        const dst = g.alloc(`tile:${biome}_${id.slice(alias.length + 1)}`, src.w, src.h);
        g.ctx.putImageData(px, dst.x, dst.y);
      }
    }
  }

  /** gfx round 1 (raters: "the checkered water reads as noise"): only the Fens have drawn water — every other biome's pool is the
   *  Fens' drawing moved into its own ramp (index for index), a calm pool with a few ripples instead of a 1-texel checker. */
  private waterTiles(): void {
    const g = this.env, src = g.get("tile:fens_env_water"); if (!src) return;
    const from = paletteFor("fens").map((c) => c.map((x) => Math.round(x * 255)));
    const base = g.ctx.getImageData(src.x, src.y, src.w, src.h);
    for (const biome of Object.keys(PALETTES)) {
      if (biome === "fens" || biome === "boss_flash") continue;
      const dst = g.get(`tile:${biome}_env_water`); if (!dst || dst.w !== src.w || dst.h !== src.h) continue;
      const to = paletteFor(biome).map((c) => c.map((x) => Math.round(x * 255)));
      const px = new ImageData(new Uint8ClampedArray(base.data), src.w, src.h), d = px.data;
      for (let i = 0; i < d.length; i += 4) {
        if (!d[i + 3]) continue;
        let best = 0, bd = Infinity;
        for (let k = 0; k < from.length; k++) { const f = from[k]!, e = (d[i]! - f[0]!) ** 2 + (d[i + 1]! - f[1]!) ** 2 + (d[i + 2]! - f[2]!) ** 2; if (e < bd) { bd = e; best = k; } }
        const t = to[Math.min(best, to.length - 1)]!; d[i] = t[0]!; d[i + 1] = t[1]!; d[i + 2] = t[2]!;
      }
      g.ctx.putImageData(px, dst.x, dst.y);
    }
  }

  private override(id: string, img: HTMLImageElement, r: Rect, texelH?: number): void {
    const put = (sheet: Sheet, slotId: string, w: number, h: number) => {
      const slot = sheet.alloc(slotId, w, h);
      sheet.ctx.imageSmoothingEnabled = false;
      sheet.ctx.drawImage(img, r.x, r.y, r.w, r.h, slot.x, slot.y, w, h);
    };
    const ovm = /^(?:ov_)?(gas|fire)(?:_(\d))?$/.exec(id);
    if (/^env_|_env_/.test(id)) put(this.env, `tile:${id}`, r.w, r.h);   // register 3: native size
    else if (r.h > 16) {
      // any tall frame is an entity; `boss_` prefix is stripped; box-fit at sprite density
      // meta.sprites[id].texel_h is the intended runtime height in sprite texels (masters are 2×)
      const kind = id.replace(/^boss_/, "");
      const [bw, bh] = ENTITY_BOX[kind] ?? [32, 32];
      const sc = (texelH ? texelH / r.h : Math.min(bw / r.w, bh / r.h)) * SPRITE_SCALE;
      putDown(this.sprite, `ent:${kind}`, img, r, Math.max(1, Math.round(r.w * sc)), Math.max(1, Math.round(r.h * sc)));
    } else if (ovm) {
      const frames = ovm[2] === undefined ? [0, 1] : [Number(ovm[2])];
      for (const f of frames) put(this.env, `ov:${ovm[1]}_${f}`, 8, 8);
    } else if (TILE_IDS.some((t) => id.endsWith(`_${t}`) || id.endsWith(`_${t}_alt`)) || /_(bones|shrine|nest)_[01]$|_vault(_open)?$/.test(id)) {
      put(this.env, `tile:${id}`, 8, 8);
    } else {
      put(this.env, `item:${id.replace(/^item_/, "")}`, 8, 8);
    }
  }
}

/** 307dbed control rater AR ("the ogre drew as a checkerboard blob"): a painted 2× master cut to its runtime size by nearest sampling
 *  keeps every other texel of its dither — the ogre's hide read as noise. A sprite is cut down by area: each texel the coverage-weighted
 *  mean of the master texels under it (colour weighted by alpha), kept where the master covers ≥ half of it (a crisp silhouette, the
 *  shader's alpha test unchanged). Same slot, same size; a master already at size is copied as before. */
function putDown(sheet: Sheet, id: string, img: CanvasImageSource, r: Rect, w: number, h: number): Slot {
  const slot = sheet.alloc(id, w, h);
  if (w >= r.w && h >= r.h) { sheet.ctx.imageSmoothingEnabled = false; sheet.ctx.drawImage(img, r.x, r.y, r.w, r.h, slot.x, slot.y, w, h); return slot; }
  const src = document.createElement("canvas"); src.width = r.w; src.height = r.h;
  const sc = src.getContext("2d", { willReadFrequently: true })!; sc.drawImage(img, r.x, r.y, r.w, r.h, 0, 0, r.w, r.h);
  const s = sc.getImageData(0, 0, r.w, r.h).data, out = sheet.ctx.createImageData(w, h), d = out.data;
  const fx = r.w / w, fy = r.h / h;
  for (let Y = 0; Y < h; Y++) {
    const y0 = Y * fy, y1 = y0 + fy;
    for (let X = 0; X < w; X++) {
      const x0 = X * fx, x1 = x0 + fx;
      let cr = 0, cg = 0, cb = 0, ca = 0, area = 0;
      for (let yy = Math.floor(y0); yy < Math.ceil(y1); yy++) {
        const wy = Math.min(y1, yy + 1) - Math.max(y0, yy);
        for (let xx = Math.floor(x0); xx < Math.ceil(x1); xx++) {
          const wgt = (Math.min(x1, xx + 1) - Math.max(x0, xx)) * wy, i = (yy * r.w + xx) * 4, a = (s[i + 3]! / 255) * wgt;
          cr += s[i]! * a; cg += s[i + 1]! * a; cb += s[i + 2]! * a; ca += a; area += wgt;
        }
      }
      const o = (Y * w + X) * 4;
      if (ca / area >= 0.5) { d[o] = Math.round(cr / ca); d[o + 1] = Math.round(cg / ca); d[o + 2] = Math.round(cb / ca); d[o + 3] = 255; }
    }
  }
  sheet.ctx.putImageData(out, slot.x, slot.y);
  return slot;
}

function loadImage(src: string): Promise<HTMLImageElement> {
  return new Promise((ok, err) => {
    const img = new Image();
    img.onload = () => ok(img);
    img.onerror = () => err(new Error(`image ${src}`));
    img.src = src;
  });
}

// ---- procedural fallbacks ------------------------------------------------------------------
type Ctx = CanvasRenderingContext2D;
const px = (c: Ctx, s: Rect, x: number, y: number, col: string) => { c.fillStyle = col; c.fillRect(s.x + x, s.y + y, 1, 1); };

function drawTile(c: Ctx, s: Slot, biome: string, tile: string): void {
  const p = paletteFor(biome);
  const P = (i: number) => css(p[i]!);
  const seed = biome.length * 7 + tile.length;
  for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++) {
    const r = hash(x + s.x, y + s.y, seed);
    let col: string;
    switch (tile) {
      case "floor": col = r < 0.08 ? P(2) : P(1); break;
      case "wall": {
        // brick courses: 2-row bricks offset every other course, dark mortar
        const mortarY = y % 4 === 3, mortarX = ((y >> 2) & 1 ? (x + 4) % 8 : x) === 7;
        col = mortarY || mortarX ? P(0) : r < 0.15 ? P(3) : P(2);
        if (y === 0) col = P(3);
        break;
      }
      case "door": col = x === 0 || x === 7 || y === 0 ? P(1) : (x === 3 || x === 4) && y > 2 ? P(2) : P(5); break;
      case "stairs_down": col = (y & 1) === 0 ? P(2) : P(1); if (x < y - 1) col = P(0); break;
      case "stairs_up": col = (y & 1) === 0 ? P(6) : P(4); if (x === 0 || x === 7) col = P(1); break;
      case "water":
        // fens has teal in its palette; elsewhere water is a dark liquid with sparse glints
        col = biome === "fens"
          ? (((x + y * 3 + (r < 0.3 ? 1 : 0)) & 7) < 2 ? P(4) : P(3))
          : (((x + y * 3) & 7) < 2 && (y & 1) === 0 ? P(3) : ((x + y) & 1) === 0 ? P(1) : P(0));
        break;
      case "chasm": col = r < 0.06 ? P(1) : P(0); break;
      default: col = r < 0.5 ? P(7) : P(0);
    }
    px(c, s, x, y, col);
  }
}

function drawItem(c: Ctx, s: Slot, kind: string): void {
  const D = "#1a1410", L = "#e8dcc0";
  const potion = /heal|strength|speed|invis|poison|caustic|confus|fire|potion/.test(kind);
  const scroll = /teleport|blink|fear|mapping|identify|enchant|darkness|summon|aggravate|scroll/.test(kind);
  const weapon = /dagger|sword|axe|bow/.test(kind);
  const armour = /leather|mail|plate/.test(kind);
  const gold = /gold/.test(kind);
  if (/^bones$/.test(kind)) {
    // skull (3×3 with eye pits) over two crossed long bones
    const B = "#e8dcc0", S = "#9a8e78";
    for (const [x, y] of [[0, 6], [1, 5], [2, 4], [5, 4], [6, 5], [7, 6], [0, 4], [1, 5], [6, 5], [7, 4]] as [number, number][]) px(c, s, x, y, S);
    for (let y = 1; y <= 3; y++) for (let x = 2; x <= 5; x++) px(c, s, x, y, B);
    px(c, s, 3, 2, D); px(c, s, 4, 2, D); px(c, s, 3, 4, S); px(c, s, 4, 4, S);
    return;
  }
  for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++) {
    let col: string | null = null;
    if (potion) {
      const tint = /heal/.test(kind) ? "#c8404a" : /fire/.test(kind) ? "#e08a30" : /poison|caustic/.test(kind) ? "#7fbf3a" : "#5a8ad0";
      if (y >= 1 && y <= 2 && x >= 3 && x <= 4) col = D;
      else if (y >= 3 && y <= 6 && x >= 2 && x <= 5) col = (x === 2 || x === 5 || y === 6) ? D : y === 3 ? L : tint;
    } else if (scroll) {
      if (y >= 1 && y <= 6 && x >= 1 && x <= 6) col = (x === 1 || x === 6 || y === 1 || y === 6) ? D : (y === 3 || y === 5) && x > 2 && x < 6 ? "#8a7a5a" : "#f0e6cc";
    } else if (weapon) {
      const bow = /bow/.test(kind);
      if (bow) { if ((x === 2 && y > 0 && y < 7) || (y === 1 && x === 3) || (y === 6 && x === 3) || (x === 5 && y > 1 && y < 6)) col = x === 5 ? L : "#8a5a2a"; }
      else if (x + y === 7 && x > 0 && x < 7) col = x < 3 ? "#5a3a1a" : L;
      else if ((x + y === 6 || x + y === 8) && x > 2 && x < 7) col = D;
    } else if (armour) {
      if (y >= 1 && y <= 6 && x >= 1 && x <= 6 && !(y === 1 && (x === 3 || x === 4)) && !(y > 3 && (x === 1 || x === 6)))
        col = (y === 6 || x === 1 || x === 6 || y === 1) ? D : /plate/.test(kind) ? "#b8bcc0" : /mail/.test(kind) ? "#8a8e94" : "#9a6a3a";
    } else if (gold) {
      if (Math.abs(x - 3.5) + Math.abs(y - 3.5) < 3.2) col = (Math.abs(x - 3.5) + Math.abs(y - 3.5) > 2.2) ? "#8a6a10" : "#f0c840";
    } else {
      if (y >= 1 && y <= 6 && x >= 1 && x <= 6) col = (x === 1 || x === 6 || y === 1 || y === 6) ? D : "#c060c0";
      if (y === 3 && (x === 3 || x === 4)) col = D;
      if (y === 4 && x === 4) col = D;
      if (y === 2 && x === 3) col = D;
    }
    if (col) px(c, s, x, y, col);
  }
}

// Cut 5 §4 prop fallbacks, 8×8 over a transparent ground: shrine = a small altar (plinth + a candle whose flame
// flickers between frames), vault = a barred square (bars lifted when open), nest = a mound (a glint when woken).
function drawProp(c: Ctx, s: Slot, id: string): void {
  const D = "#1a1410", S = "#8a8090", L = "#c8c0b0", G = "#f0c840";
  if (id.startsWith("shrine")) {
    const f = id.endsWith("1");
    for (let x = 1; x <= 6; x++) { px(c, s, x, 7, D); px(c, s, x, 6, S); }
    for (let x = 2; x <= 5; x++) { px(c, s, x, 5, S); px(c, s, x, 4, L); }
    px(c, s, 3, 3, L); px(c, s, 4, 3, L);
    px(c, s, f ? 4 : 3, 2, G); px(c, s, f ? 3 : 4, 1, "#e07020");
    return;
  }
  if (id.startsWith("vault")) {
    const open = id.endsWith("open");
    for (let i = 0; i < 8; i++) { px(c, s, i, 0, D); px(c, s, i, 7, D); px(c, s, 0, i, D); px(c, s, 7, i, D); }
    for (let y = 1; y <= 6; y++) for (let x = 1; x <= 6; x++) px(c, s, x, y, open ? "#2a2420" : "#3a3028");
    if (!open) for (const x of [2, 4, 6]) for (let y = 1; y <= 6; y++) px(c, s, x, y, S);
    else for (const x of [2, 4, 6]) px(c, s, x, 1, S);
    if (open) px(c, s, 3, 4, G);
    return;
  }
  // nest: a mound of straw, a glint when woken
  const woke = id.endsWith("1");
  for (let y = 3; y < 8; y++) { const r = y - 3; for (let x = 1 + Math.max(0, 2 - r); x <= 6 - Math.max(0, 2 - r); x++) px(c, s, x, y, ((x + y) & 1) ? "#8a6a3a" : "#6a4a2a"); }
  px(c, s, 3, 5, D); px(c, s, 4, 5, D);
  if (woke) { px(c, s, 3, 5, "#e03030"); px(c, s, 4, 5, "#e03030"); }
}

function drawOverlay(c: Ctx, s: Slot, kind: string, frame: number): void {
  for (let y = 0; y < 8; y++) for (let x = 0; x < 8; x++) {
    const r = hash(x, y, frame * 31 + kind.length);
    if (kind === "gas") {
      const on = ((x + y + frame) & 1) === 0 && r < 0.75;
      if (on) px(c, s, x, y, r < 0.35 ? "#b8d060" : "#7aa040");
    } else {
      // fire: flame tongues, brighter at the base, frame shifts the tips
      const h = 8 - y;
      const tip = frame ? [2, 5, 7, 3, 6, 4, 5, 3][x]! : [3, 6, 4, 7, 5, 3, 6, 4][x]!;
      if (h <= tip) px(c, s, x, y, h > tip - 2 ? "#f0d040" : h > tip - 4 ? "#e07020" : "#b02818");
    }
  }
}

function drawGlyph(c: Ctx, s: Slot, ch: string): void {
  // 8×8: 3×5 glyph scaled ×1 centred, dark box behind for contrast
  const bits = glyphBits(ch);
  c.fillStyle = "#1a1410";
  c.fillRect(s.x + 1, s.y, 6, 8);
  for (let y = 0; y < FONT_H; y++) for (let x = 0; x < FONT_W; x++)
    if (bits[y * FONT_W + x] === "1") px(c, s, x + 2, y + 1, ch === "!" ? "#f0d040" : "#e8dcc0");
  // stem for the "!" pointer
  px(c, s, 3, 7, "#1a1410"); px(c, s, 4, 7, "#1a1410");
}

function drawFontCell(c: Ctx, s: Slot, ch: string): void {
  const bits = glyphBits(ch);
  const on = (x: number, y: number) => x >= 0 && y >= 0 && x < FONT_W && y < FONT_H && bits[y * FONT_W + x] === "1";
  for (let y = 0; y < FONT_CELL_H; y++) for (let x = 0; x < FONT_CELL_W; x++) {
    const gx = x - 1, gy = y - 1;
    if (on(gx, gy)) px(c, s, x, y, "#f4ecd8");
    else if (on(gx - 1, gy) || on(gx + 1, gy) || on(gx, gy - 1) || on(gx, gy + 1) ||
             on(gx - 1, gy - 1) || on(gx + 1, gy - 1) || on(gx - 1, gy + 1) || on(gx + 1, gy + 1)) px(c, s, x, y, "#14100c");
  }
}

// Sprite-density silhouette: head + body blob, 2-px dark outline, 1-px paper rim, letter glyph.
function drawEntity(c: Ctx, s: Slot, kind: string): void {
  const [fill, ink] = ENTITY_COLOURS[kind] ?? ["#8a8090", "#c020c0"];
  const w = s.w, h = s.h;
  const mask: Uint8Array = new Uint8Array(w * h);
  const cx = (w - 1) / 2;
  const headR = Math.max(3, Math.round(w * 0.28));
  const headCy = headR + 2;
  const bodyTop = headCy + headR - 1;
  const bodyBot = h - 3;
  const bodyRx = w * 0.42, bodyRy = (bodyBot - bodyTop) / 2 + 1;
  const bodyCy = (bodyTop + bodyBot) / 2;
  const blob = /jelly|bloat/.test(kind);
  const tall = h >= 48;
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    const dxh = x - cx, dyh = y - headCy;
    let inside = false;
    if (blob) {
      const dx = (x - cx) / (w * 0.45), dy = (y - (h * 0.6)) / (h * 0.36);
      inside = dx * dx + dy * dy <= 1;
    } else {
      inside = dxh * dxh + dyh * dyh <= headR * headR;
      const dx = (x - cx) / bodyRx, dy = (y - bodyCy) / bodyRy;
      if (dx * dx + dy * dy <= 1) inside = true;
      // legs gap at the bottom for tall figures
      if (tall && y > bodyBot - 4 && Math.abs(dxh) < 1.2) inside = false;
    }
    if (inside) mask[y * w + x] = 1;
  }
  const at = (x: number, y: number) => x >= 0 && y >= 0 && x < w && y < h && mask[y * w + x] === 1;
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    if (at(x, y)) {
      // 1-px paper-white inner rim
      const edge = !at(x - 1, y) || !at(x + 1, y) || !at(x, y - 1) || !at(x, y + 1);
      px(c, s, x, y, edge ? "#f7f2e6" : fill);
    } else {
      // 2-px dark outline (dilate the mask by 2)
      let near = false;
      for (let oy = -2; oy <= 2 && !near; oy++) for (let ox = -2; ox <= 2; ox++)
        if (Math.abs(ox) + Math.abs(oy) <= 3 && at(x + ox, y + oy)) { near = true; break; }
      if (near) px(c, s, x, y, "#15110f");
    }
  }
  // shading: darker lower-right quadrant inside the fill (key light top-left)
  for (let y = 0; y < h; y++) for (let x = 0; x < w; x++) {
    if (!at(x, y) || !at(x + 2, y + 2) || !at(x - 1, y - 1)) continue;
    if (x > cx + 1 && y > bodyCy && ((x + y) & 1) === 0) px(c, s, x, y, shade(fill));
  }
  // letter glyph, 2× scale, ink colour, on the chest
  const bits = glyphBits(kind.replace(/^hero_/, "")[0] ?? "?");
  const sc = tall ? 2 : 1;
  const gx0 = Math.round(cx - (FONT_W * sc) / 2 + 0.5), gy0 = Math.round(bodyCy - (FONT_H * sc) / 2);
  for (let y = 0; y < FONT_H; y++) for (let x = 0; x < FONT_W; x++)
    if (bits[y * FONT_W + x] === "1") { c.fillStyle = ink; c.fillRect(s.x + gx0 + x * sc, s.y + gy0 + y * sc, sc, sc); }
}

function shade(hexCol: string): string {
  const r = parseInt(hexCol.slice(1, 3), 16), g = parseInt(hexCol.slice(3, 5), 16), b = parseInt(hexCol.slice(5, 7), 16);
  const f = 0.72;
  return css([r * f / 255, g * f / 255, b * f / 255] as Rgb);
}
