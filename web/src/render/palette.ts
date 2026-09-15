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
  // Ramps are the art register's (art/ART.md "Register 2", art/make_tiles.py PALETTES): index 0
  // darkest → 7 lightest. The packed tiles are authored in exactly these colours, so quantisation
  // is lossless for tile art and only bites on fog/dim, sprites (50%) and fallbacks.
  warrens: ["#14120d", "#2e2a1c", "#4a4326", "#6b6a2f", "#8c7a3c", "#b09a5a", "#d4c58a", "#efe6c0"].map(hex), // olive/umber/bone
  fens:    ["#0c1416", "#1a2b2e", "#24443f", "#2f6a5a", "#4d8a72", "#6f9f8a", "#9dbfa8", "#d6e6da"].map(hex), // teal/moss/slate
  crypt:   ["#0b0a14", "#1c1a30", "#33304f", "#4f4d6d", "#77738c", "#a39fae", "#d3cfc9", "#f1ede0"].map(hex), // indigo/ash/bone
};

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

export const ENTITY_SIZE: Record<string, [number, number]> = {
  // sprite texels; 48 tall = 3 tiles, 32 tall = 2 tiles
  hero_fighter: [24, 48], hero_rogue: [24, 48],
  ogre: [24, 48], goblin_warlord: [32, 48], bloat_mother: [32, 48], lich: [32, 48],
};

// Max box (sprite texels) an atlas frame is fitted into, per kind; default 32×32.
export const ENTITY_BOX: Record<string, [number, number]> = {
  hero_fighter: [36, 48], hero_rogue: [36, 48],
  ogre: [40, 48], goblin_warlord: [48, 48], bloat_mother: [48, 48], lich: [48, 48],
  skeleton: [32, 40], wraith: [40, 40], captive: [32, 40],
};

export const KNOWN_ENTITY_KINDS = Object.keys(ENTITY_COLOURS);
export const TILE_IDS = ["floor", "wall", "door", "stairs_down", "stairs_up", "water", "chasm"] as const;
