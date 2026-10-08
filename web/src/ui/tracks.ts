// Cut 30 §4 — the four tracks (docs/CUT30.md): character · items · scale · town, each an icon and its name. Their panel became the works
// sheet's branches in Cut 30.5 (ui/works.ts; the stages are the tree's `stage` nodes). The report leads with what grew on each track
// (`ReturnReport.grew`, `grewBlock`). No tutorial text: the stages are the copy.
import "../cut30.css";
import type { GrewLine, Lineage, ReturnReport } from "../engine/types";
import { h } from "./dom";
import { icon, packageIcon, portraitSrc } from "./skin";
import { unitLabel } from "./unit-icon";
import { kwText } from "./tips";

export const TRACK_IDS = ["character", "items", "scale", "town"] as const;
/** Each track's icon (a packed one, else a CSS glyph). */
const ICON: Record<string, [string, string]> = { character: ["", "⚔"], items: ["forge", "⚒"], scale: ["depth", "⇣"], town: ["camp", "⌂"] };
/** `hero`: the hero's painted headshot id (`hero_fighter_male`) — the character track's face (the paw read as the pets'). */
export const trackIcon = (id: string, hero?: string): HTMLElement => {
  const src = id === "character" && hero ? portraitSrc(hero) ?? portraitSrc(hero.replace(/_(male|female|cat)$/, "")) : null;
  if (src) return h("img", { class: "ico track-hero", src, alt: "", draggable: "false", "aria-hidden": "true" });
  const [ic, gl] = ICON[id] ?? ["", "✦"]; return icon(ic, gl);
};
/** The lineage's headshot id for the character track. */
export const heroFace = (L: Pick<Lineage, "class" | "look">): string => `hero_${L.class}${L.look ? `_${L.look}` : ""}`;
/** The track's name on its row (its own word). */
/* copy:label */
const NAME: Record<string, string> = { character: "hero", items: "items", scale: "scale", town: "town" };
export const trackName = (id: string): string => NAME[id] ?? id;

/** The order on a track's line: a new stage, then a package that arrived (or the record, the purse), a level, then xp (`grewRank`). */
export function grewRank(what: string): number {
  if (/^(xp|interest)$/.test(what)) return 3;
  if (/(^| )L\d+$|^a companion$|^blacksmith step$|^a find$/.test(what)) return 2;
  if (/^\+|^best D\d+$|^drilled$/.test(what)) return 1;
  return 0;
}
/** A fact's words, case and order aside (`built blacksmith` ~ `blacksmith`, `DRILLED · Warlord` ~ `Warlord drilled`, `STEADY L2` ~ `Steady L2`). */
const factKey = (s: string): string => s.toLowerCase().replace(/^built /, "").split(/[\s·]+/).filter(Boolean).sort().join(" ");
/** A track's line: at most `max` short items (~`chars` characters, one line at 400 px), the overflow folded into `+N more`. */
export function grewItems(items: string[], max = 4, chars = 34): string[] {
  if (items.length <= max && items.join(" · ").length <= chars) return items;
  const out: string[] = [];
  for (const w of items) {
    const more = items.length - out.length - 1;
    if (out.length && (out.length >= max - 1 || [...out, w].join(" · ").length + (more ? 9 : 0) > chars)) break;
    out.push(w);
  }
  return out.length < items.length ? [...out, /* copy:callout */ `+${items.length - out.length} more`] : out;
}

/** The report's lead (§4): what grew on each track over the absence — one short line per track (`hero · a class · pets · +2 more`), the
 *  stage first and xp last, and the packages' beats as plaques; a fact a plaque carries is not on a line too. Null when nothing grew (an
 *  old core: no `grew`). */
