// Canvas-2D stand-in for the replay viewer: tiles, fog, entities as blocks, overlays. No animation.
import type { Entity, Overlay, Snapshot } from "../engine/types";
import type { Viewer } from "./viewer";

const TILE_COLOUR: Record<string, string> = {
  floor: "#2a2d34", wall: "#111317", door: "#5a4a2a", stairs_down: "#c9a227", stairs_up: "#7a8a9a", water: "#1f3f6b", chasm: "#05060a",
  shrine: "#6a5a8a", vault: "#5a5a5a", vault_open: "#3a3a3a", nest: "#6a4a2a",
};
const BIOME_TINT: Record<string, string> = { warrens: "#3a3020", fens: "#1e3a2e", crypt: "#2a2038" };

export function createPlaceholderViewer(canvas: HTMLCanvasElement): Viewer {
  const ctx = canvas.getContext("2d");
  let snap: Snapshot | null = null;
  let ents = new Map<number, Entity>();
  let overlays: Overlay[] = [];
  let hero: Entity | null = null;
  let flashes: { x: number; y: number; ttl: number }[] = [];
  let raf = 0;

  function fit(): void {
    const dpr = Math.min(2, window.devicePixelRatio || 1);
    const w = canvas.clientWidth || 300, h = canvas.clientHeight || 500;
    if (canvas.width !== Math.floor(w * dpr) || canvas.height !== Math.floor(h * dpr)) { canvas.width = Math.floor(w * dpr); canvas.height = Math.floor(h * dpr); }
  }
  function draw(): void {
    raf = 0;
    if (!ctx || !snap) return;
    fit();
    const W = canvas.width, H = canvas.height;
    ctx.fillStyle = "#07080a"; ctx.fillRect(0, 0, W, H);
    const ts = Math.floor(Math.min(W / snap.w, H / snap.h));
    const ox = Math.floor((W - ts * snap.w) / 2), oy = Math.floor((H - ts * snap.h) / 2);
    const tint = BIOME_TINT[snap.biome] ?? "#000";
    for (let y = 0; y < snap.h; y++) for (let x = 0; x < snap.w; x++) {
      const i = y * snap.w + x;
      if (!snap.seen[i]) continue;
      ctx.fillStyle = TILE_COLOUR[snap.tiles[i]] ?? "#f0f";
      ctx.fillRect(ox + x * ts, oy + y * ts, ts, ts);
      if (snap.tiles[i] === "floor") { ctx.globalAlpha = 0.25; ctx.fillStyle = tint; ctx.fillRect(ox + x * ts, oy + y * ts, ts, ts); ctx.globalAlpha = 1; }
      if (!snap.visible[i]) { ctx.fillStyle = "rgba(0,0,0,0.55)"; ctx.fillRect(ox + x * ts, oy + y * ts, ts, ts); }
    }
    for (const o of overlays) { ctx.fillStyle = o.k === "gas" ? "rgba(120,200,80,0.45)" : "rgba(240,120,40,0.55)"; ctx.fillRect(ox + o.x * ts, oy + o.y * ts, ts, ts); }
    for (const it of snap.items) { if (!snap.seen[it.y * snap.w + it.x]) continue; ctx.fillStyle = it.kind === "gold" ? "#e8c24a" : "#9ad1ff"; ctx.fillRect(ox + it.x * ts + ts * 0.3, oy + it.y * ts + ts * 0.3, ts * 0.4, ts * 0.4); }
    for (const e of ents.values()) {
      if (!snap.visible[e.y * snap.w + e.x]) continue;
      ctx.fillStyle = e.ally ? "#5fbf7a" : e.tags.includes("boss") ? "#ff5aa0" : "#d94a4a";
      ctx.fillRect(ox + e.x * ts + 1, oy + e.y * ts + 1, ts - 2, ts - 2);
      if (e.telegraph) { ctx.fillStyle = "#f2c14e"; ctx.fillRect(ox + e.x * ts + ts * 0.35, oy + e.y * ts - ts * 0.3, ts * 0.3, ts * 0.3); }
      const hpw = Math.max(0, Math.round((e.hp / Math.max(1, e.max_hp)) * (ts - 2)));
      ctx.fillStyle = "#000"; ctx.fillRect(ox + e.x * ts + 1, oy + e.y * ts + ts - 3, ts - 2, 2);
      ctx.fillStyle = "#e05050"; ctx.fillRect(ox + e.x * ts + 1, oy + e.y * ts + ts - 3, hpw, 2);
    }
    if (hero) { ctx.fillStyle = "#f4f1ea"; ctx.fillRect(ox + hero.x * ts + 1, oy + hero.y * ts + 1, ts - 2, ts - 2); }
    for (const f of flashes) { ctx.fillStyle = `rgba(255,255,255,${f.ttl / 4})`; ctx.fillRect(ox + f.x * ts, oy + f.y * ts, ts, ts); f.ttl--; }
    flashes = flashes.filter((f) => f.ttl > 0);
    if (flashes.length) schedule();
  }
  function schedule(): void { if (!raf) raf = requestAnimationFrame(draw); }
  const onResize = (): void => schedule();
  window.addEventListener("resize", onResize);

  return {
    load(s) {
      snap = s; hero = { ...s.hero }; ents = new Map(s.entities.map((e) => [e.id, { ...e }])); overlays = s.overlays.map((o) => ({ ...o })); flashes = [];
      schedule();
    },
    apply(evs) {
      if (!snap) return;
      for (const ev of evs) {
        switch (ev.k) {
          case "move": { if (ev.id === 0 && hero) { hero.x = ev.x; hero.y = ev.y; reveal(snap, ev.x, ev.y); } else { const e = ents.get(ev.id); if (e) { e.x = ev.x; e.y = ev.y; } } break; }
          case "hurt": { const e = ev.id === 0 ? hero : ents.get(ev.id); if (e) { e.hp = ev.hp; flashes.push({ x: e.x, y: e.y, ttl: 3 }); } break; }
          case "die": { if (ev.id !== 0) ents.delete(ev.id); break; }
          case "spawn": ents.set(ev.e.id, { ...ev.e }); break;
          case "steal": ents.delete(ev.id); break;
          case "telegraph": { const e = ents.get(ev.id); if (e) e.telegraph = ev.what; break; }
          case "attack": { const e = ents.get(ev.src); if (e) e.telegraph = undefined; const d = ev.dst === 0 ? hero : ents.get(ev.dst); if (d && ev.hit) flashes.push({ x: d.x, y: d.y, ttl: 2 }); break; }
          case "overlay": overlays.push({ x: ev.x, y: ev.y, k: ev.ov, ttl: ev.ttl }); break;
          case "pickup": snap.items = snap.items.filter((i) => i.id !== ev.id); break;
          default: break;
        }
      }
      for (const o of overlays) o.ttl -= 0.34;
      overlays = overlays.filter((o) => o.ttl > 0);
      schedule();
    },
    setSpeed() { /* no animation to pace */ },
    skipToEvent() { flashes = []; schedule(); },
    resize() { schedule(); },
    idle() { return true; },
    dispose() { window.removeEventListener("resize", onResize); if (raf) cancelAnimationFrame(raf); },
  };
}
function reveal(snap: Snapshot, hx: number, hy: number): void {
  // cheap fog: mark a radius as seen/visible so movement between snapshots reads; the next load() corrects it
  const r = 7;
  for (let i = 0; i < snap.visible.length; i++) snap.visible[i] = false;
  for (let y = Math.max(0, hy - r); y <= Math.min(snap.h - 1, hy + r); y++) for (let x = Math.max(0, hx - r); x <= Math.min(snap.w - 1, hx + r); x++) {
    if (Math.hypot(x - hx, y - hy) > r) continue;
    const i = y * snap.w + x; snap.visible[i] = true; snap.seen[i] = snap.seen[i] || snap.tiles[i] !== "wall" || Math.hypot(x - hx, y - hy) < 3;
  }
}
