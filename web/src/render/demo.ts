// Standalone viewer demo: a fake 24×24 floor and a looping scripted event stream.
// Open /render-demo.html (Vite dev). Not part of the app bundle.
import { createViewer } from "./index";
import type { Ev, Snapshot, Tile } from "./types";

const W = 24, H = 24;

function makeFloor(depth: number, biome: string): Snapshot {
  const tiles: Tile[] = new Array<Tile>(W * H).fill("wall");
  const set = (x: number, y: number, t: Tile) => { if (x >= 0 && y >= 0 && x < W && y < H) tiles[y * W + x] = t; };
  const room = (x0: number, y0: number, w: number, h: number) => { for (let y = y0; y < y0 + h; y++) for (let x = x0; x < x0 + w; x++) set(x, y, "floor"); };
  const hcor = (x0: number, x1: number, y: number) => { for (let x = Math.min(x0, x1); x <= Math.max(x0, x1); x++) set(x, y, "floor"); };
  const vcor = (y0: number, y1: number, x: number) => { for (let y = Math.min(y0, y1); y <= Math.max(y0, y1); y++) set(x, y, "floor"); };
  room(2, 2, 7, 6);      // NW room
  room(14, 2, 8, 5);     // NE room
  room(3, 13, 6, 8);     // SW room
  room(12, 12, 10, 9);   // SE room
  hcor(9, 13, 4); set(9, 4, "door"); set(13, 4, "door");
  vcor(8, 12, 5); set(5, 8, "door");
  vcor(7, 11, 18); set(18, 7, "door"); set(18, 11, "door");
  hcor(9, 11, 16); set(9, 16, "door"); set(11, 16, "door");
  // water pool in the SE room, chasm bite in the SW room
  for (let y = 14; y <= 17; y++) for (let x = 15; x <= 19; x++) if ((x - 17) ** 2 + (y - 15.5) ** 2 < 5) set(x, y, "water");
  for (let y = 18; y <= 20; y++) for (let x = 3; x <= 4; x++) set(x, y, "chasm");
  set(21, 19, "stairs_down");
  set(3, 3, "stairs_up");
  const all = new Array<boolean>(W * H).fill(false);
  return {
    depth, biome, w: W, h: H, tiles, seen: all.slice(), visible: all.slice(),
    overlays: [{ x: 15, y: 19, k: "gas", ttl: 60 }, { x: 16, y: 19, k: "gas", ttl: 60 }, { x: 16, y: 20, k: "gas", ttl: 50 }, { x: 7, y: 6, k: "fire", ttl: 30 }],
    hero: { id: 1, kind: "hero_fighter", x: 4, y: 5, hp: 30, max_hp: 30, tags: [], inv: [], class: "fighter", trait: "stubborn", weapon: "sword", armour: "leather" },
    entities: [
      { id: 2, kind: "rat", x: 7, y: 3, hp: 4, max_hp: 4, tags: [] },
      { id: 3, kind: "goblin_archer", x: 16, y: 3, hp: 8, max_hp: 8, tags: ["ranged"] },
      { id: 4, kind: "jackal", x: 20, y: 5, hp: 6, max_hp: 6, tags: ["pack", "fast"] },
      { id: 5, kind: "ogre", x: 14, y: 14, hp: 20, max_hp: 20, tags: ["heavy"] },
      biome === "fens"
        ? { id: 6, kind: "eel", x: 19, y: 15, hp: 6, max_hp: 6, tags: ["water"] }
        : { id: 6, kind: biome === "crypt" ? "ghoul" : "monkey", x: 20, y: 15, hp: 6, max_hp: 6, tags: [] },
      // companion (Addendum A): ally + cid, follows the hero
      { id: 20, kind: "jackal", x: 4, y: 6, hp: 6, max_hp: 6, tags: ["pack", "fast"], ally: true, cid: 1 },
    ],
    items: [
      { id: 10, x: 6, y: 6, kind: "heal", known: false, label: "potion?" },
      { id: 11, x: 15, y: 5, kind: "bow", known: true, label: "bow" },
      { id: 12, x: 19, y: 13, kind: "gold", known: true, label: "gold" },
      { id: 13, x: 8, y: 6, kind: "mapping", known: false, label: "scroll?" },
    ],
    alert: 0, turn: 0, loot: 0, run: { id: 1, heir: 1, started_turn: 0 },
  };
}

