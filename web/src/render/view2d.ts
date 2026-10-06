// Cohort 24 (AW: a WebGL context loss froze the watch for 7 minutes on a placeholder view): a Canvas-2D view of the replay state.
// It stands in while the GL context is gone (index.ts draws it on an overlay canvas over the GL one, the clock running on) and is
// the whole view of `createFallbackViewer` when no GL context can be made at all. The atlas' own frames (full colour) drawn with
// drawImage, nearest-neighbour: the same tiles and sprites as the GL view, without its lighting and effects. Never game truth.
import type { ReplayState, EntState } from "./state";
import { paletteFor, css, spriteScale } from "./palette";
import { heroBase } from "./look";

type Frame = { x: number; y: number; w: number; h: number };
type Sheet = { img: HTMLImageElement; frames: Record<string, Frame>; texelH: Record<string, number> };
let sheet: Sheet | null = null, loading: Promise<void> | null = null;
function loadSheet(url = `${import.meta.env.BASE_URL}art/atlas.json`): void {
  if (loading || typeof fetch === "undefined") return;
  loading = fetch(url).then((r) => r.json()).then((j: { frames?: Record<string, Frame>; meta?: { sprites?: Record<string, { texel_h?: number }> } }) => new Promise<void>((res) => {
    const img = new Image();
    const texelH: Record<string, number> = {};
    for (const [id, m] of Object.entries(j.meta?.sprites ?? {})) if (m.texel_h) texelH[id] = m.texel_h;
    img.onload = () => { sheet = { img, frames: j.frames ?? {}, texelH }; res(); };
    img.onerror = () => res();
    img.src = url.replace(/\.json$/, ".png");
  })).catch(() => undefined);
}

const TILES_SHORT = 15;   // tiles along the short axis (the GL view's 120 env texels / 8)
const hash2 = (x: number, y: number, s: number): number => {
  let h = (x * 374761393 + y * 668265263 + s * 2246822519) | 0;
  h = Math.imul(h ^ (h >>> 13), 1274126177);
  return ((h ^ (h >>> 16)) >>> 0) / 4294967296;
};

export class View2D {
  readonly canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D | null;
  private host: HTMLCanvasElement | null;   // the GL canvas an overlay sits on (null: `canvas` is the view itself)
  private box = "";
  shown = false;

  /** `host`: overlay mode (a sibling canvas over the GL canvas); `own`: draw into this 2D canvas. */
  constructor(o: { host?: HTMLCanvasElement; own?: HTMLCanvasElement }) {
    loadSheet();
    this.host = o.host ?? null;
    if (o.own) this.canvas = o.own;
    else {
      this.canvas = document.createElement("canvas");
      this.canvas.className = "view-2d";
      this.canvas.setAttribute("aria-hidden", "true");
      Object.assign(this.canvas.style, { position: "absolute", pointerEvents: "none", display: "none", background: "#07080a", imageRendering: "pixelated" });
      this.host!.parentElement?.insertBefore(this.canvas, this.host!.nextSibling);
    }
    this.ctx = this.canvas.getContext("2d");
    if (!this.host) this.shown = true;
  }

  show(on: boolean): void {
    this.shown = on;
    if (this.host) this.canvas.style.display = on ? "block" : "none";
  }

  private fit(): [number, number, number] {
    const c = this.host ?? this.canvas;
    if (this.host) {
      const key = `${c.offsetLeft},${c.offsetTop},${c.clientWidth},${c.clientHeight}`;
      if (key !== this.box) { this.box = key; Object.assign(this.canvas.style, { left: `${c.offsetLeft}px`, top: `${c.offsetTop}px`, width: `${c.clientWidth}px`, height: `${c.clientHeight}px` }); }
    }
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = c.clientWidth || c.width || 300, h = c.clientHeight || c.height || 500;
    const W = Math.max(1, Math.floor(w * dpr)), H = Math.max(1, Math.floor(h * dpr));
    if (this.canvas.width !== W || this.canvas.height !== H) { this.canvas.width = W; this.canvas.height = H; }
    return [W, H, dpr];
  }

  private frame(id: string): Frame | undefined { return sheet?.frames[id]; }

