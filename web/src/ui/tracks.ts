// Cut 30 §4 — the tracks panel (docs/CUT30.md): one panel, four rows — character · items · scale · town — each its stage (an icon and
// ≤ 2 words), a progress bar when the next trigger is numeric, and `next · <stage> · <trigger>`. Opened from the portrait-mini in the
// top bar. The report leads with what grew on each track (`ReturnReport.grew`, `grewBlock`). No tutorial text: the stages are the copy.
import "../cut30.css";
import type { App } from "../app";
import type { GrewLine, Lineage, ReturnReport, Track } from "../engine/types";
import { h } from "./dom";
import { openSheet } from "./sheet";
import { icon, portraitSrc } from "./skin";
import { kw, kwText } from "./tips";

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

/** The lineage has tracks on the wire (a Cut 30 core). */
export const hasTracks = (L: Pick<Lineage, "tracks"> | undefined): boolean => !!L?.tracks?.length;
/** Stages reached in all (the portrait-mini glints when it grows). */
export const stagesReached = (L: Pick<Lineage, "tracks">): number => (L.tracks ?? []).reduce((a, t) => a + t.stages, 0);
/** The mini opens the panel once any track has grown past its first stage (a fresh camp keeps its few surfaces). */
export const tracksShown = (L: Pick<Lineage, "tracks"> | undefined): boolean => hasTracks(L) && (L!.tracks ?? []).some((t) => t.stages > 1);

const KEY = "riddle.tracks.seen";
/** Whether the stages grew since the last look at this lineage (per seed; the glint plays once, then the count is stored). */
export function tracksGrew(L: Pick<Lineage, "tracks" | "seed">): boolean {
  const n = stagesReached(L);
  try {
    const s = JSON.parse(localStorage.getItem(KEY) ?? "null") as { seed: number; n: number } | null;
    if (!s || s.seed !== L.seed) { localStorage.setItem(KEY, JSON.stringify({ seed: L.seed, n })); return false; }
    if (n !== s.n) { localStorage.setItem(KEY, JSON.stringify({ seed: L.seed, n })); return n > s.n; }
  } catch { /* a per-viewer convenience */ }
  return false;
}

/** One track's row: icon, the stage, the bar (a numeric trigger), `next · <stage> · <trigger>` (none once its stages are done). */
export function trackRow(t: Track, hero?: string): HTMLElement {
  return h("div", { class: "track-row", "data-track": t.id },
    h("span", { class: "track-ico" }, trackIcon(t.id, hero)),
    h("div", { class: "track-main" },
      h("div", { class: "track-head" }, h("small", { class: "track-name dim" }, trackName(t.id)), h("b", { class: "track-stage" }, t.stage)),
      t.progress !== undefined && t.progress !== null && t.next ? h("span", { class: "track-bar", "aria-hidden": "true" }, h("span", { class: "fill", style: `width:${Math.round(Math.max(0, Math.min(1, t.progress)) * 100)}%` })) : "",
      t.next ? h("small", { class: "track-next num" }, /* copy:callout */ `next · ${t.next}${t.trigger ? ` · ${t.trigger}` : ""}`) : h("small", { class: "track-next done dim" }, "✓")));
}

/** Opens the tracks panel (a sheet anchored to the portrait-mini). */
export function openTracks(app: App, anchor?: HTMLElement | null): void {
  openSheet(() => {
    const ts = app.lineage.tracks ?? [];
    const order = (t: Track): number => { const i = (TRACK_IDS as readonly string[]).indexOf(t.id); return i < 0 ? 9 : i; };
    return h("div", { class: "sheet-body tracks-panel" }, h("div", { class: "label row-label" }, kw("track", /* copy:label */ "tracks")),
      ...[...ts].sort((a, b) => order(a) - order(b)).map((t) => trackRow(t, heroFace(app.lineage))));
  }, { anchor });
}

/** The report's lead (§4): what grew on each track over the absence — one line per track that grew, its icon and its things
 *  (`hero · L7 · package level`, `items · +$2400`). Null when nothing grew (an old core: no `grew`). */
export function grewBlock(r: Pick<ReturnReport, "grew" | "packages">, hero?: string): HTMLElement | null {
  const g: GrewLine[] = r.grew ?? [];
  const beats = r.packages ?? [];
  if (!g.length && !beats.length) return null;
  const by = new Map<string, string[]>();
  for (const x of g) by.set(x.track, [...(by.get(x.track) ?? []), x.what]);
  const ids = [...TRACK_IDS.filter((t) => by.has(t)), ...[...by.keys()].filter((t) => !(TRACK_IDS as readonly string[]).includes(t))];
  return h("div", { class: "grew" },
    ...ids.map((id, i) => h("div", { class: "grew-line reveal", "data-track": id, style: `animation-delay:${0.12 * i}s` }, h("span", { class: "track-ico" }, trackIcon(id, hero)),
      h("small", { class: "track-name dim" }, trackName(id)), h("span", { class: "grew-what num" }, ...kwText((by.get(id) ?? []).join(" · "))))),
    beats.length ? h("div", { class: "beats" }, ...beats.slice(0, 5).map((b, i) => h("span", { class: `beat-plaque reveal${/^QUEST DONE/.test(b) ? " quest" : /^DRILLED/.test(b) ? " drill" : ""}`, style: `animation-delay:${0.15 * (i + ids.length)}s` }, b)),
      beats.length > 5 ? h("small", { class: "beat-more dim" }, /* copy:callout */ `+${beats.length - 5} more`) : "") : "");
}
