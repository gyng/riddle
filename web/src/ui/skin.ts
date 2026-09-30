// Cut 17 §1: the UI skin — 9-slice frames and command icons packed by tools/ui-skin.py into web/public/ui/, listed in
// skin.json. Only listed files are ever referenced (a missing PNG never 404s: art never blocks the game). Frames are CSS
// `border-image` / `background-image` rules gated on a `skin-<frame>` class on <html>; without it every element keeps its flat
// CSS fallback. Icons are <img> when present; absent, a glyph drawn by CSS (`data-glyph`), so a button's text stays its word.
import skin from "./skin.json";
import { h } from "./dom";

type Skin = { frames: Record<string, { w: number; h: number; slice?: number }>; icons: string[]; portraits?: string[]; backdrops?: string[]; deco?: string[]; foes?: string[]; fx?: string[] };
const S = skin as Skin;
const ICONS = new Set(S.icons);
const PORTRAITS = new Set(S.portraits ?? []);

/** Art pass: a painted headshot (`hero_<class>`, `pet_<kind>`, `captive`, `boss_<kind>`) when packed, else null (the caller keeps
 *  its fallback — the atlas sprite crop). */
export const portraitSrc = (id: string): string | null => (PORTRAITS.has(id) ? `/ui/portraits/${id}.webp` : null);

/** gfx round 10 (raters: "show the killer"): a foe's painted head and shoulders (tools/foe-portraits.py) by kind, else null */
const FOES = new Set(S.foes ?? []);
export const foeSrc = (kind: string): string | null => { const k = kind.replace(/ /g, "_"); return FOES.has(k) ? `/ui/foes/${k}.webp` : null; };

export const hasFrame = (name: string): boolean => name in S.frames;
export const hasIcon = (name: string): boolean => ICONS.has(name);

/** Sets `skin-<frame>` on <html> for every packed frame (the stylesheet's frame rules key on them). */
export function applySkin(): void {
  const root = document.documentElement;
  for (const name of Object.keys(S.frames)) root.classList.add(`skin-${name.replace(/_/g, "-")}`);
  for (const name of S.backdrops ?? []) root.classList.add(`skin-bd-${name.replace(/_/g, "-")}`);   // gfx round 1: painted backdrops (a place's back wall)
  for (const name of S.deco ?? []) root.classList.add(`skin-deco-${name.replace(/_/g, "-")}`);
  // gfx round 19 (raters: the report's strip "starts as an empty black frame" — the scroll's frame decoded on first use): the frames and
  // backdrops decode at boot
  if (typeof Image !== "undefined") for (const u of [...Object.keys(S.frames).map((n) => `/ui/frames/${n}.png`), ...(S.backdrops ?? []).map((n) => `/ui/backdrops/${n}.webp`)]) { const im = new Image(); im.decoding = "async"; im.src = u; void im.decode?.().catch(() => undefined); }   // gfx round 4: the frame's carved pillar, the camp's braziers
}

/** An icon: the packed PNG, or a CSS-drawn glyph (no text node, so `textContent` is the caller's label alone). */
export function icon(name: string, glyph = ""): HTMLElement {
  if (hasIcon(name)) return h("img", { class: `ico ico-${name}`, src: `/ui/icons/${name}.png`, alt: "", draggable: "false", "aria-hidden": "true" });
  return h("span", { class: `ico glyph ico-${name}`, "data-glyph": glyph, "aria-hidden": "true" });
}

/** gfx round 1 (camp.png: each rule tablet ends in its action's icon plaque): the icon for a rule's action — a family per icon; null
 *  when the family's icon is not packed (the tablet then has no plaque). */
const VERB_ICON: [RegExp, string][] = [
  [/^(attack|cleave|backstab|flank|ambush|feint)$/, "v_attack"], [/^(shield_bash|bulwark|ward|taunt)$/, "v_shield"],
  [/^(drink|second_wind)$/, "v_drink"], [/^read$/, "v_read"], [/^(throw|trap)$/, "v_throw"], [/^(retreat|kite)$/, "v_retreat"],
  [/^back_corridor$/, "v_corridor"], [/^descend$/, "v_descend"], [/^return$/, "bail"], [/^bank$/, "gold"], [/^(rest|hold|wait)$/, "v_rest"],
  [/^(shoot|volley|double_shot|mark)$/, "v_shoot"], [/^(bolt|nova|blink|slow|drain|burst|split)$/, "v_magic"],
  [/^(vanish|smoke|shadowstep|steal)$/, "v_shadow"], [/^(explore|pick_up|free_captive)$/, "v_explore"], [/^(tame|recall|send|follow)$/, "party"],
];
export function verbIcon(v: string): string | null {
  const id = VERB_ICON.find(([re]) => re.test(v))?.[1];
  return id && hasIcon(id) ? id : null;
}
