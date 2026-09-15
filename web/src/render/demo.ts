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
    overlays: [{ x: 15, y: 19, k: "gas", ttl: 6 }, { x: 16, y: 19, k: "gas", ttl: 6 }, { x: 16, y: 20, k: "gas", ttl: 5 }, { x: 7, y: 6, k: "fire", ttl: 3 }],
    hero: { id: 1, kind: "hero_fighter", x: 4, y: 5, hp: 30, max_hp: 30, tags: [], inv: [], class: "fighter", trait: "stubborn", weapon: "sword", armour: "leather" },
    entities: [
      { id: 2, kind: "rat", x: 7, y: 3, hp: 4, max_hp: 4, tags: [] },
      { id: 3, kind: "goblin_archer", x: 16, y: 3, hp: 8, max_hp: 8, tags: ["ranged"] },
      { id: 4, kind: "jackal", x: 20, y: 5, hp: 6, max_hp: 6, tags: ["pack", "fast"] },
      { id: 5, kind: "ogre", x: 14, y: 14, hp: 20, max_hp: 20, tags: ["heavy"] },
      biome === "fens"
        ? { id: 6, kind: "eel", x: 17, y: 15, hp: 6, max_hp: 6, tags: ["water"] }
        : { id: 6, kind: biome === "crypt" ? "wraith" : "captive", x: 13, y: 19, hp: 6, max_hp: 6, tags: [] },
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

function script(): Ev[] {
  const ev: Ev[] = [];
  let t = 1;
  const mv = (id: number, x: number, y: number) => ev.push({ t, k: "move", id, x, y });
  // hero walks east through the door towards the NE room; rat approaches
  const path: [number, number][] = [[5, 5], [6, 4], [7, 4], [8, 4], [9, 4], [10, 4], [11, 4], [12, 4], [13, 4], [14, 4]];
  const rat: [number, number][] = [[7, 4], [7, 5], [7, 5], [8, 5], [9, 5], [10, 5], [11, 5], [12, 5], [13, 5], [13, 4]];
  for (let i = 0; i < path.length; i++) {
    mv(1, path[i]![0], path[i]![1]);
    if (i === 1) ev.push({ t, k: "callout", text: "rat!" });
    mv(2, rat[i]![0], rat[i]![1]);
    t++;
  }
  // rat bites, hero kills rat
  ev.push({ t, k: "attack", src: 2, dst: 1, dmg: 2, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 2, hp: 28, cause: "rat" });
  t++;
  ev.push({ t, k: "attack", src: 1, dst: 2, dmg: 5, hit: true, verb: "attack" });
  ev.push({ t, k: "hurt", id: 2, dmg: 5, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 2, cause: "hero" });
  ev.push({ t, k: "callout", text: "got it" });
  t++;
  // archer telegraphs, fires; jackal rushes (fast: 2 moves a turn)
  ev.push({ t, k: "telegraph", id: 3, what: "draws" });
  mv(4, 19, 5); mv(4, 18, 5);
  t++;
  ev.push({ t, k: "attack", src: 3, dst: 1, dmg: 3, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 3, hp: 25, cause: "arrow" });
  mv(4, 17, 5); mv(4, 16, 5);
  t++;
  mv(1, 15, 4); mv(4, 15, 5);
  ev.push({ t, k: "callout", text: "pack" });
  t++;
  ev.push({ t, k: "attack", src: 4, dst: 1, dmg: 0, hit: false });
  ev.push({ t, k: "attack", src: 1, dst: 4, dmg: 6, hit: true });
  ev.push({ t, k: "hurt", id: 4, dmg: 6, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 4, cause: "hero" });
  t++;
  // pick up the bow, drink the unknown potion (fact), archer flees
  mv(1, 15, 5);
  ev.push({ t, k: "pickup", id: 1, item: "bow" });
  mv(3, 17, 3);
  t++;
  ev.push({ t, k: "use", item: "potion?", outcome: "heal" });
  ev.push({ t, k: "fact", fact: "item:murky=heal" });
  ev.push({ t, k: "callout", text: "heal" });
  t++;
  // ogre winds up from far away (visible when we get there); gas spreads
  ev.push({ t, k: "overlay", x: 15, y: 18, ov: "gas", ttl: 4 });
  mv(1, 16, 4); mv(3, 18, 3);
  t++;
  ev.push({ t, k: "attack", src: 1, dst: 3, dmg: 8, hit: true });
  ev.push({ t, k: "hurt", id: 3, dmg: 8, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 3, cause: "hero" });
  t++;
  // spawn a goblin conjurer that summons
  ev.push({ t, k: "spawn", e: { id: 7, kind: "goblin_conjurer", x: 21, y: 4, hp: 6, max_hp: 6, tags: ["caster"] } });
  t++;
  ev.push({ t, k: "telegraph", id: 7, what: "summons" });
  mv(1, 17, 4);
  t++;
  ev.push({ t, k: "spawn", e: { id: 8, kind: "goblin", x: 20, y: 4, hp: 5, max_hp: 5, tags: [] } });
  ev.push({ t, k: "callout", text: "blades!" });
  mv(1, 18, 4);
  t++;
  ev.push({ t, k: "attack", src: 8, dst: 1, dmg: 2, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 2, hp: 23, cause: "goblin" });
  t++;
  ev.push({ t, k: "attack", src: 1, dst: 8, dmg: 7, hit: true, verb: "shield_bash" });
  ev.push({ t, k: "hurt", id: 8, dmg: 7, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 8, cause: "hero" });
  t++;
  mv(1, 19, 4); mv(7, 21, 3);
  t++;
  ev.push({ t, k: "attack", src: 1, dst: 7, dmg: 7, hit: true });
  ev.push({ t, k: "hurt", id: 7, dmg: 7, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 7, cause: "hero" });
  t++;
  // south down the east corridor into the SE room: pool, ogre, gas
  const south: [number, number][] = [[18, 5], [18, 6], [18, 7], [18, 8], [18, 9], [18, 10], [18, 11], [18, 12], [18, 13]];
  const ogre: [number, number][] = [[14, 14], [14, 14], [14, 14], [14, 14], [14, 14], [14, 13], [15, 13], [16, 13], [17, 13]];
  for (let i = 0; i < south.length; i++) {
    mv(1, south[i]![0], south[i]![1]);
    if (i >= 5) mv(5, ogre[i]![0], ogre[i]![1]);
    if (i === 6) ev.push({ t, k: "callout", text: "ogre" });
    t++;
  }
  ev.push({ t, k: "overlay", x: 19, y: 17, ov: "gas", ttl: 12 });
  ev.push({ t, k: "overlay", x: 20, y: 17, ov: "gas", ttl: 12 });
  ev.push({ t, k: "overlay", x: 20, y: 18, ov: "gas", ttl: 12 });
  ev.push({ t, k: "overlay", x: 21, y: 18, ov: "gas", ttl: 12 });
  ev.push({ t, k: "overlay", x: 14, y: 19, ov: "fire", ttl: 5 });
  ev.push({ t, k: "overlay", x: 15, y: 19, ov: "fire", ttl: 5 });
  ev.push({ t, k: "telegraph", id: 5, what: "winds up" });
  t++;
  ev.push({ t, k: "attack", src: 1, dst: 5, dmg: 6, hit: true });
  ev.push({ t, k: "hurt", id: 5, dmg: 6, hp: 14, cause: "hero" });
  t++;
  ev.push({ t, k: "attack", src: 5, dst: 1, dmg: 8, hit: true });
  ev.push({ t, k: "hurt", id: 1, dmg: 8, hp: 15, cause: "ogre" });
  ev.push({ t, k: "callout", text: "ouch" });
  t++;
  ev.push({ t, k: "telegraph", id: 5, what: "winds up" });
  mv(1, 19, 13);
  ev.push({ t, k: "pickup", id: 1, item: "gold" });
  t++;
  mv(1, 20, 13); mv(5, 18, 13);
  t++;
  mv(1, 20, 14); mv(5, 19, 13);
  t++;
  ev.push({ t, k: "attack", src: 1, dst: 5, dmg: 7, hit: true });
  ev.push({ t, k: "hurt", id: 5, dmg: 7, hp: 7, cause: "hero" });
  t++;
  ev.push({ t, k: "attack", src: 5, dst: 1, dmg: 0, hit: false });
  ev.push({ t, k: "attack", src: 1, dst: 5, dmg: 7, hit: true });
  ev.push({ t, k: "hurt", id: 5, dmg: 7, hp: 0, cause: "hero" });
  ev.push({ t, k: "die", id: 5, cause: "hero" });
  t++;
  // through the gas to the stairs
  for (const [x, y] of [[20, 15], [20, 16], [20, 17], [21, 18]] as [number, number][]) {
    mv(1, x, y);
    if (y >= 17) { ev.push({ t, k: "hurt", id: 1, dmg: 3, hp: 12, cause: "gas" }); }
    t++;
  }
  ev.push({ t, k: "callout", text: "cough" });
  mv(1, 21, 19);
  ev.push({ t, k: "note", text: "stairs found" });
  t++;
  ev.push({ t, k: "callout", text: "down" });
  ev.push({ t, k: "descend", depth: 2, biome: "fens" });
  return ev;
}

function main(): void {
  const canvas = document.getElementById("view") as HTMLCanvasElement;
  const viewer = createViewer(canvas);
  const biomes = ["warrens", "fens", "crypt"];
  let depth = 1;
  let floor = makeFloor(depth, biomes[0]!);
  viewer.load(floor);
  viewer.apply(script());

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
    hud.textContent = `D${depth} ${floor.biome}  k=${s.k} dpr=${s.dpr}  dev ${s.device.join("×")}  env ${s.envTexels.join("×")}  target ${s.target.join("×")}  calls ${s.calls}  tris ${s.triangles}  queue ${s.pending}`;
    if (now - lastLog > 2000) { lastLog = now; console.log(`[render-demo] draw calls=${s.calls} target=${s.target.join("x")} k=${s.k}`); }
    if (viewer.idle()) {
      if (restartAt === 0) restartAt = now + 1200;
      else if (now > restartAt) {
        restartAt = 0;
        depth = (depth % 3) + 1;
        floor = makeFloor(depth, biomes[depth - 1]!);
        viewer.load(floor);
        viewer.apply(script());
      }
    }
  };
  requestAnimationFrame(loop);
  (window as unknown as { viewer: unknown }).viewer = viewer;
}

main();