  draw(st: ReplayState): void {
    const g = this.ctx; if (!g || !this.shown) return;
    const [W, H] = this.fit();
    const pal = paletteFor(st.biome);
    g.imageSmoothingEnabled = false;
    g.fillStyle = "#07080a"; g.fillRect(0, 0, W, H);
    if (!st.loaded) return;
    const T = Math.max(8, Math.floor(Math.min(W, H) / TILES_SHORT));
    const hero = st.hero;
    const cx = hero ? hero.px + 0.5 : st.w / 2, cy = hero ? hero.py + 0.5 : st.h / 2;
    const ox = Math.round(W / 2 - cx * T), oy = Math.round(H / 2 - cy * T);
    const x0 = Math.max(0, Math.floor(-ox / T) - 1), x1 = Math.min(st.w - 1, Math.ceil((W - ox) / T) + 1);
    const y0 = Math.max(0, Math.floor(-oy / T) - 1), y1 = Math.min(st.h - 1, Math.ceil((H - oy) / T) + 1);
    const b = st.biome, img = sheet?.img;
    const open = (x: number, y: number): boolean => x >= 0 && y >= 0 && x < st.w && y < st.h && !!st.seen[y * st.w + x] && st.tiles[y * st.w + x] !== "wall";
    const blit = (id: string, dx: number, dy: number, dw: number, dh: number, flip = false): boolean => {
      const f = this.frame(id); if (!f || !img) return false;
      if (flip) { g.save(); g.translate(dx + dw, dy); g.scale(-1, 1); g.drawImage(img, f.x, f.y, f.w, f.h, 0, 0, dw, dh); g.restore(); }
      else g.drawImage(img, f.x, f.y, f.w, f.h, dx, dy, dw, dh);
      return true;
    };
    // tiles
    for (let y = y0; y <= y1; y++) for (let x = x0; x <= x1; x++) {
      const i = y * st.w + x;
      if (!st.seen[i]) continue;
      const t = st.tiles[i]!, px = ox + x * T, py = oy + y * T, hh = hash2(x, y, 11);
      let id: string;
      if (t === "wall") id = open(x, y + 1) ? `${b}_env_wall_face_${hh < 0.72 ? 0 : 1}` : `${b}_env_wall_top`;
      else if (t === "floor" || t === "shrine" || t === "vault" || t === "vault_open" || t === "nest") id = `${b}_env_floor_${hh < 0.46 ? 0 : hh < 0.76 ? 1 : hh < 0.92 ? 2 : 3}`;
      else id = `${b}_env_${t}`;
      if (!blit(id, px, py, T, T) && !blit(`${b}_${t}`, px, py, T, T)) {
        g.fillStyle = css(t === "wall" ? pal[1]! : t === "water" ? pal[3]! : t === "chasm" ? pal[0]! : t.startsWith("stairs") ? pal[6]! : pal[2]!);
        g.fillRect(px, py, T, T);
      }
      if (t === "shrine" || t === "vault" || t === "vault_open" || t === "nest") blit(`${b}_env_${t === "shrine" || t === "nest" ? `${t}_0` : t}`, px, py, T, T);
      if (t === "wall" && !open(x, y + 1)) { g.fillStyle = "rgba(0,0,0,0.28)"; g.fillRect(px, py, T, T); }
      if (!st.visible[i]) { g.fillStyle = "rgba(0,0,0,0.38)"; g.fillRect(px, py, T, T); }
    }
    // overlays, items
    const fr = Math.floor(performance.now() / 250) & 1;
    for (const o of st.overlays) {
      if (!st.visible[o.y * st.w + o.x]) continue;
      const px = ox + o.x * T, py = oy + o.y * T;
      if (!blit(`${o.k}_${fr}`, px, py, T, T)) { g.fillStyle = o.k === "gas" ? "rgba(120,200,80,0.45)" : "rgba(240,120,40,0.55)"; g.fillRect(px, py, T, T); }
    }
    for (const it of st.items) {
      if (!st.seen[it.y * st.w + it.x]) continue;
      const px = ox + it.x * T, py = oy + it.y * T;
      const kind = it.kind === "bones" ? "bones" : it.known ? it.kind : /scroll/.test(it.label) ? "scroll" : "potion";
      if (!blit(`env_item_${kind}`, px, py, T, T) && !blit(kind, px, py, T, T)) { g.fillStyle = it.kind === "gold" ? "#e8c24a" : "#9ad1ff"; g.fillRect(px + T * 0.3, py + T * 0.3, T * 0.4, T * 0.4); }
    }
    // entities: sprite texel = T/16 (a tile is 8 env texels, a sprite texel half of one); feet at the tile's bottom centre, lower rows in front
    const ents = [...st.ents.values()].filter((e) => e.kind !== "bones" && (e.hero || st.visible[e.y * st.w + e.x] || (e.remembered && st.seen[e.y * st.w + e.x])));
    ents.sort((a, c) => (a.hero ? 1 : 0) - (c.hero ? 1 : 0) || a.py - c.py);
    const s = T / 16;
    for (const e of ents) this.ent(g, e, ox, oy, T, s, blit, st.tacticMarked(e));
    // the hero's light: the room around him lit, the edges falling to dark
    if (hero) {
      const hx = ox + (hero.px + 0.5) * T, hy = oy + (hero.py + 0.5) * T, r = T * Math.max(4, st.vision + 1);
      const grad = g.createRadialGradient(hx, hy, T * 1.5, hx, hy, r);
      grad.addColorStop(0, "rgba(255,190,110,0.10)"); grad.addColorStop(0.55, "rgba(0,0,0,0)"); grad.addColorStop(1, "rgba(0,0,0,0.55)");
      g.fillStyle = grad; g.fillRect(0, 0, W, H);
    }
    if (st.fade > 0) { g.fillStyle = `rgba(0,0,0,${Math.min(1, st.fade)})`; g.fillRect(0, 0, W, H); }
  }