// Straight-line path from a (exclusive) to b (inclusive), one tile per step.
function line(a: [number, number], b: [number, number]): [number, number][] {
  const out: [number, number][] = [];
  let [x, y] = a;
  const dx = Math.abs(b[0] - x), dy = -Math.abs(b[1] - y), sx = x < b[0] ? 1 : -1, sy = y < b[1] ? 1 : -1;
  let err = dx + dy;
  while (x !== b[0] || y !== b[1]) {
    const e2 = 2 * err;
    if (e2 >= dy) { err += dy; x += sx; }
    if (e2 <= dx) { err += dx; y += sy; }
    out.push([x, y]);
  }
  return out;
}

// Timestamps are ticks (Addendum E): the hero acts every 10 ticks, the fast jackal every 7.
function script(biome: string): Ev[] {
  const tameKind = biome === "fens" ? "eel" : biome === "crypt" ? "ghoul" : "monkey";
  const ev: Ev[] = [];
  let t = 10;
  let heroAt: [number, number] = [4, 5];
  let compAt: [number, number] = [4, 6];
  // hero moves; the companion steps into the hero's previous tile
  const mv = (id: number, x: number, y: number, at = t) => {
    ev.push({ t: at, k: "move", id, x, y });
    if (id === 1) {
      if (compAt[0] !== heroAt[0] || compAt[1] !== heroAt[1]) { ev.push({ t: at, k: "move", id: 20, x: heroAt[0], y: heroAt[1] }); compAt = heroAt; }
      heroAt = [x, y];
    }
  };
  // hero walks east through the door towards the NE room; rat approaches
  const path: [number, number][] = [[5, 5], [6, 4], [7, 4], [8, 4], [9, 4], [10, 4], [11, 4], [12, 4], [13, 4], [14, 4]];
  const rat: [number, number][] = [[7, 4], [7, 5], [7, 5], [8, 5], [9, 5], [10, 5], [11, 5], [12, 5], [13, 5], [13, 4]];
  for (let i = 0; i < path.length; i++) {
    mv(1, path[i]![0], path[i]![1]);
    if (i === 1) ev.push({ t, k: "callout", text: "rat!" });
    mv(2, rat[i]![0], rat[i]![1]);
    t += 10;
  }
  // rat bites, hero kills rat
  ev.push({ t, k: "attack", src: 2, dst: 1, dmg: 2, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 2, hp: 28, cause: "rat" });
  t += 10;
  ev.push({ t, k: "attack", src: 1, dst: 2, dmg: 5, hit: true, verb: "attack" });
  ev.push({ t, k: "hurt", id: 2, dmg: 5, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 2, cause: "hero" });
  ev.push({ t, k: "callout", text: "got it" });
  t += 10;
  // archer telegraphs, then an arrow flies 1 tile per tick; jackal rushes (fast: every 7 ticks)
  ev.push({ t, k: "telegraph", id: 3, what: "draws" });
  mv(4, 19, 5); mv(4, 18, 5, t + 7);
  t += 10;
  const arrow = line([16, 3], [14, 4]);
  ev.push({ t, k: "projectile", src: 3, dst: 1, path: arrow });
  ev.push({ t: t + arrow.length, k: "attack", src: 3, dst: 1, dmg: 3, hit: true });
  ev.push({ t: t + arrow.length, k: "hurt", id: 1, dmg: 3, hp: 25, cause: "arrow" });
  mv(4, 17, 5, t + 4); mv(4, 16, 5, t + 11);
  t += 10;
  mv(1, 15, 4); mv(4, 15, 5, t + 8);
  ev.push({ t, k: "callout", text: "pack" });
  t += 10;
  ev.push({ t, k: "attack", src: 4, dst: 1, dmg: 0, hit: false });
  ev.push({ t: t + 5, k: "attack", src: 1, dst: 4, dmg: 6, hit: true });
  ev.push({ t: t + 5, k: "hurt", id: 4, dmg: 6, hp: 0, cause: "hero" });
  ev.push({ t: t + 5, k: "die", id: 4, cause: "hero" });
  t += 10;
  // pick up the bow, drink the unknown potion (fact), archer flees
  mv(1, 15, 5);
  ev.push({ t, k: "pickup", id: 1, item: "bow" });
  mv(3, 17, 3);
  t += 10;
  ev.push({ t, k: "use", item: "potion?", outcome: "heal" });
  ev.push({ t, k: "fact", fact: "item:murky=heal" });
  ev.push({ t, k: "callout", text: "heal" });
  t += 10;
  ev.push({ t, k: "overlay", x: 15, y: 18, ov: "gas", ttl: 40 });
  mv(1, 16, 4); mv(3, 18, 3);
  t += 10;
  mv(1, 17, 4);
  t += 10;
  ev.push({ t, k: "attack", src: 1, dst: 3, dmg: 8, hit: true });
  ev.push({ t, k: "hurt", id: 3, dmg: 8, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 3, cause: "hero" });
  t += 10;
  // a goblin conjurer appears and summons
  ev.push({ t, k: "spawn", e: { id: 7, kind: "goblin_conjurer", x: 21, y: 4, hp: 6, max_hp: 6, tags: ["caster"] } });
  t += 10;
  ev.push({ t, k: "telegraph", id: 7, what: "summons" });
  mv(1, 18, 4);
  t += 10;
  ev.push({ t, k: "spawn", e: { id: 8, kind: "goblin", x: 20, y: 4, hp: 5, max_hp: 5, tags: [] } });
  ev.push({ t, k: "callout", text: "blades!" });
  mv(1, 19, 4);
  t += 10;
  ev.push({ t, k: "attack", src: 8, dst: 1, dmg: 2, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 2, hp: 23, cause: "goblin" });
  t += 10;
  ev.push({ t, k: "attack", src: 1, dst: 8, dmg: 7, hit: true, verb: "shield_bash" });
  ev.push({ t, k: "hurt", id: 8, dmg: 7, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 8, cause: "hero" });
  t += 10;
  mv(1, 20, 4); mv(7, 21, 3);
  t += 10;
  ev.push({ t, k: "attack", src: 1, dst: 7, dmg: 7, hit: true });
  ev.push({ t, k: "hurt", id: 7, dmg: 7, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 7, cause: "hero" });
  t += 10;
  // south down the east corridor into the SE room: pool, ogre (slow: every 14 ticks), gas
  const south: [number, number][] = [[19, 5], [18, 6], [18, 7], [18, 8], [18, 9], [18, 10], [18, 11], [18, 12], [18, 13]];
  const ogre: [number, number][] = [[14, 13], [15, 13], [16, 13], [17, 13]];
  let ot = t + 50;
  for (const [x, y] of ogre) { mv(5, x, y, ot); ot += 14; }
  for (let i = 0; i < south.length; i++) {
    mv(1, south[i]![0], south[i]![1]);
    if (i === 6) ev.push({ t, k: "callout", text: "ogre" });
    t += 10;
  }
  t = Math.max(t, ot);
  ev.push({ t, k: "overlay", x: 19, y: 17, ov: "gas", ttl: 120 });
  ev.push({ t, k: "overlay", x: 20, y: 17, ov: "gas", ttl: 120 });
  ev.push({ t, k: "overlay", x: 20, y: 18, ov: "gas", ttl: 120 });
  ev.push({ t, k: "overlay", x: 21, y: 18, ov: "gas", ttl: 120 });
  ev.push({ t, k: "overlay", x: 14, y: 19, ov: "fire", ttl: 50 });
  ev.push({ t, k: "overlay", x: 15, y: 19, ov: "fire", ttl: 50 });
  ev.push({ t, k: "telegraph", id: 5, what: "winds up" });
  t += 10;
  ev.push({ t, k: "attack", src: 1, dst: 5, dmg: 6, hit: true });
  ev.push({ t, k: "hurt", id: 5, dmg: 6, hp: 14, cause: "hero" });
  t += 4;
  ev.push({ t, k: "attack", src: 5, dst: 1, dmg: 8, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 8, hp: 15, cause: "ogre" });
  ev.push({ t, k: "callout", text: "ouch" });
  t += 6;
  ev.push({ t, k: "telegraph", id: 5, what: "winds up" });
  mv(1, 19, 13);
  ev.push({ t, k: "pickup", id: 1, item: "gold" });
  t += 10;
  mv(1, 20, 13); mv(5, 18, 13, t + 4);
  t += 10;
  mv(1, 20, 14); mv(5, 19, 13, t + 8);
  t += 10;
  ev.push({ t, k: "attack", src: 1, dst: 5, dmg: 7, hit: true });
  ev.push({ t, k: "hurt", id: 5, dmg: 7, hp: 7, cause: "hero" });
  t += 10;
  ev.push({ t: t + 2, k: "attack", src: 5, dst: 1, dmg: 0, hit: false });
  ev.push({ t, k: "attack", src: 1, dst: 5, dmg: 7, hit: true });
  ev.push({ t, k: "hurt", id: 5, dmg: 7, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 5, cause: "hero" });
  t += 10;
  // tame the creature by the pool: first attempt fails (shake), second succeeds (flash, ring)
  ev.push({ t, k: "tame", id: 6, kind: tameKind, ok: false });
  t += 10;
  ev.push({ t, k: "tame", id: 6, kind: tameKind, ok: true });
  ev.push({ t, k: "fact", fact: `tamed:${tameKind}` });
  t += 10;
  ev.push({ t, k: "hatch", kind: "jackal" });
  // through the gas to the stairs; the new companion trails
  for (const [x, y] of [[21, 15], [21, 16], [20, 17], [21, 18]] as [number, number][]) {
    mv(1, x, y);
    mv(6, x - 1, y - 1);
    if (y >= 17) { ev.push({ t: t + 5, k: "hurt", id: 1, dmg: 3, hp: 12, cause: "gas" }); }
    t += 10;
  }
  ev.push({ t, k: "callout", text: "cough" });
  mv(1, 21, 19);
  ev.push({ t, k: "note", text: "stairs found" });
  t += 10;
  ev.push({ t, k: "callout", text: "down" });
  ev.push({ t, k: "descend", depth: 2, biome: "fens" });
  ev.sort((a, b) => a.t - b.t);
  return ev;
}

