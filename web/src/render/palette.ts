// Biome palettes: 8-colour ramps, index 0 = darkest (clear colour / fog floor), 7 = lightest.
// Uploaded as a vec3[8] uniform; a biome swap is one uniform update.

export type Rgb = [number, number, number];
export type Palette = Rgb[];

const hex = (h: string): Rgb => [
  parseInt(h.slice(1, 3), 16) / 255,
  parseInt(h.slice(3, 5), 16) / 255,
  parseInt(h.slice(5, 7), 16) / 255,
];

export const PALETTES: Record<string, Palette> = {
  // Ramps are the art register's (art/make_tiles.py PALETTES, atlas.json meta.palettes is the authority at runtime): index 0 darkest →
  // 7 lightest. Art direction phase 2 (docs/ART_DIRECTION.md §2): the named palette with the place's tint — INK, UMBRA, DUSK', ·,
  // MOON', ·, MIST, BONE (DUSK and MOON lean toward the tint). These are the fallbacks until the atlas loads.
  warrens: ["#0d0c14", "#1c1b2b", "#3e383a", "#4f505a", "#61697a", "#8292a8", "#a4bcd6", "#eadfc5"].map(hex),
  burrows: ["#0d0c14", "#1c1b2b", "#473e37", "#595757", "#6b7076", "#8896a6", "#a4bcd6", "#eadfc5"].map(hex),
  fens:    ["#0d0c14", "#1c1b2b", "#2c4951", "#3c6272", "#4d7c94", "#789cb5", "#a4bcd6", "#eadfc5"].map(hex),
  crypt:   ["#0d0c14", "#1c1b2b", "#2d2f4f", "#3e4770", "#4e5f92", "#798db4", "#a4bcd6", "#eadfc5"].map(hex),
};

// Replace ramps at runtime (atlas.json meta.palettes is the authority when present).
export function setPalettes(p: Record<string, string[]>): void {
  for (const [biome, cols] of Object.entries(p)) if (cols.length >= 2) PALETTES[biome] = cols.slice(0, 8).map(hex);
}

/** Cut 16 §3: biomes drawn with another biome's tile art, recoloured index for index into their own ramp. */
export const TILE_ALIAS: Record<string, string> = { burrows: "warrens" };

export function paletteFor(biome: string): Palette {
  return PALETTES[biome] ?? PALETTES.warrens!;
}

export function css(c: Rgb, a = 1): string {
  return `rgba(${Math.round(c[0] * 255)},${Math.round(c[1] * 255)},${Math.round(c[2] * 255)},${a})`;
}

// Per-kind fill colours for the procedural entity silhouettes (sprite density). Not palette-locked:
// sprites are only 50% quantised in the blit, so these keep some hue identity across biomes.
export const ENTITY_COLOURS: Record<string, [string, string]> = {
  // [fill, letter]
  hero_fighter:   ["#e8dcc0", "#8c2f1f"],
  hero_rogue:     ["#c9c2b0", "#2f4d8c"],
  hero_ranger:    ["#b8c49a", "#3a5a2a"],
  hero_caster:    ["#c8b8e0", "#4a2a7a"],
  rat:            ["#8a7460", "#2a1f16"],
  jackal:         ["#c29a5a", "#3a2a12"],
  goblin:         ["#6d9a3c", "#1f3312"],
  goblin_archer:  ["#7fa04a", "#5a3a1a"],
  goblin_conjurer:["#7a5c9e", "#2a1a3e"],
  monkey:         ["#9a6a3a", "#2c1a0a"],
  ogre:           ["#7d8a6a", "#1e2418"],
  bloat:          ["#b7c25a", "#4a4a12"],
  pink_jelly:     ["#e38ab0", "#7a2a50"],
  eel:            ["#4a7fb0", "#10263e"],
  skeleton:       ["#e6e0cc", "#3a3630"],
  ghoul:          ["#8fa08a", "#26301f"],
  wraith:         ["#9a8cc8", "#2a1f4a"],
  captive:        ["#d8c8b8", "#3a5a8c"],
  goblin_warlord: ["#4f7a2e", "#a8241e"],
  bloat_mother:   ["#c8c04a", "#5a5a10"],
  lich:           ["#5a4f9a", "#e8e0c8"],
};

/** gfx round 7 (every rater, rounds 0–6: "the hero is 2–3 tiles tall, hides the foe he fights"): the entities' runtime size against the
 *  tiles — a loaded sprite's height (atlas `texel_h`, or its box fit) and a procedural one's `ENTITY_SIZE` times this, cut down by area
 *  (atlas.ts `putDown`, still on the sprite grid). Dev: `?sprite=0.5`. */
export const SPRITE_SCALE: number = (() => {
  try { if (import.meta.env?.DEV) { const q = Number(new URLSearchParams(location.search).get("sprite")); if (q > 0 && q <= 1) return q; } } catch { /* no location */ }
  return 0.5;   // two fresh pick raters (O, P), four candidates blind: 0.5 at the watch's zoom unchanged, both first ("one world at one density")
})();

/** gfx round 10 (the coordinator: "bosses read as bosses"; r9 raters: the break and the fall "read small" at half size): a boss's own scale */
export const BOSS_SCALE: number = 0.75;
export const BOSS_KINDS = new Set(["goblin_warlord", "bloat_mother", "lich", "foundry_master", "lurker_queen", "mirror_king"]);
/** gfx round 10 (raters, every round: "wraiths are grey static blobs"): the ethereal kinds — drawn half-translucent, cold, with their own
 *  faint light (index.ts `L.ghosts`; the `low` tier draws them as any sprite) */
export const ETHEREAL = new Set(["wraith", "mirror_shade", "echo", "spectral_blade", "spectral_hound", "siren"]);
export const spriteScale = (kind: string): number => (BOSS_KINDS.has(kind.replace(/_dead$/, "")) ? BOSS_SCALE : SPRITE_SCALE);

export const ENTITY_SIZE: Record<string, [number, number]> = {
  // sprite texels; 48 tall = 3 tiles, 32 tall = 2 tiles
  hero_fighter: [24, 48], hero_rogue: [24, 48], hero_ranger: [24, 48], hero_caster: [24, 48],
  ogre: [24, 48], goblin_warlord: [32, 48], bloat_mother: [32, 48], lich: [32, 48],
};

// Max box (sprite texels) an atlas frame is fitted into, per kind; default 32×32.
export const ENTITY_BOX: Record<string, [number, number]> = {
  hero_fighter: [36, 48], hero_rogue: [36, 48], hero_ranger: [36, 48], hero_caster: [36, 48],
  ogre: [40, 48], goblin_warlord: [48, 48], bloat_mother: [48, 48], lich: [48, 48],
  skeleton: [32, 40], wraith: [40, 40], captive: [32, 40],
};

export const KNOWN_ENTITY_KINDS = Object.keys(ENTITY_COLOURS);
export const TILE_IDS = ["floor", "wall", "door", "stairs_down", "stairs_up", "water", "chasm"] as const;