  private ent(g: CanvasRenderingContext2D, e: EntState, ox: number, oy: number, T: number, s: number,
              blit: (id: string, dx: number, dy: number, dw: number, dh: number, flip?: boolean) => boolean, marked = false): void {
    const ids = [e.kind, `boss_${e.kind}`, heroBase(e.kind) ?? "", e.hero ? "hero_fighter" : ""].filter(Boolean);
    const f = ids.map((id) => this.frame(id)).find(Boolean);
    const fx = ox + (e.px + 0.5) * T, fy = oy + (e.py + 1) * T;
    g.globalAlpha = e.remembered ? 0.5 : Math.max(0.12, 1 - e.fade * 0.9);
    let top = fy - T;
    if (marked) {
      g.strokeStyle = '#eac77c'; g.lineWidth = Math.max(1, s);
      g.beginPath(); g.ellipse(fx, fy - s, T * .4, s * 2.5, 0, 0, Math.PI * 2); g.stroke();
    }
    if (f) {
      // the sprite's runtime height (atlas `texel_h` sprite texels, the masters are 2×) × the GL view's scale for its kind
      const id = ids.find((x) => this.frame(x))!, th = sheet?.texelH[id] ?? f.h / 2;
      const h = th * s * spriteScale(e.kind), w = (f.w / f.h) * h; top = fy - h;
      if (!e.remembered && !e.dying) { g.fillStyle = "rgba(0,0,0,0.35)"; g.beginPath(); g.ellipse(fx, fy - s, Math.min(w * 0.35, T * 0.45), s * 2.5, 0, 0, Math.PI * 2); g.fill(); }
      blit(id, Math.round(fx - w / 2), Math.round(top), Math.round(w), Math.round(h), e.flip);
    } else {
      g.fillStyle = e.hero ? "#f4f1ea" : e.ally ? "#5fbf7a" : e.boss ? "#ff5aa0" : "#d94a4a";
      g.fillRect(fx - T * 0.4, fy - T * 0.9, T * 0.8, T * 0.8);
    }
    g.globalAlpha = 1;
    if (!e.hero && !e.neutral && !e.dying && !e.remembered && e.maxHp > 0) {
      const bw = T * 0.8, p = Math.max(0, Math.min(1, e.hp / e.maxHp));
      g.fillStyle = "#1a0f0c"; g.fillRect(fx - bw / 2, top - 5, bw, 4);
      g.fillStyle = e.ally ? "#4fbf3a" : "#d0302a"; g.fillRect(fx - bw / 2, top - 5, bw * p, 4);
    }
  }

