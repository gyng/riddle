import { classIcon } from './class-icons';
import type { ReturnReport } from "../engine/types";
import { h } from "./dom";
import { detailHost } from "./tips";

type Xp = ReturnReport["xp"];
/** Add earned deltas by their reported class, preserving unknown old data. */
export function mergeClassXp(a?: Xp[], b?: Xp[]): Xp[] | undefined {
  if (a === undefined && b === undefined) return undefined;
  const by = new Map<string, Xp>();
  for (const x of [...(a ?? []), ...(b ?? [])]) {
    const prev = by.get(x.class);
    by.set(x.class, { class: x.class, gained: (prev?.gained ?? 0) + x.gained, level_ups: (prev?.level_ups ?? 0) + x.level_ups });
  }
  return [...by.values()].sort((x, y) => x.class.localeCompare(y.class));
}

/** Earned class progress belongs to the report, never today's selected hero. */
export function classXpBlock(r: Pick<ReturnReport, "xp" | "bloodlines">): HTMLElement | null {
  const slots = r.bloodlines;
  const gains = slots?.some((s) => s.xp !== undefined)
    ? [...slots].sort((a, b) => a.id - b.id).flatMap((s) => (s.xp ?? []).map((xp) => ({ xp, owner: s.name, id: s.id })))
    : r.xp ? [{ xp: r.xp, owner: undefined, id: undefined }] : [];
  const earned = gains.filter(({ xp }) => xp.gained > 0 || xp.level_ups > 0);
  if (!earned.length) return null;
  return h("section", { class: "report-class-xp" }, h("b", { class: "row-label" }, /* copy:label */ "Class XP"),
    ...earned.map(({ xp, owner, id }) => h("div", { class: "class-xp-line num", "data-class": xp.class, ...(id === undefined ? {} : { "data-bloodline": String(id) }) },
      owner ? h("small", { class: "class-xp-owner" }, owner) : "",
      classIcon(xp.class), h("span", null, xp.class.charAt(0).toUpperCase() + xp.class.slice(1).replace(/_/g, " ")),
      xp.gained > 0 ? h("b", { class: "class-xp-gain" }, `+${xp.gained} XP`) : "",
      xp.level_ups > 0 ? h("b", { class: "class-xp-level" }, xp.level_ups === 1 ? /* copy:label */ "+1 level" : /* copy:label */ `+${xp.level_ups} levels`) : "")));
}

const className = (cls: string): string => cls.charAt(0).toUpperCase() + cls.slice(1).replace(/_/g, " ");

/** Owner 2026-10-10 ("class level up matters"): a level crossed while away is a highlight — `Fighter L4` (the level reached; the selected
 *  bloodline's first), its gain on hover. `level` reads the level reached (the slot's own, else the selected hero's classes). */
export function levelUpLine(r: Pick<ReturnReport, "xp" | "bloodlines">, level: (cls: string, bloodline?: number) => number | undefined, selected = 1): HTMLElement | null {
  const slots = r.bloodlines;
  const gains = slots?.some((s) => s.xp !== undefined)
    ? [...slots].sort((a, b) => (a.id === selected ? -1 : b.id === selected ? 1 : a.id - b.id)).flatMap((s) => (s.xp ?? []).map((xp) => ({ xp, id: s.id as number | undefined, owner: s.name as string | undefined })))
    : r.xp ? [{ xp: r.xp, id: undefined, owner: undefined }] : [];
  const ups = gains.filter(({ xp }) => xp.level_ups > 0);
  if (!ups.length) return null;
  const top = ups[0];
  const lvl = level(top.xp.class, top.id);
  const el = h("div", { class: "report-levelup report-hl num", "data-hl": "level", "data-class": top.xp.class, "data-ups": String(top.xp.level_ups) },
    classIcon(top.xp.class), " ", h("b", null, className(top.xp.class)), " ",
    h("b", { class: "levelup-l up" }, lvl ? `L${lvl}` : /* copy:callout */ `+${top.xp.level_ups} level${top.xp.level_ups > 1 ? "s" : ""}`),
    ups.length > 1 ? h("small", { class: "dim" }, ` +${ups.length - 1}`) : "");
  return detailHost(el, () => [h("div", { class: "kw-tip-head" }, h("b", null, /* copy:label */ "Level up")),
    ...ups.map(({ xp, id, owner }) => { const l = level(xp.class, id); return h("div", { class: "num" }, owner ? `${owner} · ` : "", className(xp.class),
      l ? ` · L${l}` : "", xp.level_ups === 1 ? /* copy:callout */ " · +1 level" : /* copy:callout */ ` · +${xp.level_ups} levels`, xp.gained > 0 ? ` · +${xp.gained} XP` : ""); })]);
}