// Juice (docs/JUICE.md): `?busy=1&biome=burrows&depth=5` — a sustained melee in the SE room for frame-time measurement: six foes
// round the hero, fire and gas on the floor, arrows, a hit every ~3 ticks, a death and a respawn every ~25, for `ticks` ticks.
const BUSY_KINDS: Record<string, string[]> = {
  warrens: ["rat", "jackal", "goblin", "goblin_archer", "monkey", "goblin_conjurer"],
  burrows: ["goblin", "jackal", "goblin_archer", "goblin_captain", "monkey", "goblin_conjurer"],
  fens: ["eel", "bloat", "pink_jelly", "jackal", "goblin", "ogre"],
  crypt: ["skeleton", "ghoul", "wraith", "skeleton", "acolyte", "ghoul"],
  foundry: ["forge_imp", "iron_golem", "slag_crawler", "smith", "forge_imp", "bell_sentinel"],
  deep: ["lurker", "cave_troll", "siren", "eel", "lurker", "echo"],
  sanctum: ["warden", "mirror_shade", "acolyte", "sentinel", "warden", "echo"],
};
function busyFloor(depth: number, biome: string): Snapshot {
  const f = makeFloor(depth, biome);
  f.hero = { ...f.hero, x: 16, y: 16 };
  const kinds = BUSY_KINDS[biome] ?? BUSY_KINDS.warrens!;
  const ring: [number, number][] = [[15, 15], [17, 15], [15, 17], [17, 17], [14, 16], [18, 16]];
  f.entities = ring.map(([x, y], i) => ({ id: 100 + i, kind: kinds[i]!, x, y, hp: 12, max_hp: 12, tags: i === 3 ? ["boss"] : [] }));
  f.entities.push({ id: 20, kind: "jackal", x: 16, y: 18, hp: 6, max_hp: 6, tags: ["pack", "fast"], ally: true, cid: 1 });
  const ov = (k: "fire" | "gas", ps: number[][]): Snapshot["overlays"] => ps.map(([x, y]) => ({ x: x!, y: y!, k, ttl: 9999 }));
  f.overlays = [...ov("fire", [[13, 14], [13, 15], [14, 18], [19, 19], [20, 19]]), ...ov("gas", [[19, 13], [20, 13], [20, 14], [21, 14], [12, 19]])];
  f.tiles = f.tiles.map((t, i) => (t === "water" && i % 3 ? "floor" : t));
  return f;
}
function busyScript(biome: string, ticks: number): Ev[] {
  const kinds = BUSY_KINDS[biome] ?? BUSY_KINDS.warrens!;
  const ring: [number, number][] = [[15, 15], [17, 15], [15, 17], [17, 17], [14, 16], [18, 16]];
  const ev: Ev[] = [];
  const hp = new Map<number, number>(ring.map((_, i) => [100 + i, 12]));
  const at = new Map<number, [number, number]>(ring.map((p, i) => [100 + i, p]));
  let heroHp = 60, next = 200, r = 7;
  const rnd = () => ((r = (r * 1103515245 + 12345) & 0x7fffffff) / 0x7fffffff);
  for (let t = 5; t < ticks; t += 3) {
    const ids = [...hp.keys()];
    const id = ids[Math.floor(rnd() * ids.length)]!;
    if (rnd() < 0.55) {   // the hero strikes
      const dmg = 2 + Math.floor(rnd() * 5), left = Math.max(0, hp.get(id)! - dmg);
      ev.push({ t, k: "attack", src: 1, dst: id, dmg, hit: true, verb: "attack" }, { t, k: "hurt", id, dmg, hp: left, cause: "hero" });
      hp.set(id, left);
      if (left === 0) {
        ev.push({ t, k: "die", id, cause: "hero" });
        hp.delete(id);
        const p = at.get(id)!; at.delete(id);
        const nid = next++;
        ev.push({ t: t + 8, k: "spawn", e: { id: nid, kind: kinds[nid % kinds.length]!, x: p[0], y: p[1], hp: 12, max_hp: 12, tags: [] } });
        hp.set(nid, 12); at.set(nid, p);
      }
    } else if (rnd() < 0.3) {   // an arrow
      const p = at.get(id)!, dmg = 1 + Math.floor(rnd() * 3);
      const path = line([p[0] + 3, p[1] - 3], [16, 16]);
      ev.push({ t, k: "projectile", src: id, dst: 1, path });
      heroHp = Math.max(8, heroHp - dmg);
      ev.push({ t: t + path.length, k: "hurt", id: 1, dmg, hp: heroHp, cause: "arrow" });
    } else {   // a foe strikes
      const dmg = rnd() < 0.2 ? 0 : 1 + Math.floor(rnd() * 4);
      heroHp = dmg ? Math.max(8, heroHp - dmg) : heroHp;
      if (heroHp <= 10) heroHp = 60;
      ev.push({ t, k: "attack", src: id, dst: 1, dmg, hit: dmg > 0 });
      if (dmg) ev.push({ t, k: "hurt", id: 1, dmg, hp: heroHp, cause: kinds[0]! });
    }
    if (t % 60 === 5) ev.push({ t, k: "telegraph", id, what: "winds up" });
  }
  ev.sort((a, b) => a.t - b.t);
  return ev;
}