export function grewBlock(r: Pick<ReturnReport, "grew" | "packages">, hero?: string, highlighted: string[] = []): HTMLElement | null {
  const g: GrewLine[] = r.grew ?? [];
  const beats = (r.packages ?? []).filter((b) => !highlighted.includes(b));
  if (!g.length && !beats.length) return null;
  const shown = beats.slice(0, 5);
  const onPlaque = new Set([...shown, ...highlighted].map(factKey));
  const by = new Map<string, string[]>();
  for (const x of g) {
    if (onPlaque.has(factKey(x.what))) continue;
    const l = by.get(x.track) ?? [];
    if (!l.includes(x.what)) l.push(x.what);
    by.set(x.track, l);
  }
  for (const [t, l] of by) by.set(t, grewItems(l.map((w, i) => [w, i] as const).sort((a, b) => grewRank(a[0]) - grewRank(b[0]) || a[1] - b[1]).map(([w]) => w)));
  const ids = [...TRACK_IDS.filter((t) => by.has(t)), ...[...by.keys()].filter((t) => !(TRACK_IDS as readonly string[]).includes(t))];
  return h("div", { class: "grew" },
    ...ids.map((id, i) => h("div", { class: "grew-line reveal", "data-track": id, style: `animation-delay:${0.12 * i}s` }, h("span", { class: "track-ico" }, trackIcon(id, hero)),
      h("small", { class: "track-name dim" }, trackName(id)), h("span", { class: "grew-what num" }, ...kwText((by.get(id) ?? []).join(" · "))))),
    shown.length ? h("div", { class: "beats" }, ...shown.map((b, i) => h("span", { class: `beat-plaque reveal${/^QUEST DONE/.test(b) ? " quest" : /^DRILLED/.test(b) ? " drill" : ""}`, style: `animation-delay:${0.15 * (i + ids.length)}s` }, b)),
      beats.length > 5 ? h("small", { class: "beat-more dim" }, /* copy:callout */ `+${beats.length - 5} more`) : "") : "");
}

/** Already-earned core level/drill beats, surfaced without exposing routine details. */
export const trainingBeats = (beats: string[] | undefined): string[] => (beats ?? []).filter((b) => /^DRILLED\b/.test(b) || / L\d+$/.test(b));
// Reverse the core's sifter::boss_short labels, not the selected hero's knowledge.
const TRAINING_BOSSES: Record<string, string> = { Warlord: "goblin_warlord", Mother: "bloat_mother", Lich: "lich", Master: "foundry_master", Queen: "lurker_queen", King: "mirror_king" };
/** Blind 77030eb (B: "tapping 'Training · Warlord tactic' did nothing"): with `open` a badge is a button onto the tactics panel, where
 *  the drill (On / Off) and the level stand. */
type OpenTraining = (anchor: HTMLElement, beat: string) => void;
function trainingBadge(beat: string, open?: OpenTraining): HTMLElement {
  const drill = /^DRILLED · (.+)$/.exec(beat);
  const boss = drill ? TRAINING_BOSSES[drill[1]] : undefined;
  const level = /^(.*) L\d+$/.exec(beat);
  const id = level?.[1].toLowerCase().replace(/ /g, "_");
  const label = beat.replace(/^DRILLED · (.+)$/, /* copy:label */ "$1 tactic");
  const badge = unitLabel(boss ?? "", label, { px: 32,
    art: boss ? undefined : id ? packageIcon(id) : icon("unlocks", "✦"),
    className: `beat-plaque${drill ? " drill" : ""}` });
  badge.dataset.training = beat;
  if (!open) return badge;
  const b: HTMLButtonElement = h("button", { class: "training-open", onclick: () => open(b, beat) }, badge);
  return b;
}
const trainingBadges = (earned: string[], open?: OpenTraining): HTMLElement => h("div", { class: "beats" }, ...earned.map((b) => trainingBadge(b, open)));
export function trainingBlock(beats: string[] | undefined, open?: OpenTraining): HTMLElement | null {
  const earned = trainingBeats(beats);
  return earned.length ? h("section", { class: "run-training" },
    h("b", { class: "row-label" }, /* copy:label */ "Training"), trainingBadges(earned, open)) : null;
}
/** Multihero progress uses the report's persistent slot, never today's selected heir. */
export function reportTrainingBlock(r: Pick<ReturnReport, "bloodlines" | "packages">, open?: OpenTraining): HTMLElement | null {
  if (!r.bloodlines?.some((s) => s.packages !== undefined)) return trainingBlock(r.packages, open);
  const earned = [...r.bloodlines].sort((a,b) => a.id-b.id).map((s) => ({ ...s, beats: trainingBeats(s.packages) })).filter((s) => s.beats.length);
  return earned.length ? h("section", { class: "run-training" },
    h("b", { class: "row-label" }, /* copy:label */ "Training"),
    ...earned.map((s) => h("div", { class: "bloodline-training", "data-bloodline": s.id },
      h("small", { class: "training-lineage" }, s.name), trainingBadges(s.beats, open)))) : null;
}
