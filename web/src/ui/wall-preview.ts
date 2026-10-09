// Cut 118 round 2 (new): a roster preview of the next wall on the camp's chart — the first band boss not yet slain, his floor, and his
// counter as the lineage knows it (`counter · fire`, or `counter ?` with what would teach it). Its face carries the enemy tip (the
// facts, the ledger). The wire has no per-wall affix or guard yet: they are the core fields this wants (`BossWall.affix`, `.guard`,
// each with its counter) — drawn here the moment they arrive.
import type { BossWall, Lineage } from "../engine/types";
import { h } from "./dom";
import { unitLabel } from "./unit-icon";
import { enemyHost } from "./enemy-tips";

type WallMore = BossWall & { affix?: string; guard?: string; affix_counter?: string; guard_counter?: string };

/** The next wall: the shallowest band boss not slain (the core's `walls`, the Warlord to the first past the best). */
export function nextWall(L: Pick<Lineage, "walls">): WallMore | undefined {
  return [...(L.walls ?? [])].filter((w) => !w.slain).sort((a, b) => a.depth - b.depth)[0] as WallMore | undefined;
}

export function wallPreview(L: Lineage): HTMLElement | null {
  const w = nextWall(L);
  if (!w) return null;
  const counter = w.known && w.counter ? w.counter : w.learn ? `? · ${w.learn}` : "?";
  const more = [w.affix ? /* copy:callout */ `affix ${w.affix}${w.affix_counter ? ` → ${w.affix_counter}` : ""}` : "",
    w.guard ? /* copy:callout */ `guard ${w.guard}${w.guard_counter ? ` → ${w.guard_counter}` : ""}` : ""].filter(Boolean);
  const face = enemyHost(unitLabel(w.boss, h("span", { class: "wall-who" }, h("b", null, w.title), " ", h("span", { class: "num" }, `D${w.depth}`)), { px: 30 }), w.boss, L);
  return h("div", { class: "wall-preview num", "data-boss": w.boss, "data-depth": w.depth },
    h("span", { class: "label wall-label" }, /* copy:label */ "next wall"), face,
    h("small", { class: "wall-counter", "data-known": w.known && w.counter ? "1" : "0" }, /* copy:label */ "counter", ` · ${counter}`),
    ...more.map((m) => h("small", { class: "wall-more dim" }, m)));
}