  dispose(): void { if (this.host) this.canvas.remove(); this.ctx = null; }
}

// ---- Cut 30 §3: the town (render/town.ts) — the same frame the GL view draws, on a 2D canvas ----------------------------------
import type { TownFrame, Quad } from "./town";
const byZ = (a: Quad, b: Quad): number => a.z - b.z || a.y - b.y;
let townSorted: Quad[] = [];
/** Draws a town frame (`render/town.ts`): the ground, the sprites by depth, then the light — the ambient multiplied over the scene
 *  and each light's pool added (the GL view's light pass, without its dither bands). `atlas`: the town's own sprite sheet. */
export function drawTown2D(c: HTMLCanvasElement, F: TownFrame, atlas: HTMLCanvasElement): void {
  if (c.width !== F.W || c.height !== F.H) { c.width = F.W; c.height = F.H; }
  const g = c.getContext("2d"); if (!g) return;
  const k = F.k;
  g.imageSmoothingEnabled = false;
  g.globalCompositeOperation = "source-over"; g.globalAlpha = 1;
  g.fillStyle = "#1b2412"; g.fillRect(0, 0, F.W, F.H);
  const draw = (q: Quad): void => {
    const dx = Math.round((q.x - q.w / 2 - F.x0) * k), dy = Math.round((q.y - q.h - F.y0) * k), dw = q.w * k, dh = q.h * k;
    if (dx > F.W || dy > F.H || dx + dw < 0 || dy + dh < 0) return;
    g.globalAlpha = q.fade > 0 ? Math.max(0, 1 - q.fade) : 1;
    if (q.flip) { g.save(); g.translate(dx + dw, dy); g.scale(-1, 1); g.drawImage(atlas, q.s.x, q.s.y, q.s.w, q.s.h, 0, 0, dw, dh); g.restore(); }
    else g.drawImage(atlas, q.s.x, q.s.y, q.s.w, q.s.h, dx, dy, dw, dh);
  };
  for (const q of F.ground) { draw(q); if (q.dim < 1) { g.globalAlpha = 1 - q.dim; g.fillStyle = "#0d0c14"; g.fillRect(Math.round((q.x - q.w / 2 - F.x0) * k), Math.round((q.y - q.h - F.y0) * k), q.w * k, q.h * k); g.globalAlpha = 1; } }
  townSorted.length = 0;
  for (let i = 0; i < F.n; i++) townSorted.push(F.quads[i]!);
  townSorted.sort(byZ);
  for (const q of townSorted) draw(q);
  townSorted = townSorted.slice(0, 0);
  g.globalAlpha = 1;
  const [r, gg, b] = F.amb;
  if (r < 0.99 || gg < 0.99 || b < 0.99) {
    g.globalCompositeOperation = "multiply";
    g.fillStyle = `rgb(${Math.round(Math.min(1, r) * 255)},${Math.round(Math.min(1, gg) * 255)},${Math.round(Math.min(1, b) * 255)})`;
    g.fillRect(0, 0, F.W, F.H);
  }
  g.globalCompositeOperation = "lighter";
  for (const l of F.lights) {
    const x = (l.x - F.x0) * k, y = (l.y - F.y0) * k, rr = l.r * k;
    const grad = g.createRadialGradient(x, y, 0, x, y, rr);
    const col = (a: number): string => `rgba(${Math.round(Math.min(1, l.c[0]) * 255)},${Math.round(Math.min(1, l.c[1]) * 255)},${Math.round(Math.min(1, l.c[2]) * 255)},${a})`;
    grad.addColorStop(0, col(0.32 + 0.3 * F.night)); grad.addColorStop(1, col(0));
    g.fillStyle = grad; g.fillRect(x - rr, y - rr, rr * 2, rr * 2);
  }
  g.globalCompositeOperation = "source-over";
}
