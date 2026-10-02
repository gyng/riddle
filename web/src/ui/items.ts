// Run-clear (the owner, 2026-10-02: "include item rarity colours + icons"): an item drawn wherever the client shows one — its icon
// (the packed `it_<kind>`, else a primitive glyph by its family: art never blocks the game) in a rim of its rarity, its name tinted
// the same, and a small glint from rare up. The rarity is the core's (`InvItem.rarity`, `item::rarity`); the colours are
// docs/ART_DIRECTION.md's rarity row (items.css).
import "../items.css";
import type { InvItem, Rarity } from "../engine/types";
import { h } from "./dom";
import { hasIcon } from "./skin";
import { kwHost } from "./tips";

export const RARITIES: Rarity[] = ["common", "uncommon", "rare", "epic", "legendary"];
export const rarityRank = (r?: Rarity): number => RARITIES.indexOf(r ?? "common");
/** The rarest of a list (common for none). */
export const bestRarity = (xs: { rarity?: Rarity }[]): Rarity => RARITIES[Math.max(0, ...xs.map((x) => rarityRank(x.rarity)))];

/** The family an item kind is drawn as when its own icon is not packed (display only: which picture, never what it does). */
const FAMILY: [RegExp, string, string][] = /* copy:none */ [
  [/^(dagger|sword|axe|bow|spear|mace)$/, "weapon", "⚔"],
  [/^(leather|mail|plate|scale)$/, "armour", "⛨"],
  [/^(potion|heal|strength|speed|invisibility|poison|caustic|confusion|fire|regen|resist_fire|clarity)$/, "potion", "⚗"],
  [/^(scroll|teleport|blink|fear|mapping|identify|enchant|darkness|summon_ally|aggravate|recall|silence|earthquake|mirror)$/, "scroll", "§"],
  [/^gold$/, "gold", "$"],
];
const familyOf = (kind: string): [string, string] => { const f = FAMILY.find(([re]) => re.test(kind)); return f ? [f[1], f[2]] : ["misc", "◇"]; };
/** The kind an icon is drawn for: the item's own (`sword`), a potion or scroll by its family (their icons are the flask and the roll). */
export function iconId(kind: string): string | null {
  const k = kind.replace(/ /g, "_");
  if (hasIcon(`it_${k}`)) return `it_${k}`;
  const [fam] = familyOf(k);
  return (fam === "potion" || fam === "scroll") && hasIcon(`it_${fam}`) ? `it_${fam}` : null;
}

type ItemLike = Pick<InvItem, "kind" | "label"> & { rarity?: Rarity };

/** The item's icon in its rarity rim (`.item-ico.r-rare`): the packed picture, else the family glyph. */
export function itemIcon(it: ItemLike, o: { size?: "s" | "m" | "l"; delay?: number } = {}): HTMLElement {
  const r = it.rarity ?? "common";
  const id = iconId(it.kind);
  const [fam, glyph] = familyOf(it.kind.replace(/ /g, "_"));
  const pic = id ? h("img", { class: "item-pic", src: `${import.meta.env.BASE_URL}ui/icons/${id}.png`, alt: "", draggable: "false", "aria-hidden": "true" })
    : h("span", { class: `item-pic glyph fam-${fam}`, "data-glyph": glyph, "aria-hidden": "true" });
  const el = h("span", { class: `item-ico r-${r} sz-${o.size ?? "m"}${rarityRank(r) >= 2 ? " glint" : ""}`, "data-rarity": r, "data-kind": it.kind, title: it.label }, pic);
  if (o.delay !== undefined) el.style.setProperty("--pop-delay", `${o.delay}ms`);
  return el;
}

/** The item's name tinted by its rarity. */
export const itemName = (it: ItemLike, text = it.label): HTMLElement => h("span", { class: `item-name r-${it.rarity ?? "common"}`, "data-rarity": it.rarity ?? "common" }, text);

/** An item as one chip: icon and tinted name. */
export const itemChip = (it: ItemLike, text = it.label, cls = ""): HTMLElement =>
  h("span", { class: `item-chip r-${it.rarity ?? "common"}${cls ? ` ${cls}` : ""}`, "data-rarity": it.rarity ?? "common" }, itemIcon(it, { size: "s" }), itemName(it, text));

/** A row of item icons, rarest first (the core sends them so), its rarity tooltip on the row (docs/TOOLTIPS.md: `rarity`). */
export function itemRow(xs: ItemLike[], o: { size?: "s" | "m" | "l"; pop?: boolean; max?: number } = {}): HTMLElement | null {
  if (!xs.length) return null;
  const shown = xs.slice(0, o.max ?? 6);
  // finds pop in by rarity: the commonest first, the rarest last (the beat lands on the best)
  const order = [...shown].map((x, i) => ({ x, i })).sort((a, b) => rarityRank(a.x.rarity) - rarityRank(b.x.rarity) || b.i - a.i);
  const delayOf = new Map(order.map((o2, k) => [o2.i, 380 + 160 * k]));
  const row = h("div", { class: `item-row${o.pop ? " pop" : ""}` }, ...shown.map((x, i) => itemIcon(x, { size: o.size, delay: o.pop ? delayOf.get(i) : undefined })));
  return kwHost(row, "rarity");
}
