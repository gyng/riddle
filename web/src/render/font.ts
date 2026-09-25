// 3×5 bitmap font, drawn from bitmask arrays (no image assets). Each glyph is 5 rows of 3 bits,
// MSB = left column. Rendered into 5×7 atlas cells with a 1-texel dark outline (see atlas.ts).

const G: Record<string, string> = {
  // Cut 23 (AJ read `IT SHELLS` for `it swells`): W carries its bar one row lower than H (`101 101 101 111 101`), so the two differ at a glance
  A: "010101111101101", B: "110101110101110", C: "011100100100011", D: "110101101101110",
  E: "111100110100111", F: "111100110100100", G: "011100101101011", H: "101101111101101",
  I: "111010010010111", J: "001001001101010", K: "101101110101101", L: "100100100100111",
  M: "101111111101101", N: "110101101101101", O: "010101101101010", P: "110101110100100",
  Q: "010101101110011", R: "110101110101101", S: "011100010001110", T: "111010010010010",
  U: "101101101101011", V: "101101101101010", W: "101101101111101", X: "101101010101101",
  Y: "101101010010010", Z: "111001010100111",
  "0": "010101101101010", "1": "010110010010111", "2": "110001010100111", "3": "110001010001110",
  "4": "101101111001001", "5": "111100110001110", "6": "011100110101010", "7": "111001010010010",
  "8": "010101010101010", "9": "010101011001110",
  "!": "010010010000010", "?": "110001010000010", ".": "000000000000010", ",": "000000000010100",
  "-": "000000111000000", "+": "000010111010000", ":": "000010000010000", "'": "010010000000000",
  "/": "001001010100100", "%": "101001010100101", "*": "101010111010101", "<": "001010100010001",
  ">": "100010001010100", " ": "000000000000000",
  // QA 1a2a4a9 (P: `ATTACK ? NO PATH` — the core's `attack ✗ no path` fell back to `?`): the refusal's cross, the times sign, the coin, equals
  "✗": "000101010101000", "×": "000101010101000", "$": "011110010011110", "=": "000111000111000",
};

export const FONT_W = 3;
export const FONT_H = 5;
export const FONT_CELL_W = 5; // glyph + 1 outline texel each side
export const FONT_CELL_H = 7;
export const FONT_ADVANCE = 4; // cells overlap by one outline texel

export function glyphBits(ch: string): string {
  const c = ch.toUpperCase();
  return G[c] ?? G["?"]!;
}

export function hasGlyph(ch: string): boolean {
  return ch.toUpperCase() in G;
}
