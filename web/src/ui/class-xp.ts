import type { ReturnReport } from "../engine/types";
import { h } from "./dom";

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
      h("span", null, xp.class.charAt(0).toUpperCase() + xp.class.slice(1).replace(/_/g, " ")),
      xp.gained > 0 ? h("b", { class: "class-xp-gain" }, `+${xp.gained} XP`) : "",
      xp.level_ups > 0 ? h("b", { class: "class-xp-level" }, xp.level_ups === 1 ? /* copy:label */ "+1 level" : /* copy:label */ `+${xp.level_ups} levels`) : "")));
}