function main(): void {
  const canvas = document.getElementById("view") as HTMLCanvasElement;
  const q = new URLSearchParams(location.search);
  const viewer = createViewer(canvas, q.get("texels") ? { baseTexels: Number(q.get("texels")) } : {});
  if (q.get("busy") === "1") {
    const biome = q.get("biome") ?? "burrows", depth = Number(q.get("depth") ?? 5), ticks = Number(q.get("ticks") ?? 3000);
    const floor = busyFloor(depth, biome);
    floor.seen = floor.seen.map(() => false);
    viewer.load(floor);
    viewer.apply(busyScript(biome, ticks));
    viewer.setSpeed(Number(q.get("speed") ?? 1));
    if (q.get("fight") !== "0") viewer.setFrame("fight");
    const seekT = q.get("seek");
    if (seekT !== null) setTimeout(() => { viewer.seek(Number(seekT)); viewer.setSpeed(0); }, 600);
    window.addEventListener("resize", () => viewer.resize());
    (window as unknown as { viewer: unknown }).viewer = viewer;
    document.getElementById("bar")!.style.display = "none";
    const hud = document.getElementById("hud")!;
    if (q.get("hud") === "0") hud.hidden = true;
    return;
  }
  const biomes = ["warrens", "fens", "crypt"];
  let depth = 1;
  let floor = makeFloor(depth, biomes[0]!);
  viewer.load(floor);
  viewer.apply(script(floor.biome));

  const hud = document.getElementById("hud")!;
  const speeds = document.querySelectorAll<HTMLButtonElement>("button[data-speed]");
  const setSpeed = (n: number) => {
    viewer.setSpeed(n);
    speeds.forEach((b) => b.classList.toggle("on", Number(b.dataset.speed) === n));
  };
  speeds.forEach((b) => b.addEventListener("click", () => setSpeed(Number(b.dataset.speed))));
  document.getElementById("skip")!.addEventListener("click", () => viewer.skipToEvent());
  setSpeed(1);
  window.addEventListener("resize", () => viewer.resize());

  let lastLog = 0;
  let restartAt = 0;
  const loop = (now: number) => {
    requestAnimationFrame(loop);
    const s = viewer.stats();
    const f1 = (n: number) => (Number.isNaN(n) ? "-" : n.toFixed(2));
    hud.textContent = `D${depth} ${floor.biome}  k=${s.k} dpr=${s.dpr}  dev ${s.device.join("×")}  env ${s.envTexels.join("×")}  target ${s.target.join("×")}  tris ${s.triangles}  queue ${s.pending}  tick ${s.tick}\n`
      + `frame ms (cpu) ${f1(s.cpuMs)} p95 ${f1(s.cpuP95)} (build ${f1(s.buildMs)}) / gpu ms ${f1(s.gpuMs)} p95 ${f1(s.gpuP95)}${s.gpuTimer ? "" : " (no timer ext)"} / calls ${s.calls} / fps ${Number.isNaN(s.fps) ? "-" : s.fps.toFixed(0)}`;
    if (now - lastLog > 2000) { lastLog = now; console.log(`[render-demo] cpu=${f1(s.cpuMs)}ms gpu=${f1(s.gpuMs)}ms calls=${s.calls} fps=${Number.isNaN(s.fps) ? "-" : s.fps.toFixed(0)} target=${s.target.join("x")} k=${s.k}`); }
    if (viewer.idle()) {
      if (restartAt === 0) restartAt = now + 1200;
      else if (now > restartAt) {
        restartAt = 0;
        depth = (depth % 3) + 1;
        floor = makeFloor(depth, biomes[depth - 1]!);
        viewer.load(floor);
        viewer.apply(script(floor.biome));
      }
    }
  };
  requestAnimationFrame(loop);
  // debugging hooks for headless drivers
  (window as unknown as { viewer: unknown; makeFloor: unknown; script: unknown }).viewer = viewer;
  (window as unknown as { makeFloor: unknown }).makeFloor = makeFloor;
  (window as unknown as { script: unknown }).script = script;
}

main();
